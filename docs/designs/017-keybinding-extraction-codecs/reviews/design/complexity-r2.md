# complexity review r2

scope: the single editorial confirmation diff of round 2 —
`docs/designs/017-keybinding-extraction-codecs/reviews/design/editorial-r2.diff`
(5 hunks) — read against `02-proposal.md`, ADR-017, and the proposal's Revision log.

verdict: approve

## Findings

None. All five hunks are language-only and meaning-preserving; none makes a
section the complexity seat relies on false, and none adds, removes or renames a
mechanism, constant, dependency edge or status.

Hunk-by-hunk confirmation (each verified against the current text and, where the
sentence carries a checkable fact, against the tree):

1. §The codec face — "A family id is `[a-z][a-z0-9-]*` **of** at most
   `bindforge_core::MAX_SLUG_OCTETS` octets": inserted preposition only. The
   cited constant exists in core and is the same bound the manifest `codec`
   entry rides (`crates/discovery/tests/slug_tests.rs:23,179` uses
   `bindforge_core::MAX_SLUG_OCTETS`; `zz_api_probe.rs:100-105` confirms it is a
   published core constant), so the reuse claim the sentence carries still holds.
2. §Read discipline — "the product also **caps** the *opens*" replacing
   "ceilings the *opens*": same verb as the clause it rests on ("§The first
   family caps a tree's reads at `MAX_CANDIDATE_FILES`"), same arithmetic
   (`17 × 32 × MAX_BINDING_FILE_OCTETS` = 544 MiB), same `(MAX_USER_DIRS_PER_GAME
   + 1) × MAX_CANDIDATE_FILES` product, still stated in the
   `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` shape the README already uses
   (`schemas/README.md:132-133`). No budget changed.
3. Test strategy 3 — "two fixture roots holding **9- and 8-digit** directories":
   hyphenation only; the math the cap claim depends on is unchanged
   (9 + 8 = 17 trees against `MAX_USER_DIRS_PER_GAME = 16`, 17th past the cap).
4. Test strategy 6 — "a platform directory name other than **those the fixtures
   use**" replacing "other than the fixtures' others": the no-platform-constant
   requirement (A15) is intact, so the walk stays free of a hard-coded directory
   name rather than gaining a knob.
5. Test strategy 16 — "and **pins** that the only `Cargo.lock` delta is the
   `bindforge-codecs` stanza and the `discovery-cli` dependency edge": the verb
   the sentence was missing. The pinned claim now reads identically to §`crates/codecs`
   layout's dependency claim, and both check out against the tree: `crates/*` and
   `apps/*` are the workspace globs (`Cargo.toml:12`), `discovery` is a path
   package (`crates/discovery/Cargo.toml:2`), and `serde_json = "1.0"` is already
   declared per-crate rather than workspace-wide (`apps/discovery-cli/Cargo.toml:13,22`;
   `crates/discovery/Cargo.toml:16`), so a codecs dev-dependency adds no new
   external package and moves no resolved version.

Checklist disposition: (1) reuse — no new abstraction where one exists; the
reader posture, the table renderer's width algorithm, `escape_and_truncate` and
the shared budget constants are all reused, confirmed again for the two constants
the diff touches. (2) Goal traceability — the diff introduces no moving part.
(3) smaller design — unchanged by a language pass. (4) dependencies — unchanged,
and re-verified. (5) deferrability — unchanged. (6) knobs with no consumer /
one-implementation abstractions — unchanged; the appid pass remains the one
mechanism shipped ahead of its data, and it is already carried as a Risk
("Certain (A21) · Low") with the Revision log's recorded decision to keep the
seam pinned by a synthetic family rather than invent appids. That rebuttal is
sound: inventing two AppIDs would be fabricated content, and the seam's cost is
one trait method plus a pass order that `017-R4` otherwise has to design in. Not
re-raised. (7) value check — the design's complexity still looks proportionate:
one crate, one format, two path templates, one family, four statuses, and every
budget in the table has a named consumer.

## FYI (no severity)

- Test strategy 16 lists `ScanRecord` among the names the `bindforge-codecs`
  `zz_api_probe` resolves, but §Discovery plumbing puts `ScanRecord` in
  `crates/discovery/src/game.rs`; whether the codecs API probe may assert a
  neighbouring crate's type (or should assert a re-export) is a correctness/API
  question, not a complexity one. Untouched by this diff — the hunk only supplied
  the verb.
