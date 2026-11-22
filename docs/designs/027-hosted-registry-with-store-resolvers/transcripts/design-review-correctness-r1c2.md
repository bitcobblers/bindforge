# Transcript: design-review:correctness r1c2

## Provenance
- stage: design-review:correctness (round r1c2)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:53:19.917Z
- ended: 2026-11-22T04:07:32.233Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":65497,"output":39618,"reasoning":0,"cache.read":895695,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff fcb66a83bd31cc82493e01e93f55b745bd37e25c 19dcdcb170ba3f06749f7a84641e94b5839d1f77` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "The steam snapshot resolver (with The hosted lookup policy step 5 and the Risks row on 013-B4) — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound; `0` marks a worldlist, and the only kind allowed to certify a miss\" / \"every steam key that is not an appid hit takes the named `503` `steam snapshot incomplete`\"",
    "problem": "One over-long store name anywhere in the ~200k catalog converts every steam miss into a named 503 — and since the pull is games-only (the r1 pinning this pass confirmed), the misses are exactly the DLC/tool/video/hardware/delisted appids users actually have installed. The Compatibility matrix's own r1 sentence, verified in apps/discovery-cli/src/catalog.rs (`code != \"200\"` → CatalogError::unreachable) and main.rs:197 (\"A mid-scan failure fails closed: no partial table (`025-C4`)\"), makes that a refused scan for every affected user. Nothing in the delta rules clears the mark and a forced full refresh re-drops the same record, so 027-L3's \"'never heard of it' is a statement the snapshot is entitled to make\" becomes unavailable service-wide and indefinitely, with only a budget amendment as remedy. The design rates this likelihood Medium, so the r1 mitigation swaps the r0 cliff for an equivalent one instead of removing it.",
    "fix": "Bound the non-certification to the discarded records, not the document: the pull knows each dropped record's appid, so keep them in a small capped `dropped_appids` list in the snapshot, refuse `200 []` only for those appids, and let every other appid certify its miss honestly (all pages were examined). A breach of that list's cap is the pull failure. This also makes the mark self-clearing and removes the global flag Options' \"no partial list is ever certified either way\" leans on."
  },
  {
    "severity": "minor",
    "location": "The steam snapshot resolver / The store pull — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound\" against \"a delta merges by `appid` … a drop for the `013-B4` bound bumps `incomplete` rather than failing the page\"",
    "problem": "The field is defined per-pull but merged cumulatively, and the document never says which the implementation carries. A delta that sets the count from its own drops clears a record an earlier pull discarded, and the snapshot is then entitled to certify `200 []` against a worldlist it knows is short — the 027-L3 breach the mark exists to prevent.",
    "fix": "State that the dropped-record mark is cumulative across deltas and resets only on a full-pull replace, and that it is what tells the operator to pursue the Deferred name-bound resize."
  },
  {
    "severity": "minor",
    "location": "The steam snapshot resolver / Test strategy #9 — \"exactly `{version: 1, fetched_at, incomplete, apps: [{appid, name}]}`\" and health's \"`{\"games\": <count>, \"fetched_at\": <unix>, \"incomplete\": <count>}`\"",
    "problem": "027-C3 pins both shapes verbatim (\"`{version, fetched_at, apps: [{appid, name}]}` … refused by name outside its shape\"; \"a `steam_snapshot` fact carries `{games, fetched_at}` or `none`\"), so the design ships a snapshot document outside the ADR's own shape sentence and pins a health document the ADR does not allow. Rollout 1's annotations cover only \"the `026` units this ADR amends\", and the runner accepts ADR-027 as written, so nothing in the run reconciles the deviation.",
    "fix": "One line in Rollout 1, mirrored at the section, recording that 027-C3's snapshot shape and the `steam_snapshot` health fact gain the `incomplete` fact under this run; drop or restate the word \"exactly\"."
  },
  {
    "severity": "minor",
    "location": "Concurrency — \"Lock order is `mutations → corpus read → retired read → issued read/write` and never the reverse: … deletes write the retired list then their catalog document\"",
    "problem": "The order line puts corpus ahead of retired; the retire-first posture pinned in Issued entries and The curation face puts the retired write ahead of the corpus write for a corpus delete, and the section's \"there is no cycle\" rests on the line as written. With writers serialized by the mutex, the remaining exposure is a writer holding one document's RwLock write half while taking another's against a reader holding the later one and waiting for the earlier.",
    "fix": "State that a delete's two model swaps are sequential (retired write half, swap, release, then corpus write half) and never held together, and restate the order line to cover the write direction with the corpus write named as the one place a delete departs from it."
  },
  {
    "severity": "minor",
    "location": "Issued entries, mint rule step 5 — \"A mint whose chosen id exceeds `MAX_SLUG_OCTETS` mints `steam-<appid>`: appid-unique, ≤ 26 octets under the closed key grammar's appid bound\"",
    "problem": "No such appid bound exists: keys.rs's `key_is_admissible` caps the whole key at `MAX_DISCOVERY_SLUG_OCTETS = 128` (placement.rs:95) and bounds the external id only by separators and control bytes, so a `steam/…` key may carry 122 digits and the fallback lands at exactly 6 + 122 = 128 octets — in budget, but only just, and not by the quoted arithmetic. The step also never says which appid string the fallback is spelled from.",
    "fix": "Cite `MAX_DISCOVERY_SLUG_OCTETS` (whole-key cap, external id ≤ 122) instead of the invented appid bound, name the snapshot record's appid as the spelling source, and bound appid digit count in the snapshot loader so the claim is true outright."
  },
  {
    "severity": "minor",
    "location": "Test strategy #6 \"Transport assurance\" — \"the *production* `StoreTransport` … driven against a loopback TLS listener serving a self-signed cert and one serving a cert for another name; it refuses the handshake both times\"",
    "problem": "The store pull pins the authority at compile time with \"no operator-configurable store URL\" and no seam for pointing the production client at 127.0.0.1 is named. Both loopback cases can therefore refuse for server-name mismatch against partner.steam-api.com while chain validation is broken, so the very case the entry says catches \"a verifier relaxed to accept the fake\" passes vacuously.",
    "fix": "Name the test seam (a StoreTransport constructor taking connect address and verification name, production value still compile-time-pinned) and make the self-signed case present a cert whose name matches the connection's, so what it fails on is the chain."
  },
  {
    "severity": "nit",
    "location": "The store pull / Budgets — \"a pull of either kind that has not published inside `STEAM_PULL_DEADLINE_SECONDS` fails by name\" with `STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`, `STEAM_CONNECT_TIMEOUT = 2 s` / `STEAM_READ_TIMEOUT = 30 s`, `STEAM_REFRESH_MAX_PAGES = 12`",
    "problem": "The stated worst case is 12 × (3 × 32 + 2 × 30) ≈ 1.9 ks, above the 900 s deadline, so a store that is merely slow rather than stalled is a guaranteed named failure (startup refusal, or a delta that never lands) — a posture presented only as a stall guard.",
    "fix": "One clause saying the deadline is intended to cut off a pull that would run past ~15 minutes including retries, or size the deadline from the page budget."
  },
  {
    "severity": "nit",
    "location": "Budgets — \"`MAX_STEAM_PAGE_OCTETS = 8_388_608` (A17's page fields at ~150 octets for 50k records, ~12% over the mid-estimate)\" beside `STEAM_PAGE_MAX_RESULTS = 50_000`",
    "problem": "The cap covers a mid-estimate page, not a legal one: a 50k page whose names ride the 013-B4 bound the loader itself admits (128 octets, A10's surveyed maximum) is ~10 MiB, so the endpoint can return a page the design must refuse, and the pull with it.",
    "fix": "Derive the pinned `max_results` from the cap (⌊8 MiB / worst-legal-record⌋) or size the cap for 50k worst-legal records, and say which the number is chosen for."
  },
  {
    "severity": "nit",
    "location": "Hosted mode / Concurrency / Rollout 6 — \"A `query` can mint and persist, exactly like a served query — which is why it takes the same `<data>` write lock and refuses to run against one a live `serve` holds\"",
    "problem": "027-C1 makes the hosted instance the one instance everyone points at, so the operator's one-shot run of the lookup policy can never run against a live instance — only a stopped one, or a copy of `<data>` it would then mint into.",
    "fix": "Name the supported invocation in Hosted mode and the runbook, or permit a read-only `query` against a locked `<data>` that answers from corpus/issued/snapshot and refuses to mint, saying which."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c2.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "The steam snapshot resolver / The store pull — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound `027-C3` names — the pull never writes one, §The store pull)\"", "problem": "new: the parenthetical is asserted in the changed text but no pull rule backs it. §The store pull's named refusals are non-`200`, `Transfer-Encoding`, `3xx`, over-`MAX_STEAM_PAGE_OCTETS`, wrong envelope, page-budget, non-decimal `appid`, and \"The one *dropped*, not refused, case is a record whose `name` breaks the `013-B4` bound\" — per A10 that bound is `MAX_NATIVE_NAME_OCTETS = 128` alone; the control-byte rule is the loader's separate refusal. A pulled record whose name carries an ASCII control byte is therefore neither refused nor dropped: it is published, the next load refuses the file, and \"a present-but-unloadable snapshot that the pull then cannot replace is a named startup refusal\" does not save the operator because the pull can replace it and re-publishes the same unloadable bytes — the run serves from memory while every later restart is a named refusal unless `--no-steam`. Same remote-content-into-availability-cliff class this pass set out to clear, one rule from the drop rule that now exists.", "fix": "Widen the one dropped case to \"a record whose `name` breaks the `013-B4` octet bound **or carries a control byte**\" (the two refusals the loader already names), so the parenthetical becomes the rule it points at."}, {"severity": "minor", "location": "Issued entries, mint rule step 5 — \"mints `steam-<appid>`, spelled from the queried key's external id … appid-unique\"", "problem": "new: spelling the fallback from the client's digits makes the id depend on the queried spelling of one appid. `steam/42` and `steam/0042` are both admissible (`keys.rs:73` bounds length, token set and path safety only — I verified there is no appid bound), and the design never pins how the \"one exact-`appid` probe\" treats them. Numeric match: one snapshot record mints `steam-42` and `steam-0042`, two held ids for one game, so \"appid-unique\" is false and step 4's suffix has no stated appid source (`mint.rs::mint_id(display_name, appid, held)` carries one `appid` for both jobs). Textual match: `steam/0042` is not present in the snapshot, takes the second bullet, and answers a false `200 []` for an appid the catalog holds — the claim `027-L3` exists to police.", "fix": "Name the canonical spelling in step 5 and at the probe: spell the fallback from the snapshot record's `appid` (or refuse a non-canonical decimal external id before the probe, saying which); keep `MAX_DISCOVERY_SLUG_OCTETS` as the cited arithmetic, which is verified correct."}, {"severity": "nit", "location": "Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k *worst-legal* records at ~240 octets\"", "problem": "re-raised (r1 nit, narrowed but not closed): ~240 octets costs the name at its 128-octet *decoded* bound, but the response body carries JSON string escapes, so a quote- or backslash-heavy name serializes at up to 2× and a record lands near 340 octets — a legal 50k page of such names is ~17 MiB, over the cap, and the design must refuse that page and fail the pull (a startup refusal) on a legal response.", "fix": "One clause saying which the cap is chosen for: either double the name allowance for escapes in the derivation (⌈2 × 128⌉) or pin `max_results` from the worst-escaped record."}, {"severity": "nit", "location": "Hosted mode / Risks / Deferred — \"`dropped_appids` is not a health fact — the pull writing the list prints it\" beside \"its `appid` is listed in the snapshot's `dropped_appids`, which is the operator's signal to pursue that amendment\"", "problem": "new: with the fact off `health` (rightly, per `027-C3`) and off `stats` (Deferred), the operator's only channels are one stdout line at publish time and opening the snapshot file; Rollout 6 enumerates \"reading the retired list's size against its cap\" but never the dropped list, so the signal the design leans on twice has no runbook line and no restart-surviving surface.", "fix": "Add the dropped list to Rollout 6's reading duty in the same clause as the retired-list size; no new surface needed."}, {"severity": "nit", "location": "The store pull — \"sized in §Budgets to cover this grid with its retries, so a slow store costs a late snapshot, not a guaranteed failure\"", "problem": "new: true for the background delta, not for the pre-bind full pull, where the same 1,872 s grid ends in `027-C3`'s named startup refusal after ~31 minutes of a process that has not bound. This pass deleted the sentence that bounded that path (\"the pre-bind full pull is paced by the socket timeouts and a seconds-scale retry, never an hour-long sleep\") while raising `STEAM_PULL_DEADLINE_SECONDS` from 900 to 1,920, so nothing states what a slow store costs an operator's start.", "fix": "Scope the clause — \"costs a late delta; a full pull that runs the grid is the named `steam pull deadline` startup refusal\" — and keep the deadline value, which is arithmetically right."}]}
```
