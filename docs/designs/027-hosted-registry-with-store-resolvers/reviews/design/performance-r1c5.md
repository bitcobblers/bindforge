# Performance review r1 (c5 ratification pass)

verdict: approve

Scope as directed: `git diff 19dcdcb170ba3f06749f7a84641e94b5839d1f77
d105d78caaf272e507760abf17dedfdd966810b6` (the post-r1 revisions), then the full
current `02-proposal.md`. Inputs: the proposal, ADR-027, its Revision log, my r1
findings. Checklist `references/performance.md`, seats 1–6.

## What this diff does, performance-wise

The diff is a deletion, not an addition: the whole `dropped_appids` bookkeeping
subsystem — the snapshot field, `MAX_DROPPED_APPIDS = 1_024`, its delta set-merge
and reset rules, its loader refusal, its `steam snapshot incomplete` policy
branch, its `503` posture, its Test-strategy cases and its Rollout-1 ledger
amendment — is replaced by normalization at the pull seam (a
`char::is_control()` strip, then the shipped `discovery::game::friendly_name`
whitespace-and-clamp rule), with the record and its appid kept in the snapshot
(§The store pull, §The steam snapshot resolver, §The hosted lookup policy,
§Risks, Revision log `r0→r4`).

Checks run against that, and what they established:

1. **Deletion is clean — no dangling references.** Grep of the current document
   for `dropped_appids` / `MAX_DROPPED` / `snapshot incomplete` returns exactly
   one hit, the Revision-log entry that records the deletion. §Budgets, §The
   steam snapshot resolver's loader list, §The hosted lookup policy step 5, Test
   strategy #2/#3/#6/#13 and Rollout 1/3 all read consistently without it. No
   section left cites a constant or a refusal that no longer exists — the usual
   regression mode for a subtraction this size, and the one that would have
   broken what I approved.
2. **The replacement bounds what the deletion un-bounded.** Because every record
   the pull is handed now stays in the snapshot, the snapshot's per-record stored
   size is what keeps `027-B1`'s `MAX_STEAM_SNAPSHOT_OCTETS = 33_554_432`
   publish-time check reachable. I verified the clamp the design now leans on:
   `crates/discovery/src/game.rs:102-113` `friendly_name` collapses
   `split_whitespace` then takes "the longest `char`-boundary prefix inside
   `bindforge_core::MAX_NATIVE_NAME_OCTETS`" — so every stored `name` is ≤128
   octets by construction, and A7's amended wording (that rule drops no
   non-whitespace control byte, which is why the strip is the design's own)
   matches the code. A 200k-record catalog at the clamp is ≈30 MiB of document,
   inside the 32 MiB ceiling: the posture the r1 design bought with dropping is
   now bought with the clamp, at the same bound and without the cap breach.
3. **Boot-load arithmetic still holds after the constant's removal.**
   `MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216` (`apps/registry/src/constants.rs:27`),
   so §Budgets' "144 MiB of JSON (16 corpus + 64 issued + 32 retired + 32
   snapshot) as the boot load ceiling" is right, and the deleted
   `MAX_DROPPED_APPIDS` was a list cap, not a document cap, so it was never in
   that sum — its removal changes neither the 144 MiB figure nor the transient
   sentence, which the diff leaves untouched.
4. **No new cost on any hot path.** The added work is one control-character
   filter plus one clamp per record, on the pull thread only (≤50k records per
   page, ~200k per full pull), against a body already being JSON-parsed; the new
   observability is one count on the publish line. The request path, the
   candidate budgets, the issued/mint write path, the §Concurrency lock scope and
   the delta merge's "one linear two-way merge plus one index build" are all
   outside the diff's changed lines.
5. **Checklist 5 (invalidation) improves.** The mechanism I flagged for a
   permanently sticky, self-referential breach state no longer exists; nothing
   new was introduced in its place that needs invalidating.
6. **Checklists 1–4, 6**: no per-event cost added on the serving path; no new
   unbounded structure; no new startup read; no new gap against the research
   baseline. Nothing in the diff touches the A5/A17 figures my r1 arithmetic used.

## Findings

None. No blocker, no major, and no new minor or nit attributable to this diff:
the revisions did not break anything I approved at r1 and did not make any
section I cited false.

## Carry-over status (no severity, per the ratification gate)

- **r1 finding 1 (`MAX_DROPPED_APPIDS` breach re-importing remote denial of the
  supply) — closed by deletion, not by my proposed fix.** The author rejected
  the smaller remedy (degrade to the whole-list posture) and deleted the
  subsystem instead (Revision log `r0→r4`, reasons stated: the clamp makes it
  redundant; its cap breach returned the remote-chosen denial; a listed appid
  could never be answered against `027-S1`). The rebuttal is sound and the
  deletion is complete — I verified points 1 and 2 above. Nothing to re-raise.
- **r1 finding 2 (§Budgets transient family omits the full pull; 12 × 12.58 MiB
  admissible read against the 33,554,432 publish ceiling) — unchanged, still
  open at minor.** The diff re-words `MAX_STEAM_PAGE_OCTETS` (now "the response
  *read* budget … not a worst case"), which keeps the read envelope exactly as
  large as when I priced it; the sentence that enumerates the writer transients
  ("on top of it a mint holds one model plus one serialized buffer, and a delta
  merge the old snapshot, the merged model, one serialize buffer and the current
  page body") is not in the diff at all. Not reportable this pass — nothing in
  the changed text made it newly true or newly worse.
- **r1 finding 3 (the 32 s grid charges timeouts as page bounds; ≤1,920 s
  pre-bind block unnamed) — unchanged, still open at nit.** Neither
  `STEAM_CONNECT_TIMEOUT` / `STEAM_READ_TIMEOUT` nor the
  `12 × (3 × 32 + 2 × 30) = 1,872` line nor the "sized in §Budgets to cover this
  grid" sentence is touched by the diff (the shipped precedent at
  `apps/registry/src/upstream.rs:97` still binds a socket read, as I described).
  Not reportable this pass.

## FYI (no severity)

- With the drop gone, an appid whose store name is over the `013-B4` bound moves
  from "instant named `503`" to "eligible for the mint path on first query", so
  the population that can trigger an issued-document rewrite under the mutation
  lock grew by whatever over-bound-name appids clients actually query. The
  per-mint rewrite is already priced (`High`/`Medium` write-amplification row),
  the count is bounded by `MAX_ISSUED_ENTRIES`, and the same row's
  "the cost is a shorter label, never a different identity" is a deliberate
  framing choice rather than an arithmetic error — flagging only so the Rollout 4
  rewrite measurement is taken against a record population that includes these.
  Mine to raise, but under the ratification gate it is not a finding.
