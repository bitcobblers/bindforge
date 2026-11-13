# performance review r1c2

verdict: approve

Ratification pass over the r2 revisions only (diff of the two 02-proposal.md
blobs), checked against the r1c1 report's single finding. The r1c2 report path
was empty at review time (nothing there to re-read); the quoted r1 findings
are the baseline.

## Findings

None. The revisions break nothing the performance seat approved in r1, and
the one open nit is closed:

- [closed][re-raised → resolved] Chosen design 2 — "the record keeps the
  `boundedGateEvidence` bounding (first N paths plus the count)" — r1c1's nit
  (unbounded recorded/named path list) is answered: the checkpoint record
  caps `paths` to first N with the count kept alongside, exactly the proposed
  fix. §3's status line needs only the count ("how many harness files it
  changed"), and §4's cross-reference use survives for the common small
  window. Verified the precedent exists and bounds at write time:
  `boundedGateEvidence` (runner.ts:3057–3075) keeps `legs.slice(0,
  MAX_GATE_LEGS)` and appends a `${elided} further gate leg(s) elided by the
  record cap` entry — the "(first N paths plus the count)" parenthetical
  matches the shipped posture. No finding raised; N itself is an
  implementation constant on the precedent of `MAX_GATE_LEGS`.

Checklist re-walk over the changed text:

1. Input-path cost: the net-diff call runs once per finalize window (minutes-
   long windows, A12 unchanged), not per event — unchanged posture from r1.
2/3. Bounded costs: the changed command `git diff -z --no-renames
   --name-only <headBefore>..HEAD` is performance-neutral-to-positive —
   `--no-renames` opts out of rename detection (on by default via
   `diff.renames` since Git 2.9), and `-z` skips path quoting. The parse
   reuses the existing linear `nulPaths` (runner.ts:262–264), already fed
   `-z --name-only` output elsewhere (runner.ts:307–309). The stored list is
   now bounded (above). No O(n²) introduced.
4. Startup: no new boot-time reads; the widened run/resume result and the
   inert-key load add no cost (verified `loadState` returns
   `parsed as PipelineState`, state-store.ts:239–256 — A7's revised claim is
   accurate; consumer-visible consequences are the compatibility seat's).
5. Caching/invalidation: resume-clears-at-entry and the loop-top check are
   untouched by the diff.
6. Baseline: A12 (seconds vs minutes) remains the unverified magnitude
   already carried in r1; the diff does not change it.

## FYI (no severity)

- The Revision log credits the bounding to security r1, but the same
  sentence closes this seat's r1 nit — the reviser can mark both closed by
  the r2 Chosen design 2 entry.
- A7's rewording ("strips" → "inert", key round-trips through later saves)
  is a correctness/compatibility claim; it verified against
  state-store.ts:256 but is not the performance seat's verdict.
