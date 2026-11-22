# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:22:42.562Z
- ended: 2026-11-22T03:36:16.708Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":58066,"output":29685,"reasoning":0,"cache.read":873130,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 7b5cec46a35f016bf0e05e5cb90abfc015a6cc32 fcb66a83bd31cc82493e01e93f55b745bd37e25c` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/027-hosted-registry-with-store-resolvers.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Issued entries, the mint, and identity bookkeeping — mint rule step 3: \"Truncation is **not** part of the rule … a derived slug is grammar-valid and within `MAX_SLUG_OCTETS` by construction\", against step 5: \"the id becomes `<slug>-steam-<appid>`\" (also Test strategy #1: \"every output satisfies `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS`\")",
    "problem": "The by-construction bound covers only the plain derivation. The collision spelling appends `-steam-<appid>` (6 octets plus up to 10 appid digits), so a base slug of ≥115 octets — legal, since the snapshot loader admits a 128-octet `name` and a 128-octet ASCII name folds to a 128-octet slug — yields an id over `MAX_SLUG_OCTETS = 128` (crates/core/src/placement.rs), which `corpus.rs::game_id_is_valid` refuses both at load and at the mint's own write. The mint can therefore emit an id the document type it writes rejects, Test strategy #1's assertion is unachievable for that branch, and no named refusal covers it.",
    "fix": "Add a length rule for the collision spelling only: truncate the stem to `MAX_SLUG_OCTETS − len(\"-steam-\") − digits(appid)` at the last `-` boundary before the cut, then append the appid suffix (appid-unique, so record-stability holds and `024-L2`'s ordinals stay unrevived). Replace step 3's blanket \"by construction\" claim with that rule and add a 128-octet colliding name to Test strategy #1."
  },
  {
    "severity": "major",
    "location": "Concurrency: three writers, one mutation lock — \"Two `registry serve` instances against one `<data>` directory stay a refused, unsupported state by the fixed-tmp `create_new` discipline (A2) — unchanged from `026`, now covering three documents\", against Hosted mode: \"A `query` can mint and persist, exactly like a served query\"",
    "problem": "The cited discipline is not a startup refusal and does not refuse a second process: persist.rs's tmp name exists only during one write, so two processes writing at different times never collide and the later rename silently replaces the earlier document with a model read at its own startup. The gap is new in scale because `query` is a separate mint-capable writer of the identity ledger on the same `<data>` (ADR `027-C1` makes it the operator's one-shot policy run), while the mutation lock is a process-local `Mutex`: a mint `serve` persisted from a stale model can overwrite a mint `query` persisted-before-serve, silently dropping an issued record — the ledger `027-L2` exists to protect. The document states no cross-process posture for query-beside-serve.",
    "fix": "Name one mechanism that binds every writer of `<data>` — an owner-only lock file in `<data>/bindforge/registry/` taken under `create_new` at startup and refused by presence — and re-point the unsupported-state sentence at it, or make `query` refuse to mint against a locked data directory and say which. This replaces the create_new claim as the enforcement."
  },
  {
    "severity": "minor",
    "location": "The store pull — \"`fetched_at` is stamped from the wall clock at publish time\", with \"for delta pulls `if_modified_since` = the loaded snapshot's `fetched_at`\"",
    "problem": "The stamp is later than the data it certifies. An app added between the last page's data and the publish stamp falls below the next delta's `if_modified_since` and is never seen again until the operator deletes the snapshot, while all that time it answers `200 []` from a snapshot treated as fresh — a silent record hole under `027-L3`, the same failure mode the design refused for over-long store names.",
    "fix": "Stamp `fetched_at` from the wall clock read before the pull's first page request, replacing the publish-time stamp; the overlap costs only re-fetched items, which the appid merge absorbs."
  },
  {
    "severity": "minor",
    "location": "Issued entries (retired ids) — \"`MAX_RETIRED_IDS_OCTETS = 33_554_432`\" and \"**Every** delete — issued or corpus — appends the removed record's `game_id` there before the mutation's persist\", against The curation face — \"move the record's id into the retired list and remove the record … in one mutation-lock persist\", and Concurrency — \"Lock order is `mutations → corpus read → issued write`\"",
    "problem": "The delete writes two documents but is described once as one persist and once as an append-before-persist. Unspecified: what a retired append that breaches its octet cap answers, what happens when the second rename fails after the first landed, and where the retired document sits in the stated lock order although mints read it for the held set and deletes write it.",
    "fix": "Keep the retire-first ordering, correct the delete bullet to \"two persists under one lock, retired first\" (a failed second persist leaves the id retired and the record held — the safe direction), name the cap-breach refusal, and add the retired read/write to the lock-order line."
  },
  {
    "severity": "minor",
    "location": "Assumption register A3 and The store pull — A3 (marked verified against the IStoreService page): \"pages via `last_appid`/`have_more_results`\", and the pull's response rules: \"more than `STEAM_REFRESH_MAX_PAGES` (12) pages while `have_more_results` is true\"",
    "problem": "The cited Valve page documents the parameters but no response body at all: `have_more_results`, the pagination cursor it returns, the response envelope, and the per-record `last_modified`/`price_change_number` fields are all unverified by the basis, yet every page-budget and partial-snapshot rule turns on `have_more_results`, and `MAX_STEAM_PAGE_OCTETS = 8_388_608` is sized from a \"~150-octet records\" estimate that excludes those fields. The pull's named refusals also never cover a body that is not the expected envelope or has no apps list.",
    "fix": "Cite the response shape where `have_more_results` is first used (or mark that half of A3 unverified), add a named malformed-envelope/no-apps-list refusal to the pull's response rules, and restate the page-budget arithmetic over the fields the endpoint actually returns."
  },
  {
    "severity": "minor",
    "location": "The store pull — \"Per page, up to `STEAM_PAGE_ATTEMPTS` (3) attempts separated by `STEAM_RETRY_SECONDS`\", with `STEAM_RETRY_SECONDS = 3_600`",
    "problem": "The rule is stated once and the pull is shared by startup and background paths, so a startup full pull against an unreachable store blocks `serve` for roughly two hours of inter-attempt waits before it reaches \"a failed full pull is a startup refusal\" — on a process whose startup contract is to print the bound port.",
    "fix": "State that a startup pull's attempts are separated by no wait (or by the connect/read timeout) and leave the hourly `STEAM_RETRY_SECONDS` pacing to the refresh thread, which already states its own retry interval."
  },
  {
    "severity": "minor",
    "location": "The hosted lookup policy, step 5 — \"Fresh snapshot, no such appid (including a non-decimal external id — the snapshot knows every appid there is): `200 []`\"",
    "problem": "By A3's own cited basis the listing is games-only, so the snapshot knows every game appid, not every appid; keys naming DLC, software, video or hardware apps get a \"never heard of it\". The posture itself is the ADR's (`027-C3` games-only pages, `027-L3`), but §The store pull's request rules never name the item class requested, which is what the honesty claim rests on.",
    "fix": "Pin the request as the endpoint's default games-only set (no `include_*` flag is sent) in the request rules, and narrow the parenthetical to \"the snapshot knows every game appid there is\"."
  },
  {
    "severity": "nit",
    "location": "Hosted mode — \"one lookup with its provenance (`corpus` or `issued`), non-zero exit on every named-503 posture\"",
    "problem": "The policy's two `200 []` branches (`local/…`, and the fresh-snapshot appid miss) carry no provenance, and the shipped `query_command` prints `answered from <provenance>` even for an empty candidate list, so the operator surface has no defined word (or exit) for the empty answer; Test strategy #15 pins only the two provenance words and the 503 exits.",
    "fix": "Add one clause naming what `query` prints for an empty answer (the shipped `(no candidates)` shape with a named source, or an explicit `none`) and extend Test strategy #15 to it."
  },
  {
    "severity": "nit",
    "location": "Hosted mode — \"either `<data>/bindforge/registry/cache-v1.json` or `catalog-v1.json` on disk is refused by name before anything binds\"",
    "problem": "An operator upgrading by pointing `--corpus` at their existing `catalog-v1.json` — the shipped caching-mode local-entries document, and the natural migration move — is refused for holding the very file the runbook then tells them to clean up.",
    "fix": "Add one carve-out: the presence refusal does not apply to the path `--corpus` names."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/027-hosted-registry-with-store-resolvers.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c1.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"The steam snapshot resolver (with The hosted lookup policy step 5 and the Risks row on 013-B4) — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound; `0` marks a worldlist, and the only kind allowed to certify a miss\" / \"every steam key that is not an appid hit takes the named `503` `steam snapshot incomplete`\"","problem":"One over-long store name anywhere in the ~200k catalog converts every steam miss into a named 503 — and since the pull is games-only (the r1 pinning this pass confirmed), the misses are exactly the DLC/tool/video/hardware/delisted appids users actually have installed. The Compatibility matrix's own r1 sentence, verified in apps/discovery-cli/src/catalog.rs (`code != \"200\"` → CatalogError::unreachable) and main.rs:197 (\"A mid-scan failure fails closed: no partial table (`025-C4`)\"), makes that a refused scan for every affected user. Nothing in the delta rules clears the mark and a forced full refresh re-drops the same record, so 027-L3's \"'never heard of it' is a statement the snapshot is entitled to make\" becomes unavailable service-wide and indefinitely, with only a budget amendment as remedy. The design rates this likelihood Medium, so the r1 mitigation swaps the r0 cliff for an equivalent one instead of removing it.","fix":"Bound the non-certification to the discarded records, not the document: the pull knows each dropped record's appid, so keep them in a small capped `dropped_appids` list in the snapshot, refuse `200 []` only for those appids, and let every other appid certify its miss honestly (all pages were examined). A breach of that list's cap is the pull failure. This also makes the mark self-clearing and removes the global flag Options' \"no partial list is ever certified either way\" leans on."},{"severity":"minor","location":"The steam snapshot resolver / The store pull — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound\" against \"a delta merges by `appid` … a drop for the `013-B4` bound bumps `incomplete` rather than failing the page\"","problem":"The field is defined per-pull but merged cumulatively, and the document never says which the implementation carries. A delta that sets the count from its own drops clears a record an earlier pull discarded, and the snapshot is then entitled to certify `200 []` against a worldlist it knows is short — the 027-L3 breach the mark exists to prevent.","fix":"State that the dropped-record mark is cumulative across deltas and resets only on a full-pull replace, and that it is what tells the operator to pursue the Deferred name-bound resize."},{"severity":"minor","location":"The steam snapshot resolver / Test strategy #9 — \"exactly `{version: 1, fetched_at, incomplete, apps: [{appid, name}]}`\" and health's \"`{\"games\": <count>, \"fetched_at\": <unix>, \"incomplete\": <count>}`\"","problem":"027-C3 pins both shapes verbatim (\"`{version, fetched_at, apps: [{appid, name}]}` … refused by name outside its shape\"; \"a `steam_snapshot` fact carries `{games, fetched_at}` or `none`\"), so the design ships a snapshot document outside the ADR's own shape sentence and pins a health document the ADR does not allow. Rollout 1's annotations cover only \"the `026` units this ADR amends\", and the runner accepts ADR-027 as written, so nothing in the run reconciles the deviation.","fix":"One line in Rollout 1, mirrored at the section, recording that 027-C3's snapshot shape and the `steam_snapshot` health fact gain the `incomplete` fact under this run; drop or restate the word \"exactly\"."},{"severity":"minor","location":"Concurrency — \"Lock order is `mutations → corpus read → retired read → issued read/write` and never the reverse: … deletes write the retired list then their catalog document\"","problem":"The order line puts corpus ahead of retired; the retire-first posture pinned in Issued entries and The curation face puts the retired write ahead of the corpus write for a corpus delete, and the section's \"there is no cycle\" rests on the line as written. With writers serialized by the mutex, the remaining exposure is a writer holding one document's RwLock write half while taking another's against a reader holding the later one and waiting for the earlier.","fix":"State that a delete's two model swaps are sequential (retired write half, swap, release, then corpus write half) and never held together, and restate the order line to cover the write direction with the corpus write named as the one place a delete departs from it."},{"severity":"minor","location":"Issued entries, mint rule step 5 — \"A mint whose chosen id exceeds `MAX_SLUG_OCTETS` mints `steam-<appid>`: appid-unique, ≤ 26 octets under the closed key grammar's appid bound\"","problem":"No such appid bound exists: keys.rs's `key_is_admissible` caps the whole key at `MAX_DISCOVERY_SLUG_OCTETS = 128` (placement.rs:95) and bounds the external id only by separators and control bytes, so a `steam/…` key may carry 122 digits and the fallback lands at exactly 6 + 122 = 128 octets — in budget, but only just, and not by the quoted arithmetic. The step also never says which appid string the fallback is spelled from.","fix":"Cite `MAX_DISCOVERY_SLUG_OCTETS` (whole-key cap, external id ≤ 122) instead of the invented appid bound, name the snapshot record's appid as the spelling source, and bound appid digit count in the snapshot loader so the claim is true outright."},{"severity":"minor","location":"Test strategy #6 \"Transport assurance\" — \"the *production* `StoreTransport` … driven against a loopback TLS listener serving a self-signed cert and one serving a cert for another name; it refuses the handshake both times\"","problem":"The store pull pins the authority at compile time with \"no operator-configurable store URL\" and no seam for pointing the production client at 127.0.0.1 is named. Both loopback cases can therefore refuse for server-name mismatch against partner.steam-api.com while chain validation is broken, so the very case the entry says catches \"a verifier relaxed to accept the fake\" passes vacuously.","fix":"Name the test seam (a StoreTransport constructor taking connect address and verification name, production value still compile-time-pinned) and make the self-signed case present a cert whose name matches the connection's, so what it fails on is the chain."},{"severity":"nit","location":"The store pull / Budgets — \"a pull of either kind that has not published inside `STEAM_PULL_DEADLINE_SECONDS` fails by name\" with `STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`, `STEAM_CONNECT_TIMEOUT = 2 s` / `STEAM_READ_TIMEOUT = 30 s`, `STEAM_REFRESH_MAX_PAGES = 12`","problem":"The stated worst case is 12 × (3 × 32 + 2 × 30) ≈ 1.9 ks, above the 900 s deadline, so a store that is merely slow rather than stalled is a guaranteed named failure (startup refusal, or a delta that never lands) — a posture presented only as a stall guard.","fix":"One clause saying the deadline is intended to cut off a pull that would run past ~15 minutes including retries, or size the deadline from the page budget."},{"severity":"nit","location":"Budgets — \"`MAX_STEAM_PAGE_OCTETS = 8_388_608` (A17's page fields at ~150 octets for 50k records, ~12% over the mid-estimate)\" beside `STEAM_PAGE_MAX_RESULTS = 50_000`","problem":"The cap covers a mid-estimate page, not a legal one: a 50k page whose names ride the 013-B4 bound the loader itself admits (128 octets, A10's surveyed maximum) is ~10 MiB, so the endpoint can return a page the design must refuse, and the pull with it.","fix":"Derive the pinned `max_results` from the cap (⌊8 MiB / worst-legal-record⌋) or size the cap for 50k worst-legal records, and say which the number is chosen for."},{"severity":"nit","location":"Hosted mode / Concurrency / Rollout 6 — \"A `query` can mint and persist, exactly like a served query — which is why it takes the same `<data>` write lock and refuses to run against one a live `serve` holds\"","problem":"027-C1 makes the hosted instance the one instance everyone points at, so the operator's one-shot run of the lookup policy can never run against a live instance — only a stopped one, or a copy of `<data>` it would then mint into.","fix":"Name the supported invocation in Hosted mode and the runbook, or permit a read-only `query` against a locked `<data>` that answers from corpus/issued/snapshot and refuses to mint, saying which."}]}
```
