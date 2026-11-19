# Code review — testing seat — round 1, dissent pass c1

**Scope:** `git diff 06d528febd4bd7d8b875553668b4ebab6a3bd097..HEAD -- ':(exclude)docs/designs'`
(20 files, +1777/−155), read against the full current `02-proposal.md` and
`docs/adrs/026-registry-service.md`. **Inputs:** that diff, the proposal, ADR-026, the
runner's gate record, and this seat's round-1 findings. No review report or transcript was
opened. **Pass type:** dissent — the revision was written to answer this seat's round-1
findings, so those are checked first; new findings are limited to what the changed code makes
true.

**Checklist:** items 1–7 applied. Items 1 (Test-strategy citation coverage) and 2 (unit +
integration layers) are unchanged in shape from round 1 and stay satisfied; the revision adds
legs at all three layers rather than only one. Item 3 is the ledger below. Items 4–7 produced
the open findings.

## Gate evidence

Runner-supplied (gate at 2026-11-19T23:01:03.401Z, all four legs exit 0): `npm run check`,
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --locked`. The excerpt carries one `test result:` line reading
`ok. 0 passed; 0 failed`, which is a single test-less target's tail, not the workspace total —
so the exit code is the citable green signal and I re-ran the legs my findings turn on:

| check I executed | result |
| --- | --- |
| `cargo test -p bindforge-registry --bins` (78 unit + in-process `src/wire.rs` legs, incl. the new corpus/cache/serve-gate/wire legs) | `ok. 78 passed; 0 failed` |
| `cargo test -p bindforge-registry --test candidates_join -- --exact the_octet_cap_trims_a_wide_join_to_a_stable_prefix_under_the_response_budget` | `ok. 1 passed` |
| `cargo test -p bindforge-registry --test caching_policy` (5 consecutive runs) | `ok. 11 passed` every run |
| `cargo test -p bindforge-registry --test admin_auth` | `ok. 12 passed` |
| `cargo test -p bindforge-registry --test cli_end_to_end` (and the explicit-port leg alone) | `ok. 8 passed` / `ok. 1 passed` |

No leg I ran failed; no required gate is red or missing.

## Round-1 ledger (checklist item 3)

1. **[major] Nothing pins the answer against `MAX_RESPONSE_OCTETS` — CLOSED.**
   `corpus::response_bounded` (src/corpus.rs:84) trims the join to the largest prefix that
   fits both A1 caps and `corpus::candidates_body` (src/corpus.rs:102) is the **single** answer
   seam — `serve.rs:301` is its only caller on the serving path, so root and caching answers
   and both provenance branches go through it. Three legs now pin it, at the layer each claim
   needs: the unit seam (`src/corpus.rs:579`, which first proves the untrimmed 256-record join
   really is over the cap, so the leg cannot pass vacuously), the in-process wire face
   (`src/wire.rs:606`, single decimal `Content-Length` equal to the body, stable bytes across
   two requests), and the subprocess face (`tests/candidates_join.rs:194`), which restates the
   budget as its own `const MAX_RESPONSE_OCTETS: usize = 65_536` rather than importing the
   server's constant — so a widened server constant fails that leg instead of moving its goal
   posts. All three assert the octet cap *binds below* `MAX_CANDIDATES` and that the window is
   the `game_id`-ordered prefix, which is the stability property my finding asked for. The
   arithmetic is now stated beside the cap in `schemas/catalog/v1/spec.md` §The answer's
   budget (lines 119–135), including the two failure modes I named. Verified green by running
   the subprocess leg. The cache side is closed too: `Cache::stage` truncates to
   `MAX_CANDIDATES` (src/cache.rs) and a stored entry over the cap is a named load refusal,
   each with its own unit leg.
2. **[minor] Fresh-cache legs had a one-second freshness budget spanning subprocess start —
   CLOSED.** `tests/caching_policy.rs:156` now runs on the default `026-B3` TTL (no
   `--refresh-interval`), so a seed stamped before the spawn is honest; `:333` deletes the
   pre-stamped seed entirely and uses the entry the instance caches on its own first lookup, so
   `fetched_at` is a server-side stamp and the 1 s override is the only wall clock in play.
   Five consecutive runs of the target: green every time. The residual sub-second boundary
   (an entry with `ttl = 1` is fresh only inside the same integer second, `is_fresh` at
   src/cache.rs:40) is now spanned by one loopback round trip rather than a spawn and a port
   poll; I accept that as the cost of legging expiry at all.
3. **[minor] The leg named for probe re-timing never counted probes — PARTIALLY CLOSED,
   re-raised narrowed** as finding 1 below.
4. **[nit] `MAX_CATALOG_DOCUMENT_OCTETS` had no fixture and was not named in the contract's
   refusal roster — CLOSED.** `schemas/catalog/v1/spec.md` now carries a roster row for it that
   says plainly that no fixture ships and names the two suites that pin the refusal; the
   generated over-cap corpus leg (`tests/catalog_document.rs`) is the pin. The fixture harness
   enumerates `fixtures/invalid/`, so the new fixture
   (`record-entry-outside-closed-set.json`) is consumed by the #1-cited golden leg rather than
   being decoration.
5. **[nit] `REQUEST_READ_TIMEOUT`/`RESPONSE_WRITE_TIMEOUT` are unlegged knobs — OPEN,
   unchanged.** Re-raised as finding 2. The revision did add a *second* timeout knob
   (`UNREAD_BODY_DRAIN_TIMEOUT`, src/constants.rs:53), but that one is at least traversed by
   the two credential-before-body legs.
6. **[minor] No leg started an instance on an explicit non-zero port — CLOSED.**
   `tests/cli_end_to_end.rs:520` is the leg I asked for: a port learned from a throwaway
   loopback bind, `--port <that port>`, the printed value asserted equal to it, then `/v1/health`
   and the join dialled on it. The harness's `wait_for_listener` connects to the *printed* port,
   so an instance that binds elsewhere than it prints panics rather than passing. Ran it green.

## Open findings

1. **[minor] No leg proves the reachability probe re-fires in a later TTL window — the leg
   named for re-timing still counts only the instance's first-ever probe.**
   *Location:* `apps/registry/tests/caching_policy.rs:333` (capture at `:375`, assertion at
   `:405-411`); mechanism at `apps/registry/src/state.rs:337-340` (probe returns early while the
   cache is empty) and `:266-271` (a fresh-cache answer returns before `maybe_probe`); the other
   probe leg is `apps/registry/tests/upstream_client.rs:314-355`.
   *Problem:* the reshaped leg seeds no cache document, so the cold first lookup never probes
   (empty cache), the warm lookup answers from the fresh cache and returns before `maybe_probe`,
   and `probes_before` is therefore always `0`. The one probe the leg then counts is the first
   probe that instance ever paid for. `upstream_client.rs:343` measures a single default-TTL
   window and so proves only "at most one per TTL". That leaves my round-1 mutant alive: replace
   `probe_is_due` with "`last_probe_at.is_none()`" — probe once, cache the verdict forever, never
   re-probe after expiry — and every leg in the suite still passes: `upstream_client.rs:314`
   still sees exactly one probe across five lookups, and `caching_policy.rs:333` still sees
   exactly one across the wait. The consequence is the one `026-C6` exists to prevent: an
   instance whose upstream went dark keeps answering `health.upstream: ok` indefinitely while its
   cache goes stale, and nothing on the run record distinguishes that from the pinned behaviour.
   Test strategy #6's clause ("re-times the once-per-TTL upstream probe") and the leg's own name
   both promise the half that is still unpinned.
   *Fix:* in the same leg, after the `:405` assertion, `sleep(1_500)` once more and issue a third
   lookup, then assert `upstream.lines_for("GET /v1/health").len() == 2` (and
   `lines_for("GET /v1/candidates").len() == 3`). Two probes in two windows is what turns "at
   most one per TTL" into "one per TTL"; keep `upstream_client.rs:314` for the ceiling half.
2. **[nit] The two low-and-slow timeouts are still wired, named and unlegged.**
   *Location:* `apps/registry/src/constants.rs:91,95`, exposed at `src/http.rs:480-488`, applied
   at `src/serve.rs:119-120`.
   *Problem:* unchanged from round 1 — no leg observes either value, and no seam lets one run at
   a testable scale (checklist item 7: a config knob with no test-side consumer is a finding, not
   a pass). `tests/admin_auth.rs:357` bounds its own 20 s socket timeout, which is a leg about
   the credential gate, not about `REQUEST_READ_TIMEOUT`.
   *Fix:* either give the read timeout a test seam (`--request-read-timeout <seconds>`, or a
   constants override in `serve.rs`'s unit tests) with a leg asserting a partially-sent request
   is closed, or record both knobs as deliberately unlegged in `03-build-notes.md` the way the
   run has recorded other accepted deviations.
3. **[nit] The new persist leg's doc claims it would catch a dropped `sync_all`; it cannot.**
   *Location:* `apps/registry/tests/persist_and_concurrency.rs:348` (doc comment), the behaviour
   at `apps/registry/src/persist.rs:220`.
   *Problem:* the leg is a good leg for what it actually observes — after each `2xx` the renamed
   document holds the answered record at the bumped `revision` with no tmp beside it — but its
   comment says a refactor that "drops the `sync_all`-then-rename ordering … is what this leg
   stops from silently surviving". Nothing observable from a subprocess seam distinguishes
   `write_all` → rename from `write_all` → `sync_all` → rename: delete the call and the leg still
   passes. The addition itself is fine; the run record should not credit it with coverage it does
   not have (checklist item 7).
   *Fix:* restate the comment to the observable half (answer-after-rename) and mark the
   durability step as deliberately unlegged, since proving it needs a crash harness this suite
   does not have.
4. **[nit] The octet arithmetic the trim is justified by is pinned only for names that need no
   JSON escaping.**
   *Location:* `apps/registry/src/corpus.rs:81-82` and `schemas/catalog/v1/spec.md:129-135`
   against `apps/registry/tests/candidates_join.rs:194` and `apps/registry/src/corpus.rs:579`.
   *Problem:* both texts state that a candidate "costs at most ≈283 octets" and conclude from it
   that a non-empty join "always keeps at least one" (and put the count crossover "near 231").
   The bound that produces 283 is the raw `name` length; `record_from_value` bounds `name` by
   octet count alone (src/corpus.rs:424), so a 128-octet name of quote, backslash or escaped
   control characters re-serializes to ≈768 octets and one candidate costs ≈918. The conclusion
   survives (918 is nowhere near the 65,536 body cap, so the trim can never empty a non-empty
   join), but the figure every reader takes as the reason does not, and no leg generates an
   escape-bearing name at the bound.
   *Fix:* state the escaped worst case beside the figure (raw ≈283, escaped ≈918), or add one
   leg with 256 max-bound records whose `name` is `"`.repeat(128) asserting a `200`, a non-empty
   body, `body.len() <= 65_536` and `kept >= 1`.

## FYI (no severity, out of this seat's specialty)

- `apps/registry/src/serve.rs:299-301` says "`query` prints the same projection it answers with,
  so no second copy of the arithmetic exists". `main.rs:291-307` prints every candidate from
  `state.candidates(...)` with no octet trim, so for a trimmed join `query` prints more lines
  than the face answers with. The absence of a second arithmetic copy is true; the "same
  projection" sentence is not.
- `http.rs:383`'s `.min(UNREAD_BODY_DRAIN_OCTETS)` cap is not distinguished by any leg: the two
  head-gate legs declare more than the cap and send nothing, so they exercise the drain *timeout*
  rather than the discard ceiling. Nothing is buffered either way, so the mutation that survives
  (raising the cap) has no memory consequence.
- `schemas/catalog/v1/spec.md`'s new closed-set roster row names one fixture for two conditions
  (record level shipped, document level refused "by the same gate" with no fixture) — the same
  family as round-1 finding 4, which is otherwise closed.

## Verdict

**approve.** My round-1 major is closed with three legs at three layers plus the contract
arithmetic; both round-1 minors are closed, one of them (probe re-timing) only halfway and
re-raised narrowed as a minor. What remains open is one minor and three nits, none of which
leaves a required property of the shipped change unestablished.
