# Code review — correctness seat, round 1 pass c2 (ratification)

- **Seat:** correctness (faithful to the design)
- **Pass:** c2 ratification — later revisions are checked only for whether they
  broke what this seat approved at round 1, plus whatever the changed code makes
  newly true.
- **Scope as given:** `git diff a8fd09ec6d6b8210574447f9b68e7cc199f73153..HEAD --
  ':(exclude)docs/designs'` (15 files, +504/−44).
- **Inputs used:** that diff; `docs/designs/017-keybinding-extraction-codecs/02-proposal.md`
  (full current document); `docs/adrs/017-keybinding-extraction-codecs.md`; the
  runner's gate record; this seat's round-1 findings as quoted by the caller; and
  the source/test files the diff touches, read for context. No review report or
  transcript was opened. `03-build-notes.md` is outside this seat's inputs, so no
  deviation was scored as undeclared on declaration grounds; severities below
  reflect behavior, per the round-1 calibration.

## Verdict

**approve.** Zero blocker, zero major. Six findings are new to this revision
(1 … 6); the seven findings this seat raised at round 1 are unchanged — none was
made worse, none was broken further (7 … 13). Nothing the caller asked about is
regressed: no round-1 blocker/major existed for this seat, so checklist item 7
has nothing to re-verify as mandatory.

## What the revision did, from this seat's angle

The landed fixes answer other seats: the locate open-cap is counted on opens
(perf-1), `Fold::remove` short-circuits on absent pairs (perf-3), pass 2 skips
families pass 1 already probed and the CLI memoizes a claim per anchor triple
(perf-5), `decode_owned` removes a full-size copy (perf-7), refusal echoes go
through `escape_and_truncate` (security-2), `provenance.files` gains a closed
spelling bound plus an invalid fixture (security-1), the walk queues
`DirEntry::path()` instead of a lossy-name join (compat-A), `tree_relative`
emits a `/`-joined contract spelling, and the README/spec aggregate moved to two
passes. All of those are in-scope responses to accepted findings, not scope
creep (item 4), and each is pinned by a test whose comment names the finding it
answers. Two of them, however, changed behavior this seat had verified against
the design: the reader-refusal arm of `locate` (finding 1) and the loader's new
`files` spelling bound (finding 2).

## Checklist

1. **Components and interfaces exist and match names/signatures** — pass. No
   component the proposal names was removed or renamed. `read::decode_owned` is a
   new public sibling of `read::decode`, additive and pinned in
   `crates/codecs/tests/zz_api_probe.rs:79`; `decode` now delegates to it with
   identical semantics (`crates/codecs/src/read.rs:61-81`). The `CodecFamily`,
   `Detection`, `UserCandidates`, `GameContext`, `TreeKind`/`UserTree` shapes are
   untouched by the diff.
2. **Assumption register honored** — pass. Nothing in the diff touches the
   assumption-backed choices (A12 dependency posture — no `Cargo.toml` change;
   A15 no platform constant; A17 `steamuser` template constant; A19 BOM
   handling; A21 empty `appids()`; A22 compile-only non-Linux legs). The new
   byte-exact `grammar_match` note restates the shipped behavior rather than
   changing it.
3. **Deviations from the proposal declared** — cannot be checked here
   (`03-build-notes.md` is not a permitted input). Two *live* divergences from
   proposal text exist after this revision (findings 1 and 2) and one text-vs-text
   inconsistency the revision created (finding 6); each needs a build-notes or
   Revision-log line if the implemented rule is the intended one.
4. **Scope creep** — pass, with one boundary case. The claim memo
   (`apps/discovery-cli/src/bindings.rs:227-237`) is caching, not new behavior,
   for the shipped family; its soundness hole is reported as finding 4 rather
   than as creep. No behavior in the diff serves a goal the proposal never named.
5. **`03-build-notes.md` claims true** — not applicable to this seat (file not in
   inputs). The notes are cited by the new tests' comments ("Test-update item …
   in `03-build-notes.md`"); whether those items exist is for the runner's audit.
6. **Error paths from the design exist and fail closed** — pass except finding 1:
   every named `ExtractionRefusal` still exists, still prints by name, and now
   escapes its echoes; but one previously-loud path (an unreadable grammar match
   at locate time) became a silent skip.
7. **Prior round's blocker/major findings addressed** — this seat filed none, so
   there is no mandatory fix to confirm. The six minors and one nit it filed are
   all still open and unchanged (findings 7 – 13 below).

## Gate evidence

The runner's record (green at 2026-11-24T07:31:52Z) is cited for green legs:
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo test --workspace --locked` and `npm run check` all exit 0. The
single `test result:` line the renderer captured reads `ok. 0 passed; 0 failed`,
which is a leg-summary artifact, not evidence about the suites; to avoid resting
this pass on it I re-ran one read-only leg myself:
`cargo test -p bindforge-codecs --locked` → `test result: ok. 27 passed; 0 failed`
for `tests/unreal_ini.rs`, including the two pins this revision added that cover
the same code as findings 1 and 3 (`a_reader_refused_match_is_located_and_refused_by_name`,
`the_open_cap_bites_on_matches_that_never_locate`) — i.e. the findings below are
gaps the green suite does not reach, not failing tests.

## New findings (this revision)

1. **minor — an unreadable grammar match is now silently skipped, where the
   design says it is located and refused by name.**
   `crates/codecs/src/families/unreal_ini.rs:257-266` (the new
   `Err(FileOverBudget | UndecodableText) => locate` / `Err(_) => {}` split, with
   the rule stated at `:175-180`). §The first family (proposal:507-509) says
   "*A match the reader refuses* cannot pass the content test either, so it is
   located on path grammar alone and `extract` refuses it by name", and 017-L1
   (ADR:117-123) gives the reason: a named refusal, "never an empty snapshot,
   *because a claimed game with an unreadable binding file is a different fact
   from a game no codec claimed*". `read::open_capped` returns
   `UnreadableFile` for a `metadata` or `open` failure and for a mid-read I/O
   error (`crates/codecs/src/read.rs:27`, `:36`, `:45`), so a `Saved/Config/<dir>/*.ini`
   the OS will not open (mode `000`, a file another uid created, EIO) now
   contributes nothing: if it is the tree's only stating file, `detect` returns
   `None` and the face answers `claim-refused` (3) — a false statement about the
   game — and if the tree has another readable stating file, the run exits `ok`
   (0) over a document whose `provenance.files` silently omits a file that exists
   and states bindings. At the revision's base this same input located the file
   and exited `extraction-refused` (5) naming it, which is what I approved. The
   existing pin for the rule, `crates/codecs/src/families/unreal_ini.rs:502-522`,
   exercises only `FileOverBudget`, so the new arm is unpinned and the pin's name
   now claims more than it proves.
   *Fix:* keep the perf fix (the open is already counted against
   `MAX_CANDIDATE_FILES` at `:245-248`) and locate on the `UnreadableFile` arm
   too, so `extract` refuses it by name as designed; if the silent skip is
   instead the chosen rule (the dangling/raced-entry argument behind the new
   comment), amend §The first family's sentence and 017-L1's reading in
   `03-build-notes.md` plus a Revision-log line, and pin the arm with a
   non-readable candidate file asserting exit 3 vs exit 5 explicitly.

2. **minor — the new loader spelling bound is not enforced on the producer, so
   the shipped family can emit a document its own loader refuses.**
   `crates/codecs/src/snapshot.rs:311-315` refuses any `provenance.files` entry
   containing `\` or `:` (with `schemas/bindings/v1/spec.md:75-80` now stating
   that bound), but the locator's grammar is `Saved`/`Config`/`.ini` only —
   `crates/codecs/src/families/unreal_ini.rs:280-287` puts no character
   constraint on `<one directory>` or `<name>`, and `extract` maps
   `tree_relative` over every located file with no filter
   (`:128-134`). On any POSIX host `:` and `\` are ordinary name bytes, so
   `Saved/Config/LinuxNoEditor/Game:Settings.ini` (or `Game\Settings.ini`)
   locates, folds, and lands in `provenance.files` verbatim — and
   `load_snapshot` then refuses it naming `files`. Test strategy #11
   (proposal:703-704: "documents produced by the family all pass the loader — the
   extraction and contract halves of the run cannot disagree") is the invariant
   this breaks; its fixture route simply has no such name, which is why the gate
   is green. It also quietly decides that the digest rule of contract amendment 3
   (proposal:446-455) can never cover such a file.
   *Fix:* close the loop on the producer side — skip (or locate-and-refuse) a
   grammar match whose name bytes are outside the contract spelling, so the
   family never states a path its contract rejects — or, if the family is to stay
   name-agnostic, narrow the spec bound to the spellings that actually resolve
   out of the tree and record the choice.

3. **nit — `tree_relative`'s new doc-comment claims a `/`-based sort that the
   code does not do.**
   `crates/codecs/src/families/unreal_ini.rs:544-546` states, as the reason the
   `/`-joined spelling is safe, that "the byte-order sort stays `/`-based"; the
   sort is `files.sort()` over `PathBuf` (`:113-114`), i.e. host-native separators
   and native byte order. On Linux the two orders coincide, so nothing observable
   changes; on a host whose separator is not `/` they diverge (`\` is `0x5C`, so
   `Config\a\Z.ini` sorts after `Config\a2\A.ini` while the emitted
   `a/Z.ini` sorts before `a2/A.ini`), so the emitted `provenance.files` array is
   not in the byte order §Determinism fixes (proposal:542-544) and the fold order
   is not the order the same revision restated at `schemas/bindings/v1/spec.md:85`
   ff. The Windows leg claims nothing (Compatibility matrix:632), so no test can
   reach it; the finding is the false claim plus the latent non-conformance.
   *Fix:* sort the relative spellings (`files.sort_by_key(|f| tree_relative(f,
   &group.tree_root))`) so fold order, emitted order and the contract's byte order
   are one order on every host, and drop or fix the comment's sentence.

4. **minor — the CLI's claim memo is keyed on a projection of `GameContext`, and
   the records it actually collapses are the ones that differ in exactly the
   dropped fields.**
   `apps/discovery-cli/src/bindings.rs:227-237` memoizes `registry::claim` under
   `(appid, library, roots)`, dropping `lookup_key`, `platform` and
   `install_path` (`crates/codecs/src/context.rs:27-41`). 017-L4 defines a claim
   as pure over *its inputs*, and `detect(&GameContext)` receives the whole
   context, so keying on three of its six fields is sound only for a family that
   reads just those three. The shipped `unreal-ini` does
   (`crates/codecs/src/paths.rs:63-133`), so today the memo is behavior-preserving
   — but the memo's real hit case is the duplicate-record case, and that is
   exactly where the dropped fields disagree: with no `--steam-root`, both
   `SteamLinuxPlugin` and `SteamWindowsPlugin` are constructed
   (`bindings.rs:169-181`), each carrying its own `roots()`; when the two resolve
   overlapping roots they hand back the same `steam/<appid>` row in the same
   library with different `platform` tags. A future family that consults
   `platform` — the natural key for a lineage-scoped grammar — would then be
   handed the other plugin's claim. The new comment at `:222-225` also misnames
   the case it optimizes: two records of one game *in two libraries* have
   different `library` values, hence different keys, hence no memo hit at all.
   *Fix:* key the memo on everything a family may read (the whole `GameContext`,
   or `(appid, lookup_key, install_path, platform, library, roots)`), or state in
   §The codec face / the codec trait docs that `detect` may consult only
   `appid`/`library`/`roots` — that is a contract change, not a comment; and
   correct the comment's description of the collapse case.

5. **nit — the new `proton_trees` comment states the Compatibility matrix
   backwards, and papers over a matrix row the code does not implement.**
   `crates/codecs/src/paths.rs:133-141` now asserts that for the broken/symlinked
   prefix row "the matrix's 'named degradation' … is `claim-refused` (exit 3)",
   and that "`family-confirmed-files-absent` (exit 4) belongs to appid-table
   claims, which `017-R4`'s routing data would be the first to create". The matrix
   row says the opposite of both sentences: "a broken prefix reads as
   `family-confirmed-files-absent` (exit 4), never a hang or an out-of-tree read"
   (proposal:628), as does the Risks row ("a broken prefix degrades to
   files-absent", proposal:615); and the exclusivity claim is contradicted by the
   shipped pin `apps/discovery-cli/tests/cli_bindings_status_tests.rs:102-127`,
   which reaches exit 4 on an `install-tree-probe` claim whose file removes its
   own pairs — a route whose own comment cites that very matrix row. The
   underlying divergence is not new (the code has always answered 3 when the
   prefix is the game's only tree), but this revision is the first to write into
   the source that the design says 3, which is the kind of statement a later
   reader takes as the record.
   *Fix:* drop the matrix citation and the exit-4-exclusivity claim from the
   comment (state only what the code does: the prefix is never anchored, so a
   prefix-only game locates nothing), and settle the row itself on the record —
   either route a family-confirmed broken prefix to exit 4, or file the
   3-vs-4 reading as a Revision-log divergence with its basis.

6. **nit — the aggregate-read sentence now disagrees three ways.**
   `schemas/README.md:200-213` and `schemas/bindings/v1/spec.md:76-83` now bound
   the worst case at `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES ×
   MAX_BINDING_FILE_OCTETS` (1,088 MiB), while the design they are supposed to
   match still states `(17) × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) *per
   extraction* and asserts "spec.md and the README budget bullet state it as such"
   (proposal:357-366). The new text is the truthful description of the shipped
   code (`locate` opens the files for the content test; `extract` re-reads each
   located file), and the arithmetic is right, but the contract-tree text and the
   design now quote different composites, which is exactly the sentence the
   proposal says must match.
   *Fix:* one line in `03-build-notes.md` and a Revision-log entry recording that
   the shipped aggregate counts two passes where the design's sentence counted
   one (or reword the design's sentence when the operator next touches it).

## Carried findings — unchanged by this revision

Each was filed at round 1 at the severity shown; the diff does not touch the code
it concerns, and nothing in the diff makes any of them worse.

7. **minor** — `crates/codecs/src/registry.rs:37-42` still treats an appid-table
   hit as only *selecting* whose `detect` runs, against §The registry's "an
   appid-table hit claims regardless of what the grammar's files turn out to
   contain" — a contradiction the module's own header restates at `:10-11`.
   Latent while `appids()` is empty (A21); the declined-through-the-table case
   remains unpinned (`crates/codecs/tests/registry.rs:169-178` uses an appid the
   table does not list). The new `probed` bookkeeping does not change this.
8. **minor** — `crates/codecs/src/families/unreal_ini.rs:102-104` and
   `apps/discovery-cli/src/bindings.rs:152-159` order documents by `tree_root`
   bytes alone, while the ordering law in
   `schemas/bindings/v1/spec.md` §Ordering (untouched by this revision) and
   proposal:317-319 defines `user_trees` order as `TreeKind` declaration order
   then root bytes — opposite orders for the ordinary
   userdata-plus-`compatdata` case.
9. **minor** — `crates/codecs/src/families/unreal_ini.rs:147-168` still
   reconstructs `game` by scanning tree components for a literal `compatdata` /
   `userdata`, refusing `UnreadableFile` (exit 5) when a root or library folder
   merely carries one of those names; the authoritative value is
   `ctx.lookup_key` at detect time.
10. **minor** — the locate walk is still breadth-first
    (`unreal_ini.rs:191-196`, `:234-241`) where proposal:498-499 says
    "depth-first in byte order"; caps and determinism unaffected, only which
    files a budget-exhausted tree surfaces.
11. **nit** — `Fold::clear` (`unreal_ini.rs:371-375`) still empties `seen` with
    `pairs`, so the admitted counter is not literally "distinct pairs ever
    admitted" and a clear-and-re-add cycle re-charges the budget. The new
    `remove` short-circuit (`:358-369`) is sound precisely because `clear` clears
    both — the comment's lockstep claim holds today, which makes the coupling
    worth noting rather than fixing under this seat's head.
12. **nit** — `apps/discovery-cli/src/bindings.rs:261-281` still derives
    `family-confirmed-files-absent` per family over the merged claim rather than
    per matching record; the memoization added at `:227-237` does not change that
    (see finding 4 for the new part).
13. **nit** — `crates/codecs/Cargo.toml:16-20` still carries `serde_json` as a
    load dependency against proposal:214-217; unchanged (not in this diff), still
    in need of a notes line.

## FYI (out of specialty, no severity)

`schemas/bindings/v1/spec.md:45-51` now documents the *locator's*
`Saved/Config/<dir>/*.ini` grammar and its case-exactness inside the document
format's conformance text; §Determinism's "stated once here and cited elsewhere"
discipline is about the ordering law, so this is placement taste for another
seat, not a correctness issue.
