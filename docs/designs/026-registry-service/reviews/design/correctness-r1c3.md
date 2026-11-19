# correctness review r3 (dissent pass; scope = blob diff 07e8b3f8..1e8f240c plus the full current document)

verdict: request-changes

## Checklist disposition

1. **Assumption register** — the only register row the diff adds is `A23`
   (`verified`, `apps/discovery-cli/src/catalog.rs`). Checked: `catalog.rs:437-445`
   builds one `GET … Connection: close` request per `TcpStream`, calls
   `read_response` once, and drops the stream; no read loop anywhere in the
   client. The row's basis supports the claim. `A20`/`A21` are unchanged and
   already hedged by the paragraph at the register's foot.
2. **Research-URL spot-check** — not applicable to this pass: the diff adds no
   URL-cited row and changes none. (No new external claims to anti-hallucinate.)
3. **Internal contradictions** — two found in the revised text (findings 1, 2).
4. **Missing edge cases** — finding 1 (admitted key octets at the outbound
   hop) and finding 2 (the one-octet revision-digit window).
5. **Interfaces** — the names the diff touches are used consistently:
   `MAX_CATALOG_DOCUMENT_OCTETS` now has one rule with a read side (§Catalog
   document loader) and a write side (§admin upsert, §Budgets, #10);
   `--refresh-interval <seconds>` matches #6's `--refresh-interval 1`; the
   removed `026-S1`/`winreg` sentences leave no orphan claim (the §`apps/registry`
   pointer "retired by the deferred lift below" still lands on the lift, which
   the re-worded §Deferred bullet keeps describing; `winreg` confirmed
   `#[cfg(windows)]` in `crates/discovery/Cargo.toml`).
6. **Decides what the ADR asked** — the diff changes no open-question answer.
7. **Previous round closed for real** — yes, all four (below).

## r1 findings — disposition

- **[major, closed]** corpus-document cap had no write-time rule. The upsert
  bullet now serializes the model and names the over-cap `422` "leaving model,
  `revision` and file untouched with no tmp written" before `revision` bumps;
  §Budgets states the cap "caps a document at **both** ends of its life";
  `spec.md` is said to carry "the named refusals that enforce it on both sides";
  #10 pins it. Closed — with one residual ordering hole (finding 2).
- **[nit, closed]** cache eviction re-serializes "after each dropped entry — one
  extra pass per eviction at most", so the exact-length claim no longer pulls
  against the mechanism.
- **[nit, closed]** the persist-only lock is deleted outright and the wait is
  named as accepted ("A reader may wait on one local write, and that latency is
  accepted"), with the Revision log recording the deletion; §Deferred no longer
  carries a lock it never had.
- **[nit, closed]** the fixture-name rationale now points at "#19's
  case-unique-fixture-name claim for a case-insensitive filesystem" rather than
  at the Windows matrix row.
- The r3 space-refusal deletion is itself sound in its stated terms:
  `crates/discovery/src/game.rs::spells_lookup_key` checks only the
  `<token>/<non-empty external-id>` shape, so it does admit a space, and keys
  are "never joined onto paths". But see finding 1 for the seam the rebuttal
  does not cover.

## Findings

- [major][new] §The serving face step 1 + §Caching mode (upstream client) — "r3
  deletes the space refusal, which traverses nothing and would `404` a key the
  shipped pair admits" / "Miss with reachable upstream fetches `GET
  <upstream>/v1/candidates?slug=`" — widening the admitted key alphabet to every
  non-control, non-separator, valid-UTF-8 byte proves the keys safe against the
  *filesystem* ("never joined onto paths") but never against the *wire*, and no
  text in the design says how the caching registry spells an admitted key on the
  request line it sends upstream. The §`apps/registry` copy list is explicitly
  closed at five invariants — "the response reader (#7), the endpoint gate (#14,
  #15), the `game_id_is_valid` predicate (#8), the `<data>` rule (#13), and the
  tmp-and-rename persist (#10, #11)" — and A19's shipped `percent_encode`/
  `candidates_url` request builder is not among them, so nothing re-pins it and
  #7 pins only response refusals. Concretely: the ADR's own `local/<dir>` case is
  a directory name, and `local/Steam Big Game` is now a legal slug; a caching
  instance that fetches `GET /v1/candidates?slug=steam/Steam Big Game` sends a
  request target with a raw space, which the root's own request reader refuses as
  a framing fault → the user gets the stale/`503` path for a key their corpus can
  hold. Worse for a `%`-bearing key (also admitted: `%` is not a control byte,
  not a separator): forwarding the decoded `steam/foo%2Fbar` raw makes the
  upstream decode it a second time to `steam/foo/bar`, refuse it as a further
  separator, or answer for a *different* key — which this registry then caches
  under *its* key, a wrong answer that outlives the request. Fix: name the
  shipped `percent_encode`/`candidates_url` discipline (A19) as the sixth copied
  invariant in §`apps/registry` (and say "six" in §`apps/registry` and §Deferred
  in place of "five", since the lift now carries it too), and replace step 4's
  "`fetches `GET <upstream>/v1/candidates?slug=`, validates the answer, and
  caches it`" with "`fetches `GET <upstream>/v1/candidates?slug=`, the request
  target built from the *decoded* slug through the copied `percent_encode`
  (A19), so the bytes the upstream decodes are exactly the key this instance
  asked about, validates the answer, and caches it`"; pin it in #7: "a key
  bearing a space or `%` reaches the upstream only as its encoded spelling".

- [minor][new] §The admin face and auth — "the model with the upsert is then
  serialized whole, and one over `MAX_CATALOG_DOCUMENT_OCTETS` … is the named
  `422` … Only then does `revision` bump and the tmp-and-rename persist run" —
  the buffer tested is the pre-bump document, but the bytes renamed into place
  are the post-bump model (§Concurrency: "a mutation serializes the new model,
  writes the fixed tmp name, renames it into place"), so the persisted document
  can be one octet longer than the checked one whenever `revision` grows a digit
  (9→10, 99→100, …). A corpus that serializes to exactly 16,777,216 with
  `revision` 9 therefore passes the check and persists 16,777,217, and the next
  start refuses the whole corpus by name — falsifying the two absolutes this
  revision just wrote: "no accepted mutation can persist a corpus its own loader
  refuses" (§Budgets) and "a restart cannot lose an accepted mutation (#11)"
  (§Concurrency). Fix: order the bump before the check on the candidate model,
  checking the exact buffer that is renamed — "the model with the upsert **and
  the next `revision` value** is then serialized whole, and one over
  `MAX_CATALOG_DOCUMENT_OCTETS` … leaving model, `revision` and file untouched
  with no tmp written; only then is that same buffer written to the fixed tmp,
  renamed, and the model swapped in", and in #10 "whose serialized corpus" →
  "whose serialized corpus **with its bumped `revision`**".

## FYI (no severity)

- `MAX_REQUEST_BODY_OCTETS` = 16,777,216 with `MAX_CONCURRENT_CONNECTIONS` = 64
  is up to ~1 GiB of admitted inbound bodies on a routable bind; the diff's
  one-request-per-connection close helps the idle-slot half of that, not the
  body-size half. Performance/security seats' ground, not this one's.
- The diff's A23-backed close (`http.rs` answers one request, then closes) is
  consistent everywhere I could check: §serve.rs, §Budgets' "≤64 live threads",
  #11's over-cap refusal, and the shipped client's connect-per-query
  (`catalog.rs:415-445`); no leftover keep-alive or request-loop assumption
  survives in the document.
