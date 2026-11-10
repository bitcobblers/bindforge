import { printable } from './independence.js';
import { hasSeats, reviewSessionsUsed, seatSummary } from './panel.js';
import type { FunnelPhase } from './panel.js';
import type { ReviewPhase } from './states.js';
import type { PipelineState } from './types.js';

// ---------------------------------------------------------------------------
// Operator-facing escalation report. Reporting only: the gate reads verdict
// JSON, never these files, so the way a round is described here can change
// without touching any ADR-001 review semantic.
//
// Run 1's ESCALATION.md listed every finding of the last round, at every
// severity, with no history. Across 9 rounds that is an unreadable wall: 81%
// of r2-r9 findings re-raised an earlier theme of the same specialty, and the
// report could not tell the operator which ones the last revise had actually
// failed to close versus which ones the closure itself introduced.
// ---------------------------------------------------------------------------

export interface ReportFinding {
  specialty: string;
  severity: string;
  location: string;
  problem: string;
  fix?: string;
  /** First round of this phase that reported the same theme. */
  firstSeen: number;
  /**
   * `new` never appeared in this phase before; `re-raised` was also open in
   * the previous round (the revise did not close it); `previously-closed` was
   * reported, dropped off for at least one round, and came back.
   */
  status: 'new' | 're-raised' | 'previously-closed';
}

const CODE_STAGES = new Set([
  'test-authoring',
  'implement',
  'fix',
  'test-update',
  'code-review',
  'doc-sync',
]);

/**
 * Which review phase an escalation belongs to. The old guess (`lastStage
 * .includes('code')`) mislabelled `doc-sync` and `test-update` escalations as
 * design failures and read findings out of the wrong round.
 */
export function escalationPhase(state: PipelineState): ReviewPhase {
  if (CODE_STAGES.has(state.lastStage) || state.lastStage.startsWith('code-review')) {
    return 'code';
  }
  if (state.lastStage.startsWith('editorial')) {
    return 'editorial';
  }
  return 'design';
}

/** Round of that phase. */
export function escalationRound(state: PipelineState, phase: ReviewPhase): number {
  return phase === 'code' ? state.codeRound : state.designRound;
}

interface RawFinding {
  specialty?: string;
  severity: string;
  location: string;
  problem: string;
  fix?: string;
}

/**
 * Theme identity for reporting. Findings carry no id, so a theme is matched by
 * specialty plus a normalized location: `file:line` and `lines 12-20`
 * suffixes are dropped because every revise renumbers the document, which is
 * exactly why run 1's reviewers kept re-deriving anchors. Approximate by
 * design, and used only to describe a round to the operator.
 */
function themeKey(specialty: string, location: string): string {
  const loc = location
    .toLowerCase()
    .replace(/\b(lines?|line)\s+\d+(\s*[-–]\s*\d+)?\b/g, ' ')
    .replace(/:\d+(-\d+)?/g, ' ')
    .replace(/[^a-z0-9\s-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  return `${specialty.toLowerCase()}|${loc}`;
}

function roundRawFindings(
  state: PipelineState,
  phase: ReviewPhase,
  round: number,
): Map<string, RawFinding> {
  const byTheme = new Map<string, RawFinding>();
  for (const r of state.results) {
    if (!r.stage.startsWith(`${phase}-review:`) || r.round !== round || !r.reply) {
      continue;
    }
    const verdict = r.reply as { findings?: RawFinding[] };
    const specialty = r.stage.split(':')[1] ?? '?';
    for (const f of verdict.findings ?? []) {
      byTheme.set(themeKey(specialty, f.location), { ...f, specialty });
    }
  }
  return byTheme;
}

/**
 * The round's findings, restricted to blocking severities, each labelled with
 * its history in this phase.
 */
export function reportFindings(
  state: PipelineState,
  phase: ReviewPhase,
  round: number,
  blockingSeverities: readonly string[],
): ReportFinding[] {
  const current = roundRawFindings(state, phase, round);
  const previous = round > 1 ? roundRawFindings(state, phase, round - 1) : new Map();
  const earlier = new Map<string, number>();
  for (let r = 1; r < round; r += 1) {
    for (const key of roundRawFindings(state, phase, r).keys()) {
      if (!earlier.has(key)) {
        earlier.set(key, r);
      }
    }
  }
  const out: ReportFinding[] = [];
  for (const [key, f] of current) {
    if (!blockingSeverities.includes(f.severity)) {
      continue;
    }
    const firstSeen = earlier.get(key) ?? round;
    out.push({
      ...f,
      specialty: f.specialty ?? '?',
      firstSeen,
      status: previous.has(key) ? 're-raised' : earlier.has(key) ? 'previously-closed' : 'new',
    });
  }
  const rank = { blocker: 0, major: 1, minor: 2, nit: 3 } as Record<string, number>;
  return out.sort(
    (a, b) =>
      (rank[a.severity] ?? 9) - (rank[b.severity] ?? 9) ||
      a.firstSeen - b.firstSeen ||
      a.specialty.localeCompare(b.specialty),
  );
}

const GROUPS: ReportFinding['status'][] = ['re-raised', 'previously-closed', 'new'];

const GROUP_HEADING: Record<ReportFinding['status'], string> = {
  're-raised': 'Still open from the previous round (the last revise did not close these)',
  'previously-closed':
    'Reported earlier, closed, and now back (the closure introduced or missed the defect)',
  new: 'New this round',
};

export interface ProposalGrowth {
  /** Lines in `02-proposal.md` after the last design stage that reported a size. */
  docLines: number;
  added: number;
  removed: number;
  /** Design stages carrying a size record. */
  stages: number;
  /** Consecutive net-positive design stages at the end of the run. */
  netPositiveStreak: number;
}

/** Design-scope growth over the run, from the size meter the runner records. */
export function proposalGrowth(state: PipelineState): ProposalGrowth | undefined {
  const sized = state.results.filter((r) => typeof r.docLines === 'number');
  if (sized.length === 0) {
    return undefined;
  }
  let netPositiveStreak = 0;
  for (let i = sized.length - 1; i >= 0; i -= 1) {
    const r = sized[i];
    if (r && (r.docLinesAdded ?? 0) > (r.docLinesRemoved ?? 0)) {
      netPositiveStreak += 1;
    } else {
      break;
    }
  }
  const last = sized[sized.length - 1];
  return {
    docLines: last?.docLines ?? 0,
    added: sized.reduce((sum, r) => sum + (r.docLinesAdded ?? 0), 0),
    removed: sized.reduce((sum, r) => sum + (r.docLinesRemoved ?? 0), 0),
    stages: sized.length,
    netPositiveStreak,
  };
}

/**
 * One status/report line for design scope, plus a warning once the proposal has
 * grown on several revise rounds running. Run 2 added 2,295 net lines across 8
 * rounds; the round-end gate only ever reported findings, so growth was not
 * something an operator could act on. Warn, do not gate: a hard size limit
 * buys compressed prose, which is harder to review than long prose.
 */
export function proposalGrowthLine(state: PipelineState): string | null {
  const g = proposalGrowth(state);
  if (!g) {
    return null;
  }
  const warning =
    g.netPositiveStreak >= 3
      ? `  WARNING: the proposal has grown for ${String(g.netPositiveStreak)} design rounds running`
      : '';
  return (
    `proposal:  ${String(g.docLines)} lines ` +
    `(+${String(g.added)}/-${String(g.removed)} across ${String(g.stages)} design stage(s))${warning}`
  );
}

export function escalationMarkdown(
  state: PipelineState,
  phase: ReviewPhase,
  blockingSeverities: readonly string[],
): string {
  const round = escalationRound(state, phase);
  const findings = reportFindings(state, phase, round, blockingSeverities);
  const denied = state.gateLog.filter((g) => !g.proceed).slice(-8);
  const growth = proposalGrowthLine(state);
  const lines = [
    `# ESCALATED: ${state.nnn}-${state.slug}`,
    '',
    `Run stopped after stage \`${state.lastStage}\` in the **${phase}** phase`,
    `(round ${round}, budgets: design ${String(state.designBudgetUsed)}, code ${String(
      state.codeBudgetUsed,
    )}; status: ${state.status}).`,
    `Reviewer sessions spent: design ${String(
      reviewSessionsUsed(state, 'design'),
    )}, code ${String(reviewSessionsUsed(state, 'code'))}.`,
    ...(phase === 'design' || phase === 'code'
      ? hasSeats(state, phase as FunnelPhase)
        ? ['', seatSummary(state, phase as FunnelPhase), '']
        : ['']
      : ['']),
    ...(growth === null ? [] : ['## Proposal size', '', growth, '']),
    `## Open blocking findings from ${phase}-review round ${round}`,
    '',
  ];
  if (findings.length === 0) {
    lines.push('- (no blocking findings recorded; see the gate denials below)');
    lines.push('');
  }
  for (const status of GROUPS) {
    const group = findings.filter((f) => f.status === status);
    if (group.length === 0) {
      continue;
    }
    lines.push(`### ${GROUP_HEADING[status]} (${String(group.length)})`);
    for (const f of group) {
      lines.push(
        `- [${f.severity}] (${f.specialty}, first seen r${String(f.firstSeen)}) ` +
          `${printable(f.location)} — ${printable(f.problem)}`,
      );
    }
    lines.push('');
  }
  const reported = state.results
    .filter((r) => r.stage.startsWith(`${phase}-review:`) && r.round === round && r.reply)
    .reduce((sum, r) => sum + ((r.reply as { findings?: RawFinding[] }).findings ?? []).length, 0);
  if (reported > findings.length) {
    lines.push(
      `_Non-blocking findings are not listed here: ${String(
        reported - findings.length,
      )} minor/nit findings from this round live in the review reports._`,
      '',
    );
  }
  lines.push('## Recent gate denials', '');
  if (denied.length === 0) {
    lines.push('- (none)');
  } else {
    for (const g of denied) {
      lines.push(`- ${g.gate} (r${String(g.round)}): ${printable(g.reason)}`);
    }
  }
  lines.push(
    '',
    'Fix the underlying cause (usually the brief or the proposal), then run:',
    '',
    '`pipeline resume ' + `${state.nnn}-${state.slug}` + '`',
    '',
  );
  return lines.join('\n');
}
