# Code review — correctness seat — round r1, dissent pass c3

**Verdict: request-changes** (1 open blocker, 1 carried minor, 2 nits; two of
round 1's three findings are now discharged.)

## Scope and inputs

- Diff under review: `git diff 73ef6ca37c8e8205a51f37586425e971db9a4fd9..HEAD --
  ':(exclude)docs/designs'` — 2 files, 8 insertions / 5 deletions:
  `crates/core/src/placement.rs` (doc comments only) and `schemas/README.md`
  (the `MAX_SLUG_OCTETS` bullet's follow-up clause). No code, no test, no
  accept/refuse set, no public surface changed by this revision.
- Inputs read: that diff; `docs/designs/024-game-slug-identity/02-proposal.md`;
  `docs/adrs/024-game-slug-identity.md`; the runner's gate record; this seat's
  round-1 findings. Surrounding source was opened only to re-verify carried
  findings: `docs/adrs/013-game-manifests.md:114-116`,
  `crates/core/src/placement.rs:67-113`, `schemas/README.md:112-149`,
  `crates/core/tests/placement.rs:315-333`, and the deviation/deferred
  registers in `03-build-notes.md` (:232-259, :261-276) that the changed lines
  cite. No review report or transcript was read, listed, or grepped.
- Gate record (2026-11-18T20:00:37Z, green): `npm run check`, `cargo fmt --all
  -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo
  test --workspace --locked` all exit 0. Nothing in that record reads the ADR
  ledger, so it neither proves nor discharges F1.

## Findings from round 1 — status

| Round-1 finding | Status at this revision |
|---|---|
| F1 (blocker) — `013-L5` ledger amendment never lands | **Still open.** The revision did not touch `docs/adrs/`; the annotation is absent at HEAD and in the working tree. See F1 below. |
| F2 (minor) — geometry-name cap follow-up has no referent | **Discharged** in the code-side scope. Residual (ADR `024-B1` scope note) is a runner/ADR-owner hand-off, now disclosed on the published contract's face. Carried as minor, F2 below. |
| F3 (minor) — undeclared refusal-message restructure | **Discharged.** Declared; see F3 disposition. |

## Checklist

| # | Item | Result |
|---|---|---|
| 1 | Named components/interfaces exist and match | **Fails** — the ledger amendment the proposal names as a deliverable does not exist (F1). Everything else in scope exists; this revision adds no interface. |
| 2 | Assumption register honored | Pass. The revision changes only prose; A1 (grammar carries no length bound), A9 (twelve exported constants, `schemas/README.md:147-149`), and A3's four amended specs are unaffected. |
| 3 | Deviations declared in `03-build-notes.md` | Pass for the two deviations in view: the geometry-name scope (`03-build-notes.md:229-244`) and the refusal-message restructure (`:245-259`, naming `action.rs`, `manifest.rs`, both `document.rs` checks including `check_pin`, and `policy.rs`). |
| 4 | Scope creep | Pass. The diff is exactly the citation fix this seat asked for; nothing beyond it. |
| 5 | Build-note claims true | Pass for the claims the diff depends on — see F2 disposition. |
| 6 | Error paths fail closed | Pass, unchanged: no error path is touched; `is_slug_name` (`placement.rs:103-113`) and `MAX_SLUG_OCTETS = 128` (`:90`) are byte-identical. |
| 7 | Prior blocker/major addressed | **Fails** — the round-1 blocker is untouched for the third consecutive pass. |

## F1 — BLOCKER — the `013-L5` ledger amendment still has not landed

- **Location:** `docs/adrs/013-game-manifests.md:114-116` (file is in neither
  `main...HEAD`'s 45-file source/test pathspec, nor this revision's 2 files, nor
  the working tree).
- **Problem.** The design mandates this amendment three times — Goals
  (`02-proposal.md:31`), "Specs, fixtures, ledger" (`:226-228`), Rollout 3
  (`:322-323`) — and ADR-024 makes it a rollout item of this run
  (`docs/adrs/024-game-slug-identity.md:142-143`: "`013-L5` gains its
  `amended-by` annotation naming `024-B1`, per `021-L4`"). At HEAD the unit line
  still reads only: "**013-L5** Name agreement: the game directory's name must
  equal the `id` entry byte-for-byte; the refusal never echoes a local path. →
  spec: Table data, Load-time validation." — with no `amended-by` and no
  `024-B1` string anywhere in the file (verified by direct read and by a
  whole-file pattern search for `amended-by|013-L5|024-B1`, which returns only
  the bare `013-L5` heading). `git diff HEAD -- docs/adrs` and
  `git diff --cached -- docs/adrs` are both empty, so it is not in flight
  either. The branch that raises the shared bound and amends four published
  contracts — `schemas/manifests/v1/spec.md` already annotates its `id` row with
  `013-L5` `024-B1` — would be recorded complete with the amended unit's ledger
  row un-annotated in a `ledger: yes` ADR, and no gate leg reads the ledger, so
  the green record does not cover it. Violates checklist 1 (a named deliverable
  that does not exist) and 7 (prior-round blocker not addressed).
- **Fix.** Land the `amended-by 024-B1` annotation on the `013-L5` unit line in
  `docs/adrs/013-game-manifests.md` before completion, leaving 013's prose and
  `013-R1` untouched as history (`02-proposal.md:227-228`). The commits are the
  runner's (`:228`, ADR-005), and stage ownership bars the build stage from
  `docs/adrs/**` — so this is a runner action at a stage boundary, exactly as
  this seat's round-1 fix prescribed. Recording it in `03-build-notes.md` as a
  hand-off is the correct bookkeeping but does not discharge it.

## F2 — MINOR (residual) — the accepted `024-B1` text still states a wider enforced scope than the corpus

- **Location:** `docs/adrs/024-game-slug-identity.md:97-100`; disclosed at
  `schemas/README.md:131-137` and `crates/core/src/placement.rs:71-77`.
- **Problem — and what this revision fixed.** Round 1's substantive half is
  discharged: the self-referential "that format's named follow-up" is replaced,
  in all three places this seat named, by a handle with a real referent —
  `DF-024-1`, deferred item 1 of `docs/designs/024-game-slug-identity/03-build-notes.md`
  (`schemas/README.md:138-139`; `placement.rs:78-81`; `:100-101`). I verified
  the referent exists and says what the citations promise: `03-build-notes.md:261-273`
  registers "`DF-024-1` — cap `keyboard-geometry/v1` geometry names at
  `MAX_SLUG_OCTETS` at that format's next major", names the citing sites, and
  notes the pin it must take over when it lands. The README bullet and the
  constant's doc comment remain textually verbatim copies of one another
  (`02-proposal.md:206-208`), so the one-stated-home property survives the edit.
  What remains: `024-B1` itself still enumerates "a geometry name" among the ids
  "every slug-grammar id rides", while the reference implementation enforces no
  length on it (`is_slug_name` carries no bound; the pin
  `crates/core/tests/placement.rs:334-335` proves a 129-character geometry name
  loads). The pipeline flips this ADR `proposed` → `accepted` on completion, so
  the accepted unit and the shipped corpus would state two scopes, reconciled
  only by a reader who happens to reach the README bullet.
- **Fix.** Runner/ADR-owner action, same family as F1 and equally outside the
  build stage's ownership: annotate `024-B1` with the narrower enforced scope,
  or record the divergence in the acceptance note written when the ADR flips, so
  the accepted unit and the corpus state one scope. No code change sought; the
  disclosure now on the bullet's face is why this stays minor rather than major.

## F3 — CLOSED — refusal-message restructure is a declared deviation

`03-build-notes.md:245-259` declares exactly what this seat described in round
1: the shape/bound fault split across `action.rs`, `manifest.rs`, `document.rs`
(both `check_slug`-side checks and `check_pin`) and `policy.rs`, bound messages
interpolated from the exported constant so the number survives the
escape-and-truncate cut, and `check_pin`'s new bound arm bypassing the local
`fault` wrapper — "a manifest id is at most 128 octets: `<head>`" instead of the
"`<text>` is not a manifest pin: …" frame its siblings keep — with the
accepted/rejected sets unchanged. That matches my own round-1 read of
`crates/core/src/document.rs:1232-1243` and `crates/core/src/policy.rs:50-65`
(both build the number with `format!` from `MAX_SLUG_OCTETS`, per
`02-proposal.md:125-127`) and of the pins at `crates/core/tests/policy.rs:874-919`
and `crates/core/tests/chain_ops.rs:692-716`. This seat's fix offered the
declaration as an alternative to restoring the frame; the declaration is the leg
taken, so checklist 3 is satisfied and the finding is withdrawn. No code change
sought, and none was made — correct, since the shapes are test-pinned.

## New in this revision

- **NIT — `crates/core/tests/placement.rs:330`** still names the follow-up as
  "the bullet's named follow-up", the phrasing this revision retired everywhere
  else. The pin's comment spells out the follow-up's content, so nothing dangles
  wrongly, but the one place a future implementer must edit when `DF-024-1`
  lands is the one place that does not carry the handle. **Fix:** cite
  `DF-024-1` there too, in the same edit that would take the pin over.
- **FYI (out of specialty, no severity):** `schemas/README.md:138-139` are now
  ~100 and ~118 characters, the only lines in that bullet that break the file's
  ~80-column prose wrap; the `placement.rs` copy is re-wrapped normally. No
  check reads markdown width.

## Why this pass still requests changes

One blocker, unchanged for three passes and outside the build stage's ability to
fix: ADR-024 Rollout 5's ledger annotation. The runner owns that commit; landing
it — with, if convenient, the `024-B1` scope note from F2 — clears this seat.
