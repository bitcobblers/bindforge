# Code review — security seat — round 1, pass c0

- Seat: security (full panel c0)
- Scope: `git diff 4caa688933b2969f569f6126dddb713180c7c410...HEAD -- ':(exclude)docs/designs'`
- Inputs used: the diff above; `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`;
  `docs/tasks/T-001-steam-linux-discovery.md`; the runner's gate record (green, 2026-11-18T04:11:25Z);
  the repo's read-only checks run by this seat.
- Previous findings: none (first pass for this seat).

## Checklist walk-through

1. **External input validated at the boundary** — PASS with one minor finding.
   `libraryfolders.vdf` and `appmanifest_*.acf` (untrusted, user-editable) are parsed only by the
   hand-rolled `keyvalues::parse`, which enforces an explicit accepted subset and rejects conditionals,
   `#base`, unbalanced braces, unterminated strings and >16-level nesting with positioned errors
   (`crates/discovery/src/keyvalues.rs`), rejects >1 MiB documents (`keyvalues.rs` parse, cap
   `MAX_DOCUMENT_BYTES`), and requires valid UTF-8 at `read_text_file`. No code evaluation anywhere.
   The per-file size cap, however, rests on a pre-read `metadata()` probe that does not bound the read
   for non-regular files — see finding S1.
2. **Path handling** — PASS. `installdir` is refused when empty, `.`, `..`, absolute, containing `/`,
   or containing a NUL before it is appended to the emitted path
   (`crates/discovery/src/steam_linux.rs:259-271`); `install_path` therefore carries no unresolved `..`
   and no interior NUL (§5 / §2 canonical statement honored). Library paths go through the canonical
   path rule — `std::fs::canonicalize` + `steamapps`-directory gate + sort + de-dupe
   (`steam_linux.rs:316-328`) — before any `read_dir`, so `..` in a VDF `path` cannot escape into a
   scan of an arbitrary tree without that tree also looking like a Steam library. No
   string-interpolated shell commands exist in the diff; the CLI test invokes the binary via
   `Command::new(env!("CARGO_BIN_EXE_discovery-cli")).args(argv)` — argv, no shell
   (`apps/discovery-cli/tests/cli_tests.rs:24-26`).
3. **Injection** — PASS. No shell command construction and no regex anywhere in the diff (grepped); the
   only interpolation into strings is the §7 error message, which carries root paths and a skip count
   (`steam_linux.rs:74-86`), and the parser is linear-time over a size-capped input, so there is no
   catastrophic-backtracking or unbounded-parse surface.
4. **Secrets and PII** — PASS. No secrets in code, fixtures or log lines; fixtures use synthetic
   9000xx AppIDs and machine-free relative library paths. The §7 message is explicitly content-free and
   the test pins it: `discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents`
   asserts against a leak list (`crates/discovery/tests/steam_linux_tests.rs:464-492`). The message
   reaches stderr only, via the unchanged engine `eprintln!` (`crates/discovery/src/engine.rs:28`), out
   of the `--json` stdout stream.
5. **New dependencies** — N/A. `git diff` over `Cargo.lock`, `crates/discovery/Cargo.toml` and
   `apps/discovery-cli/Cargo.toml` is empty; the reader is hand-rolled as designed (§10). No advisory
   scan needed.
6. **Filesystem writes** — PASS (none). The plugin is strictly read-only; no test creates, writes,
   renames or removes anything (grepped: no `fs::write/create/remove/File::create` in the three test
   files); the fixture tree is checked-in read-only. No atomicity concern because no state is written.
7. **Clean rejection of malformed/hostile input** — PASS. Skip-and-continue per file
   (`read_text_file` / `scan_library` / `library_paths` count skips, never abort); the single §7 `Err`
   carries only roots + count; no `unwrap`/`expect` on plugin-controlled data in the product path (the
   only `unwrap_or` is on `vdf.parent()`, which is statically `Some`); recursion depth is bounded by the
   16-level cap so hostile nesting cannot overflow the stack; gate record green and independently
   re-run by this seat: `cargo test --workspace --locked` with 14 `keyvalues_tests`, 23
   `steam_linux_tests` and 3 `cli_tests` all passing.

## Findings

### S1 (minor) — per-file size cap does not bound the read for non-regular files or post-metadata growth

- Location: `crates/discovery/src/steam_linux.rs:285-311` (`read_text_file`), read path at
  `crates/discovery/src/steam_linux.rs:168` (`library_paths`).
- Problem: the 1 MiB cap is decided from a `std::fs::metadata(path)` length and then applied with
  `std::fs::read(path)`. For character devices and similar special files (`/dev/zero`, `/dev/urandom`)
  `metadata().len()` is 0, so a symlink at the fixed VDF path — `…/steamapps/libraryfolders.vdf →
  /dev/zero` — passes the cap check and `fs::read` then reads an unbounded stream into a growing
  `Vec`, OOM-killing or hanging `discovery list`. The manifest leg is guarded by
  `path.is_file()` (`steam_linux.rs:216`); the VDF leg (`steam_linux.rs:168`) has no such gate.
  The metadata-then-read split is also a TOCTOU: a regular file grown past the cap between the probe
  and the read is fully allocated before `parse` rejects it. This weakens the design's stated per-file
  bound (§4 "Files larger than 1 MiB are skipped as malformed"; Risks: "no recursion, 1 MiB per file")
  for non-regular files. Severity is minor, not major: the precondition is write access inside a
  scanned root's `steamapps/` — the same access that already lets an attacker rewrite the VDF to name
  any directory — and the impact is DoS of a read-only CLI command, not leakage (file contents never
  reach any output, per §7) or integrity loss.
- Fix: bound the read by the handle, not by pre-read metadata: `File::open`, check
  `file.metadata()?.is_file()`, then `Read::take(MAX_FILE_BYTES as u64 + 1)` into a buffer and treat
  "read hit the limit" as an oversize skip. This makes the cap hold for every file type and closes the
  growth race in one step; alternatively add an `is_file()` gate at `steam_linux.rs:168` and keep the
  current read (weaker — does not close the growth race).

## FYI (out of specialty, no severity)

- `crates/discovery/src/game.rs:8` adds `#[serde(rename_all = "kebab-case")]`, changing `Platform`'s
  serialized value from `"SteamLinux"` to `"steam-linux"`; the proposal's Context said `Platform` is
  unchanged and §8 said the JSON is unchanged (the field set is unchanged; the value is not).
  Compatibility-seat territory.
- The diff bundles pipeline infrastructure unrelated to T-001:
  `.opencode/agents/code/{coder,test-engineer}.md` (transcript-read permissions flipped allow→deny — a
  tightening) and `tools/orchestration/src/pipeline/flows.ts` (light-flow `stageTimeoutMs` 15→12 min).
- Untrusted `display_name` reaches the table/JSON output verbatim; `normalize_slug` scrubs control
  characters from the slug only, so a hostile manifest could place terminal escape sequences in the
  printed Display Name. Outside this checklist's enumerated injection sinks; the design explicitly
  accepted "reports what Steam reports" (§6).

## Verdict

approve — zero blocker/major findings; one minor hardening finding (S1) recorded above.
