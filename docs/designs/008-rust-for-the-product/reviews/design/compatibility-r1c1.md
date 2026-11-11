# compatibility review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff 04a4ae1 6e4ea21` of `02-proposal.md`, then the full current document.
Checklist applied: `references/compatibility.md` only.

## Closed from r1 (verified in changed text, not re-raised)

- macOS cells: `| macOS | no (v1) |` and `| Contributor dev env | Linux + Windows yes; macOS no (v1) |`
  now agree with Deferred ("excluded for v1 per Compatibility matrix"). No `unknown` cell remains,
  and no cell is empty — checklist item 1 passes.
- OS input APIs (item 2): Context now states "it never reads or injects input events, so no
  X11/Wayland/Raw Input/HID API is targeted", with the per-component escalation clause. Justification
  is consistent with ADR-008 ("files remain the product"; filesystem-level work only).
- Linux row relabeled to "Linux (glibc distros; artifacts deb + AppImage)"; Steam Deck / SteamOS row
  added with a definite `no (v1)`. The deb-only contradiction is gone.
- Node floor (item 3): pinned in a `setup-node` step, explicit in Rollout 4, recorded in
  `repo-practices` per Rollout 5, as prescribed. No Node API above a floor is introduced.
- Fixture file-name constraint added verbatim to the versioning rule (r1 nit).
- Windows gate greenness: Test #7 now folds "the gate commands" into the one-time Windows
  verification — the option my r1 fix offered, so the mechanism is accepted. What is left is the
  scheduling gap reported below, at reduced severity.

## Findings

- [major][new] Gates (ADR-008 clause 4) — the gate block's new fourth command
  "`cargo deny check advisories`", plus Rollout 1 ("Add root `Cargo.toml`, `rust-toolchain.toml` …
  and `deny.toml`") and Rollout 6 ("contributor prerequisite: rustup one-liner and the Linux
  system-deps list (A16)") — the gate now depends on a binary that is not part of the pinned
  toolchain and is nowhere acquired or pinned. `cargo-deny` is not a rustup component and not a
  workspace dependency, so neither `rust-toolchain.toml channel = "1.85.0"` nor the committed
  `Cargo.lock` + `--locked` story covers it: the `rust` CI job as specified has no install step,
  the contributor prerequisite list does not mention it, and Rollout 5 records "the cargo gate
  commands" without saying how each is obtained. Its advisory database is also live network state,
  so the fourth command can flip an unchanged PR red on any day RUSTSEC publishes — the one gate
  step outside the design's "every contributor and CI job builds identically" goal (Goals). Test
  strategy #5 compounds it: the one-gate promise enumerates only `cargo fmt`, `cargo clippy`,
  `cargo test`, so after this revision the promise no longer covers the whole gate and nothing
  exercises the fourth command anywhere. Fix: name the acquisition and the pin once in Gates and
  repeat it in Rollout 4/6 — `cargo install cargo-deny@<pinned> --locked` (or a SHA-pinned install
  action) in the `rust` job, the same pin in the contributor prerequisite — pin the advisory
  database revision in `deny.toml` (`[advisories] database = <rev>`) or state that the command is
  network-dependent and allowed to change result; and either add `cargo deny check advisories` to
  #5's enumeration or replace #5's three-command list with "the gate commands listed in Gates".
  If the fourth command is instead dropped from the per-PR gate, that also closes the finding, but
  it must be said in Gates, not left implied.

- [minor][re-raised] Test strategy #7 / Rollout — "the gate commands and the `rust-toolchain.toml`
  auto-install are verified once on Linux and once on a Windows contributor machine during rollout,
  which is where Windows gate greenness is established" — the verification is asserted to happen
  "during rollout" but no rollout step performs it: step 7 is "Verify Test strategy #5 on a fresh
  clone before announcing to contributors", and #5 is the Linux fresh-clone gate. The matrix still
  claims `| Contributor dev env | Linux + Windows yes |`, so the only thing standing behind that
  cell is currently unassigned work — exactly where path-length and case-insensitive-filesystem
  breakage first appears (r1's concern, narrowed: the mechanism is now chosen, only the schedule is
  missing). Fix: extend Rollout 7 to "Verify Test strategy #5 on a fresh clone and the Windows leg
  of Test strategy #7 on a Windows machine before announcing to contributors."

- [nit][new] Compatibility matrix — "| Postgres (hub runtime, hub era) | yes (target) |
  compile-time path per the Options sqlx decision; server version pinned by ADR-014 |" — deleting
  the hub-era `.sqlx` test from Test strategy left this the only row with a positive status and no
  test anywhere in the design: the Windows `yes (target)` row at least gets #7's "the desktop
  packaging matrix is exercised only when the desktop design lands". Checklist item 7 is therefore
  satisfied by omission rather than by statement. Fix: one clause in that row's Notes — "tested by
  ADR-014's strategy, see Deferred" — so both `yes (target)` rows name where they get exercised.

## FYI (no severity)

- `cargo deny check advisories` extends the check gate beyond ADR-008 clause 4's named "`cargo
  clippy` + `cargo fmt`"; that is an extension rather than a contradiction of the ADR, but the
  scope question belongs to the complexity seat.
- `license = "Apache-2.0"    # matches the LICENSE file at the repo root` against
  `tools/orchestration/package.json`'s `"license": "MIT"`, recorded in A23 as "a repo fix outside
  this design" — a legal/repo-consistency matter, not a compatibility one; flagging only that the
  mismatch stays open at merge.
- The new "fixture file names are lowercase ASCII, `-`-separated" rule is scoped to file names, and
  non-ASCII binding names *inside* fixtures are still unaddressed here; Deferred assigns
  "patch/policy/manifest semantics and their fixture cases" to ADR-009, so it is a consistent
  deferral — worth confirming ADR-009's corpus actually contains a non-ASCII-name case.
- `resolver = "3"` in a virtual workspace: valid, and the comment's reasoning is a correctness
  detail for another seat, not a platform concern.
