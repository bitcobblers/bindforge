# correctness review r1 (dissent pass c1)

verdict: approve

Scope: `git diff d5bafff8 b12a4dd7` over 02-proposal.md, then the full current
document. Inputs: the proposal, the task text, the Revision log, my r0 findings.

## Round-0 findings — closure check

- [closed] **A1 dead citation (major).** The doubled-backslash spelling is split
  out as `A1b` marked `unverified` with exactly the prescribed note (dead
  citation; no spelling assumed; A2's verified `\\`/`\"` unescape, re-checked
  live at keyvalues.rs:186-207, makes either form arrive usable). Playmoir is
  gone from the document (grep: no matches); A19 keeps the lutris basis, which I
  re-fetched live — `read_library_folders` reads `config/libraryfolders.vdf`,
  supporting the second-location claim. A1's remaining scope (same VDF shape,
  same ACF field set) is unchanged and stands as reviewed in r0.
- [closed] **WOW64 view ownership (major).** `RegistryKey` is now
  `CurrentUserValveSteam | LocalMachineValveSteam32 | LocalMachineValveSteam64`;
  §3 makes `registry_string_value` read exactly one (key, view) pair per call
  ("Which HKLM view to ask first is `host_candidates`' policy, not the
  reader's"); §5 orders the two HKLM variants inside the pure, Linux-tested
  `host_candidates`; Test strategy #3's "HKLM-32-bit-view only;
  HKLM-64-bit-view only" cases are writable on the `HashMap<(RegistryKey,
  &str), String>` fake; #8 is re-pinned to three variants. The enum order, §5's
  list, §4's "three-variant enum" and the Compatibility-matrix row ("32-bit view
  first, the 64-bit view second") all agree. The Microsoft
  alternate-registry-view doc, re-fetched live, supports §3's flags and the
  rejection of direct `Wow6432Node` use.
- [closed] **`--steam-root` vs Done-when :65-67 (major).** §8 now implements the
  r0 fix option I named: `#[cfg(windows)]` feeds the flag to
  `SteamWindowsPlugin::from_roots`, `#[cfg(not(windows))]` keeps T-001's Linux
  branch (byte-for-byte today's main.rs:33-38 shape, so the three CLI tests are
  untouched and #9 holds); the Windows branch is inside #12's
  `cargo check --workspace --all-targets` leg. The Deferred item contradicting
  the Done-when clause is deleted, and the reading of :65-67 now lives in the
  Revision log (r1 entry) and the rewritten Options paragraph.
- [closed] **`absolute_env` sharing (minor).** §1's moved-items list names it;
  verified against code: it is the private `fn` at steam_linux.rs:338-342 and
  Linux's `host_candidates` uses it at :111 and :128, so "the same
  `absolute_env` helper Linux uses, shared per §1" is accurate.
- [open → re-raised, minor] env-gate absoluteness coverage — see Finding 1.
- [closed] **`name()` literal nit.** §1 sketch now reads "name() is
  self.core.name()", matching §2's no-drift claim.

## Findings

- [minor][re-raised] §5 Root candidates — "A relative value is ignored by the
  same `absolute_env` helper Linux uses, shared per §1" + Test strategy #3 —
  the revision makes the sharing explicit but not the coverage gap: the gate's
  predicate is `PathBuf::is_absolute` (steam_linux.rs:341), host-dependent as §6
  itself states, and `host_candidates` is compiled and unit-tested on Linux, so
  the fake can only pass POSIX-absolute values and the Windows branch of this
  rule — a `C:\Program Files (x86)`-shaped value being kept, a Windows-relative
  one dropped — is exercised on no host. §5 still does not state what the fake
  supplies. Runtime behavior is correct (each host gets native semantics), so
  this remains a verification gap, not a functional defect. Fix: in §5, one
  sentence stating the #3 fake supplies POSIX-absolute values and that the
  Windows-absolute case is gate-#12/#13 territory; or generalize the
  absoluteness predicate over `SteamTarget` next to
  `install_dir_rejects_backslash` (§1's existing mechanism) and add one #3 case.
- [minor][new] Test strategy #2 — "`foo\bar`, the doubled form `foo\\bar` as a
  real Windows ACF writes it (A2)" — after the r1 split, the doubled-spelling
  claim is exactly what A1b marks unverified ("the design assumes no spelling"),
  so this body sentence now states as fact what the register declines to assume,
  and misattributes it: A2 is the reader's verified unescape behavior, not a
  source on what ACFs write. The test case itself stays valid either way, which
  is why this is language-level, not behavioral. Fix: "the doubled form a
  Windows writer may emit (A1b, spelling unverified)".

## FYI (no severity)

- A1 is now supported by a single basis, the StackOverflow question that again
  returns 403 to a non-browser fetcher (inconclusive, as in r0 — not evidence of
  death). One more live basis (a real Windows `libraryfolders.vdf` sample with
  numeric-keyed blocks) would firm the register's only linchpin row.
- Compatibility matrix: the probe branch registers `SteamLinuxPlugin::from_host()`
  on a Windows host too; its HOME/env probe can only yield roots that survive the
  canonicalize-and-require-`steamapps` rule, i.e. nothing in practice — the
  matrix has the Linux mirror row ("steam-windows registers with zero roots")
  but no Windows-side counterpart. Another seat may want the symmetry.
