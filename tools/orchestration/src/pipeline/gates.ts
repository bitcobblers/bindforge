import { spawn } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import { flowDef } from './flows.js';
import type { FlowId } from './flows.js';
import type { PipelineConfig } from './stages.js';
import { namesReservedSegment } from './types.js';
import type { Finding, PipelineState, VerdictReply } from './types.js';

// ---------------------------------------------------------------------------
// Gates are code: deterministic guards, round-end verdict aggregation, and
// schema/shape checks. Every gate returns a reason so the log can explain a
// stuck run without re-reading review files.
// ---------------------------------------------------------------------------

export interface GateResult {
  proceed: boolean;
  reason: string;
}

export const allow = (reason: string): GateResult => ({ proceed: true, reason });
export const deny = (reason: string): GateResult => ({ proceed: false, reason });

// --- source gate ------------------------------------------------------------

export interface ParsedSource {
  kind: 'adr' | 'brief';
  status: string;
  hasContext: boolean;
  hasDecision: boolean;
  /**
   * ADR-010: the frontmatter `flow:` key, as spelled. An absent key is `full`;
   * anything else — `Light`, `light `, `sideways`, an empty value — is carried
   * verbatim to the flow registry's lookup, which is the only gate, so a
   * misspelling surfaces as a usage error instead of a silent `full` run.
   */
  flow: FlowId;
  /**
   * The document with its frontmatter block removed — the part a brief *is*.
   * The brief gate judges this, because with ADR-010 a `flow:` key is a legal
   * first line: a document that is only frontmatter carries no brief.
   */
  body: string;
}

/**
 * Minimal frontmatter + section parsing (no YAML dependency).
 *
 * `flow` is deliberately not normalised: the registry lookup decides, so the
 * comparison stays case-sensitive and whitespace-sensitive by construction.
 */
export function parseSource(content: string, kind: 'adr' | 'brief'): ParsedSource {
  const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(content);
  const status = /status:\s*(\w+)/.exec(front?.[1] ?? '')?.[1] ?? 'none';
  const body = front ? content.slice(front[0].length) : content;
  const flow = /^flow:[ \t]*([^\r\n]*)/m.exec(front?.[1] ?? '')?.[1];
  const section = (name: string): boolean => {
    // The end anchor must be end-of-string, not multiline `$`: `$` matches
    // before every newline, so a section whose body starts with a blank line
    // would capture nothing and read as empty.
    const re = new RegExp(`^## ${name}[ \\t]*\r?\n([\\s\\S]*?)(?=^## |$(?![\\r\\n]))`, 'm');
    const m = re.exec(body);
    return m !== null && m[1]!.trim().length > 0;
  };
  return {
    kind,
    status,
    hasContext: section('Context'),
    hasDecision: section('Decision'),
    flow: flow ?? 'full',
    body,
  };
}

/**
 * Fail-fast gate over the source document, evaluated before any inference is
 * spent. ADRs must parse, be `proposed`, and carry non-empty Context and
 * Decision sections; briefs must be non-empty markdown — the standing rules,
 * in both flows: the frontmatter key selects the flow, never the document's
 * shape (Compatibility matrix).
 *
 * The `flow` argument is validated before anything is read: this gate runs on
 * every entry path, so an unreadable flow is a refusal here rather than a
 * silently-full run.
 *
 * `parsed` is the caller's own parse of this document, so an entry path that
 * already read the file reads it once (Selecting the flow: "one frontmatter
 * read per entry, so the value that cleared the pre-lock check is the value
 * stamped and printed"). Omitted, this gate does its own single read.
 */
export async function sourceAdmissible(
  sourcePath: string,
  kind: 'adr' | 'brief',
  flow: FlowId = 'full',
  parsed?: ParsedSource,
): Promise<GateResult> {
  // Validation only: the registry lookup is the whole gate.
  flowDef(flow);
  let doc = parsed;
  if (doc === undefined) {
    try {
      doc = parseSource(await readFile(sourcePath, 'utf8'), kind);
    } catch {
      return deny(`source document not readable: ${sourcePath}`);
    }
  }
  if (doc.kind === 'brief') {
    // The gate judges the brief, not its frontmatter: the raw document still
    // carries the `flow:` block, so a brief that is only frontmatter is empty.
    return doc.body.trim().length > 0 ? allow('brief is non-empty') : deny('brief is empty');
  }
  if (doc.status !== 'proposed') {
    return deny(`ADR status is "${doc.status}", expected "proposed"`);
  }
  if (!doc.hasContext) {
    return deny('ADR has no non-empty "## Context" section');
  }
  if (!doc.hasDecision) {
    return deny('ADR has no non-empty "## Decision" section');
  }
  return allow('ADR is proposed and complete');
}

/**
 * ADR-010 §4 — the light flow's plan line ceiling, a deterministic stop at the
 * same stage-ok/pre-commit point the size meter runs at: the runner hands it
 * the meter's own line count for the run's `02-proposal.md`, because a stage
 * cannot report its own size. The cap is config, so a registry change moves it
 * and no code does; its absence is the full flow's policy of measuring and
 * warning without gating, which is why `planMaxLines` is not in
 * `DEFAULT_CONFIG`.
 */
export function planOutgrewFlow(lines: number, cfg: PipelineConfig): GateResult {
  const cap = cfg.planMaxLines;
  if (cap === undefined) {
    return allow('this flow sets no plan line cap');
  }
  if (lines <= cap) {
    return allow(`plan is ${String(lines)} lines, within the cap of ${String(cap)}`);
  }
  return deny(`plan outgrew the light flow (${String(lines)} lines, cap ${String(cap)})`);
}

// --- round-end aggregation ---------------------------------------------------

export function verdictsFor(
  state: PipelineState,
  phase: string,
  round: number,
  specialties: readonly string[],
): VerdictReply[] {
  const out: VerdictReply[] = [];
  for (const specialty of specialties) {
    // A retry's verdict supersedes the failed attempt it retried: take the
    // LAST valid reply for stage+round, not the first.
    const matches = state.results.filter(
      (r) =>
        r.stage === `${phase}-review:${specialty}` &&
        r.round === round &&
        r.reply !== undefined &&
        r.reply !== null &&
        typeof r.reply === 'object',
    );
    const last = matches.at(-1);
    if (last?.reply) {
      out.push(last.reply as VerdictReply);
    }
  }
  return out;
}

/** The newest valid verdict reply a seat produced in this phase, if any. */
export function lastSeatReply(
  state: PipelineState,
  phase: string,
  specialty: string,
): VerdictReply | undefined {
  const matches = state.results.filter(
    (r) =>
      r.stage === `${phase}-review:${specialty}` &&
      r.reply !== undefined &&
      r.reply !== null &&
      typeof r.reply === 'object',
  );
  return matches.at(-1)?.reply as VerdictReply | undefined;
}

export function openBlockingFindings(
  verdicts: readonly VerdictReply[],
  blocking: readonly string[],
): Finding[] {
  return verdicts.flatMap((v) => v.findings).filter((f) => blocking.includes(f.severity));
}

/**
 * Aggregated review gate: every required specialty must have a verdict for
 * this round, every verdict must approve, and no blocking-severity finding
 * may be open. Missing verdicts are never treated as passes.
 */
export function allReviewsApproved(
  state: PipelineState,
  phase: 'design' | 'editorial' | 'code',
  round: number,
  specialties: readonly string[],
  cfg: PipelineConfig,
): GateResult {
  const verdicts = verdictsFor(state, phase, round, specialties);
  if (verdicts.length < specialties.length) {
    const seen = new Set(
      state.results
        .filter(
          (r) =>
            r.stage.startsWith(`${phase}-review:`) &&
            r.round === round &&
            r.reply !== undefined &&
            r.reply !== null &&
            typeof r.reply === 'object',
        )
        .map((r) => r.stage.split(':')[1]),
    );
    const missing = specialties.filter((s) => !seen.has(s));
    return deny(`reviews missing or invalid: ${missing.join(', ') || '(unparsable verdict)'}`);
  }
  const rejecting = verdicts.filter((v) => v.verdict !== 'approve');
  if (rejecting.length > 0) {
    return deny(`${rejecting.length} reviewer(s) requested changes`);
  }
  const open = openBlockingFindings(verdicts, cfg.blockingSeverities);
  if (open.length > 0) {
    return deny(`${open.length} blocking findings open`);
  }
  return allow(`all ${specialties.length} reviewers approved with no blocking findings`);
}

// --- deterministic gates -----------------------------------------------------

/**
 * Find npm packages in the repo (root, one level of subdirectories, and
 * `tools/*`), so the check gate works in monorepos with no root package.
 */
export function packageDirs(repoDir: string): string[] {
  const dirs: string[] = [];
  const probe = (dir: string): void => {
    if (existsSync(join(dir, 'package.json'))) {
      dirs.push(dir);
    }
  };
  probe(repoDir);
  for (const top of readdirSync(repoDir)) {
    if (top === 'node_modules' || top.startsWith('.')) continue;
    probe(join(repoDir, top));
    if (top === 'tools' || top === 'packages') {
      const parent = join(repoDir, top);
      try {
        for (const child of readdirSync(parent)) probe(join(parent, child));
      } catch {
        /* ignore unreadable dirs */
      }
    }
  }
  return dirs;
}

/** A cargo workspace at the repo root is gated by cargo, not by npm. */
export function hasCargoWorkspace(repoDir: string): boolean {
  return existsSync(join(repoDir, 'Cargo.toml'));
}

const CARGO_GATE: readonly string[][] = [
  ['fmt', '--all', '--', '--check'],
  ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'],
  ['test', '--workspace', '--locked'],
];

/** Run the cargo gate mirroring `repo-practices`: fmt check, clippy, tests. */
async function cargoGreen(repoDir: string): Promise<boolean> {
  for (const args of CARGO_GATE) {
    const ok = await new Promise<boolean>((resolve) => {
      const child = spawn('cargo', args, { cwd: repoDir, stdio: 'ignore' });
      child.on('error', () => resolve(false));
      child.on('exit', (code) => resolve(code === 0));
    });
    if (!ok) {
      return false;
    }
  }
  return true;
}

/**
 * The check gate: `npm run check` in every package, plus the cargo gate when the
 * repo has a cargo workspace. A repo with neither has no gate to pass and fails.
 */
export async function testsGreen(repoDir: string): Promise<boolean> {
  const pkgs = packageDirs(repoDir);
  const cargo = hasCargoWorkspace(repoDir);
  if (pkgs.length === 0 && !cargo) {
    return false;
  }
  const npmResults = await Promise.all(
    pkgs.map(
      (cwd) =>
        new Promise<boolean>((resolve) => {
          const child = spawn('npm', ['run', 'check'], { cwd, stdio: 'ignore' });
          child.on('error', () => resolve(false));
          child.on('exit', (code) => resolve(code === 0));
        }),
    ),
  );
  const npmOk = npmResults.every(Boolean);
  const cargoOk = cargo ? await cargoGreen(repoDir) : true;
  return npmOk && cargoOk;
}

const COMMIT_RE = /^[a-z]+(\([a-z0-9-]+\))?: [a-z0-9].{1,90}$/;

/** Conventional Commits validation for model-authored commit messages. */
export function isSemanticCommit(message: string): boolean {
  return COMMIT_RE.test(message);
}

/**
 * Deterministic ADR status flip: patch the `status:` line in the frontmatter
 * and re-verify by re-parse. Returns the patched content, or null when the
 * input is not a well-formed proposed ADR.
 */
export function flipAdrStatus(content: string): string | null {
  if (!/^---\r?\n[\s\S]*?\r?\n---/.test(content)) {
    return null;
  }
  const patched = content.replace(/^status:[ \t]*proposed[ \t]*$/m, 'status: accepted');
  const ok =
    /^status:[ \t]*accepted[ \t]*$/m.test(patched) && /^status:[ \t]*proposed/m.test(content);
  return ok ? patched : null;
}

/** NNN allocation: zero-padded, shared between adrs and designs. */
export function nextNnn(existing: string[]): string {
  const used = existing
    .map((name) => /^(\d{3})-/.exec(name)?.[1])
    .filter((n): n is string => n !== undefined)
    .map(Number);
  const next = used.length === 0 ? 1 : Math.max(...used) + 1;
  return String(next).padStart(3, '0');
}

// --- stage file ownership ----------------------------------------------------

/** Test paths: owned by the test engineer; reverted out of coder diffs. */
export const TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/;

/**
 * Which paths a stage is allowed to change. Agent-config permissions are only
 * advisory (a model can route around them via subagents), so the runner
 * enforces this matrix deterministically after every successful stage.
 *
 * `flow` is the ADR-010 widening: under `light` the coder owns the tests,
 * because the flow has no test engineer. Exactly `implement` and `fix` gain
 * `TEST_PATH_RE` — the predicate's whole test-path set, co-located `src/x.test.ts`
 * included — so the light coder is never told it owns a file the sweep takes
 * back. Every other rule — including the runner-reserved names, which stay
 * first and win — is the unchanged matrix in both flows.
 *
 * The predicate parses no path (A28): paths reach it only as git plumbing
 * output — repo-root-relative, POSIX separators, the real spelling — and
 * `moves.ts` Rule 1 rejects absolute, backslash and `.`/`..` spellings as
 * usage errors, so no spelling arrives here to be normalized.
 */
export function stagePathAllowed(stage: string, path: string, flow: FlowId = 'full'): boolean {
  // The flow is validated before any rule: an unknown flow must not fall
  // through to `full`, whose rules are the wider ones.
  flowDef(flow);
  // ADR-006 §1: the checkpoint, its `saveState` tmp shape, the escalation
  // marker, and the run lock are runner-reserved. This rule precedes every
  // per-stage rule, so no stage class — document, coder, or test — can own
  // them at any depth or under another case. The worktree and committed
  // copies can then only ever carry runner-authored bytes, and the §4
  // single-flight premise holds inside a run: no stage may write the lock
  // (security r1, finding 4; the runner re-asserts it at stage boundaries).
  if (namesReservedSegment(path)) {
    return false;
  }
  const inDocs = /^docs\//.test(path);
  const inTests = TEST_PATH_RE.test(path);
  if (stage === 'implement' || stage === 'fix') {
    // Coder may touch anything except tests; docs allowed for build notes.
    // Under `light` it owns the tests too — no test engineer exists in the
    // flow — so the reserved names above are its only fence.
    if (flow === 'light') {
      return true;
    }
    return !inTests;
  }
  if (stage === 'test-authoring' || stage === 'test-update') {
    // Test engineer: tests plus its own design-dir artifacts, never source.
    return inDocs || inTests;
  }
  // All document/review stages (plan, research, proposal*, reviews, doc-sync).
  return inDocs;
}
