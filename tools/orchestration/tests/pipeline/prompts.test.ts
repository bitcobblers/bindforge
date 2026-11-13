import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, readdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { configFor } from '../../src/pipeline/flows.js';
import {
  extractJsonObject,
  loadTemplate,
  normalizeReplyShape,
  renderTemplate,
  templateHash,
  transcriptFileName,
  writeTranscript,
} from '../../src/pipeline/prompts.js';
import { DEFAULT_CONFIG, STAGE_NAMES, stageDef } from '../../src/pipeline/stages.js';
import { StageReplySchemas } from '../../src/pipeline/types.js';

const PROMPTS_DIR = fileURLToPath(new URL('../../prompts/', import.meta.url));

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

  it('renders substituted content that carries placeholder syntax', () => {
    // Run 010: the design under review specifies a prompt template, so a
    // seat's own findings — injected as `{{prevFindings}}` — quoted
    // `{{planMaxLines}}`. Substituted content is data, never syntax.
    const out = renderTemplate('your findings:\n{{prevFindings}}', {
      prevFindings: 'line 259 interpolates {{planMaxLines}} from the registry',
    });
    expect(out).toBe('your findings:\nline 259 interpolates {{planMaxLines}} from the registry');
  });

  it('reports the first unfilled placeholder of the template itself', () => {
    expect(() =>
      renderTemplate('a {{reportPath}} for {{specialty}} at {{round}}', {
        reportPath: 'reviews/design/security-r1c1.md',
        round: 1,
      }),
    ).toThrow(/unresolved template placeholder: \{\{specialty\}\}/);
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
      const def = stageDef(name, 'correctness');
      if (def.template) {
        const text = await loadTemplate(PROMPTS_DIR, def.template);
        expect(text.length, `template ${def.template}`).toBeGreaterThan(0);
        expect(existsSync(join(PROMPTS_DIR, def.template)), def.template).toBe(true);
      }
    }
  });

  it('review stages resolve specialty agents and reject missing specialties', () => {
    expect(stageDef('design-review', 'security').agent).toBe('review/design-security');
    expect(stageDef('code-review', 'testing').agent).toBe('codereview/code-testing');
    expect(() => stageDef('design-review', undefined)).toThrow(/requires a review specialty/);
  });

  it('revise reply admits reasoned non-changes', () => {
    const r = StageReplySchemas.revise.safeParse({
      changed: false,
      summary: 'finding A is already resolved in section 4',
    });
    expect(r.success).toBe(true);
    expect(stageDef('proposal-revise', undefined).kind).toBe('revise');
  });

  it('stages carry no step cap: the stage timeout is the only bound', () => {
    expect(DEFAULT_CONFIG).not.toHaveProperty('stepsDefault');
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

describe('run-2 stage guidance', () => {
  // Run 1 churned because the instructions never told reviewers how to
  // anchor findings in a document that renumbers every round, or where the
  // Revision log and prior-round reports live (harness-audit T1/T2/T4).
  // These pins keep that guidance from being silently reverted again.
  it('tells design reviewers the round, their own injected findings, and the Revision log', async () => {
    const text = renderTemplate(await loadTemplate(PROMPTS_DIR, 'design-review.md'), {
      designDir: 'docs/designs/002-x',
      sourcePath: 'docs/adrs/002-x.md',
      round: 9,
      specialty: 'correctness',
      scope: 'the whole document',
      passNote: 'dissent pass c2: the last revision was written to answer YOUR findings',
      reportPath: 'docs/designs/002-x/reviews/design/correctness-r9c2.md',
      reviewFiles: '',
      prevFindings: JSON.stringify([
        { severity: 'major', location: 'Decision', problem: 'still broken' },
      ]),
    });
    expect(text).toContain('round 9');
    expect(text).toContain('correctness-r9c2.md');
    expect(text).toContain('dissent pass c2');
    expect(text).toContain('still broken');
    expect(text).toContain('Revision log');
    // Review independence: a seat never reads another seat's report or any
    // transcript; its own previous findings travel through the prompt.
    expect(text).toContain('Review independently');
    expect(text).toContain('Do not open review reports or transcripts');
    // proves: Test strategy #8 — Design 007 §4's bright line: walking a
    // forbidden directory is already opening it, and every path a seat needs
    // is named for it.
    expect(text).toContain(
      'Listing, globbing, or grepping those directories counts as opening them',
    );
    expect(text).toContain('never browse to verify one');
    expect(text).not.toContain('Earlier reports for every specialty');
    // T4: "Apply only this change set: the whole document" is contradictory.
    expect(text).not.toContain('Apply only this change set');
  });

  it('anchors review findings by section + quote, not file:line', async () => {
    const text = await readFile(
      fileURLToPath(
        new URL('../../../../.opencode/skills/design-review/SKILL.md', import.meta.url),
      ),
      'utf8',
    );
    expect(text).toContain('section name plus a short verbatim');
    expect(text).toContain('Confirm a location once');
    expect(text).toContain('Revision log');
    expect(text).toContain('re-raised');
    expect(text).toContain('regression');
  });

  it('caps proposal-revise at slice reading', async () => {
    const text = renderTemplate(await loadTemplate(PROMPTS_DIR, 'proposal-revise.md'), {
      designDir: 'docs/designs/002-x',
      round: 6,
      specialty: 'correctness',
      scope: 'the whole document',
      reviewFiles: 'docs/designs/002-x/reviews/design/correctness-r5.md',
    });
    expect(text).toContain('Read in slices');
    expect(text).toContain('never re-read a passage you are not going to change');
    expect(text).toContain('anchored by section name (never a');
  });

  // Run 002-run2 answered every finding by adding text: 8 revise rounds took the
  // proposal from 401 lines to 2,696. Revision guidance must offer the other moves.
  it('lets a revise resolve a finding by deleting or deferring', async () => {
    const text = renderTemplate(await loadTemplate(PROMPTS_DIR, 'proposal-revise.md'), {
      designDir: 'docs/designs/002-x',
      round: 6,
      specialty: 'correctness',
      scope: 'the whole document',
      reviewFiles: 'docs/designs/002-x/reviews/design/correctness-r5.md',
    });
    expect(text).toContain('Deleting content is a');
    expect(text).toContain('net-zero or net-negative line count');
    expect(text).toContain('delete the passage or sub-system');

    const skill = await readFile(
      fileURLToPath(
        new URL('../../../../.opencode/skills/design-proposal/SKILL.md', import.meta.url),
      ),
      'utf8',
    );
    expect(skill).toContain('addition is the last resort');
  });

  it('names a deferred-scope boundary the reviewers respect', async () => {
    const template = await readFile(
      fileURLToPath(
        new URL(
          '../../../../.opencode/skills/design-proposal/references/template.md',
          import.meta.url,
        ),
      ),
      'utf8',
    );
    expect(template).toContain('## Deferred / out of scope');
    expect(template).toContain('## Revision log');
    expect(template.indexOf('## Deferred / out of scope')).toBeLessThan(
      template.indexOf('## Revision log'),
    );

    const skill = await readFile(
      fileURLToPath(
        new URL('../../../../.opencode/skills/design-review/SKILL.md', import.meta.url),
      ),
      'utf8',
    );
    expect(skill).toContain('A deferral consistent with the source document is');
    expect(skill).toContain('the smallest change that resolves the finding');

    // The reviewer prompt must be able to name the source document.
    const review = renderTemplate(await loadTemplate(PROMPTS_DIR, 'design-review.md'), {
      designDir: 'docs/designs/002-x',
      prevFindings: '(none)',
      sourcePath: 'docs/adrs/002-x.md',
      round: 3,
      specialty: 'complexity',
      scope: 'the whole document',
      passNote: 'full-panel pass c0',
      reportPath: 'docs/designs/002-x/reviews/design/complexity-r3c0.md',
      reviewFiles: '',
    });
    expect(review).toContain('re-open one only if it contradicts docs/adrs/002-x.md');
  });

  it('records the revision log as a ledger that replaces superseded entries', async () => {
    const skill = await readFile(
      fileURLToPath(
        new URL('../../../../.opencode/skills/design-proposal/SKILL.md', import.meta.url),
      ),
      'utf8',
    );
    expect(skill).toContain('never stack a');
    expect(skill).not.toContain('then append to the Revision log');
  });
});

describe('moves guidance in stage prompts', () => {
  // proves: Test strategy #7 — the four prompts the move-capable stages run
  // must carry the moves sentence from the proposal's Prompts section: "to move
  // or rename a file, do not touch the paths yourself — declare it in `moves`,
  // and the runner performs it". Fragments are pinned, not exact punctuation.
  for (const template of ['implement.md', 'fix.md', 'fix-tests.md', 'test-update.md']) {
    it(`${template} routes file moves through the moves reply field`, async () => {
      const text = await loadTemplate(PROMPTS_DIR, template);
      expect(text, template).toMatch(/move or rename a file/i);
      expect(text, template).toMatch(/do not touch the paths yourself/i);
      expect(text, template).toMatch(/declare it in `?moves`?/i);
      expect(text, template).toMatch(/runner performs it/i);
    });
  }
});

describe('test-tree convention', () => {
  function gitRoot(): string {
    return execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
  }

  // proves: Test strategy #6 — both convention surfaces that encoded the flat
  // form (A13: repo-practices SKILL.md and test-strategy SKILL.md) must state
  // the recursive glob after the move.
  it('states the recursive test glob in the amended skill files', async () => {
    for (const skill of ['repo-practices', 'test-strategy']) {
      const path = join(gitRoot(), '.opencode', 'skills', skill, 'SKILL.md');
      const text = await readFile(path, 'utf8');
      expect(text, skill).toContain('tests/**/*.test.ts');
    }
  });

  // proves: Test strategy #6 — the flat convention statement must be gone from
  // every surface stage prompts and skills read, so future stages write the
  // mirrored layout. In-file fixture strings (Deferred) live in test sources,
  // not in these two directories, and stay out of scope.
  it('has no remaining flat tests/*.test.ts convention statement', async () => {
    const offenders: string[] = [];
    for (const dirRel of [join('.opencode', 'skills'), join('tools', 'orchestration', 'prompts')]) {
      const dir = join(gitRoot(), dirRel);
      const entries = await readdir(dir, { recursive: true, withFileTypes: true });
      for (const entry of entries) {
        if (!entry.isFile()) continue;
        const rel = join(entry.parentPath ?? dir, entry.name).slice(dir.length + 1);
        const text = await readFile(join(dir, rel), 'utf8');
        if (text.includes('tests/*.test.ts')) offenders.push(join(dirRel, rel));
      }
    }
    expect(offenders).toEqual([]);
  });
});

describe('run-7 stage guidance', () => {
  // proves: Test strategy #8 — Design 007 §4: the test stages own the files
  // they write, so the last `format:check` step belongs to them. These pins
  // keep that ownership line from being silently reverted (the run-005
  // prettier-drift escalation).
  it('tells both test stages to format the files they own before replying', async () => {
    const authoring = renderTemplate(await loadTemplate(PROMPTS_DIR, 'test-authoring.md'), {
      designDir: 'docs/designs/007-x',
    });
    expect(authoring).toContain('npx prettier --write');
    expect(authoring).toContain('the files you touched');

    const update = renderTemplate(await loadTemplate(PROMPTS_DIR, 'test-update.md'), {
      designDir: 'docs/designs/007-x',
      round: 2,
      reviewFiles: 'docs/designs/007-x/reviews/code/testing-r2c0.md',
    });
    expect(update).toContain('npx prettier --write');
    expect(update).toContain('the files you touched');
  });
});

describe('run-010 plan prompt and the flow-owned test rule', () => {
  // The light flow replaces research/design/proposal with one `plan` stage, and
  // replaces the hard-coded "Never edit test files" line in the coder prompts
  // with one `{{testOwnership}}` variable rendered from the flow: one variable,
  // one source, covering the `fix` entered from a red `testsGreen` gate too.
  const LIGHT = configFor('light', DEFAULT_CONFIG);

  const LIGHT_OWNERSHIP =
    'This is the light flow: you own the test files, and no test engineer exists in this flow.';
  const FULL_OWNERSHIP =
    'Never edit test files: a test engineer owns them. If they need updating, reply with testsNeedUpdate: true.';

  /** Render a template filling EVERY placeholder it declares. The test names
   * only the variables it is about, so the pin is the template's shape, not a
   * guessed copy of the whole `stageVars` set: anything the template asks for
   * that the test does not know lands as `<unset>` and is visible in the
   * `{{`-free assertion. */
  function fill(template: string, vars: Record<string, string>): string {
    const declared: Record<string, string> = {};
    for (const match of template.matchAll(/\{\{(\w+)\}\}/g)) {
      const name = match[1];
      if (name !== undefined) declared[name] = '<unset>';
    }
    return renderTemplate(template, { ...declared, ...vars });
  }

  const vars = (ownership: string): Record<string, string> => ({
    designDir: 'docs/designs/002-x',
    sourcePath: 'docs/prompts/002-x.md',
    planMaxLines: String(LIGHT.planMaxLines),
    testOwnership: ownership,
  });

  // proves: Test strategy #11 — the plan template renders from stageVars with
  // no unresolved placeholder, and its interpolated cap is the registry's, so
  // the number a planner reads is the number the gate enforces.
  it('renders plan.md with no unresolved placeholder and the registry cap', async () => {
    const template = await loadTemplate(PROMPTS_DIR, 'plan.md');
    const text = fill(template, vars(LIGHT_OWNERSHIP));

    expect(text).not.toContain('{{');
    expect(LIGHT.planMaxLines).toBe(80);
    expect(text).toContain('80');

    // The plan lands at the full flow's proposal path, which is what keeps the
    // design stamp, the size meter and the `reviews/code/` reports working.
    expect(text).toContain('docs/designs/002-x/02-proposal.md');
  });

  // proves: Test strategy #11 — the plan stage is wired to that template and to
  // the planner agent, so the template this suite renders is the one the runner
  // sends.
  it('wires the plan stage to the plan template and the planner agent', async () => {
    const def = stageDef('plan', undefined);
    expect(def.template).toBe('plan.md');
    expect(def.agent).toBe('design/planner');
    expect(existsSync(join(PROMPTS_DIR, 'plan.md'))).toBe(true);
    const text = await loadTemplate(PROMPTS_DIR, def.template ?? 'plan.md');
    expect(text.length).toBeGreaterThan(0);
  });

  // proves: Test strategy #11 — the three coder-facing templates carry the
  // variable, not the standing rule: the sentence "Never edit test files" moves
  // into the full flow's value, so a light prompt cannot inherit it.
  it('renders the light ownership line in implement, fix and fix-tests', async () => {
    for (const template of ['implement.md', 'fix.md', 'fix-tests.md']) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      expect(raw, template).toContain('{{testOwnership}}');
      expect(raw, template).not.toContain('Never edit test files');

      const text = fill(raw, vars(LIGHT_OWNERSHIP));
      expect(text, template).toContain('no test engineer exists in this flow');
      expect(text, template).toContain('you own the test files');
    }
  });

  // proves: Test strategy #11 — the same templates render the full flow's line,
  // which keeps the standing rule and names the `testsNeedUpdate` escape: one
  // variable, both flows, no second copy of the rule.
  it('renders the full ownership line with the testsNeedUpdate escape', async () => {
    for (const template of ['implement.md', 'fix.md', 'fix-tests.md']) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      const text = fill(raw, vars(FULL_OWNERSHIP));
      expect(text, template).toContain('Never edit test files');
      expect(text, template).toContain('testsNeedUpdate');
    }
  });

  // proves: Test strategy #11 + The `plan` stage — the five sections exist
  // exactly once, in the skill's template, and the skill states no number: the
  // cap travels through the prompt from the registry.
  it('keeps the plan-note template to the five sections and the cap out of it', async () => {
    const text = await readFile(
      fileURLToPath(
        new URL('../../../../.opencode/skills/plan-note/references/template.md', import.meta.url),
      ),
      'utf8',
    );
    const lower = text.toLowerCase();
    for (const section of [
      'problem',
      'chosen approach',
      'files to touch',
      'test plan',
      'done criteria',
    ]) {
      expect(lower, `section: ${section}`).toContain(section);
    }
    // The five sections exist once: a second copy is a second source of truth.
    expect(text.split('\n').filter((line) => line.startsWith('## '))).toHaveLength(5);
    expect(text).not.toContain('{{planMaxLines}}');
    expect(text).not.toMatch(/\b80\b/);
  });
});
