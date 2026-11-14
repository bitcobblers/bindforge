import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { CARGO_GATE } from '../../src/pipeline/gates.js';

// ---------------------------------------------------------------------------
// Run 008 wrote "no stage in this pipeline can run a toolchain" into its build
// notes, and ADR-011 inherited that as a `verified` assumption: its stages
// hand-wrote rustfmt-shaped code they had never formatted, and 4 of its 5 code
// rounds went to formatter complaints. The cause was an allowlist of narrow
// command prefixes — a compound command is scanned into subcommands and every
// leg must pass, so `cd crates/core && cargo test` was denied on its `cd` leg.
// These tests pin the widened posture so a future tightening cannot silently
// re-blind a stage.
// ---------------------------------------------------------------------------

const AGENTS_DIR = fileURLToPath(new URL('../../../../.opencode/agents/', import.meta.url));

const CODE_AGENTS = ['code/coder.md', 'code/test-engineer.md'];
const REVIEW_AGENTS = [
  'codereview/code-correctness.md',
  'codereview/code-compatibility.md',
  'codereview/code-security.md',
  'codereview/code-performance.md',
  'codereview/code-testing.md',
];
const STAGE_AGENTS = [...CODE_AGENTS, ...REVIEW_AGENTS];

interface ShellRule {
  resource: string;
  effect: 'allow' | 'deny' | 'ask';
}

/**
 * The `shell` rules of an agent, in declaration order. Only the shell rules
 * matter here: the edit/read rules are covered by the ownership tests.
 */
function shellRules(agentFile: string): ShellRule[] {
  const text = readFileSync(`${AGENTS_DIR}${agentFile}`, 'utf8');
  const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text);
  if (front === null) {
    throw new Error(`${agentFile}: no frontmatter`);
  }
  const rules: ShellRule[] = [];
  let pending: { resource?: string } | undefined;
  for (const line of (front[1] as string).split(/\r?\n/)) {
    const action = /^\s*- action: (\S+)\s*$/.exec(line);
    if (action !== null) {
      if (pending?.resource !== undefined) {
        throw new Error(`${agentFile}: shell rule without an effect`);
      }
      pending = action[1] === 'shell' ? {} : undefined;
      continue;
    }
    if (pending === undefined) {
      continue;
    }
    const resource = /^\s*resource: "(.*)"\s*$/.exec(line);
    if (resource !== null) {
      pending.resource = resource[1];
      continue;
    }
    const effect = /^\s*effect: (allow|deny|ask)\s*$/.exec(line);
    if (effect !== null && pending.resource !== undefined) {
      rules.push({ resource: pending.resource, effect: effect[1] });
      pending = undefined;
    }
  }
  return rules;
}

/**
 * OpenCode's documented matching: whole-value wildcards (`*` spans `/`, `?` is
 * one character), and a shell pattern ending in ` *` also matches the command
 * with no arguments.
 */
function matches(pattern: string, value: string): boolean {
  const asRegex = (source: string): RegExp =>
    new RegExp(
      `^${source
        .replace(/[.+^${}()|[\]\\]/g, '\\$&')
        .replace(/\*/g, '[\\s\\S]*')
        .replace(/\?/g, '[\\s\\S]')}$`,
    );
  return (
    asRegex(pattern).test(value) ||
    (pattern.endsWith(' *') && asRegex(pattern.slice(0, -2)).test(value))
  );
}

/** The last matching rule wins; no match is an ask, which a stage never sees. */
function resolveShell(rules: readonly ShellRule[], command: string): 'allow' | 'deny' | 'ask' {
  let verdict: 'allow' | 'deny' | 'ask' = 'ask';
  for (const rule of rules) {
    if (matches(rule.resource, command)) {
      verdict = rule.effect;
    }
  }
  return verdict;
}

/**
 * The scanner checks a compound command leg by leg, so a leg that is denied
 * denies the call. Quoting is not exercised by any fixture here.
 */
function legs(command: string): string[] {
  return command
    .split(/\s*(?:&&|\|\||[|;])\s*/)
    .map((leg) => leg.trim())
    .filter((leg) => leg.length > 0);
}

const gateCommands = CARGO_GATE.map((args) => `cargo ${args.join(' ')}`);

describe('stage agent shell rights', () => {
  it('covers every stage agent', () => {
    expect(STAGE_AGENTS).toHaveLength(7);
    for (const agent of STAGE_AGENTS) {
      expect(shellRules(agent).length, agent).toBeGreaterThan(0);
    }
  });

  it('runs each cargo gate leg, and npm run check', () => {
    for (const agent of STAGE_AGENTS) {
      const rules = shellRules(agent);
      for (const command of [...gateCommands, 'npm run check', 'npm test']) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('allow');
      }
    }
  });

  it('passes a compound gate command leg by leg', () => {
    const compounds = [
      'cd crates/core && cargo test --workspace --locked',
      'cargo test --workspace --locked | tail -20',
      'cd tools/orchestration && npm run check',
      'cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings',
    ];
    for (const agent of STAGE_AGENTS) {
      const rules = shellRules(agent);
      for (const command of compounds) {
        for (const leg of legs(command)) {
          expect(resolveShell(rules, leg), `${agent}: ${leg}`).toBe('allow');
        }
      }
    }
  });

  it('keeps commits, branch state, moves and publishing with the runner', () => {
    const forbidden = [
      'git commit -m feat: x',
      'git push origin main',
      'git checkout -b side',
      'git reset --hard HEAD~1',
      'mv a.rs b.rs',
      'rm -rf target',
      'tee src/lib.rs',
      'cargo publish',
    ];
    for (const agent of STAGE_AGENTS) {
      const rules = shellRules(agent);
      for (const command of forbidden) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
    }
  });

  it('lets a writing stage format, and keeps a review seat read-only', () => {
    for (const agent of CODE_AGENTS) {
      expect(resolveShell(shellRules(agent), 'cargo fmt --all')).toBe('allow');
      // A writing stage formats in place, through either formatter leg.
      expect(resolveShell(shellRules(agent), 'rustfmt --edition 2021 src/foo.rs'), agent).toBe(
        'allow',
      );
    }
    for (const agent of REVIEW_AGENTS) {
      const rules = shellRules(agent);
      // The seat grades the tree; it cannot rewrite it — not through the bare
      // rustfmt binary, the cargo manifest-mutating subcommands, or the
      // package's write scripts.
      for (const command of [
        'cargo fmt --all',
        'cargo fix --all',
        'rustfmt --edition 2021 src/foo.rs',
        'cargo new scratch',
        'cargo add serde',
        'cargo remove serde',
        'cargo update',
        'npm run format',
        'npm run clean',
        'npm run lint:fix',
      ]) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
      // ...while still being able to verify the read-only fmt legs of the gate.
      expect(resolveShell(rules, 'cargo fmt --all -- --check'), agent).toBe('allow');
      expect(resolveShell(rules, 'rustfmt --check src/foo.rs'), agent).toBe('allow');
    }
  });
});

// ---------------------------------------------------------------------------
// ADR-016 Decision 3: read-only reach and the permission-layer run-record fence.
// The five document families gain the read-only inspection set (star form, so a
// bare `cat`/`wc` command name never grants reach), still with the leading
// `shell *` deny keeping `cargo`/`npm` out. The five design seats copy the code
// seats' check block, so they run the read-only gate legs while keeping every
// write-leg denied. The run-record block — `shell *transcripts*`/`*reviews*`
// denies and `glob`/`grep` denies over the run-record paths — is appended to
// every shell-holding agent; the review seats then re-open exactly their own
// specialty's report through `git diff`, last-match-wins, so the re-read
// Decision 1 legalizes stays open while every sibling path stays denied.
//
// The matcher in this file stays pinned to the grammar the pinned client's
// server documents (A16/A17): §3's deny-then-reallow order only holds there.
// ---------------------------------------------------------------------------

const DESIGN_SEATS = [
  'review/design-correctness.md',
  'review/design-compatibility.md',
  'review/design-security.md',
  'review/design-performance.md',
  'review/design-complexity.md',
];
const DOC_FAMILIES = [
  'research/researcher.md',
  'design/designer.md',
  'design/planner.md',
  'design/editor.md',
  'ops/doc-sync.md',
];
const REVIEW_SEATS = [...REVIEW_AGENTS, ...DESIGN_SEATS];
const SHELL_HOLDING = [...STAGE_AGENTS, ...DESIGN_SEATS, ...DOC_FAMILIES];

interface AnyRule {
  action: string;
  resource: string;
  effect: 'allow' | 'deny' | 'ask';
}

/** Every permission rule of an agent, in declaration order, any action. */
function allRules(agentFile: string): AnyRule[] {
  const text = readFileSync(`${AGENTS_DIR}${agentFile}`, 'utf8');
  const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(text);
  if (front === null) throw new Error(`${agentFile}: no frontmatter`);
  const rules: AnyRule[] = [];
  let action: string | undefined;
  let resource: string | undefined;
  for (const line of (front[1] as string).split(/\r?\n/)) {
    const a = /^\s*- action: (\S+)\s*$/.exec(line);
    if (a !== null) {
      action = a[1];
      resource = undefined;
      continue;
    }
    if (action === undefined) continue;
    const r = /^\s*resource: "(.*)"\s*$/.exec(line);
    if (r !== null) {
      resource = r[1];
      continue;
    }
    const e = /^\s*effect: (allow|deny|ask)\s*$/.exec(line);
    if (e !== null && resource !== undefined) {
      rules.push({ action, resource, effect: e[1] });
      resource = undefined;
    }
  }
  return rules;
}

function deniesResource(agentFile: string, action: string, resource: string): boolean {
  return allRules(agentFile).some(
    (r) => r.action === action && r.resource === resource && r.effect === 'deny',
  );
}

/** The specialty an agent file names: `code-testing.md` → `testing`. */
function specialtyOf(agentFile: string): string {
  const base = agentFile.slice(agentFile.lastIndexOf('/') + 1).replace(/\.md$/, '');
  return base.replace(/^code-/, '').replace(/^design-/, '');
}

describe('design panel read-only reach (ADR-016 Decision 3)', () => {
  // proves: Test strategy #4 — the five design seats run every compound gate
  // command leg, read-only, just as the code seats already do.
  it('runs every compound gate leg', () => {
    const compounds = [
      'cd crates/core && cargo test --workspace --locked',
      'cargo test --workspace --locked | tail -20',
      'cd tools/orchestration && npm run check',
      'cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings',
    ];
    for (const agent of DESIGN_SEATS) {
      const rules = shellRules(agent);
      for (const command of [...gateCommands, 'npm run check', 'npm test']) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('allow');
      }
      for (const command of compounds) {
        for (const leg of legs(command)) {
          expect(resolveShell(rules, leg), `${agent}: ${leg}`).toBe('allow');
        }
      }
    }
  });

  // proves: Test strategy #5 — the design seat keeps the code-seat invariants:
  // read-only gate legs allowed, every write leg denied.
  it('keeps the code-seat read-only posture', () => {
    const allowed = [
      'cargo test --workspace --locked',
      'cargo fmt --all -- --check',
      'rustfmt --check src/foo.rs',
    ];
    const forbidden = [
      'cargo fmt --all',
      'cargo fix --all',
      'cargo clippy --fix',
      'rustfmt --edition 2021 src/foo.rs',
      'cargo new scratch',
      'cargo add serde',
      'cargo remove serde',
      'cargo update',
      'npm run format',
      'npm run clean',
      'npm run lint:fix',
      'git commit -m feat: x',
      'git push origin main',
      'git checkout -b side',
      'rm -rf target',
      'mv a.rs b.rs',
      'cargo publish',
    ];
    for (const agent of DESIGN_SEATS) {
      const rules = shellRules(agent);
      for (const command of allowed) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('allow');
      }
      for (const command of forbidden) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
    }
  });
});

describe('document families read-only reach (ADR-016 Decision 3)', () => {
  // proves: Test strategy #4 — the doc families allow inspection with arguments
  // (`wc <file>`, `cat <path>`, star form never the bare name) and deny the
  // whole toolchain, keeping the leading `shell *` deny authoritative.
  it('allows inspection, denies the toolchain', () => {
    for (const agent of DOC_FAMILIES) {
      const rules = shellRules(agent);
      expect(resolveShell(rules, 'wc docs/designs/002-x/01-research.md'), `${agent}: wc`).toBe(
        'allow',
      );
      expect(resolveShell(rules, 'cat docs/adrs/016-x.md'), `${agent}: cat`).toBe('allow');
      // With arguments, never the bare command names (section 3 star form): a
      // bare `cat` allow rule matches only the argument-less command, so the
      // with-argument calls above are exactly what proves the families copied
      // the star-form inspection set byte-for-byte from `code/coder.md`.
      expect(resolveShell(rules, 'cargo test'), `${agent}: cargo test`).toBe('deny');
      expect(resolveShell(rules, 'npm run check'), `${agent}: npm run check`).toBe('deny');
    }
  });

  // proves: Test strategy #4 — the doc families carry the same run-record
  // `glob`/`grep` denies as the review seats.
  it('carries the run-record glob/grep denies', () => {
    const globDenies = ['docs/designs/**/transcripts/**', 'docs/designs/**/reviews/**'];
    const grepDenies = ['*transcripts*', '*reviews*'];
    for (const agent of [...DOC_FAMILIES, ...DESIGN_SEATS]) {
      for (const resource of globDenies) {
        expect(deniesResource(agent, 'glob', resource), `${agent}: glob ${resource}`).toBe(true);
      }
      for (const resource of grepDenies) {
        expect(deniesResource(agent, 'grep', resource), `${agent}: grep ${resource}`).toBe(true);
      }
    }
  });
});

describe('run-record shell fence and the own-report re-open (ADR-016 Decision 3)', () => {
  // proves: Test strategy #4 — every shell-holding agent denies a command that
  // names `transcripts` or `reviews`.
  it('denies a command naming a run record', () => {
    const commands = [
      'ls docs/designs/002-x/transcripts/research-r0.md',
      'cat docs/designs/002-x/reviews/design/security-r1c1.md',
    ];
    for (const agent of SHELL_HOLDING) {
      const rules = shellRules(agent);
      for (const command of commands) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
    }
  });

  // proves: Test strategy #4 — each review seat's own-specialty `git diff` allow
  // beats that deny, while a sibling-specialty path stays denied.
  it('re-opens the seat’s own report and keeps a sibling denied', () => {
    for (const agent of REVIEW_SEATS) {
      const specialty = specialtyOf(agent);
      const phase = agent.startsWith('codereview/') ? 'code' : 'design';
      const other = otherSpecialty(phase, specialty);
      const own = `git diff docs/designs/002-x/reviews/${phase}/${specialty}-r1c1.md`;
      const sibling = `git diff docs/designs/002-x/reviews/${phase}/${other}-r1c1.md`;
      const rules = shellRules(agent);
      expect(resolveShell(rules, own), `${agent}: ${own}`).toBe('allow');
      expect(resolveShell(rules, sibling), `${agent}: ${sibling}`).toBe('deny');
    }
  });
});

/** A specialty of the same phase, guaranteed different from `own`. */
function otherSpecialty(phase: string, own: string): string {
  const pool =
    phase === 'code'
      ? ['correctness', 'compatibility', 'security', 'performance', 'testing']
      : ['correctness', 'compatibility', 'security', 'performance', 'complexity'];
  return pool.find((s) => s !== own) as string;
}

// ---------------------------------------------------------------------------
// security r2c0 finding 2 (accepted): the fence covers the run checkpoint too.
// `.pipeline.json` is a run record — it carries every seat's findings and
// verdicts (what ADR-002/007 independence guards) and now the gate record
// beside them — and every agent config already denies *edit* on it. The diff
// that widened the document families' shell reach granted them a route to it
// (`cat`, `git show`, `git log` name no `transcripts`/`reviews` token), so the
// run-record block now names the checkpoint on the shell, read and glob routes.
// ---------------------------------------------------------------------------

describe('the run checkpoint is inside the run-record fence (ADR-016 Decision 3)', () => {
  const CHECKPOINT_RESOURCE = 'docs/designs/**/.pipeline.json';

  // proves: security r2c0 finding 2 — every shell-holding agent denies a command
  // that names the checkpoint, whichever read command spells it.
  it('denies a command naming the run checkpoint', () => {
    const commands = [
      'cat docs/designs/002-x/.pipeline.json',
      'wc -c docs/designs/002-x/.pipeline.json',
      'git show HEAD:docs/designs/002-x/.pipeline.json',
      'git log --oneline -- docs/designs/002-x/.pipeline.json',
    ];
    for (const agent of SHELL_HOLDING) {
      const rules = shellRules(agent);
      for (const command of commands) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
    }
  });

  // proves: security r2c0 finding 2 — the fence denies the tool routes as well:
  // `read` and `glob` over the checkpoint path, on every shell-holding agent.
  it('carries the checkpoint read and glob denies', () => {
    for (const agent of SHELL_HOLDING) {
      for (const action of ['read', 'glob'] as const) {
        const label = `${agent}: ${action} ${CHECKPOINT_RESOURCE}`;
        expect(deniesResource(agent, action, CHECKPOINT_RESOURCE), label).toBe(true);
      }
    }
  });

  // proves: security r2c0 finding 2, the other direction — the fence is scoped to
  // the run record. The run's own documents stay readable through the inspection
  // set the families were widened with; a `pipeline.json` deny that swallowed
  // them would deny the work, not the record.
  it('leaves the run documents themselves readable', () => {
    const readable = [
      'cat docs/designs/002-x/01-research.md',
      'wc docs/designs/002-x/02-proposal.md',
      'cat docs/designs/002-x/03-build-notes.md',
    ];
    for (const agent of DOC_FAMILIES) {
      const rules = shellRules(agent);
      for (const command of readable) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('allow');
      }
    }
  });
});
