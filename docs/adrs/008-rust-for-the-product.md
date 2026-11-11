---
status: accepted
date: 2026-11-11
---

# ADR 008: Build the product in Rust; confine TypeScript to the pipeline

We will write every end-user-facing BindForge component — CLI, desktop app,
and later the hub service — in Rust, in one cargo workspace under `crates/`,
with the file formats under `schemas/` as the public contract. TypeScript
stays exactly where it is: `tools/orchestration`, internal pipeline tooling.

## Context

The repo's only code today is TypeScript, and it fits its job: the
orchestration harness is internal, single-operator, and rides the OpenCode
SDK. The product's audience is different. A desktop keybinding tool lives on
gamers' machines — native packaging, no Node runtime to install and version,
filesystem-level work (game scanning, INI codecs, Proton prefixes) where a
compiled core is the natural home — and its hub is a long-lived public
service. A second constraint is the profile/manifest format itself: it is the
sharing substrate, so it must outlive any implementation and be workable by
community tooling in languages we will not choose.

The hub's stack had a real alternative: ASP.NET + Postgres, matching a
maintainer's strongest language. It was dropped to keep the repo at two
languages total — Rust for the product, TypeScript for the pipeline — which
also lets the model crate compile directly into the hub server.

## Decision

1. **One product language.** Everything user-facing is Rust.
   `tools/orchestration` stays TypeScript and nothing user-facing is added
   in TS.
2. **Workspace layout.** A cargo workspace:

   ```
   crates/core         # model: taxonomy, reach tables, chain resolution,
                       # manifest types — the reference implementation of schemas/
   crates/transforms   # policy stacks
   crates/adapters     # codec families, config path discovery, write policy
   crates/scanner      # installed-game detection, config fingerprinting
   apps/cli            # bindforge CLI (clap)
   apps/desktop        # Tauri v2 app (lands with the hub era)
   apps/hub            # axum + Postgres service (lands with the hub era)
   ```

   Framework choices fixed here: clap (CLI), Tauri v2 (desktop), axum +
   Postgres via sqlx (hub). The webview UI framework is deferred to the
   desktop design.
3. **Formats are the contract.** `schemas/` at the repo root holds the
   language-neutral, versioned definitions of the profile, patch, manifest,
   and policy formats, plus golden-fixture conformance tests. `crates/core`
   and any third-party reader validate against these, not against Rust
   struct churn. Postgres is used only by the hub — files remain the product.
4. **Repo practices extend.** `repo-practices` gains a Rust section:
   workspace package shape, `cargo clippy` + `cargo fmt` as the `check`
   gate (mirroring `npm run check`'s role for the pipeline), tests under
   each crate.

## Alternatives considered

**TypeScript for the product.** Fastest start given the existing tooling,
but it ships a Node runtime to end users, and game-filesystem work (path
discovery across Windows and Proton prefixes, encoding-preserving codecs)
lands in the weakest position. It also buys nothing: the pipeline already
proves the workflow, and the product's model layer is small enough to redo
in the right language.

**ASP.NET + Postgres hub.** Kept the hub in the maintainer's specialty, but
adds a third language, and the hub consumes the same formats and validation
logic the client already implements — a Rust hub compiles them instead of
re-specifying them. The polyglot conformance tax (Rust ↔ C# readers agreeing
forever) is exactly the cost this removes.

**Rust frontend without Tauri (egui/iced).** Single language end to end, but
binding *presentation* — comparison sheets, profile browsers, diff views —
is web UI territory; webview reuse and design velocity favor Tauri.

## Consequences

Becomes possible: one `cargo check` gates the whole product; the hub
validates uploaded profiles with the same compiled types the client uses;
format evolution is driven by `schemas/` versioning rather than a library
release.

Accepted costs: the Rust toolchain is now a contributor prerequisite. The
maintainer's strongest language (C#) has no home in the repo — accepted
deliberately. Two toolchains (npm + cargo) remain in the repo with separate
gate commands; CI must run both. Rust learning curve lands where it always
does — lifetimes in the codec layer, Tauri IPC plumbing — with no escape
hatch.
