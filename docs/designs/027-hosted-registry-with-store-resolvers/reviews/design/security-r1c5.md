# security review r1 (c5 ratification pass)

verdict: approve

Scope: the blob diff `fcb66a83 → d105d78c` of `02-proposal.md`, then the full
current document. Ratification only: checking that the post-r1 revisions did
not break what the security seat approved in r1, per the caller's narrow
scope. Inputs: the proposal, ADR-027, the Revision log, and my quoted r1
findings; code verified with read-only inspection only.

## Findings

None. No new blocker, major, minor, or nit findings in the changed text.

## Status of the seat's r1 findings

- **r1 minor 1 (control-byte half of the loader's name rule unbacked by any
  pull rule) — closed, and closed more strongly than my proposed fix.**
  The revision deletes the drop rule entirely and moves normalization to the
  pull seam: a `char::is_control()` strip, then `discovery::game::friendly_
  name` (§The store pull). I verified the seam's claims against the tree:
  `crates/discovery/src/game.rs::friendly_name` is `pub` under `pub mod
  game` (no crate-root re-export, matching A7's amended wording), and it is
  exactly whitespace collapse (`split_whitespace`/join) plus the longest
  char-boundary prefix inside `MAX_NATIVE_NAME_OCTETS` — it does **not** drop
  NUL/ESC/DEL/C1, which is precisely why the design keeps its own strip
  first (A7 and the r0→r4 Revision-log entry say so; my r1 finding's exact
  hole is the named rationale). The order (strip, then clamp) makes the
  loader's two name refusals (`> MAX_NATIVE_NAME_OCTETS`, control bytes)
  unreachable for any name the pull writes, so the "the pull normalizes one
  before writing it" parenthetical in §The steam snapshot resolver is now
  backed by a rule. The mint-contamination half is also closed: the issued
  document is the `026-C3` shape and I verified `apps/registry/src/
  corpus.rs::validate_record` and `schemas/catalog/v1/spec.md` refuse `name`
  only on the 128-octet bound — a clamped (or empty) normalized name always
  passes its own loader, so no store-chosen name can mint a record the next
  boot refuses.
- **r1 minor 2 (delta `incomplete` carry-forward unpinned) — mooted by
  deletion.** The `incomplete` field, the `dropped_appids` variant, the
  `steam snapshot incomplete` policy branch and the health field are all
  gone; the snapshot now ships ADR-027 `027-C3`'s shape verbatim
  (`{version, fetched_at, apps}`, matching `docs/adrs/027-…md` `027-C3`,
  which the r1 text had deviated from by adding a field). Nothing is dropped
  ("Nothing is *dropped*", §The store pull), so the dishonest-worldlist-
  regains-certification scenario my finding described has no state to
  occur in; the policy's strengthened claim ("Every page was examined, none
  skipped, and every record the pull was handed is in it") is true by
  construction under merge-without-drop, and I grepped the current document:
  no dangling `incomplete` reference survives outside the Revision log's
  history lines.

## Changed-text checks (all pass)

- **A7 amendment**: `friendly_name` and the module list verified
  (`crates/discovery/src/game.rs`, `lib.rs`); "no crate-root re-export" is
  accurate; `apps/registry` already depends on `discovery` (Cargo.toml), so
  the clamp adds no dependency and checklist item 5 sees no new crate.
- **Mint step 5**: `MAX_DISCOVERY_SLUG_OCTETS = 128` exists
  (`crates/core/src/placement.rs:95`) and `apps/registry/src/keys.rs` caps
  the whole joined key at it with the suite pinning a 122-octet external id
  at the cap — the cited 6 + 122 = 128 arithmetic is true, and the residual
  case is the named fail-closed `503` `mint identity over bound`. This
  replaces r1's invented "closed key grammar's appid bound" with a verified
  one.
- **Budgets**: `MAX_STEAM_PAGE_OCTETS = 12 MiB` is a read cap that fails the
  page cleanly and keeps the previous snapshot; the unbounded-true-size
  residual is now *named* in Risks rather than papered over — strictly
  better than the r1 figure I approved, which could have failed pulls over
  legal CJK/emoji titles. Deadline arithmetic `12 × (3 × 32 + 2 × 30) =
  1,872 ≤ 1,920` checks.
- **Transport assurance (Test #6)**: the test constructor separates connect
  address from verification name with the production authority still
  compile-time-pinned, and the self-signed cert now carries the verified
  name so the case fails on chain validation, not hostname. This strengthens
  the CWE-319/CWE-1428 posture my r1 pass relied on; the seam is test-only.
- **Retired-ids and concurrency rewording**: crash-between-persists overlap
  is set-insert-idempotent and recoverable; the delete's sequential model
  swaps and the named corpus-write lock-order departure are deadlock
  reasoning, security-neutral; secrets custody, no-URI-in-logs, and the
  environment key seam are untouched.

## FYI (no severity)

- §Issued entries step 5's new "spelled from the queried key's external id"
  makes the fallback id's digits client-chosen (`steam/042` vs `steam/42`
  are both admissible under `keys.rs`), while the snapshot's
  "exact-`appid`" probe's matching rule for non-canonical decimal ids stays
  unpinned — the correctness seat's call whether to anchor the spelling to
  the snapshot record's `appid`; the security-relevant bound claim itself is
  verified and fails closed.
