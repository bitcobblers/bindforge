import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { latestTextReply, listSessionMessages } from '../src/pipeline/runner.js';

// --------------------------------------------------------------------------
// The round-4 correctness incident: the server returns messages newest-first
// on a default 50-message page, but the drive loop assumed oldest-first and
// unpaged. A reviewer produced nine valid verdict replies; the runner read a
// stale narration turn 60 messages back and exhausted the nudge budget.
// --------------------------------------------------------------------------

function msg(created: number, type: string, extra: object = {}): SessionMessageInfo {
  return { id: `m_${String(created)}`, time: { created }, type, ...extra } as SessionMessageInfo;
}

function assistantWithText(created: number, text: string): SessionMessageInfo {
  return msg(created, 'assistant', { content: [{ type: 'text', text }] });
}

/** Stub server with the real endpoint's semantics: newest-first pages,
 * default page size 50, opaque cursor carrying the offset. */
function stubServer(stored: SessionMessageInfo[]) {
  const calls: Array<{ limit?: number; order?: string; cursor?: string }> = [];
  const client = {
    message: {
      list: async (input: { limit?: number; order?: string; cursor?: string }) => {
        calls.push(input);
        // The real cursor encodes the page order; following it keeps the
        // paging direction without an explicit `order` parameter.
        const order =
          input.cursor !== undefined ? input.cursor.split(':')[0] : (input.order ?? 'desc');
        const sorted =
          order === 'asc'
            ? [...stored].sort((a, b) => a.time.created - b.time.created)
            : [...stored].sort((a, b) => b.time.created - a.time.created);
        const offset = input.cursor === undefined ? 0 : Number(input.cursor.split(':')[1]);
        const size = input.limit ?? 50;
        const end = Math.min(offset + size, sorted.length);
        return {
          data: sorted.slice(offset, end),
          cursor: {
            previous: offset > 0 ? `${order}:${String(Math.max(0, offset - size))}` : null,
            next: end < sorted.length ? `${order}:${String(end)}` : null,
          },
        };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

describe('listSessionMessages', () => {
  it('pages newest-first servers fully, oldest-first', async () => {
    const stored = Array.from({ length: 450 }, (_, k) =>
      msg(k + 1, 'user', { text: `t${String(k)}` }),
    );
    const { client, calls } = stubServer(stored);

    const got = await listSessionMessages(client, 'ses_x');

    expect(got).toHaveLength(450);
    for (let k = 1; k < got.length; k++) {
      expect(got[k].time.created).toBeGreaterThan(got[k - 1].time.created);
    }
    expect(calls[0].order).toBe('asc');
  });

  it('reads past the default 50-message window', async () => {
    const stored = Array.from({ length: 63 }, (_, k) =>
      msg(k + 1, 'user', { text: `t${String(k)}` }),
    );
    const { client } = stubServer(stored);

    const got = await listSessionMessages(client, 'ses_x');

    // The raw default page the old code used would have lost 13 messages.
    expect(got).toHaveLength(63);
  });
});

describe('latestTextReply', () => {
  it('picks the newest text-carrying assistant, not the oldest', () => {
    const stored = [
      assistantWithText(1000, 'Let me verify the file tail and revision log.'),
      msg(1100, 'assistant', { content: [{ type: 'tool' }] }),
      assistantWithText(1200, '{"verdict":"request-changes","findings":[]}'),
      msg(1300, 'assistant', { content: [{ type: 'tool' }] }),
    ];
    const assistants = stored.filter((m) => m.type === 'assistant');

    const reply = latestTextReply(assistants);

    expect((reply?.content ?? []).map((p) => (p as { text?: string }).text).join('')).toBe(
      '{"verdict":"request-changes","findings":[]}',
    );
  });

  it('recovers the round-4 failure: compliant reply inside a long session', async () => {
    // 60 messages: early prose reply, much later the real JSON verdict —
    // exactly the shape the drive loop misread as "narration instead of reply".
    const stored: SessionMessageInfo[] = [
      assistantWithText(1000, 'All r3 finding carriers are present in the body.'),
      msg(1100, 'idle', { outcome: 'succeeded' }),
    ];
    for (let k = 0; k < 56; k++) {
      stored.push(msg(1200 + k * 10, 'assistant', { content: [{ type: 'tool' }] }));
    }
    stored.push(assistantWithText(2000, '{"verdict":"approve","findings":[]}'));
    stored.sort((a, b) => b.time.created - a.time.created); // as the server sends them

    const { client } = stubServer(stored);
    const all = await listSessionMessages(client, 'ses_x');
    const assistants = all.filter((m) => m.type === 'assistant');

    const reply = latestTextReply(assistants);
    const text = (reply?.content ?? [])
      .filter((p): p is { type: 'text'; text: string } => p.type === 'text')
      .map((p) => p.text)
      .join('');
    expect(text).toBe('{"verdict":"approve","findings":[]}');
  });
});
