# Code review — correctness, round 1 dissent pass (c1)

Seat: correctness (faithful to the design).
Scope: `git diff 4844669384fd046eebfd47960283e178990617a4..HEAD -- ':(exclude)docs/designs'`
(crates/core/src/{chain_ops,document,error,lib}.rs, crates/core/tests/chain_ops.rs,
schemas/profile/v2/spec.md), read against `02-proposal.md` and
`docs/adrs/014-profiles-as-chains.md`. Per the caller's input restriction I did not
read `03-build-notes.md` or any other seat's report; the proposal-vs-code deviation
check below is therefore made against the proposal and the shipped `spec.md`.

Verification run in-tree: `cargo test --workspace --locked` — all suites green
(`chain_ops.rs` 74/74); `cargo clippy --workspace --all-targets --locked` — clean.

## Round-1 findings, re-checked

**1. blocker — `emit_composition` partner guard keyed wrong: RESOLVED.**
`crates/core/src/chain_ops.rs:772-802` now reads `if !reciprocal { return
Err(composition_fault(action, other)); }` then `if exchanged.contains(other) {
continue; }` then `exchanged.insert(action); emitted.swap.push((action, other))`
— exactly the shape requested. I re-derived the guard over the cases that
previously broke and over the ones that must keep breaking:
- `p1 = swap a b`, `p2 = set c k` → one patch, `set [c k]` + `swap [a b]` (test
  `merge_keeps_an_exchange_the_second_patch_does_not_touch`, chain_ops.rs:2060,
  which also writes the patch beside the base, `load_chain`s it, and asserts its
  layout equals the stacked `game-base ← p-one ← p-two` layout — the pin my fix
  asked for, including element order).
- `p1 = swap a b`, `p2 = clear a` → a→`Base(b)`, b→`Unbound` → the reciprocity
  check fails → refuses; still pinned at :2207 (`swap X Y` under `clear X`),
  naming `move.a`.
- three-cycle (`swap a b` + `swap b c` in one patch) → `composed[b] = Base(c)` ≠
  `Base(a)` → refuses: correct, a 3-cycle has no one-patch form under pairwise
  `swap`.
- identical pair in both patches → cancels → the empty-patch refusal (:2236). ✓
The emission is additionally bounded by the built-in proof at :804-818
(re-apply the emitted lists over the same symbols, demand identical symbols for
every touched action), so no mis-emission can escape as a wrong patch — the
residual risk from the old guard was over-refusal, and that is now closed too.
`emitted.swap` order follows `merge_touch_order` (:694-705), i.e. the proposal's
"first-patch touch order, then second-patch-only actions". ✓

**2. major — shipped `Cycle` message edited to satisfy a test: RESOLVED.**
`crates/core/src/error.rs:149` is back to `write!(f, "chain derives from itself")`;
`grep -rn "derives from itself" crates` shows the single shipped spelling and no
test anywhere asserts the removed "(`Cycle`)" wording (A15's byte-exactness
surface is intact). The needle list is gone from the cycle case
(`crates/core/tests/chain_ops.rs:1289-1303`): it now pins the *variant* plus the
one-line/control-free shape, which is what Test strategy #3 pins. The new module
doc paragraph at `error.rs:17-21` records the rule that broke, which is the right
durable fix.

**3. minor — one-directory rule re-implemented with a second variant: RESOLVED.**
`check_link_version` is deleted (chain_ops.rs, old :429-452 region) and both open
paths go through `check_dir_version(directory, &link.version,
&format!("{expected}.toml"))` (chain_ops.rs:405). `document.rs:1346-1361` answers
`InvalidValue` naming file, declared major, and directory; the chain now answers
the same variant as `reach.rs:160` and `placement.rs:160` for the identical fault,
which is exactly A5's "one definition … cannot drift into two spellings"
(document.rs:1363-1367). `schemas/profile/v2/spec.md:120-127` was updated to
state `InvalidValue` and to keep `UnsupportedVersion` for an unshipped major — so
the third-party contract and the code agree, and the wrong test comment is
replaced (chain_ops.rs:1456-1466) with one that names both halves of the
distinction. The test pins the variant with `matches!` plus needles the shipped
message really carries, and asserts the scratch path stays out of it.

**4. minor — merge carried provenance labels: RESOLVED.**
`inherit_provenance` is deleted and `merge` no longer sets the two labels
(chain_ops.rs:355-359: only `new_patch`'s deltas are assigned); `new_patch`
(:650-663) leaves both `None`, so `fork`/`diff`/`merge` are consistent. The new
case `merge_leaves_the_provenance_labels_absent_from_its_output` (:2135) pins the
absent labels even when both operands agree — the stronger form, matching ADR
Decision Four. `merge`'s doc comment (:302-309) states the rule.

**5. minor — missing head answered `ChainLinkMissing` with a raw stem: RESOLVED.**
`load_chain` (:122-151) now opens the head through `open_head` (:371-374, straight
to `parse_file`, whose `Io` payload carries the path) and keeps
`open_ancestor` (:381-389) — the only producer of `ChainLinkMissing` — for links
the walk constructed from a `derives` element. New case
`refuses_an_absent_head_as_the_read_refusal_carrying_its_path` (:1378) pins the
variant, `source_path == head`, and the path absent from the message (A13). The
ancestor pin (:1348-1371) is untouched, `error.rs:60-71` and `spec.md:128-136`
were corrected to the same rule, and the walk's `Cycle`/`ChainTooDeep` accounting
is unchanged by the restructure (the depth check still sits after the push and
before the next open, so "exactly 32 opened resolves, one more refuses" still
holds — tests :1310-1345 green).

**6. nit — merge's base assumption stated nowhere: PARTIALLY RESOLVED (still open,
see finding 1 below).**

**7. nit — v2 writer skipped `kind`/`for` when `kind` was `None`: RESOLVED.**
`document.rs:643-650` writes `kind` and `for` unconditionally on the v2 branch
(`serialize` still branches on `version` alone at :482/:495, so v1's bytes stay
byte-exact, A15), and the precondition is now documented on both
`serialize_profile_link` (:629-634) and `serialize` (:468-471). I checked the
claimed consequence: `parse_profile_kind` (document.rs:853-861) refuses `kind =
""` as `InvalidValue` naming `kind`, never a guessed shape — so a hand-built
model cannot serialize to text that re-parses as a different document.

## Findings

### 1. nit — the merge base assumption is now in the rustdoc but still not in the contract prose

- location: `crates/core/src/chain_ops.rs:302-309` (`merge` doc), `:720-731`
  (`apply_lists` doc); contract side: `02-proposal.md` → Chain operations
  (`merge`, 460-475)
- problem: my round-1 ask was one sentence in the merge section stating that the
  composition assumes the base binds both sides of any exchange either patch
  names, and that a merge needing otherwise refuses. That sentence now exists in
  the rustdoc for `merge` and for `apply_lists`, and the refusal behavior it
  describes is correct and deterministic (`set X a` under `swap X Y` refuses, as
  does `swap X Y` under `clear X`). But it is still absent from the proposal's
  merge contract, and `schemas/profile/v2/spec.md` carries no merge section at
  all — so the only normative text a third-party implementer has for `merge`
  remains the proposal, whose "composes … exactly, not heuristically" does not
  disclose that a pair with one side re-bound by the other patch is a refusal
  rather than a composition. The docs half of the fix is out of this diff's
  declared scope (`docs/designs` excluded), so nothing in the code is wrong; this
  is a documentation gap that survives the revision.
- fix: add to the proposal's merge paragraph — "the base is assumed to bind both
  sides of any exchange either patch names; a composition whose answer would
  depend on the base's bindings has no one-patch form and refuses" — and keep the
  rustdoc sentence beside it. (If merge semantics are meant to be conformable by a
  non-Rust implementer, that sentence belongs in `spec.md` too, where the rest of
  the load-level contract already lives.)

## Checklist walk

1. **Named components exist and match** — `Chain`, `load_chain`, `resolve`,
   `fork`, `diff`, `merge`, `ResolvedBinding`, `ProfileKind`,
   `CoreError::ChainLinkMissing`, `MAX_PATCH_DELTA_ELEMENTS` /
   `MAX_GENERATED_BY_OCTETS`, `profile_entries(version)` selector: present with
   the proposal's signatures; `merge(p1, p2, id) -> Result<Profile, CoreError>`
   unchanged. `lib.rs`'s public-surface paragraph was updated to the new
   head/ancestor refusal split, so the re-export doc matches the code.
2. **Assumption register honored** — A15 (v1 serialize, existing `Cycle`
   wording), A5 (one shared directory rule, one variant), A18 (named refusal only
   for walk-constructed links), A13 (path only in the `Io` payload), A21 (cost
   paragraph: the action join is now one per-resolution lookup rather than a scan
   per touched action — behavior-identical, refusal order unchanged because the
   `BTreeSet` only answers "is this a row?").
3. **Deviations** — against the proposal, the only gaps I found in this diff are
   the two the spec.md edit already declares (dir-major variant, head refusal)
   and finding 1. Not evaluable against `03-build-notes.md`: the caller fixed this
   seat's inputs and excluded it, so item 3/5 of the checklist were checked
   against the proposal and the shipped spec only — FYI, no severity.
4. **Scope creep** — none beyond the round-1 asks: the diff's only non-asked
   change is the `rows: BTreeSet<&ActionId>` lookup in `check_joins`
   (chain_ops.rs:542-552), which implements the cost paragraph the proposal
   already states ("one per-resolution lookup of the `MAX_MANIFEST_ROWS`-capped
   action table") and changes no refusal, message, or ordering.
5. **Build-notes claims** — out of this pass's declared inputs (above).
6. **Error paths fail closed** — head/ancestor/pin/id/depth/cycle/join/merge
   refusals all still refuse, verified by the green suite; the merge emission is
   additionally self-proven before it can be returned.
7. **Round-1 blocker/major actually addressed** — yes: blocker 1 and major 2 are
   fixed in the code (not in the tests), each with the pin my fix named.

## FYI (no severity)

- `check_dir_version` answers `Ok(())` when the head's directory is not named for
  a major (`document.rs:1351-1353`), so a chain in an arbitrary directory loads.
  That is now precisely what `reach.rs:160` and `placement.rs:160` do, which is
  what the A5 fix asked for; `spec.md:120-122` reads as if the head must sit in a
  `vN/` directory. Pre-existing, behavior unchanged by this diff — flagging only
  so the wording is not mistaken for a rule the loader enforces.

## Verdict

approve — 0 blockers, 0 majors open; one nit carried (contract prose only).
