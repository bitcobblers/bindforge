# complexity review r1c5

verdict: approve

Ratification pass. Scope: the r1 → r1c3 diff (A27 row, Gates paragraph,
TypeScript boundary sentence, Rollout 4, Deferred removal, Revision log).
My r1 verdict carried no findings, so this pass reports only what the new
text broke or newly left unanchored. All r1 closures I checked still hold:
the per-PR cargo gate is still exactly the three commands, Test strategy #5's
"nothing installed beyond the pinned toolchain" is still true because
`cargo-deny` lives in a separate non-PR workflow, and the Deferred section no
longer claims an unowned advisory obligation.

## Findings

- [minor][new] Gates — "That same run is the trigger for the `rust-toolchain.toml`
  security-advisory bump (Workspace shape)" — the revision bolts a second
  obligation onto the new weekly run, and the named mechanism cannot perform
  it: `cargo deny check advisories` matches advisory entries against packages
  in `Cargo.lock`, and rustc/rustup is not a lockfile package, so no run of
  that command can surface a toolchain advisory. Workspace shape already
  carries the policy on its own ("bumped only when a reviewed dependency
  upgrade or a published toolchain security advisory requires it"), so the
  clause adds a trigger path that does not exist and gives false assurance
  that the bump is covered. Fix: delete that clause; leave the Workspace shape
  bump policy as the single home for it. (If the intent was a cadence hook
  rather than a detection mechanism, say "the weekly maintainer triage is also
  when a toolchain advisory is checked", not "that same run".)

- [minor][new] Gates / Rollout 4 — "a second workflow,
  `.github/workflows/advisories.yml` (Rollout 4), runs weekly and on manual
  dispatch" plus "the root `deny.toml`" — three new moving parts (`cargo-deny`
  tool, a workflow, a config file with a `skip` waiver knob) trace to no
  entry in Goals / Non-goals and to no row in Risks, which is unique in this
  design: every other mechanism maps to a Goal or a risk. They also land
  before their subject: at Rollout 4 the committed lockfile holds `serde` and
  its transitive tree only, since `clap`, `toml_edit`, `axum` and `sqlx`
  "enter when their consumer lands" (ADR-012 / hub era) and `tauri` arrives
  with the desktop design. The ownership decision itself is sound and I am
  not reopening it — the r1c3 closure (replacing the unowned deferral) is the
  right call. What is missing is the trace and the timing. Fix: add one Goals
  bullet (or one Risks row) naming the dependency-advisory obligation, and
  move only the `advisories.yml` + `deny.toml` half of Rollout 4 to the
  rollout step of the design that first puts a non-`serde` dependency in
  `Cargo.lock` — the obligation stays owned here, the first merged slice
  stays thin.

- [minor][new] TypeScript boundary — "the desktop design pins the newest
  `tauri` release available when it lands in `[workspace.dependencies]`
  (never below 2.11.1) and checks Tauri's own repository advisory list" — the
  revision hands a recurring obligation to a document that does not exist
  ("no number yet"), and Deferred / out of scope lists the desktop design's
  other handoffs (webview framework, npm toolchain, members path, packaging)
  but not this one. That is the same unowned-obligation shape r1c3 just
  closed one level up, reintroduced at the pin itself. Fix: one Deferred line
  — "Tauri's own advisory-list check at desktop-pin time — owned by the
  desktop design (no number yet)" — or fold the rule into the Rollout 5
  `repo-practices` Rust section, which already carries the capability rules.

- [nit][new] TypeScript boundary — "this design carries no patch number for
  it" sits in the same sentence as "(never below 2.11.1)", and A27 names
  2.11.5/2.11.6 explicitly; with A13 recording the current release as the
  "2.12 line", the floor can never bind, so it is a constraint with no
  consumer. Fix: say "carries no pin for it" instead of "no patch number", or
  drop the parenthetical floor and let the newest-at-landing rule plus A27 do
  the work.

## Value check (checklist #7)

Proportionate. The r1c3 revision adds one non-blocking workflow, one config
file and one pinned tool to a slice that is otherwise a workspace skeleton, a
fixture tree and three gate commands; nothing in it needs a new crate, trait
or abstraction layer, and the r1 deletions (`clap`/`toml_edit` entries, empty
lints tables, sqlx profile block, `crates/transforms` creation) kept the
first merged slice thin. The three minor findings above are about tracing and
timing, not about the mechanism being oversized. A 30–50% smaller design is
not available here without dropping an ADR-008 clause.

## FYI (no severity)

- The advisory obligation covers only `Cargo.lock`. This design is also the
  document that creates the repo's CI, and `tools/orchestration/package-lock.json`
  is committed today with no audit step in `npm run check` (`lint` +
  `format:check` + `test`); the asymmetry is a security-seat call, not a
  complexity one — adding an npm-side scan would only grow the slice.
- A27 is a verified incident record (GHSA id, CVSS, dates) rather than an
  assumption; the same detail arguably belongs in Risks next to the
  dependency-churn row. Editorial/correctness call.
