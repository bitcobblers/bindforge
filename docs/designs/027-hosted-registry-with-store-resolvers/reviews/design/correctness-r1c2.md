# correctness review r2 (dissent pass, c2)

verdict: approve

Scope: `git diff fcb66a83 19dcdcb1` on `02-proposal.md`, then the full current
document. Inputs: the proposal, ADR-027, its Revision log, my r1 findings.
Checklist applied: `references/correctness.md`. Spot-checks run this pass:
the pinned authority's parameter table (`partner.steamgames.com/doc/webapi/
IStoreService` — games-only by default, `last_appid`/`max_results` with
default 10k / max 50k, `if_modified_since` filtering "changed or added",
`last_modified` and `price_change_number` on items, no response schema) —
A3 and the request rules hold; and the shipped code the r2 arithmetic now
cites: `apps/registry/src/keys.rs:73` (`key.len() > MAX_DISCOVERY_SLUG_OCTETS`
on the *joined* key, token set + path safety only — no appid bound),
`crates/core/src/placement.rs:83` (`MAX_SLUG_OCTETS = 128`),
`apps/registry/src/constants.rs:68,72` (the `026-C7` tolerance pair).

## Status of my r1 findings

- **major (whole-snapshot `incomplete`)** — **closed for real.** `dropped_appids`
  is exactly the record-scoped mark the r1 fix asked for; policy step 5's
  second/third bullets now certify `200 []` for every appid the pull actually
  examined, the Options "no partial list is ever certified" flag is gone, and
  the global `503` family collapsed to the listed appids. The residual
  aggregate case (>1,024 breaches) is a budget-amendment matter at 1,024× the
  r1 trigger and is now named as a failure; not re-raised.
- **minor (cumulative vs per-pull mark)** — **closed**: "the list merges as a
  set on delta … and resets only on a full-pull replace, so it is
  self-clearing", with Test #6 pinning it and §Deferred naming it as the
  amendment signal.
- **minor (`027-C3` shape / health fact)** — **closed**: health ships
  `027-C3`'s `{games, fetched_at}` verbatim, "exactly" is gone from the
  document sentence, and Rollout 1 carries the `027-C3` `amended-by` line for
  the one added fact.
- **minor (lock order)** — **closed**: the order names the write direction and
  the delete's two swaps are sequential and "never held together", with the
  corpus write acknowledged as the single departure.
- **minor (invented appid bound)** — **closed** on the citation: `MAX_
  DISCOVERY_SLUG_OCTETS` is the real whole-key cap and I verified there is no
  appid bound, so 6 + 122 = 128 is right; the named `mint identity over bound`
  refusal covers the unit seam. A *new* problem in the same sentence is my
  finding 2.
- **minor (transport-assurance seam)** — **closed**: the seam is named
  (address and verification name separately, authority still compile-time-
  pinned) and the self-signed cert now carries the verified name, so the case
  fails on the chain.
- **nit (deadline under the grid)** — **closed** arithmetically
  (`12 × (3 × 32 + 2 × 30) = 1,872 ≤ 1,920`), with a consequence noted below.
- **nit (page octet cap)** — **partly**: re-derived for worst-legal records at
  the octet bound, but the derivation still misses JSON string escaping
  (finding 3).
- **nit (`query` invocation)** — **closed**: §Hosted mode names it a
  stopped-instance tool and Rollout 6 carries the invocation.

## Findings

- [minor][new] §The steam snapshot resolver / §The store pull — "`name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound `027-C3` names — the pull never writes one, §The store pull)" — the parenthetical is asserted in the changed text but no pull rule backs it. §The store pull's response refusals are non-`200`, `Transfer-Encoding`, `3xx`, over-`MAX_STEAM_PAGE_OCTETS`, wrong envelope, page-budget, non-decimal `appid`; "The one *dropped*, not refused, case is a record whose `name` breaks the `013-B4` bound", and per A10/`027-C3` that bound *is* `MAX_NATIVE_NAME_OCTETS = 128` — a control byte is the loader's separate refusal. So a pulled record whose name carries an ASCII control byte is neither refused nor dropped: it is published, and the next load refuses the file — "a present-but-unloadable snapshot that the pull then cannot replace is a named startup refusal" does not help, because the pull *can* replace it and re-publishes the same unloadable bytes: the run serves from memory and every later restart is a named refusal unless the operator reaches `--no-steam`. That is the same remote-content-into-availability-cliff class this pass was meant to clear, one rule away from the drop rule that now exists. Fix: widen the one dropped case to "a record whose `name` breaks the `013-B4` octet bound **or carries a control byte**" (the two refusals the loader already names), so the claim in the parenthetical becomes the rule it points at.
- [minor][new] §Issued entries, mint rule step 5 — "mints `steam-<appid>`, spelled from the queried key's external id … appid-unique" — spelling the fallback from the *client's* digits makes the id depend on the queried spelling of one appid. `steam/42` and `steam/0042` are both admissible (`keys.rs` bounds only length, token and path safety), and the design never pins how the snapshot's "one exact-`appid` probe" treats them: numeric match → the same snapshot record mints `steam-42` and `steam-0042`, two held ids for one game, so "appid-unique" is false and the step-4 suffix has no stated appid source (and `mint.rs::mint_id(display_name, appid, held)` carries one `appid` for both jobs); textual match → `steam/0042` is not present in the snapshot and takes the second bullet, answering a false `200 []` for an appid the catalog does hold — the claim `027-L3` exists to police. Fix: name the canonical spelling in step 5 and at the probe — spell the fallback from the snapshot record's `appid` (or refuse a non-canonical decimal external id before the probe, saying which) — and keep `MAX_DISCOVERY_SLUG_OCTETS` as the arithmetic, which is already verified correct.
- [nit][re-raised] §Budgets — "`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k *worst-legal* records at ~240 octets" — the r1 gap narrowed but the number is still not a worst-legal bound: `~240` costs the name at its 128-octet *decoded* bound, while the body carries JSON string escapes, so a name made of quotes or backslashes serializes at up to 2× and a record lands near 340 octets — a legal 50k page of such names is ~17 MiB, over the cap, and the design must refuse it and fail the pull (a startup refusal, on a legal response). Fix: one clause saying which the cap is chosen for — either double the name allowance for escapes (⌈2 × 128⌉ in the derivation) or pin `max_results` from the worst-escaped record — as r1 asked, "and say which the number is chosen for".
- [nit][new] §Hosted mode / §Risks / §Deferred — "`dropped_appids` is not a health fact — the pull writing the list prints it" beside "its `appid` is listed in the snapshot's `dropped_appids`, which is the operator's signal to pursue that amendment" — with the fact off `health` (rightly, per `027-C3`) and off `stats` (Deferred), the operator's only channels are one stdout line at publish time and opening the snapshot file, and Rollout 6 enumerates "reading the retired list's size against its cap" but never the dropped list — so the signal the design leans on twice has no runbook line and no restart-surviving surface. Fix: add the dropped list to Rollout 6's reading duty in the same clause as the retired-list size (the file and the publish line are already the named surfaces); no new surface needed.
- [nit][new] §The store pull — "sized in §Budgets to cover this grid with its retries, so a slow store costs a late snapshot, not a guaranteed failure" — true for the background delta, not for the pre-bind full pull, where the same 1,872 s grid ends in `027-C3`'s named startup refusal after ~31 minutes of a process that has not bound. This pass deleted the sentence that used to bound that path ("the pre-bind full pull is paced by the socket timeouts and a seconds-scale retry, never an hour-long sleep") while raising the deadline from 900 s to 1,920 s, so nothing now states what a slow store costs an operator's start. Fix: scope the clause ("… costs a late delta; a full pull that runs the grid is the named `steam pull deadline` startup refusal") and keep the deadline value — the grid arithmetic is right, only the sentence's claim is too wide.

## FYI (no severity)

- §The store pull's delta rule has one state the two-way merge leaves visible: a rename that *breaks* the bound on an appid the snapshot already holds drops the delta record (so the appid joins `dropped_appids`) while the old within-bound record stays in `apps`, and policy bullet 1 answers from it — the mark is inert and the stale name is the accepted "names freeze" posture. Harmless as written; worth one clause if the reviser wants the file invariant (`apps` ∩ `dropped_appids` = ∅) stated.
- `mint identity over bound` is unreachable through the serving face (external id ≤ 122) but reachable at the unit seam, where `mint_id` takes an `appid` directly; Test strategy #1's case for it is therefore a unit-only assertion. Fine as it stands.
