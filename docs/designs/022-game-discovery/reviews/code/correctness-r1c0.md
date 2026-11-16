# Code Review Report: Correctness (r1c0)

## Overview
Review of the implementation of the `discovery` core library and the `discovery-cli` bootstrap application against the design in `02-proposal.md`. This pass covers the implementation of Phase 1 (Core Library) and Phase 3 (Bootstrap CLI).

## Findings

### [Major] Tests are not being executed
**Location**: Gate record
**Problem**: The gate record shows that `cargo test --workspace --locked` resulted in `0 passed` tests. This is a significant issue as the diff introduces several new integration tests in `crates/discovery/tests/` and `apps/discovery-cli/tests/`. As a result, the correctness of the implementation has not been verified by the automated test suite.

### [Minor] Platform tag formatting in CLI output
**Location**: `apps/discovery-cli/src/main.rs`
**Problem**: The CLI output for the `Platform` column in the `discovery list` command uses the `Debug` representation of the `Platform` enum (e.g., `SteamLinux`), whereas the design specification's example output and documentation specify kebab-case platform tags (e.g., `steam-linux`).
**Fix**: Implement `std::fmt::Display` for the `Platform` enum to return the kebab-case string, or use a mapping function in the CLI output to ensure compliance with the design.

## Checklist
- [x] Requirements (Phase 1 & 3) are implemented.
- [x] Data model (`DiscoveredGame`) matches the design.
- [x] Slug normalization logic matches the design specification.
- [ ] Tests are passing (FAILED: Gate record shows 0 tests passed).
- [x] Error handling is implemented in the engine and plugin trait.
