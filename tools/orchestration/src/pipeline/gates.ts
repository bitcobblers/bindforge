import { spawn } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import type { PipelineConfig } from './stages.js';
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
}

/** Minimal frontmatter + section parsing (no YAML dependency). */
export function parseSource(content: string, kind: 'adr' | 'brief'): ParsedSource {
  const front = /^---\r?\n([\s\S]*?)\r?\n---/.exec(content);
  const status = /status:\s*(\w+)/.exec(front?.[1] ?? '')?.[1] ?? 'none';
  const body = front ? content.slice(front[0].length) : content;
  const section = (name: string): boolean => {
    const re = new RegExp(`^## ${name}[ \\t]*\r?\n([\\s\\S]*?)(?=^## |$)`, 'm');
    const m = re.exec(body);
    return m !== null && m[1]!.trim().length > 0;
  };
  return { kind, status, hasContext: section('Context'), hasDecision: section('Decision') };
}

/**
 * Fail-fast gate over the source document, evaluated before any inference is
 * spent. ADRs must parse, be `proposed`, and carry non-empty Context and
 * Decision sections; briefs must be non-empty markdown.
 */
export async function sourceAdmissible(
  sourcePath: string,
  kind: 'adr' | 'brief',
): Promise<GateResult> {
  let content: string;
  try {
    content = await readFile(sourcePath, 'utf8');
  } catch {
    return deny(`source document not readable: ${sourcePath}`);
  }
  const parsed = parseSource(content, kind);
  if (parsed.kind === 'brief') {
    return content.trim().length > 0 ? allow('brief is non-empty') : deny('brief is empty');
  }
  if (parsed.status !== 'proposed') {
    return deny(`ADR status is "${parsed.status}", expected "proposed"`);
  }
  if (!parsed.hasContext) {
    return deny('ADR has no non-empty "## Context" section');
  }
  if (!parsed.hasDecision) {
    return deny('ADR has no non-empty "## Decision" section');
  }
  return allow('ADR is proposed and complete');
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

/** Run `npm run check` in every package and return overall pass/fail. */
export async function testsGreen(repoDir: string): Promise<boolean> {
  const pkgs = packageDirs(repoDir);
  if (pkgs.length === 0) {
    return false;
  }
  const results = await Promise.all(
    pkgs.map(
      (cwd) =>
        new Promise<boolean>((resolve) => {
          const child = spawn('npm', ['run', 'check'], { cwd, stdio: 'ignore' });
          child.on('error', () => resolve(false));
          child.on('exit', (code) => resolve(code === 0));
        }),
    ),
  );
  return results.every(Boolean);
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
 */
export function stagePathAllowed(stage: string, path: string): boolean {
  const inDocs = /^docs\//.test(path);
  const inTests = TEST_PATH_RE.test(path);
  if (stage === 'implement' || stage === 'fix') {
    // Coder may touch anything except tests; docs allowed for build notes.
    return !inTests;
  }
  if (stage === 'test-authoring' || stage === 'test-update') {
    // Test engineer: tests plus its own design-dir artifacts, never source.
    return inDocs || inTests;
  }
  // All document/review stages (research, proposal*, reviews, doc-sync).
  return inDocs;
}
