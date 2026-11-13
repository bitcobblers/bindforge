# security review r1 c2 (ratification of the r1 revisions)

verdict: approve

Scope: `git diff bbebe401 fe5671d` on `02-proposal.md`, plus the full current
document and `docs/adrs/013-game-manifests.md`. Ratification rule applied: a
finding is filed only where the revised text broke something this seat approved
in round 1 or made a section this seat cited false.

## What r1 closed, from the security side

- **`presets` is no longer uncapped.** "Budgets" now reads
  "`MAX_MANIFEST_ROWS = 256` bounds each array-shaped entry (`actions`,
  `dead-keys`, `docked-actions`, `presets`)", the invalid corpus gains
  `presets-past-cap.toml`, and Test strategy #4 gains "a 257th preset id".
  Combined with the unchanged "Identity grammar" clause (each preset id is
  slug-grammar checked and at most 63 octets), one hostile document inside the
  1 MiB budget now carries at most 256 × 63 octets of preset data, so the
  uniqueness pass / store / re-serialize cost my r1 finding described is
  bounded. The Risk row's mitigation — "The four named budgets cap every array
  in the entry table before any join" — is now true for all five array entries
  (`actions`, `exclusive-groups` via `MAX_CONSTRAINT_GROUPS`/`MAX_GROUP_MEMBERS`,
  `dead-keys`, `docked-actions`, `presets`). This closes the r1 finding.
- Nothing else security-relevant moved. The diff does not touch Row encoding,
  the Load-time validation steps, the location rule, or the error contract, so
  the sections my other r1 findings cited are byte-identical to what I approved.
- The revised example note ("`ui.inventory` is not yet in `registry.toml` …
  see Registry growth order, and Deferred") points at a section that exists
  (Chosen design → Identity, revision, and the series pins) and does not weaken
  the load-time join in step 3, nor contradict ADR-013's Decision clause
  ("ids are validated against the loaded registry"). No contradiction to
  report under the Deferred rule.
- Dependency check (checklist item 5): the design adds no dependency — "std-only,
  no new dependency, `Cargo.lock` byte-unchanged" — so there is no advisory to
  fetch. Items 1, 3, 4, 6 are unchanged by this diff and were clean in r1.

## Findings

- [nit][new] Chosen design → **Budgets** — "`MAX_MANIFEST_ROWS = 256` bounds
  each array-shaped entry (`actions`, `dead-keys`, `docked-actions`, `presets`)"
  — the r1 wording ("bounds each of `actions`, `dead-keys`, and
  `docked-actions`") is a closed list; the replacement generalizes to a
  category, "each array-shaped entry", that in this entry table also contains
  `exclusive-groups`, whose intended cap is 64 groups × 16 members, not 256.
  Since `spec.md` + fixtures are the third-party contract, a reader who takes
  the category literally implements a 4× weaker group cap than the design
  intends (fail-open on an untrusted-input cap), and the parenthetical reads as
  exhaustive only if the reader already knows why `exclusive-groups` is absent.
  The cap itself is not lost — the entry table still says "at most 64 groups"
  and `MAX_CONSTRAINT_GROUPS = 64` follows one clause later — so this is a
  precision regression, not a hole. Fix: revert to the closed list and name the
  other cap's owner in the same sentence: "`MAX_MANIFEST_ROWS = 256` bounds
  each of `actions`, `dead-keys`, `docked-actions`, and `presets`;
  `exclusive-groups` is bounded by `MAX_CONSTRAINT_GROUPS`".

## Notes (no severity, no gate effect)

- r1 findings 2–5 — Unicode `Cc`/`Cf` ban in native names, the unjoined
  `Manifest` reachable through the public generic `parse` arm, the non-regular
  `manifest.toml` (symlink/FIFO) bypassing the pre-read size budget, and the
  unbounded upper end of `revision` — are untouched by this diff and are not
  regressions, so they are not filed here under the ratification rule. The
  Revision log records decisions only for the two Budgets entries; it carries
  no entry accepting or rejecting those four, which leaves the run's record
  ambiguous about whether they are deferred or simply missed. (Record-keeping,
  for the editor rather than this checklist.)
- FYI (correctness/grounding, not security): the new Budgets sentence, "Each cap
  sits well above the baselines surveyed for it (40 registry ids, A7; TF2-scale
  bind lists and multi-word native names, A9, A13)", grounds
  `MAX_CONSTRAINT_GROUPS = 64` and `MAX_GROUP_MEMBERS = 16` in baselines the
  sentence does not name; whether those baselines exist in 01-research.md is a
  research-seat question.
