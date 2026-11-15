# Performance review r1

verdict: approve

Scope: whole document, pass c0, round 1. Inputs used: `02-proposal.md`,
`docs/adrs/021-decision-unit-ledger.md`, the Revision log, and read-only
corpus checks (bounded globs over the two corpus prefixes named in ¶5).
Shell was denied in this session, so no `npm`/`cargo` leg was executed and
no prior report was re-read (the injected previous-findings list is empty).
Revision log read first: r0 records no dispositioned findings, so nothing
below collides with an authorial rejection.

## Checklist walk

1. **Input-path latency budget.** No keystroke/event path exists in this
   design. The gate runs synchronously at exactly two points per run —
   `runPipeline` entry (after `sourceAdmissible`, A9) and `finalizeDone`
   (A10) — once each, never per stage or per event. Per-event cost on an
   interactive path: not applicable. The entry-path IO is bounded and
   verified below; no budget number is stated but none is needed for a
   once-per-run scan of a repo-local set. Not a finding.
2. **Structure costs / O(n²).** `checkLedgers` is a linear walk: parse each
   ledger → registry; scan each corpus line → handle set; resolve handle↔unit
   by lookup. Growth axes are hard-bounded by the grammars themselves: the id
   regex fixes unit numbers to 2 digits (`\d{1,2}`, ¶2) and ADR numbers to 3,
   so per-file unit count ≤ 5×99 and file count is corpus-bounded. No
   bindings × devices × platforms product appears. Clean.
3. **Memory growth.** ¶1 defines pure readers — `node:fs` reads, never
   writes, no cross-run cache. The registry and handle set are per-call and
   discarded; the deny reason is explicitly bounded ("first twelve findings
   … plus `+N more`", ¶7). Clean.
4. **Startup cost.** The entry hook reads the corpus: ¶5's two prefixes.
   Verified by glob: `docs/adrs/*.md` = 19 files, `schemas/**/*.md` = 9,
   total 28 — A23's count is exact, and the same listing corroborates A1
   (017–019 files absent). Twenty-eight small repo-local markdown files read
   once per run at entry is negligible against pipeline startup. Clean.
5. **Caching/invalidation.** The design caches nothing — every call re-reads
   from disk, so invalidation is correct by construction. Not applicable.
6. **Research baseline.** `01-research.md` is not an input on this pass, so
   citations to it cannot be quoted. Mitigated: every quantitative claim the
   design makes for its costs is checkable from the given inputs and checks
   out — the 28-file corpus (verified, item 4) and 021's ten units
   (C1–C3 + L1–L5 + R1–R2 = 10, verified against the ADR's Decision).
   A3's "24 units in 013" cannot be independently verified without opening
   013 (not an input), but it bears no performance property: unit counts are
   bounded by the 2-digit id grammar regardless. No expected-number vs
   baseline gap is identifiable; recorded as partial evidence, not blocking.

## Findings

- [nit][new] Chosen design ¶4 — "A handle is the maximal trailing run of
  backticked unit ids on a source line: one or more code spans, separated by
  single spaces, each containing only ids separated by single spaces,
  followed only by spaces and at most one table-cell `|` before the line
  terminator." — as the "canonical statement," this grammar leaves the
  matching strategy open to a naive nested-quantifier regex (span-internal
  spaces, inter-span spaces, and trailing spaces are adjacent ambiguous
  separators), which backtracks polynomially on a long non-matching line.
  Corpus lines are short, so real cost is negligible — hence nit, not major.
  Fix: one sentence in ¶4 stating the match is a deterministic single pass
  per line (right-to-left scan splitting on backticked spans, remainder must
  be spaces plus at most one `|`), replacing any unanchored nested-quantifier
  regex an implementer might reach for; #2's fixtures already pin the
  accept/refuse cases this scan must reproduce.

## FYI (no severity)

- A24 ("no CI workflow run was observed") is the design's only unverified
  load-bearing assumption, and the red-corpus risk row leans entirely on it
  — an availability/process concern the correctness seat may want to weigh;
  from a cost standpoint, #11 as a standing `npm run check` leg is a 28-file
  millisecond-scale scan, so the mitigation is performance-cheap if it lands.
