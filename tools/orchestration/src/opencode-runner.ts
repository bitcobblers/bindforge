import { Buffer } from 'node:buffer';

import { OpenCode } from '@opencode/client';
import type {
  ModelRef,
  OpenCodeClient,
  SessionInfo,
  SessionMessageAssistant,
  TokenUsageInfo,
} from '@opencode/client';
import { z } from 'zod';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

// The v2 client already prefixes every request path with `/api` (e.g. its
// `session.create` uses `/api/session`), so the base URL must point at the
// server root — not at `/api` — to avoid a doubled `/api/api` path.
export const DEFAULT_BASE_URL = 'http://127.0.0.1:4096';

const SESSION_TITLE = 'Untitled Session';

// Polling settings used to wait for the assistant's reply after sending a prompt.
const POLL_INTERVAL_MS = 250;
const POLL_TIMEOUT_MS = 30_000;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/**
 * HTTP basic auth credentials for the server connection. The username is
 * optional; without a password, requests are sent unauthenticated.
 */
export interface ClientAuth {
  username?: string;
  password?: string;
}

/**
 * Create an OpenCode client pointed at the given server. When a password is
 * supplied, every request is sent with an HTTP Basic auth header
 * (`Authorization: Basic base64(<username>:<password>)`).
 */
export function createClient(
  baseUrl: string = DEFAULT_BASE_URL,
  auth: ClientAuth = {},
): OpenCodeClient {
  const headers = auth.password
    ? {
        Authorization: `Basic ${Buffer.from(
          `${auth.username ?? 'opencode'}:${auth.password}`,
        ).toString('base64')}`,
      }
    : undefined;

  return OpenCode.make({
    baseUrl,
    ...(headers ? { headers } : {}),
  });
}

// ---------------------------------------------------------------------------
// Message helpers
// ---------------------------------------------------------------------------

/**
 * A minimal structural view of an assistant message content part. This is
 * compatible with the SDK's `SessionMessageAssistant['content']` parts.
 */
type ContentPart = { type: string; text?: string };

/**
 * Concatenate the text content of an assistant message's parts, ignoring
 * non-text parts such as reasoning and tool entries.
 */
function extractText(parts: readonly ContentPart[]): string {
  return parts
    .filter((part): part is { type: 'text'; text: string } => part.type === 'text')
    .map((part) => part.text)
    .join('');
}

/**
 * Parse a JSON string and validate it against the given Zod schema, returning
 * the validated value of the schema's inferred type.
 */
function parseContent<T extends z.ZodType>(schema: T, json: string): z.infer<T> {
  const data: unknown = JSON.parse(json);
  return schema.parse(data);
}

// ---------------------------------------------------------------------------
// Prompt flow
// ---------------------------------------------------------------------------

/**
 * Options for a single prompt run.
 */
export interface PromptOptions {
  /**
   * The agent that should process the prompt, e.g. `build` or `explore`. When
   * omitted the server uses its default agent.
   *
   * The v2 API binds an agent to the session rather than to a single message:
   * `session.create` accepts an `agent`, while `session.prompt` does not. The
   * agent is therefore set when the session is created, and the prompt is then
   * handled by it.
   */
  agent?: string;

  /**
   * The title of the created session. Defaults to `Untitled Session`.
   */
  title?: string;
}

/**
 * A structured, JSON-serializable summary of a prompt run: the session that
 * handled it, the full prompt that was sent (including the generated
 * reply-format instruction), and the assistant's reply. The reply `content`
 * is the reply text deserialized as JSON into type `T`.
 */
export interface PromptResult<T> {
  session: {
    id: string;
    title?: string;
    agent?: string;
    projectID: string;
    model?: ModelRef;
    created: number;
    updated: number;
  };
  prompt: string;
  reply: {
    messageID: string;
    content: T;
    model?: ModelRef;
    finish?: string;
    tokens?: TokenUsageInfo;
    cost?: number;
    created: number;
    completed?: number;
  };
}

/**
 * Return the deserialized reply content of a prompt run: the reply text that
 * `runPrompt` parsed as JSON and validated against its schema, typed as that
 * schema's inferred type `T`.
 */
export function extractReply<T>(result: PromptResult<T>): T {
  return result.reply.content;
}

/**
 * Create a session, optionally with a specific agent and title (defaulting to
 * {@link SESSION_TITLE}), wrapping any failure (including a missing session
 * id) in a descriptive error.
 */
async function createSession(
  client: OpenCodeClient,
  agent?: string,
  title?: string,
): Promise<SessionInfo> {
  try {
    const session = await client.session.create({
      title: title ?? SESSION_TITLE,
      ...(agent ? { agent } : {}),
    });
    if (!session.id) {
      throw new Error('the server did not return a session id');
    }
    return session;
  } catch (error) {
    throw new Error(`Failed to create a session: ${describeError(error)}`, { cause: error });
  }
}

/**
 * Build the reply-format instruction that is appended to every prompt. It is
 * derived from `schema` via Zod's native JSON Schema conversion, so the
 * format the model is asked for always matches the format the reply is
 * validated against.
 */
function replyFormat(schema: z.ZodType): string {
  const jsonSchema = JSON.stringify(z.toJSONSchema(schema));
  return [
    'Respond with exactly one JSON object that validates against this JSON Schema.',
    'Output only the JSON object, with no markdown fences and no other text.',
    `JSON Schema: ${jsonSchema}`,
  ].join('\n');
}

/**
 * Create a session, send a prompt, and return a structured summary of the run
 * (session, prompt, and the assistant's reply). A reply-format instruction
 * generated from `schema` is appended to `prompt` before sending; the reply's
 * text content is then deserialized as JSON and validated against the same
 * `schema`, which determines the reply's `content` type.
 *
 * Pass `options.agent` to have a specific agent process the prompt; see
 * {@link PromptOptions} for why it is applied when the session is created.
 *
 * In the v2 client, `session.prompt` only enqueues the user's message; the
 * assistant's reply is produced asynchronously. We therefore poll
 * `message.list` until a completed assistant message is available.
 */
export async function runPrompt<T extends z.ZodType>(
  client: OpenCodeClient,
  options: PromptOptions,
  schema: T,
  prompt: string,
): Promise<PromptResult<z.infer<T>>> {
  const session = await createSession(client, options.agent, options.title);
  const text = `${prompt}\n\n${replyFormat(schema)}`;
  await client.session.prompt({ sessionID: session.id, text });

  const reply = await waitForReply(client, session.id);
  if (!reply) {
    throw new Error('No response received from OpenCode');
  }

  return {
    session: {
      id: session.id,
      title: session.title,
      agent: session.agent,
      projectID: session.projectID,
      model: session.model,
      created: session.time.created,
      updated: session.time.updated,
    },
    prompt: text,
    reply: {
      messageID: reply.id,
      content: parseContent(schema, extractText(reply.content)),
      model: reply.model,
      finish: reply.finish,
      tokens: reply.tokens,
      cost: reply.cost,
      created: reply.time.created,
      completed: reply.time.completed,
    },
  };
}

/**
 * Poll the session's messages until a completed assistant message is
 * available, returning it (or `null` on timeout).
 */
async function waitForReply(
  client: OpenCodeClient,
  sessionID: string,
): Promise<SessionMessageAssistant | null> {
  const deadline = Date.now() + POLL_TIMEOUT_MS;

  const poll = async (): Promise<SessionMessageAssistant | null> => {
    const reply = await findCompletedReply(client, sessionID);

    if (reply) {
      return reply;
    }
    if (Date.now() >= deadline) {
      return null;
    }

    await sleep(POLL_INTERVAL_MS);
    return poll();
  };

  return poll();
}

/**
 * Look up the session's messages and return the first completed assistant
 * message, or `null` if none is available yet.
 */
async function findCompletedReply(
  client: OpenCodeClient,
  sessionID: string,
): Promise<SessionMessageAssistant | null> {
  const { data } = await client.message.list({ sessionID });

  const assistant = data.find(
    (message): message is SessionMessageAssistant =>
      message.type === 'assistant' && message.finish !== undefined,
  );
  if (!assistant) {
    return null;
  }

  return assistant;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/**
 * Render an SDK error object as a short human-readable message.
 */
function describeError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'object' && error !== null) {
    const e = error as { message?: string; _tag?: string };
    return e.message ?? e._tag ?? JSON.stringify(error);
  }
  return String(error);
}
