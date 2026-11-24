# performance review r1 — dissent pass c1

Scope: `git diff 37bd2f3 a4dda6a` of `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/adrs/017-keybinding-extraction-codecs.md`,
the Revision log, and this seat's round-1 findings.

verdict: request-changes

## Closed by this revision (no re-report)

- R1-finding 1 (walk bounded by a result-set cap): `MAX_LOCATE_ENTRIES_PER_TREE
  = 16_384` is named in the budget table, the over-cap entry is "skipped and
  counted", and Test strategy 6 pins the stop. The walk leg is bounded. Its
  residual in the content-probe leg is finding 2 below.
- R1-finding 2 (`017-B2` counted grouped actions, not pairs): the budget-table
  row, §`bindings/v1` and Test strategy 4 all count (`action`, input-token)
  pairs, and `MAX_SNAPSHOT_OCTETS` plus the loader's pre-read length check bound
  the document. The unit choice is fixed; the unnamed check site is finding 4.
- R1-finding 3 (aggregate product omitted a tree and the per-record multiplier):
  the sentence now reads `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES`
  (544 files, 544 MiB — arithmetic confirmed) and adds "that worst case repeats
  per matching record the face walks". The `+1` is right now that
  `NativeHost` does not ship.

## Findings

- [major][new] §The first family — "examining at most `MAX_LOCATE_ENTRIES_PER_TREE` entries per tree and never descending a directory whose remaining depth is under the grammar's four trailing components" — the prune compares the remaining depth budget against the constant 4 without the case where the directory being descended *is itself* one of the grammar's named components, in which case the components still needed below it are 3, 2 or 1 rather than 4. With `MAX_LOCATE_DEPTH = 8` the rule stops descending at relative depth 5, so a candidate at relative depth 6–8 whose `Saved` component sits at relative depth 2–5 is never enumerated. The design's flagship Proton layout is exactly that shape: the `ProtonPrefix` root `<ctx.library>/steamapps/compatdata/<ctx.appid>/pfx/drive_c/users/steamuser` holds a packaged UE title's config at `AppData/Local/<Project>/Saved/Config/<Platform>/<Game>.ini` — relative depth 7 — and the walk must descend `Config` at relative depth 5, where `8 − 5 = 3 < 4` prunes it. That title answers no-claim / `family-confirmed-files-absent` for the one layout the row exists to serve, and Test strategy 6's pin "the `Saved/Config/<dir>/` grammar is found at depth ≤ 8 and not at depth 9" is false for it. Fix: delete the clause — `MAX_LOCATE_DEPTH` plus the newly added `MAX_LOCATE_ENTRIES_PER_TREE` already bound the walk, which was the whole point of round 1's finding 1 — or restate it name-anchored: a directory at relative depth `d` need not be descended only when neither its own name nor any name within the three levels above it is one of the grammar's components (`Saved`, `Config`, the platform directory), so the surviving prune costs no locatable file.

- [major][re-raised] §The first family / §Read discipline and budgets — "A candidate is an `*.ini` whose `Path::components()` end in `Saved/Config/<one directory>/<name>.ini` … holding a `[/Script/Engine.PlayerInput]` section with a `Bindings=` line, capped at `MAX_CANDIDATE_FILES` per tree and opened through `read::open_capped` and `read::decode`" — the content test is part of *candidacy*, so every path-grammar match has to be opened and decoded before it can be counted; the 32-file cap therefore bounds the retained set and not the opens, and after this revision the only thing bounding opens is `MAX_LOCATE_ENTRIES_PER_TREE = 16_384`. Worst case is then 16,384 opens of up to `MAX_BINDING_FILE_OCTETS` each per tree (≈16 GiB) and up to 17 trees per game, against the figure the same section instructs `schemas/README.md` to copy: "one extraction opens at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files … (544 MiB)". That understates the shipped read ceiling by up to ~512× per tree — the round-1 result-not-work shape moved from the walk leg to the content-probe leg — and nothing refuses: `detect` returns `Option<Detection>`, so the cost `017-L1` calls untrusted input is paid silently. Fix: gate the opens with the constant that already exists — "at most `MAX_CANDIDATE_FILES` path-grammar matches per tree are opened and content-tested, in the walk's byte order; a further match is skipped and counted the way an entry past `MAX_LOCATE_ENTRIES_PER_TREE` is" — which leaves the located set, the byte order and §Determinism untouched and makes the aggregate product the actual open ceiling. ADR-017 Rollout 5 already asks for this ("budgets enforced at read and locate time").

- [minor][new] §Config-path resolvers / §Read discipline and budgets — "`SteamUserdata`: `<root>/userdata/<dir>/<ctx.appid>` for every `root` in `ctx.roots`, one per digit-named `<dir>`, byte order, at most `MAX_USER_DIRS_PER_GAME`" — this revision moved `userdata/` off a single anchor onto a `Vec` that no constant bounds (`crates/discovery/src/steam_linux.rs:92-128 hands every candidate to `SteamCore::with_roots`, so a machine with a native plus a Flatpak plus a Snap install resolves around ten roots), and the sentence does not say whether `MAX_USER_DIRS_PER_GAME` caps the union over all roots or the directories found under each root. Test strategy 3's "byte-order union over `ctx.roots`" and "a 17th digit-named directory is past `MAX_USER_DIRS_PER_GAME`" imply the union, which is the only reading under which the aggregate's 17-tree term holds; the per-root reading makes the ceiling `schemas/README.md` copies wrong by the roots factor. Fix: make the scope explicit in the same clause — "at most `MAX_USER_DIRS_PER_GAME` `SteamUserdata` trees in total across all `ctx.roots`" — and mirror it in the constant's Bounds cell, so the shipped number is derivable from the budget table alone.

- [minor][re-raised] §Read discipline and budgets / §`bindings/v1` — "`SnapshotEntryBudget` on the pair total" and "the (`action`, input-token) pair total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries` and every `inputs` array" — the count is now the right one but no check site is named, and `extract` folds every located file of a tree (`files` ≤ `MAX_CANDIDATE_FILES`, each ≤ `MAX_BINDING_FILE_OCTETS`) into a working set before it builds the document the bound is stated on. Checked at document construction, the bound prices the result while the transient peak stays bounded by bytes read: up to ~32 MiB of `+Bindings=` lines per tree folded into strings before `SnapshotEntryBudget` fires, across the 17 trees `extract` returns as one `Vec`. Round 1's fix asked for "the same `SnapshotEntryBudget` check site evaluated on the pair total"; the pair total arrived, the site did not. Fix: one clause in §Read discipline — the check runs as each pair enters the tree's working set, so the fold stops at `MAX_SNAPSHOT_ENTRIES + 1` pairs and names itself — after which Test strategy 4's two breach shapes read as one mechanism. No new constant, no new refusal name.

## FYI (no severity)

- §Read discipline and Test strategy 4: `UserCandidates::push` "refuses a 33rd file" while `detect` returns `Option<Detection>`, which carries no refusal channel; the 33rd-file outcome needs a named home (correctness seat).
- §Config-path resolvers: with `userdata/` enumerated under every resolved root, one SteamID can yield two trees and so two `MAX_CANDIDATE_FILES` budgets, while `017-B3`'s wording is "the cap on one game's located candidate set per local SteamID" (compatibility seat).
- §Determinism: the dedup key (`user`, `provenance.files`) leaves two same-`user` documents from two roots both standing, with no stated tie-break between them (correctness/compatibility seats).

## Checklist coverage

Items 1 (per-event cost) and 5 (cache invalidation): not applicable — the face is
offline, one game at a time, print-only, and caches nothing. Item 3 (memory
growth) and item 4 (file reads at startup) are findings 2 and 4. Item 2
(O(n²) over unbounded n): none found — every collection cited is now budget-
named except the opens in finding 2 and the roots vector in finding 3. Item 6
(research baseline): not evaluated — `01-research.md` is not an input to this
pass.
