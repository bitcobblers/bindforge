import type { MockInstance } from 'vitest';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { main, resolveReplyTimeoutSeconds } from '../src/cli.js';

const TIMEOUT_ENV = 'OPENCODE_REPLY_TIMEOUT_SECONDS';

// proves: Test strategy #10/#11 — the SDK side of the CLI is replaced wholesale, so
// main() is observed in-process through its runPrompt call and never reaches the SDK
// codec (A17/A19: the vi.mock route, not stubbed HTTP).
const runnerMocks = vi.hoisted(() => ({
  createClient: vi.fn(),
  runPrompt: vi.fn(async () => ({
    session: {
      id: 'ses_test',
      title: 'Untitled Session',
      projectID: 'project-test',
      time: { created: 1000, updated: 2000 },
    },
    prompt: 'sample prompt',
    reply: {
      messageID: 'msg_test',
      finish: 'stop',
      content: { fact: 'stub reply' },
    },
  })),
}));

vi.mock('../src/opencode-runner.js', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/opencode-runner.js')>();
  return {
    ...actual,
    createClient: runnerMocks.createClient,
    runPrompt: runnerMocks.runPrompt,
  };
});

let spies: MockInstance[] = [];

/** Silences the CLI's printing for one test and returns the spies for assertions. */
function silenceOutput(): { log: MockInstance; error: MockInstance } {
  const log = vi.spyOn(console, 'log').mockImplementation(() => undefined);
  const error = vi.spyOn(console, 'error').mockImplementation(() => undefined);
  spies.push(log, error);
  return { log, error };
}

/** Runs the resolver and returns the message of the error it throws. */
function resolverMessage(flagValue: number | undefined, envValue: string | undefined): string {
  try {
    resolveReplyTimeoutSeconds(flagValue, envValue);
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
  throw new Error(
    `expected resolveReplyTimeoutSeconds(${String(flagValue)}, ${JSON.stringify(envValue)}) to throw`,
  );
}

/** Reads the options object main() handed to the mocked runPrompt. */
function runPromptOptions(callIndex = 0): unknown {
  return runnerMocks.runPrompt.mock.calls[callIndex]?.[1];
}

describe('resolveReplyTimeoutSeconds', () => {
  // proves: Test strategy #7 — precedence is flag -> env -> 30 s default
  it('prefers the flag over the environment and the environment over the 30 s default', () => {
    expect(resolveReplyTimeoutSeconds(45, '5')).toBe(45);
    expect(resolveReplyTimeoutSeconds(undefined, '5')).toBe(5);
    expect(resolveReplyTimeoutSeconds(undefined, undefined)).toBe(30);
  });

  // proves: Test strategy #8 — out-of-range / non-integer flag values are rejected by a
  // range check whose message names --timeout and the offending value
  it('rejects flag values that are not integer seconds in 1..3600', () => {
    for (const flagValue of [0, -5, 2.5, Number.POSITIVE_INFINITY]) {
      const message = resolverMessage(flagValue, undefined);
      expect(message).toContain('--timeout');
      expect(message).toContain(String(flagValue));
    }
    // yargs hands junk flag input through as a non-finite number (A8)
    const message = resolverMessage(Number.NaN, undefined);
    expect(message).toContain('--timeout');
    expect(message).toContain('NaN');
  });

  // proves: Test strategy #8 — the ^\d{1,5}$ format gate runs before any Number()
  // conversion, so loose numeric strings never silently change the written value
  it('rejects $OPENCODE_REPLY_TIMEOUT_SECONDS values that are not plain digits', () => {
    for (const envValue of ['abc', '30s', '0x2d', '1e4', ' 45\n', '+45']) {
      const message = resolverMessage(undefined, envValue);
      expect(message).toContain(`$${TIMEOUT_ENV}`);
      expect(message).toContain(JSON.stringify(envValue));
    }
  });

  // proves: Test strategy #8 — the set-but-empty env value (OPENCODE_REPLY_TIMEOUT_SECONDS=)
  // is a configuration error, not the unset default
  it('rejects an empty $OPENCODE_REPLY_TIMEOUT_SECONDS value', () => {
    const message = resolverMessage(undefined, '');
    expect(message).toContain(`$${TIMEOUT_ENV}`);
    expect(message).toContain('""');

    const overridden = resolverMessage(5, '');
    expect(overridden).toContain(`$${TIMEOUT_ENV}`);
  });

  // proves: Test strategy #8 (security r1) — any reflected env value is
  // JSON.stringify-escaped, so stderr cannot carry raw control/ANSI bytes
  it('escapes control characters from an environment value instead of echoing them raw', () => {
    const message = resolverMessage(undefined, '\x1b[2J');
    expect(message).toContain(JSON.stringify('\x1b[2J'));
    expect(message).not.toContain('\x1b');
    expect(message).not.toContain('[2J\n');
  });

  // proves: Test strategy #8 — the format gate is precedence-independent: a valid flag
  // does not mask a broken environment
  it('rejects a malformed environment value even when a valid flag overrides it', () => {
    const message = resolverMessage(5, '30s');
    expect(message).toContain(`$${TIMEOUT_ENV}`);
    expect(message).toContain('"30s"');
  });

  // proves: Test strategy #9 — the accepted bounds are 1 and 3600 s (A16), and 3601 s is
  // a range error, not a format error
  it('accepts the 1 s and 3600 s bounds and rejects 3601 s', () => {
    expect(resolveReplyTimeoutSeconds(1, undefined)).toBe(1);
    expect(resolveReplyTimeoutSeconds(3_600, undefined)).toBe(3_600);
    expect(resolveReplyTimeoutSeconds(undefined, '1')).toBe(1);
    expect(resolveReplyTimeoutSeconds(undefined, '3600')).toBe(3_600);

    const flagMessage = resolverMessage(3_601, undefined);
    expect(flagMessage).toContain('--timeout');
    expect(flagMessage).toContain('3601');

    const envMessage = resolverMessage(undefined, '3601');
    expect(envMessage).toContain(`$${TIMEOUT_ENV}`);
    expect(envMessage).toContain('"3601"');
  });
});

describe('main', () => {
  beforeEach(() => {
    runnerMocks.createClient.mockReset();
    runnerMocks.runPrompt.mockReset().mockResolvedValue({
      session: {
        id: 'ses_test',
        title: 'Untitled Session',
        projectID: 'project-test',
        time: { created: 1000, updated: 2000 },
      },
      prompt: 'sample prompt',
      reply: {
        messageID: 'msg_test',
        finish: 'stop',
        content: { fact: 'stub reply' },
      },
    });
  });

  afterEach(() => {
    for (const spy of spies) {
      spy.mockRestore();
    }
    spies = [];
    delete process.env[TIMEOUT_ENV];
  });

  // proves: Test strategy #10 — argv -> yargs -> resolver -> PromptOptions.timeoutMs
  // (seconds cross the CLI boundary, milliseconds cross the CLI<->runner boundary)
  it('passes --timeout 2 to runPrompt as timeoutMs 2000', async () => {
    silenceOutput();

    await main(['node', 'cli', '--timeout', '2']);

    expect(runnerMocks.runPrompt).toHaveBeenCalledTimes(1);
    expect(runPromptOptions()).toEqual(expect.objectContaining({ timeoutMs: 2_000 }));
  });

  // proves: Test strategy #10 — $OPENCODE_REPLY_TIMEOUT_SECONDS is the flag's default
  it('passes an environment default to runPrompt when no flag is given', async () => {
    process.env[TIMEOUT_ENV] = '2';
    silenceOutput();

    await main(['node', 'cli']);

    expect(runnerMocks.runPrompt).toHaveBeenCalledTimes(1);
    expect(runPromptOptions()).toEqual(expect.objectContaining({ timeoutMs: 2_000 }));
  });

  // proves: Test strategy #10 — the explicit flag wins over the environment
  it('lets --timeout win over $OPENCODE_REPLY_TIMEOUT_SECONDS', async () => {
    process.env[TIMEOUT_ENV] = '2';
    silenceOutput();

    await main(['node', 'cli', '--timeout', '5']);

    expect(runnerMocks.runPrompt).toHaveBeenCalledTimes(1);
    expect(runPromptOptions()).toEqual(expect.objectContaining({ timeoutMs: 5_000 }));
  });

  // proves: Test strategy #10 — the 30 s default survives when nothing is configured
  it('passes the 30 s default to runPrompt when neither flag nor environment is set', async () => {
    silenceOutput();

    await main(['node', 'cli']);

    expect(runnerMocks.runPrompt).toHaveBeenCalledTimes(1);
    expect(runPromptOptions()).toEqual(expect.objectContaining({ timeoutMs: 30_000 }));
  });

  // proves: Test strategy #11 — the resolver runs before createClient() and before any
  // output, so a config error is one line and side-effect free
  it('fails a malformed environment value before building a client or printing anything', async () => {
    process.env[TIMEOUT_ENV] = '30s';
    const { log } = silenceOutput();

    const pending = main(['node', 'cli', '--timeout', '5']);
    const error = await pending.then(
      () => undefined,
      (reason: unknown) => reason,
    );

    expect(error).toBeInstanceOf(Error);
    const message = error instanceof Error ? error.message : String(error);
    expect(message).toContain(`$${TIMEOUT_ENV}`);
    expect(message).toContain('"30s"');
    expect(message.split('\n')).toHaveLength(1);
    expect(runnerMocks.createClient).not.toHaveBeenCalled();
    expect(log).not.toHaveBeenCalled();
  });
});
