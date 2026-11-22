# Complexity review r1 (dissent pass — changed sections only)

verdict: request-changes

Scope: `git diff 7b5cec46a35f016… fcb66a83bd31cc82…` (845 lines of `02-proposal.md`),
then the full current document, plus `docs/adrs/027-…md` and the Revision log.
Inputs were exactly those; no run record other than my own r0 findings (as
injected by the caller) was opened.

Checklist disposition: 1 reuse — the revisions add no new module or abstraction;
`incomplete`, `write.lock` and the split retry knobs all reuse shipped machinery
(`persist::read_capped` metadata gate verified at `apps/registry/src/persist.rs:251`,
reused `026-C7` tolerances at `constants.rs:68`, `corpus.rs` loader reused for the
issued document). 2 goal-trace — every new moving part traces to a named unit
(`027-L2` retired posture, `027-L3` certification, `027-B4`, `027-C5`'s bound
clause); nothing new is untraceable. 3 smaller design — not re-argued at this
scope; the changed text does not foreclose the deferrals §Deferred keeps.
4 dependencies — no new dependency beyond the accepted `rustls`/`ring`/
`webpki-roots` trio, now honestly described as a C/asm build-script cost with the
CI leg narrowed to pay for it. 5 sliceable — Rollout 2/3/4/5 ordering still keeps
each slice thin; nothing in the revisions welds the halves together.
6 knobs — `STEAM_PAGE_RETRY_SECONDS`, `STEAM_PULL_DEADLINE_SECONDS` and
`STEAM_RETRY_SECONDS` each have exactly one named consumer (§The store pull,
§Budgets, Test #6/#11), which closes my r0 finding 3; no operator-facing knob was
added. 7 value — see the value paragraph below.

## Findings

- [major][re-raised] §Issued entries — "The mint rule", step 5 — "`steam-<appid>`:
  appid-unique, ≤ 26 octets under the closed key grammar's appid bound" — the
  bound this clause cites does not exist in any grammar the document or the repo
  states. The registry's whole key rule is `apps/registry/src/keys.rs`
  (`key_is_admissible`): a 128-octet cap on the joined key plus
  `Source::spells_lookup_key` (`crates/discovery/src/game.rs:84`), which is
  "shape only" with a *non-empty* external id — no decimal requirement, no
  appid-length rule; a 122-octet `steam/<external-id>` is admissible (the shipped
  test pins exactly that: `format!("steam/{}", "a".repeat(122))` admitted). The
  only appid-digit bound in the tree is `MAX_APPID_DIGITS = 20`
  (`crates/discovery/src/steam_common.rs:26`), a *private* const gating the
  client-side ACF emission seam — not the grammar, and not readable by
  `apps/registry`. The snapshot's own rules refuse only "an `appid` that is not a
  decimal integer", so a decimal appid of any length fits a 32 MiB file and loads.
  Consequence: `steam-<appid>` is 7 + digits octets, so the rule's terminal case
  is not provably ≤ `MAX_SLUG_OCTETS = 128` (a 122-digit appid → 129), and
  Test strategy #1's pinned invariant "every output satisfies `is_slug_name` ∧
  `≤ MAX_SLUG_OCTETS`" is still not guaranteed by the rule — the same unsound
  by-construction arithmetic I raised in r0, now narrowed to the substitution's
  own justification: r1 fixed the rule's shape (step 5 exists, the "by
  construction" claim is deleted, the held ∪ retired check on the suffix is added)
  but rests its one remaining length claim on a bound nothing states. A minted id
  over the bound is refused by the reused `026-C3` loader
  (`corpus.rs:390 game_id_valid`), so the failure is a load refusal of the issued
  document or a per-mint `503`, exactly as in r0. Fix: make the bound the
  design's own named rule instead of a citation — add an appid digit cap (the
  20-digit figure `steam_common.rs` uses, restated as this design's budget) to
  §The store pull's record rules and to the loader's named refusals, alongside
  the existing "not a decimal integer" refusal it shares, then keep step 5's
  "≤ 26 octets" as arithmetic over that stated cap. This replaces the phrase
  "under the closed key grammar's appid bound" with one integer compare that the
  pull and the load already perform on the same field; it adds no new stage.
  Alternative of equal size: drop the numeric claim and make step 5's terminal
  case "an id still over `MAX_SLUG_OCTETS` after substitution is the named `503`",
  which is sound without any appid bound at all.

- [minor][new] §Issued entries — "Retired ids" — "**Every** delete — issued or
  corpus — is two persists under one mutation-lock hold, retired first" — this
  sentence is the only place in the design where on-disk state can be
  half-applied, and the state it creates is unnamed. If the process dies between
  the two renames, the id sits in `retired-ids-v1.json` *and* still stands in its
  catalog document: `games ∩ retired ≠ ∅`, a condition no rule governs — the
  mint's steps 4/6 ask "held **or** retired by a different record" and the loader
  refusals list neither the overlap nor its absence. And the recovery path is
  exactly the named `503` retry: re-issuing the refused delete re-appends an id
  the list already holds, and the document never says whether that is a no-op or
  a second entry in a file defined as strictly "in byte order" (binary-searched
  membership, per the same paragraph). Test strategy #5 pins the append ordering
  and the cap refusal but not this half-applied state. The r0 text this replaced
  ("in one mutation-lock persist") had no such window, so the defect is the
  revision's. Fix: one sentence plus one branch — the retired-list insert is a set
  insert (already present ⇒ no-op, since a duplicate breaks the byte-order
  invariant the paragraph itself fixes), and `games ∩ retired` is defined as the
  legal, recoverable state it is: the catalog record keeps answering, the id is
  held-and-retired, and the operator's retried delete completes the removal. No
  new mechanism, no new file; it replaces the silence about the retry.

- [nit][new] §Budgets — "`STEAM_PAGE_ATTEMPTS = 3` separated by
  `STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS = 900`" —
  the worst-case attempt grid the three knobs describe (12 pages ×
  (3 × 30 s gaps + 32 s of connect+read timeout) = 912 s) overshoots the deadline,
  so a fully degraded page sequence can never execute the retry plan the other two
  constants state; the deadline fires first. Nothing breaks — the outcome is the
  named refusal the section wants, and the §Risks coupling I raised in r0 is
  genuinely gone — but the budget family is this design's one readable statement
  of the pull's worst case, so it should close. Fix: either set the deadline to
  cover the grid (960) or add half a clause to §The store pull: "the deadline may
  cut a page's remaining attempts".

## FYI (no severity)

- The client-facing consequence the revisions surfaced — a snapshot past `027-B2`'s
  age bound (or `incomplete` > 0) turns a bound CLI's whole `discovery list` into
  a refused closed scan (`025-C4`), so the operator's freshness is user-visible
  availability — is now stated in §Compatibility matrix, §Risks and Rollout 6.
  Correctly stated, but it is availability/correctness surface, not this seat's.
- `stats` carries no retired-id count while `MAX_RETIRED_IDS_OCTETS` is enforced
  by a delete-time refusal; §Deferred records the count's absence. The cap is
  effectively unreachable at one-id-per-call admin deletes, so the blind spot is
  proportionate — noted only so another seat can weigh it.
- §The hosted lookup policy step 5's three sub-bullets can match at once
  (`incomplete` > 0 *and* past the age bound) and answer two different named
  `503`s; precedence is unstated but the status is `503` either way, so no seat
  need act on it.

## Value check

Proportionate. The revisions add one snapshot integer, one lock file, one refusal,
two retry constants and one CI-leg narrowing, and each buys back a named posture:
the supply survives attacker-chosen names instead of aging into permanent `503`,
`query`-beside-`serve` becomes a refused state instead of a silent
`027-L2` hazard, and no pre-bind path sleeps an hour. Nothing in the changed text
is a knob without a consumer or an abstraction with one implementation. The one
place where the design now claims more than its own arithmetic supports is the
mint's terminal length claim, which is the major above.
