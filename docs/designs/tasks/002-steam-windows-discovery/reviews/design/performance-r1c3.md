# performance review r1c3

verdict: approve

Ratification pass. Scope: `git diff d5bafff bc928d9` on `02-proposal.md` (the r1
revisions: A1 → A1/A1b split, three-variant `RegistryKey` with one (key, view)
per reader call and `host_candidates` owning view order, `#[cfg(windows)]` CLI
roots branch, Compatibility-matrix and Test-strategy #2/#3/#8/#13 and Rollout #4/#5
rewordings, Deferred's CLI-seam item removed, Revision-log r1 entries), then the
full current document. Question asked throughout: does any of this break what the
performance seat approved in round 1, or make §5 Root candidates — the section the
r1 round cited — false?

## Checklist walk-through

1. **Input-path latency budget** — not applicable as a keystroke/event path: the
   changed text touches only the startup host probe and CLI plugin selection. The
   one cost delta in the diff is real but trivial and bounded: §3's reader is now
   "one `get_raw_value(name)` in that call" per `RegistryKey` variant and §5 appends
   both HKLM views unconditionally ("as two ordered candidates"), which *Test
   strategy* #8 pins ("querying exactly the three `RegistryKey` variants §5 names,
   in that order"). That is three in-process `RegOpenKeyEx`+`RegQueryValueEx` calls
   per `discovery list` where r0's reader short-circuited the 64-bit view, replacing
   the short-circuit with a collapse at the canonical-path rule. No filesystem
   access, no allocation on any repeated path; `host_candidates` stays pure.
2. **Structure costs, O(n²)** — none introduced. The candidate list is a fixed
   five entries (HKCU, HKLM×2 views, two env defaults); `canonical_paths` is a
   `BTreeSet` insert over that list (crates/discovery/src/steam_linux.rs:322-334),
   and the roots→libraries→manifests nesting is unchanged from r0 (§2 is not in
   this diff).
3. **Memory growth** — no caches, tables or persistence in the changed text;
   *Non-goals* still excludes persistence and caching.
4. **Startup cost** — the surviving-candidate cost is what §5 governs; see the
   finding. `absolute_env`, moved verbatim into `steam_common` per §1, keeps the
   relative-value drop at §5 item 3's cost of one string check (crates/discovery/
   src/steam_linux.rs:338-342).
5. **Caching / invalidation** — no caching added; not applicable.
6. **Research baseline gap** — the diff adds no expected-numbers claim, so there is
   no new gap to measure against the research baseline. The one quantitative claim
   the diff does touch ("doubling every CLI assertion") is a test-assertion cost,
   and the design's choice to keep the Linux roots branch one-plugin (§8) is the
   cheaper of the two options considered, so no cost was added to the approved
   Linux path.

## Verification run

Read-only inspection of the code the changed §5 text relies on (paths confirmed
from the prompt and the document, no browsing of run records):
`canonical_paths` canonicalizes into a `BTreeSet` and drops non-canonicalizable
and `steamapps`-less entries (crates/discovery/src/steam_linux.rs:322-334);
`from_roots` runs roots through it (steam_linux.rs:46-50); `discover()` unions
every root's libraries and de-dups that union before scanning
(steam_linux.rs:61-71); `libraries_in_root_with_skips` reads exactly one
`<root>/steamapps/libraryfolders.vdf` (steam_linux.rs:148-152); `scan_library` is
one `read_dir` plus one `read_text_file` per `appmanifest_*.acf`
(steam_linux.rs:200-231); `read_text_file` applies the cap *before* allocating
(steam_linux.rs:287-298) at `keyvalues::MAX_DOCUMENT_BYTES = 1 MiB`
(crates/discovery/src/keyvalues.rs:21). §5's new claim that "the canonical-path
rule collapses both views reporting one path" is true by that construction, and
the *Test strategy* #3 case "one path from both → one root" is the pinned version
of it. §5's `absolute_env` cross-reference to §1 checks out: §1 lists it among the
moved-verbatim items.

## Findings

- [minor][re-raised] §5 Root candidates — "`canonical_paths` collapses the common
  case where the registry points at the default folder, so a double-listed root
  costs nothing." — the substance of the round-1 finding is untouched, and this
  round the revision rewrote the very line the sentence sits on (candidate item 2
  now appends two HKLM-view values, and *Test strategy* #3 now pins "distinct
  paths → two roots and rows from two installs"), so the paragraph now governs
  three doubling sources — registry-vs-env default, HKLM 32-bit view vs 64-bit view,
  and the two env defaults against each other — while the cost statement still
  establishes only the identical-path collapse. When two candidates resolve to
  *different* folders that both exist (Steam on another drive with a leftover
  `%ProgramFiles(x86)%\Steam\steamapps`, or a stale 64-bit-view `InstallPath`
  pointing at a previous install), both roots survive and each is scanned: one
  `libraryfolders.vdf` read capped at 1 MiB (crates/discovery/src/keyvalues.rs:21,
  applied pre-allocation at crates/discovery/src/steam_linux.rs:287-298), then per
  library only that root declares a `read_dir` plus one metadata+read per
  `appmanifest_*.acf` (steam_linux.rs:200-231). Library-level de-dup over the union
  of all roots' libraries (steam_linux.rs:61-71) bounds the overlap, so the real
  cost is small — the finding is that the design asserts the cost away in the
  section that decides to append candidates without an existence gate, not that the
  cost is large. Fix: replace "so a double-listed root costs nothing" with the
  bound, once, covering all three sources: "…so candidates that resolve to one
  folder collapse to one scan; when two resolve to different folders that both
  exist, each extra root costs at most one 1 MiB-capped `libraryfolders.vdf` read
  plus, for the libraries only it declares, one `read_dir` and one capped read per
  manifest — and `discover()` de-dups the libraries roots share
  (crates/discovery/src/steam_linux.rs:61-71), so no library is scanned twice."
  No mechanism added, no candidate ordering or candidate set changed, so nothing
  else in §5, §2, #3 or #8 needs to move.

## FYI (no severity)

- §3 / *Test strategy* #8: dropping the reader's 32-bit→64-bit short-circuit for a
  pure-code policy in `host_candidates` is the right trade — it moves the ordering
  decision into the unit-tested, Linux-runnable path and costs at most one extra
  registry read per probe. Noted only so the correctness and compatibility seats see
  that both views' values are now always collected, which is what makes #3's
  "distinct paths → two roots and rows from two installs" reachable, and whether
  two live installs ought both to be scanned is their call, not the performance
  seat's.
- Assumption register A1b: the split is sound on the point the design leans on —
  `keyvalues` keeps a lone `\` before any other byte verbatim
  (crates/discovery/src/keyvalues.rs:186-207), so "no spelling assumed" holds for
  the reader regardless of which spelling Steam writes. What a single-backslash
  `path` value then *means* as a library path is a correctness/compatibility
  question.
- §8 / Deferred: removing the "explicit Windows-roots CLI seam" item and replacing it
  with the `#[cfg(windows)]` roots branch is cost-neutral on every host (the Linux
  roots branch keeps T-001's one-plugin shape, and on the probe path the Linux plugin
  on a Windows host yields zero candidates, so `discover()` skips its scan loop
  entirely at steam_linux.rs:61). Whether §8 now satisfies the task's ":65-67"
  clause is for the correctness and compatibility seats.
