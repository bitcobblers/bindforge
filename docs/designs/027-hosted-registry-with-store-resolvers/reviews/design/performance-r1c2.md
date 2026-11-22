# performance review r1 c2 (dissent pass — changed sections only)

verdict: approve

Scope: `git diff fcb66a83 19dcdcb1` on `02-proposal.md`, then the full current
document. Seat: performance checklist only. Diff sections reviewed: §Options
(partial-snapshot posture), §Hosted mode (query, health), §The hosted lookup
policy step 5, §The steam snapshot resolver, §The store pull, §Issued entries
(retired ids, mint step 5), §Concurrency, §Budgets, §Risks, §Test strategy
#1–#6/#13, §Rollout 1/3/6, §Deferred, §Revision log.

All five of my r1 findings are closed by this revision, and the closures are
the ones I asked for, not evasions:

- r1-major (whole-snapshot `incomplete` → every steam miss `503`): replaced by
  the bounded, named `dropped_appids` list, with the `503` scoped to a listed
  appid only (§The hosted lookup policy step 5, §The steam snapshot resolver),
  the Deferred sentence's false reassurance deleted, and the Risks row
  restated. The availability-cliff claim I flagged is gone, and the residual
  exposure is a new, separately capped budget — reported below as finding 1,
  not as the r1 finding.
- r1-minor (deadline overriding `027-B4`): `STEAM_PULL_DEADLINE_SECONDS =
  1_920` ≥ `12 × (3 × 32 + 2 × 30) = 1,872` (arithmetic checked: 12 × 156 =
  1,872 ✓), and `steam pull deadline` is its own named refusal, distinct from
  the over-`STEAM_REFRESH_MAX_PAGES` one. Residual wording precision is
  finding 3 (nit).
- r1-minor (delta merge algorithm and transient): "one linear two-way merge of
  the loaded vector and the pulled changes plus one index build (never one
  insert per changed appid), re-persisting once", and the transient is in the
  §Budgets aggregate. Closed (its enumeration is incomplete in one direction —
  finding 2).
- r1-nit (only one pull initiator counted): "the refresh thread's worst day is
  24 × 12 × 3 = 864 calls, ~115× inside A6's 100,000/day, and a startup pull
  spends ≤36 of them … bounded only by process starts", with the restart policy
  in Rollout 6. Closed; 100,000/864 = 115.7 ✓, and the 32-minute startup block
  caps a restart loop at ~45 starts/day ≈ 1,620 calls, so the posture holds.
- r1-nit (retired cap has no count reading): "≈250k ids at the 128-octet id
  bound, >1M at typical ids" (33,554,432 / 131 ≈ 256k ✓; at a 20-octet id,
  /23 ≈ 1.46M ✓) and "the runbook reads the file's size". Closed.

New findings below are all created by the r2 text itself.

## Findings

- [minor][new] §The store pull — "a list that would exceed `MAX_DROPPED_APPIDS`
  is itself a named pull failure" (with §Budgets' `MAX_DROPPED_APPIDS =
  1_024`, §The steam snapshot resolver's "the list merges as a set on delta …
  and resets only on a full-pull replace", §Risks' trigger "a 43-character CJK
  or 32-character emoji title reaches it", and §Rollout 6's "forcing a full
  refresh by deleting the snapshot"). The new cap's breach re-imports, by
  accumulation, the exact posture the drop exists to avoid — "failing the pull
  over one is remote, permanent denial of the supply" — and nothing in the
  changed text prices it. The mechanics the r2 text commits to: the list only
  grows on delta (an entry clears only when a delta re-delivers that appid with
  a within-bound `name`, and "Deltas never see removals"); a failed pull
  publishes nothing, so `fetched_at` never advances, so the next delta after
  `STEAM_RETRY_SECONDS` asks the same `if_modified_since` window and re-sees the
  same over-bound record — the breach is therefore permanent, not transient.
  Consequence chain: the snapshot ages past `STEAM_SNAPSHOT_MAX_AGE_SECONDS`,
  §The hosted lookup policy's last bullet turns *every* steam miss into a named
  `503`, and per §Compatibility matrix a bound CLI "fail[s] the whole scan
  closed", i.e. the r1 blast radius, reached by count instead of by one record.
  Recovery is refused by the same rule: since an entry leaves the list only when
  a delta shows that appid within bound, the list's contents are a subset of the
  catalog's currently over-bound records (bar delistings deltas cannot see), so
  a list at 1,024 implies a full pull over the whole catalog that also breaches
  1,024 — the runbook's recovery lever, deleting the snapshot, is then refused
  too, and `--no-steam` is the only boot left, which is the terminal state. Same
  cap also bites the loader ("a `dropped_appids` longer than
  `MAX_DROPPED_APPIDS`" is a named refusal), so a drifted file plus a refusing
  pull is a startup refusal. And the signal is thin: r2 removed the count from
  health ("`dropped_appids` is not a health fact — the pull writing the list
  prints it"), `stats` is `{games, source_keys, revision, issued}`, and Rollout
  6's watch line covers only "reading the retired list's size against its cap".
  Likelihood is genuinely low — A10's basis is that 128 was set "one octet above
  the longest surveyed store title", so breaches sit beyond the observed
  maximum, and 1,024 of them is a large aggregate — which is why this is priced
  rather than gated. Fix: make the breach degrade instead of denying — replace
  "a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure"
  with publishing the capped list under the whole-list posture (the r1 posture
  retained *only* here, where the unenumerated set genuinely is unknown: no
  steam miss is certifiable until a full pull fits under the cap), so a breach
  costs certification, not the supply and not the recovery path. If the
  fail-the-pull posture stands, price it in the §Risks row and §Budgets with
  scope (all steam misses, via the age bound), trigger (≥1,025 over-bound names
  inside one inter-full-pull interval, cumulative on deltas), permanence (a
  failed delta never advances `fetched_at`), and the recovery path, and add the
  list's length to Rollout 6's watch line beside the retired file's size.

- [minor][new] §Budgets — "on top of it a mint holds one model plus one
  serialized buffer, and a delta merge the old snapshot, the merged model, one
  serialize buffer and the current page body." The sentence was rewritten to
  enumerate the process's transients and still omits its largest one, the full
  pull, whose input cap *this same revision* raised: `MAX_STEAM_PAGE_OCTETS`
  went 8,388,608 → 12,582,912 on the stated ground that "the pinned
  `max_results` can answer that page, not the ~150-octet mid-estimate" (the
  arithmetic is sound: 50k worst-legal records at `{"appid":…,"name":<128>,"
  last_modified":…,"price_change_number":…}` ≈ 217–230 octets ≈ 11.4 MiB, inside
  12 MiB). But 12 pages × 12.58 MiB is a 151 MiB admissible input envelope, and
  the only gate on the other side is `027-B1`'s 33,554,432 publish-time check —
  the pull can legitimately read ~4.7× more record bytes than the document may
  hold, and pays the whole read before refusing. A full pull accumulates every
  page before it publishes (it "replaces the snapshot"), so its peak is the
  accumulated model over up to 600k `{appid, name}` records plus the appid index
  plus the current page body, on top of the corpus/issued/retired documents the
  same boot already loaded (16 + 64 + 32 MiB at their caps). The "144 MiB of
  JSON … as the boot load ceiling" figure is a load bound, not a pull bound, and
  is being read as the process's memory story. Fix: one clause in the same
  sentence naming the full pull's transient and how it is bounded — check the
  `027-B1` publish ceiling as the pages accumulate (or state that ≤12 pages of
  records is the process's largest transient) — so the three writer transients
  (mint, delta merge, full pull) are all on the table the §Deferred batched-writes
  deferral defers against.

- [nit][new] §The store pull — "sized in §Budgets to cover this grid with its
  retries, so a slow store costs a late snapshot, not a guaranteed failure."
  The grid in §Budgets, `12 × (3 × 32 + 2 × 30) = 1,872`, charges each attempt
  `STEAM_CONNECT_TIMEOUT + STEAM_READ_TIMEOUT` = 32 s as if those bounded a
  *page*; the framing the design says it copies hand-rolls them as socket
  timeouts (`apps/registry/src/upstream.rs:97`
  `set_read_timeout(Some(constants::UPSTREAM_READ_TIMEOUT))`, with the body read
  to completion), which bound one `read` call, so a steadily trickling body
  never trips the 30 s bound and a page's legal wall time is unbounded. The grid
  is therefore a floor, and the implied floor on throughput — 12,582,912 / 32 s ≈
  393 KiB/s per worst-legal page — is the deadline's real precondition, with the
  deadline as the only wall-clock bound in the family. For a typical catalog
  (≈200k records ≈ 11–16 MiB serialized) 1,920 s is ample, so this bites only
  worst-legal pages on a slow link, where the honest outcome is `steam pull
  deadline`. Second half of the same sentence's scope: on the pre-bind path a
  deadline-killed pull is a startup refusal, so the figure the document never
  states is that `serve` (and `query`, which takes "same startup as `serve`
  including load-or-pull") with no fresh snapshot can block up to
  `STEAM_PULL_DEADLINE_SECONDS` — 32 minutes — before binding. Fix: one clause
  saying whether the two timeouts bound a read or a page (and, if a read, that
  the grid is a floor and the deadline is the only page-total bound), and one
  naming the worst-case pre-bind block as ≤1,920 s before bind so the runbook's
  start-timeout and restart-policy line has a number to sit against.

## FYI (no severity)

- Request-path cost check, clean: policy step 5 now probes `dropped_appids`
  membership on every steam miss. The design names the probe shape for `apps`
  ("a sorted vector plus an appid index") and for the retired list ("ordered
  insert with membership by binary search") but not for the dropped list; at
  `MAX_DROPPED_APPIDS = 1_024` even a linear probe is sub-microsecond, so no
  finding — the loader already pins the list ascending, so a binary search is
  free if an implementer wants it.
- For the correctness seat: a `dropped_appids` entry clears only when a delta
  re-delivers that appid with a within-bound `name`, and "Deltas never see
  removals", so a delisted over-bound app's entry persists until a full pull —
  the list is self-clearing for renames, not for delistings. Also the
  cap-breach interaction with the loader refusal (finding 1) is a startup-state
  question as much as a budget one.
- For the compatibility seat: r2's removal of the `incomplete` fact from
  `health.steam_snapshot`, to ship "`027-C3` shape verbatim", buys ledger
  fidelity at the cost of the only *pollable* surface for the dropped-list
  state — the surviving signals are the pull's printed line and the file
  itself. Noted inside my finding 1's fix rather than argued here.
- The mint-bound substitution (step 5's `steam-<appid>` from the queried key's
  external id, bounded by `MAX_DISCOVERY_SLUG_OCTETS` with the named
  `mint identity over bound` refusal) has no performance content; the added
  refusal is a `503` on a path that was previously claimed-in-budget, which is
  the correctness seat's read.
