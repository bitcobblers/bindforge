# correctness review r2

verdict: approve

Scope: `docs/designs/014-profiles-as-chains/reviews/design/editorial-r2.diff` (12
hunks), ratification pass. Inputs used: the diff, `02-proposal.md` as it now
stands, `docs/adrs/014-profiles-as-chains.md`, the Revision log. My r1 record
carries no blocker/major correctness findings, so there was nothing to
re-verify for closure; the pass question was whether the editorial reflow broke
anything.

Per-hunk verification (all language-only unless noted):

- Options "Where ancestor documents come from" — `ADR-018's, and storage
  location is explicitly undecided (A11)`: comma only; the clause still matches
  A11's "storage location is explicitly undecided".
- Options "May a v2 chain include a v1 link" — `Chosen: refuse`: now matches
  the option's own heading ("default its `kind` to `base`, or refuse") and the
  single-major directory rule in Cross-file loading. No change to the decision.
- Options "Within-patch collisions" — `the fixed list order `set` → `clear` →
  `swap` applies per patch`: still equals determinism rule 2 ("the lists apply
  in the fixed order `set` → `clear` → `swap`").
- Delta inventory — `renamed to`: matches Rollout 2's rename of
  `PROFILE_ENTRIES` → `PROFILE_V1_ENTRIES`.
- The contract — `the set is selected from the declared `version``: now agrees
  with the `profile_entries(version)` selector named in The model and Rollout 2.
- Cross-file loading step 2 — `An absent file is `ChainLinkMissing` naming the
  id`: still matches the new variant in The model and Test strategy 3.
- Resolution cost paragraph — `totalling at most `MAX_CHAIN_DEPTH ×
  MAX_DOCUMENT_BYTES` (32 MiB)`: arithmetic holds (A6's 1 MiB × A1's 32), and
  "at most" still absorbs A6's one-byte-over stream allowance.
- Chain operations (`fork`, `diff`, `merge`) and Test strategy 5 and 7 —
  re-wrap plus one semicolon; wording is byte-for-byte the r1 claims, so the
  merge non-representability rule (`swap X Y` under `clear X`), the
  empty-patch identity refusal, and the collision-id rules are unchanged and
  still mutually consistent.
- Test strategy 7 — `forged-log-line.toml` was previously split across lines as
  `forged-log-line` / `.toml`; the diff rejoins it, so Test strategy 7 now names
  the same fixture as The contract's invalid list. This hunk removes an
  interface ambiguity rather than creating one (checklist item 5).
- Deferred — `needed before the first pinned chain ships`: wording only; still
  consistent with the ADR, which does not decide a CI matrix, so the deferral
  stands.

Nothing in the diff contradicts the ADR's Decisions One–Four, and no hunk
touches a `verified` row's basis or the Revision log's recorded rejections.

## Findings

- [nit][new] Checklist coverage — Correctness checklist item 2 (spot-check 3–5
  research URLs) is not executable in this pass: the diff adds no citation and
  no claim resting on one, and the pass's inputs exclude `01-research.md`.
  Noted rather than passed: the register's only `unverified` row, A20 ("The
  research's standards convergences"), is honestly labelled and no hunk changed
  its text or a sentence depending on it. Fix: no document change; run the URL
  gate on a whole-proposal pass, or have the reviser confirm A20's research
  citations were gated in r1.

## FYI (no severity)

- Options → "`merge`'s conflict rule" reads `a conflict is representable iff
  the composed effect cannot be written as one patch`; the sentence's sense
  looks inverted relative to Chosen design's "When some action's composed
  effect has no one-patch form … `merge` refuses". Pre-existing (outside this
  diff), and wording-only — editorial's seat.
- The contract's per-link rule "`base` … no `clear`/`swap` elements" is pinned
  by the single-document invalid fixtures (`base-clear-element.toml`), while
  Test strategy 2 files the same refusal "at load". Pre-existing level
  attribution, untouched by this diff — test-strategy's seat.
