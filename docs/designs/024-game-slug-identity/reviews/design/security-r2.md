# security review r2

verdict: approve

Scope: single editorial confirmation pass of round 2 over
`docs/designs/024-game-slug-identity/reviews/design/editorial-r2.diff`, with
`02-proposal.md`, `docs/adrs/024-game-slug-identity.md`, and the Revision log as
inputs. No previous security findings were open.

## Checklist disposition

1. Trust boundaries — unchanged by the diff. Every hunk is language-only
   (wording, articles, voice, backticks on unit ids, line rewrapping); the
   classification of plugin/CLI-boundary rows and shared-file inputs is
   untouched. Not re-litigated: no changed text alters a trust classification.
2. Parsing untrusted data — not applicable to this diff; no size-limit or
   schema-validation text changed. The bound wording ("checks the final value
   **against** `bindforge_core::is_slug_name` and the bound") preserves the
   check; only "with" → "against" and "install-dir" → "install-directory".
3. Path handling — the diff leaves "and no path (`024-L1`)" and the grammar
   statements intact; the backticks added around `024-L1` / `024-L3` conform
   the unit handles to the ADR's code-span convention. Nothing new is
   interpolated into a shell or path context.
4. Secrets and PII / echo discipline — the refusal-echo sentences survive the
   rewrite verbatim in meaning: "refusals echoed only through core's
   escape-and-truncate renderer" (Goals) and the stderr-line shape
   `<escape_and_truncate(value)>` and no path. "nothing is transliterated
   multi-character" → "nothing maps to more than one character" is a clearer
   restatement of the same constraint, matching "no multi-character
   transliteration" in Derivation.
5. Dependencies — the diff names no new dependency; the rejected
   `unicode-normalization` option is unchanged. No advisory lookup required
   for a language-only pass (bindforge-core is in-workspace).
6. Privileges — untouched by the diff.
7. Fail-closed behaviour — "returns `None` on refusal so the game is never
   added", "an invalid row is refused: dropped", and the count-only
   plugin-side rule are preserved; the comma move in "is the CLI-boundary line
   of the next section alone" does not change which boundary names refusals,
   and that rule is a recorded Revision-log decision, not new text from this
   diff. Anchors re-verified: `one_line` escape-and-truncate at
   `crates/core/src/error.rs:166-186` (escapes controls, truncates with `…`,
   single line); the skip diagnostic at `crates/discovery/src/steam_common.rs:101`
   fires only when `games.is_empty() && skipped > 0`, matching the reworded
   "fires only when a plugin emitted nothing at all".

## Findings

None. No hunk in the editorial diff creates, weakens, or relocates a security
property; no previously approved passage is falsified by the changed text.

## FYI (no severity)

- The backticking of `024-L1` and `024-L3` aligns those unit handles with the
  ADR's "one trailing code span" handle convention — a convention/correctness
  improvement, noted for those seats.
