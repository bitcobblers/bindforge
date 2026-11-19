# Retired suites (ADR-025, design 025 Rollout 1 / Test strategy #14)

ADR-025 retires the ADR-024 identity machinery — `derive_slug` with
`crates/discovery/src/slug.rs`, the `-2` collision pass, and the `Namespace`
enum — and names the two suites that pinned them as retired:

- `slug_tests.rs` (the `024-C2` derivation table)
- `collision_tests.rs` (design 024 Test strategy #4's collision pass)

The suites that pinned that machinery stand retired with it. They are kept
here, outside any compiled test target, as the retirement's record; the
retirement gate is `crates/discovery/tests/retirement_tests.rs`, which pins
that `tests/slug_tests.rs` and `tests/collision_tests.rs` are gone from the
live suite tree and that the retired names resolve nowhere in the shipped
`src/` trees. The live re-pins live in `game_tests.rs`, `emission_tests.rs`,
`plugin_tests.rs`, `steam_linux_tests.rs`, and `steam_windows_tests.rs`,
whose assertions name `steam/<appid>` only. Nothing here compiles.
