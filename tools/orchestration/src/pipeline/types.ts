import { z } from 'zod';

// ---------------------------------------------------------------------------
// Verdicts: the control signal of the pipeline. Markdown review files are
// the human record; these JSON replies are what the state machine reads.
// ---------------------------------------------------------------------------

export const Severity = z.enum(['blocker', 'major', 'minor', 'nit']);
export type Severity = z.infer<typeof Severity>;

export const FindingSchema = z.object({
  severity: Severity,
  location: z.string().min(1),
  problem: z.string().min(1),
  fix: z.string().optional(),
});
export type Finding = z.infer<typeof FindingSchema>;

export const VerdictSchema = z.object({
  verdict: z.enum(['approve', 'request-changes']),
  findings: z.array(FindingSchema),
});
export type VerdictReply = z.infer<typeof VerdictSchema>;

/** Optional model-authored commit message every mutating stage may include. */
const commitMessage = z.string().min(3).optional();

export const StageReplySchemas = {
  research: z.object({ summary: z.string().min(1), commitMessage }),
  proposal: z.object({ commitMessage }),
  'proposal-final': z.object({
    ambiguities: z.array(z.string()).default([]),
    commitMessage,
  }),
  'test-authoring': z.object({
    testsAdded: z.number().int().min(0).default(0),
    commitMessage,
  }),
  'test-update': z.object({ commitMessage }),
  implement: z.object({ testsNeedUpdate: z.boolean(), commitMessage }),
  fix: z.object({ testsNeedUpdate: z.boolean(), commitMessage }),
  'doc-sync': z.object({ drifted: z.boolean(), commitMessage }),
  review: VerdictSchema,
} as const;
export type StageKind = keyof typeof StageReplySchemas;

// ---------------------------------------------------------------------------
// Persistent pipeline state, stored in docs/designs/NNN-slug/.pipeline.json
// ---------------------------------------------------------------------------

export interface SourceRef {
  kind: 'adr' | 'brief';
  path: string;
}

export interface StageResult {
  /** Stage name; review stages are `<phase>-review:<specialty>`. */
  stage: string;
  round: number;
  sessionID: string;
  agent: string;
  at: number;
  reply: unknown;
  transcript: string;
}

export interface GateLogEntry {
  gate: string;
  round: number;
  proceed: boolean;
  reason: string;
  at: number;
}

export type RunStatus = 'running' | 'done' | 'escalated';

export interface PipelineState {
  version: 1;
  slug: string;
  nnn: string;
  branch: string;
  /**
   * Commit the pipeline branch forked from: the diff base for code review
   * and doc sync (never the repo default branch, which may be far behind).
   */
  baseRef?: string;
  source: SourceRef;
  status: RunStatus;
  /** Last completed stage ('' before the first stage). */
  lastStage: string;
  /** Number of the last executed design/editorial and code review rounds. */
  designRound: number;
  codeRound: number;
  /** Round-budget consumption; incremented on entry to any design/code
   * review round AND on entry to any fix stage (termination guarantee). */
  designBudgetUsed: number;
  codeBudgetUsed: number;
  editorialDone: boolean;
  /** Outcome of the most recent `npm run check` pre-check. */
  testsGreen: boolean;
  /** Set by an implement/fix reply or by a test-path ownership revert. */
  testsNeedUpdate: boolean;
  /** Stage failures consumed so far (step exhaustion, invalid reply). */
  stageFailures: Record<string, number>;
  /** One-shot forced stage for `pipeline run --stage <name>`. */
  forceNext: string | null;
  results: StageResult[];
  gateLog: GateLogEntry[];
  startedAt: number;
  updatedAt: number;
}

export interface LockInfo {
  slug: string;
  pid: number;
  stage: string;
  at: number;
}
