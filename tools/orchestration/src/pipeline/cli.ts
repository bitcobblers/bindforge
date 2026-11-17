import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import yargs, { type Argv } from 'yargs';
import { hideBin } from 'yargs/helpers';

import { DEFAULT_BASE_URL, createClient } from '../opencode-runner.js';
import { printable } from './independence.js';
import {
  type PipelineRunResult,
  type ServerConnection,
  renderStatus,
  resumePipeline,
  runPipeline,
} from './runner.js';
import { DEFAULT_CONFIG } from './stages.js';
import { designDirCandidates, loadStateForSlug } from './state-store.js';
import { ESCALATION_FILE } from './types.js';

// ---------------------------------------------------------------------------
// `pipeline` CLI: run / resume / status. Prompts are always files; the CLI
// never carries prose. Exit codes (ADR-020 §3): 0 done, 3 paused at a restart
// boundary (resume from this worktree to carry the run on), 2 escalated (read
// ESCALATION.md), 1 error. `2` keeps its exact pre-020 meaning, so no existing
// driver automates past an escalation by code collision.
// ---------------------------------------------------------------------------

/**
 * The stage-time prompt read root. Exported because it is part of the
 * executable surface the restart boundary is judged by (design 020 §1): the
 * honesty test pins that this root sits under `tools/orchestration/prompts/`,
 * the other prefix the surface rule names.
 */
export const PROMPTS_DIR = join(fileURLToPath(new URL('../../prompts/', import.meta.url)));

/**
 * ADR-020 §3: the exit-code table, in one place — `done → 0`, a standing
 * restart record `→ 3`, any other non-done status `→ 2`, a thrown error `→ 1`
 * (the catch around `main`'s dispatch, so an invocation whose run never started
 * answers the error integer on the same channel as the table's other rows).
 * The order carries the two guarantees §3 makes:
 * the record is checked after `done` so a finished run can never carry the
 * pause's code, and an `escalated` result is answered before the record so an
 * escalation always outranks a pause — a run with a committed `ESCALATION.md`
 * must never exit 3, because the documented driver loop
 * (`while [ $? -eq 3 ]; do pipeline resume …`) would resume past it.
 * `finalizeEscalate` clears any standing record, so that arm is the belt
 * against a record reaching the seam beside an escalation at all
 * (code-review r1: security #1, compatibility #2).
 */
export function exitCodeFor(result: PipelineRunResult): number {
  if (result.status === 'done') {
    return 0;
  }
  if (result.status === 'escalated') {
    return 2;
  }
  if (result.restartRequired !== undefined) {
    return 3;
  }
  return 2;
}

function connectionOptions(cmd: Argv): Argv {
  return cmd
    .option('base-url', {
      type: 'string',
      describe: `Base URL of the OpenCode server (default: ${DEFAULT_BASE_URL})`,
      default: process.env.OPENCODE_BASE_URL ?? DEFAULT_BASE_URL,
    })
    .option('username', {
      type: 'string',
      describe: 'Username for HTTP basic auth',
      default: process.env.OPENCODE_SERVER_USERNAME,
    })
    .option('password', {
      type: 'string',
      describe: 'Password for HTTP basic auth',
      default: process.env.OPENCODE_SERVER_PASSWORD,
    })
    .option('repo', {
      type: 'string',
      describe: 'Repository root (default: enclosing git repository)',
      default: detectRepo(),
    });
}

/** Repo root via git, falling back to the current directory. */
function detectRepo(): string {
  const res = spawnSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' });
  return res.status === 0 ? res.stdout.trim() : process.cwd();
}

/** Source documents are addressed repo-root-relative (or absolute). */
function sourcePath(repo: string, source: string): string {
  return isAbsolute(source) ? source : resolve(repo, source);
}

/**
 * `--max-parallel <N>` with `$OPENCODE_MAX_PARALLEL` as fallback (ADR-003
 * §Configuration), following the same strict fail-closed pattern as
 * `resolveReplyTimeoutSeconds` in `src/cli.ts`: a `^\d{1,5}$` format gate on
 * the env string that runs EVEN when a valid flag overrides it — a broken
 * environment is a configuration error — and a separate range check naming the
 * offending source. Values are reflected through `JSON.stringify` / `String`
 * so no control or ANSI bytes can reach the terminal raw. The runner cannot
 * know the inference server's slot count, so the only hard limit is `>= 1`:
 * `N <= slots` stays operator guidance (`docs/workflows.md`).
 */
export function resolveMaxParallel(
  flagValue: number | undefined,
  envValue: string | undefined,
): number {
  const fallback = DEFAULT_CONFIG.maxParallelReviews ?? 1;

  if (envValue !== undefined && !/^\d{1,5}$/.test(envValue)) {
    throw new Error(
      `$OPENCODE_MAX_PARALLEL must be an integer number of parallel review slots ` +
        `(plain digits only), got ${JSON.stringify(envValue)}`,
    );
  }

  if (flagValue !== undefined) {
    if (!Number.isInteger(flagValue) || flagValue < 1) {
      throw new Error(
        `--max-parallel must be an integer >= 1 (the review lane always runs at ` +
          `least one seat at a time), got ${String(flagValue)}`,
      );
    }
    return flagValue;
  }

  if (envValue !== undefined) {
    const seats = Number(envValue);
    if (!Number.isInteger(seats) || seats < 1) {
      throw new Error(
        `$OPENCODE_MAX_PARALLEL must be an integer >= 1 (the review lane always runs ` +
          `at least one seat at a time), got ${JSON.stringify(envValue)}`,
      );
    }
    return seats;
  }

  return fallback;
}

export async function main(argv: string[] = process.argv): Promise<number> {
  try {
    return await dispatch(argv);
  } catch (error: unknown) {
    // ADR-020 §3, the table's last row: a thrown error is exit `1`, answered on
    // the same channel as every other row so a driver never reads a stop posture
    // it could loop on (`0`/`2`/`3`) for an invocation that never opened a run —
    // no checkpoint, no git repo, an unreadable one, a bad knob. The message is
    // the operator's reason for the integer; `printable` keeps a model- or
    // path-authored byte off the terminal line, the posture every other
    // operator-facing copy takes.
    console.error(printable(error instanceof Error ? error.message : String(error)));
    return 1;
  }
}

/** The parse-and-dispatch half of `main`; every throw is the caller's exit 1. */
async function dispatch(argv: string[]): Promise<number> {
  const args = await connectionOptions(
    yargs(hideBin(argv))
      .command(
        'run <source>',
        'Run the pipeline for an ADR (docs/adrs/NNN-*.md), a task (docs/tasks/T-NNN-*.md), ' +
          'or a brief (docs/prompts/*.md)',
        (cmd) =>
          cmd
            .positional('source', { describe: 'source document path', type: 'string' })
            .option('stage', {
              type: 'string',
              describe: 'Force the next stage (human-directed retry)',
            })
            .option('nnn', {
              type: 'string',
              describe: 'Pin the run number instead of reading it from the source',
            })
            .option('slug', {
              type: 'string',
              describe: 'Pin the run slug, e.g. --slug run2 for a second attempt',
            })
            .option('force', {
              type: 'boolean',
              describe: 'Start even if the run identity is already taken',
              default: false,
            })
            .option('extend-budget', {
              type: 'number',
              describe: 'Grant this many review SESSIONS beyond the cap (recorded)',
            })
            .option('max-parallel', {
              type: 'number',
              describe: 'Review seats in flight at once (default 1; <= your server slots)',
            }),
      )
      .command('resume <slug>', 'Resume a run from its checkpoint', (cmd) =>
        cmd
          .positional('slug', { describe: 'design slug, e.g. 002-my-feature', type: 'string' })
          .option('stage', {
            type: 'string',
            describe: 'Force the next stage or review round (human-directed retry)',
          })
          .option('extend-budget', {
            type: 'number',
            describe: 'Grant this many review SESSIONS beyond the cap (recorded)',
          })
          .option('max-parallel', {
            type: 'number',
            describe: 'Review seats in flight at once (default 1; <= your server slots)',
          }),
      )
      .command('status <slug>', 'Explain where a run is and why it is stuck', (cmd) =>
        cmd.positional('slug', { describe: 'design slug', type: 'string' }),
      )
      .demandCommand(1),
  ).parseAsync();

  const repo = String(args.repo);
  // Resolved before anything with a side effect (client, lock, checkpoint): a
  // bad knob is a usage error, never a half-started run.
  const maxParallelReviews = resolveMaxParallel(
    args.maxParallel as number | undefined,
    process.env.OPENCODE_MAX_PARALLEL,
  );
  const client = createClient(String(args.baseUrl), {
    username: args.username as string | undefined,
    password: args.password as string | undefined,
  });
  // Same connection the client was created with: the child-session walk's
  // direct `GET /session/:id/children` fallback (compat matrix) needs the
  // baseUrl and the basic-auth credentials to reach the server the client
  // cannot reach through its own methods.
  const server: ServerConnection = {
    baseUrl: String(args.baseUrl),
    username: args.username as string | undefined,
    password: args.password as string | undefined,
  };

  if (args.$0 === undefined) {
    return 1;
  }

  switch (args._[0]) {
    case 'status': {
      const slug = String(args.slug);
      // ADR-023 (023-C3): the slug can name a run in either design root; the
      // shared lookup searches both and refuses an ambiguous slug.
      let loaded;
      try {
        loaded = await loadStateForSlug(repo, slug);
      } catch (error) {
        console.error(printable(error instanceof Error ? error.message : String(error)));
        return 1;
      }
      if (!loaded) {
        console.error(
          `no pipeline checkpoint for ${slug} (searched ${designDirCandidates(slug).join(' and ')})`,
        );
        return 1;
      }
      // ADR-006 §3: the marker sits alongside the loaded state, and status
      // prints the split-record line when they disagree.
      const marker = existsSync(join(repo, loaded.dir, ESCALATION_FILE));
      console.log(renderStatus(loaded.state, marker));
      return 0;
    }
    case 'run': {
      const result = await runPipeline({
        repoDir: repo,
        sourcePath: sourcePath(repo, String(args.source)),
        promptsDir: PROMPTS_DIR,
        client,
        server,
        forceStage: args.stage as string | undefined,
        pin: { nnn: args.nnn as string | undefined, slug: args.slug as string | undefined },
        force: Boolean(args.force),
        extendBudget: args.extendBudget as number | undefined,
        cfg: { ...DEFAULT_CONFIG, maxParallelReviews },
      });
      return exitCodeFor(result);
    }
    case 'resume': {
      const result = await resumePipeline({
        repoDir: repo,
        slug: String(args.slug),
        promptsDir: PROMPTS_DIR,
        client,
        server,
        forceStage: args.stage as string | undefined,
        extendBudget: args.extendBudget as number | undefined,
        cfg: { ...DEFAULT_CONFIG, maxParallelReviews },
      });
      return exitCodeFor(result);
    }
    default:
      return 1;
  }
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);

if (isMain) {
  main().then(
    (code) => process.exit(code),
    (error: unknown) => {
      console.error(error instanceof Error ? error.message : error);
      process.exit(1);
    },
  );
}
