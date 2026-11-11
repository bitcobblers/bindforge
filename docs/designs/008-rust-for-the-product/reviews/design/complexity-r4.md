# complexity review r4

verdict: approve

Scope: `reviews/design/editorial-r4.diff` (the r4 editorial pass over
`02-proposal.md`), read against `02-proposal.md` and
`docs/adrs/008-rust-for-the-product.md`. Inputs per the caller: the proposal,
the ADR, the Revision log, and my previous complexity findings (none — the
editorial phase carries no prior complexity findings).

## Findings

None.

## Checklist evaluation

The diff is language-only across four passages; none of it adds, moves, or
reshapes a mechanism:

1. **New package/module/interface reuse:** none introduced. The diff touches
   the Goals bullet list, one sentence of the Gates CI paragraph, one
   Risks-table mitigation cell, and one Rollout bullet. No new artifact names
   appear on the `+` side that were not on the `-` side.
2. **Moving parts trace to a Goal:** no moving parts added or removed. The
   `advisories.yml` / `deny.toml` / `cargo-deny` machinery touched by the
   Gates rewrite is unchanged in substance — the edit only removes a
   redundant trailing "it commits" ("so it commits to the only recurring look
   at the lockfile it commits:" → "so it commits to the only recurring look
   at the lockfile:"). The obligation still traces to the ADR consequence
   ("CI must run both", lockfile integrity) and the Revision log's r1c3
   decision, which it restates faithfully.
3. **30–50% smaller design:** the diff is net word-count-reducing (Goals
   bullet collapsed to a noun phrase parallel to its siblings; one redundancy
   deleted; Risks mitigation cell switched passive → active with identical
   content: codec conformance fixtures still pin `toml_edit`'s
   non-preservation gaps (A21) when `crates/adapters` lands, ADR-012 still
   takes the obligation). Nothing the smaller wording implies contradicts the
   ADR or the Goals.
4. **New dependencies:** none. No crate, action, or tool name appears on a
   `+` line that was absent on the matching `-` line.
5. **Deferrable second half:** unchanged by the diff; rollout step ordering
   (CI in step 4, advisories workflow marked Rollout 4) is untouched.
6. **Knobs/abstractions without consumers:** no new knobs. Rollout 5's edit
   is an inserted list conjunction ("..., and tests-under-each-crate") — the
   `repo-practices` bullet list's content is identical to what Revision-log
   entry "Chosen design → Gates (r2)" commits to.
7. **Value check:** complexity is proportionate and unchanged by this
   pass. Every element the touched text describes — two CI jobs mirroring the
   ADR's two toolchains, one scheduled non-blocking advisory job rather than a
   per-PR advisory gate, one shared Node floor stated once and mirrored —
   traces to an ADR-008 clause or consequence, and the diff made the
   statements slightly leaner, not heavier.

Cross-checks performed: `+`-side text matches the current proposal at
Goals bullet 5, the Gates CI paragraph, the Risks "Adapter file-write bugs"
row, and Rollout step 5; no deferred item (sqlx offline cache → ADR-014,
desktop Node floor → desktop design) was altered, so no Deferred contradiction
with the ADR was introduced. The Revision log shows these areas were settled
in r1c3/r2; the editorial wording does not reopen any rejected decision.

## FYI (no severity)

- None. Nothing in the diff falls outside the complexity seat that looked
  like it belonged to another specialty.
