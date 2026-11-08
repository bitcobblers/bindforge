import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

// ---------------------------------------------------------------------------
// Stage prompt templates: on-disk, rendered with {{placeholder}}
// substitution; and the per-stage transcript with its provenance header.
// ---------------------------------------------------------------------------

export interface TemplateVars {
  [key: string]: string | number;
}

/** Strict render: every placeholder must be filled, every fill must be used. */
export function renderTemplate(text: string, vars: TemplateVars): string {
  const rendered = text.replace(/\{\{(\w+)\}\}/g, (whole, key: string) =>
    key in vars ? String(vars[key]) : whole,
  );
  const leftover = /\{\{(\w+)\}\}/.exec(rendered);
  if (leftover) {
    throw new Error(`unresolved template placeholder: {{${leftover[1]}}}`);
  }
  return rendered.trim();
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
  agent: string;
  model?: ModelInfo;
  templateFile: string | null;
  templateSha256?: string;
  startedAt: number;
  endedAt: number;
  /** Inference parameters exposed by the endpoint (best-effort). */
  parameters?: Record<string, unknown>;
  prompt: string;
  rawReply: string;
  gate?: { name: string; proceed: boolean; reason: string };
}

export function transcriptFileName(t: TranscriptInput): string {
  return `${t.stage.replace(/:/g, '-')}-r${t.round}.md`;
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
    `# Transcript: ${t.stage} r${t.round}`,
    '',
    '## Provenance',
    `- stage: ${t.stage} (round ${t.round})`,
    `- agent: ${t.agent}`,
    `- model: ${model}`,
    `- template: ${t.templateFile ?? '(built-in)'}${t.templateSha256 ? ` @ ${t.templateSha256}` : ''}`,
    `- started: ${new Date(t.startedAt).toISOString()}`,
    `- ended: ${new Date(t.endedAt).toISOString()}`,
    `- parameters: ${JSON.stringify(t.parameters ?? { note: 'not exposed by endpoint' })}`,
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
