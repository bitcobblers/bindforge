import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { loadTemplate, renderTemplate } from '../../src/pipeline/prompts.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 2, the prompt seam. Four templates gain the `gateEvidence`
// template variable — `code-review.md` (its "Your inputs are exactly"
// enumeration names the block, so a seat may cite it), `fix.md`, `fix-tests.md`
// and `implement.md`. The var is defaulted in `stageVars`/`reviewVars` exactly
// as `planMaxLines` is, so a strict render of any of these templates fills it,
// and a run with no record fills it with the stand-in line rather than breaking.
// ---------------------------------------------------------------------------

const PROMPTS_DIR = fileURLToPath(new URL('../../prompts/', import.meta.url));

const TEMPLATES = ['code-review.md', 'fix.md', 'fix-tests.md', 'implement.md'];

/** Fills every placeholder the template declares; the ones under test override. */
function fill(template: string, vars: Record<string, string>): string {
  const declared: Record<string, string> = {};
  for (const match of template.matchAll(/\{\{(\w+)\}\}/g)) {
    const name = match[1];
    if (name !== undefined) declared[name] = '<unset>';
  }
  return renderTemplate(template, { ...declared, ...vars });
}

describe('the gateEvidence prompt variable (ADR-016 Decision 2)', () => {
  // proves: Test strategy #7 — each of the four templates declares the variable.
  it('each coder/seat template carries {{gateEvidence}}', async () => {
    for (const template of TEMPLATES) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      expect(raw, template).toContain('{{gateEvidence}}');
    }
  });

  // proves: Test strategy #7 — a strict render fills the variable: with a
  // gate block supplied, none of these templates renders an unresolved
  // placeholder, and the supplied block reaches the prompt.
  it('renders the gate block with no unresolved placeholder', async () => {
    const block = [
      '```',
      'gate evidence captured by the runner from the code under review:',
      '$ cargo test --workspace --locked  (exit 101)',
      'test result: FAILED. 4 passed; 1 failed',
      '```',
    ].join('\n');

    for (const template of TEMPLATES) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      const text = fill(raw, { gateEvidence: block });
      expect(text, template).not.toContain('{{');
      expect(text, template).toContain('test result: FAILED. 4 passed; 1 failed');
    }
  });

  // proves: Test strategy #7 — the absent record renders the stand-in line into
  // the prompt (a pre-016 resume / the first implement): a seat still gets a
  // well-formed prompt, never an empty or missing section.
  it('renders the no-record stand-in line for a run without a gate record', async () => {
    for (const template of TEMPLATES) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      const text = fill(raw, { gateEvidence: '(no gate record for this run yet)' });
      expect(text, template).toContain('(no gate record for this run yet)');
    }
  });

  // proves: §2 as revised in r2 — the coder prompts place a failing-leg tail in
  // the committed build note's "Gate posture" section, cited LINE-SELECTIVELY:
  // the credential-bearing and absolute-path lines are dropped there, since the
  // build note is the one committed carrier of the gate record. The prompt has to
  // state the rule, because a tail is raw output of the repo's own checks.
  it('routes the gate tail into the build note line-selectively', async () => {
    for (const template of ['implement.md', 'fix.md']) {
      const raw = await loadTemplate(PROMPTS_DIR, template);
      expect(raw, template).toContain('Gate posture');
      expect(raw, template).toContain('03-build-notes.md');
      expect(raw, template).toMatch(/credential/i);
      expect(raw, template).toMatch(/absolute[-\s]path/i);
    }
  });
});
