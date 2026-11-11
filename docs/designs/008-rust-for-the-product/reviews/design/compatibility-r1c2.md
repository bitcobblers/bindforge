# compatibility review r1 (dissent pass c2)

verdict: approve

Scope: `git diff 6e4ea21 796b2a6` on `02-proposal.md`, then the full current
document, against `docs/adrs/008-rust-for-the-product.md` and the Revision
log. Reviewed independently; no other seat's report or transcript opened.

## Prior findings — closure check

- [closed] Gates / Rollout 1 / Rollout 6 / Test #5 (r1 major, gate tool
  acquisition). The author took the third option offered: the command is out
  of the per-PR gate and Gates says so explicitly — "Per-PR
  dependency-advisory scanning is deliberately excluded from the gate: it
  needs a tool outside the pinned toolchain and an advisory database that is
  live network state". The gate block is back to exactly three commands, all
  shipped by the pinned rustup toolchain (A11), so the `rust` job needs no
  install step; Rollout 1 no longer adds `deny.toml`; the contributor
  prerequisite (Rollout 6) is complete for the gate as specified; and Test #5
  now says "every command listed in the Gates block", so the one-gate promise
  cannot drift from the gate. The exclusion is recorded in Deferred with
  matching wording, and the Revision log entry matches the text. Removing the
  command does not contradict ADR-008: clause 4 fixes "`cargo clippy` +
  `cargo fmt` as the `check` gate", and the ADR nowhere requires advisory
  scanning — so this is not a re-opened deferral.
- [closed] Test #7 / Rollout 7 (r1 minor, unassigned Windows verification).
  Rollout 7 now reads "Verify Test strategy #5 on a fresh Linux clone and the
  Windows leg of Test strategy #7 on a Windows contributor machine before
  announcing to contributors", and Test #7 names "(Rollout 7)" as where the
  Windows leg happens. The `Contributor dev env | Linux + Windows yes` cell
  is now backed by a scheduled step rather than an assertion.
- [closed] Compatibility matrix / Postgres row (r1 nit). The row now reads
  "server version pinned and tested by ADR-014's strategy, see Deferred", so
  both `yes (target)` rows name where they get exercised and checklist item 7
  is satisfied by citation rather than omission.

## Findings

- [nit][new] Chosen design → Workspace shape / Gates / Test strategy #5 —
  "`rust-toolchain.toml` with `[toolchain] channel = "1.85.0"`" … "`rust`
  (the three gate commands above, and nothing installed beyond the pinned
  toolchain (A11))" … "every command listed in the Gates block succeeds …
  with nothing installed beyond the pinned toolchain." — the revised text
  makes "nothing installed beyond the pinned toolchain" a load-bearing claim
  (it is now both the reason advisory scanning is excluded and part of
  Test #5's promise), but two of the three gate commands are rustup
  *components*, not part of the `channel` pin: `cargo fmt` needs `rustfmt`,
  `cargo clippy` needs `clippy`. They are present only because rustup's
  default profile happens to include them, so a `--profile minimal`
  contributor install, or a toolchain that predates them, fails the gate for
  a setup reason rather than a code reason. A11's basis covers the channel
  pin, not components. Fix: in Workspace shape, pin the components with the
  channel — `[toolchain] channel = "1.85.0", components = ["clippy",
  "rustfmt"]` — so the claim is guaranteed by the committed file instead of
  by the installer profile default.

## Checklist notes (no findings)

1. Matrix: every row has a status; no `unknown`, no empty cells.
2. Input APIs: Context states the design targets no X11/Wayland/Raw Input/HID
   API ("BindForge reads and writes binding *files* only") and pushes any
   future capture component to name its API in its own matrix — justified
   abstraction.
3. Node floor: unchanged text; `setup-node` pins the version explicitly and
   Rollout 5 carries the floor into `repo-practices`; no new Node API above
   the floor is introduced.
4. Native modules: no silent native dependency — the Linux dev deps are named
   (A16, Rollout 6); desktop packaging is deferred consistently with ADR-008
   clause 2 ("The webview UI framework is deferred to the desktop design").
5. Keyboard reality: unchanged; format-level naming constraints (lowercase
   ASCII, `-`-separated, case-unique fixture names) and binding-name semantics
   stay with ADR-009.
6. File paths: path canonicalization and `..`/absolute rejection deferred to
   ADR-012, matching ADR-008 clause 2 (`crates/adapters` owns config path
   discovery, write policy) — consistent, not re-opened.
7. Platform test per supported platform: Linux via Test #5/#7, Windows via
   Test #7's Windows leg scheduled in Rollout 7, CI runner covered, Postgres
   via the ADR-014 citation.

## FYI (no severity)

- The Deferred line "Dependency-advisory scanning (`cargo deny`, a root
  `deny.toml`) — no follow-up yet" leaves supply-chain advisory scanning with
  no owner at all in any era; the compatibility-consistent exclusion argument
  (network state, unpinned tool) does not decide whether some pinned, offline
  alternative belongs to the hub era. That is the security seat's call, not
  mine, and ADR-008 does not require it.
