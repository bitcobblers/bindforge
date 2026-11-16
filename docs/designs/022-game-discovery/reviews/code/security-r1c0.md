# Security Review: Game Discovery (r1c0)

## Overview
This review covers the core `crates/discovery` library and the `apps/discovery-cli` application introduced in this change.

## Findings

### [minor] Terminal Injection via Plugin Metadata
- **Location**: `apps/discovery-cli/src/main.rs:58`
- **Problem**: The CLI prints `game.display_name` directly to the terminal when displaying the discovery table. Similarly, `crates/discovery/src/engine.rs:25` prints `plugin.name()` to stderr. If a plugin is untrusted or provides malicious data, it can inject ANSI escape sequences into the user's terminal, potentially leading to unauthorized terminal manipulation or masking of output.
- **Fix**: Sanitize all strings (e.g., `display_name`, `plugin.name()`) to remove or escape control characters/ANSI escape sequences before printing.

### [minor] Denial of Service (DoS) via Hanging Plugins
- **Location**: `crates/discovery/src/engine.rs:17`
- **Problem**: The `DiscoveryEngine::discover_all` method iterates through plugins and calls `plugin.discover()` synchronously. There is no timeout mechanism in place. A single plugin that hangs or takes an excessive amount of time will cause the entire `discovery-cli` process to hang indefinitely.
- **Fix**: Implement a timeout mechanism for the `discover()` call, for example by running the discovery process in a separate thread and using a timeout to join it.

## Checklist
- [x] External input validated at the boundary: N/A (Plugins not in diff)
- [x] Path handling: Pass
- [x] Injection: Found (Terminal injection)
- [x] Secrets and PII: Pass
- [x] New dependencies: Pass
- [x] Filesystem writes: N/A
- [x] Failures on malformed/hostile input: Pass
