# Code review — compatibility seat, round 1 pass c0

**Change under review:** `git diff 4caa688933b2969f569f6126dddb713180c7c410...HEAD -- ':(exclude)docs/designs'`
(22 files, +1639/−33).
**Design:** `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`. **Source task:**
`docs/tasks/T-001-steam-linux-discovery.md`.
**Inputs:** the diff above, the proposal, the task, the runner's gate record, my own quoted
previous findings (none — first pass). No other run record was opened.
**Checklist applied:** `.opencode/skills/code-review/references/compatibility.md` (7 items), only.
**Verdict:** `request-changes` — 1 major, 2 nit.

I re-ran the project's read-only gate myself:
`cargo test --workspace --locked` → green, with
`tests/keyvalues_tests.rs … test result: ok. 14 passed`,
`tests/steam_linux_tests.rs … test result: ok. 23 passed`,
`tests/cli_tests.rs … test result: ok. 3 passed`.
The gate record's only captured `test result: ok. 0 passed; …` tail is an empty-units/doc-test
leg, not a skipped suite; the real suites run and pass, so no compatibility claim rests on a
leg that executed nothing.

## Findings

### 1. [major] `crates/discovery/src/game.rs:8` — the change silently re-spells `platform` on the JSON channel the proposal promised was unchanged

`#[serde(rename_all = "kebab-case")]` is added to `Platform`. Every prior serialisation of
`DiscoveredGame.platform` was the default variant name — `"SteamLinux"` / `"SteamWindows"` —
and it is now `"steam-linux"` / `"steam-windows"`. Because `Deserialize` derives from the same
renamed identifiers and no `#[serde(alias = …)]` is present, JSON that `discovery list --json`
printed before this commit no longer deserializes into `DiscoveredGame` (`unknown variant`).

This is a violated requirement, not a style call:

- proposal `Context` (02-proposal.md:14-15): "The task is additive: **no change to
  `DiscoveredGame`, `Platform`**, the `DiscoveryPlugin` trait, `normalize_slug`,
  `DiscoveryEngine`, or any `schemas/` spec."
- proposal `Non-goals` (:35): "Any change to `DiscoveredGame` … " is out of scope.
- proposal §8 (:262): "The printing code is untouched, so **the table header and JSON shape are
  unchanged**."

The break is also the least visible kind. The human table is untouched because `Display`
already printed `steam-linux` (game.rs:14-21), so `test_cli_list_format` and every human reader
see no difference; only `--json` changes — the exact machine-readable channel the proposal
itself reasons about (§7: "a `--json` consumer must wade through"). Nothing in this run pinned
the old spelling, so nothing caught it: the base `cli_tests.rs` asserted only `[]`, and the new
assertion `apps/discovery-cli/tests/cli_tests.rs:121` (`obj["platform"] == "steam-linux"`) was
written against the new behaviour. There is no version umbrella for the change either — no file
under `schemas/` mentions `platform` or `steam-linux` at all (grep: no matches), and `data/`
contains no consumer, so the wire format is defined only by this derive.

The *direction* is defensible — ADR-022 (`docs/adrs/022-game-discovery.md:34`) names the values
`steam-linux`, `steam-windows` — which is why the fix below is not simply "revert". But a
serialised-name change to an existing public type cannot ride inside an additive plugin task
under a proposal that states the opposite twice.

**Fix:** either drop the derive attribute from this change (T-001 is additive; `Platform` stays
as proposed), or make the break deliberate and survivable: keep
`#[serde(rename_all = "kebab-case")]` and add `#[serde(alias = "SteamLinux")]` /
`#[serde(alias = "SteamWindows")]` so previously emitted output still deserializes; pin both
spellings in a round-trip test in `crates/discovery/tests/game_tests.rs`; record the renamed
serialised form in ADR-022 (or a schema, which `schemas/` currently lacks for this payload);
and correct §8's "JSON shape … unchanged" sentence so the run record matches the code. If the
rename stays, the `cli_tests.rs:121` expectation moves with it as one statement, not two.

### 2. [nit] `crates/discovery/src/steam_linux.rs:273-280` — `installdir` is validated against POSIX separators only

`usable_install_dir` refuses `''`, `.`, `..`, absolute and `/`, matching §5 exactly, so nothing
on a supported platform can traverse: on Linux `\` is an ordinary filename byte and
`..\..\x` stays one directory name. The residual exposure is the emitted `PathBuf` itself —
§5's stated reason for the guard is that the value "is appended unvalidated to the emitted
path" that a later consumer opens, and a consumer on a separator-agnostic host reads `\` as a
separator, so the guard the design names covers one of two spellings.

**Fix:** add `&& !install_dir.contains('\\')` and extend §5's reject list to
`` `.`/`..`/absolute/`/`/`\` ``, so the guard is separator-agnostic rather than
platform-lucky; or state in §5 that the emitted `install_path` is a Linux path by contract.

### 3. [nit] `crates/discovery/src/steam_linux.rs:126-129` — the Snap probe is stricter than the Compatibility matrix row it implements

§2 item 4 and the matrix's Snap row say the trio is taken from `$SNAP_USER_DATA/steam/common`
"**when that variable is set**"; `absolute_env` additionally requires the value to be absolute,
so a set-but-relative `SNAP_USER_DATA` silently falls back to `$HOME/snap/steam/common`. The
direction is the safe one (A9's relative-is-invalid rule, and real Snap always exports an
absolute `SNAP_USER_DATA`), so there is no user-visible loss — but the matrix is the document a
Flatpak/Snap user reads, and code and row now describe different probes.

**Fix:** either say "when it is set to an absolute path" in §2 item 4 and the matrix's Snap row,
or drop the absoluteness test for this one variable.

## Checklist walk-through

1. **Platform code matches the Compatibility matrix — pass.** Each "Yes"/"Partial" row has a
   code path: `host_candidates` (steam_linux.rs:99-135) emits exactly §2's ten candidates in
   §2's order, pinned path-by-path by `steam_linux_tests.rs:216-317` (native `~/.steam/*`,
   Flatpak trio, Snap trio); `$XDG_DATA_HOME` honoured only when absolute (:109-112, tests
   :237, :260); absolute library `path`s pass through as-is for arbitrary mounts (:152-158);
   the `steamapps`-directory gate in `canonical_paths` (:326) is the "sandbox-only paths are
   dropped" mechanism the two Partial rows claim; "Steam not installed / no home directory →
   `Ok(vec![])`, no error, no stderr" holds — `from_host` (:39-43) yields no candidates for an
   unset/empty/relative `HOME`, and `engine.rs:20-33` prints nothing on `Ok`, verified by
   `steam_linux_tests.rs:452` and `cli_tests.rs:143`. Windows / macOS / Deck are "No" and have
   no code path. **Not a finding:** `steam_linux.rs` carries no `#[cfg(target_os = "linux")]`
   guard, so a Windows/macOS build compiles the Linux policy and probes `HOME`. It cannot do
   harm there (the candidate list matches nothing those platforms write, and the result is
   `Ok(vec![])`), the proposal never asks for a cfg gate, and no matrix row is contradicted —
   flagging it would be inventing a defect.
2. **Node floor — pass.** The only Node-side edit is a constant,
   `tools/orchestration/src/pipeline/flows.ts:92` (`15 * 60_000` → `12 * 60_000`): no API above
   `package.json`'s `engines.node: ">=22.23.5"`, nothing version-sensitive. It moves code
   *toward* the contract: `docs/workflows.md:341` documents the light flow's `stageTimeoutMs` as
   12 min, and `tools/orchestration/tests/pipeline/flows.test.ts:104` already asserted
   `12 * 60_000` at the base commit, so the edit satisfies the suite rather than regressing it.
3. **Paths — pass with nit #2.** Every composed path is a `join`: `home.join(".local/share/Steam")`,
   `flatpak.join(suffix)`, `snap.join(suffix)` (:109-132), `root.join("steamapps").join(…)`,
   `vdf.parent().join(path)` (:150-158), `library.join("steamapps").join("common").join(&install_dir)`
   (:238-241), and the tests' cross-package fixture path (`cli_tests.rs:13-15`). No string
   concatenation, no `..` survives output (`canonical_paths` :320-332, canonicalize + sort +
   de-dup via `BTreeSet`). Case assumptions are deliberate, not accidental: `steamapps` lowercase
   only (A20 decision; matrix row "legacy `SteamApps` spelling | Partial"), `libraryfolders.vdf`
   exact, `appmanifest_`/`.acf` matched case-sensitively (:212-215) — matching what the Linux
   client writes, with the legacy spelling explicitly *Deferred*. Keys resolve
   `eq_ignore_ascii_case` key matching (`keyvalues.rs:51-66`) is first-match per A5.
4. **Guarded platform imports — N/A.** The two new modules import nothing beyond
   `std::fs`/`std::env`/`std::path` and crate siblings, so there is no platform-only module that
   could break load on another platform.
5. **Locale / layout reality — pass.** Both file kinds are read as bytes and validated as UTF-8
   once (`read_text_file` :306), and the reader tokenises byte-wise, so a non-ASCII `name` or
   `installdir` survives quoted and bare values intact (no truncation at a multibyte boundary);
   `normalize_slug` (slug.rs:1-35) keeps Unicode `is_alphanumeric()` characters, so a
   non-Latin-script title yields a populated slug rather than a bare `unknown/` row; a non-ASCII
   `--steam-root` reaches `PathBuf` through clap without a UTF-8 assumption. ASCII-only
   `eq_ignore_ascii_case` key matching loses nothing: every key in both file kinds is an ASCII
   identifier.
6. **Native dependencies — pass.** The diff adds no dependency: no `Cargo.lock` or `Cargo.toml`
   entry appears in it, and my own `cargo test --workspace --locked` runs green, which is §10's
   and A15's "the dependency posture held" check.
7. **Tests run on the platforms they claim — pass.** Every new test is an ungated `#[test]` with
   no `#[ignore]` and no host probe: fixture roots come from `env!("CARGO_MANIFEST_DIR")`
   (`steam_linux_tests.rs:18-22`), the CLI binary from `CARGO_BIN_EXE_discovery-cli`
   (`cli_tests.rs:24`), and the fixtures carry relative library `path`s (`".."`, `"../secondary"`)
   rather than this machine's absolute paths, per §9. All ten Test-strategy legs (#1-#9) are
   therefore hermetic on `ubuntu-latest` with no Steam; my run's counts above (14 / 23 / 3) are
   the evidence. Nothing is platform-guarded-and-therefore-skipped-everywhere.

## Evidence I could not establish (recorded, not blocking)

- The Flatpak and Snap "Partial" rows are established by reading the candidate list and the
  `steamapps` gate, not by a test: no fixture models a sandbox-only or unreadable library, and
  A10's real behaviour was not exercised. Not blocking — both rows are labelled *Partial*, the
  dropping mechanism is code-visible at :326, and no claim inside those rows is stronger than
  "probed, then dropped by the gate".
- Rollout step 5 (one real `appmanifest_*.acf` and a real `discovery list` capture on a real
  Steam install) is an explicitly human step still open, so A6/A7's grammar subset and A20's
  lowercase-spelling assumption remain unverified against a real file by design. T-001's
  Done-when is met by the implementation and the hermetic suite; the residual is the proposal's
  own stated risk, not a gap this change introduced.

## FYI (no severity, outside this seat's checklist)

The branch also carries three harness files outside T-001's declared scope:
`.opencode/agents/code/coder.md` and `.opencode/agents/code/test-engineer.md` flip
`transcripts` read/glob/grep from allow to deny (which is what `docs/workflows.md:119-126`
documents for every shell-holding stage agent), and `flows.ts:92` as covered in item 2. All
three move code toward an already-documented contract, so none has a compatibility impact;
whether a product change should carry them is a scope question for another seat.
