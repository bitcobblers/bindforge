---
status: proposed
date: 2026-11-13
---

# ADR 020: Restart the runner when a run changes the runner

A pipeline run that edits the harness's own source or prompts **pauses at a
restart boundary** and is resumed in a fresh process. The runner detects that a
stage commit changed the files it executes or renders from, stops cleanly with a
`restart-required` marker instead of an escalation, and the next `resume`
continues with the new code in force. No code round is charged, no attempt is
wasted, and a harness self-change can add a prompt variable without dying on it.

## Context

Run 016 escalated at 13:57 on `unresolved template placeholder:
{{gateEvidence}}`. Nothing about its implementation was wrong: the check gate
had gone green a moment earlier, `npm run check` and the three cargo legs
passing, and the branch carried five new suites for the feature it was adding.
The failure was in the harness's own liveness.

The runner is a long-lived process. It imported `src/pipeline/prompts.ts` when
the run started, at 12:38 — before ADR-016's implementation existed. But a stage
prompt is assembled by reading its template from disk at stage time
(`loadTemplate`), so the moment the `implement` stage rewrote
`prompts/implement.md`, `fix.md`, `fix-tests.md` and `code-review.md` to carry
the new `{{gateEvidence}}` placeholder, the *live* runner was rendering
post-change templates with a pre-change renderer: one that never supplied the
variable and had no default for it. Strict render — the rule that exists because
run 010 died reading a model's quotation of `{{planMaxLines}}` as an unfilled
placeholder — threw, and the run escalated before its code review.

The coder had already reasoned about the same hazard one layer away. Its build
note records adding `gateEvidence` to `TEMPLATE_DEFAULTS` precisely because
three pre-existing fixtures render the real templates with hand-built vars and
would throw. The fix was in the tree. A running process cannot read a new
version of its own module.

ADR-006 settled the checkpoint side of this problem: a run's state must survive
being resumed by a runner whose state shape has changed. It said nothing about
the process, because until now no self-change had altered the *prompt contract*
mid-run: 007 (the deed audit), 008 (product language) and 010 (the plan flow)
each changed harness behaviour without adding a template variable that the old
process would be asked to fill. So the rule was never exercised, and the gap is
not a violation of ADR-006 but a case it never covered.

The cost of the gap is not one retry — an escalation is the run's hardest stop.
It required a human to read a gate log, diagnose a staleness bug that the tests
could not see, and start a fresh process by hand.

## Decision

1. **A stage commit that changes what the runner executes or renders opens a
   restart boundary.** After each stage commit the runner compares the paths
   that commit touched against the harness's own executable surface: the
   orchestration sources it imports, and the prompt templates it loads at stage
   time. A hit opens the boundary; a stage commit that does not touch them does
   not, so ordinary product runs never see this.

2. **The boundary is a clean pause, not a failure.** The runner finishes the
   stage commit, writes the checkpoint as usual, and stops with a
   `restart-required` marker naming the changed paths and the stage that will
   run next. It is not a stage failure: it consumes no attempt, charges no code
   round, and produces no `ESCALATION.md`. The operator — or the driver script —
   resumes, and because the checkpoint grammar already survives self-update
   (ADR-006), the fresh process picks the run up at the same stage with the new
   code in force.

3. **The exit code distinguishes a restart from an escalation.** A run that
   pauses at a boundary exits with its own status, separate from the escalation
   status, so a driver can treat "restart and continue" as routine automation
   rather than a stop-and-read condition. The status is recorded in the run's
   checkpoint and named in the summary a driver logs.

4. **The prompt contract is the thing that makes this necessary, so it is the
   thing that gets pinned.** The runner keeps recording each stage's template
   hash in its transcript, and the run's records make it visible which revision
   of a template a stage actually saw. A stage that ran before the boundary is
   judged against the template it was given, not the current one.

## Alternatives considered

**Require a human to resume after any harness self-change.** True of the
incident, and too blunt as policy: most self-change runs never touch the prompt
contract, and a mandatory restart would slow those for no reason. The rule is
keyed to what actually changed.

**Default every unknown template variable at the render boundary.** Rejected:
it is the strict-render rule that turns run 010's quotation bug and this run's
stale renderer into a loud failure rather than a model silently receiving
`{{gateEvidence}}` as text. Loosening it hides the class instead of ending it.

**Forbid a self-change run from adding prompt variables.** Rejected: that is
precisely the change ADR-016 needed, and a harness that cannot improve its own
prompts cannot implement most of what it decides.

**Hot-reload the runner's own modules.** Rejected on mechanism: a partially
reloaded process is a worse failure mode than the one being fixed, and the
harness already has a clean restart primitive — the checkpoint and `resume`.

**Treat it as an escalation and leave the fix to the code round.** Rejected:
the code was correct and green; the failure was in the harness's process
lifecycle, which no code round can repair from inside the run.

## Consequences

Becomes easy: a harness run may change its own prompts, templates and prompt
variables in one pass; the pause is automatable by the driver that started the
run; and run records state which template revision each stage saw.

Costs: the flow gains a state (`restart-required`) that resume, status output,
the lock, and every driver must understand; the set of "files the runner
executes" is a rule that has to be kept honest as the harness grows, and it is
covered by a test that fails when a new executable path is added without it;
and a run that changes the harness now takes one more process start, which is
seconds — far less than the escalation it replaces.

Not decided here: whether the runner should restart *itself* rather than pause
for the driver to resume it; whether the boundary should also cover the flow
configuration and agent permission files, which the runner does not re-read
within a stage today; how many boundaries one run may cross before the run is
considered too large for a single pass.
