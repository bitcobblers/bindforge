# correctness review r1 — dissent pass c2

verdict: approve

Scope: `git diff ec0ea477 2aae9c9e` (blob ids of `02-proposal.md`) plus the full
current document and `docs/adrs/014-profiles-as-chains.md`. Checklist applied:
`references/correctness.md`.

## Re-check of my r1 findings (the changed text is the answer to these)

1. **major — merge's non-representable exemplar was backwards — CLOSED.** The
   bullet now reads "lower `swap X Y` with both bound in the base, upper
   `clear X`: the net needs the `swap` before the `clear`". Verified against the
   determinism rules (`set` → `clear` → `swap`, rule 2) and `swap`'s semantics
   (rule 3, "unbound sides staying unbound"): base X=a, Y=b → after the lower
   swap X=b, Y=a → after the upper `clear X` the net is X unbound, Y bound to
   `a`. The only fixed-order one-patch candidates fail: `clear=[X]` +
   `swap=[X Y]` yields X bound to b, Y unbound; the correct form
   `set Y a` + `clear X` needs the base's binding of X, which `merge(p1, p2, id)`
   is not given (no base document, no manifest) and so cannot be named. The
   refusal matches the Options criterion ("a conflict is representable iff the
   composed effect cannot be written as one patch"), and Test strategy #5 now
   mirrors the same flipped pair.
2. **minor — identity half of the empty-output edge — CLOSED.** "a composition
   that cancels to the identity refuses as the empty-patch case" plus the
   matching Test #5 clause. Syntactically detectable without the base (e.g.
   `swap X Y` under `swap X Y`), and it ties to the grammar's own refusal
   (`patch-empty-deltas.toml`, "an empty patch is the vacuous link the guard
   refuses"). Consistent with diff's "chains resolving to equal layouts refuse
   too".
3. **minor — `Chain` closed with no route for the promised consumer access —
   CLOSED.** The author took the file-level route: "a consumer mounts a prefix
   by `load_chain`ing a lower link as head, or reads the documents singly with
   `parse_file`". Verified expressible through the shipped public surface —
   `parse_file` is a public re-export (`crates/core/src/lib.rs:67`), the head's
   single `derives` element plus the same-major-directory rule gives a lower
   link's path, and a prefix loaded that way is still a valid chain (every link
   sits in the same major directory). Test #5's round-trip is now "write it
   beside `a`'s head, `load_chain` it as the new head" — expressible, and no
   accessor is needed anywhere, so the private `links` stays consistent.
4. **nit — bidi branch had no fixture — MOSTLY CLOSED.** `generated-by-bidi-char.toml`
   is now named. Residual half: see finding [nit][re-raised] below.
5. **nit — "(`fork`'s collision rule)" left the refused id set unspecified —
   CLOSED.** diff: "against every link id of `a`"; merge: "against `p1` and
   `p2`". Both sets are checkable from the operands the functions are given.

## Findings

- [minor][new] Chain operations — merge bullet: "the output carries the caller's
  `id` (`fork`'s collision rule against `p1` and `p2`) and inherits `derives`
  and `for` from `p1`" — the set just named is one member short, and it is the
  member that matters most: the merged patch inherits `derives = [base id]`, and
  by the directory rule it must live in that base's own directory as
  `<id>.toml`. A caller `id` equal to the shared base id therefore names the base
  document's file, so the consumer's write replaces the base; core only notices
  afterwards, at `load_chain`, as a repeated-id `Cycle` on a chain whose base is
  gone. diff guards exactly this ("against every link id of `a`"), so the two
  operations are asymmetric. My r1 fix supplied this set and, on re-reading, is
  itself incomplete — that is stated here rather than blamed on the revision.
  Fix: extend the parenthetical, no new mechanism — "`fork`'s collision rule
  against `p1`, `p2`, and the `derives` base id inherited from `p1`" — and mirror
  it in Test strategy #5's collision clause.
- [nit][re-raised] The contract — `generated-by` row: "no ASCII control,
  default-ignorable, or bidi-control characters" — three refused classes, two
  fixtures: `generated-by-control-char.toml` and the newly named
  `generated-by-bidi-char.toml` cover the ASCII-control and bidi branches, and
  the default-ignorable branch is still pinned by nothing, while the same section
  makes the corpus the thing that "makes `profile/v2` supported". Test strategy
  #1 also still enumerates only "`generated-by` past 128 octets or
  control-character-bearing", which does not name either new class. The bidi half
  of my r1 nit is fixed as prescribed; this is the remaining half. Fix: name one
  more fixture (e.g. `generated-by-default-ignorable.toml`, a soft-hyphen label)
  or state that the control-char fixture carries a default-ignorable codepoint,
  and let Test #1 say "control-, bidi-, or default-ignorable-character-bearing".

## Checklist notes

- **Assumption register (item 1):** untouched by this diff; no `verified` row was
  re-scoped by the revised text, and A15/A16/A18/A21 are still cited exactly
  where the changed passages lean on them.
- **Anti-hallucination gate (item 2):** the diffed text adds no external source;
  every new assertion resolves in-document (determinism rules 2–3, the directory
  rule, `patch-empty-deltas`) or to ADR-014, which I read. Decision Three's
  "mount at any rung — take the chain whole, or take its base and patch
  independently" is what the new mount sentence implements (documents singly via
  `parse_file`, prefixes via `load_chain` of a lower head); it does not ask for a
  suffix without its base, so no contradiction.
- **Interfaces (items 3, 5):** `load_chain`, `parse_file`, `Chain`, `fork`,
  `diff`, `merge` appear under one name and one shape in Cross-file loading,
  Chain operations, Test strategy, and Rollout; no accessor was introduced, so
  the private `links` and the public surface agree.
- **Deferred (item 5 of the skill):** the diff touches nothing handed to
  ADR-015/016/017/018; the directory-as-bundle-shape sentence stays consistent
  with the ADR's "not decided here" list.

## FYI (no severity)

- The r1 revision log entry was consolidated and no longer records several r1
  decisions the body still asserts (the withdrawn v2 depth fixture, "guard unit
  is documents opened", `MAX_MANIFEST_ROWS` in the composite, entry count eleven,
  `generated-by` as a public label). The body is unchanged and correct on all of
  them, so this is log-completeness, not correctness — the editor/process seat's
  call.
- The Options merge bullet's sentence "a conflict is representable iff the
  composed effect cannot be written as one patch" reads as a double negative
  ("exists iff" / "unrepresentable iff" is what is meant); meaning is unchanged,
  so it belongs to proposal-editing.
- Some edited lines carry continuation indentation that does not align with the
  bullet text (`   id]`, `for` the joined` / `    composition refuses as the
  empty-patch case`); cosmetic only.
