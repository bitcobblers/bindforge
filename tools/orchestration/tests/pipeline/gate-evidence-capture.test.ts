import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

// `CARGO_GATE` is the shipped leg data; `checkGate` is this design's capture
// seam — until it exists, every test in this file fails at the import.
import { CARGO_GATE, checkGate } from '../../src/pipeline/gates.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 2, capture side (`checkGate`). The check gate used to spawn
// every leg with `stdio: 'ignore'` and persist one boolean; the harness held
// the streams and dropped them (01-research.md, Decision 2). `checkGate` now
// records, per leg, the command line as spawned, the package dir, the child
// exit code, and the LAST `MAX_GATE_TAIL_CHARS` characters of the leg's merged
// output — bounded at the write so a green gate's `test result:` line and a red
// gate's failure both survive into the checkpoint, and a spawn failure is a
// recorded `exit: null` rather than a discarded stream.
//
// This is the capture-budget test the ADR names as load-bearing (01-research.md
// § Risks): the two budgets — the bounded tail and the bounded leg array — are
// what keep one gate record from blowing past the 8 MiB checkpoint ceiling.
// ---------------------------------------------------------------------------

// The shipped capture budgets (proposal Rollout 2): a tail of 2 000 characters,
// a 32-leg array. Pinned numerically here because the design names them as the
// bound the capture must respect, not as an implementation constant.
const MAX_GATE_TAIL_CHARS = 2_000;
const MAX_GATE_LEGS = 32;

/**
 * A generator a fixture package's `check` script runs. Every leg the gate spawns
 * is `npm run check` in a package dir whose `check` is `node <gen> <mode>`, so
 * each leg's captured output is exactly what the mode asks for.
 */
function generatorScript(): string {
  return [
    "const mode = process.argv[2] ?? 'green';",
    "if (mode === 'green') {",
    "  console.log('compiling the crate');",
    "  console.log('test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured');",
    '  process.exit(0);',
    '}',
    "if (mode === 'big') {",
    "  process.stdout.write('x'.repeat(4000) + '\\nTAIL_END\\n');",
    '  process.exit(0);',
    '}',
    "if (mode === 'badutf') {",
    '  process.stdout.write(Buffer.from([0x66, 0x6f, 0x6f, 0xff, 0xfe, 0x80, 0x41]));',
    "  process.stdout.write('\\nDONE\\n');",
    '  process.exit(0);',
    '}',
    "if (mode === 'fail') {",
    "  console.log('test result: FAILED. 0 passed; 1 failed; 0 ignored');",
    "  console.error('assertion failed at src/lib.rs:3:5');",
    '  process.exit(1);',
    '}',
    'process.exit(0);',
  ].join('\n');
}

/**
 * A repo with four `npm run check` packages — root, two under `tools/`, one
 * under `packages/` — and no `Cargo.toml`, so `checkGate` probes exactly these
 * four legs (no cargo legs fire without a workspace). `modes` selects each
 * package's behaviour, keyed by the same relative path `packageDirs` yields.
 */
function fixtureRepo(modes: Record<string, string>): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-gate-capture-'));
  writeFileSync(join(dir, 'gate.mjs'), generatorScript(), 'utf8');

  const pkg = (rel: string, mode: string): void => {
    const pkgDir = join(dir, rel);
    mkdirSync(pkgDir, { recursive: true });
    // `cwd` for a package leg is that package dir, so the generator is reached
    // relative to it: `gate.mjs` at the root, `../../gate.mjs` from a grandchild.
    const gen = rel === '.' ? 'gate.mjs' : join('..', '..', 'gate.mjs');
    writeFileSync(
      join(pkgDir, 'package.json'),
      JSON.stringify({ name: `pkg-${rel}`, scripts: { check: `node ${gen} ${mode}` } }),
      'utf8',
    );
  };

  for (const [rel, mode] of Object.entries(modes)) pkg(rel, mode);
  return dir;
}

function legByCwd(legs: { cwd: string }[], name: string): { cwd: string } | undefined {
  return legs.find((leg) => leg.cwd === name || leg.cwd.endsWith(`/${name}`));
}

describe('checkGate — capture (ADR-016 Decision 2)', () => {
  // proves: Test strategy #1 — one leg per probed package, each naming its
  // command line, its package dir, and its exit code; a green leg's tail carries
  // the `test result:` line behind a behaviour claim (A18).
  it('records one leg per probed package with command, cwd, exit and a test-result tail', async () => {
    const repoDir = fixtureRepo({ '.': 'green', 'tools/a': 'green', 'tools/b': 'green' });
    const gate = await checkGate(repoDir);

    expect(gate.green).toBe(true);
    expect(gate.legs).toHaveLength(3);

    for (const leg of gate.legs) {
      expect(leg.command).toContain('npm run check');
      expect(leg.exit).toBe(0);
      expect(leg.tail).toContain('test result: ok. 5 passed; 0 failed');
    }
    expect(legByCwd(gate.legs, repoDir)).toBeDefined();
    expect(legByCwd(gate.legs, join('tools', 'a'))).toBeDefined();
    expect(legByCwd(gate.legs, join('tools', 'b'))).toBeDefined();
  });

  // proves: Test strategy #1 — a red leg records its nonzero exit and a tail
  // that carries the failure; the gate is not green.
  it('records the failing command with a nonzero exit and a tail naming the failure', async () => {
    const repoDir = fixtureRepo({ '.': 'green', 'tools/a': 'fail' });
    const gate = await checkGate(repoDir);

    expect(gate.green).toBe(false);
    const failing = gate.legs.find((leg) => leg.exit !== 0);
    expect(failing, 'a leg must record the nonzero exit').toBeDefined();
    expect(failing?.exit).not.toBe(0);
    expect(failing?.tail).toContain('FAILED');
    expect(failing?.tail).toContain('assertion failed at src/lib.rs');
  });

  // proves: Test strategy #1 — the tail is the last `MAX_GATE_TAIL_CHARS`
  // characters of the leg's output, taken from the END: a far longer stream is
  // cut to the budget and still ends with the leg's final line.
  it('bounds each tail to the last MAX_GATE_TAIL_CHARS characters', async () => {
    const repoDir = fixtureRepo({ '.': 'big' });
    const gate = await checkGate(repoDir);
    const leg = gate.legs[0];

    expect(leg).toBeDefined();
    expect(leg?.tail.length).toBeLessThanOrEqual(MAX_GATE_TAIL_CHARS);
    // The cut is from the end, so the leg's last line survives the budget.
    expect(leg?.tail).toContain('TAIL_END');
    // and the very start of a 4 000-character fill does not.
    expect(leg?.tail.startsWith('xxxx')).toBe(false);
  });

  // proves: Test strategy #1 — bytes that are not valid UTF-8 become replacement
  // characters in the decoded tail, never a split code point: the tail carries a
  // U+FFFD and no lone surrogate, and the bytes after the bad ones still land.
  it('decodes a tail with replacement characters, never a split code point', async () => {
    const repoDir = fixtureRepo({ '.': 'badutf' });
    const gate = await checkGate(repoDir);
    const leg = gate.legs[0];

    expect(leg).toBeDefined();
    expect(leg?.tail).toContain('\uFFFD');
    expect(leg?.tail).toContain('DONE');
    // No unpaired surrogate anywhere in the stored tail.
    expect(leg?.tail).not.toMatch(/[\uD800-\uDBFF](?![\uDC00-\uDFFF])/);
    expect(leg?.tail).not.toMatch(/(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/);
  });

  // proves: Test strategy #1 — a spawn failure records `exit: null` and spells
  // the failure in the tail (a signal kill is the other `null` case). Forcing
  // `npm` unresolvable makes every leg fail to spawn.
  it('records exit null and a failure tail when a leg cannot spawn', async () => {
    const repoDir = fixtureRepo({ '.': 'green', 'tools/a': 'green' });
    const savedPath = process.env.PATH;
    try {
      process.env.PATH = join(repoDir, 'no-binaries-here');
      const gate = await checkGate(repoDir);
      expect(gate.green).toBe(false);
      expect(gate.legs.length).toBeGreaterThan(0);
      for (const leg of gate.legs) {
        expect(leg.exit, leg.command).toBeNull();
        expect(leg.tail.length, `${leg.command} tail names the failure`).toBeGreaterThan(0);
      }
    } finally {
      process.env.PATH = savedPath;
    }
  });

  // proves: Test strategy #1 — legs over `MAX_GATE_LEGS` collapse to an elision
  // note leg at capture, exactly as `boundedFindings` appends its note (A7):
  // whatever the probe finds, the array the checkpoint stores is bounded, so
  // one gate record cannot climb toward the 8 MiB loader ceiling. The fixture
  // probes 34 packages (root + 33 under `tools/`, the nesting `packageDirs`
  // pins), beyond the 32-leg array.
  it('collapses legs over MAX_GATE_LEGS to an elision leg', async () => {
    const modes: Record<string, string> = { '.': 'green' };
    for (let i = 0; i < 33; i += 1) {
      modes[join('tools', `p${String(i).padStart(2, '0')}`)] = 'green';
    }
    const repoDir = fixtureRepo(modes);
    const gate = await checkGate(repoDir);

    // The record is bounded, and the bound is array-level: a capped number of
    // real legs plus one appended note.
    expect(gate.legs.length).toBeGreaterThan(0);
    expect(gate.legs.length).toBeLessThanOrEqual(MAX_GATE_LEGS + 1);
    const notes = gate.legs.filter((leg) => `${leg.command}\n${leg.tail}`.match(/elided/i));
    expect(notes, 'one elision note leg').toHaveLength(1);
  }, 120_000);

  // proves: Test strategy #1/#14 — with a Cargo workspace present, the capture
  // runs the three `CARGO_GATE` legs as data: one leg per cargo command line,
  // and a green `cargo test` leg's tail from the END of its output carries the
  // `test result:` summary line (A18). The cargo binary is stubbed on `PATH`
  // so the workspace probe, the leg spawning, and the capture are all the real
  // code paths while the host toolchain stays out of the unit test (rule: unit
  // tests stub the outside, never reach for it).
  it('captures every CARGO_GATE leg when a cargo workspace is present', async () => {
    const repoDir = fixtureRepo({ '.': 'green' });
    writeFileSync(join(repoDir, 'Cargo.toml'), '[workspace]\nmembers = []\n', 'utf8');

    const binDir = join(repoDir, 'fake-bin');
    mkdirSync(binDir, { recursive: true });
    writeFileSync(
      join(binDir, 'cargo'),
      [
        '#!/bin/sh',
        'if [ "$1" = "test" ]; then',
        '  echo "running 3 tests"',
        '  echo "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured"',
        'fi',
        'exit 0',
      ].join('\n'),
      { encoding: 'utf8', mode: 0o755 },
    );

    const savedPath = process.env.PATH;
    try {
      process.env.PATH = `${binDir}:${savedPath}`;
      const gate = await checkGate(repoDir);
      expect(gate.green).toBe(true);

      for (const args of CARGO_GATE) {
        const command = `cargo ${args.join(' ')}`;
        const leg = gate.legs.find((l) => l.command === command);
        expect(leg, `a leg records ${command}`).toBeDefined();
        expect(leg?.exit).toBe(0);
      }
      const testLeg = gate.legs.find((l) => l.command === 'cargo test --workspace --locked');
      expect(testLeg?.tail, 'the test leg tail carries the summary line').toContain(
        'test result: ok. 3 passed',
      );
    } finally {
      process.env.PATH = savedPath;
    }
  }, 60_000);
});
