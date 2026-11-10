import type { SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { auditSessionIndependence } from '../../src/pipeline/independence.js';

// ---------------------------------------------------------------------------
// The runner-side review-independence audit: what the session's own message
// list proves it read or ran. Transcripts are out of band for every stage;
// report files are out of band for review seats only (the designer must read
// the dissenting reports by design).
// ---------------------------------------------------------------------------

const NOW = 1_000_000;

function toolPart(
  name: string,
  input: Record<string, unknown>,
  status: 'completed' | 'error' = 'completed',
): SessionMessageInfo {
  const state =
    status === 'completed'
      ? { status, input, content: [] }
      : { status, input, error: { name: 'PermissionDeniedError' } };
  return {
    id: 'm_t',
    time: { created: NOW },
    type: 'assistant',
    agent: 'a',
    model: {},
    content: [{ type: 'tool', id: 'p1', name, state }],
  } as unknown as SessionMessageInfo;
}

function shellPart(command: string, at: number = NOW): SessionMessageInfo {
  return {
    id: 'm_s',
    time: { created: at },
    type: 'shell',
    shellID: 's',
    command,
    status: 'exited',
  };
}

describe('auditSessionIndependence', () => {
  it('catches any stage reading a transcript', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/transcripts/research-r0.md' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain('transcripts/');
  });

  it('catches shell reaching for run records through the git-diff allow', () => {
    const v = auditSessionIndependence(
      [shellPart('git diff main...HEAD -- docs/designs/003-x/transcripts/')],
      { since: 0, reviewStage: true },
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain('shell touched run records');
  });

  it('catches a review seat reading another seat report', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/security-r1c0.md' })],
      { since: 0, reviewStage: true },
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain('read a review report');
  });

  it('catches a review seat reading its own report back', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/security-r1c1.md' })],
      { since: 0, reviewStage: true },
    );
    expect(v).toHaveLength(1);
  });

  it('allows the editorial diff file — the editorial seat named scope', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/editorial-r1.diff' })],
      { since: 0, reviewStage: true },
    );
    expect(v).toEqual([]);
  });

  it('allows mutating stages to read reports (the hand-off is the design)', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/security-r1c0.md' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toEqual([]);
  });

  it('allows a review seat to write its own report (edit/write parts)', () => {
    const v = auditSessionIndependence(
      [
        toolPart('write', {
          filePath: 'docs/designs/003-x/reviews/design/security-r1c1.md',
          content: '...',
        }),
      ],
      { since: 0, reviewStage: true },
    );
    expect(v).toEqual([]);
  });

  it('ignores messages from before this attempt started', () => {
    // Attempt 1 already paid for the violation; the fresh attempt must not be
    // failed again for the older session history.
    const v = auditSessionIndependence(
      [
        toolPart('read', { filePath: 'docs/designs/003-x/transcripts/x.md' }),
        shellPart('cat docs/designs/003-x/transcripts/x.md', 5),
      ],
      { since: NOW + 1, reviewStage: true },
    );
    // the tool part is at exactly NOW, below since; the shell at 5 likewise
    expect(v).toEqual([]);
  });

  it('scans nested input values, not just the first field', () => {
    const v = auditSessionIndependence(
      [toolPart('execute', { tools: [{ path: 'docs/designs/003-x/transcripts/y.md' }] })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
  });

  // --- run 003 misfire regressions: the audit failed test-authoring twice ---

  it('does not flag write content that merely mentions transcripts/', () => {
    // The test engineer writing tests ABOUT run record handling had the
    // string in the source they were writing to a src path. Code about run
    // records is not a read of run records.
    const v = auditSessionIndependence(
      [
        toolPart('write', {
          filePath: 'tools/orchestration/src/pipeline/inference.test.ts',
          content: "readFile('docs/designs/003-x/transcripts/research-r0.md')",
        }),
        toolPart('edit', {
          path: 'tools/orchestration/tests/pipeline-lane.test.ts',
          oldString: 'x',
          newString: "expect(p).toContain('transcripts/');",
        }),
      ],
      { since: 0, reviewStage: false },
    );
    expect(v).toEqual([]);
  });

  it('does not flag a read the permission system already denied', () => {
    // An error-state part delivered no content: the fence worked, so the
    // audit must not punish the attempt a second time for the same act.
    const v = auditSessionIndependence(
      [
        toolPart(
          'read',
          { filePath: 'docs/designs/003-x/transcripts/test-authoring-r0.md' },
          'error',
        ),
      ],
      { since: 0, reviewStage: true },
    );
    expect(v).toEqual([]);
  });

  it('flags a COMPLETED glob over the transcripts directory — strict rule kept', () => {
    const v = auditSessionIndependence(
      [toolPart('glob', { pattern: 'docs/designs/*/transcripts/editorial*' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain('glob accessed run records');
  });

  // --- security r1 finding 2 (accepted): segment matching, not the
  // `transcripts/` substring. The bypasses the finding named must now be
  // caught: a path that ENDS at the run-records segment has no trailing
  // slash for the old substring to bite on.

  it('catches a completed shell listing the transcripts directory itself (no trailing slash)', () => {
    const v = auditSessionIndependence([shellPart('ls docs/designs/003-x/transcripts')], {
      since: 0,
      reviewStage: false,
    });
    expect(v).toHaveLength(1);
  });

  it('catches a glob pattern ending at the transcripts segment', () => {
    const v = auditSessionIndependence(
      [toolPart('glob', { pattern: 'docs/designs/003-x/transcripts' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
  });

  it('does not flag grep QUERY text but does flag its target path', () => {
    const query = auditSessionIndependence(
      [toolPart('grep', { pattern: 'transcripts/', path: 'tools/orchestration/src' })],
      { since: 0, reviewStage: false },
    );
    expect(query).toEqual([]);
    const target = auditSessionIndependence(
      [toolPart('grep', { pattern: 'status', path: 'docs/designs/003-x/transcripts/' })],
      { since: 0, reviewStage: false },
    );
    expect(target).toHaveLength(1);
  });

  it('catches a shell TOOL part (not just shell messages) touching transcripts/', () => {
    const v = auditSessionIndependence(
      [toolPart('shell', { command: 'ls docs/designs/003-x/transcripts/' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
  });

  it('does not flag a review seat writing its own report under reviews/', () => {
    const v = auditSessionIndependence(
      [
        toolPart('write', {
          filePath: 'docs/designs/003-x/reviews/design/security-r1c1.md',
          content: 'my report',
        }),
      ],
      { since: 0, reviewStage: true },
    );
    expect(v).toEqual([]);
  });
});
