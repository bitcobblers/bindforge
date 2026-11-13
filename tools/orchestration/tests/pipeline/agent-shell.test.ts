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
    }
    for (const agent of REVIEW_AGENTS) {
      const rules = shellRules(agent);
      // The seat grades the tree; it cannot rewrite it.
      for (const command of ['cargo fmt --all', 'cargo fix --all', 'npm run format']) {
        expect(resolveShell(rules, command), `${agent}: ${command}`).toBe('deny');
      }
      // ...while still being able to verify the read-only fmt leg of the gate.
      expect(resolveShell(rules, 'cargo fmt --all -- --check'), agent).toBe('allow');
    }
  });
});
