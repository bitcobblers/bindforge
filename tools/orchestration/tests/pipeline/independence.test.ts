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

// ---------------------------------------------------------------------------
// Design 007 (audit by deeds): the audit judges where a tool looks, never
// what text it carries. Delegation parts and payload keys are never accesses —
// the child-session walk supplies the delegation evidence, and unknown tools
// are judged on path-naming access keys only. The deeds themselves (pin (c))
// stay detected at matcher level so the exemptions cannot hide them.
// ---------------------------------------------------------------------------

describe('judging deeds, not payload text (design 007)', () => {
  // proves: Test strategy #1 — a completed subagent part's delegation text
  // carries no access fields: a hardened prompt reciting the transcripts
  // prohibition must not fail an attempt that accessed nothing (both lane
  // kinds).
  it('does not flag a subagent delegation prompt reciting the transcripts prohibition', () => {
    const delegating = [
      toolPart('subagent', {
        task: 'gather test output',
        agent: 'explore',
        prompt:
          'Run the package tests and report failures. Never open anything under ' +
          'docs/designs/003-x/transcripts/ and never glob docs/designs/003-x/reviews/ — ' +
          'those are not your inputs.',
      }),
    ];
    expect(auditSessionIndependence(delegating, { since: 0, reviewStage: false })).toEqual([]);
    expect(auditSessionIndependence(delegating, { since: 0, reviewStage: true })).toEqual([]);
  });

  // proves: Test strategy #2 — an execute part's program text is payload, not
  // an access: test code whose string literals name transcripts/ and
  // reviews/ must not fail the attempt that only wrote that code.
  it('does not flag execute code containing transcripts/ and reviews/ string literals', () => {
    const v = auditSessionIndependence(
      [
        toolPart('execute', {
          code:
            "const t = 'docs/designs/003-x/transcripts/research-r0.md';\n" +
            "const r = 'docs/designs/003-x/reviews/design/security-r1c0.md';",
        }),
      ],
      { since: 0, reviewStage: true },
    );
    expect(v).toEqual([]);
  });

  // proves: Test strategy #3 — unknown tools keep the path-key rule and lose
  // the payload scan: the existing nested `{ path: '…/transcripts/y.md' }`
  // pin above stays green, and the same fixture under a payload key
  // (`code`) yields no violation.
  it('does not flag a nested transcripts path under a payload key on an unknown tool', () => {
    const v = auditSessionIndependence(
      [toolPart('execute', { tools: [{ code: 'docs/designs/003-x/transcripts/y.md' }] })],
      { since: 0, reviewStage: false },
    );
    expect(v).toEqual([]);
  });

  // proves: Test strategy #4 — pin (c) at matcher level: a message list (the
  // runner will feed child-session lists through here) whose completed read
  // names a transcripts path fails with the violation naming the path.
  it('fails a child message list completing a read of a transcripts path', () => {
    const v = auditSessionIndependence(
      [toolPart('read', { filePath: 'docs/designs/003-x/transcripts/research-r0.md' })],
      { since: 0, reviewStage: false },
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain('transcripts');
  });

  // proves: Test strategy #4 — a review-stage child globbing `reviews/**`
  // fails the audit of that child's message list.
  it('fails a child message list globbing reviews/** on a review stage', () => {
    const v = auditSessionIndependence(
      [toolPart('glob', { pattern: 'docs/designs/003-x/reviews/**' })],
      { since: 0, reviewStage: true },
    );
    expect(v).toHaveLength(1);
  });

  // proves: Test strategy #4 — a child message list writing a report other
  // than the attempt's ownReportPath fails; the write to ownReportPath
  // itself stays allowed.
  it('fails a child message list writing a report other than ownReportPath', () => {
    const opts = {
      since: 0,
      reviewStage: true,
      ownReportPath: 'docs/designs/003-x/reviews/design/correctness-r1c0.md',
    };
    expect(
      auditSessionIndependence(
        [
          toolPart('write', {
            filePath: 'docs/designs/003-x/reviews/design/correctness-r1c0.md',
            content: 'my own report',
          }),
        ],
        opts,
      ),
    ).toEqual([]);
    expect(
      auditSessionIndependence(
        [
          toolPart('write', {
            filePath: 'docs/designs/003-x/reviews/design/security-r1c0.md',
            content: 'another seat',
          }),
        ],
        opts,
      ),
    ).toHaveLength(1);
  });
});

// ---------------------------------------------------------------------------
// ADR-016 Decision 1: the audit's read side gains the same own-report identity
// the write side already holds. A `reviews/` access is the seat's own report iff
// its canonical spelling lies under the attempt's own design directory (the
// directory of `ownReportPath`) and its report key — everything from the
// `reviews` segment on — equals the own report's key. Anything broader fails:
// a sibling report, another design directory's same-named report, a directory,
// a glob, a grep path, or a command naming a second report. The exemption is
// report-identity-scoped, so it propagates through the child walk (A3) and every
// sibling still fails. Transcripts are never exempted — there is no own report
// under `transcripts/`.
// ---------------------------------------------------------------------------

const OWN = 'docs/designs/003-x/reviews/design/security-r1c1.md';
const OWN_OPTS = { since: 0, reviewStage: true, ownReportPath: OWN };

describe('own-report identity on the read side (ADR-016 Decision 1)', () => {
  // proves: Test strategy #3 — reading own report in repo-relative, absolute and
  // separator- / case-variant spellings is clean (the identity normalizes the way
  // the permission layer does: `\` to `/`, case folded, resolved against root).
  it('clears every canonical spelling of the seat’s own report', () => {
    for (const spelling of [
      OWN,
      '/home/op/repo/docs/designs/003-x/reviews/design/security-r1c1.md',
      'docs\\designs\\003-x\\reviews\\design\\security-r1c1.md',
      'DOCS/DESIGNS/003-X/REVIEWS/DESIGN/SECURITY-R1C1.MD',
    ]) {
      expect(
        auditSessionIndependence([toolPart('read', { filePath: spelling })], OWN_OPTS),
        spelling,
      ).toEqual([]);
    }
  });

  // proves: Test strategy #3 — the seat’s `git diff` route to its own report (the
  // spelling §3’s per-seat allow opens) is clean.
  it('clears a shell git-diff of the seat’s own report', () => {
    expect(auditSessionIndependence([shellPart(`git diff ${OWN}`)], OWN_OPTS)).toEqual([]);
  });

  // proves: Test strategy #3 — a sibling report, another design directory’s
  // same-spelled report, a directory, a glob pattern and a grep path under
  // `reviews/` all fail.
  it('fails anything broader than the one report identity', () => {
    expect(
      auditSessionIndependence(
        [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/correctness-r1c1.md' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence(
        [toolPart('read', { filePath: 'docs/designs/099-y/reviews/design/security-r1c1.md' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence(
        [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence(
        [toolPart('glob', { pattern: 'docs/designs/003-x/reviews/design/*' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence(
        [toolPart('grep', { pattern: 'status', path: 'docs/designs/003-x/reviews' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
  });

  // proves: Test strategy #3 — a shell naming own under another run, and own plus
  // a sibling in one command, fail; listing the reviews directory fails.
  it('fails a git-diff off the run, a git-diff naming a second report, and a listing', () => {
    expect(
      auditSessionIndependence(
        [shellPart('git diff docs/designs/099-z/reviews/design/security-r1c1.md')],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence(
        [
          shellPart(
            'git diff docs/designs/003-x/reviews/design/security-r1c1.md ' +
              'docs/designs/003-x/reviews/design/correctness-r1c1.md',
          ),
        ],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence([shellPart('ls docs/designs/003-x/reviews')], OWN_OPTS),
    ).toHaveLength(1);
  });

  // proves: Test strategy #3 — a prefix-less `reviews` token — the report tail
  // without the attempt's design-directory prefix — resolves outside the
  // attempt's design directory and fails closed (§1's identity rule).
  it('fails closed on a prefix-less reviews token', () => {
    expect(
      auditSessionIndependence([shellPart('git diff reviews/design/security-r1c1.md')], OWN_OPTS),
    ).toHaveLength(1);
  });

  // proves: Test strategy #3 — transcripts access is never exempted, even with an
  // own-report identity in scope.
  it('never exempts transcripts access', () => {
    expect(
      auditSessionIndependence(
        [
          toolPart('read', {
            filePath: 'docs/designs/003-x/transcripts/design-review-security-r1.md',
          }),
        ],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence([shellPart('cat docs/designs/003-x/transcripts/x.md')], OWN_OPTS),
    ).toHaveLength(1);
  });

  // proves: Test strategy #3 — the exemption is scoped to the one report, so a
  // child session reading the parent’s own report inherits it (the child walk
  // reuses one options object, A3) while reading a sibling fails.
  it('inherits the exemption into a child reading the parent’s own report, not a sibling', () => {
    expect(auditSessionIndependence([toolPart('read', { filePath: OWN })], OWN_OPTS)).toEqual([]);
    expect(
      auditSessionIndependence(
        [toolPart('read', { filePath: 'docs/designs/003-x/reviews/design/correctness-r1c1.md' })],
        OWN_OPTS,
      ),
    ).toHaveLength(1);
  });
});

// proves: Test strategy #11 — a staged design-seat list that writes its report,
// then re-reads exactly that path via `read` and `git diff`, passes the audit;
// adding one sibling path fails it, naming the sibling.
describe('the own-report seat round trip (ADR-016 Decision 1)', () => {
  it('passes a seat that re-reads only its own report', () => {
    const v = auditSessionIndependence(
      [
        toolPart('write', { filePath: OWN, content: 'my findings' }),
        toolPart('read', { filePath: OWN }),
        shellPart(`git diff ${OWN}`),
      ],
      OWN_OPTS,
    );
    expect(v).toEqual([]);
  });

  it('fails the same seat that also opens a sibling report, naming it', () => {
    const sibling = 'docs/designs/003-x/reviews/design/correctness-r1c1.md';
    const v = auditSessionIndependence(
      [
        toolPart('write', { filePath: OWN, content: 'my findings' }),
        toolPart('read', { filePath: OWN }),
        shellPart(`git diff ${OWN}`),
        toolPart('read', { filePath: sibling }),
      ],
      OWN_OPTS,
    );
    expect(v).toHaveLength(1);
    expect(v[0]).toContain(sibling);
  });
});

// ---------------------------------------------------------------------------
// security r2c0 finding 2 (accepted): the run checkpoint is a run record. Every
// agent config already denies *edit* on `docs/designs/**/.pipeline.json`, the
// permission fence now denies the shell/read/glob routes to it, and the deed
// audit names its segment as the backstop exactly as it names `transcripts`:
// the file carries every seat's findings and verdicts (the anchoring ADR-002/007
// independence guards) and now the gate record beside them, so a stage that
// opens it has opened the run's records. No own-report exemption reaches it.
// ---------------------------------------------------------------------------

describe('the run checkpoint is out of band (ADR-016 run records)', () => {
  const CHECKPOINT = 'docs/designs/003-x/.pipeline.json';

  // proves: security r2c0 finding 2 — a completed read or glob of the checkpoint
  // fails the audit for any stage, review seat or not.
  it('catches any stage reading the run checkpoint', () => {
    expect(
      auditSessionIndependence([toolPart('read', { filePath: CHECKPOINT })], {
        since: 0,
        reviewStage: false,
      }),
    ).toHaveLength(1);
    expect(
      auditSessionIndependence([toolPart('glob', { pattern: CHECKPOINT })], {
        since: 0,
        reviewStage: true,
      }),
    ).toHaveLength(1);
  });

  // proves: security r2c0 finding 2 — the shell routes the widened allowlists
  // open (`cat`, `git show`) are caught by the audit as well, with the seat's
  // own-report identity in scope.
  it('catches a shell command naming the run checkpoint', () => {
    expect(auditSessionIndependence([shellPart(`cat ${CHECKPOINT}`)], OWN_OPTS)).toHaveLength(1);
    expect(
      auditSessionIndependence([shellPart(`git show HEAD:${CHECKPOINT}`)], OWN_OPTS),
    ).toHaveLength(1);
  });

  // proves: security r2c0 finding 2, the other direction — the audit judges the
  // access, never the text (design 007): a stage writing source that merely
  // names the checkpoint path — exactly what this run's own tests do — has not
  // opened the run's records.
  it('does not flag write content that merely names the checkpoint', () => {
    const v = auditSessionIndependence(
      [
        toolPart('write', {
          filePath: 'tools/orchestration/tests/pipeline/gate-evidence-integration.test.ts',
          content: "readFileSync(join(dir, 'docs/designs/002-x/.pipeline.json'), 'utf8')",
        }),
      ],
      { since: 0, reviewStage: false },
    );
    expect(v).toEqual([]);
  });
});
