# security review r4

verdict: approve

## Scope

Ratification pass over `reviews/design/editorial-r4.diff` only, with
`02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`, and the Revision
log as context. My previous security findings: none. Per the pass rules I
report only what the editorial revisions broke or made false.

## Checklist evaluation against the diff

The diff is four language-only hunks. Security substance checked per hunk:

1. **Goals bullet** ("CI running both toolchains' gates, and a Rust section
   in `repo-practices`.") — noun-phrase rewrite of the previous sentence;
   the ADR-008 "CI must run both" obligation and the repo-practices Rust
   section (which carries the capability-file rule per Rollout 5) are
   preserved verbatim elsewhere. No change to trust-boundary or gate text.
2. **Gates paragraph** ("so it commits to the only recurring look at the
   lockfile:") — grammar fix of "keeps the only recurring look at the
   lockfile it commits". The security mechanism it introduces is
   untouched in the hunk: weekly/manual `advisories.yml`, pinned
   `cargo-deny`, `cargo deny check advisories` against the committed
   `Cargo.lock`, root `deny.toml` with skips naming waived advisory ids,
   maintainer triage within the week, job-level `permissions: contents:
   read`, action SHAs pinned, and the toolchain security-advisory bump
   trigger. The rewrite keeps the design *committed* to that obligation —
   it weakens nothing in checklist items 5 or 6.
3. **Risks row** ("Codec conformance fixtures pin `toml_edit`'s known
   non-preservation gaps (A21)...") — passive-to-active voice; same
   obligation, same owner (ADR-012), same fixture mechanism. No weakening
   of the write-path mitigation.
4. **Rollout 5** ("...plugins), and tests-under-each-crate.") — list
   conjunction added; the capability-file rule (own commands only; no
   `fs`/`shell`/`dialog` plugins) is intact.

Unchanged by the diff and therefore out of this pass's scope: untrusted-
input premise and 1 MiB / 16-level parse budget, `Cycle`/`ChainTooDeep`
guards, path-handling deferral (consistent with ADR-008 — no contradiction
with its Decision or requirements), and the tauri pin/floor text.

## Findings

None. No hunk breaks previously approved security text, makes a cited
section false, or introduces a new trust-boundary, parsing, path, secret,
dependency, privilege, or fail-closed exposure.

## FYI (no severity)

- TypeScript boundary (not touched by this diff): the tauri floor "never
  below 2.11.1" sits below the 2.11.6 patch level of A27's GHSA-
  w28w-mhc8-qvjv, but the operative sentence pins "the newest `tauri`
  release available when it lands" and requires checking Tauri's own
  advisory list — the Revision log (r2) records this as a deliberate
  choice, so no finding; noted only so the desktop design's seat keeps
  the 2.11.6+ reality in view when it pins.
