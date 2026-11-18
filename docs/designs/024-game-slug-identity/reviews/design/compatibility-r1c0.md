# Compatibility review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`), pass c0 of round 1. Inputs used:
the proposal, `docs/adrs/024-game-slug-identity.md`, the Revision log (single
r1 entry, no rejected findings), and read-only inspection of the code/spec/CI
surface named in the proposal (`crates/core/src/{placement,lib,policy,manifest,document,action,error}.rs` surface facts, `crates/discovery/Cargo.toml`, `crates/discovery/src/slug.rs` area, `schemas/` spec statements and `fixtures/invalid/` corpora, `.github/workflows/check.yml` + `advisories.yml`, `tools/orchestration/src/pipeline/types.ts`, `rust-toolchain.toml`). No shell was available to this session (permission denied), so the `git diff*` re-read of a previous own-report was not performed; the caller's prompt states this seat has no previous findings, so nothing depended on it.

Verified-without-finding: A2 (four private 63-constants: policy.rs:37,
manifest.rs:63, document.rs:220, action.rs:38 `MAX_SEGMENT_OCTETS`), A3 (the
"63" text appears in exactly the four named `spec.md` files, and no other
schema spec states a numeric slug bound), A4 (discovery deps: serde + cfg-
windows winreg), A8 (`MAX_NATIVE_NAME_OCTETS = 128` public), A9 (lib.rs
re-exports exactly eleven constants, so "twelfth" is arithmetically right),
A11 (manifests/v1 invalid corpus has `id-uppercase` and `name-past-cap`, no id
past-cap), `SEGMENT_SHAPE_RULE` "1-63 octets" at action.rs:44, toolchain pin
1.85.0.

## Findings

- [major][new] Compatibility matrix — "Third-party schema readers | Unknown | Built against the 63-octet text they refuse 64–128-octet ids until they re-read the amended specs" — a matrix row whose status is `unknown` is a major finding under the checklist (item 1); the row commits to no status. The proposal's own material lets it commit: 024-L3 makes this relaxation-only (every document valid under the 63-octet text stays valid under 128), and A7 establishes no shipped corpus document exceeds 63 today, so every known artifact is accepted by both old-text and new-text readers; the divergence is confined to future ids no known consumer produces. Fix: replace "Unknown" with a definite status — e.g. "Yes, with a known behaviour change" — and keep the note: old-text readers reject only 64–128-octet ids, of which zero ship at merge; that is a committed status, not an unknown population.

- [major][new] Test strategy #11 (and matrix rows Windows/macOS) — "Platform: the whole suite runs unchanged on the Linux and Windows CI matrix" — the CI matrix this names does not exist, and nothing in the run creates it. Every job in the repository's workflows runs `ubuntu-latest` (check.yml jobs `rust` and `orchestration`, advisories.yml); the Windows leg of the gate is a cross-compile `cargo check --target x86_64-pc-windows-msvc`, whose own comment states "cargo check performs no code generation or linking" — no test binary ever runs on Windows. Rollout item 7 pins the gates "per `.github/workflows/check.yml`" and no rollout item adds a Windows runner, so #11 is unexecutable as written and the matrix's Windows "Yes, with named gaps" claim carries no runtime verification. Checklist item 7 fails twice more: the matrix claims macOS (APFS) "Yes" and no Test strategy item names a macOS platform test at all. Fix: one of two single edits — (a) add CI legs on `windows-latest` (and `macos-latest`, if the Yes claims stand) to check.yml as a named rollout item and keep #11's wording; or (b) rewrite #11 to the real gate — "full suite runs on Linux; Windows is compile-verified via the existing `cargo check --target x86_64-pc-windows-msvc` leg; derivation/collision/refusal unit outcomes are platform-independent" — and downgrade the Windows and macOS matrix rows' status to match that verification level ("compile-verified", not "Yes" with runtime suite).

- [minor][new] Compatibility matrix — "`tools/orchestration` (Node) | Not affected | No slug-grammar surface there" — the stated basis is false as written: `tools/orchestration/src/pipeline/types.ts:287` defines `SLUG_RE = /^[a-z0-9][a-z0-9-]*$/`, the run-identity `nnn-slug` grammar, so a slug grammar surface is there. The conclusion survives — that grammar is pipeline-internal (design directory segment / branch name), differs from the product grammar (it admits a leading digit), and rides no shared length bound — but a reviewer or conformance checker grepping "slug" in that package will hit this constant and read the row's basis as refuted. Fix: restate the basis as "the pipeline-internal `nnn-slug` run-identity grammar is a distinct grammar that rides no shared product slug bound; no code there reads the changed surface".

## Checklist dispositions

1. Matrix: no empty cells; one `Unknown` row → major above.
2. OS input APIs (X11/Wayland/Raw Input/HID): not applicable — the design touches no input API; it is identity, derivation, bound and spec text.
3. Node version floor: pass — no Node code changes; the orchestration row's conclusion is right (its stated basis is the minor finding above).
4. Native modules: pass — no new native dependencies; the one new edge (`discovery → bindforge-core`) is a workspace-internal Rust dependency; `winreg` remains `cfg(windows)` (Cargo.toml:12-13).
5. Keyboard reality / non-ASCII names: pass — non-Latin-script names fold to `-` or are refused by named decision consistent with ADR-024's stated consequence; the fold-table boundary is pinned (Test strategy #1) and disclosed (A15, Risks row). Dead keys/IME/QWERTY not applicable.
6. File paths: pass — grammar-before-filesystem discipline preserved (`is_slug_name` + bound before any emit; `.`/`..` and separators excluded by the unchanged character class); no new path concatenation; the deferred Windows reserved-name refusal does not contradict any ADR-024 Decision or requirement clause (see FYI).
7. Platform test per supported platform: fail — see the major finding (Windows test names nonexistent infrastructure; macOS claimed but unnamed).

## FYI (no severity)

- This run is the moment discovered slugs join the population the ADR's Context says "doubles as a file or directory name" (they become byte-for-byte adoptable manifest ids), so the deferred Windows reserved-device-name refusal covers a gap whose exposure this design widens even though the character class is unchanged. ADR-024 decides nothing about reserved names, so per the deferral rule it is not a finding here; flagged for the panel/ADR owner to weigh whether the candidate grammar-amendment ADR deserves a named owner.
- `pub enum Namespace { Unknown }` is a closed serde value set; the first new namespace value (ADR-025's `publisher`) will break any consumer that deserializes `DiscoveredGame`. Fine for v1's contract-as-fixtures posture; worth a sentence when ADR-025 drafts the JSON contract (deferred item already names that gap).
