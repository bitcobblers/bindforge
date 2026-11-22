# Transcript: design-review:performance r1c2

## Provenance
- stage: design-review:performance (round r1c2)
- agent: review/design-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:53:19.922Z
- ended: 2026-11-22T04:04:11.791Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":46547,"output":24336,"reasoning":0,"cache.read":394944,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "§Deferred /name-bound entry/ with §The steam snapshot resolver and §Risks — \"until then the breaching record is dropped and the snapshot publishes `incomplete`, which is what keeps the bound from being an availability cliff\"",
    "problem": "The r1 drop-and-mark substitution moves the availability cliff rather than removing it, and the quoted claim is false as the changed text now stands. The blast radius is the whole steam miss path for the entire instance for as long as the offending record stays listed: policy step 5 says \"`incomplete` > 0: no miss is certifiable, so every steam key that is not an appid hit takes the named `503` `steam snapshot incomplete`\", and the newly revised Compatibility-matrix client row states the consequence — \"The new `503`s render as the shipped status-line-only refusal *and fail the whole scan closed*\". Nothing clears the mark: a full pull re-encounters and re-drops the same name, a delta only \"bumps `incomplete`\", deleting the snapshot re-pulls the same catalog, `--no-steam` is strictly worse, and re-sizing the bound is deferred — leaving a visible but unrepairable permanent state. The trigger is content the design itself calls attacker-chosen (A11), and no attacker is needed: the bound is … [elided by the checkpoint cap]",
    "fix": "Keep the drop but scope the uncertifiability to what was actually dropped: publish a bounded list of dropped appids beside the count, refuse `200 []` only for keys naming those appids, and let the whole-list posture apply only when that list exceeds its own cap. Alternatively, if the count-only posture stands, delete the \"availability cliff\" clause and price the posture in §Risks with its scope (all steam misses), trigger (one over-bound name, remote or natural), permanence (until the deferred bound amendment) and recovery path, replacing the Deferred sentence's reassurance."
  },
  {
    "severity": "minor",
    "location": "§The store pull / §Budgets — \"`STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS = 900`\"",
    "problem": "The deadline does not enclose the pull the same family authorizes, so it silently overrides `027-B4`'s page budget instead of bounding it. At the nominal values the design's own worst legal pull is 12 pages × (3 × (2 s + 30 s) + 2 × 30 s) = 1,872 s, already ~2× the 900 s deadline; and the shipped shape these constants copy (apps/registry/src/upstream.rs:97 `set_read_timeout(Some(UPSTREAM_READ_TIMEOUT))` with the body read to completion) bounds one read, not one body, so a page's wall time is not 32 s and the deadline is the only wall-clock bound in the family — a pull honestly progressing at ~110 KiB/s across 12 ≤8 MiB pages is killed. Path consequences are unnamed: pre-bind a killed pull is the startup refusal (a slow-link operator cannot boot); on the refresh thread it keeps the snapshot aging toward `027-B2`'s conversion of misses to `503`s.",
    "fix": "State the relation STEAM_PULL_DEADLINE_SECONDS ≥ STEAM_REFRESH_MAX_PAGES × (STEAM_PAGE_ATTEMPTS × (STEAM_CONNECT_TIMEOUT + STEAM_READ_TIMEOUT) + (STEAM_PAGE_ATTEMPTS − 1) × STEAM_PAGE_RETRY_SECONDS) and pick numbers that satisfy it (1,920 s, or a cut in per-page pacing), and name the deadline-killed pull as a refusal distinct from the over-`STEAM_REFRESH_MAX_PAGES` one."
  },
  {
    "severity": "minor",
    "location": "§The store pull with §The steam snapshot resolver — \"a delta merges by `appid` (replace, or insert keeping ascending order) and re-persists\", against \"In memory the snapshot is a sorted vector plus an appid index\"",
    "problem": "Read literally, a delta of m changed appids into the n ≈ 200k sorted vector is m ordered inserts, each memmoving up to the whole structure and re-shifting the appid index beside it. The design names the read probe (\"one exact-`appid` probe\") and, for the retired list, the mutation shape and its cost (\"a delete costs two bounded document rewrites\"), but for the one daily bulk merge it names neither algorithm nor transient. §Budgets' new aggregate counts only the mint (\"a mint adds one model plus one serialized buffer\"), understating the process's largest transient: a merge holds the old snapshot model, the merged model, the ≤32 MiB serialize buffer and the current ≤8 MiB page body at once.",
    "fix": "One clause: the delta publishes via a single linear two-way merge of the loaded vector and the pulled changes plus one index build (never one insert per changed appid), with that transient named in the §Budgets aggregate sentence beside the mint's multiple."
  },
  {
    "severity": "nit",
    "location": "§Budgets — \"the worst pull is 12 pages × 3 attempts = 36 calls, ~2,800× inside A6's 100,000/day, and ~115× even counting the refresh thread's daily 24 × 36 = 864 — no separate quota machinery\"",
    "problem": "The r1 restatement fixed the attempt shape but enumerates only one of the design's three pull initiators. §Hosted mode gives `registry query` \"same startup as `serve` including load-or-pull\", and the startup rule \"load the snapshot if present, loadable, and fresh, else pull in full\" means every `serve` start and every `query` against a missing or aged snapshot spends 12 calls (36 with retries) with none of the refresh thread's 3,600 s spacing; a supervisor restarting a crashing `serve` has no gap at all.",
    "fix": "One clause stating that the counted spend is the refresh thread's and that startup pulls are bounded by process starts, naming the restart-loop case as the runbook's restart-policy line."
  },
  {
    "severity": "nit",
    "location": "§Issued entries (retired ids) — \"`MAX_RETIRED_IDS_OCTETS = 33_554_432` gated at load (metadata-gated) and against the buffer a write renames ... A delete whose retired append would breach the cap is the named refusal `retired id list full` and mutates nothing\"",
    "problem": "This is the one budget whose breach is its own first signal: `stats` deliberately does not report the retired count (§Deferred), and unlike the other three documents the new aggregate sentence gives no entry-count reading of its octet cap. The list also only grows for the instance's life (every delete, including every promotion), so it is the only cap curation cannot relieve.",
    "fix": "One clause giving the id count the cap implies (≈250k ids at the 128-octet id bound, >1M at typical ids), making the refusal's likelihood an operator-judgementable figure, with the runbook carrying the file-size check."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/performance-r1c2.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "§The store pull — \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\" (with §Budgets' `MAX_DROPPED_APPIDS = 1_024`, §The steam snapshot resolver's \"the list merges as a set on delta … and resets only on a full-pull replace\", §Risks' \"a 43-character CJK or 32-character emoji title reaches it\", §Rollout 6's \"forcing a full refresh by deleting the snapshot\")", "problem": "New mechanism introduced by this revision. The cap's breach re-imports, by accumulation, the posture the drop exists to avoid (\"failing the pull over one is remote, permanent denial of the supply\"). The list only grows on delta — an entry clears only when a delta re-delivers that appid with a within-bound `name`, and \"Deltas never see removals\"; a failed pull publishes nothing, so `fetched_at` never advances, so the next delta asks the same `if_modified_since` window and re-sees the same over-bound record, making the breach permanent rather than transient. The consequence is the r1 blast radius reached by count: the snapshot ages past `STEAM_SNAPSHOT_MAX_AGE_SECONDS`, the policy's last bullet turns every steam miss into a named `503`, and per §Compatibility matrix a bound CLI \"fail[s] the whole scan closed\". Recovery is refused by the same rule: entries leave the list only via a delta showing them within bound, so a list at 1,024 is a subset of the catalog's currently over-bound records and implies a full pull that also breaches — the runbook's recovery lever is refused too, leaving `--no-steam` (the terminal state). The loader carries the same cap (\"a `dropped_appids` longer than `MAX_DROPPED_APPIDS`\"), so a drifted file plus a refusing pull is a startup refusal. Signals are thin: r2 removed the count from health (\"`dropped_appids` is not a health fact — the pull writing the list prints it\"), `stats` is `{games, source_keys, revision, issued}`, and Rollout 6 watches only \"the retired list's size against its cap\". Likelihood is low (A10 sets 128 as \"one octet above the longest surveyed store title\"), so this is priced rather than gated.", "fix": "Make the breach degrade instead of denying: replace \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\" with publishing the capped list under the whole-list posture (the r1 posture retained only here, where the unenumerated set genuinely is unknown), so a breach costs miss certification, not the supply or the recovery path. If fail-the-pull stands, price it in the §Risks row and §Budgets with scope (all steam misses via the age bound), trigger (≥1,025 over-bound names in one inter-full-pull interval, cumulative on deltas), permanence (a failed delta never advances `fetched_at`) and recovery, and add the list's length to Rollout 6's watch line beside the retired file's size."}, {"severity": "minor", "location": "§Budgets — \"on top of it a mint holds one model plus one serialized buffer, and a delta merge the old snapshot, the merged model, one serialize buffer and the current page body\"", "problem": "The sentence rewritten to answer my r1 transient finding still enumerates an incomplete family: it omits the full pull, the process's largest transient, whose input cap this same revision raised (`MAX_STEAM_PAGE_OCTETS` 8,388,608 → 12,582,912, justified because \"the pinned `max_results` can answer that page, not the ~150-octet mid-estimate\" — that arithmetic is sound: 50k worst-legal records at ~217–230 octets ≈ 11.4 MiB). But 12 pages × 12.58 MiB is a 151 MiB admissible input envelope against one gate on the other side, `027-B1`'s 33,554,432 publish-time check: the pull may legitimately read ~4.7× more record bytes than the document may hold, and pays the whole read before refusing. A full pull accumulates every page before publishing (it \"replaces the snapshot\"), so its peak is the model over up to 600k `{appid, name}` records plus the appid index plus the current page body, on top of the corpus/issued/retired documents boot already loaded (16+64+32 MiB at caps). \"144 MiB of JSON … as the boot load ceiling\" is a load bound being read as the process's memory story.", "fix": "One clause in the same sentence naming the full pull's transient and its bound — check the `027-B1` publish ceiling as pages accumulate, or state that ≤12 pages of records is the process's largest transient — so all three writer transients (mint, delta merge, full pull) are on the table that §Deferred's batched-writes deferral defers against."}, {"severity": "nit", "location": "§The store pull — \"sized in §Budgets to cover this grid with its retries, so a slow store costs a late snapshot, not a guaranteed failure\"; §Budgets — \"`STEAM_PULL_DEADLINE_SECONDS = 1_920` ≥ the grid that family authorizes (`12 × (3 × 32 + 2 × 30) = 1,872)`\"", "problem": "The relation and the named `steam pull deadline` refusal close my r1 finding (12 × 156 = 1,872 ≤ 1,920 checks out), but the grid charges each attempt `STEAM_CONNECT_TIMEOUT + STEAM_READ_TIMEOUT` = 32 s as though those bounded a page, while the framing the design says it copies hand-rolls them as socket timeouts (`apps/registry/src/upstream.rs:97` `set_read_timeout(Some(constants::UPSTREAM_READ_TIMEOUT))`, body read to completion), which bound one `read` call — a steadily trickling body never trips the 30 s bound, so a page's legal wall time is unbounded and the grid is a floor, not a ceiling. Its real precondition is ~393 KiB/s per worst-legal page (12,582,912/32 s), leaving the deadline as the only wall-clock bound; a typical catalog (≈11–16 MiB) finishes with room, so it bites only worst-legal pages on a slow link, where the honest outcome is the refusal. Second half: on the pre-bind path a deadline-killed pull is a startup refusal, so the unstated figure is that `serve`, and `query` with its \"same startup as `serve` including load-or-pull\", can block up to 1,920 s before binding.", "fix": "One clause stating whether the two timeouts bound a read or a page (and, if a read, that the grid is a floor and the deadline is the only page-total bound), plus one naming the worst-case pre-bind block as ≤1,920 s before bind, so the runbook's start-timeout and restart-policy line has a number to sit against."}]}
```
