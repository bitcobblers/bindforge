import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// The decision-unit ledger readers (design 021, Test strategy #1–#8): the
// anchored unit-id grammar (¶2), the handle grammar (¶4), CRLF tolerance (A13),
// `parseLedger` findings, reservation owners (021-L5), the total join in both
// directions (021-L2), `depends-order` (021-L3) and the `**Unit handles.**`
// declaration note (¶5). Unit tests only: in-module fixture strings, and the
// scratch repos of `tmp-dir.ts` when `checkLedgers` needs a tree — no
// committed markdown for prettier to reformat out from under a shape pin.
//
// The import is dynamic on purpose: `src/pipeline/ledger.ts` is what the
// design adds, and a named static import would fail this file at load while
// the module is unimplemented; the accessors below name the missing export so
// each behavior goes red on its own.
// ---------------------------------------------------------------------------

type LedgerFindingCode =
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

interface LedgerFinding {
  file: string;
  line: number;
  code: LedgerFindingCode;
  detail: string;
}

interface Handle {
  id: string;
}

interface LedgerUnit {
  id: string;
}

interface LedgerFile {
  units: LedgerUnit[];
  findings: LedgerFinding[];
}

interface LedgerSurface {
  parseLedger?: (content: string, nnn: string) => LedgerFile | undefined;
  extractHandles?: (content: string) => Handle[];
  checkLedgers?: (repoDir: string) => LedgerFinding[];
}

const LEDGER = (await import('../../src/pipeline/ledger.js').catch(
  (): LedgerSurface => ({}),
)) as LedgerSurface;

function parseLedger(): NonNullable<LedgerSurface['parseLedger']> {
  if (typeof LEDGER.parseLedger !== 'function') {
    throw new Error('ledger.js exports no parseLedger yet');
  }
  return LEDGER.parseLedger;
}

function extractHandles(): NonNullable<LedgerSurface['extractHandles']> {
  if (typeof LEDGER.extractHandles !== 'function') {
    throw new Error('ledger.js exports no extractHandles yet');
  }
  return LEDGER.extractHandles;
}

function checkLedgers(): NonNullable<LedgerSurface['checkLedgers']> {
  if (typeof LEDGER.checkLedgers !== 'function') {
    throw new Error('ledger.js exports no checkLedgers yet');
  }
  return LEDGER.checkLedgers;
}

const codes = (findings: LedgerFinding[]): LedgerFindingCode[] => findings.map((f) => f.code);
const handleIds = (content: string): string[] => extractHandles()(content).map((h) => h.id);

/** A ledger-bearing ADR document: `front` lines in the frontmatter block,
 * `decision` lines under `## Decision`. Also returns the line array so a test
 * can name the exact line a finding must point at (1-based). */
function adrDoc(front: string[], decision: string[]): { content: string; lines: string[] } {
  const lines = [
    '---',
    'status: proposed',
    ...front,
    '---',
    '',
    '# ADR fixture',
    '',
    '## Context',
    '',
    'why.',
    '',
    '## Decision',
    '',
    ...decision,
  ];
  return { content: `${lines.join('\n')}\n`, lines };
}

const lineOf = (lines: string[], text: string): number => {
  const index = lines.indexOf(text);
  if (index < 0) {
    throw new Error(`fixture line not found: ${text}`);
  }
  return index + 1;
};

/** Write a corpus tree (docs/adrs/**, schemas/**) into a scratch repo. */
function corpusRepo(prefix: string, files: Record<string, string>): string {
  const dir = mkTmpDir(`bindforge-ledger-${prefix}-`);
  for (const [rel, text] of Object.entries(files)) {
    const path = join(dir, rel);
    mkdirSync(join(path, '..'), { recursive: true });
    writeFileSync(path, text, 'utf8');
  }
  return dir;
}

const FINDING_NOTE =
  '**Unit handles.** A line whose final code span holds unit ids is a handle for those units.';

// ===========================================================================
// Test strategy #1 — the anchored id grammar (¶2)
// ===========================================================================

describe('the anchored unit-id grammar', () => {
  // proves: Test strategy #1 — the accept side: a single id, a two-digit
  // number, a multi-id span (spec:111's shape) and a cell-final id.
  it('accepts anchored ids in every shipped handle placement', () => {
    expect(handleIds('Claim.\n`013-C1`\n')).toEqual(['013-C1']);
    expect(handleIds('Claim.\n`021-R12`\n')).toEqual(['021-R12']);
    expect(handleIds('Claim.\n`013-B1 013-B2 013-B3 013-B4`\n')).toEqual([
      '013-B1',
      '013-B2',
      '013-B3',
      '013-B4',
    ]);
    expect(handleIds('| budgets | caps | `013-B4` |\n')).toEqual(['013-B4']);
  });

  // proves: Test strategy #1 — the refuse side: the unanchored grep form
  // `[0-9]{3}-[CLBRS][0-9]{1,2}` matches inside longer tokens; the anchored
  // grammar (look-behind plus trailing guard) must not. A code span whose
  // content is not a valid id list is no handle at all.
  it('refuses ids inside or beside longer tokens', () => {
    expect(handleIds('Claim.\n`1234-C1`\n')).toEqual([]);
    expect(handleIds('Claim.\n`013-C1x`\n')).toEqual([]);
    expect(handleIds('Claim.\n`013-C123`\n')).toEqual([]);
    expect(handleIds('Claim.\n`013-X1`\n')).toEqual([]);
    expect(handleIds('Claim.\n`v013-C1`\n')).toEqual([]);
  });

  // proves: Test strategy #1 — the same grammar reads a unit bullet: an
  // anchored id opens a unit, a malformed bold id is `unit-malformed`.
  it('accepts an anchored id in a bullet and reports a malformed one', () => {
    const good = adrDoc(
      ['ledger: yes'],
      ['### Contracts', '', '- **013-C1** A bounded choice. → spec: Grammar.', ''],
    );
    expect(parseLedger()(good.content, '013')?.units.map((u) => u.id)).toEqual(['013-C1']);

    const bad = adrDoc(
      ['ledger: yes'],
      ['### Contracts', '', '- **013-C1x** A bounded choice. → spec: Grammar.', ''],
    );
    expect(codes(parseLedger()(bad.content, '013')?.findings ?? [])).toContain('unit-malformed');
  });
});

// ===========================================================================
// Test strategy #2 — the handle grammar (¶4, the canonical statement)
// ===========================================================================

describe('extractHandles', () => {
  // proves: Test strategy #2 — the three shipped spellings: a line ending in
  // a handle, a multi-id span (spec:111), and a table cell whose final span
  // closes the cell (spec:115–118).
  it('returns the ids of every shipped handle placement', () => {
    const content = [
      '# Spec',
      '',
      'A claim governed by one unit. `013-C1`',
      'A claim with four. `013-B1 013-B2 013-B3 013-B4`',
      '| entry | norm | closes the cell | `013-B4` |',
      'A later claim. `021-C3`',
    ].join('\n');
    expect([...new Set(handleIds(content))].sort()).toEqual([
      '013-B1',
      '013-B2',
      '013-B3',
      '013-B4',
      '013-C1',
      '021-C3',
    ]);
  });

  // proves: Test strategy #2 — the exclusions: a mid-sentence span (A5/A6),
  // fenced-block lines (``` and ~~~, toggled — fenced content is literal to a
  // markdown parser), frontmatter lines (not prose), and an id span with a
  // trailing word after it.
  it('returns nothing for mid-sentence, fenced, frontmatter or trailing-word ids', () => {
    const content = [
      '---',
      'status: proposed',
      'frontmatter: `013-C5`',
      '---',
      '',
      '# Spec',
      '',
      'The id `013-C1` sits mid-sentence and governs nothing.',
      '```md',
      'A quoted example line. `013-C2`',
      '```',
      '~~~',
      'A tilde-fenced line. `013-C3`',
      '~~~',
      'Words trail the span. `013-C4` and more text.',
    ].join('\n');
    expect(extractHandles()(content)).toEqual([]);
  });
});

// ===========================================================================
// Test strategy #3 — CRLF tolerance (A13)
// ===========================================================================

describe('CRLF tolerance', () => {
  // proves: Test strategy #3 — the same fixture body rewritten `\r\n` in
  // memory yields the identical handle set, and the same tree rewritten CRLF
  // on disk yields the identical findings list (both deliberately red: one
  // `unit-unhandled`, one `handle-dangling`).
  it('yields identical handles and findings for the CRLF twin of a fixture', () => {
    const adr = adrDoc(
      ['ledger: yes'],
      ['### Contracts', '', '- **013-C1** Nobody handles this unit. → spec: Grammar.', ''],
    );
    const spec = [`# Manifests spec`, '', FINDING_NOTE, '', 'Dangling claim. `013-C9`', ''].join(
      '\n',
    );
    const files = {
      'docs/adrs/013-fixture.md': adr.content,
      'schemas/manifests/v1/spec.md': spec,
    };
    const crlfFiles: Record<string, string> = {};
    for (const [rel, text] of Object.entries(files)) {
      crlfFiles[rel] = text.replace(/\n/g, '\r\n');
    }

    expect(handleIds(adr.content.replace(/\n/g, '\r\n'))).toEqual(handleIds(adr.content));

    const summarize = (tree: Record<string, string>): string[] =>
      checkLedgers()(corpusRepo('crlf-', tree))
        .map((f) => `${f.file}:${String(f.line)} ${f.code}`)
        .sort();
    const lf = summarize(files);
    const crlf = summarize(crlfFiles);
    expect(lf).toHaveLength(2);
    expect(crlf).toEqual(lf);
  });
});

// ===========================================================================
// Test strategy #4 — parseLedger and the unit shape (¶2)
// ===========================================================================

describe('parseLedger', () => {
  // proves: Test strategy #4 — the 013 shape is accepted: five kind letters
  // map to their sections, every bullet carries its own pointer, and the
  // clean post-run 013 shape yields no parse findings.
  it('accepts the 013 shape with per-bullet pointers', () => {
    const { content } = adrDoc(
      ['ledger: yes'],
      [
        '### Contracts',
        '',
        '- **013-C1** The manifest declares its schema. → spec: Grammar.',
        '',
        '### Laws',
        '',
        '- **013-L1** Unknown keys refuse. → spec: Version gate.',
        '',
        '### Budgets',
        '',
        '- **013-B1** Id caps bind. → spec: Budgets.',
        '- **013-B2** Key caps bind. → spec: Budgets.',
        '',
        '### Reservations',
        '',
        '- **013-R4** The seed set arrives later. → ADR-019. Owner: ADR-019 (reserved).',
        '- **013-R5** The codec slot waits. → unassigned. Owner: unassigned.',
        '',
      ],
    );
    const file = parseLedger()(content, '013');
    expect(file).toBeDefined();
    expect(file?.units.map((u) => u.id).sort()).toEqual([
      '013-B1',
      '013-B2',
      '013-C1',
      '013-L1',
      '013-R4',
      '013-R5',
    ]);
    expect(file?.findings).toEqual([]);
  });

  // proves: Test strategy #4 — the pre-run 013 shape (A3): `B1`–`B4` inherit
  // the section preamble's `→` and `R4`/`R5` carry none — every one of the six
  // bullets is `pointer-missing`, pointed at its own line. The preamble
  // paragraph and the kind-section prose are not units.
  it('reports pointer-missing for the pre-run 013 shape', () => {
    const bullets = [
      '- **013-B1** Id caps bind.',
      '- **013-B2** Key caps bind.',
      '- **013-B3** Name caps bind.',
      '- **013-B4** Depth caps bind.',
    ];
    const { content, lines } = adrDoc(
      ['ledger: yes'],
      [
        '### Budgets',
        '',
        '→ spec: Budgets.',
        '',
        ...bullets,
        '',
        '### Reservations',
        '',
        '- **013-R4** The seed set arrives later. Owner: ADR-019 (reserved).',
        '- **013-R5** The codec slot waits. Owner: unassigned.',
        '',
      ],
    );
    const file = parseLedger()(content, '013');
    const missing = (file?.findings ?? []).filter((f) => f.code === 'pointer-missing');
    expect(missing.map((f) => f.line).sort((a, b) => a - b)).toEqual(
      [
        ...bullets.map((b) => lineOf(lines, b)),
        lineOf(lines, '- **013-R4** The seed set arrives later. Owner: ADR-019 (reserved).'),
        lineOf(lines, '- **013-R5** The codec slot waits. Owner: unassigned.'),
      ].sort((a, b) => a - b),
    );
  });

  // proves: Test strategy #4 — a `C` unit under `### Laws` is a
  // `unit-kind-mismatch` (the kind letter must match its section heading).
  it('reports unit-kind-mismatch for a C unit under Laws', () => {
    const { content } = adrDoc(
      ['ledger: yes'],
      ['### Laws', '', '- **013-C1** A contract wearing a law badge. → spec: Grammar.', ''],
    );
    expect(codes(parseLedger()(content, '013')?.findings ?? [])).toContain('unit-kind-mismatch');
  });

  // proves: Test strategy #4 — a unit bullet outside any kind section is
  // `unit-misplaced`.
  it('reports unit-misplaced for a unit bullet outside a kind section', () => {
    const { content } = adrDoc(
      ['ledger: yes'],
      [
        '- **013-C1** Orphaned above every kind section. → spec: Grammar.',
        '',
        '### Contracts',
        '',
        '- **013-C2** A homed unit. → spec: Grammar.',
        '',
      ],
    );
    const misplaced = (parseLedger()(content, '013')?.findings ?? []).filter(
      (f) => f.code === 'unit-misplaced',
    );
    expect(misplaced).toHaveLength(1);
  });

  // proves: Test strategy #4 — a repeated id is `unit-duplicate` (per file,
  // ids are unique and per-kind numbers strictly increase).
  it('reports unit-duplicate for a repeated id', () => {
    const { content } = adrDoc(
      ['ledger: yes'],
      [
        '### Contracts',
        '',
        '- **013-C1** The first. → spec: Grammar.',
        '- **013-C1** The repeat. → spec: Grammar.',
        '',
      ],
    );
    expect(codes(parseLedger()(content, '013')?.findings ?? [])).toContain('unit-duplicate');
  });

  // proves: Test strategy #4 — `ledger: yes` with no kind section at all is
  // `ledger-empty` (a ledger's Decision must carry at least one kind section).
  it('reports ledger-empty for ledger: yes with no kind section', () => {
    const { content } = adrDoc(['ledger: yes'], ['Plain prose, no kind sections.', '']);
    const file = parseLedger()(content, '013');
    expect(file).toBeDefined();
    expect(codes(file?.findings ?? [])).toContain('ledger-empty');
  });
});

// ===========================================================================
// Test strategy #5 — reservation owners, 021-L5 (¶2)
// ===========================================================================

describe('reservation owners (021-L5)', () => {
  const ownerFindings = (ownerText: string): LedgerFindingCode[] => {
    const bullet =
      ownerText === ''
        ? '- **013-R5** The codec slot waits. → unassigned.'
        : `- **013-R5** The codec slot waits. → unassigned. ${ownerText}`;
    const { content } = adrDoc(['ledger: yes'], ['### Reservations', '', bullet, '']);
    const file = parseLedger()(content, '013');
    if (file === undefined) {
      throw new Error('a ledger: yes document must parse as a ledger');
    }
    return codes(file.findings);
  };

  // proves: Test strategy #5 — a reservation bullet with no `Owner:` (the
  // shipped 013-R5 shape) is `owner-missing`.
  it('reports owner-missing for a bullet with no Owner', () => {
    expect(ownerFindings('')).toContain('owner-missing');
  });

  // proves: Test strategy #5 — an owner that is none of the three accepted
  // forms (ADR-NNN, unassigned bare or backticked, this ADR's run) is
  // `owner-unknown`: a two-digit `ADR-17` and a free-word owner.
  it('reports owner-unknown for a malformed owner form', () => {
    expect(ownerFindings('Owner: ADR-17')).toContain('owner-unknown');
    expect(ownerFindings('Owner: someday')).toContain('owner-unknown');
  });

  // proves: Test strategy #5 — the clean forms: `ADR-019 (reserved)` (a
  // number no file answers to yet — the ledger line is the registry), the
  // backticked `` `unassigned` `` with 021-R1's shipped trailing prose, bare
  // `unassigned`, and `this ADR's run.`.
  it('accepts every shipped and reserved owner form', () => {
    expect(ownerFindings('Owner: ADR-019 (reserved).')).toEqual([]);
    expect(ownerFindings('Owner: `unassigned` — the backfill run takes it later.')).toEqual([]);
    expect(ownerFindings('Owner: unassigned.')).toEqual([]);
    expect(ownerFindings("Owner: this ADR's run.")).toEqual([]);
  });
});

// ===========================================================================
// Test strategy #6 — the total join, both directions (021-L2)
// ===========================================================================

describe('the total join (021-L2)', () => {
  // proves: Test strategy #6 — a handle with no unit is `handle-dangling`.
  it('reports handle-dangling for a handle that resolves to no unit', () => {
    const dir = corpusRepo('dangling-', {
      'schemas/manifests/v1/spec.md': [
        '# Manifests spec',
        '',
        FINDING_NOTE,
        '',
        'A claim no ledger ever made. `013-C9`',
        '',
      ].join('\n'),
    });
    expect(codes(checkLedgers()(dir))).toEqual(['handle-dangling']);
  });

  // proves: Test strategy #6 — a unit with no handle is `unit-unhandled`
  // except a reservation carrying `unassigned`, and a handle that resolves to
  // a unit in a *different* ledger-bearing ADR is clean (013 Backfill
  // findings 5's shape: the registry spans every ledger-bearing ADR).
  it('reports unit-unhandled except for the unassigned reservation, and resolves across ledgers', () => {
    const ledger013 = [
      '---',
      'status: proposed',
      'ledger: yes',
      '---',
      '',
      '# ADR 013: fixture',
      '',
      '## Context',
      '',
      'why.',
      '',
      '## Decision',
      '',
      '### Contracts',
      '',
      '- **013-C1** A handled contract. → spec: Grammar.',
      '- **013-C2** Nobody points back here. → spec: Grammar.',
      '',
      '### Reservations',
      '',
      '- **013-R5** The codec slot waits. → unassigned. Owner: unassigned.',
      '',
    ];
    const dir = corpusRepo('join-', {
      'docs/adrs/013-fixture.md': `${ledger013.join('\n')}\n`,
      // 013-C1's only handle lives in ANOTHER ledger-bearing ADR — the
      // cross-attribution 013's Backfill findings 5 needs before the
      // series is backfilled. 021-C1 carries its own.
      'docs/adrs/021-fixture.md': [
        '---',
        'status: proposed',
        'ledger: yes',
        '---',
        '',
        '# ADR 021: fixture',
        '',
        '## Context',
        '',
        'why.',
        '',
        '## Decision',
        '',
        '### Contracts',
        '',
        '- **021-C1** The join is total. → spec: Grammar.',
        '',
        FINDING_NOTE,
        '',
        '013-C1 is attributed across ledgers here. `013-C1`',
        '021-C1 is attributed here. `021-C1`',
        '',
      ].join('\n'),
      'schemas/manifests/v1/spec.md': ['# Manifests spec', '', FINDING_NOTE, ''].join('\n'),
    });
    const findings = checkLedgers()(dir);
    expect(findings.filter((f) => f.code === 'handle-dangling')).toEqual([]);
    const unhandled = findings.filter((f) => f.code === 'unit-unhandled');
    expect(unhandled).toHaveLength(1);
    expect(unhandled[0]?.file).toBe('docs/adrs/013-fixture.md');
    expect(unhandled[0]?.line).toBe(
      lineOf(ledger013, '- **013-C2** Nobody points back here. → spec: Grammar.'),
    );
    // Both handle-carrying ADRs state the convention, so the note rule stays silent.
    expect(codes(findings)).not.toContain('note-missing');
  });
});

// ===========================================================================
// Test strategy #7 — depends-order, 021-L3 (¶6)
// ===========================================================================

describe('depends-order (021-L3)', () => {
  // proves: Test strategy #7 — `depends-on: [021]` on a `013-*.md` is
  // `depends-order`, and the shipped series values (A16) are the clean case.
  it('reports depends-order for a dependency on a larger number', () => {
    const dir = corpusRepo('depends-', {
      'docs/adrs/013-fixture.md': [
        '---',
        'status: proposed',
        'depends-on: [021]',
        '---',
        '',
        '# ADR 013: fixture',
        '',
        'why.',
        '',
      ].join('\n'),
      'docs/adrs/012-fixture.md': [
        '---',
        'status: proposed',
        'depends-on: [011]',
        '---',
        '',
        '# ADR 012: fixture',
        '',
        'why.',
        '',
      ].join('\n'),
      'docs/adrs/015-fixture.md': [
        '---',
        'status: proposed',
        'depends-on: [011, 012, 014]',
        '---',
        '',
        '# ADR 015: fixture',
        '',
        'why.',
        '',
      ].join('\n'),
    });
    const findings = checkLedgers()(dir);
    expect(codes(findings)).toEqual(['depends-order']);
    expect(findings[0]?.file).toBe('docs/adrs/013-fixture.md');
  });

  // proves: Test strategy #7 — the check binds every ADR that carries
  // `depends-on`, ledger-bearing or not (¶6): the absent `ledger:` key
  // exempts the unit, pointer, owner and kind checks only — while
  // `parseLedger` still answers `undefined` for a file without the key.
  it('checks an ADR that carries depends-on but no ledger key', () => {
    const content = [
      '---',
      'status: proposed',
      'depends-on: [021]',
      '---',
      '',
      '# ADR 014: fixture',
      '',
      'No ledger key, no unit-shaped text.',
      '',
    ].join('\n');
    const dir = corpusRepo('depends-nokey-', { 'docs/adrs/014-fixture.md': content });
    expect(codes(checkLedgers()(dir))).toEqual(['depends-order']);
    expect(parseLedger()(content, '014')).toBeUndefined();
  });
});

// ===========================================================================
// Test strategy #8 — the note-missing declaration rule (¶5)
// ===========================================================================

describe('note-missing (the Unit handles declaration)', () => {
  // proves: Test strategy #8 — a corpus file that carries a handle but never
  // states the convention in a `**Unit handles.**` note is `note-missing`.
  it('fires for a handle-bearing corpus file without the note', () => {
    const dir = corpusRepo('note-', {
      'docs/adrs/013-fixture.md': [
        '---',
        'status: proposed',
        'ledger: yes',
        '---',
        '',
        '# ADR 013: fixture',
        '',
        '## Context',
        '',
        'why.',
        '',
        '## Decision',
        '',
        '### Contracts',
        '',
        '- **013-C1** A handled contract. → spec: Grammar.',
        '',
      ].join('\n'),
      'schemas/alpha/v1/spec.md': [
        '# Alpha spec',
        '',
        'A handle, no declaration. `013-C1`',
        '',
      ].join('\n'),
    });
    expect(codes(checkLedgers()(dir))).toEqual(['note-missing']);
  });

  // proves: Test strategy #8 — and it does not fire for a file whose ids sit
  // only in prose positions (A5): no handle, therefore no declaration owed.
  it('stays silent for a file whose ids sit in prose positions', () => {
    const dir = corpusRepo('prose-', {
      'schemas/beta/v1/notes.md': [
        '# Beta notes',
        '',
        'The unit `013-C2` is quoted mid-sentence, so this line is not a handle.',
        '',
      ].join('\n'),
    });
    expect(checkLedgers()(dir)).toEqual([]);
  });
});
