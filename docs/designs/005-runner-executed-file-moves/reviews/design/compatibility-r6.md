# Compatibility review r6

verdict: approve

Scope: `reviews/design/editorial-r6.diff` (round 6 single editorial confirmation pass). Inputs: the diff, `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log. No previous findings from this seat.

## Diff-by-diff compatibility check

Every hunk was checked against the compatibility checklist; all are language-only and preserve the compatibility-bearing text:

1. **Context comma removal** — no compat content.
2. **Reply field reflow** — `MovePairSchema` / `moves` bounds byte-identical after reflow; the `default([])` additive-checkpoint semantics (A13) are intact.
3. **Wiring em-dash appositive** ("a failure — a returned issue or a throw from a rule — takes") — same routing to the `invalid-reply:` path; retry/escalation semantics unchanged.
4. **Apply "i.e.," and "as a rename error"** — the guard-refusal classification (`moves-apply-failed`, preserved worktree, fresh attempt 2) is unchanged; consistent with #6 and #11.
5. **"Sweep interaction" / "Prompts" heading periods** — formatting only; the shared-`stagePathAllowed` invariant (checklist item 7's cross-reference to Test strategy #4) is untouched.
6. **Risks row reword** ("breaks Test strategy #4 rather than silently breaking production") — same meaning; the case-insensitivity row's `yes (unverified)` / Windows-not-exercised claims are untouched.
7. **#11 "can no longer be applied" for "made unappliable"** — meaning preserved; still consistent with the Apply section's rollback (`byte-identical worktree`) and un-swept-preserved-worktree state.
8. **Rollout #2 "optional, defaulting to `[]`"** — checkpoint-compatibility claim (version stays 1, old checkpoints re-parse) unchanged.
9. **Rollout #4 comma→"and"** — dogfood acceptance unchanged.
10. **Appended "Editor notes" line** — flags the Deferred entry "Directory moves as single pairs — this ADR-005 if a real change needs them" as ambiguous. Per the deferral rule, checked against the ADR: ADR-005's Decision covers file moves only ("Every `from` must be a tracked path at `headBefore`") and never requires directory moves, so the deferral does not contradict the ADR's Decision or stated requirements. The note is an author-directed flag, not a compat regression.

Unchanged and still passing (previously approved text, untouched by this diff): compatibility matrix rows all carry a status (no `unknown`, no empty cell — item 1); `git ls-tree` is the only added git call with no new floor (item 3 adjacent); no new native module or runtime dependency (item 4); POSIX-path rule 1 handles separators, Windows reserved names, and non-ASCII-relevant controls, with XDG noted in A6 (item 6); Test strategy #12 names macOS for the `yes (unverified)` row, #13 names linked worktrees, #5 names the partial mount row, and Windows is not claimed supported (item 7).

## Findings

None.

## FYI (no severity)

- The editorial pass appended an "Editor notes" block to the artifact itself ("author to clarify"). Whether an editorial pass should leave an unresolved author TODO inside the proposal rather than resolving or omitting it is a proposal-editing/process question, not a compatibility one. The flagged sentence is awkward ("this ADR-005 if a real change needs them") but its deferral is consistent with the ADR.
