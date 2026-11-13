# Performance review r1 (dissent pass c1)

verdict: approve

Scope: `git diff bbebe401 fe5671d` of `02-proposal.md`, then the full current
document. Checked first against my own r1 findings; everything else reported
only where the changed text makes it true.

## Closure check on my r1 findings

- **major — `presets` outside the named budget family: CLOSED.** Budgets now
  reads "`MAX_MANIFEST_ROWS = 256` bounds each array-shaped entry (`actions`,
  `dead-keys`, `docked-actions`, `presets`)" — no new constant, as asked. This
  closes the actual performance hole, not just the fixture gap: with the
  63-octet slug bound on a preset id (Identity grammar), the superlinear path is
  now `256 × 63 octets ≈ 16 KiB` of preset data, so the per-element slug check
  and the "each preset id appears once" uniqueness scan run on n = 256, not the
  ~10⁴ the 1 MiB budget allowed. Every array in the entry table is now covered
  by a named cap (`exclusive-groups` by `MAX_CONSTRAINT_GROUPS`), so total
  element count across the document is bounded at roughly 4×256 + 64×16, which
  bounds the uniqueness/disjointness scans and ADR-015's membership lookups
  without needing a hash index. `presets-past-cap.toml` pins it in the corpus
  (and it refuses at parse, so the "corpus is registry-free" split still
  holds), and Test strategy #4 now names "a 257th preset id". The Risks row was
  updated to match — "The four named budgets cap every array in the entry table
  before any join" is now true, where "the four named budgets refuse before any
  join" was not, since `presets` was uncapped.
- **minor — cap values with no derivation: CLOSED, with one residue (nit
  below).** The added sentence ties the row caps and the name cap to the
  research baselines quoted in the assumption register (40 registry ids, A7;
  TF2-scale bind lists, A9).

## Checklist evaluation

1. Input-path latency: no per-keystroke or per-event cost is added — manifests
   are read by `load_manifest` at translation/validation time, one file read
   with the size budget applied before the read. Pass.
2. Structure costs: the one unbounded n in the format (`presets`) is now
   capped; every remaining pairwise rule (native-name uniqueness, one group per
   input id) runs over a capped n. Pass.
3. Memory growth: `Manifest` is bounded by the four caps plus the 63-octet slug
   bound and the 51-char fixed-width fingerprint; no translation table or cache
   is introduced. Pass.
4. Startup cost: nothing loads at boot; the only added cost is the fixture
   corpus under the existing three gate commands. Pass.
5. Caching/invalidation: no cache; invalidation is the identity + monotone
   `revision` + bump rule, and the caps do not interact with it. Pass.
6. Research baseline: quoted for the row and name caps; two caps ride the same
   sentence without their own baseline (nit below). No unexplained gap between
   the caps and the surveyed numbers.

## Findings

- [nit][re-raised] Chosen design → Rules → Budgets — "Each cap sits well above
  the baselines surveyed for it (40 registry ids, A7; TF2-scale bind lists and
  multi-word native names, A9, A13)" — the sentence claims a baseline for
  *each* cap but names baselines for only two of the four: `MAX_CONSTRAINT_GROUPS
  = 64` and `MAX_GROUP_MEMBERS = 16` carry no stated observation, and the
  baselines offered are about bind lists and display names, not about group
  counts. The claim is defensible from A9 (the surveyed TF2 key set is ~30
  named inputs, so 64 groups × 16 members sits above anything surveyed), but the
  reader has to reconstruct that. Separately, `A13` is marked `unverified` in
  the assumption register, so naming it as a surveyed baseline for the 128-octet
  name cap anchors that cap on an unconfirmed observation. Fix: extend the
  parenthetical with the one fact the group caps sit above (e.g. "no surveyed
  game constrains more than ~30 named keys, A9, so 64 groups × 16 members is
  above any surveyed geometry"), and drop `A13` from the baseline list or
  rewrite as "multi-word native names, A9's `MOUSE1`–`MOUSE5`-style names, plus
  the unverified A13 survey" so the cap's strength does not rest on an
  unverified row.

## FYI (no severity)

- The example manifest's parenthetical was shortened in this diff to "see
  Registry growth order, and Deferred" for two different facts: `Tab` is
  genuinely Deferred (vocabulary major), while an unregistered `ui.inventory`
  id is handled in-body by Registry growth order, not Deferred. Combined into
  one pointer it reads as if both are deferred; correctness/consistency seat's
  call, no performance consequence.
- The model stores `dead_keys`, `exclusive_groups`, `docked_actions` as `Vec`,
  so "a set-membership lookup over loaded data" is a linear scan (~1.5k
  elements worst case under the caps) at chain-build time. Bounded and off any
  event path, so not a finding; worth a note if an implementation round wants
  to say "lookup" without qualifying it.
