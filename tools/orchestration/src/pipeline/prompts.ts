import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import type { Flow } from './flows.js';

// ---------------------------------------------------------------------------
// Stage prompt templates: on-disk, rendered with {{placeholder}}
// substitution; and the per-stage transcript with its provenance header.
// ---------------------------------------------------------------------------

export interface TemplateVars {
  [key: string]: string | number;
}

/**
 * The `{{testOwnership}}` line (ADR-010 §4): the one place the test-ownership
 * rule is written, rendered from the flow so a light prompt cannot inherit the
 * full flow's sentence and vice versa.
 */
export const TEST_OWNERSHIP_FULL =
  'Never edit test files: a test engineer owns them. If they need updating, reply with testsNeedUpdate: true.';

export const TEST_OWNERSHIP_LIGHT =
  'This is the light flow: you own the test files, and no test engineer exists in this flow.';

/**
 * The line a flow's coder prompts carry in place of the hard-coded rule. The
 * parameter is a `Flow`, not an open `string`: any other value would silently
 * render the full flow's sentence, which is the failure the variable exists to
 * remove.
 */
export function testOwnershipLine(flow: Flow): string {
  return flow === 'light' ? TEST_OWNERSHIP_LIGHT : TEST_OWNERSHIP_FULL;
}

/**
 * What a prompt shows when the run has no gate record yet: the first
 * `implement`, or a run resumed from a checkpoint written before ADR-016
 * (Decision 2). A template that asks for the block still renders a section.
 */
export const NO_GATE_EVIDENCE = '(no gate record for this run yet)';

/**
 * Values a template may ask for that are NOT flow config: the standing rule is
 * the full flow's, so a caller that renders a coder prompt without naming a
 * flow gets the rule every coder prompt carried before ADR-010. The runner
 * always passes the flow's line, so this default is the fallback, never the
 * authority. `gateEvidence` sits here for the same reason: the runner renders
 * it from the run's record, and a caller that assembles its own vars gets the
 * no-record line rather than a failed render.
 */
const TEMPLATE_DEFAULTS: TemplateVars = {
  testOwnership: TEST_OWNERSHIP_FULL,
  gateEvidence: NO_GATE_EVIDENCE,
};

/**
 * Strict render: every placeholder in the TEMPLATE must be filled.
 *
 * The check scans the template text, never the rendered result. Vars carry
 * model-authored content — a review seat's own previous findings are injected
 * as `{{prevFindings}}`, `{{scope}}` quotes git ranges, `{{reviewFiles}}`
 * lists paths a model wrote — and content that quotes prompt syntax carries
 * `{{...}}` bytes legitimately. Run 010 died on exactly that: its design
 * specifies the light flow's plan template, a seat cited `{{planMaxLines}}`
 * from it in a finding, and the post-substitution scan read the quotation as
 * an unfilled placeholder and threw on the next seat's prompt.
 */
export function renderTemplate(text: string, vars: TemplateVars): string {
  const filled: TemplateVars = { ...TEMPLATE_DEFAULTS, ...vars };
  for (const match of text.matchAll(/\{\{(\w+)\}\}/g)) {
    const key = match[1] as string;
    if (!(key in filled)) {
      throw new Error(`unresolved template placeholder: {{${key}}}`);
    }
  }
  return text.replace(/\{\{(\w+)\}\}/g, (_whole, key: string) => String(filled[key])).trim();
}

export async function loadTemplate(promptsDir: string, name: string): Promise<string> {
  return readFile(join(promptsDir, name), 'utf8');
}

export function templateHash(text: string): string {
  return createHash('sha256').update(text).digest('hex').slice(0, 12);
}

export interface ModelInfo {
  providerID?: string;
  id?: string;
  variant?: string;
}

export interface TranscriptInput {
  /** Stage key; review stages use `<phase>-review:<specialty>`. */
  stage: string;
  round: number;
  /** Funnel pass within the round (ADR-002); absent for single-pass stages. */
  pass?: number;
  agent: string;
  model?: ModelInfo;
  templateFile: string | null;
  templateSha256?: string;
  startedAt: number;
  endedAt: number;
  /** Inference parameters exposed by the endpoint (best-effort). */
  parameters?: Record<string, unknown>;
  /**
   * ADR-016 Decision 4: the attempt's token usage, summed per key over the
   * session's assistant messages (`sessionTokens`). Absent when no message
   * carried usage — the transcript then prints no `- tokens:` line, and the
   * `- parameters:` line keeps its old meaning of endpoint exposure.
   */
  tokens?: Record<string, number>;
  prompt: string;
  rawReply: string;
  gate?: { name: string; proceed: boolean; reason: string };
}

/** `rN` or `rNcM` when the stage ran inside a review pass of round N. */
export function roundLabel(t: Pick<TranscriptInput, 'round' | 'pass'>): string {
  return `r${String(t.round)}${t.pass === undefined ? '' : `c${String(t.pass)}`}`;
}

/** Adds one numeric usage field to the per-key totals; non-numeric is skipped. */
function addUsage(total: Record<string, number>, key: string, value: unknown): boolean {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    return false;
  }
  total[key] = (total[key] ?? 0) + value;
  return true;
}

/**
 * ADR-016 Decision 4 — the token usage of a stage's whole session. A stage is
 * a tool loop of many assistant messages, so the reply message's usage is one
 * turn's, not the session's; summing every assistant message's `tokens` over
 * the attempt's message list answers what Decision 4 names. The sums are
 * per key, because the SDK's usage shape is not enumerated here (A13): a field
 * the endpoint adds is counted, a non-numeric one is skipped. A list with no
 * usage anywhere returns undefined — the transcript then prints no line
 * rather than a row of zeros.
 *
 * One level of grouping is flattened into dotted keys rather than skipped. The
 * pinned client types `tokens` as `TokenUsageInfo = { input, output, reasoning,
 * cache: { read, write } }`, so the two cached-usage terms sit under `cache`:
 * summing only the top-level keys would drop them silently, and `cache.read` is
 * the largest term for a session whose prefix is re-prefilled — the very
 * quantity Decision 4 exists to make checkable. A nested object therefore
 * contributes `cache.read`/`cache.write`; nesting deeper than one level, and
 * anything non-numeric, is still skipped, so an unexpected shape can only add a
 * key, never lose one.
 */
export function sessionTokens(messages: readonly unknown[]): Record<string, number> | undefined {
  const total: Record<string, number> = {};
  let seen = false;
  for (const message of messages) {
    if (message === null || typeof message !== 'object') {
      continue;
    }
    const { type, tokens } = message as { type?: unknown; tokens?: unknown };
    if (type !== 'assistant' || tokens === null || typeof tokens !== 'object') {
      continue;
    }
    for (const [key, value] of Object.entries(tokens as Record<string, unknown>)) {
      if (addUsage(total, key, value)) {
        seen = true;
        continue;
      }
      if (value === null || typeof value !== 'object' || Array.isArray(value)) {
        continue;
      }
      for (const [inner, innerValue] of Object.entries(value as Record<string, unknown>)) {
        if (addUsage(total, `${key}.${inner}`, innerValue)) {
          seen = true;
        }
      }
    }
  }
  return seen ? total : undefined;
}

export function transcriptFileName(t: TranscriptInput): string {
  return `${t.stage.replace(/:/g, '-')}-${roundLabel(t)}.md`;
}

/** Write the transcript under <designDir>/transcripts/ and return its path. */
export async function writeTranscript(designDir: string, t: TranscriptInput): Promise<string> {
  const dir = join(designDir, 'transcripts');
  await mkdir(dir, { recursive: true });
  const file = join(dir, transcriptFileName(t));

  const model = t.model?.id
    ? `${t.model.providerID ?? '?'}/${t.model.id}${t.model.variant ? `#${t.model.variant}` : ''}`
    : 'unknown';
  const gate = t.gate
    ? `${t.gate.name}: ${t.gate.proceed ? 'proceed' : 'deny'} — ${t.gate.reason}`
    : '(no gate)';

  const body = [
    `# Transcript: ${t.stage} ${roundLabel(t)}`,
    '',
    '## Provenance',
    `- stage: ${t.stage} (round ${roundLabel(t)})`,
    `- agent: ${t.agent}`,
    `- model: ${model}`,
    `- template: ${t.templateFile ?? '(built-in)'}${t.templateSha256 ? ` @ ${t.templateSha256}` : ''}`,
    `- started: ${new Date(t.startedAt).toISOString()}`,
    `- ended: ${new Date(t.endedAt).toISOString()}`,
    `- parameters: ${JSON.stringify(t.parameters ?? { note: 'not exposed by endpoint' })}`,
    // ADR-016 Decision 4: one measured line per stage, beside the parameters
    // line it must never be confused with.
    ...(t.tokens === undefined ? [] : [`- tokens: ${JSON.stringify(t.tokens)}`]),
    `- gate: ${gate}`,
    '',
    '## Prompt',
    '```text',
    t.prompt,
    '```',
    '',
    '## Raw reply',
    '```text',
    t.rawReply,
    '```',
    '',
  ].join('\n');

  await writeFile(file, body, 'utf8');
  return file;
}

/**
 * Best-effort reply extraction: strict parse first, then the last balanced
 * JSON object embedded in the text (fences, prose wrappers). Returns
 * undefined only when no JSON object can be recovered.
 */
export function extractJsonObject(text: string): unknown | undefined {
  const trimmed = text.trim();
  try {
    return JSON.parse(trimmed) as unknown;
  } catch {
    /* fall through to extraction */
  }
  const candidates: string[] = [];
  let i = text.indexOf('{');
  while (i >= 0) {
    let depth = 0;
    let inStr = false;
    let esc = false;
    let close = -1;
    for (let j = i; j < text.length; j++) {
      const c = text[j] as string;
      if (inStr) {
        if (esc) esc = false;
        else if (c === '\\') esc = true;
        else if (c === '"') inStr = false;
        continue;
      }
      if (c === '"') inStr = true;
      else if (c === '{') depth++;
      else if (c === '}') {
        depth--;
        if (depth === 0) {
          close = j;
          break;
        }
      }
    }
    if (close < 0) {
      break; // no balanced object from here on
    }
    candidates.push(text.slice(i, close + 1));
    // Non-overlapping: inner braces of a balanced candidate are never
    // treated as candidate starts.
    i = text.indexOf('{', close + 1);
  }
  // The LAST balanced object: replies append the JSON after any prose, and
  // zod validation rejects stray earlier objects that happen to parse.
  for (const c of candidates.reverse()) {
    try {
      const value = JSON.parse(c) as unknown;
      // An empty object is never a stage reply (every schema has fields);
      // hallucinated tool-call snippets like `({})` must not shadow the
      // real reply candidates.
      if (
        typeof value === 'object' &&
        value !== null &&
        !Array.isArray(value) &&
        Object.keys(value).length === 0
      ) {
        continue;
      }
      return value;
    } catch {
      /* try the next candidate */
    }
  }
  return undefined;
}

/**
 * Normalize control-signal enums before zod validation: small models drift
 * on spelling ("approved", "critical") while the MEANING is unambiguous.
 * Only verdict/severity strings are touched; everything else passes through.
 */
const VERDICT_SYNONYMS: Record<string, string> = {
  approve: 'approve',
  approved: 'approve',
  accept: 'approve',
  accepted: 'approve',
  pass: 'approve',
  'request-changes': 'request-changes',
  request_changes: 'request-changes',
  'request changes': 'request-changes',
  'changes-requested': 'request-changes',
  'changes requested': 'request-changes',
  needs_changes: 'request-changes',
  'needs-changes': 'request-changes',
  reject: 'request-changes',
  rejected: 'request-changes',
  fail: 'request-changes',
};

const SEVERITY_SYNONYMS: Record<string, string> = {
  blocker: 'blocker',
  blocking: 'blocker',
  critical: 'blocker',
  high: 'blocker',
  major: 'major',
  important: 'major',
  medium: 'minor',
  moderate: 'minor',
  minor: 'minor',
  should_fix: 'minor',
  'should-fix': 'minor',
  nit: 'nit',
  nits: 'nit',
  low: 'nit',
  info: 'nit',
  cosmetic: 'nit',
};

function normalizeEnum(value: unknown, synonyms: Record<string, string>): unknown {
  if (typeof value !== 'string') return value;
  const key = value
    .trim()
    .toLowerCase()
    .replace(/\s+/g, value.trim().includes(' ') ? ' ' : '-');
  return synonyms[key] ?? synonyms[value.trim().toLowerCase().replace(/\s+/g, '-')] ?? value;
}

export function normalizeReplyShape(value: unknown): unknown {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return value;
  const obj = { ...(value as Record<string, unknown>) };
  if ('verdict' in obj) obj['verdict'] = normalizeEnum(obj['verdict'], VERDICT_SYNONYMS);
  if (Array.isArray(obj['findings'])) {
    obj['findings'] = (obj['findings'] as unknown[]).map((f) => {
      if (typeof f !== 'object' || f === null || Array.isArray(f)) return f;
      const fo = { ...(f as Record<string, unknown>) };
      if ('severity' in fo) fo['severity'] = normalizeEnum(fo['severity'], SEVERITY_SYNONYMS);
      return fo;
    });
  }
  return obj;
}
