# compatibility review r1 (pass c0, full panel, whole document)

verdict: request-changes

Inputs used: `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md`, the proposal's Revision log, and
read-only inspection of the code the design cites (`crates/discovery`,
`crates/core`, `apps/discovery-cli`, `schemas/`, `.github/workflows/check.yml`,
`Cargo.lock`, `tools/orchestration`). No review report or transcript was opened.

All Revision-log entries were read first; none of the findings below is a
re-report of a rejected decision. The three recorded divergences (`017-C1`'s
bound, `017-C3`'s `inputs`, `017-C4`'s "same scan") are argued soundly against
A5/A13/A20 and are not re-opened here.

## Findings

- [major][new] Compatibility matrix, Flatpak / Snap Steam row — "The face
  inherits `host_candidates` roots (A3) unchanged, and every candidate template
  is relative to the owning root, not `$HOME`" — the sentence is false of one of
  the three resolvers the design ships. §Config-path resolvers defines
  `NativeHost` as `$XDG_CONFIG_HOME`, else `$HOME/.config`, read from the
  BindForge process's own environment; it is root-independent, while only
  `SteamUserdata` (`<ctx.root>/userdata/…`) and `ProtonPrefix`
  (`<ctx.library>/steamapps/compatdata/…`) are root-relative. A Flatpak or Snap
  Steam install puts a native-launched game's `$HOME` inside the package
  (`$HOME/.var/app/com.valvesoftware.Steam/config/…`,
  `~/snap/steam/common/.config/…`), which no shipped template reaches, so the
  row's unqualified `Yes` rests on a premise §Config-path resolvers contradicts;
  a UE title bound that way answers no-claim, not the claimed support. Evidence:
  `crates/discovery/src/steam_linux.rs:112-125` (the flatpak/snap root
  candidates the row leans on) against the proposal's own `NativeHost` rule.
  Fix: narrow the row to what the templates actually cover — root-relative
  trees (`SteamUserdata`, `ProtonPrefix`) Yes; the sandboxed launcher's own
  config home is not searched, so the row's honest status for a native-context
  title is no-claim — and either derive a per-root config-home candidate or name
  the gap in §Deferred beside `017-R2`.

- [major][new] Compatibility matrix, Steam Deck and macOS rows — "Steam Deck |
  Unknown" and "macOS | Unknown" — the checklist admits no `unknown`: every row
  must commit to a status, and both of these are decidable from text and code the
  design already cites. macOS is a definite `No`: `crates/discovery/src/game.rs`
  publishes `Platform` as exactly `SteamLinux`/`SteamWindows` (game.rs:21-28),
  `apps/discovery-cli/src/main.rs:131-132` constructs only those two plugins, so
  no macOS row can ever exist for `GameContext::from_record` to consume — and
  §Deferred already defers "the macOS Steam root" to `017-R2`, which contradicts
  calling the row unknown. The Deck is a Linux Steam install with the same
  `$HOME`-relative layout and the same Proton prefix rule the Proton row claims
  `Yes` for, so its status follows from the Linux row; "no Deck leg exists (A22)"
  is a coverage fact, not a compatibility status. Fix: mark macOS `No — no macOS
  discovery source exists, so no row resolves` (pointing at `017-R2`), and the
  Deck `Yes, by the Linux row` with the no-Deck-fixture note moved into that
  row's Notes.

- [major][new] Test strategy #16 and Rollout item 9 — "the workspace
  `Cargo.lock` is unchanged" / "any `Cargo.lock` change is a finding" — the pin
  contradicts the run's own required gate. `Cargo.lock` records every workspace
  member (`bindforge-core`, `bindforge-transforms`, `discovery`,
  `discovery-cli` each carry a `[[package]]` stanza at Cargo.lock:56,72,141,151),
  so landing `crates/codecs` necessarily adds a `bindforge-codecs` stanza plus
  the `apps/discovery-cli` → `bindforge-codecs` edge, and Rollout 9's own
  `cargo test --workspace --locked` fails while the lock is stale. As written the
  design makes its own gate unsatisfiable, and an implementer must break the
  stated rule to pass it. Fix: restate both places as the narrower claim the
  matrix row already makes — "the only `Cargo.lock` delta is the
  `bindforge-codecs` stanza and the `discovery-cli` dependency edge; no
  third-party package enters and no version moves" — and pin that instead.

- [major][new] The CLI face, step 5 — "`EXIT_FAILURE` 1 (usage, undiscovered
  slug, scan failure — today's one voice) · `claim-refused` 2" — the code assigns
  exit 1 to a class that never reaches this face's code. The binary parses
  through clap 4 (`Cli::parse()`, apps/discovery-cli/src/main.rs:73;
  `clap = { version = "4", features = ["derive"] }`,
  apps/discovery-cli/Cargo.toml:11), and clap exits **2** on a usage error before
  `main` runs. So `discovery bindings` with a missing `<slug>` or a misspelled
  flag exits 2 — the code the design reserves for `claim-refused` — which breaks
  the section's own guarantee that "the code is a function of what stdout showed"
  and makes Test strategy #13's "its own exit code (0/2/3/4)" pin a contract
  consumers cannot decode. Fix: state that the face parses with
  `Cli::try_parse()` and routes the parse error through the existing
  `error: ` + `ExitCode::FAILURE` voice (main.rs:80-83), keeping usage at 1; or
  move the four status codes above clap's 2 and say so in the same sentence.

- [minor][new] The first family fixture list and Test strategy #6 —
  "`symlinked-prefix`" / "a symlinked directory and a symlinked `.ini` in
  `symlinked-prefix` are neither descended nor read" — the design never says how
  those links come to exist, and the most likely route breaks the gate Rollout 9
  promises. The Windows leg is
  `cargo check --workspace --exclude bindforge-registry --all-targets --target
  x86_64-pc-windows-msvc` (.github/workflows/check.yml:42-46), so `crates/codecs`'
  **test** targets are compiled for `x86_64-pc-windows-msvc`; a test-time
  `std::os::unix::fs::symlink` helper (the shape a scratch-tree fixture needs, and
  this would be the repo's first symlink test — no existing `crates/**.rs` calls
  `symlink`) does not compile for that target, while a symlink checked into git is
  not a link in a Windows checkout. Fix: one sentence pinning the mechanism — links
  created at test time behind `#[cfg(unix)]` with a `#[cfg(not(unix))]` early
  return so the compile leg stays green — plus a note that the assertion runs on
  the Linux leg only.

- [minor][new] `bindings/v1` bounds — "`user` one of the three spellings above" —
  the contract never says whether those three values are a closed set the loader
  refuses anything outside, or simply the set `v1` emits. The choice is a
  compatibility commitment with a permanent cost on one side: `bindings/v1` ages
  through majors, and `017-R2`'s Windows launcher context has a local user who is
  neither a SteamID nor `steamuser` nor `host`, so a closed enumeration forces
  either an overload of `host` (losing the real label) or a format major. Fix: one
  sentence — the loader validates `user` against a bounded token grammar (the
  shared slug-shaped bound plus the same control-character and space rules the
  section already states) and `spec.md` lists the spellings `v1` emits; or keep
  the closed set and state plainly that a new launcher-context label is a
  `bindings` major.

- [minor][new] Read discipline and budgets / `bindings/v1` bounds — "each input
  token the same bound" and the `ExtractionRefusal` variant list "`FileOverBudget`,
  `UndecodableText`, `UnparsableEntry` … and `SnapshotEntryBudget`" — a located
  line whose verbatim `Command=` or `Name=` string exceeds
  `MAX_NATIVE_NAME_OCTETS` has no named outcome at the extraction seam. The format
  refuses an over-bound value "naming the key" only for a *loaded* document; at
  extraction the family must either drop the pair (which §Deferred denies: "`v1`
  reports the file's tokens verbatim and drops nothing") or refuse under a name the
  list does not carry, which `017-L1` forbids ("a parse that fails after a claim is
  a named `ExtractionRefusal`"). UE's `Command` field is a free-form console command
  line, so this is real-file territory, not a hypothetical. Fix: name the outcome —
  state that an over-bound verbatim token is an `UnparsableEntry` carrying the
  bound's name with the tree-relative path and 1-based line, or add one variant —
  with the status staying `extraction-refused`.

- [minor][new] Read discipline and budgets — "MAX_USER_DIRS_PER_GAME = 16 | `userdata/`
  directories considered per game" and Test strategy #3's "a 17th digit-named
  directory is past `MAX_USER_DIRS_PER_GAME` and unvisited" — this is the one
  budget in the design that ends in silent omission rather than a named outcome: a
  machine with more local Steam users produces fewer snapshot documents and no
  status, refusal or stderr line says so, so an operator cannot distinguish "every
  user's bindings" from "the first sixteen". Every other budget here names itself
  (`FileOverBudget`, `SnapshotEntryBudget`, the candidate cap's refusal). Fix: one
  sentence deciding it — either the cap is reported (counted into the run's output
  as a named line while the run still exits `ok`) or `schemas/bindings/v1/spec.md`
  and the README budget bullet state it as a silent bound on users per game.

- [minor][new] Compatibility matrix, Windows host row — "no Windows tree template
  ships, so the face answers `claim-refused`" — the stated reason is not what the
  design does. `SteamUserdata` and `ProtonPrefix` are root-relative and
  OS-neutral, so on a Windows host with a discovered Steam row both templates are
  built and stat'd; the `claim-refused` answer arrives because no `Saved/Config`
  grammar is located inside them, not because the face resolves nothing. The row
  therefore describes the wrong failure to whoever reads it when `017-R2` starts.
  Fix: restate as "the root-relative trees resolve; the Windows host config home
  (`%LOCALAPPDATA%`, `Documents\My Games`) ships no template, so a real title
  answers `claim-refused`".

- [nit][new] The first family — "collecting `*.ini` files whose path ends
  `Saved/Config/<one directory>/<name>.ini`" — the design does not say the
  comparison is component-wise. A suffix test on `path.display()` is
  separator-dependent and would read differently on the `017-R2` Windows trees the
  same resolver section hands to a follow-on. Fix: one clause — matched on
  `Path::components()` (suffix components plus `file_name()` ending `.ini`), never
  on a displayed string.

## Checklist disposition

1. Compatibility matrix — every row carries a status; two `Unknown` rows are
   findings (above). Two rows' statuses are argued from premises the design itself
   contradicts (above). No empty cells.
2. OS input APIs (X11 / Wayland / Raw Input / HID) — **not applicable**: nothing
   in this run touches an input event API; every fact arrives from a file on disk,
   so no per-platform input-API target is owed.
3. Node version floor — **not applicable**: `tools/orchestration` is untouched
   (`engines.node: ">=22.23.5"`, and no pipeline source reads discovery output), so
   no Node API is used at or above the floor.
4. Native modules — clean: no new third-party crate and no build-script/C
   dependency is introduced; the packaging concern that does land here is the
   `Cargo.lock` pin (finding 3).
5. Keyboard reality — the layout story is sound by posture: tokens are the file's
   own words, UTF-16 and BOM cases are handled in `read::decode`, and the
   action/token character rules match `schemas/manifests/v1/spec.md`'s native-name
   grammar (1–128 octets, no ASCII control character, no leading/trailing space),
   so a snapshot action is joinable as a manifest row. Dead keys and IME are not
   applicable to file extraction. The reality gap left is the unbounded verbatim
   string (finding on the over-bound token).
6. File paths — separators and per-OS config dirs are library-shaped
   (`PathBuf`, `absolute_env`, the `XDG_CONFIG_HOME`/`$HOME/.config` convention
   mirroring `steam_common::absolute_env`); the residual items are the
   root-independence of `NativeHost` (finding 1) and the suffix-match wording
   (nit).
7. Platform test per supported platform — Linux, Proton and the
   NTFS/exFAT-symlink degradation are named (`native-linux`, `proton-prefix`,
   `steam-userdata`, `symlinked-prefix`); the **Flatpak / Snap row has no named
   test**, which the finding on that row's status covers rather than counting
   twice.

## FYI (no severity)

- §Local paths decides that `--probe` prints absolute paths while `013-L5` and the
  README's no-path-in-a-message rule govern documents "that arrived from
  elsewhere". The scoping argument is coherent; whether it is the right line for a
  tool whose output an operator will paste into issues is the security seat's call.
- The aggregate read-cost sentence (512 MiB worst case for one game) and the
  `16 × 32` product are the performance seat's to weigh; the shape matches the
  README's `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` precedent, which I verified.
- Determinism's single ordering law, and `serde_json::to_string` giving
  struct-declared key order, are the correctness seat's; the ordering law is at
  least stated once and cited elsewhere, which is what the contract tree expects.
