# correctness review r1

verdict: request-changes

Scope: whole document (c0, round 1). Inputs: `02-proposal.md` (incl. Revision log — empty,
"No findings yet — r0 initial draft"), `docs/adrs/008-rust-for-the-product.md`. All findings
are labelled `new`.

Source spot-checks (anti-hallucination gate): clap 4.6.7 / MSRV 1.74 / WG-CLI (docs.rs) ✓;
axum MSRV 1.80 + "0.9 in dev, 0.8.x is what's released" + consumes tower (README) ✓ except the
exact patch number (see F8); sqlx repo moved `launchbadge/sqlx` → `transact-rs/sqlx` and README
installs `version = "0.9"` ✓ (A3's cited URL redirects to the canonical repo, so the row holds);
`query!` + `DATABASE_URL` same-schema requirement ✓ (A4); `prepare --workspace` root `.sqlx/`,
commit → no DB, `--check` for CI, `SQLX_OFFLINE=true` forces offline ✓ (A5);
`[profile.dev.package.sqlx-macros] opt-level = 3` ✓ (A6); workspaces incl. flat-members
recommendation and `workspace.package`/`dependencies`/`lints` ✓ (A7); cargo-test unit+integration+
doc tests, `--locked` determinism, "working directory when running each unit and integration test
is set to the root directory of the package" ✓ (A8); cargo issue #7760 open, user crate named
`core` shadows the implicit one ✓ (A12); Tauri prerequisites: libwebkit2gtk-4.1-dev,
WebView2 preinstalled from 1803, MSI requires the deprecated VBSCRIPT feature, macOS 10.15+,
Node only for a JS framework, no numeric Rust MSRV published ✓ (A14–A16, A24); WebView2
distribution page: Evergreen auto-updates, preview-channel testing, "some devices might not
have the Runtime pre-installed" ✓ (A17); TOML v1.1.0 + "over 40 programming languages" ✓ (A20);
toml_edit `0.25.19+spec-1.1.0`, preserves comments/spaces/relative order, does not preserve
dotted-key order or trailing newline ✓ (A21). Repo state checked for A23: no `crates/`,
`Cargo.*`, `schemas/`, `rust-toolchain.toml`, or `.github/`; `tools/orchestration/package.json:23`
is the `check` script ✓ — but see F1 for the license claim A23 is asked to carry.

## Findings

- [major][new] Chosen design → Workspace shape — `license = "MIT"          # matches LICENSE + tools/orchestration (A23)` — the claim is false: the repo's `LICENSE` file is the Apache License 2.0 text, while `tools/orchestration/package.json:29` declares `"license": "MIT"`; the repo is already split, and A23's recorded basis ("repo listing 2026-11-11; `tools/orchestration/package.json:23`") covers only the absence of the product tree and the npm `check` script, so the `verified` tag does not support this row's license half. Rollout step 1 copies the block verbatim, so the design commits every crate to MIT against a repo whose license file is Apache-2.0. Fix: set `license = "Apache-2.0"` to match `LICENSE` (or record an explicit relicensing decision), and replace the comment with a citation of the two license files (or a new register row) instead of A23; note the `package.json` MIT vs `LICENSE` Apache-2.0 mismatch as a repo fix rather than papering over it with a comment.

- [major][new] Chosen design → "Adversarial guard", against Non-goals and Deferred / out of scope — Non-goals: "The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009" and Deferred: "chain/patch/policy/manifest semantics and their fixture cases — owned by ADR-009", while this design fixes `ChainTooDeep` "(chain longer than `MAX_CHAIN_DEPTH = 32`)", puts the constant in `schemas/README.md` "so third-party readers can conform", and Test strategy #3 puts chain-boundary fixtures under `schemas/profile/v1/fixtures/invalid/` in the scaffold. An implementer cannot tell which document owns the depth bound, its two error variants, and their fixtures; the guard is the one piece of chain semantics this design needs to be a public, cross-language contract, and as written it is simultaneously decided here and deferred there. Fix: one sentence in the guard paragraph — the bound and the `Cycle`/`ChainTooDeep` variants are decided by this design and are a constraint ADR-009 must adopt — and narrow the Deferred clause to "chain semantics other than the depth guard and its boundary fixtures".

- [minor][new] Chosen design → Workspace shape (root `Cargo.toml` block) — `members = ["crates/*", "apps/*"]` with no `resolver` — the block is a virtual manifest, and the very page A7 cites says a virtual workspace must set `resolver` explicitly because it has no `package.edition` to infer it from; as committed, cargo warns on every command and falls back to the edition-independent resolver, silently splitting feature unification across `core`/`transforms` — exactly what "the floor and lint policy are declared exactly once" is meant to prevent. Fix: add `resolver = "3"` to `[workspace]`.

- [minor][new] Chosen design → Workspace shape (`[workspace.dependencies]`) — `serde = { features = ["derive"] }   # exact version pinned at merge` — a workspace dependency entry needs a source, and the cited Cargo page states that members inheriting it "cannot use any other dependency key (such as `version`)", so no member can supply the pin later; the block as written is not a resolvable manifest. Fix: give the floor now (`serde = { version = "1", features = ["derive"] }`), or drop the entry until the first crate needs serde and say so in the comment.

- [minor][new] Chosen design → Workspace shape (members note + table) — "(the `crates/*` and `apps/*` globs follow A7's flat-members recommendation)" vs the table row `apps/desktop` → "`src-tauri` package `bindforge-desktop`" — `apps/*` matches one path segment, so the desktop crate the table places at `apps/desktop/src-tauri` is never picked up by the globs the design commits. The TypeScript boundary defers "the desktop-era `src-tauri` package's exact `members` path", but it does not say the glob cannot reach it, so the committed workspace silently drops a member when the desktop design lands. Fix: one clause in the members note — the desktop era adds an explicit `members` entry for the `src-tauri` path (the glob alone will not match it).

- [minor][new] Formats contract → harness paragraph vs the `schemas/` tree — the harness "walks every `schemas/*/v*/fixtures/{valid,invalid}` directory, and asserts: every valid fixture parses and re-serializes to a semantically equal value", but the tree gives `reach-classes/v1/` only "tables as TOML data + fixtures/" (no `valid`/`invalid` split), and Test strategy #4 tests reach tables through a separate loader contract ("per-hand (left/right) tables kept distinct"). The glob either sweeps reach-class data into a round-trip assertion that is not defined for it, or matches nothing and the directory goes unexercised. Fix: give `reach-classes/v1/` the same `fixtures/valid` + `fixtures/invalid` layout and state explicitly that table data is asserted by load + distinctness (Test #4), not by re-serialization.

- [minor][new] Formats contract → versioning rule — "a file's presence in `vN/fixtures/valid/` is the definition of 'supported'" (a repo-side fixture fact) sits beside a runtime rule, rejection "with a structured error naming the supported majors" (a compiled-in list in `bindforge-core`), and nothing in Test strategy keeps the two equal; the rule also never says what happens to a document that declares no version at all. Fix: add a test asserting the majors `bindforge-core` declares equal the majors present under `schemas/*/v*`, and state that an absent version declaration is a structured rejection with a fixture under `schemas/profile/v1/fixtures/invalid/`.

- [nit][new] Assumption register A2 — "axum 0.8.9 ships, MSRV 1.80, breaking 0.9 in dev" marked verified against the axum README, which supports the MSRV, the 0.9-in-dev warning, and `tower`, but only says the `0.8.x` branch is what is released to crates.io, not the patch number. Fix: cite crates.io/docs.rs for `0.8.9`, or write "0.8.x".

- [nit][new] Assumption register A16 — the quoted material "some devices might not" is not on the cited Tauri prerequisites page (it says WebView2 "is already installed on Windows 10 (from version 1803 onward)"); the sentence is on the WebView2 distribution page cited by A17. Fix: move that quote's citation to the A17 URL.

- [nit][new] Gates → canonical sqlx rule — "every gate job builds with `SQLX_OFFLINE=true` and no `DATABASE_URL`; a dedicated hub-path CI job with a Postgres service runs `cargo sqlx prepare --check --workspace`" — the sqlx-cli README says all `sqlx` commands require a database URL, so the hub-path job cannot be one of the "gate jobs"; as written the two sentences overlap. Fix: say "the two PR gate jobs build with `SQLX_OFFLINE=true`", and note the hub-path job runs with `DATABASE_URL` set and `SQLX_OFFLINE` unset.

- [nit][new] Compatibility matrix — "macOS | unknown | BindForge commits nothing" is ambiguous (reads as "contributes nothing" as easily as "commits to nothing"), which obscures the one thing the row is deciding. Fix: reword as "no macOS support committed for v1".

## FYI (no severity)

- The cargo gate runs only on the Linux CI runner while the design pins Windows contributors to
  the toolchain and names Windows-only failure modes (MSI/VBSCRIPT, WebView2); the first Windows
  run of fmt/clippy/test is a one-off manual check in Test strategy #8. Platform/rollout seat.
- ADR-008 clause 4 names `cargo clippy` + `cargo fmt` as the check gate; the design's gate adds
  `cargo test --workspace --locked`. Consistent with the "one cargo gate" goal, but it widens the
  ADR clause — worth a sentence for the consistency/editor seat.
- `schemas/README.md` is described as the single home for versioning rules and then also hosts
  `MAX_CHAIN_DEPTH`; a second responsibility for the same file. Complexity seat.
