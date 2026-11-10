import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { loadTemplate, renderTemplate } from '../src/pipeline/prompts.js';
import { stageTemplate } from '../src/pipeline/runner.js';

const PROMPTS_DIR = fileURLToPath(new URL('../prompts', import.meta.url));

// ---------------------------------------------------------------------------
// Which fix prompt runs. The 003 run sent a tests-red fix to a session with
// five report paths that had never been written; the coder had to guess that
// the real task was "make npm run check green".
//
// The formatter-stub block that used to live here is gone on purpose: the
// runner's auto-format step (security r1 major 1) was dropped — the runner
// must never exec a model-plantable binary under a git-ignored
// `node_modules/.bin` path. Format violations now go the ownership route:
// the red `format:check` gate summons `fix`, which runs `npm run format`
// itself, and test-owned dirty files route to `test-update`.
// ---------------------------------------------------------------------------

function stateWith(testsGreen: boolean) {
  return { testsGreen } as never;
}

describe('stageTemplate', () => {
  const fixDef = { name: 'fix', template: 'fix.md' };

  it('red tests pick the check-green prompt, not phantom review reports', () => {
    expect(stageTemplate(fixDef, 'fix', stateWith(false))).toBe('fix-tests.md');
  });

  it('green tests (a review dissent triggered the fix) keep the findings prompt', () => {
    expect(stageTemplate(fixDef, 'fix', stateWith(true))).toBe('fix.md');
  });

  it('every other stage keeps its registered template', () => {
    for (const green of [true, false]) {
      expect(
        stageTemplate(
          { name: 'implement', template: 'implement.md' },
          'implement',
          stateWith(green),
        ),
      ).toBe('implement.md');
      expect(
        stageTemplate(
          { name: 'proposal-revise', template: 'proposal-revise.md' },
          'proposal-revise',
          stateWith(green),
        ),
      ).toBe('proposal-revise.md');
    }
  });
});

describe('fix-tests.md', () => {
  it('names the real task and says no review reports exist for this round', async () => {
    const text = renderTemplate(await loadTemplate(PROMPTS_DIR, 'fix-tests.md'), {
      designDir: 'docs/designs/003-x',
    });
    expect(text).toContain('npm run check is red');
    expect(text).toContain('no review reports');
    expect(text).toContain('Never edit test files');
    expect(text).not.toContain('reviews/code/');
  });
});
