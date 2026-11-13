import { mkdtemp, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { sessionTokens, writeTranscript } from '../../src/pipeline/prompts.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 4, measurement. `sessionTokens` sums the numeric fields of
// every assistant message's `tokens` over the attempt's whole message list —
// per-key, no field names enumerated (A13) — which is the session's usage, not
// one reply's turn (the Options note why "record only the reply message's
// tokens" was rejected). `writeTranscript` prints a `- tokens:` line beside the
// unchanged `- parameters:` line, and omits it entirely when no message carried
// usage; the parameters note keeps its old meaning (endpoint exposure, not
// usage). Nothing new lands in the checkpoint (Compatibility matrix).
// ---------------------------------------------------------------------------

function assistant(tokens?: Record<string, unknown>): unknown {
  return { type: 'assistant', tokens };
}

describe('sessionTokens (ADR-016 Decision 4)', () => {
  // proves: Test strategy #8 — sums per-key numeric usage across a fixture
  // message list, ignoring non-numeric fields and messages with no usage.
  it('sums numeric usage per key across the message list', () => {
    const total = sessionTokens([
      assistant({ input: 10, output: 5, reason: 2 }),
      assistant({ input: 3, output: 2, cached: 1 }),
      assistant(),
      assistant({ input: 'not-a-number', output: 4 }),
    ]);
    expect(total?.input).toBe(13);
    expect(total?.output).toBe(11);
    expect(total?.reason).toBe(2);
    expect(total?.cached).toBe(1);
  });

  // proves: Test strategy #8 — a list with no usage anywhere yields no total, so
  // the transcript prints no `- tokens:` line for it.
  it('returns nothing when no message carries usage', () => {
    expect(sessionTokens([assistant(), assistant()])).toBeUndefined();
  });

  // proves: Test strategy #8, closed by correctness r2c0 finding 1 (accepted) —
  // the fixture is the shape the pinned client actually types `tokens` as
  // (`TokenUsageInfo`: `input`/`output`/`reasoning` plus a nested `cache` group
  // of `read`/`write`). A nested group of numeric fields flattens into dotted
  // keys, so no whole group is dropped silently — the cached-prefix term is the
  // biggest number in a stage session and the one Decision 4 exists to make
  // checkable. Non-numeric members are still skipped, not summed as NaN.
  it('sums the real TokenUsageInfo shape, flattening the nested cache group', () => {
    const total = sessionTokens([
      assistant({ input: 100, output: 20, reasoning: 3, cache: { read: 900, write: 50 } }),
      assistant({ input: 10, output: 5, cache: { read: 200, write: 10 } }),
      assistant({ input: 1, output: 1, cache: { read: 'not-a-number' } }),
    ]);
    expect(total?.input).toBe(111);
    expect(total?.output).toBe(26);
    expect(total?.reasoning).toBe(3);
    expect(total?.['cache.read']).toBe(1_100);
    expect(total?.['cache.write']).toBe(60);
  });
});

describe('writeTranscript tokens line (ADR-016 Decision 4)', () => {
  const baseInput = {
    stage: 'implement',
    round: 0,
    agent: 'code/coder',
    templateFile: 'implement.md',
    templateSha256: 'abc123',
    startedAt: 1,
    endedAt: 2,
    prompt: 'do it',
    rawReply: '{"ok":true}',
  };

  // proves: Test strategy #8 — prints `- tokens:` when usage is present, beside
  // the parameters line which stays byte-unchanged.
  it('prints the tokens line when usage is present', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-tokens-'));
    const path = await writeTranscript(dir, {
      ...baseInput,
      tokens: { input: 120, output: 40 },
    });
    const content = await readFile(path, 'utf8');
    expect(content).toContain('- tokens:');
    expect(content).toContain('120');
    expect(content).toContain('40');
    // the parameters line keeps its old meaning and text
    expect(content).toContain('- parameters:');
    expect(content).toContain('not exposed by endpoint');
  });

  // proves: Test strategy #8, closed by correctness r2c0 finding 1 — the printed
  // line carries the flattened keys, so the cache terms reach the transcript the
  // operator re-reads (Rollout 4) instead of being summed away upstream.
  it('prints the flattened cache terms when the total carries them', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-tokens-cache-'));
    const path = await writeTranscript(dir, {
      ...baseInput,
      tokens: { input: 120, output: 40, 'cache.read': 1_800, 'cache.write': 90 },
    });
    const content = await readFile(path, 'utf8');
    expect(content).toContain('- tokens:');
    expect(content).toContain('cache.read');
    expect(content).toContain('1800');
  });

  // proves: Test strategy #8 — omits the tokens line when no usage is present,
  // and still prints the unchanged parameters line.
  it('omits the tokens line when no usage is present', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-tokens-absent-'));
    const path = await writeTranscript(dir, baseInput);
    const content = await readFile(path, 'utf8');
    expect(content).not.toContain('- tokens:');
    expect(content).toContain('- parameters:');
    expect(content).toContain('not exposed by endpoint');
  });
});
