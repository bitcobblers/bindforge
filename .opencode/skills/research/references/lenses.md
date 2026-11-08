# Research lenses

Pick at most three that fit the topic. Apply each under the named section of
the artifact.

## Prior art
- How have comparable projects solved this exact problem?
- What names/formats did they choose, and did any win by convention?
- What did they later regret (issue trackers, changelogs)?

## Security landscape
- Known CVEs or attack classes in this problem space.
- How do comparable tools handle untrusted files from the network?

## Performance baselines
- Published latency/throughput numbers for comparable input-path work.
- Budget the design should not exceed (e.g. per-event latency at 1000 Hz
  polling).

## Platform notes
- X11 vs Wayland vs Windows (Raw Input / DirectInput) vs macOS (HID /
  IOKit) differences for input handling.
- Node version floors implied by the APIs in play.
- Packaging constraints (flatpak/sandbox device access, node-gyp).

## Standards
- Relevant standards or registries: evdev codes, HID usage tables, SDL
  gamepad mappings, USB-IF usage pages.
- Which format is the common denominator for cross-tool sharing?
