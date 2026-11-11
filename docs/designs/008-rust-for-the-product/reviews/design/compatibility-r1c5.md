# compatibility review r1c5

verdict: approve

Ratification scope: `git diff 796b2a63..67a6a1a3` on `02-proposal.md` (A27 added;
Gates reintroduces advisory scanning as the scheduled `advisories.yml` job +
root `deny.toml`; TypeScript boundary drops the carried `tauri` patch number;
Rollout 4 extended; the advisory line removed from Deferred) plus the full
current document against `docs/adrs/008-rust-for-the-product.md`.

Ratified unchanged: every compatibility-matrix row still carries a definite
status (no `unknown`, no empty cell) and the diff does not touch the matrix;
Context still names no input-event API (checklist 2); the Node floor handling
(explicit `setup-node` version) is untouched (checklist 3); the Postgres row
still points at ADR-014's strategy (checklist 7's platform rows still covered by
Test #5/#6/#7). The three closures from r1 hold: the per-PR cargo gate is still
exactly the three toolchain commands, Rollout 7 still schedules the Windows leg
of Test #7, and the Postgres row still cites ADR-014. Reintroducing
`cargo deny` as a weekly non-PR job does not contradict ADR-008: clause 4 fixes
only `cargo clippy` + `cargo fmt` as the `check` gate and Consequences only
require "CI must run both" toolchains, so a scheduled third workflow neither
removes a required gate nor adds a product-language claim.

## Findings

- [minor][new] Chosen design → Gates / Test strategy #5 — the revised Gates
  paragraph now says "installing `cargo-deny` at a pinned version and running
  `cargo deny check advisories` against the committed `Cargo.lock` and a root
  `deny.toml`", while Test strategy #5 still promises "from a fresh clone with
  no `DATABASE_URL` set and no Postgres reachable, every command listed in the
  Gates block succeeds on the Linux CI runner, with nothing installed beyond the
  pinned toolchain". In the version I approved, the only commands named anywhere
  in the Gates section were the three fenced `cargo` commands, so #5's referent
  was unambiguous; the advisory tool lived in Deferred, where naming a command
  cost nothing. The revision moves a fourth command *into* the Gates section —
  one whose own rationale in the same paragraph says it "needs a tool outside
  the pinned toolchain" — so #5, the acceptance criterion for the one-gate
  promise, now has a referent that reads either as three commands (true) or as
  all commands named in the section (false by construction, since
  `cargo deny check advisories` cannot run with only the pinned toolchain).
  The revision log records the Gates change (r1, r1c3) but not this collision
  with #5's wording, and #5 was deliberately reworded in r1 ("cites the Gates
  commands instead of restating them, so it cannot drift from the gate") — the
  anti-drift mechanism it relies on is exactly what this insertion defeats.
  Fix: scope the citation in #5 — "every command in the Gates fenced block (the
  three `cargo fmt`/`clippy`/`test` commands)" — instead of "every command
  listed in the Gates block".
- [nit][re-raised] Chosen design → Workspace shape / Gates / Test strategy #5 —
  "`rust-toolchain.toml` with `[toolchain] channel = "1.85.0"`" … "`rust` (the
  three gate commands above, and nothing installed beyond the pinned toolchain
  (A11))" … "with nothing installed beyond the pinned toolchain". Unchanged by
  this revision, and still load-bearing in two places now rather than one. Two
  of the three gate commands need rustup components the committed pin does not
  name — `cargo fmt` needs `rustfmt`, `cargo clippy` needs `clippy` — and they
  are present only because rustup's default install profile happens to include
  them; a `--profile minimal` contributor install fails the gate for a setup
  reason, not a code reason, and A11's basis covers the channel pin only. Fix:
  extend the Workspace shape contents to `[toolchain] channel = "1.85.0",
  components = ["clippy", "rustfmt"]`, replacing the reliance on rustup's
  default install profile.
- [nit][new] Chosen design → Gates — "installing `cargo-deny` at a pinned
  version" pins the tool's version but never names how it is installed or which
  toolchain it builds with, and the checkout's `rust-toolchain.toml` applies to
  that job too, so an install-by-source path compiles it under the pinned 1.85.
  The document is disciplined about exactly this elsewhere — A25 registers "No
  direct or transitive dependency at merge needs rustc newer than 1.85" for the
  product lockfile — but the new third workflow adds a tool to the CI story with
  no equivalent entry, and the r1 exclusion rationale ("unpinned tool outside
  the rustup toolchain") is the reason it was never characterized. It runs on
  the Linux CI runner the matrix already claims, so no matrix row is affected;
  this is the install-path gap only. Fix: name the install path in the same
  sentence (a pinned prebuilt release binary, or `cargo install
  cargo-deny@<ver> --locked` with a register entry asserting that version builds
  on 1.85), so the pin the paragraph asserts is actually reproducible.

## FYI (no severity)

- Gates asserts "That same run is the trigger for the `rust-toolchain.toml`
  security-advisory bump (Workspace shape)", but the run's command is
  `cargo deny check advisories` against `Cargo.lock`, which contains no
  toolchain entry — the mechanism that surfaces a *published toolchain*
  advisory is unnamed. Correctness seat's territory.
- TypeScript boundary keeps a floor of "never below 2.11.1" while A27 says
  GHSA-w28w-mhc8-qvjv is patched only at 2.11.6 and is invisible to the
  crates.io/RustSec data the scheduled scan reads, so the covering control is
  the manual "checks Tauri's own repository advisory list" assigned to the
  desktop design, which Deferred records as having no number yet. Not a defect
  today — `tauri` is not in the lockfile until the desktop era, and A13's
  "2.12 line" already clears 2.11.6 — but security should decide whether the
  obligation needs an owner or a date.
- Rollout 4 lands `advisories.yml` and `deny.toml` with no verification step;
  Test strategy has no case that exercises the scheduled job or a `deny.toml`
  `skip` entry. Test-strategy seat's call.
