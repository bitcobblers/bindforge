# correctness review r1

verdict: request-changes

Scope: whole document (panel pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/027-hosted-registry-with-store-resolvers.md`, the Revision log, and the
repository's shipped sources (`apps/registry/src/{constants,corpus,persist,state,main,keys}.rs`,
`crates/core/src/placement.rs`, `crates/discovery/{Cargo.toml,tests/retirement_tests.rs,tests/slug_tests.rs}`,
`data/catalog/v1/catalog.json`). Sources spot-checked with webfetch:
`partner.steamgames.com/doc/webapi/IStoreService`, `steamcommunity.com/dev/apiterms`,
plus third-party `IStoreService/GetAppList/v1` clients for the response shape.

Assumption register audit: A1, A2, A7, A8, A9, A10, A13, A15 verified against the
tree exactly as the rows state them (six seed records including the 32-hex `epic/`
key; `Mode::{Root,Caching}` and the six-field health list; `MAX_SLUG_OCTETS = 128`
and `is_slug_name` = `[a-z][a-z0-9-]*`; the retirement gate scanning only the two
discovery trees with `slug_tests` behind `retired-adr-024-suites`; the shipped suite
names). A3 and A6 verified against the live Valve pages (key, games-only default,
appid order, `last_appid`, `max_results` 10k default / 50k max, `if_modified_since`
"changed or added", `partner.steam-api.com`; 100,000 calls/day, §7/§8 disclaimers,
personal/non-transferable, revocable). A5/A11 are honestly carried as unverified and
the body keeps them as estimates. Evidence gap: the cargo/rustfmt/npm check legs
could not be run — this session's shell was denied — so every code claim was
verified by reading the cited sources rather than by executing the gates.

## Findings

- [major][new] Issued entries, the mint — "Truncation is **not** part of the rule:
  since every code point folds to at most one octet and the snapshot loader has
  already bounded `name` to `MAX_NATIVE_NAME_OCTETS = 128` … a derived slug is
  grammar-valid and within `MAX_SLUG_OCTETS` by construction" (step 3) vs step 5's
  collision spelling "`<slug>-steam-<appid>`" — the by-construction bound covers
  only the plain derivation. The collision spelling appends `-steam-<appid>` (6
  octets plus up to 10 appid digits), so a base slug of ≥ 115 octets — legal, since
  the snapshot loader admits a 128-octet `name` and a 128-octet ASCII name folds to
  a 128-octet slug — produces an id over `MAX_SLUG_OCTETS = 128`
  (`crates/core/src/placement.rs:83`), which `corpus.rs::game_id_is_valid` refuses
  at load and at write. So the mint's own output can be an id the document type it
  writes rejects, and Test strategy #1's assertion "every output satisfies
  `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS`" is unachievable for that branch. The design
  names no refusal for it either, so the failure is unnamed. Fix: state a length
  rule for the collision spelling only — truncate the *stem* to
  `MAX_SLUG_OCTETS − len("-steam-") − digits(appid)` at the last `-` boundary before
  the cut, then append the suffix (the appid suffix keeps it record-stable and
  appid-unique, so `024-L2`'s ordinals stay unrevived); the plain id keeps the
  never-truncated posture. Add the case to Test strategy #1 (a 128-octet colliding
  name).

- [major][new] Concurrency — "Two `registry serve` instances against one `<data>`
  directory stay a refused, unsupported state by the fixed-tmp `create_new`
  discipline (A2) — unchanged from `026`, now covering three documents" — the cited
  discipline is not a startup refusal and does not refuse a second process.
  `apps/registry/src/persist.rs`'s rule is a fixed `<document>.tmp` opened
  `create_new(true)` and renamed into place: the tmp name exists only during one
  write, so two instances that write at different times never collide — the second
  one's `create_new` succeeds and its rename silently replaces the first's document
  with a model it read at its own startup. The claim matters more here than in
  `026` because §Hosted mode hands a *second, separate* writer of the identity
  ledger: "`registry query <slug>` survives as the one-shot hosted lookup policy …
  A `query` can mint and persist, exactly like a served query", and the ADR's
  `027-C1` makes `query` the operator's run of that policy — i.e. on the same host,
  against the same `<data>`, while `serve` is up. The mutation lock is a process
 -local `Mutex` (§Concurrency), so a mint persisted by `query` and a later mint
  persisted by `serve` are not serialized against each other: `serve` renames its
  stale model over `query`'s record, and a record `027-C4` persisted-before-serve
  has vanished. `027-L2` is the law the design says "makes minting admissible at
  all", and nothing in the document states the cross-process posture for
  `query`-beside-`serve` at all. Fix: name one mechanism that holds for every
  writer of `<data>` — an owner-only lock file in `<data>/bindforge/registry/` taken
  under `create_new` at startup and refused by presence, replacing the `create_new`
  claim as the enforcement — or make `query` mint-free against a live data directory
  and say which; then re-point §Concurrency's unsupported-state sentence at it.

- [minor][new] The store pull — "`fetched_at` is stamped from the wall clock at
  publish time" together with "for delta pulls `if_modified_since` = the loaded
  snapshot's `fetched_at`" — the stamp is later than the data. Everything the pull
  fetched reflects the store as of the page fetch, so an app added in the window
  between the last page's data and the publish stamp is below the next delta's
  `if_modified_since` and is never seen again until the operator deletes the
  snapshot. Meanwhile it answers `200 []` from a snapshot the design calls fresh —
  the exact "never heard of it" the design refused to fabricate when it rejected
  silent record drops for over-long names ("a store name outside our budget fails
  the pull loudly rather than entering a silent hole under `027-L3`'s honesty
  claim"). Fix: stamp `fetched_at` from the wall clock read *before the first page
  request* of the pull (overlap only costs a re-fetched item, which the appid merge
  absorbs), replacing the publish-time stamp.

- [minor][new] Issued entries — retired ids / The curation face / Concurrency —
  "`MAX_RETIRED_IDS_OCTETS = 33_554_432`" appears only as a budget, and the delete's
  write is described two ways: "**Every** delete — issued or corpus — appends the
  removed record's `game_id` there before the mutation's persist" (two documents,
  two tmp-and-renames) against "`DELETE /v1/admin/issued/<game_id>` … move the
  record's id into the retired list and remove the record … in one mutation-lock
  persist" (one). Three consequences are unspecified: what a retired-append that
  breaches its octet cap answers (a delete refused, or an id not retired — which is
  the `027-L2` breach), what happens when the second of the two renames fails after
  the first landed, and where the retired document sits in the stated order
  "Lock order is `mutations → corpus read → issued write`", though mints read it for
  the held set and deletes write it. Fix: keep the retire-first ordering the §Issued
  entries sentence already states, correct the delete bullet to "two persists under
  one lock, retired first" (a failed second persist leaves the id retired and the
  record held — the safe direction), name the cap-breach refusal, and add the
  retired read/write to the lock-order line.

- [minor][new] Assumption register A3 and The store pull — A3 is marked verified
  against `partner.steamgames.com/doc/webapi/IStoreService`, and that page does
  carry the key parameter, the games-only default, appid ordering, `last_appid`,
  `max_results` (10k default, 50k max) and `if_modified_since`'s "changed or added"
  semantics; it documents no response body at all. `have_more_results` — the
  predicate every budget rule in the document turns on ("more than
  `STEAM_REFRESH_MAX_PAGES` (12) pages while `have_more_results` is true", and the
  rejection of partial snapshots) — is third-party-confirmed but not in the cited
  basis, and the response's envelope and per-record `last_modified` /
  `price_change_number` fields appear nowhere in the design. §The store pull's named
  refusals cover status, framing, octets and per-record bounds but never name a body
  that is not the expected envelope or has no `apps` list, and
  `MAX_STEAM_PAGE_OCTETS = 8_388_608` is sized from a "~150-octet records" estimate
  that excludes those two fields. Fix: cite the response shape where `have_more_results`
  is first used (or mark that half of A3 unverified), add a named
  malformed-body/envelope refusal to the pull's response rules, and restate the page
  budget's arithmetic over the fields the endpoint actually returns.

- [minor][new] The store pull — "Per page, up to `STEAM_PAGE_ATTEMPTS` (3) attempts
  separated by `STEAM_RETRY_SECONDS`" with `STEAM_RETRY_SECONDS = 3_600` — the rule
  is written once and the pull is shared, so a startup full pull against an
  unreachable store blocks `serve` for roughly two hours of inter-attempt waits
  before it reaches "a failed full pull is a startup refusal", on a process whose
  contract is to print the bound port. Fix: state that a startup pull's attempts are
  separated by no wait (or by the connect/read timeout), and leave the hourly
  `STEAM_RETRY_SECONDS` pacing to the refresh thread, which already has its own
  "retrying after `STEAM_RETRY_SECONDS`" sentence.

- [minor][new] The hosted lookup policy, step 5 — "Fresh snapshot, no such appid
  (including a non-decimal external id — the snapshot knows every appid there is):
  `200 []`" — per A3's own cited basis the pull is games-only by default, so what
  the snapshot knows is every *game* appid; a `steam/<appid>` key for DLC, software,
  video or hardware is a real app the registry answers "never heard of it" about.
  The posture is the ADR's (`027-C3` "games-only pages", `027-L3`), so nothing needs
  to change in behavior — but §The store pull's request rules never name the item
  class it asks for, which is what the honesty claim rests on. Fix: pin the request
  as the endpoint's default games-only set (no `include_*` flag is sent) in the
  request rules, and narrow the parenthetical to "every game appid there is".

- [nit][new] Hosted mode — "`registry query <slug>` survives as the one-shot hosted
  lookup policy … one lookup with its provenance (`corpus` or `issued`)" — the
  policy's two `200 []` branches (step 3's `local/…`, step 5's fresh appid miss)
  carry no provenance, and the shipped printer prints
  `answered from <provenance>` for an empty candidate list
  (`apps/registry/src/main.rs`, `query_command`). Fix: one clause naming what
  `query` prints for an empty answer (the shipped `(no candidates)` shape with a
  named source, or an explicit `none`), and extend Test strategy #15 to it.

- [nit][new] Hosted mode — "either `<data>/bindforge/registry/cache-v1.json` or
  `catalog-v1.json` on disk is refused by name before anything binds" — an operator
  who upgrades by pointing `--corpus` at their existing `catalog-v1.json` (the
  shipped local-entries document, and the natural migration move) is refused for
  holding the very file the runbook then tells them to clean up. Fix: one carve-out —
  the presence refusal does not apply to the path `--corpus` names.

## FYI (no severity)

- `026`'s ToS audit in Rollout item 6 names the §7/§8 disclaimers, key
  confidentiality/non-transferability, named-Application registration and
  no-endorsement rules; the same Terms' first restriction (post a privacy policy and
  store Steam Data in a country named in it) is not in that list — security/compat
  seat's call whether the runbook must name it.
- `MAX_STEAM_PAGE_OCTETS` sits about 12% above the document's own mid-estimate for a
  full 50k page, and every mint re-serializes a document budgeted at 64 MiB under
  the single mutation mutex (≤64 threads deep, A8) — performance seat.
- Health drops `local_entries`, `cached_slugs` and `last_upstream_fetch` and renames
  the count field to `corpus_entries`; the ADR only promises `mode` and the
  `steam_snapshot` fact — compatibility seat.
