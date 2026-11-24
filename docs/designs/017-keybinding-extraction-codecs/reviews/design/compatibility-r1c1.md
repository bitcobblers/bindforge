# compatibility review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 37bd2f37… a4dda6a8…` of `02-proposal.md`, then the full current document,
`docs/adrs/017-keybinding-extraction-codecs.md`, and the Revision log. Read-only verification used:
`.github/workflows/check.yml:25-126`, `crates/discovery/src/steam_common.rs:62-131,363-381`,
`crates/discovery/src/steam_linux.rs:60-128`, `crates/discovery/src/steam_windows.rs:85-120`,
`crates/discovery/src/game.rs:15-19`, `apps/discovery-cli/src/main.rs:72-83,129-142`.

All ten of this seat's round-1 findings are closed by the revised text, and none of the closures
undoes an earlier approval: `NativeHost` is gone so the Flatpak/Snap row's narrowing is true
(`host_candidates` puts the sandbox roots at `<home>/.var/app/com.valvesoftware.Steam/.local/share/Steam`
and `<home>/snap/steam/common/.local/share/Steam`, which do hold `userdata/` and `steamapps/`, so
"both templates hang off a resolved root" is sound); macOS and Deck now commit
(`game.rs:15-19` really is two variants, and the CLI builds only the two Steam plugins,
`main.rs:130-132`); the `Cargo.lock` claim is restated narrowly enough that
`cargo test --workspace --locked` is satisfiable; the status codes moved to 3/4/5 above clap's 2
(`Cli::parse()` at `main.rs:73` — the face does not own 2, and the section now says so); the symlink
fixtures name their `#[cfg(unix)]` mechanism and Test #17 covers the compile-only arm on the
`x86_64-pc-windows-msvc` leg; `user` rides the token grammar; the over-bound verbatim token is the
existing `UnparsableEntry`; `MAX_USER_DIRS_PER_GAME` is declared silent; the Windows row's failure
model is corrected; the grammar match is component-wise.

Checklist 1 (no `unknown`, no empty cell): clean. Checklist 2 (OS input APIs): not applicable — this
run reads files and touches no input API. Checklist 3 (Node floor): `tools/orchestration` unaffected.
Checklist 4 (native modules): none. Checklist 5 (keyboard reality): unchanged and adequate — tokens
are verbatim and every bound is octet-based, so a non-QWERTY or non-ASCII spelling is representable.
Checklist 6 (paths, per-OS config conventions): the component-wise match plus "no resolver reads the
process environment" is an improvement; finding 1 is the residue. Checklist 7 (a platform test named
for each supported platform): finding 2.

## Findings

- [minor][new] §Discovery plumbing + §GameContext — "`roots()` (the selected root candidates, in
  candidate order)" and "`pub roots: Vec<PathBuf>,  // every root the scan resolved, candidate
  order`" — the order is false of the field the wrapper would expose. `SteamCore` holds its roots
  through `canonical_paths` (`crates/discovery/src/steam_common.rs:369-381`), which canonicalizes
  (resolving symlink aliases such as `~/.steam/root`), drops anything that fails to canonicalize or
  lacks a `steamapps/` directory, and returns a byte-sorted, de-duplicated set — not the candidate
  list. Test #3 already reads "the byte-order union over `ctx.roots`", so the two statements in the
  same design disagree about the order an implementer must produce. Fix: in both places state "the
  canonicalized, de-duplicated roots that actually hold a `steamapps/`, in byte order (the set
  `SteamCore` retains, `steam_common.rs:369-381`)", drop "candidate order", and keep the
  existence-filter half as the reason "a tree is used only if it exists" is already true.

- [minor][new] Compatibility matrix — "Flatpak / Snap Steam | Yes, for the root-relative trees" and
  "Steam Deck | Yes, by the Linux row" — both upgrades rest on the new `roots()` seam, and the Test
  strategy names nothing that reaches it: #1 pins `ScanRecord.root`/`library`, #2 pins that
  `from_record` carries the slice it was handed, #3 pins the union over `ctx.roots`. No test or
  fixture is shaped like a Flatpak/Snap or Deck root, so the two rows this round upgraded from
  `Unknown`/overclaimed are covered only by a structural test of the generic mechanism. Fix: add one
  clause to #1 or #2 — `roots()` returns exactly what `SteamCore::with_roots` kept — and one
  `ctx.roots` entry in #3 shaped `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` that yields
  a `SteamUserdata` tree. No discovery-side work is implied: the candidate list itself is already
  pinned at `crates/discovery/tests/steam_linux_tests.rs:294-345`.

- [minor][new] §Read discipline and budgets — "`MAX_USER_DIRS_PER_GAME = 16` | `userdata/`
  directories considered per game" against §Config-path resolvers' "`<root>/userdata/<dir>/<ctx.appid>`
  for every `root` in `ctx.roots`, one per digit-named `<dir>`, byte order, at most
  `MAX_USER_DIRS_PER_GAME`" — turning the template into a union over roots leaves the cap's unit
  undecided (sixteen per game in total, or per root), and the aggregate sentence counts one set of
  sixteen ("`(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` … (544 MiB)"). Under the per-root
  reading, A3's nine Linux candidates — plus any number of repeated `--steam-root` values — put the
  worst case at `(|roots| × 16 + 1) × 32` files, several multiples of the figure
  `schemas/README.md`'s budget bullet is told to state. Fix: one clause deciding it (the cap counts
  directories across all `ctx.roots` for one game, so the arithmetic stands) and which directories
  are dropped when the union exceeds it. (Shared with the performance seat.)

- [minor][new] §Determinism + §The CLI face step 3 — "across documents, deduplicated by (`user`,
  `provenance.files`) — two records that resolve the same tree state it once" — the key is sound for
  one game in two libraries, but not for the roots union this round introduced: two distinct resolved
  roots can each carry `userdata/<same SteamID>/<appid>/` (a native install plus a Flatpak or Snap
  Steam on one account), and because `files` are deliberately tree-relative ("the portability rule:
  the same layout states the same document on any machine"), those two trees share the key and one
  document is dropped in silence — no refusal, no status, and no path printed anywhere that says
  which install answered. Fix: deduplicate at the `UserTree` level on the tree's absolute root path
  — which is exactly what the two-records case repeats — and let §The CLI face emit one document per
  distinct tree; same one-fact-stated-once property, no document identity built out of values the
  contract strips machine context from.

- [minor][new] §Rollout item 9 — "`cargo deny check advisories`, and `npm ci && npm run check` in
  `tools/orchestration`, per `.github/workflows/check.yml`" — that leg is not one of the run's cargo
  gates: the workflow runs it as the separate ADR-027 `advisory-gate` job, which builds its own
  `cargo-deny` 0.20.2 from source on toolchain 1.88.0 inside a 30-minute budget precisely because it
  is "too heavy for the 15-minute `rust` job" (`check.yml:83-126`). A22's own enumeration of CI ("the
  cargo trio plus a Windows *compile* check … and the Node check") does not count it, and this run
  adds no external package for it to screen. Listing it inside the run's gate set, ungated by any of
  that, imports a tool install no other part of the design owns. Fix: cite it as the `advisory-gate`
  job with its pinned scanner and toolchain and note nothing here adds a package, or keep A22's list;
  the same line could also name the `cargo check -p bindforge-registry --all-targets` step the `rust`
  job actually runs.

- [minor][new] §Rollout item 8 (Ledger) — "the units whose shipped reading differs from their
  wording (`017-C1`'s bound and `extract` return shape, `017-C3`'s `inputs`, `017-C4`'s 'same scan',
  `017-S1`'s launcher contexts)" — `017-C2` is missing, although the Revision log already records a
  divergence from it ("§GameContext — `roots` is `017-C2`'s whole resolved set … not a 'main' one")
  and the ADR clause reads "every steam root the scan resolved — main root and library roots
  separately, because `userdata/` lives only under the main root". What ships is one undifferentiated
  `roots` vector with none labelled main, which is a shipped reading that differs from the wording;
  the hand-off is the only mechanism that keeps `docs/adrs/` (operator-owned) accurate. Fix: add
  `017-C2`'s main/library separation to the list and to the "Five clauses" count in §Context and the
  matching Risks row.

- [nit][new] §Goals — "`unreal-ini` ships with fixture trees for the two launcher contexts whose
  roots a record can justify" — ADR-017's Terminology fixes a launcher context as "the native host
  layout, or the Proton prefix the game was launched under", so after this round's removal of the host
  template the two shipped trees (`steam-userdata`, `proton-prefix`) are not the ADR's two contexts,
  and the phrase hides that the native context is exactly what moved to §Deferred. Fix: "the two
  root-relative trees a record can justify", with the context named where §Deferred already names it.

- [nit][new] Compatibility matrix, Steam Deck row — "A Linux Steam install with the `$HOME` and
  Proton prefix layout the two rows above claim" — nothing hangs off `$HOME` any more (§Config-path
  resolvers: "each built from a `GameContext` anchor and nothing else — no resolver reads the process
  environment"); the resolved Steam roots merely live under it. Fix: "the root-relative `userdata/`
  and Proton prefix layout the Linux and Proton rows claim".

## FYI (no severity)

- §Read discipline — "`MAX_SNAPSHOT_ENTRIES = 4096` | one snapshot's distinct (`action`,
  input-token) pairs — which bounds its `entries` and every entry's `inputs` at once": because
  `inputs` may be empty (§`bindings/v1` — "may be empty when the file's own merge operators removed
  every input for an action"), the pair total does not bound `entries`; only `MAX_SNAPSHOT_OCTETS`
  does. If `017-B2`'s "cap on one snapshot's entries" is meant literally, state `entries` ≤
  `MAX_SNAPSHOT_ENTRIES` beside the pair total. (correctness/performance seat)
- §The CLI face step 1 — "keep `roots()` beside the records": `list` builds two plugins per host
  (`main.rs:130-132`, and `SteamWindowsPlugin::from_host()` yields no roots on a non-Windows host,
  `steam_windows.rs:96-99`), so one clause that each record's context carries the roots of the plugin
  that produced it would settle the flat `&[PathBuf]` argument of `from_record`. (correctness seat)
- No input API, native module or Node surface changed this round, so checklist items 2, 3 and 4 are
  not applicable to the diff.
