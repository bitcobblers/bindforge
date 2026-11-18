# security review r3

verdict: approve

Scope: whole document (full-panel pass c0, round 3). Inputs: `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the Revision log. Read-only source
checks confirmed the design's code-facing claims: `engine.rs:20-33` swallows a
plugin `Err` into one `eprintln!` and returns `Ok` (A14 ✓); `Cargo.lock` has
exactly 32 `[[package]]` entries and `crates/discovery/Cargo.toml` carries only
`serde` (A15 ✓); `keyvalues.rs`/`steam_linux.rs` do not exist and `lib.rs` holds
the five listed modules (additive posture ✓); the `List` arm's raw `println!`
of `display_name`/`install_path` at `apps/discovery-cli/src/main.rs:48-57` is
the untouched printing code §8 references.

## Findings
- [minor][new] Chosen design §4 KeyValues reader — "Files larger than 1 MiB are skipped as malformed (A16): the reader's work is bounded per file" — the cap's enforcement point is unspecified: an implementation that reads the file and then measures it (or parses and lets the parser's bounds trip) allocates the whole file before the cap fires, so the stated "bounded per file" property does not hold for a huge or sparse file sitting in a library's `steamapps/`. Fix: one clause stating the size check happens before the contents are loaded — `fs::metadata(path)?.len()` compared against the 1 MiB constant (or a read capped at 1 MiB plus BOM), with the over-cap skip counted in §7's skipped-file tally — replacing the bare "Files larger than 1 MiB are skipped" sentence with that mechanism named.

## Checklist coverage
1. Trust boundaries — pass. The only data sources are local, user-owned Steam files (`libraryfolders.vdf`, `appmanifest_*.acf`) and caller-supplied `--steam-root` paths; no shared binding file or network input. The design defends them as if hostile anyway (canonicalize-and-gate, `installdir` rejection, fail-closed), which is the stronger posture.
2. Parsing untrusted data — one gap, the finding above. Depth cap (16), reject on `#base`/`[`/unbalanced/unterminated, no code evaluation, no compressed input.
3. Path handling — pass. Roots and libraries canonicalized (existence required, missing dropped), `installdir` rejected on `.`/`..`/absolute/`/`; a relative VDF `path` may contain `..` (the fixture's `../secondary`) but originates in the user's own Steam config the user already reads, is canonicalized, and must pass the `steamapps` gate. No path is interpolated into a shell string; all I/O is `std::fs` + clap `Vec<PathBuf>`.
4. Secrets and PII — pass. §7's single error message is explicitly bounded ("names the selected roots and the count ... nothing else — no file contents") and stays on stderr, out of the `--json` stdout stream. Nothing leaves the machine.
5. Dependencies — not applicable: zero new dependencies, verified 32-package lock unchanged is a stated gate (Rollout #3).
6. Privileges — pass. Read-only filesystem access under `$HOME` plus user-configured library mounts; no device access, no elevation, `home_dir() == None` fails closed to `Ok(vec![])`.
7. Fail closed — pass. Per-file skip, non-UTF-8 skip, no partial rows (a game is emitted whole or not at all), one narrow `Err` distinguishes "Steam present, nothing scannable" from "nothing installed" despite A14's swallowed error.

## FYI (no severity)
- `apps/discovery-cli/src/main.rs:48-57` prints `display_name` and `install_path` verbatim into the table, so a game name carrying ANSI escapes or newlines would render raw; the printing code is pre-existing and §8 declares it untouched, and the input files are the user's own — worth a seat elsewhere only if a future change pipes this table into another consumer.
- Duplicate-key "first match wins" (§4) is a determinism/correctness choice, not a security one; flagging only for the correctness seat's awareness.
