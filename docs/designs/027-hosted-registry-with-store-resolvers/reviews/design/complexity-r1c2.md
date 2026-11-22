# complexity review r1c2 (dissent pass: the r2 revision)

verdict: request-changes

Scope as directed: `git diff fcb66a83 19dcdcb1` on `02-proposal.md`, then the
full current document. Inputs: the proposal, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
its Revision log, my r1 findings. Read-only verification against the tree:
`apps/registry/src/keys.rs`, `crates/discovery/src/game.rs`,
`crates/discovery/src/steam_common.rs`, `apps/registry/src/cache.rs`,
`apps/registry/src/corpus.rs`, `apps/registry/Cargo.toml`.

## My r1 findings — all three closed, verified

1. **[major, closed]** Mint step 5's appid bound. The clause now rests on a
   cited, real cap: `key_is_admissible` refuses `key.len() >
   MAX_DISCOVERY_SLUG_OCTETS` on the joined, decoded key
   (`apps/registry/src/keys.rs:73`; the constant is 128, re-exported from
   `bindforge_core`), and the external-id share is exactly `128 − len("steam/")`
   = 122 — the same subtraction the tree itself makes at
   `crates/discovery/src/steam_common.rs:469,482`. So `steam-<appid>` ≤ 6 + 122
   = 128 holds as arithmetic over a stated bound, the policy admits only
   admissible keys, and the non-decimal case never reaches the mint. The
   belt-and-braces `503 mint identity over bound` is unreachable on the served
   path but unit-testable through `mint_id`'s own parameters, so Test strategy
   #1's row is writable. No re-raise.
2. **[minor, closed]** `games ∩ retired`. Named as "the legal, recoverable
   state", the append is a set insert, and Test strategy #5 pins both the
   planted overlap and the no-op re-delete. Checked the recovery for both
   delete routes: in the half-applied state the record is still present, so the
   retry is not the `Absent` refusal. Only the cap-interaction nit below remains.
3. **[nit, closed]** The retry grid closes: `12 × (3 × 32 + 2 × 30) = 1,872 ≤
   1_920`, and `steam pull deadline` is its own named refusal.

## Findings

- [major][new] §The steam snapshot resolver — "`dropped_appids` is the
  ascending, duplicate-free list of the `appid`s the pull dropped for the
  `013-B4` name bound; normally empty" (+ §The hosted lookup policy step 5:
  "The key's appid is listed in `dropped_appids`: the named `503` `steam
  snapshot incomplete`") — the r2 answer to the over-bound-name problem adds a
  whole bookkeeping subsystem where the repo already ships the rule for this
  exact data class, and the subsystem's answer to "one store title is too long"
  is to make that game permanently unanswerable. What r2 adds, in one list: a
  new field in a contract document the ADR fixes (`027-C3`'s shape
  `{version, fetched_at, apps}`), which costs a Rollout-1 `amended-by` on the
  ADR this run implements; a new constant; set-merge/self-clear/reset semantics
  across deltas; four new loader refusals (order, duplicate, non-decimal,
  over-cap) plus two Test-strategy rows; and a new policy branch whose served
  consequence, per §Compatibility matrix ("a bound CLI fails the whole scan
  closed"), is that a user who owns a long-named title can never complete
  `discovery list` against the hosted instance — the appid is not in `apps`, so
  the mint path cannot fire and the answer is `503` forever, against `027-S1`'s
  "every other steam key answers from the snapshot on day one". The smaller
  design is already in the tree and already justified in prose:
  `crates/discovery/src/game.rs::friendly_name` clamps a store display name to
  "the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`", with
  the stated rationale "a truncated display string costs a shorter label, never
  a different identity" — and `apps/registry` already depends on `discovery`
  (`Cargo.toml:14`), so it is an import, not a dependency. A record whose name
  is over the bound (or carries control bytes) is normalized at the pull seam
  into the `013-B4`-legal form instead of dropped: the appid stays in the
  snapshot, so no miss needs un-certifying, `027-C3`'s shape ships verbatim with
  no amendment, and the mint's existing steps absorb the rest — the fold never
  exceeds the name's octets, two titles sharing a 128-octet prefix collide at
  step 4 and fall to step 5's `steam-<appid>`, and `027-L2` plus policy step 2
  mean the derivation is paid once per key, so a later store rename cannot
  re-mint. What this replaces, by name: the drop rule and `dropped_appids`
  everywhere (§The store pull, §The steam snapshot resolver, §The hosted lookup
  policy step 5's third bullet, §Risks row 7, §Deferred's fourth bullet, Rollout
  1's `027-C3` line, Rollout 3, Test #2/#3/#6/#13 rows), `MAX_DROPPED_APPIDS`,
  and the pull's "prints it" log line — one normalization sentence in §The
  store pull in their place. Honest counter, stated: `friendly_name`'s comment
  rests on "nothing parses the result", which is false in the registry because
  the mint derives from this `name`; `027-L2` + policy step 2 + steps 4–5 are
  what make the clamp admissible here instead. If the panel keeps drop-and-mark
  anyway, the least-bad variant is the ADR-free one: keep the *name* and mark
  the *appid* only when the record is genuinely withheld — but then finding 2
  still stands.

- [major][new] §Budgets — "`MAX_DROPPED_APPIDS = 1_024` (its breach is the named
  pull failure, §The store pull)" (+ §The store pull: "a list that would exceed
  `MAX_DROPPED_APPIDS` is itself a named pull failure", "the list merges as a
  set on delta ... and resets only on a full-pull replace") — the cap's number
  and its breach consequence together cancel the property the mechanism was
  added for, and this is the same mid-estimate error the same revision just
  corrected one line earlier for `MAX_STEAM_PAGE_OCTETS`. The arithmetic: one
  page the design calls legal can carry 50,000 dropped records — a 50,000-record
  page of 129-octet names is ~207 octets/record ≈ 10.4 MB, inside
  `MAX_STEAM_PAGE_OCTETS = 12_582_912` and inside `max_results=50000` — so a
  single page exceeds the cap ~49× and a full pull's worst legal drop count is
  12 × 50k = 600k, ~590×. And in steady state the list only grows: the refresh
  thread "delta-pull"s and never full-pulls, a restart with a fresh loadable
  snapshot *loads* rather than pulls, so a reset needs the operator to delete the
  file; drops therefore accumulate monotonically toward a hard stop. On breach
  the design returns to the posture the Revision log rejected in r0 — a named
  pull failure, i.e. a stale snapshot aging into `503` for every steam miss, and
  at startup a refusal to bind — reached by remote-chosen content, which §The
  store pull says the mechanism exists to prevent ("failing the pull over one is
  remote, permanent denial of the supply"). §Risks row 7's "impact Low" holds
  only while the cap is never breached, and the premise is unstated. Fix: adopt
  the finding above and the constant disappears; if the list stays, make
  saturation a *posture* and not a pull failure — while `dropped_appids` is
  full, no appid miss is certifiable (exactly the r1 `incomplete` posture, as
  the overflow case) — and size the cap from the page budget rather than a guess.
  Either change replaces the named pull failure; neither adds a mechanism.

- [minor][new] §Budgets — "`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k
  *worst-legal* records at ~240 octets (A17's four fields, `name` at
  `MAX_NATIVE_NAME_OCTETS`), because the pinned `max_results` can answer that
  page, not the ~150-octet mid-estimate" — the re-sizing is right for the
  mid-estimate reason and wrong for the *worst-legal* claim, because its premise
  (`name` at 128 octets) is the very thing §The store pull's drop case denies:
  names *over* the bound are what the design expects to arrive, so no finite
  page cap is "worst-legal" and a page heavy with over-bound names is legal
  content that trips a named page refusal and fails the pull. As written the
  constant is a read budget wearing a worst-case label, and the label is what a
  reader will trust when the budgets are re-derived. Fix: one clause — name it
  the response read budget it is, record that over-bound names make the true
  worst case unbounded so an over-cap page is the residual the §Risks row carries
  (the same fix as finding 2's posture), and drop the word "worst-legal". This
  is a relabel plus one risk clause, not a new mechanism.

- [nit][new] §Issued entries — "safe because the append is a set insert, so an
  already-present id is a no-op" beside "A delete whose append would breach the
  cap is the named refusal `retired id list full` and mutates nothing" — read
  literally, "would breach" tests the candidate entry, so a no-op append against
  an at-cap list refuses the very retry the same paragraph promises completes the
  removal. Fix: three words — the cap is checked against "the buffer the write
  renames", which is the check the sentence already names for other documents.

## FYI (no severity)

- Observability seat: with `incomplete` gone from health and `stats` kept at
  counts, the only signal that the name-bound posture is active is a printed
  line at pull time — potentially 1,024 appids on one line — plus reading
  `steam-snapshot.json` by hand; the r1 Risks row's "health shows the fact" was
  dropped along with the field.
- Compatibility / ledger seat: Rollout 1 now adds an `amended-by` line to
  `027-C3`, a unit of the ADR this run is implementing, to carry
  `dropped_appids`; that is a design amending its own source contract inside the
  run that implements it. The clamp route needs no such line.
- Correctness seat: finding 2's breach path converts a background delta failure
  into "`503` for every steam miss" once `027-B2`'s age bound passes; my read is
  that the posture choice, not just the number, is what needs the round.

## Checklist disposition

1. Reuse — flagged: the shipped `friendly_name` clamp is the unused in-repo
   answer to the over-bound-name class (finding 1).
2. Goal trace — `dropped_appids` traces to the resolver Goal, but its breach
   posture and its `503` outcome cut against `027-S1`/`027-L3` (findings 1, 2).
3. Smaller design — sketched in finding 1 (one normalization sentence for a
   field + constant + merge rules + four loader refusals + a policy branch).
4. New dependencies — none introduced by the r2 text; `rustls`/`webpki-roots`
   posture unchanged and proportionate as reviewed in r1.
5. Deferrability — unchanged; Rollout order still ships thin slices.
6. Knobs — `MAX_DROPPED_APPIDS` is a knob whose consumer is a failure path that
   contradicts its own section (finding 2); `MAX_STEAM_PAGE_OCTETS`'s stated
   basis is mislabeled (finding 3). The r2 `STEAM_PULL_DEADLINE_SECONDS` grid,
   `MAX_RETIRED_IDS_OCTETS` id count, and the delta two-way merge all close.
7. Value — proportionate overall: retiring the caching mode while adding a
   snapshot + mint + retired-id story is a fair trade, and the r2 lock-order,
   `query`-as-stopped-instance-tool, and deadline closures are pure simplifiers.
   The one disproportionate part is the name-bound subsystem, which is both
   larger and less available than the shipped clamp.
