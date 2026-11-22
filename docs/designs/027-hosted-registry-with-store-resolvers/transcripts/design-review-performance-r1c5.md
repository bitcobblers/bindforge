# Transcript: design-review:performance r1c5

## Provenance
- stage: design-review:performance (round r1c5)
- agent: review/design-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:30:48.054Z
- ended: 2026-11-22T04:36:02.314Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":36437,"output":12881,"reasoning":0,"cache.read":316915,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "minor",
    "location": "§The store pull — \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\" (with §Budgets' `MAX_DROPPED_APPIDS = 1_024`, §The steam snapshot resolver's \"the list merges as a set on delta … and resets only on a full-pull replace\", §Risks' \"a 43-character CJK or 32-character emoji title reaches it\", §Rollout 6's \"forcing a full refresh by deleting the snapshot\")",
    "problem": "New mechanism introduced by this revision. The cap's breach re-imports, by accumulation, the posture the drop exists to avoid (\"failing the pull over one is remote, permanent denial of the supply\"). The list only grows on delta — an entry clears only when a delta re-delivers that appid with a within-bound `name`, and \"Deltas never see removals\"; a failed pull publishes nothing, so `fetched_at` never advances, so the next delta asks the same `if_modified_since` window and re-sees the same over-bound record, making the breach permanent rather than transient. The consequence is the r1 blast radius reached by count: the snapshot ages past `STEAM_SNAPSHOT_MAX_AGE_SECONDS`, the policy's last bullet turns every steam miss into a named `503`, and per §Compatibility matrix a bound CLI \"fail[s] the whole scan closed\". Recovery is refused by the same rule: entries leave the list only via a delta showing them within bound, so a list at 1,024 is a subset of the catalog's currently over-bound record… [elided by the checkpoint cap]",
    "fix": "Make the breach degrade instead of denying: replace \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\" with publishing the capped list under the whole-list posture (the r1 posture retained only here, where the unenumerated set genuinely is unknown), so a breach costs miss certification, not the supply or the recovery path. If fail-the-pull stands, price it in the §Risks row and §Budgets with scope (all steam misses via the age bound), trigger (≥1,025 over-bound names in one inter-full-pull interval, cumulative on deltas), permanence (a failed delta never advances `fetched_at`) and recovery, and add the list's length to Rollout 6's watch line beside the retired file's size."
  },
  {
    "severity": "minor",
    "location": "§Budgets — \"on top of it a mint holds one model plus one serialized buffer, and a delta merge the old snapshot, the merged model, one serialize buffer and the current page body\"",
    "problem": "The sentence rewritten to answer my r1 transient finding still enumerates an incomplete family: it omits the full pull, the process's largest transient, whose input cap this same revision raised (`MAX_STEAM_PAGE_OCTETS` 8,388,608 → 12,582,912, justified because \"the pinned `max_results` can answer that page, not the ~150-octet mid-estimate\" — that arithmetic is sound: 50k worst-legal records at ~217–230 octets ≈ 11.4 MiB). But 12 pages × 12.58 MiB is a 151 MiB admissible input envelope against one gate on the other side, `027-B1`'s 33,554,432 publish-time check: the pull may legitimately read ~4.7× more record bytes than the document may hold, and pays the whole read before refusing. A full pull accumulates every page before publishing (it \"replaces the snapshot\"), so its peak is the model over up to 600k `{appid, name}` records plus the appid index plus the current page body, on top of the corpus/issued/retired documents boot already loaded (16+64+32 MiB at caps). \"144 MiB of JSON … a… [elided by the checkpoint cap]",
    "fix": "One clause in the same sentence naming the full pull's transient and its bound — check the `027-B1` publish ceiling as pages accumulate, or state that ≤12 pages of records is the process's largest transient — so all three writer transients (mint, delta merge, full pull) are on the table that §Deferred's batched-writes deferral defers against."
  },
  {
    "severity": "nit",
    "location": "§The store pull — \"sized in §Budgets to cover this grid with its retries, so a slow store costs a late snapshot, not a guaranteed failure\"; §Budgets — \"`STEAM_PULL_DEADLINE_SECONDS = 1_920` ≥ the grid that family authorizes (`12 × (3 × 32 + 2 × 30) = 1,872)`\"",
    "problem": "The relation and the named `steam pull deadline` refusal close my r1 finding (12 × 156 = 1,872 ≤ 1,920 checks out), but the grid charges each attempt `STEAM_CONNECT_TIMEOUT + STEAM_READ_TIMEOUT` = 32 s as though those bounded a page, while the framing the design says it copies hand-rolls them as socket timeouts (`apps/registry/src/upstream.rs:97` `set_read_timeout(Some(constants::UPSTREAM_READ_TIMEOUT))`, body read to completion), which bound one `read` call — a steadily trickling body never trips the 30 s bound, so a page's legal wall time is unbounded and the grid is a floor, not a ceiling. Its real precondition is ~393 KiB/s per worst-legal page (12,582,912/32 s), leaving the deadline as the only wall-clock bound; a typical catalog (≈11–16 MiB) finishes with room, so it bites only worst-legal pages on a slow link, where the honest outcome is the refusal. Second half: on the pre-bind path a deadline-killed pull is a startup refusal, so the unstated figure is that `serve`, and `query`… [elided by the checkpoint cap]",
    "fix": "One clause stating whether the two timeouts bound a read or a page (and, if a read, that the grid is a floor and the deadline is the only page-total bound), plus one naming the worst-case pre-bind block as ≤1,920 s before bind, so the runbook's start-timeout and restart-policy line has a number to sit against."
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
Load the `design-review` skill and apply the performance checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c5.md. That file is the only path this session
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
{"verdict": "approve", "findings": []}
```
