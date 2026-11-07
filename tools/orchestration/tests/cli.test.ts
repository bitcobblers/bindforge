import { Buffer } from 'node:buffer';

import type { OpenCodeClient } from '@opencode/client';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { z } from 'zod';

import {
  DEFAULT_BASE_URL,
  SAMPLE_PROMPT,
  createClient,
  extractText,
  parseContent,
  runPrompt,
} from '../src/cli.js';

const FactSchema = z.object({ fact: z.string() });

const fakeSession = {
  id: 'ses_test',
  title: 'SDK connection test',
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

describe('extractText', () => {
  it('joins the text of all text parts', () => {
    expect(
      extractText([
        { type: 'text', text: 'a' },
        { type: 'text', text: 'b' },
      ]),
    ).toBe('ab');
  });

  it('ignores non-text parts', () => {
    const parts = [
      { type: 'text', text: 'yes' },
      { type: 'reasoning', text: 'no' },
      { type: 'tool' },
    ];
    expect(extractText(parts)).toBe('yes');
  });

  it('returns an empty string when there are no parts', () => {
    expect(extractText([])).toBe('');
  });
});

describe('parseContent', () => {
  it('parses JSON and returns the schema-validated value', () => {
    expect(parseContent(FactSchema, '{"fact":"hello"}')).toEqual({ fact: 'hello' });
  });

  it('throws a ZodError when the JSON does not match the schema', () => {
    expect(() => parseContent(FactSchema, '{"fact":42}')).toThrow(z.ZodError);
  });

  it('throws a SyntaxError for malformed JSON', () => {
    expect(() => parseContent(FactSchema, 'not json')).toThrow(SyntaxError);
  });
});

describe('createClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  /** Replace global fetch with a recorder; responses always fail (500). */
  function captureFetch(): Array<{ url: string; headers: Headers }> {
    const calls: Array<{ url: string; headers: Headers }> = [];
    vi.stubGlobal('fetch', async (input: URL | string, init?: RequestInit) => {
      calls.push({ url: String(input), headers: new Headers(init?.headers) });
      return new Response('{}', { status: 500 });
    });
    return calls;
  }

  // Fire one request and ignore the client error: only the request matters.
  async function attempt(client: OpenCodeClient): Promise<void> {
    await client.session.create({}).catch(() => undefined);
  }

  function basicAuth(username: string, password: string): string {
    return `Basic ${Buffer.from(`${username}:${password}`).toString('base64')}`;
  }

  it('sends no Authorization header when no password is configured', async () => {
    const calls = captureFetch();
    await attempt(createClient('http://127.0.0.1:1'));
    expect(calls).toHaveLength(1);
    expect(calls[0]?.headers.get('authorization')).toBeNull();
  });

  it('uses basic auth with the default username when only a password is set', async () => {
    const calls = captureFetch();
    await attempt(createClient('http://127.0.0.1:1', { password: 'secret' }));
    expect(calls[0]?.headers.get('authorization')).toBe(basicAuth('opencode', 'secret'));
  });

  it('uses basic auth with the configured username', async () => {
    const calls = captureFetch();
    await attempt(createClient('http://127.0.0.1:1', { username: 'me', password: 'secret' }));
    expect(calls[0]?.headers.get('authorization')).toBe(basicAuth('me', 'secret'));
  });

  it('falls back to the default base URL when none is given', async () => {
    const calls = captureFetch();
    await attempt(createClient());
    expect(calls[0]?.url.startsWith(DEFAULT_BASE_URL)).toBe(true);
  });
});

describe('runPrompt', () => {
  it('creates a session, prompts it, and returns the structured result', async () => {
    const create = vi.fn().mockResolvedValue(fakeSession);
    const prompt = vi.fn().mockResolvedValue({});
    const list = vi.fn().mockResolvedValue({ data: [fakeReply] });
    const client = stubClient({ create, prompt, list });

    const result = await runPrompt(client, FactSchema, 'say hi');

    expect(create).toHaveBeenCalledTimes(1);
    expect(prompt).toHaveBeenCalledWith({ sessionID: 'ses_test', text: 'say hi' });
    expect(result.session.id).toBe('ses_test');
    expect(result.session.projectID).toBe('project-test');
    expect(result.prompt).toBe('say hi');
    expect(result.reply.messageID).toBe('msg_test');
    expect(result.reply.content).toEqual({ fact: 'tsc is fast' });
    expect(result.reply.finish).toBe('stop');
  });

  it('uses the sample prompt by default', async () => {
    const result = await runPrompt(stubClient(), FactSchema);
    expect(result.prompt).toBe(SAMPLE_PROMPT);
  });

  it('polls message.list until a completed assistant reply appears', async () => {
    const list = vi
      .fn()
      .mockResolvedValueOnce({ data: [] })
      .mockResolvedValueOnce({ data: [{ id: 'msg_user', type: 'user' }] })
      .mockResolvedValue({ data: [fakeReply] });

    const result = await runPrompt(stubClient({ list }), FactSchema);

    expect(list).toHaveBeenCalledTimes(3);
    expect(result.reply.messageID).toBe('msg_test');
  });

  it('wraps session creation failures in a descriptive error', async () => {
    const client = stubClient({
      create: async () => {
        throw new Error('connection refused');
      },
    });

    await expect(runPrompt(client, FactSchema)).rejects.toThrow(
      'Failed to create a session: connection refused',
    );
  });

  it('rejects when the server returns a session without an id', async () => {
    const client = stubClient({
      create: async () => ({ ...fakeSession, id: undefined }),
    });

    await expect(runPrompt(client, FactSchema)).rejects.toThrow(
      'Failed to create a session: the server did not return a session id',
    );
  });

  it('gives up when no reply arrives before the polling deadline', async () => {
    vi.useFakeTimers();
    try {
      const list = vi.fn().mockResolvedValue({ data: [] });
      const pending = runPrompt(stubClient({ list }), FactSchema);
      void pending.catch(() => undefined);

      await vi.advanceTimersByTimeAsync(31_000);

      expect(list.mock.calls.length).toBeGreaterThan(1);
      await expect(pending).rejects.toThrow('No response received from OpenCode');
    } finally {
      vi.useRealTimers();
    }
  });
});
