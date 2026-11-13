---
status: proposed
date: 2026-11-13
---

# ADR 016: Hand every stage the evidence it has to judge

We will stop withholding facts the runner already has. Four changes to the
pipeline: the independence audit stops failing a review seat for re-reading
**its own** report; the gate's captured output becomes a first-class run input
instead of being discarded at spawn time; the document-side families get a
read-only shell and both review panels get the project's check commands, with
run-record isolation enforced at the permission layer as well as by the audit;
and context size becomes a measured, budgeted quantity, recorded per stage.

This is a harness self-change, in the line of ADR-001, 007 and 010. It does not
touch the product: no `crates/`, no `schemas/`.

## Context

The ADR-011…015 series ran four full-flow runs on one inference slot. Two
findings from those runs drive this ADR.

**Stages were guessing, and the cost is visible.** Before `2385d25` widened
stage shell rights, the coder could not invoke the toolchain at all: run 011
spent 3 `fix` stages and 4 of its 5 code rounds repairing shape a formatter
would have told it. After the widening, 012 and 013 went green on the *first*
`implement` and used 2 and 1 of 5 rounds. Review reports keep the same scar:
across 011–013 they mention an unverifiable claim 9, 11 and 3 times
("unevaluable", "cannot verify", "no `test result:` line"), and in 011 that
class of finding bought a revise round plus a re-review. Meanwhile the runner
runs the very checks in question — `testsGreen` and `cargoGreen` spawn `npm run
check` and the three cargo legs with `stdio: 'ignore'`, so the harness holds the
fmt/clippy/test output and throws it away. It then asks five review seats to
judge that same code with none of it in front of them. The code panel has
allowed `cargo *` and `npm run *` since `2385d25` and has never invoked them:
the prompt says "your inputs are exactly" a diff, a proposal, and an ADR, and a
model reasonably reads that as "running things is out of bounds".

**The independence audit's read rule is now too blunt.** ADR-007 judges seats
on deeds, and the read side fails an attempt on *any* access to a `reviews/`
path. In three of four runs a stage failed an attempt that way: a research
child reading a checkpoint, a design seat re-reading its own report, a code seat
globbing the whole review directory. The first and third are real violations.
The second is not — a seat re-reading what it itself wrote cannot anchor it to
another seat's verdict, which is the only harm ADR-002's panel independence
guards against. The write side of the audit already exempts the seat's own
report; the read side has no such exemption, and the asymmetry is an accident.

**Context, not lanes, is the throughput limit.** `--max-parallel` is the
harness's knob for ADR-003's lanes, and it is currently 1 — but the server
serves one slot (`--parallel 1`) on a 262,144-token context, keeps only 32,768
KV tokens resident, and the host was at 59 of 62 GiB RAM during these runs. The
largest observed stage prompt was 189,788 tokens, 72% of the window, so most
turns re-prefill past the resident prefix. Nothing measures this today: every
transcript records `parameters: {"note":"not exposed by endpoint"}`, and the
research stage is prompted in 113 bytes with no read budget, so a researcher
reads every prior ADR and every prior run's artifacts by choice of its own.
Raising `--max-parallel` against one slot and that memory picture buys queueing,
not throughput.

## Decision

1. **A seat's own report is its own to re-read.** The read side of the
   independence audit (ADR-007) exempts the seat's own report path, exactly as
   the write side already does. The exemption is an identity match on the report
   key — everything from the `reviews/` segment on — so an absolute path and the
   repo-relative path the prompt handed the seat are the same report. Every
   other `reviews/` path stays a violation, and so does anything broader than
   one report: a glob, a listing, or a directory (`reviews/code/*`) is not an
   identity match on a report and fails the attempt. The seat's report path
   stays named in its prompt, and the prompt keeps forbidding the other seats'
   reports.

2. **The gate's output is run evidence, not a discarded stream.** The check
   gate keeps its verdict and additionally captures, per leg, the exit status
   and a bounded tail of output. That record is carried in the checkpoint under
   the existing record caps and is handed to: the code-review seats, so a claim
   about behaviour has a `test result:` line behind it instead of an "unevaluable"
   finding; `fix` and `implement`, so a red gate names the failing test rather
   than a symptom; and the build note, where it becomes part of the run record.
   The capture adds no file to the worktree — it lives in the checkpoint and the
   documents stages already own — so ADR-006's reserved paths and the ownership
   rules do not change.

3. **Document and review seats get read-only reach, and run records are denied
   to them where they can be denied.** The document families — researcher,
   designer, planner, editor, doc-sync — gain a read-only shell allowlist
   (`cd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`, `rg`, `find`, and
   `git status`, `git diff`, `git log`, `git show`) so a proposal can check a
   path and a line count instead of asserting both. Both review panels hold the project's check
   commands: `cargo *`, `rustfmt *`, `npm run *`, `npm test *`, `npx vitest *`,
   with the write legs denied (`cargo fmt` without `--check`, `cargo fix`,
   `clippy --fix`, `npm run format`) and commits, branch state, deletion, and
   publishing denied. The design-review prompt's "never browse to verify" is
   reworded to match: you may run the project's checks, you may not open the
   run's records. Then the two run-record directories are denied in the
   permission layer itself — shell rules naming a path under a run's
   `transcripts/` or `reviews/`, and `glob`/`grep` pattern rules for the same
   directories — so isolation no longer rests only on the after-the-fact audit.
   That last rule is knowingly partial: shell resources are raw command text and
   a `grep` resource is a pattern, so a deny there is coarse and can be routed
   around. The deed audit stays the backstop, exactly as ADR-007 says; the
   permission rule exists to stop the accidental access, not the deliberate one.

4. **Context is budgeted and measured.** Each stage transcript records the token
   usage of its session rather than "not exposed by endpoint". The research
   prompt names its read set — the source ADR, the ADR index, and the specific
   prior artifacts it must consult — and states a ceiling on what it may read,
   instead of leaving the corpus open. The flow configuration gains a document
   size ceiling for `01-research.md` and `02-proposal.md`, the sibling of the
   plan flow's existing `planMaxLines`. This decision explicitly does *not*
   raise `--max-parallel` or the resident KV budget: with one served slot, a
   32,768-token resident prefix, and the host's memory already committed, more
   concurrency means evicted prefixes and slower stages, which is the opposite
   of what the measurement is for.

## Alternatives considered

**Raise `--max-parallel` to use more slots.** Rejected on the facts above: the
server serves one slot, and two 190K-token sessions in it evict each other's
prefix. The harness should earn its lanes by shrinking prompts first.

**Give review and document seats full shell.** Rejected. The fences that carry
weight are run-record isolation and write legs; widening those recreates the
sibling-transcript reading that ADR-007 was written to catch.

**Leave the read rule strict and treat own-report reads as acceptable
collateral.** Rejected. Each such read costs a failed attempt and a retry —
roughly three to six minutes per run, four runs in a row — and buys no
independence, since the seat wrote the report.

**Let reviewers run the checks and skip the captured gate output.** Rejected as
insufficient, and kept as a complement. A seat's own run is its own evidence at
its own moment; the runner's gate verdict is the shared ground truth that five
independent seats and the fix loop can all cite. Seats still may run things —
decision 3 exists so they can.

**Have stages self-report what they verified.** Rejected: ADR-007 already
settles that text is not evidence about what an agent did.

**Fix the delegation hole in this ADR too.** Rejected as scope, not merit. A
stage agent can spawn a `general` subagent, which starts from OpenCode's
allow-all base policy, so a family with no shell of its own can delegate its way
to any command — that is how run 012's research stage reached a checkpoint. The
fix is a `subagent` deny per family; it is left undecided here because whether
each family *should* delegate at all is a separate judgement from whether it
should be able to read.

## Consequences

Becomes easy: a review seat cites a test count instead of calling a claim
unevaluable; a `fix` round starts from the failing leg's output; a proposal
author checks its own path and line claims; the read-rule asymmetry stops
burning attempts; and prompt growth per stage becomes a number in the
transcript rather than a guess.

Costs: review prompts and build notes grow by an evidence block, which is
context spent on every seat — the same resource decision 4 measures, so the
block must stay bounded and it is covered by the test that pins the capture
budget; the checkpoint grows by one bounded record, which ADR-006's
resumability rule covers only if the field is optional and defaulted when
absent; the design panel now runs commands, so its prompt wording and the audit
must agree about what a seat may do; and coarse shell denies over
`transcripts/`/`reviews/` can deny a legitimate command that merely names those
directories, which is accepted because no stage has a reason to name them.

Not decided here: whether any family may delegate to a subagent, and with what
permissions; a middle flow tier between `full` and `light` for small ADRs
(013's run spent 21 of 28 sessions on review seats); any change to the number of
served inference slots or their memory layout.
