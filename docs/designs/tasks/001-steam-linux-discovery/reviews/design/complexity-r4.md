# complexity review r4

verdict: approve

Scope: the editorial-r4 diff only (`reviews/design/editorial-r4.diff`), checked
against `02-proposal.md` as it now stands and `docs/tasks/T-001-steam-linux-discovery.md`.
This seat's previous findings: none. The Revision log was read; no finding here
re-opens a rejected entry, and the Deferred section is untouched by the diff
and does not contradict the task document.

## Checklist walk

1. New package/module/interface introduced by the diff: none — all seven hunks
   are language-only (serial commas, `does`→`would`, `it`→`from_roots()`,
   behaviour→behavior, one line wrap). Not applicable.
2. Moving parts traced to Goals: no structural text changed; §2's rewritten
   sentence ("the seam that tests and `--steam-root` use; `from_host()` is a
   thin wrapper over `from_roots()`") resolves an ambiguous pronoun without
   changing the seam design. Not applicable.
3. 30–50% smaller design: the diff changes no size-relevant text. Not applicable.
4. New dependencies: none; §10's no-new-dependency posture and the
   `Cargo.lock` byte-identity claim are untouched by the diff. Not applicable.
5. Deferrable second half: unchanged. Not applicable.
6. Knobs/abstractions with one implementation: unchanged. Not applicable.
7. Value check: the design's complexity remains proportionate to what T-001
   brings; the editorial pass adds no mechanism and removes none.

Meaning-preservation spot checks all pass: "each one" → "each such question"
still claims settlement of exactly the T-001-tracing questions; "does" →
"would" matches the conditional in the same sentence; the behavior/behaviour
change is consistent (no other British spelling remains in scope of the diff).

## Findings
- [nit][new] Assumption register, row A9 — "; so, `home` is read from
  `std::env::var_os(\"HOME\")`" — the editorial pass inserted a comma after
  the conjunction `so` following a semicolon; the pre-diff "; so `home` is
  read from …" read correctly, and the new comma makes `so` scan as a
  sentence adverb, a small regression this round introduced. Fix: delete the
  comma after `so` in A9; no other text changes.

## FYI (no severity)
- The pass applies the serial comma in Context, §10 and the A9 row but leaves
  other three-plus lists serial-comma-free (e.g., A8's "the Flatpak … tree and
  the Snap … tree", §4's "unbalanced braces, an unterminated string, a `[`
  conditional or `#base` line"). A consistency question for the editorial
  seat's own style guide, not a complexity concern.
