# bindforge
BindForge is an open framework for creating, sharing, and translating game input bindings.

## Contributor prerequisites

The product is Rust (ADR-008); the toolchain is pinned per checkout by
`rust-toolchain.toml`, and rustup installs it automatically:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Debian-family Linux, the desktop-era Tauri system dependencies (the cargo
gate itself needs none of them):

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

One product dependency now reaches the cargo gate itself (ADR-027, Rollout 3):
`apps/registry`'s store pull rides `rustls` with the `ring` provider, and
`ring` compiles C and assembly for the build host's target through a build
script — so a C toolchain (`build-essential` above, or clang) is a
prerequisite of `cargo build`/`cargo test` for the workspace. The CI Windows
compile leg keeps `apps/registry` off the target check for exactly this
reason; see [`.github/workflows/check.yml`](.github/workflows/check.yml).

The gates: the cargo trio (`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`) and `npm ci && npm run check` in
`tools/orchestration` (Node 22.23.4). Both run in CI
([`.github/workflows/check.yml`](.github/workflows/check.yml)).

## Tools

- [`tools/orchestration`](tools/orchestration) — CLI that drives a running
  OpenCode server through the `@opencode/client` SDK, used as an
  orchestration/connectivity smoke test. See its
  [README](tools/orchestration/README.md).
