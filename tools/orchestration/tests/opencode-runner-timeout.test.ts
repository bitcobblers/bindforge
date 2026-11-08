import type { OpenCodeClient } from '@opencode/client';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { z } from 'zod';

import { ReplyTimeoutError, runPrompt } from '../src/opencode-runner.js';

const FactSchema = z.object({ fact: z.string() });

const fakeSession = {
  id: 'ses_test',
  title: 'Untitled Session',
  projectID: 'project-test',
  time: { created: 1000, updated: 2000 },
};

const fakeReply = {
  id: 'msg_test',
  type: 'assistant',
  finish: 'stop',
  content: [{ type: 'text', text: '{"fact":"tsc is fast"}' }],
  model: { id: 'test-model', providerID: 'test-provider' },
  tokens: { input: 1, output: 2, reasoning: 0, cache: { read: 0, write: 0 } },
  cost: 0,
  time: { created: 1000, completed: 2000 },
};

/** Build a minimal stand-in for the SDK client with overridable endpoints. */
function stubClient(
  handlers: {
    create?: () => Promise<unknown>;
    prompt?: () => Promise<unknown>;
    list?: () => Promise<{ data: unknown[] }>;
  } = {},
): OpenCodeClient {
  return {
    session: {
      create: handlers.create ?? (async () => fakeSession),
      prompt: handlers.prompt ?? (async () => ({})),
    },
    message: {
      list: handlers.list ?? (async () => ({ data: [fakeReply] })),
    },
  } as unknown as OpenCodeClient;
}

/** Awaits `pending` and returns its rejection reason; fails if it resolves. */
async function rejection(pending: Promise<unknown>): Promise<unknown> {
  try {
    await pending;
  } catch (error) {
    return error;
  }
  throw new Error('expected runPrompt to reject, but it resolved');
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

describe('runPrompt reply timeout', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  // proves: Test strategy #2 — runPrompt honours PromptOptions.timeoutMs and reports
  // the expiry as a distinguishable error type carrying the effective duration
  it('rejects with ReplyTimeoutError naming the configured 4 s duration', async () => {
    vi.useFakeTimers();
    try {
      const list = vi.fn().mockResolvedValue({ data: [] });
      const pending = runPrompt(stubClient({ list }), { timeoutMs: 4_000 }, FactSchema, 'say hi');
      void pending.catch(() => undefined);

      await vi.advanceTimersByTimeAsync(4_250);

      const error = await rejection(pending);
      expect(error).toBeInstanceOf(ReplyTimeoutError);
      const message = messageOf(error);
      // the pre-existing report string stays an exact prefix (A5)
      expect(message.startsWith('No response received from OpenCode')).toBe(true);
      expect(message).toContain('within 4 s');
    } finally {
      vi.useRealTimers();
    }
  });

  // proves: Test strategy #3 — the timeout is a cap, not a forced failure
  it('resolves when a reply arrives after one poll but before a custom deadline', async () => {
    vi.useFakeTimers();
    try {
      const list = vi
        .fn()
        .mockResolvedValueOnce({ data: [] })
        .mockResolvedValue({ data: [fakeReply] });
      const pending = runPrompt(stubClient({ list }), { timeoutMs: 4_000 }, FactSchema, 'say hi');

      await vi.advanceTimersByTimeAsync(500);

      const result = await pending;
      expect(list).toHaveBeenCalledTimes(2);
      expect(result.reply.messageID).toBe('msg_test');
      expect(result.reply.content).toEqual({ fact: 'tsc is fast' });
    } finally {
      vi.useRealTimers();
    }
  });

  // proves: Test strategy #4 — DEFAULT_REPLY_TIMEOUT_MS (30 s) is the runner-side fallback
  it('keeps polling until 30 s elapsed when no timeoutMs is given', async () => {
    vi.useFakeTimers();
    try {
      const list = vi.fn().mockResolvedValue({ data: [] });
      const pending = runPrompt(stubClient({ list }), {}, FactSchema, 'say hi');
      let settled = false;
      pending.then(
        () => {
          settled = true;
        },
        () => {
          settled = true;
        },
      );

      await vi.advanceTimersByTimeAsync(29_000);
      expect(settled).toBe(false);
      expect(list.mock.calls.length).toBeGreaterThan(1);

      await vi.advanceTimersByTimeAsync(2_000);

      const error = await rejection(pending);
      expect(error).toBeInstanceOf(ReplyTimeoutError);
      expect(messageOf(error)).toContain('within 30 s');
    } finally {
      vi.useRealTimers();
    }
  });

  // proves: Test strategy #5 — fail closed at the exported API boundary: an invalid
  // timeoutMs rejects with a TypeError and emits zero requests (no infinite poll loop)
  for (const timeoutMs of [Number.POSITIVE_INFINITY, Number.NaN, 0, -5, 3_600_001]) {
    it(`rejects with a TypeError and no requests for timeoutMs of ${String(timeoutMs)}`, async () => {
      const create = vi.fn();
      const prompt = vi.fn();
      const list = vi.fn();
      const client = stubClient({ create, prompt, list });

      const error = await rejection(runPrompt(client, { timeoutMs }, FactSchema, 'say hi'));

      expect(error).toBeInstanceOf(TypeError);
      expect(create).not.toHaveBeenCalled();
      expect(prompt).not.toHaveBeenCalled();
      expect(list).not.toHaveBeenCalled();
    });
  }

  // proves: Test strategy #6 — sub-poll-interval contract: the deadline is only tested
  // after each reply check, so the loop overshoots by at most one 250 ms interval and
  // the <n> s report is timeoutMs / 1000 rendered as a plain number
  it('checks message.list exactly twice before timing out when the timeout is under one poll interval', async () => {
    vi.useFakeTimers();
    try {
      const list = vi.fn().mockResolvedValue({ data: [] });
      const pending = runPrompt(stubClient({ list }), { timeoutMs: 100 }, FactSchema, 'say hi');
      void pending.catch(() => undefined);

      await vi.advanceTimersByTimeAsync(500);

      const error = await rejection(pending);
      expect(error).toBeInstanceOf(ReplyTimeoutError);
      expect(list).toHaveBeenCalledTimes(2);
      expect(messageOf(error)).toContain('within 0.1 s');
    } finally {
      vi.useRealTimers();
    }
  });
});
