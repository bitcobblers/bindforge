import { type Dirent, readFileSync, readdirSync } from 'node:fs';
import { basename, join } from 'node:path';

import { allow, deny, frontmatterOf } from './gates.js';
import type { GateResult } from './gates.js';

// ---------------------------------------------------------------------------
// The decision-unit ledger: readers and the join gate (ADR-021, design 021).
//
// A product ADR's `## Decision` is a ledger of typed, numbered units, each a
// bounded choice with a `→` pointer to the normative text that carries it
// (021-C1), and a normative line's trailing code span is the handle naming the
// unit that governs it (021-C2). ADR-021 defers the gate's error taxonomy to
// "021-R2's run": this module is that taxonomy. The join is checked in both
// directions over the two trees where normative text lives — `docs/adrs/` and
// `schemas/` — and the same function runs at entry, beside `sourceAdmissible`,
// and at finalize, before the ADR status flip: a run cannot complete while one
// of its own units is unhandled, which is 013's seven-units failure mode closed.
//
// Two rules bound what this module may do. It is a reader: it reads the tree
// and never writes it, so a check can never be satisfied by reformatting. And
// it parses no path it did not list itself — every path here comes out of its
// own directory walk, joined through `node:path` and printed repo-root-relative
// with POSIX separators, which is the spelling the gate log and the tests pin.
//
// `docs/designs/**` is outside the corpus on purpose: a design document is a
// record, not a contract, and it quotes unit ids freely (¶5).
// ---------------------------------------------------------------------------

/** What one finding names: where, which rule, and a one-line reason. */
export type LedgerFindingCode =
  | 'ledger-empty'
  | 'unit-malformed'
  | 'unit-duplicate'
  | 'unit-kind-mismatch'
  | 'unit-misplaced'
  | 'pointer-missing'
  | 'owner-missing'
  | 'owner-unknown'
  | 'handle-dangling'
  | 'unit-unhandled'
  | 'note-missing'
  | 'depends-order';

export interface LedgerFinding {
  /** Repo-root-relative, POSIX separators — the spelling the gate log prints. */
  file: string;
  /** 1-based, in the file's own line numbering. */
  line: number;
  code: LedgerFindingCode;
  detail: string;
}

/** One unit id carried by a handle (¶4). */
export interface Handle {
  id: string;
  line: number;
}

/** One parsed unit, in the ledger file that defines it (¶2). */
export interface LedgerUnit {
  id: string;
  /** The kind letter the id carries, which the section heading must match. */
  kind: string;
  line: number;
  /** A reservation whose owner is the explicit `unassigned` marker (021-L2). */
  unassigned: boolean;
}

export interface LedgerFile {
  nnn: string;
  units: LedgerUnit[];
  findings: LedgerFinding[];
}

/**
 * How many findings one denial lists — the sibling of `MAX_GATE_LEGS`: a red
 * corpus can outgrow what a gate log explains, so the reason keeps the leading
 * findings and states the count it dropped.
 */
export const MAX_LEDGER_FINDINGS = 12;

/**
 * One unit id, `NNN-Kn`, anchored by look-behind and a trailing guard so it
 * cannot match inside a longer token: the ADR's grep form `[0-9]{3}-[CLBRS]
 * [0-9]{1,2}` also matches the `013-C1` inside `v013-C1` or `013-C123` (¶2).
 */
const ID_SOURCE = '(?<![\\dA-Za-z-])\\d{3}-[CLBRS]\\d{1,2}(?![\\dA-Za-z])';
const UNIT_ID_RE = new RegExp(`^${ID_SOURCE}$`);
/** A handle's span content: unit ids separated by single spaces, nothing else. */
const HANDLE_SPAN_RE = new RegExp(`^${ID_SOURCE}(?: ${ID_SOURCE})*$`);
/**
 * The trailing code span of a line: followed only by spaces and at most one
 * table-cell `|`. The regex is end-anchored and a span's content holds no
 * backtick, so the match is the LAST span on the line — a mid-sentence span
 * governs nothing (¶4).
 */
const TRAILING_SPAN_RE = /`([^`\r\n]*)`[ \t]*\|?[ \t]*$/;
/** A fence delimiter's trimmed start; fenced content is literal, not prose. */
const FENCE_RE = /^(```|~~~)/;
/** A unit bullet: a `- ` item at column zero. Ordered items are prose (¶2). */
const BULLET_RE = /^- /;
const BOLD_ID_RE = /^-[ \t]+\*\*([^*]*)\*\*/;

/** Kind letter → the `###` heading that carries it (021-C1). */
const KINDS: readonly { letter: string; heading: string }[] = [
  { letter: 'C', heading: 'Contracts' },
  { letter: 'L', heading: 'Laws' },
  { letter: 'B', heading: 'Budgets' },
  { letter: 'R', heading: 'Reservations' },
  { letter: 'S', heading: 'Seeds' },
];

/** The declaration note every handle-bearing corpus file owes (¶5). */
const NOTE_MARKER = '**Unit handles.**';
/** The three owner forms 021-L5 accepts; backticks are stripped first. */
const OWNER_ADR_RE = /^ADR-\d{3}(?![\dA-Za-z])/;
const OWNER_UNASSIGNED_RE = /^unassigned(?![\dA-Za-z])/;
const OWNER_RUN_RE = /^this ADR's run(?![\dA-Za-z])/;

/** The document's lines: BOM dropped, CRLF folded (A13). */
function linesOf(content: string): string[] {
  return content.replace(/^\uFEFF/, '').split(/\r?\n/);
}

/** How many source lines the frontmatter block covers, 0 when it has none. */
function frontmatterSpan(content: string): number {
  const fm = frontmatterOf(content);
  if (fm === undefined) {
    return 0;
  }
  if (fm.front === '') {
    return 2;
  }
  return fm.front.split(/\r?\n/).length + 2;
}

/**
 * Every handle in a document (021-C2). Frontmatter is not prose and a fenced
 * block is literal to a markdown reader, so neither carries a handle; both are
 * skipped rather than special-cased inside the span rule.
 */
export function extractHandles(content: string): Handle[] {
  const lines = linesOf(content);
  const handles: Handle[] = [];
  let fenced = false;
  for (let i = frontmatterSpan(content); i < lines.length; i += 1) {
    const line = lines[i] ?? '';
    if (FENCE_RE.test(line.trim())) {
      fenced = !fenced;
      continue;
    }
    if (fenced) {
      continue;
    }
    const span = TRAILING_SPAN_RE.exec(line);
    const text = span?.[1];
    if (text === undefined || !HANDLE_SPAN_RE.test(text)) {
      continue;
    }
    for (const id of text.split(' ')) {
      handles.push({ id, line: i + 1 });
    }
  }
  return handles;
}

/**
 * The ledger of one `docs/adrs/NNN-*.md`, or `undefined` when the file is not
 * a ledger at all: the frontmatter `ledger: yes` key is the only trigger (¶6),
 * and a name that carries no number — the template — never is one. Absent the
 * key the file is exempt from the unit, pointer, owner and kind checks; it is
 * not exempt from the handle and note rule or from `depends-order`.
 */
export function parseLedger(content: string, nnn: string, file = ''): LedgerFile | undefined {
  if (!/^\d{3}$/.test(nnn)) {
    return undefined;
  }
  const fm = frontmatterOf(content);
  if (fm === undefined || !/^ledger:[ \t]*yes[ \t]*$/m.test(fm.front)) {
    return undefined;
  }

  const lines = linesOf(content);
  const findings: LedgerFinding[] = [];
  const units: LedgerUnit[] = [];
  const say = (line: number, code: LedgerFindingCode, detail: string): void => {
    findings.push({ file, line, code, detail });
  };

  // The one section a ledger lives in: `## Decision` up to the next `## `.
  let from = -1;
  let to = lines.length;
  for (let i = frontmatterSpan(content); i < lines.length; i += 1) {
    const line = lines[i] ?? '';
    if (from < 0) {
      if (/^## Decision[ \t]*$/.test(line)) {
        from = i;
      }
    } else if (line.startsWith('## ')) {
      to = i;
      break;
    }
  }
  if (from < 0) {
    say(1, 'ledger-empty', 'ledger: yes with no "## Decision" section');
    return { nnn, units, findings };
  }

  let kind: string | undefined;
  let kindHeading = '';
  let kinds = 0;
  const seen = new Map<string, number>();
  const lastNumber = new Map<string, number>();

  for (let i = from + 1; i < to; i += 1) {
    const line = lines[i] ?? '';
    const heading = /^###[ \t]+(.+?)[ \t]*$/.exec(line);
    if (heading !== null) {
      const title = heading[1] ?? '';
      kind = KINDS.find((entry) => entry.heading === title)?.letter;
      kindHeading = title;
      if (kind !== undefined) {
        kinds += 1;
      }
      continue;
    }
    if (!BULLET_RE.test(line)) {
      continue; // preamble prose, an ordered item, or a blank line (¶2)
    }

    // A unit's text is its bullet line plus its indented continuation lines:
    // the shipped ledgers wrap, so the pointer and the owner may sit anywhere
    // inside the bullet rather than only on its first line.
    const parts = [line.slice(2).trim()];
    let last = i;
    while (last + 1 < to) {
      const tail = lines[last + 1] ?? '';
      if (tail.trim() === '' || tail.startsWith('#') || !/^[ \t]/.test(tail)) {
        break;
      }
      parts.push(tail.trim());
      last += 1;
    }
    const text = parts.join(' ');

    const bold = BOLD_ID_RE.exec(line);
    if (bold === null) {
      say(i + 1, 'unit-malformed', 'a bullet in a kind section opens with no bold unit id');
      continue;
    }
    const id = bold[1] ?? '';
    if (!UNIT_ID_RE.test(id)) {
      say(i + 1, 'unit-malformed', `"${id}" is not an anchored unit id (NNN-Kn)`);
      continue;
    }
    if (!id.startsWith(`${nnn}-`)) {
      say(i + 1, 'unit-malformed', `${id} carries a number other than ${nnn}`);
      continue;
    }

    const letter = id.slice(4, 5);
    if (kind === undefined) {
      say(i + 1, 'unit-misplaced', `${id} sits outside every kind section`);
    } else if (kind !== letter) {
      say(i + 1, 'unit-kind-mismatch', `${id} sits under ### ${kindHeading}`);
    }
    const first = seen.get(id);
    if (first !== undefined) {
      say(i + 1, 'unit-duplicate', `${id} is already defined on line ${String(first)}`);
      continue;
    }
    const number = Number(id.slice(5));
    const previous = lastNumber.get(letter);
    if (previous !== undefined && number <= previous) {
      say(i + 1, 'unit-duplicate', `${id} does not follow ${letter}${String(previous)}`);
    }
    seen.set(id, i + 1);
    lastNumber.set(letter, number);

    // The pointer is the last `→` outside a code span — 021-C1 quotes the
    // glyph itself, and quoting a pointer is not carrying one — and it must be
    // followed by text. Which target it names is not resolved (Deferred).
    const bare = text.replace(/`[^`]*`/g, ' ');
    const arrow = bare.lastIndexOf('→');
    if (arrow < 0 || !/\S/.test(bare.slice(arrow + 1))) {
      say(i + 1, 'pointer-missing', `${id} carries no → to its target`);
    }

    // 021-L5. A named owner is clean whether or not `docs/adrs/NNN-*.md`
    // exists yet: the ledger line is the registry, which is how 013-R1–R4
    // already reserve 017 and 019 (¶2).
    let unassigned = false;
    if (letter === 'R') {
      const owner = /Owner:[ \t]*([^\r\n]*)/.exec(text);
      if (owner === null) {
        say(i + 1, 'owner-missing', `${id} names no owner (021-L5)`);
      } else {
        const named = (owner[1] ?? '').replace(/`/g, '').trim();
        unassigned = OWNER_UNASSIGNED_RE.test(named);
        const owned = unassigned || OWNER_ADR_RE.test(named) || OWNER_RUN_RE.test(named);
        if (!owned) {
          say(i + 1, 'owner-unknown', `${id} names "${named}" as its owner`);
        }
      }
    }
    units.push({ id, kind: letter, line: i + 1, unassigned });
  }

  if (kinds === 0) {
    say(from + 1, 'ledger-empty', 'the Decision section carries no kind section');
  }
  return { nnn, units, findings };
}

/** A registered unit, kept with the corpus file that defines it. */
type RegisteredUnit = LedgerUnit & { file: string };

/** One corpus document, read once for both halves of the join. */
interface CorpusDoc {
  rel: string;
  content: string;
  handles: Handle[];
}

/** The entries of `root`, or none when that tree is not in the checkout. */
function entriesOf(root: string): Dirent[] {
  try {
    return readdirSync(root, { withFileTypes: true });
  } catch {
    return []; // a checkout without one of the two normative trees reads as empty
  }
}

/** The text of `file`, or `undefined` when the reader cannot open it. */
function textOf(file: string): string | undefined {
  try {
    return readFileSync(file, 'utf8');
  } catch {
    return undefined; // the walk listed it, the reader could not open it
  }
}

/** Every `.md` under `root`, as repo-root-relative POSIX paths. */
function collectMarkdown(root: string, relPrefix: string, out: string[]): void {
  for (const entry of entriesOf(root)) {
    if (entry.name.startsWith('.') || entry.name === 'node_modules') {
      continue;
    }
    const rel = `${relPrefix}/${entry.name}`;
    if (entry.isDirectory()) {
      collectMarkdown(join(root, entry.name), rel, out);
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      out.push(rel);
    }
  }
}

/**
 * 021-L3 — `depends-on ⊆ smaller numbers`, judged on every ADR that carries the
 * key, ledger-bearing or not (¶6). A value that is not a number is not an
 * order claim, so it is nothing for this rule to refuse.
 */
function dependsOrder(content: string, rel: string, nnn: string): LedgerFinding[] {
  const fm = frontmatterOf(content);
  if (fm === undefined) {
    return [];
  }
  const index = fm.front.split(/\r?\n/).findIndex((line) => /^depends-on:/.test(line));
  const entry = /^depends-on:[ \t]*\[([^\]]*)\][ \t]*\r?$/m.exec(fm.front);
  if (index < 0 || entry === null) {
    return [];
  }
  const findings: LedgerFinding[] = [];
  for (const text of (entry[1] ?? '').split(',')) {
    const dep = text.trim();
    if (/^\d{3}$/.test(dep) && Number(dep) >= Number(nnn)) {
      findings.push({
        file: rel,
        line: index + 2,
        code: 'depends-order',
        detail: `depends-on ${dep} is not a smaller number than ${nnn}`,
      });
    }
  }
  return findings;
}

/**
 * The join, over the whole corpus (¶7): every ledger-bearing ADR's units and
 * every handle in both normative trees, resolved against one registry that
 * spans all of them — which is what 013's cross-attributions need before the
 * series is backfilled.
 */
export function checkLedgers(repoDir: string): LedgerFinding[] {
  const rels: string[] = [];
  collectMarkdown(join(repoDir, 'docs', 'adrs'), 'docs/adrs', rels);
  collectMarkdown(join(repoDir, 'schemas'), 'schemas', rels);
  rels.sort();

  const findings: LedgerFinding[] = [];
  const docs: CorpusDoc[] = [];
  const registry = new Map<string, RegisteredUnit>();

  for (const rel of rels) {
    const content = textOf(join(repoDir, ...rel.split('/')));
    if (content === undefined) {
      continue; // unreadable: the walk found it, the reader could not open it
    }
    docs.push({ rel, content, handles: extractHandles(content) });
    const nnn = /^(\d{3})-/.exec(basename(rel))?.[1];
    if (nnn === undefined) {
      continue; // the template and the spec tree carry no ledger, only handles
    }
    findings.push(...dependsOrder(content, rel, nnn));
    const ledger = parseLedger(content, nnn, rel);
    if (ledger === undefined) {
      continue;
    }
    findings.push(...ledger.findings);
    for (const unit of ledger.units) {
      if (!registry.has(unit.id)) {
        registry.set(unit.id, { ...unit, file: rel });
      }
    }
  }

  // One direction: every handle resolves to a registered unit, and a file that
  // makes use of the convention states it (¶5).
  const handled = new Set<string>();
  for (const doc of docs) {
    if (doc.handles.length === 0) {
      continue;
    }
    if (!doc.content.includes(NOTE_MARKER)) {
      findings.push({
        file: doc.rel,
        line: doc.handles[0]?.line ?? 1,
        code: 'note-missing',
        detail: 'carries a handle with no **Unit handles.** note',
      });
    }
    for (const handle of doc.handles) {
      if (registry.has(handle.id)) {
        handled.add(handle.id);
      } else {
        findings.push({
          file: doc.rel,
          line: handle.line,
          code: 'handle-dangling',
          detail: `no ledger defines ${handle.id}`,
        });
      }
    }
  }

  // The other direction: every unit carries a handle, except a reservation the
  // ledger itself marks `unassigned` — 021-L2's one exception (013-R5).
  for (const [id, unit] of registry) {
    if (handled.has(id) || unit.unassigned) {
      continue;
    }
    findings.push({
      file: unit.file,
      line: unit.line,
      code: 'unit-unhandled',
      detail: `${id} carries no handle anywhere in the corpus`,
    });
  }

  return findings.sort(byPosition);
}

/** Findings in reading order, so a denial and a test see one stable list. */
function byPosition(a: LedgerFinding, b: LedgerFinding): number {
  if (a.file !== b.file) {
    return a.file < b.file ? -1 : 1;
  }
  if (a.line !== b.line) {
    return a.line - b.line;
  }
  if (a.code !== b.code) {
    return a.code < b.code ? -1 : 1;
  }
  if (a.detail === b.detail) {
    return 0;
  }
  return a.detail < b.detail ? -1 : 1;
}

/**
 * The gate, in the shape every other deterministic gate returns, so the gate
 * log and `pipeline status` print it unchanged. Both hooks name it
 * `ledger-join`, and the reason names each finding's `file:line` and code: a
 * stuck run explains itself from the gate log (¶7).
 */
export function ledgerGate(repoDir: string): GateResult {
  const findings = checkLedgers(repoDir);
  if (findings.length === 0) {
    return allow('the ledger join holds: every handle resolves, every unit is handled');
  }
  const kept = findings.slice(0, MAX_LEDGER_FINDINGS);
  const lines = kept.map((f) => `${f.file}:${String(f.line)} ${f.code} ${f.detail}`);
  const head = `ledger join: ${String(findings.length)} finding(s) in the corpus`;
  if (kept.length === findings.length) {
    return deny([head, ...lines].join('\n'));
  }
  return deny([head, ...lines, `+${String(findings.length - kept.length)} more`].join('\n'));
}
