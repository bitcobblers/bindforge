# Code review — testing seat, r1 ratification pass c2

Scope: `git diff 4844669384fd046eebfd47960283e178990617a4..HEAD -- ':(exclude)docs/designs'`
(6 files: `chain_ops.rs` src/tests, `document.rs`, `error.rs`, `lib.rs`,
`schemas/profile/v2/spec.md`), read against the full current `02-proposal.md`
and `docs/adrs/014-profiles-as-chains.md`. This pass checks only that the
post-approval revisions did not break what the testing seat approved at r1c1.

## Verdict: approve

No blocker or major. The revisions (head/ancestor open split with `Io`-vs-
`ChainLinkMissing`, `check_dir_version` reuse, `emit_composition` reciprocal-
pair fix, provenance inheritance dropped, unconditional `kind`/`for` in the v2
writer, `Cycle` message reverted to design-008 bytes, action-row lookup set)
did not weaken or invalidate any test the seat approved at r1. My two r1 minors
remain open but are untouched by the revisions — carried below for continuity,
not re-litigated as new. Nothing the revisions changed made a new testing
finding true.

## Gate run (checklist item 6)

- `cargo fmt --all --check` — silent, clean.
- `cargo clippy -p bindforge-core --all-targets` — no warnings;
  `Finished `dev` profile [unoptimized + debuginfo] target(s)`.
- `cargo test -p bindforge-core` — every suite ok; tail:

  ```text
       Running tests/chain_ops.rs (target/debug/deps/chain_ops-19655eeb5c3dd376)
  test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
       Running tests/conformance.rs
  test result: ok. 9 passed; 0 failed; ...
       Running tests/gate_contract.rs
  test result: ok. 8 passed; 0 failed; ...
       Running tests/grammar_closure.rs
  test result: ok. 13 passed; 0 failed; ...
       Running tests/version_gate.rs
  test result: ok. 9 passed; 0 failed; ...
       Running tests/zz_api_probe.rs
  test result: ok. 3 passed; 0 failed; ...
     Doc-tests bindforge_core
  test result: ok. 0 passed; 0 failed; ...
  ```

## Checklist

1. **Test-strategy citations.** #1–#6 cited across `tests/chain_ops.rs`
   (module header at :6-11 plus per-test `proves:` lines), #7 in
   `conformance.rs:387`, #8 across `version_gate.rs`, `grammar_closure.rs`,
   `zz_api_probe.rs`. #9 remains uncited by any test — unchanged from r1
   (nit carry-over below). The three new tests added by the revisions all
   carry `proves:` lines (#3, #5).
2. **Layers.** Unit (`chain_ops.rs`) and integration (`conformance.rs`,
   `grammar_closure.rs`, `zz_api_probe.rs`) as promised; the new merge pins
   are unit-level, matching strategy #5.
3. **Previously rejected blocker/major now has a catching test.** The r1
   correctness blocker (the swapped `exchanged` key made every kept exchange
   refuse) is exactly the case the diff's new
   `merge_keeps_an_exchange_the_second_patch_does_not_touch`
   (`tests/chain_ops.rs:2060`) pins: non-empty `swap` leg asserted, written
   beside the base, and the merged-chain layout asserted equal to the stacked
   two-patch chain. On the pre-fix code that case fails with
   `composition_fault` — it would have caught the bug. The r1 correctness
   major (shipped `Cycle` wording edited for a needle) is closed on the
   testing side: the needle list is gone and the test pins the variant plus
   the A13 one-line/control-free shape (`tests/chain_ops.rs:1272-1305`).
4. **Determinism of the tests.** Scratch directories with unique labels, no
   clock, no network, no ports; the new tests follow the same shape.
5. **Names describe behavior.** The new names
   (`refuses_an_absent_head_as_the_read_refusal_carrying_its_path`,
   `merge_keeps_an_exchange_the_second_patch_does_not_touch`,
   `merge_leaves_the_provenance_labels_absent_from_its_output`) read as
   behavior, not implementation steps.
6. **Gate green.** Pasted above.
7. **Untestable additions.** The `check_positions` dead branch persists
   (minor carry-over); the revisions' other additions are pinned or documented
   as defensive (`no_head` is now commented "a defensive corner, not a
   documented one" — my r1 nit 4, resolved).

## My r1 findings — disposition

- **#3 (minor, merge provenance) — RESOLVED.** The revisions took the second
  of my two offered fixes: `inherit_provenance` is deleted, `merge` no longer
  carries `seeded-from`/`generated-by`, the doc comment and `serialize`
  docs state why (ADR Decision Four: a label names what produced the document
  it is written on), and `merge_leaves_the_provenance_labels_absent_from_its_
  output` (:2135) pins the absent labels even when both operands agree on
  them. Shipped behavior now matches the proposal's merge paragraph (id,
  derives, for — nothing else).
- **#4 (nit, `no_head`) — RESOLVED.** Commented as a defensive corner at
  `chain_ops.rs:639-641`.
- **#1 (minor, `check_positions` subsumed; three miscited tests) — partially
  addressed, still open.** The function gained a comment saying the per-
  document shape rules refuse the other contradictions (`chain_ops.rs:467-470`),
  but the tests' `proves:` comments at `:1164-1168`, `:1199-1201`, and
  `:1223-1227` still cite step 4 as what they pin; they still drive
  parse-invalid documents (base-with-`clear`, patch with empty `derives`,
  base with non-empty `derives` — the same shapes pinned at parse by
  `:620`, `:669`, `:606`) and pass on the `name_the_link` prefix. Re-filed
  below as the same minor.
- **#2 (minor, unpinned load-level refusals) — open, unchanged.** The
  revisions neither added tests for `check_link`'s non-profile arm
  (`chain_ops.rs:397-403`) nor for `link_below`'s no-kind fault
  (`:459-465`, reachable with a v1 profile head in a `v1/` directory: it
  passes `check_dir_version`, then faults). Carry-over below.
- **#5 (nit, strategy #9 uncited) — open, unchanged.** No note in the
  `chain_ops.rs` header or the design-014 block in `conformance.rs` states
  that #9's legs are the merge PR's CI job and a Rollout-7 contributor check,
  unlike design 013's note at `conformance.rs:39-42`. Carry-over below.

## What the revisions did to approved pins (no breakage)

- The `Cycle` test lost its needle list but the old list was unsatisfiable
  by a payload-free variant (`assert_refusal_names` uses `any()`, so it only
  ever pinned the substring "`Cycle`" that the message edit had added); the
  replacement pins the variant plus shape. Net pin strength: unchanged or
  better. The reverted message at `error.rs:149` matches design 008's bytes.
- The directory-major test moved from `UnsupportedVersion` to `InvalidValue`
  in lockstep with the code's reuse of `check_dir_version` and the current
  proposal's step 1 (`check_dir_version pins the head's declared major`) —
  test, spec, and proposal moved together, so the approved pin is not
  orphaned.
- `ChainLinkMissing` still pinned for a constructed link (`:1346`); the new
  absent-head test pins `Io` + `source_path` + path-absent-from-message, which
  matches proposal step 1 (`parse_file(head)`) and the revised spec rule 3 —
  strategy #3's wording ("a missing ancestor refuses as `ChainLinkMissing`")
  speaks of ancestors, so no strategy pin moved.
- The merge-conflict pin (`merge_refuses_a_swap_under_a_clear…`, :2207) stays
  green under the new `!reciprocal` branch, so the reciprocal-pair fix did not
  buy the new keep-the-exchange test at the cost of the conflict rule.
- The unconditional `kind`/`for` writer affects only `profile/v2`; the v1
  byte-for-byte pin (strategy #6/A15) and the fixture round-trip (#7) stay
  green.

## Findings

1. **minor (carry-over from r1, unchanged) — `chain_ops.rs:467-497` +
   `tests/chain_ops.rs:1164-1168, 1199-1201, 1223-1227`.** `check_positions`
   remains unreachable: the walk descends only through `kind = patch` and
   terminates on `kind = base`, so no chain that reaches it can fail it; the
   revision's new comment documents subsumption but does not make the branch
   reachable or fix the three tests' `proves:` comments, which still cite the
   step-4 position rule the tests do not actually drive (they pin "a chain
   link whose document fails its parse-level shape rules refuses at load as
   `InvalidValue` naming the link"). Fix: recite the three comments to what
   they pin (and mark `check_positions` as the subsumed belt-and-braces it
   is), as in r1.
2. **minor (carry-over from r1, unchanged) — `chain_ops.rs:397-403,
   459-465`.** The non-profile-document arm of `check_link` and the no-kind
   fault in `link_below` remain untested hostile-chain shapes under the
   one-gate-per-opened-source posture. Fix as in r1: two `chain_ops.rs` cases
   (manifest document in a chain slot → `InvalidValue` naming the id; v1
   profile head in `v1/` → the named kind fault).
3. **nit (carry-over from r1, unchanged) — `tests/chain_ops.rs:55-63`.**
   Test strategy #9 is cited nowhere; add one header line noting #9's legs
   are the merge PR's CI job and a Rollout-7 contributor check, mirroring
   design 013's note at `conformance.rs:39-42`.

## FYI (out of seat, no severity)

The directory-major refusal variant moved from `UnsupportedVersion` to
`InvalidValue` for chain heads — a refusal-shape change on the compatibility
surface; the compatibility seat's matrix covers whether the spec/test move is
the right way to carry it.
