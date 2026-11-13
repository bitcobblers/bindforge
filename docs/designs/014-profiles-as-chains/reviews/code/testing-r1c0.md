# Code review — testing seat — r1 c0

Scope: `git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'`
Judged against `02-proposal.md` (Test strategy #1–#9) and ADR-014. First pass of the round;
no previous findings to re-check.

## Checklist verdict

1. **Test-strategy citations — satisfied.** `crates/core/tests/chain_ops.rs` carries
   `proves: Test strategy #1…#6` at 71 tests, one per listed case (#1: every grammar/entry/
   budget case incl. the 257th delta element and the slug-cap boundaries; #2: each determinism
   rule with its rule number; #3: `Cycle`/`ChainTooDeep`/`ChainLinkMissing`/id↔name/major
   directory, written against `MAX_CHAIN_DEPTH`, never the literal; #4: pin-by-id, pin-by-
   revision, row join in all three lists, preset join; #5: fork/diff/merge incl. the revision-
   log r1 collision and shell cases; #6: the 11-entry hand-built round-trip with delta array
   order, and the untouched v1 `serialize`). #7 is cited in `conformance.rs` (the walk picks up
   `profile/v2`, profiles keep the round-trip path — `rejection()` also runs `resolve_chain`, so
   the v2 `self-deriving-chain.toml` fixture is pinned as `Cycle` exactly as its comment claims),
   #8 in `version_gate.rs` (v3 refusal naming `v1, v2`, majors-equality untouched),
   `grammar_closure.rs` (v2 closed entry set, `derives_` alias), and `zz_api_probe.rs` (the new
   public names). All 7 valid + 37 invalid fixtures from the proposal's contract block ship on
   disk and are exercised by the unchanged walk. #9 has no citing test — see finding 5.
2. **Layers — satisfied.** Unit (`chain_ops.rs`, grammar) and integration (`conformance.rs`
   walk, `version_gate.rs`, `grammar_closure.rs`, `zz_api_probe.rs`) both exist as promised.
3. **Revision rounds — N/A.** No previously rejected blocker/major (first seat pass of r1).
4. **Determinism — satisfied.** No wall-clock assertions, no network, no ports. Scratch trees
   come from the pre-existing `ScratchDir` (pid + counter + nanos nonce — the clock is naming,
   never a value under assertion). `chain_ops.rs` iterates `BTreeMap`/`BTreeSet` and manifest
   row order, so no iteration-order flake surface.
5. **Names describe behavior — satisfied.** e.g. `last_repeated_set_in_array_order_wins`,
   `refuses_a_missing_ancestor_as_chain_link_missing_naming_the_id`.
6. **Gate commands — green** (this repo's `npm run check` equivalent per `check.yml`; the
   TypeScript `tools/orchestration` package is untouched by this diff):

   ```
   cargo fmt --all -- --check          → no output (clean)
   cargo clippy --workspace --all-targets -- -D warnings
                                      → Finished `dev` profile … in 0.05s
   cargo test --workspace --locked → tail:
     Running tests/chain_ops.rs — test result: ok. 71 passed; 0 failed
     Running tests/conformance.rs — test result: ok. 9 passed; 0 failed
     Running tests/grammar_closure.rs — test result: ok. 13 passed; 0 failed
     Running tests/version_gate.rs — test result: ok. 9 passed; 0 failed
     Running tests/zz_api_probe.rs — test result: ok. 3 passed; 0 failed
     (all other suites ok; doc-tests 0) — no failures anywhere
   ```

## Findings

### 1. minor — the step-4 position check is unreachable, and the three tests that cite it pin a different rule

`check_positions` (`crates/core/src/chain_ops.rs:449-475`) cannot fire: `link_below`
(`chain_ops.rs:437-443`) descends only on `kind = Patch` and terminates only on `kind = Base`
(`kind = None` refuses at the same function), so every chain that completes the walk already
satisfies the position rule by construction, and the parse shape rules (`document.rs`
`check_link_shape`) refuse every other kind/derives combination at `parse`.
Consequently the three tests that cite the load-level position promise of Test strategy #2 —
`refuses_a_base_carrying_a_clear_at_load_naming_the_link` (`tests/chain_ops.rs:1162`),
`refuses_a_chain_whose_deepest_link_is_a_patch_naming_the_link` (:1195), and
`refuses_a_base_mid_chain_naming_it` (:1221) — actually drive documents that are
`parse`-invalid (the same shapes the `base-clear-element`, `missing-derives-patch`, and
`base-derives-nonempty` fixtures pin at parse). They pass only because `name_the_link`
(`chain_ops.rs:423-431`) prefixes the link id onto the wrapped parse refusal — the observable
load contract is pinned, but not the rule the comments claim.

Fix: state that step 4 is subsumed by the per-document parse rules (and either keep
`check_positions` with a `subsumed-by-parse` comment or fold it into `link_below`'s
terminating case), and recite the three tests' `proves:` comments to what they do pin —
"a chain link whose document fails its shape rules refuses at load as `InvalidValue` naming
the link". No behavior change is needed; the test labels and the dead branch are the problem.

### 2. minor — two reachable load branches have no test

- `open_link`'s non-profile arm (`chain_ops.rs:354-361`, "must hold a profile document") is
  reachable — e.g. a manifest document named `game-base.toml` in the chain directory — and no
  test covers it.
- `link_below`'s `kind = None` fault (`chain_ops.rs:439`, "a chain link is a `profile/v2`
  document") is reachable for a v1 head in a v1 directory (passes `check_link_version`, then
  faults), and no test covers it.

Fix: add two `chain_ops.rs` cases in the Test strategy #3 section: a chain link holding a
non-profile document refuses as `InvalidValue` naming the file's id, and a v1 profile head
refuses with the named `kind` fault. Both are hostile-chain shapes the design's "one gate per
opened source" posture promises.

### 3. minor — `merge`'s provenance inheritance is unpinned and unsourced

`inherit_provenance` (`chain_ops.rs:650-656`) decides that the merged patch keeps a provenance
label present on exactly one operand, keeps a shared label, and drops a disagreement. The
proposal's `merge` paragraph pins only `id`, `derives`, and `for` inheritance; nothing tests
any of the three provenance arms. An untested semantic not in the contract is exactly what the
fixture-pinned style of this repo guards against.

Fix: either pin it — a merge test asserting the one-sided, equal, and disagreeing provenance
outcomes — or drop the inheritance so the shipped surface matches the proposal.

### 4. nit — `no_head` is a dead defensive branch

`no_head` (`chain_ops.rs:614-619`) is unreachable: `Chain` is only built by `load_chain`, whose
loop pushes at least one link before any `break`, so `head()` is never `None` for a value a
caller can hold (the comment itself says "defensive corner"). Untestable by construction —
a finding under the checklist's dead-branch item, not a pass.

Fix: `debug_assert!`-style the corner or comment it as unreachable-by-construction so no one
later writes a vacuous test around it.

### 5. nit — Test strategy #9 has no citation in the tree

The platform item (three gate commands green on Linux, Windows leg run once by a contributor)
is by nature not a test — I evaluated the Linux leg myself (tail above, green), and the
Windows leg is a Rollout-7 contributor action — but the repo's precedent for such legs is an
explicit note that no test file can pin them (`conformance.rs:39-45` for design 013's fresh-
clone/CI legs). Nothing here carries that note.

Fix: one header comment in `chain_ops.rs` (or the `conformance.rs` design-014 block) saying
#9's legs are the CI job and a rollout check, mirroring the 013 precedent.

## FYI (no severity)

Test strategy #5's phrase "round-trips over the corpus pairs" is exercised with suite-built
chains (`diff-roundtrip-a`/`-b` scratch trees), not the shipped `schemas/profile/v2/fixtures/`
files; the substance — write the diff beside `a`'s head, `load_chain` it, compare to
`resolve(b, manifest)` — is pinned, so this is only wording, for whoever reads fixture-vs-suite
pairing literally.

## Verdict

approve — no blocker or major findings; the five minors/nits are test-label, coverage, and
dead-branch cleanups that do not change what the shipped behavior contract asserts.
