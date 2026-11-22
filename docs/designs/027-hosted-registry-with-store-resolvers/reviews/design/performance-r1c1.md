# Performance review r1 (dissent pass c1)

verdict: request-changes

Scope as directed: `git diff 7b5cec46 fcb66a83` over `02-proposal.md`, then the
full current document. Seat: performance checklist items 1–6. Verified against
the shipped code with read-only inspection: `apps/registry/src/corpus.rs:280-291`
(`rebuild_postings`), `:339-357` (`to_bytes` through a `serde_json::Value` DOM),
`:361-365` (`upsert` rebuilds the index), `apps/registry/src/state.rs:462-485`
(write half held across `to_bytes` + `persist::save`), `persist.rs:184-233`
(`sync_all` + rename), `constants.rs:27,76`, `upstream.rs:89-112` (socket
timeouts are `TcpStream::set_read_timeout`, i.e. per read, with the body read to
completion).

## r1 findings — closure check

- **[closed] mint on the client path under the shipped single-lock posture.**
  §Concurrency now states the fix as asked: "that mutex — not the documents'
  `RwLock` — is what keeps persist order from diverging from mutation order",
  writers "serialize the new document straight into the write buffer (a mint
  inserts the new key's posting rather than rebuilding the index), pay
  tmp-and-rename, and take their document's `RwLock` write half only for the
  model swap", and "Readers (serving lookups) take read halves only, so no
  client ever waits on an fsync." The "already an accepted `026` posture"
  mitigation text is gone from §Risks, and §Budgets now carries the measured
  rewrite at 100k/200k as Rollout 4's gate, so §Deferred's batched writes defer
  against a number. The residual per-mint whole-document rewrite is `027-C4`'s
  persist-before-serve plus one JSON document, which is the ADR's shape, and it
  is now accepted, measured and deferred rather than unpriced. Not re-raised.
- **[closed] one retry constant for two tempos.** Split as asked:
  `STEAM_PAGE_RETRY_SECONDS = 30` between page attempts,
  `STEAM_RETRY_SECONDS = 3_600` named as "the refresh thread's between-pull gap
  only, no startup path sleeps it", and `STEAM_PULL_DEADLINE_SECONDS` turns an
  unfinished pull into the named refusal. The hours-long pre-bind block is gone.
  The numbers themselves need reconciling — new finding 2, not a re-raise.
- **[closed] no process-level memory figure.** §Budgets carries the aggregate:
  "The four document caps total 144 MiB of JSON (16 corpus + 64 issued + 32
  retired + 32 snapshot) as the boot load ceiling and a mint adds one model plus
  one serialized buffer to it". Sum verified (16 + 64 + 32 + 32 MiB = 144). The
  delta-merge transient is the one multiple the sentence still misses — new
  finding 3.
- **[closed] retired-ids shape and per-delete cost.** "In memory it is ordered
  insert with membership by binary search (the affordance the snapshot's appid
  index names), and a delete costs two bounded document rewrites, admin-only",
  plus the `retired id list full` posture. As asked.
- **[closed] the A6 ratio.** Restated attempt-shaped: "the worst pull is 12
  pages × 3 attempts = 36 calls, ~2,800× inside A6's 100,000/day, and ~115× even
  counting the refresh thread's daily 24 × 36 = 864". Arithmetic checks
  (100,000/36 ≈ 2,778; 100,000/864 ≈ 115.7). The enumeration's coverage is the
  subject of new finding 4.

## Findings

- [major][new] §Deferred /name-bound entry/ + §The steam snapshot resolver +
  §Risks — "until then the breaching record is dropped and the snapshot
  publishes `incomplete`, which is what keeps the bound from being an
  availability cliff" — the r1 substitution moves the cliff, it does not remove
  it, and the claim that it does is false as the changed text now stands. The
  blast radius is the entire steam **miss** path for the whole instance, for as
  long as the offending record stays in the catalog: policy step 5 says
  "`incomplete` > 0: no miss is certifiable, so every steam key that is not an
  appid hit takes the named `503` `steam snapshot incomplete`", and the
  §Compatibility matrix's newly revised client row states the consequence —
  "The new `503`s render as the shipped status-line-only refusal *and fail the
  whole scan closed*". Nothing in the design clears the mark: a full pull
  re-encounters the same name and re-drops it, a delta only "bumps `incomplete`",
  deleting the snapshot file re-pulls the same catalog, `--no-steam` is strictly
  worse, and re-sizing the bound is deferred — so the operator has a visible
  (`health.steam_snapshot.incomplete`) but unrepairable permanent state,
  triggerable by content the design itself calls "attacker-chosen" (A11) and
  reachable without any attacker, since the bound is octet-based: a 43-character
  CJK title or a 32-character emoji title reaches `MAX_NATIVE_NAME_OCTETS = 128`
  (A10's basis is one octet above the longest *surveyed* title, which says
  nothing about the 164k–200k catalog A5 counts — checklist item 6, an
  unexplained gap between the baseline and the design's expected numbers). And
  because the pull sends no `include_*` flag, every real appid outside the
  games-only set (DLC, tools, videos) is an ordinary miss that answers `200 []`
  today and `503` while the mark stands. Fix: keep the drop, but scope the
  uncertifiability to what was actually dropped — publish a bounded list of the
  dropped appids beside the count, refuse `200 []` only for keys naming those
  appids, and let the whole-list posture apply only when that list exceeds its
  own cap; alternatively, if the count-only posture stands, delete the
  "availability cliff" clause and price the posture in §Risks with its scope
  (all steam misses), its trigger (one over-bound name, remote or natural), its
  permanence (until the deferred bound amendment), and its recovery path, so the
  choice is made at implementation rather than discovered at the first long
  title.
- [minor][new] §The store pull / §Budgets — "`STEAM_PAGE_ATTEMPTS = 3` separated
  by `STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS = 900`"
  — the deadline does not enclose the pull the same family authorizes, so it
  silently overrides `027-B4`'s page budget instead of bounding it. At the
  nominal timeouts the design's own worst legal pull is 12 pages × (3 ×
  (2 s + 30 s) + 2 × 30 s) = 1,872 s, already 2× the 900 s deadline; and the
  shipped shape these constants copy (`upstream.rs:97`:
  `set_read_timeout(Some(UPSTREAM_READ_TIMEOUT))` with the body read to
  completion, byte-at-a-time in `read_response`) bounds one **read**, not one
  body, so a page's wall time is not 32 s at all and the deadline is the only
  wall-clock bound in the family — a pull that is making honest progress at
  ~110 KiB/s across 12 ≤8 MiB pages is killed. Consequences differ by path and
  are unnamed: pre-bind, a killed pull is the startup refusal (an operator on a
  slow link cannot boot); on the refresh thread, it keeps the snapshot aging
  toward `027-B2`'s conversion of misses to `503`s. Fix: state the relation
  `STEAM_PULL_DEADLINE_SECONDS ≥ STEAM_REFRESH_MAX_PAGES × (STEAM_PAGE_ATTEMPTS ×
  (STEAM_CONNECT_TIMEOUT + STEAM_READ_TIMEOUT) + (STEAM_PAGE_ATTEMPTS − 1) ×
  STEAM_PAGE_RETRY_SECONDS)` and set numbers that satisfy it (1,920 s fits, or
  cut the per-page pacing), and name the deadline-killed pull as its own
  refusal distinct from the over-`STEAM_REFRESH_MAX_PAGES` one, so the two
  worst-case guards are checkable arithmetic rather than two independent guesses.
- [minor][new] §The store pull + §The steam snapshot resolver — "a delta merges
  by `appid` (replace, or insert keeping ascending order) and re-persists" read
  against "In memory the snapshot is a sorted vector plus an appid index" —
  taken literally, a delta of *m* changed appids into the *n* ≈ 200k sorted
  vector is *m* ordered inserts, each memmoving up to the whole structure and
  re-shifting the appid index beside it; the design names the read probe ("one
  exact-`appid` probe") and, for the retired list, the mutation shape and its
  cost, but for the one bulk merge it runs daily it names neither the algorithm
  nor the transient. §Budgets' new aggregate counts only the mint ("a mint adds
  one model plus one serialized buffer"), which understates the largest
  transient the process has: a merge holds the old snapshot model, the merged
  model, the ≤32 MiB serialize buffer and the current ≤8 MiB page body at once.
  Fix: one clause — the delta publishes via a single linear two-way merge of the
  loaded vector and the pulled changes plus one index build (never one insert
  per changed appid), with its transient named in that §Budgets sentence beside
  the mint's multiple.
- [nit][new] §Budgets — "the worst pull is 12 pages × 3 attempts = 36 calls,
  ~2,800× inside A6's 100,000/day, and ~115× even counting the refresh thread's
  daily 24 × 36 = 864 — no separate quota machinery" — the r1 restatement fixed
  the attempt shape but its enumeration counts only one of the design's three
  pull initiators. §Hosted mode gives `registry query` "same startup as `serve`
  including load-or-pull", and §The steam snapshot resolver's startup rule
  ("load the snapshot if present, loadable, and fresh, else pull in full") means
  every `serve` start and every `query` against a missing or aged snapshot
  spends 12 calls (36 with retries) with none of the refresh thread's 3,600 s
  spacing — a supervisor restarting a crashing `serve` is the case with no gap
  at all. Fix: one clause that the counted spend is the refresh thread's, and
  that startup pulls are bounded by process starts, with the restart-loop case
  named as the runbook's restart-policy line.
- [nit][new] §Issued entries/retired ids/ — "`MAX_RETIRED_IDS_OCTETS =
  33_554_432` gated at load (metadata-gated) and against the buffer a write
  renames. ... A delete whose retired append would breach the cap is the named
  refusal `retired id list full` and mutates nothing" — this is the one budget
  whose breach is its own first signal: `stats` deliberately does not report the
  retired count (§Deferred), and unlike the other three documents the r1
  aggregate sentence gives no entry-count reading of its octet cap. The list also
  only grows for the life of the instance (every delete, including every
  promotion), so it is the only cap that cannot be relieved by curation. Fix: one
  clause giving the id count the cap implies (≈250k ids at the 128-octet id
  bound, >1M at typical ids), which is what makes the refusal's likelihood an
  operator-judgementable figure; the runbook carries the file-size check for it.

## FYI (no severity)

- Health's `steam_snapshot` fact now carries `{games, fetched_at, incomplete}`
  while `027-C3` names "`{games, fetched_at}` or `none`" — an unflagged extension
  of an ADR-named document shape; the compatibility and correctness seats may
  want the amendment line.
- `write.lock` makes `registry query` refuse to run beside a live `serve`; the
  one-shot's purpose is operator inspection, so the refusal is a workflow change
  worth a correctness/complexity seat's look.
