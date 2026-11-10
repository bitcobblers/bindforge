import type { SessionMessageInfo } from '@opencode/client';

// ---------------------------------------------------------------------------
// Review-independence audit (runner-side).
//
// Agent permissions keep an honest agent out of transcripts/ and, for review
// seats, out of every report file — but run 1's lesson is that permissions are
// advisory against a model that routes around them, and the review seats carry
// a `git diff*` shell allow that could be pointed at run records anyway. So
// after each attempt the runner inspects what the session actually DID: the
// completed tool calls and shell commands in its own message list. A violation
// fails the attempt outright — same one-retry-then-escalate budget as any
// other stage failure, with the offending paths named in the gate log.
// Provenance: the live 003 run was watched reading sibling transcripts and
// reports; permissions alone did not hold, so the runner verifies after the
// fact. See the seat ledger's findings field for the injection side.
//
// Scope discipline (run 003's three audit misfires, all fixed here):
// - Only the fields that NAME where a tool looks are scanned. A `write` part
//   carries the file's content in its input, and a test engineer writing
//   tests ABOUT run records had `transcripts/` strings in that content —
//   code about run records is not a read of run records.
// - Only COMPLETED tool parts count. A read the permission system denied
//   never delivered content; failing the attempt for it would punish the
//   stage twice for one defense that worked.
// - A completed glob/list over transcripts/ IS a violation: enumerating run
//   record names is rummaging in the run-records directory, and no stage
//   prompt gives a reason to go there.
//
// What this does NOT prove, and does not claim to (security r1 finding):
// - Matching is on path SEGMENTS, so `.../transcripts`, `.../transcripts;rm`,
//   and a bare `transcripts` argument all match, and a file merely NAMED
//   `transcripts-list.md` does not. But it is still text matching over the
//   recorded command line: a shell built by obfuscation (`$(printf '\x74…')`,
//   split quotes, a script file written first) reaches run records without
//   ever spelling the segment. Proving a negative about an interpreted command
//  is not possible from the message list, so this stays defense-in-depth over
//   the agent-permission fence: it deterministically fails the plain-text
//   accesses that actually happened in runs 1 and 003, nothing more.
// - The audit sees only what the server recorded. A tool that reports no parts
//   is invisible to it.
// ---------------------------------------------------------------------------

export interface IndependenceAuditOptions {
  /**
   * Only consider messages created at or after this wall-clock: the moment
   * this attempt's prompt was sent. A retried attempt must not be failed
   * again for reads an earlier attempt already paid for.
   */
  since: number;
  /** Review-stage sessions may not read ANY report file; mutating stages must, by design. */
  reviewStage: boolean;
  /**
   * The one report path this review seat is allowed to write — the path its
   * prompt handed it. Writing any OTHER path under `reviews/**` corrupts a
   * sibling's run record, which survives the pass's ownership sweep (`docs/**`
   * is in policy) and is exactly the independence this audit exists to keep.
   * Absent (a caller that has no handed report) means "cannot judge writes",
   * and the module says so rather than pretending otherwise.
   */
  ownReportPath?: string;
}

/**
 * The strings that say where this call looked. Known tools expose their
 * access fields only; an unknown tool gets the conservative full scan so a
 * tool that hides paths in nested arguments cannot slip through.
 */
const ACCESS_FIELDS: Record<string, readonly string[]> = {
  read: ['filePath', 'path'],
  list: ['path'],
  // A completed glob's pattern matched real paths — the pattern is the access.
  glob: ['pattern', 'path'],
  // grep reads file contents, so its target fields count; its `pattern` is query text.
  grep: ['path', 'include'],
  shell: ['command', 'workdir'],
  bash: ['command', 'workdir'],
  // Writing is judged on the written-to path only (never the content).
  write: ['filePath', 'path'],
  edit: ['path'],
  patch: ['path', 'filePath'],
};

/** Tools whose completed calls deliver run-record content or names to the model. */
const READONLY_TOOLS = new Set(['read', 'list', 'glob', 'grep']);

/** Tools that place a file at the path they name. */
const WRITE_TOOLS = new Set(['write', 'edit', 'patch']);

/**
 * Whole-path-SEGMENT match for a run-record directory name. A substring test
 * (`includes('transcripts/')`) missed the cases that matter: `…/transcripts`
 * with no trailing slash enumerates the directory itself, and a delimiter
 * other than `/` (`;`, a quote, end of argument) ends the path just as well.
 * Boundaries: start/end of string, `/`, or a shell/JSON delimiter.
 */
const TRANSCRIPTS_SEGMENT = /(?:^|[/\s"'`()=,;|&])transcripts(?:[/\s"'`()=,;|&]|$)/;
const REVIEWS_SEGMENT = /(?:^|[/\s"'`()=,;|&])reviews(?:[/\s"'`()=,;|&]|$)/;

/** `reviews/<path>` captured from the segment on — the report identity of a path. */
const REVIEWS_KEY = /(?:^|[/\s"'`()=,;|&])(reviews\/[^\s"'`()=,;|&]+)/;

/** Violation descriptions; empty means the attempt stayed inside its inputs. */
export function auditSessionIndependence(
  messages: readonly SessionMessageInfo[],
  opts: IndependenceAuditOptions,
): string[] {
  const violations: string[] = [];
  // Keyed from the `reviews` segment on, so an absolute path and the
  // repo-relative path the prompt handed the seat describe the same report.
  const own = opts.ownReportPath === undefined ? undefined : reportKey(opts.ownReportPath);
  for (const m of messages) {
    if ((m.time?.created ?? 0) < opts.since) {
      continue;
    }
    if (m.type === 'shell') {
      if (
        TRANSCRIPTS_SEGMENT.test(m.command) ||
        (opts.reviewStage && REVIEWS_SEGMENT.test(m.command))
      ) {
        violations.push(`shell touched run records: ${brief(m.command)}`);
      }
      continue;
    }
    if (m.type !== 'assistant') {
      continue;
    }
    for (const part of m.content ?? []) {
      if (part.type !== 'tool' || part.state.status !== 'completed') {
        continue; // denied or failed calls delivered nothing; do not double-punish
      }
      const values = accessValues(part.name, part.state.input);
      for (const value of values) {
        if (TRANSCRIPTS_SEGMENT.test(value)) {
          violations.push(`${part.name} accessed run records: ${brief(value)}`);
        } else if (
          opts.reviewStage &&
          READONLY_TOOLS.has(part.name) &&
          REVIEWS_SEGMENT.test(value) &&
          !value.endsWith('.diff') // the editorial diff is the editorial seat's named scope
        ) {
          violations.push(`read a review report: ${brief(value)}`);
        } else if (opts.reviewStage && own !== undefined && WRITE_TOOLS.has(part.name)) {
          // A caller that hands no report path cannot be judged on writes; the
          // audit says nothing rather than guessing.
          const key = reportKey(value);
          if (key !== undefined && key !== own) {
            violations.push(`wrote a review report: ${brief(value)}`);
          }
        }
      }
    }
  }
  return violations;
}

/**
 * The report identity of a path: everything from the `reviews` segment on, or
 * undefined when the path is not under a run's reviews directory at all. Keying
 * this way makes an absolute path and the repo-relative path the prompt handed
 * the seat describe the same report.
 */
function reportKey(value: string): string | undefined {
  const match = REVIEWS_KEY.exec(value.split('\\').join('/'));
  return match?.[1];
}

function accessValues(name: string, input: Record<string, unknown>): string[] {
  const fields = ACCESS_FIELDS[name];
  if (fields === undefined) {
    return stringValues(input);
  }
  const out: string[] = [];
  for (const field of fields) {
    stringValues(input[field], out);
  }
  return out;
}

function stringValues(value: unknown, out: string[] = []): string[] {
  if (typeof value === 'string') {
    out.push(value);
  } else if (Array.isArray(value)) {
    for (const v of value) stringValues(v, out);
  } else if (value !== null && typeof value === 'object') {
    for (const v of Object.values(value)) stringValues(v, out);
  }
  return out;
}

/**
 * Makes a model- or server-authored string safe to put on the operator's
 * terminal and into the checkpoint. Paths and command text are attacker-chosen
 * data here; a control or escape byte (an ANSI cursor move, a BEL, an ESC
 * sequence) would otherwise spoof or corrupt the terminal line, and a newline
 * forges an extra line in the gate log and in `pipeline status`. Same posture
 * as `resolveMaxParallel` in `src/pipeline/cli.ts`, which reflects config
 * values through `JSON.stringify`/`String` for exactly this reason.
 */
export function printable(value: string): string {
  return value.replace(/\p{Cc}/gu, ' ');
}

function brief(s: string): string {
  const safe = printable(s);
  return safe.length > 140 ? `${safe.slice(0, 140)}…` : safe;
}
