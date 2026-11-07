import { Buffer } from 'node:buffer';
import { fileURLToPath } from 'node:url';

import { OpenCode } from '@opencode/client';
import type {
  ModelRef,
  OpenCodeClient,
  SessionInfo,
  SessionMessageAssistant,
  TokenUsageInfo,
} from '@opencode/client';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { z } from 'zod';

import { ClientAuth } from './types.js';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

// The v2 client already prefixes every request path with `/api` (e.g. its
// `session.create` uses `/api/session`), so the base URL must point at the
// server root — not at `/api` — to avoid a doubled `/api/api` path.
export const DEFAULT_BASE_URL = 'http://127.0.0.1:4096';
export const SAMPLE_PROMPT =
  'Reply with a single JSON object of the form {"fact": "<one fun fact about TypeScript>"}';

const SESSION_TITLE = 'SDK connection test';

// Polling settings used to wait for the assistant's reply after sending a prompt.
const POLL_INTERVAL_MS = 250;
const POLL_TIMEOUT_MS = 30_000;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

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
export function extractText(parts: readonly ContentPart[]): string {
  return parts
    .filter((part): part is { type: 'text'; text: string } => part.type === 'text')
    .map((part) => part.text)
    .join('');
}

/**
 * Parse a JSON string and validate it against the given Zod schema, returning
 * the validated value of the schema's inferred type.
 */
export function parseContent<T extends z.ZodTypeAny>(schema: T, json: string): z.infer<T> {
  const data: unknown = JSON.parse(json);
  return schema.parse(data);
}

// ---------------------------------------------------------------------------
// Prompt flow
// ---------------------------------------------------------------------------

/**
 * A structured, JSON-serializable summary of a prompt run: the session that
 * handled it, the prompt that was sent, and the assistant's reply. The reply
 * `content` is the reply text deserialized as JSON into type `T`.
 */
export interface PromptResult<T> {
  session: {
    id: string;
    title?: string;
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
 * Create a session, wrapping any failure (including a missing session id) in
 * a descriptive error.
 */
async function createSession(client: OpenCodeClient): Promise<SessionInfo> {
  try {
    const session = await client.session.create({ title: SESSION_TITLE });
    if (!session.id) {
      throw new Error('the server did not return a session id');
    }
    return session;
  } catch (error) {
    throw new Error(`Failed to create a session: ${describeError(error)}`, { cause: error });
  }
}

/**
 * Create a session, send a prompt, and return a structured summary of the run
 * (session, prompt, and the assistant's reply). The reply's text content is
 * deserialized as JSON and validated against `schema`, which determines the
 * reply's `content` type.
 *
 * In the v2 client, `session.prompt` only enqueues the user's message; the
 * assistant's reply is produced asynchronously. We therefore poll
 * `message.list` until a completed assistant message is available.
 */
export async function runPrompt<T extends z.ZodTypeAny>(
  client: OpenCodeClient,
  schema: T,
  prompt: string = SAMPLE_PROMPT,
): Promise<PromptResult<z.infer<T>>> {
  const session = await createSession(client);
  await client.session.prompt({ sessionID: session.id, text: prompt });

  const reply = await waitForReply(client, session.id);
  if (!reply) {
    throw new Error('No response received from OpenCode');
  }

  return {
    session: {
      id: session.id,
      title: session.title,
      projectID: session.projectID,
      model: session.model,
      created: session.time.created,
      updated: session.time.updated,
    },
    prompt,
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

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------

interface CliConfig {
  baseUrl: string;
  username?: string;
  password?: string;
}

/**
 * The JSON shape the sample prompt asks the model to return.
 */
const FunFactSchema = z.object({
  fact: z.string(),
});

/**
 * Read the CLI configuration from environment variables. These values serve as
 * the defaults for the command-line options; explicit flags take precedence.
 */
function configFromEnv(): CliConfig {
  return {
    baseUrl: process.env.OPENCODE_BASE_URL ?? DEFAULT_BASE_URL,
    username: process.env.OPENCODE_SERVER_USERNAME,
    password: process.env.OPENCODE_SERVER_PASSWORD,
  };
}

/**
 * The options accepted on the command line.
 */
interface CliArgs {
  'base-url': string;
  username?: string;
  password?: string;
}

/**
 * Parse the command-line arguments. Option defaults come from the environment
 * variables read by `configFromEnv()`.
 */
function parseArgs(argv: string[]): CliArgs {
  const env = configFromEnv();

  return yargs(hideBin(argv))
    .option('base-url', {
      type: 'string',
      describe: `Base URL of the OpenCode server (default: ${env.baseUrl})`,
      default: env.baseUrl,
    })
    .option('username', {
      type: 'string',
      describe: 'Username for HTTP basic auth (default: $OPENCODE_SERVER_USERNAME)',
      default: env.username,
    })
    .option('password', {
      type: 'string',
      describe: 'Password for HTTP basic auth (default: $OPENCODE_SERVER_PASSWORD)',
      default: env.password,
    })
    .parseSync();
}

/**
 * Entry point: call OpenCode with a sample prompt and print the reply.
 */
export async function main(argv: string[] = process.argv): Promise<void> {
  const args = parseArgs(argv);
  const config: CliConfig = {
    baseUrl: args['base-url'],
    username: args.username,
    password: args.password,
  };
  const client = createClient(config.baseUrl, config);

  console.log(`Calling OpenCode at ${config.baseUrl} with a sample prompt...`);
  if (!config.password) {
    console.log('Note: no password was provided (via --password or OPENCODE_SERVER_PASSWORD).');
  }

  const result = await runPrompt(client, FunFactSchema, SAMPLE_PROMPT);
  console.log(`\nOpenCode replied:\n${JSON.stringify(result, null, 2)}`);
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);

if (isMain) {
  main().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}
