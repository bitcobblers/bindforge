# correctness review r1 (dissent pass c1)

verdict: request-changes

Scope: blob diff `ac9cb8a..ec0ea47` of `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/adrs/014-profiles-as-chains.md`, the
revision log, my r1 findings. No review reports or transcripts opened.
Checklist note: the changed text contains no external research URLs to
spot-check; the only cited new fact — shipped `MAX_MANIFEST_ROWS = 256` —
was verified against `crates/core/src/manifest.rs:45` (exists, value 256),
so the Options and composite-cost claims hold.

## Closure check of my r1 findings

- fork empty-delta shell (major): **closed**. The fork bullet now states the
  shell is refused as `patch-empty-deltas` until filled, and Test #5 pins
  the round-trip as `a + diff`, not `fork(a) + diff`. The appended `diff`
  output is always non-empty (equal-layout chains refuse), so it is a valid
  patch.
- merge conflict predicate (major): the predicate is fixed — per-action
  non-representability under the fixed list order, and "a `set` under a
  `clear` composes to the lone `clear`" is correct — **but the exemplar and
  Test #5's pinned pair are still a representable pair** (finding 1).
- diff/merge empty-output edge (minor): **half closed** — diff's
  equal-layout refusal is stated; merge's identity composition is still
  undefined (finding 2).
- mount-at-any-rung suffix contradiction (minor): **closed** — "a suffix"
  withdrawn; the sentence and the ADR's "take the base and patch
  independently" now agree via "the documents singly".
- guard-unit mismatch (minor): **closed** — unit named once ("documents
  opened, one per link"), the conflicting v2 fixture re-issue withdrawn,
  Test #3 is "the sole depth pin". Consistent with `resolve_chain` untouched
  (its predicate can never fire on v2's 0/1-element `derives` anyway).
- diff/merge output `id`/`derives`/`for` (minor): **closed** — signatures
  take the caller's `id`; both bullets pin `derives` and `for`.
- v2 forged-log fixture (minor): **closed** — `forged-log-line.toml` added
  and Test #7 names it as the newline-bearing pin.
- `Chain` publicly buildable (minor): **closed as to the hole** (`links`
  private, "only load_chain builds one") — but the closure strands the
  consumer access the same section promises (finding 3).
- "ten entries" (minor): **closed** — both occurrences now eleven; the
  enumerated order names eleven.
- resolve/resolve_chain naming (nit): not addressed; not gating (FYI).

## Findings

- [major][re-raised] Chain operations — merge bullet, "`merge` refuses ...
  When some action's composed effect has no one-patch form under the fixed
  list order — lower `clear X`, upper `swap X Y` with `Y` bound: the net
  needs the `swap` before the `clear`" — the example is a representable
  pair, and Test #5 ("a non-representable pair (a `clear X` under a
  `swap X Y`) refuses the whole merge") pins it as a refusal. The fixed
  list order is `set` → `clear` → `swap`, i.e. the `clear` comes *before*
  the `swap` — exactly the composition order of lower-then-upper for this
  pair. If `swap` exchanges bindings (entry-table reading): the one-patch
  form `clear = ["X"]` + `swap = ["X Y"]` reproduces the composition on
  every base layout (clear X, then exchange the now-unbound X with Y's
  binding). If instead "an unbound side stays unbound" makes a half-bound
  swap a no-op, the composition collapses to just clearing X, whose
  one-patch form is `clear = ["X"]`. Under either reading the merge emits
  a patch, so the test contradicts the design's own criterion (Options:
  "a conflict is representable iff the composed effect cannot be written
  as one patch"). Disclosed: the wrong pair originated in my own r1 fix —
  the order argument there was backwards. Fix: swap the pair — lower
  `swap X Y`, upper `clear X` (both actions bound in the base): the net is
  X unbound and Y bound to X's former (base-dependent, so nameable only by
  the `swap`) input; the single patch would need the `clear` after the
  `swap`, which the fixed order forbids, and `set` cannot name a
  base-dependent input. Mirror the same flipped pair in Test #5.
- [minor][re-raised] Chain operations — merge bullet: the identity half of
  the r1 empty-output edge is still open. Two identical swap patches
  (`swap X Y` under `swap X Y`) compose to the identity, whose only
  one-patch form is the empty patch `patch-empty-deltas` refuses; unlike
  diff ("chains resolving to equal layouts refuse too"), merge states no
  refusal and Test #5 has no case. Fix: one clause — "a composition that
  cancels to the identity refuses as the empty-patch case" — and one Test
  #5 clause.
- [minor][new] Cross-file loading / Resolution — "`pub struct Chain { links:
  Vec<Profile> }  // only load_chain builds one;`" vs "a consumer takes a
  prefix or the documents singly" and Test #5's "appending its output to
  `a` resolves to `resolve(b, manifest)`" — with `links` private, no public
  constructor, and no accessor in the proposal, no consumer or integration
  test can read the chain's documents or build the extended chain the
  round-trip property names. Fix: either add a read-only accessor
  (`pub fn links(&self) -> &[Profile]`, construction still only
  `load_chain`), or restate the mount route and Test #5 as file-level moves
  (prefix = `load_chain` of the lower link as head; singly = `parse_file`;
  the round-trip writes `diff`'s output into the directory and
  `load_chain`s it as the new head) — one sentence either way, and name
  whatever is public in the `zz_api_probe` list.
- [nit][new] The contract — `generated-by` row: "no ASCII control,
  default-ignorable, or bidi-control characters" — the entry set widened to
  two new refused classes while the per-link promise "each with a named
  fixture above" still names only `generated-by-control-char.toml`
  (ASCII-control branch). Fix: extend that fixture or name one more
  (e.g. a bidi-char label), keeping the "each rule, a fixture" claim true.
- [nit][new] Chain operations — diff/merge bullets, "`fork`'s collision
  rule" — that rule is "refused if it collides with any link id in the
  chain", but `merge` takes no chain (collision with `p1`/`p2` ids?) and
  `diff` takes two (`a`'s links? `b`'s too?). Fix: one parenthetical each
  naming the set the caller's `id` must not hit (merge: `p1` and `p2`;
  diff: every link of `a`, optionally of `b`).

## FYI (no severity)

- My r1 nit — public `resolve_chain` (id-level validation) vs new `resolve`
  (delta application), and mapping ADR Decision Two's "Resolve the chain
  with `resolve_chain`" onto the split — is still unaddressed; not gating.
- Step 2's "the constructed path is always a direct child of the head"
  means the head's directory (the shown `<major dir>/<slug>.toml` makes
  the intent clear); a one-word touch would help.
