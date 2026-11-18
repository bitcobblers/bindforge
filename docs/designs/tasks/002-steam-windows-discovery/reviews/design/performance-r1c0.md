# performance review r1

verdict: approve

Seat: performance. Scope: whole document (`docs/designs/tasks/002-steam-windows-discovery/02-proposal.md`).
Inputs used: the proposal, `docs/tasks/T-002-steam-windows-discovery.md`, the
proposal's Revision log, and read-only inspection of the code the proposal
describes moving (`crates/discovery/src/steam_linux.rs`,
`crates/discovery/src/keyvalues.rs`, `crates/discovery/src/engine.rs`,
`apps/discovery-cli/src/main.rs`, `crates/discovery/Cargo.toml`, `Cargo.lock`).
No earlier review report or transcript was opened. The Revision log was read
first; every r0 entry there is a decision about *which* option was chosen, and
none of them addresses the cost model below, so the single finding is not a
re-report of a rejected decision.

## Findings

- [minor][new] §5 Root candidates — "The env defaults are appended even when a
  registry candidate was found: no candidate needs a separate existence gate
  (§2's canonical-path rule drops what is not there), and `canonical_paths`
  collapses the common case where the registry points at the default folder, so
  a double-listed root costs nothing." — "costs nothing" is established only for
  the one case the sentence names. Verified for that case: `canonical_paths`
  (crates/discovery/src/steam_linux.rs:322-334) canonicalizes and inserts into a
  `BTreeSet`, so a registry `SteamPath` and `%ProgramFiles(x86)%\Steam` that
  resolve to the same folder really do collapse to one root before any scanning.
  When they do not resolve to the same folder — the ordinary case of Steam on
  another drive with a leftover `%ProgramFiles(x86)%\Steam` that still has a
  `steamapps/` directory — both candidates survive the existence rule and each
  is scanned in full: one `libraryfolders.vdf` read capped at
  `keyvalues::MAX_DOCUMENT_BYTES` = 1 MiB (crates/discovery/src/keyvalues.rs:21,
  applied pre-allocation at steam_linux.rs:290-298), then per declared library a
  `read_dir` plus one metadata+read per `appmanifest_*.acf`
  (steam_linux.rs:200-231). Library-level de-dup over the union of all roots'
  libraries (steam_linux.rs:64-70) keeps the overlap from doubling the
  manifest work, so the real bound is small — but the design asserts the cost
  away instead of stating it, and §5 is the section a later candidate-list
  change (the *Deferred* registry-key and `BaseInstallFolder_N` items) will be
  edited against. Fix: replace "so a double-listed root costs nothing" with the
  bound in the same sentence — "…so a double-listed root collapses to one scan;
  when the registry and the env default are *different* roots, each extra root
  costs at most one 1 MiB-capped `libraryfolders.vdf` read plus, for the
  libraries only it declares, one `read_dir` and one capped read per manifest —
  and `discover()` de-dups the libraries roots share
  (crates/discovery/src/steam_linux.rs:64-70), so no library is scanned twice."
  This replaces an unbounded claim with the existing behavior's cost model; it
  adds no mechanism and changes no candidate ordering.

## Checklist coverage

1. **Input-path latency budget** — not applicable, and verified as such:
   `DiscoveryPlugin::discover(&self)` is reached only from
   `DiscoveryEngine::discover_all` (crates/discovery/src/engine.rs:20-33), called
   once per `discovery list` (apps/discovery-cli/src/main.rs:40). There is no
   keystroke or event path in this design, and the source task states no latency
   budget. No per-event allocation or table rebuild is introduced.
2. **O(n²) over unbounded n** — no quadratic term introduced. The Windows
   candidate list is a constant (2 registry values + 2 env defaults, §5); roots
   are de-duplicated before scanning; each library is scanned once
   (steam_linux.rs:68); the only superlinear steps are `BTreeSet` inserts and the
   `(slug, install_path)` sort, both O(n log n) over the game count. The
   platforms axis is the `SteamTarget` parameter, which costs nothing at runtime.
   The finding above is about the *constant*, not the order.
3. **Memory growth** — bounded. No table or cache is introduced; the only
   buffers are the per-file read (1 MiB cap checked against `metadata` before the
   bytes are allocated, steam_linux.rs:287-298 — shared verbatim per §2, so
   Windows inherits the cap) and the emitted `Vec<DiscoveredGame>`, which lives
   for one CLI call. Registry values arrive as one small `RegValue` per read.
4. **Startup cost** — verified low and, on Linux, unchanged: the §3
   `#[cfg(not(windows))] from_host()` returns `from_roots(Vec::new())`, and an
   empty root set short-circuits the scan, the error clause and the sort
   (steam_linux.rs:61-91), so the §8 step that registers
   `SteamWindowsPlugin::from_host()` on a Linux host costs zero filesystem calls.
   Windows startup is ≤3 registry opens + ≤4 canonicalize/stat pairs. Build
   footprint claim verified: `[target.'cfg(windows)'.dependencies]` keeps `winreg`
   out of the Linux compile, `crates/discovery` has `serde` alone today
   (crates/discovery/Cargo.toml:6-7), and `Cargo.lock` already carries
   `windows-sys 0.61.2` and `windows-link 0.2.1` with no `cfg-if` entry, matching
   A10 — so the lock diff is one new crate, not a second `windows-sys` tree.
   The §5 cost sentence is the finding above.
5. **Caching / invalidation** — not applicable: the design adds no cache, and
   `host_candidates` is called once per process. The *Deferred* caching item
   matches the task's scope and does not contradict it.
6. **Research baseline** — recorded as missing evidence, non-blocking: the
   proposal quotes no expected scan time or file count, and I was not given
   `01-research.md` on this pass, so no gap between the design's numbers and the
   baseline can be established. Because neither the proposal nor
   `docs/tasks/T-002-steam-windows-discovery.md` claims any performance target,
   no required property is left unestablished; I did not make this a finding.

## FYI (no severity)

- Pre-existing, not introduced here (so no severity against this design):
  `libraries_in_root_with_skips` runs `canonical_paths` per root
  (steam_linux.rs:162) and `discover()` runs it again over the union
  (steam_linux.rs:68), so every library path is canonicalized and `steamapps`-
  stat'd twice per call, and `scan_library` stats each manifest twice
  (`path.is_file()` at :218, then `metadata` in `read_text_file` at :291). The
  §2 move-verbatim rule is the reason to leave it, but the shared core is the
  natural place to fold the duplicate passes if a later task touches this code.
- For the correctness seat: with both plugins registered (§8),
  `DiscoveryEngine::discover_all` prints `Error from plugin <name>` to stderr per
  failing plugin (engine.rs:25-29) while CLI exit status stays 0, so a broken
  Windows-side root set can now print where a Linux-only run printed nothing —
  §8's "the three existing CLI tests pass untouched" covers Linux, where the
  Windows plugin has zero roots.
- For the compatibility / correctness seats: T-002's Done-when says
  "`--steam-root` keeps working for it"
  (docs/tasks/T-002-steam-windows-discovery.md:65-67); §8 keeps `--steam-root` on
  `SteamLinuxPlugin` and hands an explicit Windows-root CLI seam to *Deferred*.
  Whether that satisfies the clause is not a performance call, so I raise no
  severity; I flag it only because a Deferred item touching a Done-when clause is
  normally closed by the panel.
- Not a finding because the task mandates it (docs/tasks/T-002-steam-windows-discovery.md:68-71):
  §9's `rustup target add` plus a second full `cargo check --all-targets` roughly
  doubles the type-check work per PR and pulls a `rust-std` per run; worth a note
  in CI expectations, not a change.
