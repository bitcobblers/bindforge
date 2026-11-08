import { existsSync } from 'node:fs';
import { mkdtemp, readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import {
  extractJsonObject,
  loadTemplate,
  normalizeReplyShape,
  renderTemplate,
  templateHash,
  transcriptFileName,
  writeTranscript,
} from '../src/pipeline/prompts.js';
import { DEFAULT_CONFIG, STAGE_NAMES, stageDef } from '../src/pipeline/stages.js';

const PROMPTS_DIR = fileURLToPath(new URL('../prompts/', import.meta.url));

describe('renderTemplate', () => {
  it('substitutes placeholders', () => {
    const out = renderTemplate('review {{designDir}} r{{round}} {{specialty}}', {
      designDir: 'docs/designs/001-x',
      round: 2,
      specialty: 'security',
    });
    expect(out).toBe('review docs/designs/001-x r2 security');
  });

  it('throws on unresolved placeholders', () => {
    expect(() => renderTemplate('need {{missing}}', {})).toThrow(
      /unresolved template placeholder: \{\{missing\}\}/,
    );
  });
});

describe('transcripts', () => {
  it('name reviews by phase-specialty and round', () => {
    expect(
      transcriptFileName({
        stage: 'design-review:security',
        round: 3,
        agent: 'a',
        templateFile: null,
        startedAt: 0,
        endedAt: 0,
        prompt: '',
        rawReply: '',
      }),
    ).toBe('design-review-security-r3.md');
  });

  it('carry a provenance header', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-tr-'));
    const path = await writeTranscript(dir, {
      stage: 'implement',
      round: 0,
      agent: 'code/coder',
      model: { providerID: 'llama', id: 'swift', variant: 'medium' },
      templateFile: 'implement.md',
      templateSha256: 'abc123',
      startedAt: Date.parse('2026-11-07T10:00:00Z'),
      endedAt: Date.parse('2026-11-07T10:05:00Z'),
      prompt: 'do it',
      rawReply: '{"ok":true}',
      gate: { name: 'testsGreen', proceed: true, reason: 'green' },
    });
    const content = await readFile(path, 'utf8');
    expect(content).toContain('- agent: code/coder');
    expect(content).toContain('- model: llama/swift#medium');
    expect(content).toContain('- template: implement.md @ abc123');
    expect(content).toContain('- started: 2026-11-07T10:00:00.000Z');
    expect(content).toContain('- gate: testsGreen: proceed — green');
    expect(content).toContain('do it');
    expect(content).toContain('{"ok":true}');
  });
});

describe('stage registry', () => {
  it('every non-null template exists on disk', async () => {
    for (const name of STAGE_NAMES) {
      const def = stageDef(name, 'correctness', DEFAULT_CONFIG);
      if (def.template) {
        const text = await loadTemplate(PROMPTS_DIR, def.template);
        expect(text.length, `template ${def.template}`).toBeGreaterThan(0);
        expect(existsSync(join(PROMPTS_DIR, def.template)), def.template).toBe(true);
      }
    }
  });

  it('review stages resolve specialty agents and reject missing specialties', () => {
    expect(stageDef('design-review', 'security', DEFAULT_CONFIG).agent).toBe(
      'review/design-security',
    );
    expect(stageDef('code-review', 'testing', DEFAULT_CONFIG).agent).toBe(
      'codereview/code-testing',
    );
    expect(() => stageDef('design-review', undefined, DEFAULT_CONFIG)).toThrow(
      /requires a review specialty/,
    );
  });

  it('steps caps come from the registry, not agent files', () => {
    expect(stageDef('implement', undefined, DEFAULT_CONFIG).steps).toBe(60);
    expect(stageDef('research', undefined, DEFAULT_CONFIG).steps).toBe(DEFAULT_CONFIG.stepsDefault);
  });

  it('template hashes are stable and short', () => {
    expect(templateHash('abc')).toBe(templateHash('abc'));
    expect(templateHash('abc')).toHaveLength(12);
  });
});

describe('extractJsonObject', () => {
  it('parses a bare JSON object', () => {
    expect(extractJsonObject('{"verdict":"approve","findings":[]}')).toEqual({
      verdict: 'approve',
      findings: [],
    });
  });

  it('extracts from a markdown fence', () => {
    const out = extractJsonObject('```json\n{"ok": true}\n```');
    expect(out).toEqual({ ok: true });
  });

  it('extracts the last object from prose drift', () => {
    const out = extractJsonObject('Done. First {"step": 1}. Final reply: {"verdict": "approve"}');
    expect(out).toEqual({ verdict: 'approve' });
  });

  it('handles nested braces and strings containing braces', () => {
    const out = extractJsonObject('note {"a": "{not json}", "b": {"c": 2}} end');
    expect(out).toEqual({ a: '{not json}', b: { c: 2 } });
  });

  it('returns undefined when there is no JSON', () => {
    expect(extractJsonObject('Now let me verify the source files.')).toBeUndefined();
  });
});

describe('normalizeReplyShape', () => {
  it('maps verdict synonyms to the schema enum', () => {
    expect(normalizeReplyShape({ verdict: 'Approved', findings: [] })).toEqual({
      verdict: 'approve',
      findings: [],
    });
    expect(normalizeReplyShape({ verdict: 'Request Changes', findings: [] }).verdict).toBe(
      'request-changes',
    );
  });

  it('maps severity synonyms without touching other fields', () => {
    const out = normalizeReplyShape({
      verdict: 'approve',
      findings: [{ severity: 'CRITICAL', location: 'a.ts:1', problem: 'x' }],
    }) as { findings: { severity: string }[] };
    expect(out.findings[0]?.severity).toBe('blocker');
  });

  it('leaves unknown values untouched for zod to reject', () => {
    expect(normalizeReplyShape({ verdict: 'maybe' }).verdict).toBe('maybe');
  });
});

describe('extractJsonObject empty-object rejection', () => {
  it('skips empty objects from hallucinated tool snippets', () => {
    const out = extractJsonObject('<tool_call>{}</tool_call> real: {"verdict": "approve"}');
    expect(out).toEqual({ verdict: 'approve' });
  });
});
