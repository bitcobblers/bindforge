# Transcript: design-review:correctness r1c5

## Provenance
- stage: design-review:correctness (round r1c5)
- agent: review/design-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:23:01.635Z
- ended: 2026-11-22T04:34:40.658Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":55709,"output":25015,"reasoning":0,"cache.read":988878,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 19dcdcb170ba3f06749f7a84641e94b5839d1f77 d105d78caaf272e507760abf17dedfdd966810b6` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "minor",
    "location": "The steam snapshot resolver / The store pull — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound `027-C3` names — the pull never writes one, §The store pull)\"",
    "problem": "new: the parenthetical is asserted in the changed text but no pull rule backs it. §The store pull's named refusals are non-`200`, `Transfer-Encoding`, `3xx`, over-`MAX_STEAM_PAGE_OCTETS`, wrong envelope, page-budget, non-decimal `appid`, and \"The one *dropped*, not refused, case is a record whose `name` breaks the `013-B4` bound\" — per A10 that bound is `MAX_NATIVE_NAME_OCTETS = 128` alone; the control-byte rule is the loader's separate refusal. A pulled record whose name carries an ASCII control byte is therefore neither refused nor dropped: it is published, the next load refuses the file, and \"a present-but-unloadable snapshot that the pull then cannot replace is a named startup refusal\" does not save the operator because the pull can replace it and re-publishes the same unloadable bytes — the run serves from memory while every later restart is a named refusal unless `--no-steam`. Same remote-content-into-availability-cliff class this pass set out to clear, one rule from the drop rul… [elided by the checkpoint cap]",
    "fix": "Widen the one dropped case to \"a record whose `name` breaks the `013-B4` octet bound **or carries a control byte**\" (the two refusals the loader already names), so the parenthetical becomes the rule it points at."
  },
  {
    "severity": "minor",
    "location": "Issued entries, mint rule step 5 — \"mints `steam-<appid>`, spelled from the queried key's external id … appid-unique\"",
    "problem": "new: spelling the fallback from the client's digits makes the id depend on the queried spelling of one appid. `steam/42` and `steam/0042` are both admissible (`keys.rs:73` bounds length, token set and path safety only — I verified there is no appid bound), and the design never pins how the \"one exact-`appid` probe\" treats them. Numeric match: one snapshot record mints `steam-42` and `steam-0042`, two held ids for one game, so \"appid-unique\" is false and step 4's suffix has no stated appid source (`mint.rs::mint_id(display_name, appid, held)` carries one `appid` for both jobs). Textual match: `steam/0042` is not present in the snapshot, takes the second bullet, and answers a false `200 []` for an appid the catalog holds — the claim `027-L3` exists to police.",
    "fix": "Name the canonical spelling in step 5 and at the probe: spell the fallback from the snapshot record's `appid` (or refuse a non-canonical decimal external id before the probe, saying which); keep `MAX_DISCOVERY_SLUG_OCTETS` as the cited arithmetic, which is verified correct."
  },
  {
    "severity": "nit",
    "location": "Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k *worst-legal* records at ~240 octets\"",
    "problem": "re-raised (r1 nit, narrowed but not closed): ~240 octets costs the name at its 128-octet *decoded* bound, but the response body carries JSON string escapes, so a quote- or backslash-heavy name serializes at up to 2× and a record lands near 340 octets — a legal 50k page of such names is ~17 MiB, over the cap, and the design must refuse that page and fail the pull (a startup refusal) on a legal response.",
    "fix": "One clause saying which the cap is chosen for: either double the name allowance for escapes in the derivation (⌈2 × 128⌉) or pin `max_results` from the worst-escaped record."
  },
  {
    "severity": "nit",
    "location": "Hosted mode / Risks / Deferred — \"`dropped_appids` is not a health fact — the pull writing the list prints it\" beside \"its `appid` is listed in the snapshot's `dropped_appids`, which is the operator's signal to pursue that amendment\"",
    "problem": "new: with the fact off `health` (rightly, per `027-C3`) and off `stats` (Deferred), the operator's only channels are one stdout line at publish time and opening the snapshot file; Rollout 6 enumerates \"reading the retired list's size against its cap\" but never the dropped list, so the signal the design leans on twice has no runbook line and no restart-surviving surface.",
    "fix": "Add the dropped list to Rollout 6's reading duty in the same clause as the retired-list size; no new surface needed."
  },
  {
    "severity": "nit",
    "location": "The store pull — \"sized in §Budgets to cover this grid with its retries, so a slow store costs a late snapshot, not a guaranteed failure\"",
    "problem": "new: true for the background delta, not for the pre-bind full pull, where the same 1,872 s grid ends in `027-C3`'s named startup refusal after ~31 minutes of a process that has not bound. This pass deleted the sentence that bounded that path (\"the pre-bind full pull is paced by the socket timeouts and a seconds-scale retry, never an hour-long sleep\") while raising `STEAM_PULL_DEADLINE_SECONDS` from 900 to 1,920, so nothing states what a slow store costs an operator's start.",
    "fix": "Scope the clause — \"costs a late delta; a full pull that runs the grid is the named `steam pull deadline` startup refusal\" — and keep the deadline value, which is arithmetically right."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/correctness-r1c5.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"§The store pull / §Risks / §Deferred — \"The publish line counts the names it clamped (§Risks, §Deferred)\" beside \"the pull's clamp count is the operator's signal\"","problem":"re-raised (same class as round 1's dropped-list signal nit, which the revision answered by substituting a new signal for the deleted one). The signal named in three sections has no surface the design defines and no runbook duty: \"the publish line\" appears nowhere else in the document — §The store pull names only the startup `registry hosted listening` line and the refresh thread's failure warning line — the count is not in `health` (correctly excluded, since `027-C3` fixes that fact to `{games, fetched_at}`) nor in `stats` (counts only, and its own item is Deferred), and Rollout 6's reading duty enumerates \"reading the retired list's size against its cap\" but never the clamp count. So the §Deferred amendment trigger (\"if a store name breaches it in earnest\") is observable only on the stdout of the run that pulled, does not survive a restart, and no runbook line tells the operator to read it.","fix":"One clause in Rollout 6's reading list — the clamp count on the pull's publish line, naming where it is printed — beside the retired-list size it already carries; no new surface needed."},{"severity":"nit","location":"§Budgets / §Risks — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — the response *read* budget for one page … not a worst case, since over-bound store names make a page's true size unbounded and the over-cap page is the residual §Risks carries\"","problem":"re-raised (round 1's arithmetic nit, narrowed but not closed by the relabel). The residual is attributed to one cause only. Over-bound names are not the only way a *legal* page breaches 12 MiB: the body carries JSON string escapes, so a name at its 128-octet decoded bound serializes at up to 2x when quote- or backslash-heavy (~340 octets per record, ~17 MiB for the pinned 50k page) and more for characters the encoder emits as `\\u00XX`. §Risks also states the cost as only \"a page refusal that keeps the previous snapshot until `027-B2`'s age bound\" — the steady-state half; on a cold instance with no snapshot the same named page refusal is the named startup refusal, and the operator's only way to bind is `--no-steam`, i.e. every Steam key answering `503` and every bound CLI scan failing closed (§Compatibility matrix). The cap's arithmetic itself checks out: 12,582,912 > 50,000 x ~240.","fix":"Extend the clause §Deferred's item already asks for: name JSON-escaped serialization, not only over-bound names, as the second way a legal page breaches the read budget, and say the cold-start branch is the named startup refusal; keep the constant's value unchanged."},{"severity":"nit","location":"§The store pull / Test strategy #6 — \"A name that normalizes to empty stays empty: its label renders blank and its identity is mint step 3's `steam-<appid>`\" and \"an all-whitespace or all-control title stores empty\"","problem":"new (introduced by this revision; round 1 dropped such a record instead). The blank is not confined to the registry: that `name` is what the mint persists into the issued record and what the candidate contract serves, and `apps/discovery-cli/src/catalog.rs:46-47` documents the candidate `name` as \"the name the prompt shows\", accepting any string there (`catalog.rs:393-401`). A store title consisting only of control or whitespace characters therefore reaches a user's confirmation prompt as an invisible option, chosen by remote content — the one piece of attacker-chosen content this pass lets through to a user's decision point. (No validation cliff: `corpus.rs:424` bounds `name` only above, so the issued document still loads, and identity is unique via mint step 3.)","fix":"One clause giving the empty case a label the registry owns instead of nothing: store `steam-<appid>` (already mint step 3's spelling, no new constant) as the `name` when normalization yields empty, and let #6 assert that in place of \"stores empty\"."}]}
```
