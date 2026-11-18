# performance review r3

verdict: approve

Scope: whole document (full-panel pass c0, round 3). Previous performance verdict held no findings, so every finding below is new.

## Findings

- [minor][new] Chosen design §2 Root selection, canonical path rule — "every set of directories the plugin scans — roots from the candidate list, and library paths within a root — is built by canonicalizing each entry, ... then sorting, then de-duplicating. ... so no library is ever scanned twice." — the rule de-dupes each set *per root*, so the absolute claim doesn't follow: one external library (e.g. `/run/media/…/steamapps`) listed in the `libraryfolders.vdf` of two coexisting selected roots — a layout the Compatibility matrix itself supports ("native, Flatpak and Snap layouts coexist", §Options "Scan only the first valid Steam root") — is scanned twice: duplicated read+parse IO and two byte-identical `(slug, install_path)` rows, since the canonical library prefix makes the emitted paths equal. Fix: build the libraries-to-scan set as the union across all selected roots before scanning, run through the same canonicalize/sort/de-dupe rule; this replaces the per-root set in the canonical statement's "library paths within a root" clause (one-line algorithmic change, no new mechanism).

- [minor][new] Chosen design §4 KeyValues reader — "the caller reads bytes and skips a file that is not valid UTF-8 rather than aborting a scan. Files larger than 1 MiB are skipped as malformed (A16): the reader's work is bounded per file" — the 1 MiB cap is not tied to the IO step: the natural reading of "reads bytes" then checking validity/size (`fs::read_to_string` and test) slurps the whole file first, so an unbounded read precedes the cap and the stated per-file boundedness — the design's whole substitute for a wall-clock budget — is not guaranteed by the text. Fix: pin the cap at IO in one sentence: a file whose `fs::metadata` len exceeds the 1 MiB constant is skipped without being opened; this replaces the unspecified read-then-check order as the size-gate.

## Checklist coverage

1. Input-path latency budget — N/A: `SteamLinuxPlugin` runs only inside `discovery list` (§8); no keystroke/event path exists, and T-001 sets no latency requirement. All IO is on the on-demand scan.
2. Structure costs (O(n²)) — none found: per-file parse is linear under the caps; candidate probe, root/library sets are sort+dedupe O(n log n) over small sets (candidate list fixed-size, library count capped transitively by the 1 MiB VDF cap); result sort O(m log m). No recursion below `steamapps/` (§3) also avoids walking the `compatdata/` prefix trees, where A4's decoys live — a good perf decision. Only redundant work found is the cross-root repeat (finding 1).
3. Memory growth — no tables or caches persist across runs (persistence/caching is a stated Non-goal); transient peak is one file's parse tree — bounded only if finding 2's IO gate holds.
4. Startup cost — N/A as a boot path; per-run startup is ~15 stat/canonicalize syscalls in the §2 probe plus one VDF read per root — negligible.
5. Caching invalidation — N/A: no cache is introduced, so there is no invalidation surface.
6. Research baseline — §4 quotes the research for the absence of a scan-latency baseline (01-research.md:319-324); Revision log r0/q17 records the per-file (1 MiB / 16-level) budget as the deliberate substitute and *Deferred* ("Scan-latency budgets and a benchmark harness") keeps the follow-up explicit. T-001's Decision clauses impose no latency requirement, so the deferral does not contradict the source task — no unexplained gap, no finding; re-raising "no scan budget" against a sound recorded rebuttal would be a preference.

## Evidence

- A15 / Options dependency posture verified directly: `Cargo.lock` contains exactly 32 `[[package]]` entries and no `pest`, `keyvalues-parser`, `keyvalues-serde`, `steamlocate`, `walkdir`, `tempfile` or `dirs` — so "no new dependency, `Cargo.lock` byte-identical" (§10, Rollout #3) is true against the current tree.
- The cargo/rustfmt/npm check legs were denied by this session's shell permissions; no claim in the document's performance-relevant text requires a cargo run beyond the Cargo.lock posture, which was confirmed by direct file inspection.
- A16's "far smaller than 1 MiB" is admitted unverified with a bounded, counted consequence (skip + §7 count); acceptable as an assumption, and finding 2 restores the guarantee behind it.

## FYI (no severity)

- Finding 1's false absolute ("no library is ever scanned twice") is also a correctness claim about duplicate output rows; the correctness seat may want the same §2 anchor.
