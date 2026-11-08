import { Buffer } from 'node:buffer';

import type { OpenCodeClient } from '@opencode/client';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { z } from 'zod';

import { DEFAULT_BASE_URL, createClient, extractReply, runPrompt } from '../src/opencode-runner.js';

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

    const result = await runPrompt(client, {}, FactSchema, 'say hi');

    expect(create).toHaveBeenCalledTimes(1);
    expect(prompt).toHaveBeenCalledTimes(1);
    expect(result.session.id).toBe('ses_test');
    expect(result.session.projectID).toBe('project-test');
    expect(result.reply.messageID).toBe('msg_test');
    expect(result.reply.content).toEqual({ fact: 'tsc is fast' });
    expect(result.reply.finish).toBe('stop');
  });

  it('appends a reply-format instruction derived from the schema', async () => {
    const prompt = vi.fn().mockResolvedValue({});

    const result = await runPrompt(stubClient({ prompt }), {}, FactSchema, 'say hi');

    const [{ text }] = prompt.mock.calls[0] as [{ text: string }];
    const schemaJson = JSON.stringify(z.toJSONSchema(FactSchema));
    expect(text.startsWith('say hi\n\n')).toBe(true);
    expect(text).toContain(schemaJson);
    expect(result.prompt).toBe(text);
  });

  it('extracts the deserialized reply content from a result', async () => {
    const result = await runPrompt(stubClient(), {}, FactSchema, 'say hi');

    expect(extractReply(result)).toEqual({ fact: 'tsc is fast' });
  });

  it('creates the session with the given agent and reports it in the result', async () => {
    const create = vi.fn().mockResolvedValue({ ...fakeSession, agent: 'explore' });
    const client = stubClient({ create });

    const result = await runPrompt(client, { agent: 'explore' }, FactSchema, 'say hi');

    expect(create).toHaveBeenCalledWith({ title: 'Untitled Session', agent: 'explore' });
    expect(result.session.agent).toBe('explore');
  });

  it('creates the session without an agent when none is given', async () => {
    const create = vi.fn().mockResolvedValue(fakeSession);

    const result = await runPrompt(stubClient({ create }), {}, FactSchema, 'say hi');

    expect(create).toHaveBeenCalledWith({ title: 'Untitled Session' });
    expect(result.session.agent).toBeUndefined();
  });

  it('creates the session with the given title', async () => {
    const create = vi.fn().mockResolvedValue(fakeSession);

    await runPrompt(stubClient({ create }), { title: 'my session' }, FactSchema, 'say hi');

    expect(create).toHaveBeenCalledWith({ title: 'my session' });
  });

  it('polls message.list until a completed assistant reply appears', async () => {
    const list = vi
      .fn()
      .mockResolvedValueOnce({ data: [] })
      .mockResolvedValueOnce({ data: [{ id: 'msg_user', type: 'user' }] })
      .mockResolvedValue({ data: [fakeReply] });

    const result = await runPrompt(stubClient({ list }), {}, FactSchema, 'say hi');

    expect(list).toHaveBeenCalledTimes(3);
    expect(result.reply.messageID).toBe('msg_test');
  });

  it('joins text parts and ignores non-text parts when reading the reply', async () => {
    const reply = {
      ...fakeReply,
      content: [
        { type: 'reasoning', text: '{"fact":"ignored"}' },
        { type: 'text', text: '{"fact":' },
        { type: 'tool' },
        { type: 'text', text: '"tsc is fast"}' },
      ],
    };

    const result = await runPrompt(
      stubClient({ list: async () => ({ data: [reply] }) }),
      {},
      FactSchema,
      'say hi',
    );

    expect(result.reply.content).toEqual({ fact: 'tsc is fast' });
  });

  it('rejects with a SyntaxError when the reply is not valid JSON', async () => {
    const reply = { ...fakeReply, content: [{ type: 'text', text: 'not json' }] };

    const pending = runPrompt(
      stubClient({ list: async () => ({ data: [reply] }) }),
      {},
      FactSchema,
      'say hi',
    );

    await expect(pending).rejects.toThrow(SyntaxError);
  });

  it('rejects with a ZodError when the reply does not match the schema', async () => {
    const reply = { ...fakeReply, content: [{ type: 'text', text: '{"fact":42}' }] };

    const pending = runPrompt(
      stubClient({ list: async () => ({ data: [reply] }) }),
      {},
      FactSchema,
      'say hi',
    );

    await expect(pending).rejects.toThrow(z.ZodError);
  });

  it('wraps session creation failures in a descriptive error', async () => {
    const client = stubClient({
      create: async () => {
        throw new Error('connection refused');
      },
    });

    await expect(runPrompt(client, {}, FactSchema, 'say hi')).rejects.toThrow(
      'Failed to create a session: connection refused',
    );
  });

  it('rejects when the server returns a session without an id', async () => {
    const client = stubClient({
      create: async () => ({ ...fakeSession, id: undefined }),
    });

    await expect(runPrompt(client, {}, FactSchema, 'say hi')).rejects.toThrow(
      'Failed to create a session: the server did not return a session id',
    );
  });

  it('gives up when no reply arrives before the polling deadline', async () => {
    vi.useFakeTimers();
    try {
      const list = vi.fn().mockResolvedValue({ data: [] });
      const pending = runPrompt(stubClient({ list }), {}, FactSchema, 'say hi');
      void pending.catch(() => undefined);

      await vi.advanceTimersByTimeAsync(31_000);

      expect(list.mock.calls.length).toBeGreaterThan(1);
      await expect(pending).rejects.toThrow('No response received from OpenCode');
    } finally {
      vi.useRealTimers();
    }
  });
});
