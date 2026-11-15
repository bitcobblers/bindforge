import { describe, expect, it } from 'vitest';

import { resolveMaxParallel } from '../../src/pipeline/cli.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';

// ---------------------------------------------------------------------------
// ADR-003 §Configuration: `--max-parallel <N>` on `run` and `resume`, env
// fallback $OPENCODE_MAX_PARALLEL, resolved by a local resolveMaxParallel that
// mirrors resolveReplyTimeoutSeconds' strict fail-closed pattern: a
// ^\d{1,5}$ format gate on the env string that runs EVEN WHEN the flag
// overrides it (bad env is a config error), a range check (integer >= 1) as a
// separate error naming the source, and all validation before side effects.
// ---------------------------------------------------------------------------

const MAX_PARALLEL_ENV = 'OPENCODE_MAX_PARALLEL';

/** Runs the resolver and returns the message of the error it throws. */
function resolverMessage(flagValue: number | undefined, envValue: string | undefined): string {
  try {
    resolveMaxParallel(flagValue, envValue);
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
  throw new Error(
    `expected resolveMaxParallel(${String(flagValue)}, ${JSON.stringify(envValue)}) to throw`,
  );
}

describe('resolveMaxParallel', () => {
  // proves: Test strategy #8 — valid values pass through; precedence is flag -> env
  it('prefers the flag over the environment and passes valid values through', () => {
    expect(resolveMaxParallel(3, '2')).toBe(3);
    expect(resolveMaxParallel(undefined, '2')).toBe(2);
    expect(resolveMaxParallel(1, undefined)).toBe(1);
  });

  // proves: Test strategy #8 — the documented default is 4 (operator decision
  // 2026-11-15; DEFAULT_CONFIG is the sole default authority)
  it('defaults to 4 when neither flag nor environment is set', () => {
    expect(resolveMaxParallel(undefined, undefined)).toBe(4);
    expect(DEFAULT_CONFIG.maxParallelReviews).toBe(4);
  });

  // proves: Test strategy #8 — flag values < 1 are rejected by a range check
  // whose message names --max-parallel and the offending value
  it('rejects flag values that are not integers >= 1', () => {
    for (const flagValue of [0, -5, 2.5, Number.POSITIVE_INFINITY]) {
      const message = resolverMessage(flagValue, undefined);
      expect(message).toContain('--max-parallel');
      expect(message).toContain(String(flagValue));
    }
    // yargs hands junk flag input through as a non-finite number
    const message = resolverMessage(Number.NaN, undefined);
    expect(message).toContain('--max-parallel');
    expect(message).toContain('NaN');
  });

  // proves: Test strategy #8 — the ^\d{1,5}$ format gate runs before any
  // Number() conversion, so loose numeric strings never silently change the
  // value, and the error names $OPENCODE_MAX_PARALLEL with the raw string
  it('rejects $OPENCODE_MAX_PARALLEL values that are not plain digits', () => {
    for (const envValue of ['abc', '2x', '0x2', '1e1', ' 2\n', '+2', '100000', '']) {
      const message = resolverMessage(undefined, envValue);
      expect(message).toContain(`$${MAX_PARALLEL_ENV}`);
      expect(message).toContain(JSON.stringify(envValue));
    }
  });

  // proves: Test strategy #8 — a malformed $OPENCODE_MAX_PARALLEL is a config
  // error even when a valid --max-parallel overrides it
  it('rejects a malformed environment value even when a valid flag overrides it', () => {
    const message = resolverMessage(3, '3s');
    expect(message).toContain(`$${MAX_PARALLEL_ENV}`);
    expect(message).toContain('"3s"');
  });

  // proves: Test strategy #8 — the range check is separate from the format
  // gate: "0" passes ^\d{1,5}$ but fails integer >= 1, and the error names the
  // env source
  it('rejects a well-formed but out-of-range environment value by range', () => {
    const message = resolverMessage(undefined, '0');
    expect(message).toContain(`$${MAX_PARALLEL_ENV}`);
    expect(message).toContain('0');
  });
});
