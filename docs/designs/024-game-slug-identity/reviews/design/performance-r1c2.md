# Performance review r1 (c2 ratification pass)

verdict: approve

Scope: the `02-proposal.md` blob diff `3012370e → c326ff4a`, then the full current
document. This pass asks only whether the post-r1 revisions broke what this seat
approved or made a section it cited false. No blocker or major finding is open;
the three r1 items are carried unchanged (two minor, one nit) — none of them is
mentioned in the new Revision log entries, so none was rebutted, and none is
blocking.

## Re-ratification checks (what the changed text claims, verified)

- Emission bullet's new code claim — "that counter's 'unreadable or malformed'
  diagnostic fires only when a plugin emitted nothing at all
  (`steam_common.rs:101`)" — is accurate: that branch is gated on
  `!self.roots.is_empty() && games.is_empty() && skipped > 0`, and the message
  counts files, not rows. The revision from a widened per-row diagnostic to
  count-only refusals removes stderr work from the bulk scan path, so it is
  cost-neutral-to-cheaper; nothing I approved in that section was broken by it.
- Test strategy #11's new claim — "Windows is compile-verified by the existing
  `cargo check --target x86_64-pc-windows-msvc` leg, which runs no test binary,
  and macOS has no CI leg" — matches `.github/workflows/check.yml`: one
  `ubuntu-latest` job running that compile check plus fmt/clippy/test, and an
  ubuntu `orchestration` job. The revised gate adds no runner leg and no CI
  wall-clock cost.
- Derivation reorder (Latin fold table now runs before the ASCII case-fold, each
  entry emitting a lower-case base) changes no per-name cost class: the table is
  a compile-time `char` match, consulted once per char either way. The widened
  table is a constant, not a loop. My r1 nit about the multi-pass *shape* of the
  step list is untouched by the reorder — the section is not made false, and the
  single-buffer clause I asked for is still absent.
- `schemas/README.md` budget family (the bullet's stated home) still reads as it
  did when I cited it: every neighbouring entry ends with its aggregate-cost
  composition clause, and the specified `MAX_SLUG_OCTETS` bullet still does not
  have one.
- No changed section adds IO, allocation, a cache, or an event/keystroke-path
  cost. The tree still has no event path (the only binary is
  `apps/discovery-cli`), so checklist item 1 stays not applicable; checklist item
  6 stays satisfied by the Risks row's own "01-research.md Risks: no baseline
  exists; none claimed".

## Findings

- [minor][re-raised] Chosen design → "Emission, refusal and collisions —
  `crates/discovery`", read with the unchanged Risks row "Collision-suffix probe
  loop repeats bound checks many times" — "probe count is bounded by
  distinct-name count, no measurable cost today" — the bound is per name, not per
  run: each colliding name restarts at `base-2` and walks the claimed set, so n
  distinct display names sharing one base cost ≈ n²/2 candidate constructions,
  each a fresh `String` plus an `is_slug_name` and bound check. The revised
  emission bullet does not touch the collision pass, and nothing in the design
  caps rows a plugin returns — `crates/discovery` carries only
  `MAX_FILE_BYTES = keyvalues::MAX_DOCUMENT_BYTES` (steam_common.rs:282), a
  whole-file cap, so the multiplier is the host's `appmanifest_*.acf` count.
  Unchanged, therefore still minor. Fix: replace the row's mitigation with the
  honest shape — aggregate probes are O(n²) in one plugin run's distinct-name
  count, n bounded only by the host's manifest-file count — or store a
  next-free-suffix counter on each base entry of the assigned-slug map so a
  collision consumes one candidate instead of rescanning claimed ones. Either
  keeps one mechanism and adds no constant.
- [minor][re-raised] Chosen design → "Specs, fixtures, ledger" — "a bullet in
  the exported budget family, `MAX_SLUG_OCTETS = 128`, the shared bound for every
  slug-grammar id; sibling of `MAX_NATIVE_NAME_OCTETS` (A8) …" — the family's
  every existing entry carries an aggregate-cost clause (ADR-013's "the shared
  1 MiB / 16-deep budgets still bind first"; ADR-014's
  `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES`; ADR-015's 64 × 16 predicate moves) and
  the specified bullet still has none, although this entry doubles an existing
  bound on values inside arrays those budgets govern. One revision enlarges the
  gap: the bullet is now "the single rationale text that `placement.rs`'s doc
  comment copies verbatim", so the omission propagates from the README into the
  exported constant's own documentation in code. Fix: add one clause to the
  bullet, in ADR-013's words — the doubling adds no aggregate byte ceiling
  because `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` still bind first. One
  sentence, one stated home, no new mechanism.
- [nit][re-raised] Chosen design → "Derivation — `crates/discovery/src/slug.rs`"
  — "in order: fold precomposed Latin via a private `fold_latin_base(char) ->
  Option<char>` match table … then ASCII case-fold (`A-Z`→`a-z`); then map every
  remaining character outside `[a-z0-9]` to `-`; collapse `-` runs; trim `-` at
  both ends; prefix a leading digit with `g-`" — read as steps this is still five
  to six full-string passes over input no field-level cap constrains (the 1 MiB
  cap is on the ACF file, not on the `name` value), and the bound is deliberately
  applied only after the whole slug exists ("Length is **not** truncated
  anywhere"). Today's `normalize_slug` already allocates four intermediate
  `String`s per name, so this is no regression; the reorder neither helps nor
  hurts. Fix: one clause — `derive_slug` folds Latin, folds case, maps, and
  collapses runs in a single `char` loop into one `String` builder, with trim and
  the `g-` prefix done as slice/prepend rather than rewrites, so a name of any
  length costs one buffer.

## FYI (no severity)

- The count-only revision routes an all-refused plugin into the existing
  whole-plugin error branch (`steam_common.rs:101`), whose message reads "N
  file(s) skipped as unreadable or malformed" and lists the selected roots: a
  plugin that emitted only grammar-invalid rows now fails as a host problem
  rather than reporting refused games. That is correctness/observability, not
  this seat's gate.
- "`placement.rs`'s doc comment copies verbatim" is a second physical copy of one
  rationale string, which sits awkwardly against the bullet being "the bound's
  one stated home" — the complexity seat may want a pointer instead of a copy.
