# performance review r4

verdict: approve

Scope: the single editorial confirmation pass of round 4, limited to
`reviews/design/editorial-r4.diff` per the caller. Inputs: `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the Revision log, and the caller's
statement that this seat had no previous findings (editorial phase is a single
language pass per round).

## Findings

None. Every hunk in the diff is language-only and none changes a passage with
performance content in a way that could make a performance property false:

- Context, A9, §4, Test strategy #10 hunks: serial-comma insertions,
  `behaviour` → `behavior`, and `does` → `would` — no cost, cap, ordering or
  budget claim is touched. The 1 MiB / 16-level caps, the per-file (not
  wall-clock) budget statement (§4, citing 01-research.md:319-324), the
  `(slug, install_path)` sort, and the canonicalize→sort→de-dup rule are all
  unchanged by the diff.
- Context hunk `settles each one` → `settles each such question` — same
  reference; the Revision log's 19-question coverage (q1–q19, including q17
  performance budget) still holds.
- §2 hunk `a thin wrapper over it` → `a thin wrapper over `from_roots()`` —
  makes the referent explicit; it matches the sentence a few lines above
  ("`from_host()` is a thin wrapper over `from_roots()`") and introduces no new
  scanning or repeated-probe semantics that would affect startup cost.

## Checklist disposition

1. Event/keystroke-path latency budget: n/a — discovery is a per-`list` scan,
   not an event path; unchanged by the diff.
2. O(n²) structures: unchanged — canonicalize-then-sort-then-de-dup per
   `discover()` call, bounded by the fixed candidate list and per-root VDFs.
3. Memory growth: n/a — no tables or caches introduced; persistence/caching
   stays in *Deferred*, consistent with the task's Decision clauses.
4. Startup cost: unchanged — 1 MiB per-file cap and fixed-path reads stand; the
   diff touches only the wording around them.
5. Cache invalidation: n/a — no caching.
6. Research baseline: unchanged — design still states explicitly that no
   scan-latency baseline exists (01-research.md:319-324) and bounds work per
   file; no unexplained gap between expected numbers and a baseline.

## FYI (no severity)

- Nothing to hand to another seat; the diff is language-only.
