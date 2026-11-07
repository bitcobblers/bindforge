import { fileURLToPath } from 'node:url';

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { z } from 'zod';

import { DEFAULT_BASE_URL, SAMPLE_PROMPT, createClient, runPrompt } from './opencode-runner.js';

// ---------------------------------------------------------------------------
// CLI entry point
//
// Everything that talks to the OpenCode SDK lives in `opencode-runner.ts`; this
// file only parses options and prints the result.
// ---------------------------------------------------------------------------

interface CliConfig {
  baseUrl: string;
  username?: string;
  password?: string;
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
