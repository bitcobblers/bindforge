# Code review — performance seat — round 1, pass c0

Scope: `git diff 1af12026893971772c5ac8384ceae348d63cd064...HEAD -- ':(exclude)docs/designs'`
(53 files: core `document.rs`/`policy.rs`/`lib.rs` arms, `crates/transforms` crate,
`schemas/policy/v1/` corpus, and their tests).
Judged against `02-proposal.md` and ADR-015. Checklist: performance.

## Verdict

**approve** — zero blocker, zero major. No open performance findings on this diff.

## Checklist walk

### 1. Event/keystroke path — pass

There is no event path in this diff and no I/O added to any evaluated path.
`apply` (`crates/transforms/src/apply.rs:181`) is a pure function over already-
loaded values: no fs, no network, no clock, no randomness — which is also what
the determinism rule in the proposal rests on. The only fs touch in the diff is
`load_policy` (`crates/core/src/policy.rs:54-63`), which slug-checks the id
*before* any filesystem question and then reads exactly one `<id>.toml`. No
per-event rebuilds: `Guard::new` (`apply.rs:327`) and `Layout::new`
(`apply.rs:196`) are built once per `apply` call, not per row.

Avoidable allocations: each row's `matched` (`apply.rs:367-373`) clones the
registry's whole id list via `ActionRegistry::ids()` (`crates/core/src/action.rs:157`)
and then filters. That is one full clone per row — ≤ 64 rows × 40 shipped ids,
bounded on both factors, and `ids()` is the only public accessor core ships.
Not worth a change; recorded here, not as a finding.

### 2. Complexity — pass

The two quadratic-shaped candidates were evaluated and both sit under shipped caps:

- **Linear `Layout` lookups** (`input_of`/`action_on`/`rebind`,
  `apply.rs:285-309`). A BTreeMap index would be faster, but the base's `set`
  list is capped at `MAX_PATCH_DELTA_ELEMENTS = 256` — `parse_set` routes
  through `delta_elements`, which enforces it (`document.rs:1121-1136`) — and
  the code comment (`apply.rs:269-270`) documents the choice. n ≤ 256; the
  worst relocate cost is a few thousand comparisons. Acceptable and documented.
- **`moves` built before the `match-count` check** (`apply.rs:397-418`): a
  wide match with few targets pays one pass of matched × linear lookups before
  the count refusal. The design fixes this check order itself ("a matched
  action unbound in the working layout → `unbound-action` naming it" precedes
  "match count ≠ target count", proposal Chosen design → The stack), so the
  code is following the contract, not wasting work. Bounded either way.
- `is_cluster` (`apply.rs:617-643`) is a BFS with BTreeSet membership over at
  most matched keys; `rook_neighbours` allocates a 4-element Vec per visited
  node — trivial under the same bound. `TagPattern::matches`
  (`document.rs`, matches arm) is byte-exact with no allocation per compare.
- Parse side: duplicate-target detection uses `Vec::contains` (`document.rs:1430`)
  — O(16²) worst case by `MAX_RELOCATE_TARGETS`, no map warranted.

No unbounded O(n²) found anywhere in the diff.

### 3. Unbounded growth — pass

The diff adds no cache, no global registry, no long-lived array. The
`Guard.reserved`/`Guard.docked` sets (`apply.rs:322-339`) are per-application,
bounded by the manifest's shipped caps (≤64 groups × ≤16 members).
`MAX_POLICY_TRANSFORMS = 64` and `MAX_RELOCATE_TARGETS = 16`
(`document.rs`, new constants) bound the document-side growth; the proposal's
Budget and cost section states the composite and matches the implementation.

### 4. Startup — pass

`document.rs` gains the `policy` arm only inside `parse`; no new eager read or
table build runs at core init. `format_majors("policy")` is one match arm. The
conformance walk picks up `policy/v1` with the existing harness — the added
fixture corpus costs test time only.

### 5. Cache invalidation — pass (vacuously)

The diff introduces no cache, so there is nothing to invalidate. Determinism is
pinned instead: `applying_the_same_policy_twice_answers_serialize_identical_bytes`
in `crates/transforms/tests/apply.rs` covers the regeneration-equality rule the
proposal promises, which is the property a cache would have to preserve.

### 6. Benchmark-like check — run, numbers quoted

No benchmark test or timing note ships in the diff. The cheap check available
was the new suite's wall time, run with
`cargo test -p bindforge-transforms --locked` and
`cargo test -p bindforge-core --locked --test policy --test conformance --test version_gate --test grammar_closure`:

- `bindforge-transforms`: 27 apply tests + 2 probe tests — all ok, each binary
  `finished in 0.00s`;
- `bindforge-core` policy suite: 26 tests, `finished in 0.00s`; conformance
  walk now carrying `policy/v1`: 9 tests, `finished in 0.01s`; version_gate 11
  tests and grammar_closure 15 tests, `finished in 0.00s`.

That is consistent with the proposal's bounded-cost claim: even the full corpus
walk plus every stack outcome test runs inside a debug-build millisecond.

## Findings

None within specialty. (FYI, no severity: the per-row `ActionRegistry::ids()`
clone in `apply.rs:367` is an avoidable allocation if core ever grows a
borrowing accessor — bounded at 40 × 64 today, no action requested.)
