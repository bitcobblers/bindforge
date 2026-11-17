import { type ChildProcess, spawn } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { basename, join } from 'node:path';

import { flowDef } from './flows.js';
import type { FlowId } from './flows.js';
import type { PipelineConfig } from './stages.js';
import { STATE_FILE, namesReservedSegment } from './types.js';
import type { Finding, GateLeg, PipelineState, VerdictReply } from './types.js';

// ---------------------------------------------------------------------------
// Gates are code: deterministic guards, round-end verdict aggregation, and
// schema/shape checks. Every gate returns a reason so the log can explain a
// stuck run without re-reading review files.
// ---------------------------------------------------------------------------

export interface GateResult {
  proceed: boolean;
  reason: string;
}

export const allow = (reason: string): GateResult => ({ proceed: true, reason });
export const deny = (reason: string): GateResult => ({ proceed: false, reason });

// --- source gate ------------------------------------------------------------

export interface ParsedSource {
  kind: 'adr' | 'brief' | 'task';
  status: string;
  hasContext: boolean;
  hasDecision: boolean;
  /**
   * ADR-010: the frontmatter `flow:` key, as spelled. An absent key is `full`;
   * anything else — `Light`, `light `, `sideways`, an empty value — is carried
   * verbatim to the flow registry's lookup, which is the only gate, so a
   * misspelling surfaces as a usage error instead of a silent `full` run.
   */
  flow: FlowId;
  /**
   * ADR-023 (023-C1): the frontmatter `doc_type:` key, first token as spelled
   * (`adr`, `task`, …). Absent is `undefined`, never a guess — the source
   * gate is the only judge, and a task document missing the key is its
   * refusal, not a path fallback.
   */
  docType?: string;
  /**
   * ADR-023 (023-C4): the task's `type:` and `functional_area:` keys, first
   * token as spelled. Absent is `undefined`; the gate's presence check is the
   * only validation (023 leaves the value sets to a schema).
   */
  type?: string;
  functionalArea?: string;
  /**
   * ADR-023 (023-L4): the `depends_on:` list, entry-verbatim (whitespace
   * trimmed, quotes stripped, order kept). Absent or empty is `[]`; the
   * dependency gate is the only judge of the entries.
   */
  dependsOn: string[];
  /**
   * The document with its frontmatter block removed — the part a brief *is*.
   * The brief gate judges this, because with ADR-010 a `flow:` key is a legal
   * first line: a document that is only frontmatter carries no brief.
   */
  body: string;
}

/**
 * The frontmatter block and the body it does not cover — the harness's one
 * frontmatter regex, exported so the ledger readers share it instead of
 * growing a second one (ADR-021 ¶1): one grammar, one CRLF posture (A13). A
 * leading byte-order mark is dropped, because it is not part of the text.
 */
export function frontmatterOf(content: string): { front: string; body: string } | undefined {
  // A byte-order mark is not text, and it would otherwise hide the `---`.
  const text = content.charCodeAt(0) === 0xfeff ? content.slice(1) : content;
  const block = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text);
  if (block === null) {
    return undefined;
  }
  return { front: block[1] ?? '', body: text.slice(block[0].length) };
}

/**
 * ADR-023 (023-L2): a task source file name — `T-NNN-slug.md` — on the
 * run-identity grammar (NNN_RE / SLUG_RE in types.ts): the generator and the
 * gate must agree on the spelling or a task is unrunnable by its own name.
 */
export const TASK_SOURCE_NAME_RE = /^T-\d{3}-[a-z0-9][a-z0-9-]*\.md$/;
/** ADR-023 (023-L4): a `depends_on` entry — the task name without the `.md`. */
export const TASK_NAME_RE = /^T-\d{3}-[a-z0-9][a-z0-9-]*$/;

/**
 * Minimal frontmatter + section parsing (no YAML dependency).
 *
 * `flow` is deliberately not normalised: the registry lookup decides, so the
 * comparison stays case-sensitive and whitespace-sensitive by construction.
 * The ADR-023 keys follow the same posture: first token captured verbatim,
 * the gates the only judges.
 */
export function parseSource(content: string, kind: 'adr' | 'brief' | 'task'): ParsedSource {
  const fm = frontmatterOf(content);
  const front = fm?.front ?? '';
  const status = /status:\s*(\w+)/.exec(front)?.[1] ?? 'none';
  const body = fm === undefined ? content : fm.body;
  const flow = /^flow:[ \t]*([^\r\n]*)/m.exec(front)?.[1];
  const docType = /^doc_type:[ \t]*([\w-]+)/m.exec(front)?.[1];
  const type = /^type:[ \t]*([\w-]+)/m.exec(front)?.[1];
  const functionalArea = /^functional_area:[ \t]*([\w-]+)/m.exec(front)?.[1];
  const dependsOnLine = /^depends_on:[ \t]*\[([^\]]*)\]/m.exec(front);
  const dependsOn =
    dependsOnLine === null
      ? []
      : (dependsOnLine[1] ?? '')
          .split(',')
          .map((entry) => entry.trim().replace(/^['"]|['"]$/g, ''))
          .filter((entry) => entry.length > 0);
  const section = (name: string): boolean => {
    // The end anchor must be end-of-string, not multiline `$`: `$` matches
    // before every newline, so a section whose body starts with a blank line
    // would capture nothing and read as empty.
    const re = new RegExp(`^## ${name}[ \\t]*\r?\n([\\s\\S]*?)(?=^## |$(?![\\r\\n]))`, 'm');
    const m = re.exec(body);
    return m !== null && m[1]!.trim().length > 0;
  };
  return {
    kind,
    status,
    hasContext: section('Context'),
    hasDecision: section('Decision'),
    flow: flow ?? 'full',
    docType,
    type,
    functionalArea,
    dependsOn,
    body,
  };
}

/**
 * Fail-fast gate over the source document, evaluated before any inference is
 * spent. ADRs must parse, be `proposed`, and carry non-empty Context and
 * Decision sections; briefs must be non-empty markdown — the standing rules,
 * in both flows: the frontmatter key selects the flow, never the document's
 * shape (Compatibility matrix).
 *
 * The `flow` argument is validated before anything is read: this gate runs on
 * every entry path, so an unreadable flow is a refusal here rather than a
 * silently-full run.
 *
 * `parsed` is the caller's own parse of this document, so an entry path that
 * already read the file reads it once (Selecting the flow: "one frontmatter
 * read per entry, so the value that cleared the pre-lock check is the value
 * stamped and printed"). Omitted, this gate does its own single read.
 */
export async function sourceAdmissible(
  sourcePath: string,
  kind: 'adr' | 'brief' | 'task',
  flow: FlowId = 'full',
  parsed?: ParsedSource,
): Promise<GateResult> {
  // Validation only: the registry lookup is the whole gate.
  flowDef(flow);
  let doc = parsed;
  if (doc === undefined) {
    try {
      doc = parseSource(await readFile(sourcePath, 'utf8'), kind);
    } catch {
      return deny(`source document not readable: ${sourcePath}`);
    }
  }
  // ADR-023 (023-C1): a present `doc_type` must agree with the path's kind.
  // The path stays authoritative; the key is a redundant declaration, and a
  // disagreement is a usage error — fail closed, never silently re-routed.
  // A task document must carry the key at all (023-C4).
  if (doc.docType !== undefined && doc.docType !== doc.kind) {
    return deny(
      `doc_type "${doc.docType}" disagrees with the source path (kind: ${doc.kind}) — ` +
        'the path is authoritative: move the document or fix the key',
    );
  }
  if (doc.kind === 'task') {
    // ADR-023 (023-L2): the name is part of the contract — the series number
    // and the slug both derive from it, so an unnumbered or misnamed file
    // cannot enter.
    if (!TASK_SOURCE_NAME_RE.test(basename(sourcePath))) {
      return deny(`task source must be named T-NNN-slug.md (got ${basename(sourcePath)})`);
    }
    if (doc.docType !== 'task') {
      return deny('task source has no doc_type: task frontmatter key');
    }
    if (doc.status !== 'pending') {
      return deny(`task status is "${doc.status}", expected "pending"`);
    }
    if (doc.type === undefined) {
      return deny('task has no non-empty "type:" frontmatter key');
    }
    if (doc.functionalArea === undefined) {
      return deny('task has no non-empty "functional_area:" frontmatter key');
    }
    // The gate judges the body, not its frontmatter — the same rule the brief
    // gate has always run: a task that is only frontmatter carries no task.
    return doc.body.trim().length > 0
      ? allow('task is pending and complete')
      : deny('task body is empty');
  }
  if (doc.kind === 'brief') {
    // The gate judges the brief, not its frontmatter: the raw document still
    // carries the `flow:` block, so a brief that is only frontmatter is empty.
    return doc.body.trim().length > 0 ? allow('brief is non-empty') : deny('brief is empty');
  }
  if (doc.status !== 'proposed') {
    return deny(`ADR status is "${doc.status}", expected "proposed"`);
  }
  if (!doc.hasContext) {
    return deny('ADR has no non-empty "## Context" section');
  }
  if (!doc.hasDecision) {
    return deny('ADR has no non-empty "## Decision" section');
  }
  return allow('ADR is proposed and complete');
}

/**
 * The task design directory that checkpoints a given task source — the same
 * authority `nnnOfExistingRun` uses for light re-entry (ADR-010 A17): the
 * checkpoint's `source.path` names the run, so a `--slug`-pinned run still
 * resolves. `undefined` when no task design directory checkpoints it.
 */
function taskRunStatus(repoDir: string, dep: string): string | undefined {
  const sourceRel = join('docs', 'tasks', `${dep}.md`);
  const root = join(repoDir, 'docs', 'designs', 'tasks');
  let entries: string[];
  try {
    entries = readdirSync(root);
  } catch {
    return undefined; // no task design tree at all: the dependency has no run
  }
  for (const entry of entries) {
    if (!/^\d{3}-/.test(entry)) {
      continue;
    }
    try {
      const raw = JSON.parse(readFileSync(join(root, entry, STATE_FILE), 'utf8')) as {
        source?: { kind?: unknown; path?: unknown };
        status?: unknown;
      };
      if (raw.source?.kind === 'task' && raw.source?.path === sourceRel) {
        return typeof raw.status === 'string' ? raw.status : undefined;
      }
    } catch {
      // No readable checkpoint: a design directory left behind is not a run.
    }
  }
  return undefined;
}

/**
 * ADR-023 (023-L4): the task dependency gate — a deterministic entry check,
 * run beside `sourceAdmissible` before any inference is spent. Every
 * `depends_on` entry must name an EXISTING task source whose run is DONE:
 * its checkpoint exists and carries `status: done`. A missing checkpoint is
 * "never started", which is not done, and is refused the same way.
 */
export function taskDependenciesGate(repoDir: string, deps: readonly string[]): GateResult {
  if (deps.length === 0) {
    return allow('no dependencies declared');
  }
  for (const dep of deps) {
    if (!TASK_NAME_RE.test(dep)) {
      return deny(`depends_on entry "${dep}" is not a T-NNN-slug task name`);
    }
    const sourceRel = join('docs', 'tasks', `${dep}.md`);
    if (!existsSync(join(repoDir, 'docs', 'tasks', `${dep}.md`))) {
      return deny(`depends_on names ${dep}, but ${sourceRel} does not exist`);
    }
    const status = taskRunStatus(repoDir, dep);
    if (status === undefined) {
      return deny(
        `depends_on names ${dep}, which has no run — no task checkpoint names ${sourceRel}`,
      );
    }
    if (status !== 'done') {
      return deny(`depends_on names ${dep}, whose run is not done (status: ${status})`);
    }
  }
  return allow(`${String(deps.length)} dependency(ies) satisfied`);
}

/**
 * ADR-010 §4 — the light flow's plan line ceiling, a deterministic stop at the
 * same stage-ok/pre-commit point the size meter runs at: the runner hands it
 * the meter's own line count for the run's `02-proposal.md`, because a stage
 * cannot report its own size. The cap is config, so a registry change moves it
 * and no code does; its absence is the full flow's policy of measuring and
 * warning without gating, which is why `planMaxLines` is not in
 * `DEFAULT_CONFIG`.
 */
export function planOutgrewFlow(lines: number, cfg: PipelineConfig): GateResult {
  const cap = cfg.planMaxLines;
  if (cap === undefined) {
    return allow('this flow sets no plan line cap');
  }
  if (lines <= cap) {
    return allow(`plan is ${String(lines)} lines, within the cap of ${String(cap)}`);
  }
  return deny(`plan outgrew the light flow (${String(lines)} lines, cap ${String(cap)})`);
}

/**
 * ADR-016 Decision 4 — the full flow's document ceiling, the sibling of
 * `planOutgrewFlow` and deliberately not its twin. The plan stop is terminal
 * because the light plan is written once and the stop must outlive the
 * escalation it minted; a full-flow document over length is shrinkable by the
 * next revise, so this gate's denial is a normal stage failure that the
 * stage's own retry-then-escalate budget repairs (design 016 §4, Options).
 * The count is the size meter's, taken from the worktree — a stage cannot
 * report its own growth. The cap is registry config (`FLOWS.full.config`);
 * its absence is the unchanged measure-and-warn posture.
 */
export function documentOutgrewFlow(lines: number, cfg: PipelineConfig): GateResult {
  const cap = cfg.docMaxLines;
  if (cap === undefined) {
    return allow('this flow sets no document line cap');
  }
  if (lines <= cap) {
    return allow(`document is ${String(lines)} lines, within the cap of ${String(cap)}`);
  }
  return deny(`document outgrew the flow cap (${String(lines)} lines, cap ${String(cap)})`);
}

// --- round-end aggregation ---------------------------------------------------

export function verdictsFor(
  state: PipelineState,
  phase: string,
  round: number,
  specialties: readonly string[],
): VerdictReply[] {
  const out: VerdictReply[] = [];
  for (const specialty of specialties) {
    // A retry's verdict supersedes the failed attempt it retried: take the
    // LAST valid reply for stage+round, not the first.
    const matches = state.results.filter(
      (r) =>
        r.stage === `${phase}-review:${specialty}` &&
        r.round === round &&
        r.reply !== undefined &&
        r.reply !== null &&
        typeof r.reply === 'object',
    );
    const last = matches.at(-1);
    if (last?.reply) {
      out.push(last.reply as VerdictReply);
    }
  }
  return out;
}

/** The newest valid verdict reply a seat produced in this phase, if any. */
export function lastSeatReply(
  state: PipelineState,
  phase: string,
  specialty: string,
): VerdictReply | undefined {
  const matches = state.results.filter(
    (r) =>
      r.stage === `${phase}-review:${specialty}` &&
      r.reply !== undefined &&
      r.reply !== null &&
      typeof r.reply === 'object',
  );
  return matches.at(-1)?.reply as VerdictReply | undefined;
}

export function openBlockingFindings(
  verdicts: readonly VerdictReply[],
  blocking: readonly string[],
): Finding[] {
  return verdicts.flatMap((v) => v.findings).filter((f) => blocking.includes(f.severity));
}

/**
 * Aggregated review gate: every required specialty must have a verdict for
 * this round, every verdict must approve, and no blocking-severity finding
 * may be open. Missing verdicts are never treated as passes.
 */
export function allReviewsApproved(
  state: PipelineState,
  phase: 'design' | 'editorial' | 'code',
  round: number,
  specialties: readonly string[],
  cfg: PipelineConfig,
): GateResult {
  const verdicts = verdictsFor(state, phase, round, specialties);
  if (verdicts.length < specialties.length) {
    const seen = new Set(
      state.results
        .filter(
          (r) =>
            r.stage.startsWith(`${phase}-review:`) &&
            r.round === round &&
            r.reply !== undefined &&
            r.reply !== null &&
            typeof r.reply === 'object',
        )
        .map((r) => r.stage.split(':')[1]),
    );
    const missing = specialties.filter((s) => !seen.has(s));
    return deny(`reviews missing or invalid: ${missing.join(', ') || '(unparsable verdict)'}`);
  }
  const rejecting = verdicts.filter((v) => v.verdict !== 'approve');
  if (rejecting.length > 0) {
    return deny(`${rejecting.length} reviewer(s) requested changes`);
  }
  const open = openBlockingFindings(verdicts, cfg.blockingSeverities);
  if (open.length > 0) {
    return deny(`${open.length} blocking findings open`);
  }
  return allow(`all ${specialties.length} reviewers approved with no blocking findings`);
}

// --- deterministic gates -----------------------------------------------------

/**
 * Find npm packages in the repo (root, one level of subdirectories, and
 * `tools/*`), so the check gate works in monorepos with no root package.
 */
export function packageDirs(repoDir: string): string[] {
  const dirs: string[] = [];
  const probe = (dir: string): void => {
    if (existsSync(join(dir, 'package.json'))) {
      dirs.push(dir);
    }
  };
  probe(repoDir);
  for (const top of readdirSync(repoDir)) {
    if (top === 'node_modules' || top.startsWith('.')) continue;
    probe(join(repoDir, top));
    if (top === 'tools' || top === 'packages') {
      const parent = join(repoDir, top);
      try {
        for (const child of readdirSync(parent)) probe(join(parent, child));
      } catch {
        /* ignore unreadable dirs */
      }
    }
  }
  return dirs;
}

/** A cargo workspace at the repo root is gated by cargo, not by npm. */
export function hasCargoWorkspace(repoDir: string): boolean {
  return existsSync(join(repoDir, 'Cargo.toml'));
}

/** The cargo gate, exactly as `testsGreen` runs it: fmt check, clippy, tests. */
export const CARGO_GATE: readonly string[][] = [
  ['fmt', '--all', '--', '--check'],
  ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'],
  ['test', '--workspace', '--locked'],
];

/**
 * ADR-016 Decision 2 — the capture budgets. One gate record is cloned into
 * every later checkpoint and rendered into every later review and coder
 * prompt, so the two bounds are what keep it from climbing toward the 8 MiB
 * checkpoint ceiling (A6) or pricing the very context Decision 4 measures:
 * `MAX_GATE_TAIL_CHARS` bounds each leg's captured tail, `MAX_GATE_LEGS`
 * bounds the leg array (an over-cap run keeps its leading legs and gains one
 * elision note leg, exactly as `boundedFindings` does — A7).
 */
export const MAX_GATE_TAIL_CHARS = 2_000;
export const MAX_GATE_LEGS = 32;

/**
 * Byte window kept per leg before decoding. It sits far above the character
 * budget so the decode-then-bound step (which is what stops a tail cutting a
 * code point) always has the leg's real ending to work from; it only exists
 * so a chatty leg cannot make the runner hold its whole log.
 */
const MAX_TAIL_BYTES = 64 * 1024;

/** The one gate record: whether the gate passed, and the legs that decided it. */
export interface GateRun {
  green: boolean;
  legs: GateLeg[];
}

/** The elision marker's text with no digits in it, so its length is derivable. */
const ELISION_NOTE_OVERHEAD = '[ characters elided by the gate tail cap]'.length;

/** The stated elision, as the tail's own line-leading marker. */
function elisionNote(elided: number): string {
  return `[${String(elided)} characters elided by the gate tail cap]`;
}

/**
 * Bounds a decoded tail to `MAX_GATE_TAIL_CHARS`, keeping its END — a leg's
 * finding is in its last lines — and stating the cut, so a reader never takes
 * a truncated tail for the leg's whole output. Code points, not UTF-16 units:
 * slicing units can cut a surrogate pair and store a lone half of it.
 *
 * The kept span and the count the marker states are one equation, not two
 * guesses: the marker must fit inside the budget it explains, and it explains
 * how much the kept span drops. So `elided = chars.length − keep` and
 * `keep = MAX_GATE_TAIL_CHARS − note.length`, where `note.length` is the fixed
 * overhead plus the digit count of `elided` — one solution, found by
 * substituting it into itself. Sizing the span against a fixed 64-character
 * reservation instead made the marker overstate the elision by however far the
 * note fell short of that reservation (correctness r2, finding 4).
 */
function boundedTail(text: string): string {
  const chars = Array.from(text);
  if (chars.length <= MAX_GATE_TAIL_CHARS) {
    return chars.join('');
  }
  // Seeded one digit high; the substitution is a contraction, so this settles
  // in a step or two and the loop bound is a guard, not the mechanism.
  let elided = chars.length - MAX_GATE_TAIL_CHARS + ELISION_NOTE_OVERHEAD + 1;
  for (let step = 0; step < 8; step += 1) {
    const next = Math.min(
      chars.length,
      Math.max(0, chars.length - MAX_GATE_TAIL_CHARS + ELISION_NOTE_OVERHEAD + digits(elided)),
    );
    if (next === elided) break;
    elided = next;
  }
  const keep = chars.length - elided;
  return `${elisionNote(elided)}${chars.slice(chars.length - keep).join('')}`;
}

function digits(value: number): number {
  return String(value).length;
}

/**
 * Runs one gate leg and records it. `stdio` pipes are the whole change from
 * the boolean-era spawn: the merged stdout+stderr is decoded once, with
 * replacement characters for bytes that are not valid UTF-8, and kept from the
 * END, because `cargo test` writes the `test result:` summary last (A18) and
 * a failing leg's whole point is its last lines.
 */
function runLeg(command: string, args: readonly string[], cwd: string): Promise<GateLeg> {
  const line = [command, ...args].join(' ');
  return new Promise((resolve) => {
    let child: ChildProcess;
    try {
      child = spawn(command, args as string[], { cwd, stdio: ['ignore', 'pipe', 'pipe'] });
    } catch (error) {
      resolve({
        command: line,
        cwd,
        exit: null,
        tail: boundedTail(`spawn failed: ${errorText(error)}`),
      });
      return;
    }
    const chunks: Buffer[] = [];
    let kept = 0;
    const collect = (chunk: Buffer | string): void => {
      const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
      chunks.push(buf);
      kept += buf.length;
      while (kept > MAX_TAIL_BYTES && chunks.length > 1) {
        const drop = chunks.shift();
        if (drop !== undefined) kept -= drop.length;
      }
    };
    child.stdout?.on('data', collect);
    child.stderr?.on('data', collect);
    // A leg whose spawn failed has no exit code, and the reason sits in this
    // message rather than in an empty tail: `exit: null` without a spelled-out
    // failure is the discarded stream this decision exists to end.
    let spawnError: string | undefined;
    const tailOf = (note: string | undefined): string => {
      const text = Buffer.concat(chunks).toString('utf8').trimEnd();
      if (note === undefined) return boundedTail(text);
      return boundedTail(text.length === 0 ? note : `${note}\n${text}`);
    };
    let settled = false;
    const settle = (leg: GateLeg): void => {
      if (settled) return;
      settled = true;
      resolve(leg);
    };
    child.on('error', (error) => {
      spawnError = errorText(error);
      // A child that never started emits `error` and no numeric exit; the
      // `close` event that follows it carries nothing further.
      if (child.pid === undefined) {
        settle({ command: line, cwd, exit: null, tail: tailOf(`spawn failed: ${spawnError}`) });
      }
    });
    child.on('close', (code, signal) => {
      const note =
        signal !== null
          ? `killed by signal ${signal}`
          : code === null
            ? spawnError === undefined
              ? 'no exit code recorded'
              : `spawn failed: ${spawnError}`
            : undefined;
      // Null exit is the recorded shape of "the child never produced one":
      // a failed spawn or a signal kill, never a silent pass.
      settle({
        command: line,
        cwd,
        exit: signal === null && code !== null ? code : null,
        tail: tailOf(note),
      });
    });
  });
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Bounds the leg array at capture, appending one elision note leg (A7). */
function boundedLegs(legs: readonly GateLeg[]): GateLeg[] {
  if (legs.length <= MAX_GATE_LEGS) {
    return [...legs];
  }
  const kept = legs.slice(0, MAX_GATE_LEGS);
  kept.push({
    command: '(gate legs elided)',
    cwd: '',
    exit: 0,
    tail: `${String(legs.length - MAX_GATE_LEGS)} further gate leg(s) elided by the record cap`,
  });
  return kept;
}

/**
 * The check gate, with its evidence kept: `npm run check` in every probed
 * package, plus the three `CARGO_GATE` legs when the repo has a cargo
 * workspace. `green` judges every leg that ran — the array bound is a record
 * budget, never a reason to stop running the gate — while `legs` is the
 * bounded record the checkpoint stores and the prompts render.
 * A repo with neither packages nor a workspace has no gate to pass and fails.
 */
export async function checkGate(repoDir: string): Promise<GateRun> {
  const pkgs = packageDirs(repoDir);
  const cargo = hasCargoWorkspace(repoDir);
  if (pkgs.length === 0 && !cargo) {
    return { green: false, legs: [] };
  }
  const legs = await Promise.all(pkgs.map((cwd) => runLeg('npm', ['run', 'check'], cwd)));
  if (cargo) {
    // The cargo legs stay serial, as the boolean-era gate ran them: they share
    // one workspace target dir and each is expensive on its own.
    for (const args of CARGO_GATE) {
      legs.push(await runLeg('cargo', args, repoDir));
    }
  }
  const green = legs.every((leg) => leg.exit === 0);
  return { green, legs: boundedLegs(legs) };
}

/**
 * The check gate's boolean view — the unchanged contract every existing caller
 * and test keeps. The evidence side of the same run is `checkGate`.
 */
export async function testsGreen(repoDir: string): Promise<boolean> {
  return (await checkGate(repoDir)).green;
}

const COMMIT_RE = /^[a-z]+(\([a-z0-9-]+\))?: [a-z0-9].{1,90}$/;

/** Conventional Commits validation for model-authored commit messages. */
export function isSemanticCommit(message: string): boolean {
  return COMMIT_RE.test(message);
}

/**
 * Deterministic ADR status flip: patch the `status:` line in the frontmatter
 * and re-verify by re-parse. Returns the patched content, or null when the
 * input is not a well-formed proposed ADR.
 */
export function flipAdrStatus(content: string): string | null {
  if (!/^---\r?\n[\s\S]*?\r?\n---/.test(content)) {
    return null;
  }
  const patched = content.replace(/^status:[ \t]*proposed[ \t]*$/m, 'status: accepted');
  const ok =
    /^status:[ \t]*accepted[ \t]*$/m.test(patched) && /^status:[ \t]*proposed/m.test(content);
  return ok ? patched : null;
}

/**
 * ADR-023 (023-L3): the deterministic task status flip — the sibling of
 * `flipAdrStatus`, `pending` to `completed`. `disposition` is operator-managed
 * and this flip never touches it.
 */
export function flipTaskStatus(content: string): string | null {
  if (!/^---\r?\n[\s\S]*?\r?\n---/.test(content)) {
    return null;
  }
  const patched = content.replace(/^status:[ \t]*pending[ \t]*$/m, 'status: completed');
  const ok =
    /^status:[ \t]*completed[ \t]*$/m.test(patched) &&
    /^status:[ \t]*pending[ \t]*$/m.test(content);
  return ok ? patched : null;
}

/** NNN allocation: zero-padded, shared between adrs and designs. */
export function nextNnn(existing: string[]): string {
  const used = existing
    .map((name) => /^(\d{3})-/.exec(name)?.[1])
    .filter((n): n is string => n !== undefined)
    .map(Number);
  const next = used.length === 0 ? 1 : Math.max(...used) + 1;
  return String(next).padStart(3, '0');
}

// --- the executable surface (ADR-020, design 020 §1) ------------------------

/**
 * The two prefixes that make up the harness's executable surface: the sources
 * the runner process imports at start, and the prompt templates it renders at
 * stage time. Directory scope, not a static file list — a module or template
 * created inside either prefix is surface the moment it exists, and every
 * imported pipeline module falls under the first one for free.
 *
 * The agent permission files under `.opencode/agents/` are NOT surface in v1:
 * their per-session re-read behaviour is unestablished, and ADR-020 leaves
 * their membership undecided, so the rule that decides a pause names only what
 * the runner demonstrably runs and renders.
 */
export const SURFACE_SRC_PREFIX = 'tools/orchestration/src/';
export const SURFACE_PROMPTS_PREFIX = 'tools/orchestration/prompts/';
export const EXECUTABLE_SURFACE_PREFIXES: readonly string[] = [
  SURFACE_SRC_PREFIX,
  SURFACE_PROMPTS_PREFIX,
];

/**
 * ADR-020 §1: the rule the restart boundary is judged by — true when `path`
 * lies under one of the two surface prefixes. Like `stagePathAllowed` it parses
 * no path: paths reach it only as git plumbing output (repo-root-relative,
 * POSIX separators, the real spelling — `moves.ts` Rule 1 rejects every other
 * spelling as a usage error), so no normalization happens here and a prefix
 * miss means the file is genuinely outside what this process executes.
 */
export function touchesExecutableSurface(path: string): boolean {
  return EXECUTABLE_SURFACE_PREFIXES.some(
    (prefix) => path === prefix.slice(0, -1) || path.startsWith(prefix),
  );
}

// --- stage file ownership ----------------------------------------------------

/** Test paths: owned by the test engineer; reverted out of coder diffs. */
export const TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/;

/**
 * Which paths a stage is allowed to change. Agent-config permissions are only
 * advisory (a model can route around them via subagents), so the runner
 * enforces this matrix deterministically after every successful stage.
 *
 * `flow` is the ADR-010 widening: under `light` the coder owns the tests,
 * because the flow has no test engineer. Exactly `implement` and `fix` gain
 * `TEST_PATH_RE` — the predicate's whole test-path set, co-located `src/x.test.ts`
 * included — so the light coder is never told it owns a file the sweep takes
 * back. Every other rule — including the runner-reserved names, which stay
 * first and win — is the unchanged matrix in both flows.
 *
 * The predicate parses no path (A28): paths reach it only as git plumbing
 * output — repo-root-relative, POSIX separators, the real spelling — and
 * `moves.ts` Rule 1 rejects absolute, backslash and `.`/`..` spellings as
 * usage errors, so no spelling arrives here to be normalized.
 */
export function stagePathAllowed(stage: string, path: string, flow: FlowId = 'full'): boolean {
  // The flow is validated before any rule: an unknown flow must not fall
  // through to `full`, whose rules are the wider ones.
  flowDef(flow);
  // ADR-006 §1: the checkpoint, its `saveState` tmp shape, the escalation
  // marker, and the run lock are runner-reserved. This rule precedes every
  // per-stage rule, so no stage class — document, coder, or test — can own
  // them at any depth or under another case. The worktree and committed
  // copies can then only ever carry runner-authored bytes, and the §4
  // single-flight premise holds inside a run: no stage may write the lock
  // (security r1, finding 4; the runner re-asserts it at stage boundaries).
  if (namesReservedSegment(path)) {
    return false;
  }
  const inDocs = /^docs\//.test(path);
  const inTests = TEST_PATH_RE.test(path);
  if (stage === 'implement' || stage === 'fix') {
    // Coder may touch anything except tests; docs allowed for build notes.
    // Under `light` it owns the tests too — no test engineer exists in the
    // flow — so the reserved names above are its only fence.
    if (flow === 'light') {
      return true;
    }
    return !inTests;
  }
  if (stage === 'test-authoring' || stage === 'test-update') {
    // Test engineer: tests plus its own design-dir artifacts, never source.
    return inDocs || inTests;
  }
  // All document/review stages (plan, research, proposal*, reviews, doc-sync).
  return inDocs;
}
