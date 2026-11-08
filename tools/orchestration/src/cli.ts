import { fileURLToPath } from 'node:url';

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { z } from 'zod';

import {
  DEFAULT_BASE_URL,
  DEFAULT_REPLY_TIMEOUT_MS,
  MAX_REPLY_TIMEOUT_MS,
  createClient,
  extractReply,
  runPrompt,
} from './opencode-runner.js';

// ---------------------------------------------------------------------------
// CLI entry point
//
// Everything that talks to the OpenCode SDK lives in `opencode-runner.ts`; this
// file only parses options and prints the result.
// ---------------------------------------------------------------------------

/**
 * The validated CLI configuration used by `main()`. Every field is a resolved,
 * validated value; raw environment reads live in {@link EnvValues}.
 */
interface CliConfig {
  baseUrl: string;
  username?: string;
  password?: string;
  agent?: string;
  timeoutSeconds: number;
}

/**
 * The raw environment bag returned by `configFromEnv()`; env values are always
 * strings and are validated only where they need to be (the reply timeout).
 */
interface EnvValues {
  baseUrl: string;
  username?: string;
  password?: string;
  agent?: string;
  replyTimeoutSeconds?: string;
}

/**
 * The options accepted on the command line.
 */
interface CliArgs {
  'base-url': string;
  username?: string;
  password?: string;
  agent?: string;
  timeout?: number;
}

/**
 * The sample prompt sent to OpenCode. `runPrompt` derives the expected reply
 * format from `FunFactSchema` and appends it to this prompt.
 */
const SAMPLE_PROMPT = 'Share one fun fact about TypeScript.';

/**
 * The JSON shape the sample prompt asks the model to return.
 */
const FunFactSchema = z.object({
  fact: z.string(),
});

/**
 * Read the raw environment values that back the command-line options. Plain
 * string options use these directly as yargs defaults; the numeric reply
 * timeout is validated separately by {@link resolveReplyTimeoutSeconds}.
 */
function configFromEnv(): EnvValues {
  return {
    baseUrl: process.env.OPENCODE_BASE_URL ?? DEFAULT_BASE_URL,
    username: process.env.OPENCODE_SERVER_USERNAME,
    password: process.env.OPENCODE_SERVER_PASSWORD,
    agent: process.env.OPENCODE_AGENT,
    replyTimeoutSeconds: process.env.OPENCODE_REPLY_TIMEOUT_SECONDS,
  };
}

/**
 * Resolve the reply timeout in integer seconds: explicit `--timeout` flag
 * wins over `$OPENCODE_REPLY_TIMEOUT_SECONDS`, which wins over the
 * {@link DEFAULT_REPLY_TIMEOUT_MS} default.
 *
 * The environment value is a raw, untrusted string, so it first passes a
 * strict decimal-integer format gate (`^\d{1,5}$`) before any `Number()`
 * conversion — which would otherwise silently reinterpret values like
 * `"0x2d"`, `"1e4"`, `" 45\n"`, or `"+45"`. The gate runs whether or not the
 * flag is present: a broken environment is a configuration error even when a
 * valid flag overrides it. Range validation (`1..3600`, derived from the
 * runner's constants) is a separate check whose message names the offending
 * source. Reflected values are embedded via `JSON.stringify` so control and
 * ANSI bytes can never reach the terminal raw.
 */
export function resolveReplyTimeoutSeconds(
  flagValue: number | undefined,
  envValue: string | undefined,
): number {
  const defaultSeconds = DEFAULT_REPLY_TIMEOUT_MS / 1000;
  const maxSeconds = MAX_REPLY_TIMEOUT_MS / 1000;

  if (envValue !== undefined && !/^\d{1,5}$/.test(envValue)) {
    throw new Error(
      `$OPENCODE_REPLY_TIMEOUT_SECONDS must be an integer number of seconds in ` +
        `1..${maxSeconds} (plain digits only), got ${JSON.stringify(envValue)}`,
    );
  }

  if (flagValue !== undefined) {
    if (!Number.isInteger(flagValue) || flagValue < 1 || flagValue > maxSeconds) {
      throw new Error(
        `--timeout must be an integer number of seconds in 1..${maxSeconds} ` +
          `(unbounded waits are not supported), got ${String(flagValue)}`,
      );
    }
    return flagValue;
  }

  if (envValue !== undefined) {
    const seconds = Number(envValue);
    if (!Number.isInteger(seconds) || seconds < 1 || seconds > maxSeconds) {
      throw new Error(
        `$OPENCODE_REPLY_TIMEOUT_SECONDS must be an integer number of seconds in ` +
          `1..${maxSeconds} (unbounded waits are not supported), got ${JSON.stringify(envValue)}`,
      );
    }
    return seconds;
  }

  return defaultSeconds;
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
    .option('agent', {
      type: 'string',
      describe: 'OpenCode agent that processes the prompt (default: the server default)',
      default: env.agent,
    })
    .option('timeout', {
      type: 'number',
      describe:
        'Seconds to wait for the assistant reply (default: 30; env: $OPENCODE_REPLY_TIMEOUT_SECONDS)',
      // No yargs `default:`: precedence and validation live in
      // resolveReplyTimeoutSeconds(), so a bad env string never slips through
      // as a coerced default (A8).
    })
    .parseSync();
}

/**
 * Entry point: call OpenCode with a sample prompt and print the reply.
 */
export async function main(argv: string[] = process.argv): Promise<void> {
  const args = parseArgs(argv);
  // Config errors must be side-effect free: the resolver runs before
  // createClient() and before any output, so an invalid timeout produces
  // exactly one error line — no client built, no banner printed.
  const timeoutSeconds = resolveReplyTimeoutSeconds(
    args.timeout,
    configFromEnv().replyTimeoutSeconds,
  );
  const config: CliConfig = {
    baseUrl: args['base-url'],
    username: args.username,
    password: args.password,
    agent: args.agent,
    timeoutSeconds,
  };
  const client = createClient(config.baseUrl, {
    username: config.username,
    password: config.password,
  });

  const agentNote = config.agent ? ` using agent "${config.agent}"` : '';
  console.log(
    `Calling OpenCode at ${config.baseUrl} with a sample prompt${agentNote} ` +
      `waiting up to ${config.timeoutSeconds} s for the reply...`,
  );
  if (!config.password) {
    console.log('Note: no password was provided (via --password or OPENCODE_SERVER_PASSWORD).');
  }

  const result = await runPrompt(
    client,
    { agent: config.agent, timeoutMs: config.timeoutSeconds * 1000 },
    FunFactSchema,
    SAMPLE_PROMPT,
  );
  const fact = extractReply(result);
  console.log(`\nOpenCode replied:\n${JSON.stringify(fact, null, 2)}`);
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);

if (isMain) {
  main().catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
}
