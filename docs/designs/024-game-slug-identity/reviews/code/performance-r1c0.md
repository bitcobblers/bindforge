# Code review — performance seat, round 1 pass c0

Change under review: `git diff e786de8f6820d71dbe919300e247f72bd59ef7f3...HEAD -- ':(exclude)docs/designs'`
Design: `docs/designs/024-game-slug-identity/02-proposal.md`; ADR: `docs/adrs/024-game-slug-identity.md`.
Inputs: the diff, the proposal, the ADR, the runner's gate record (green at 2026-11-18T19:08:59Z), and my own re-runs of the read-only gates. No other review artifacts opened.

## Checklist walk

### 1. Event/keystroke path — not applicable, no issue found

No event or keystroke path exists in this change: the touched runtime surface is the one-shot `discovery-cli list` flow and the Steam discovery scan. Within that flow nothing added performs filesystem or network work: the new derivation (`crates/discovery/src/slug.rs::derive_slug`) is pure string processing run once per already-read `appmanifest_*.acf`; the CLI emit-boundary check (`apps/discovery-cli/src/main.rs:88-102`) is an in-memory pass over rows already collected; `resolve_collisions` (`crates/discovery/src/steam_common.rs`) touches no I/O. The old `normalize_slug` allocated three intermediate `String`s per name (lowercase + two rebuild passes + format); `derive_slug` allocates two with reserved capacities plus one final `to_string`/`format!` — allocation churn per name is equal or lower, and it runs on the per-file scan that is already dominated by reading and parsing each file.

### 2. Complexity — checked; worst case documented and risk-accepted, no finding

The only superlinear structure is the collision pass's suffix probe: for the k-th distinct display name deriving an already-claimed base, the loop at `steam_common.rs` (`resolve_collisions`, `for suffix in 2_u32..`) probes `claimed` linearly through claimed candidates, so a pathological corpus of k distinct names all kebabbing to one base costs O(k²) set lookups plus one `format!` per probe. Membership itself is a `BTreeSet`/`BTreeMap` (the checklist's "maps/sets where identity lookup was intended" — satisfied). This is not unbounded-`n` O(n²): `k` is bounded by the manifest-file count one plugin's scan collected, on the same pass that already pays an fs read + parse per file. The design states the risk and its bound explicitly ("Collision-suffix probe loop repeats bound checks many times … probe count is bounded by distinct-name count, no measurable cost today"; 01-research "no baseline exists; none claimed"). The implementation matches the accepted shape: the probe loop breaks at the first over-bound candidate ("candidates only grow with the counter"), so a base sitting at the bound costs exactly one probe. The per-segment bound re-check inside `is_well_formed_segment` after `ActionId::parse` already called `segment_within_bound` (`crates/core/src/action.rs`) is one redundant `len()` comparison, deliberately documented as keeping the predicate complete on its own. All fine.

### 3. Unbounded growth — no issue

No cache, registry, or long-lived array is introduced. `bases`/`assigned`/`claimed` in `resolve_collisions` are built, consumed, and dropped inside one `discover()` call; the CLI's `refused: Vec<String>` is bounded by the refused-row count of one run. No `static`/global mutable product state added (the only `static` is the test helper's `AtomicU64` nonce counter in `crates/discovery/tests/common/mod.rs`).

### 4. Startup — no issue

Boot-time work added to the CLI is the O(rows) grammar/bound scan before rendering, on rows already in memory — no new file reads or probes. `bindforge-core` joining `crates/discovery` is compile-time only. `escape_and_truncate` (renamed `one_line`) still stops scanning at `MESSAGE_VALUE_CHARS` and breaks — bounded per echo, called only on refusal paths.

### 5. Cache invalidation — not applicable

The diff ships no caches, so there is no invalidation logic to cover.

### 6. Benchmark-like tests / timing notes — none shipped; I timed the suite myself

The diff contains no benchmark or timing test, so there is no cheap check to run for a behavior claim; the design claims no baseline. As independent evidence for the "no measurable cost" acceptance in item 2, I re-ran the gate myself: `cargo test --workspace --locked` is green here, with `collision_tests` "test result: ok. 7 passed; … finished in 0.00s", `slug_tests` "10 passed … 0.00s", `steam_linux_tests` "23 passed … 0.00s", `steam_windows_tests` "26 passed … 0.00s", `slug_bound_corpus` "5 passed … 0.00s", `cli_slug_refusal_tests` "4 passed … 0.00s". The runner's captured gate (all four legs green, exit 0) matches.

## Requirements trace

- 024-B1 / "one exported constant, referenced rather than retyped": verified — `MAX_SLUG_OCTETS = 128` at one home (`placement.rs`), private copies deleted in `policy.rs`, `manifest.rs`, `document.rs`, `action.rs`; refusal strings interpolate the constant (`segment_bound_rule()`, `check_slug`, `check_profile_slug`). No per-call cost concern; the `format!`s sit on refusal paths only.
- 024-L2 collision rule ships in the documented single-pass-per-plugin shape with set-based claim checks; the acceptance above covers its probe cost.

## Findings

None at blocker, major, minor, or nit severity from the performance seat.

## FYI (out of specialty, no severity)

- `apps/discovery-cli/src/main.rs:88-102`: the refusal pass clones each refused slug into a `Vec<String>` before printing; printing inside the `retain` closure would drop one clone per refused row. Refused rows only — negligible.
- `crates/discovery/Cargo.toml`: dev-dependency `serde_json = "1.0"` is declared with its own version string rather than `{ workspace = true }`, unlike the workspace's other consumers; dependency-hygiene, not performance.

## Verdict

approve — zero open findings.
