# Transcript: design-review:complexity r1c2

## Provenance
- stage: design-review:complexity (round r1c2)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:53:19.924Z
- ended: 2026-11-22T04:05:23.622Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":53739,"output":28138,"reasoning":0,"cache.read":572345,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "§Issued entries — \"The mint rule (027-C5, restated in full)\", step 5 — \"A mint whose chosen id exceeds `MAX_SLUG_OCTETS` mints `steam-<appid>`: appid-unique, ≤ 26 octets under the closed key grammar's appid bound\"",
    "problem": "re-raised (narrowed from r0 #1). The bound this clause rests on is not stated anywhere in the design or the repo. The registry's whole key rule is apps/registry/src/keys.rs `key_is_admissible`: a 128-octet cap on the joined key plus `Source::spells_lookup_key` (crates/discovery/src/game.rs:84), documented as \"shape only\" requiring merely a non-empty external id — no decimal rule, no appid-length rule; a 122-octet external id is admissible (the shipped test pins `steam/` + 122 `a`s as admitted). The only appid-digit bound in the tree is `MAX_APPID_DIGITS = 20` (crates/discovery/src/steam_common.rs:26), a private const gating the client-side ACF emission seam, not the grammar and not visible to apps/registry. The snapshot's own rules refuse only \"an `appid` that is not a decimal integer\", so a decimal appid of any digit count fits a 32 MiB file and loads; `steam-<appid>` is 7 + digits octets, so the rule's terminal case is not provably ≤ MAX_SLUG_OCTETS = 128 (a 122-digit appid gives 129… [elided by the checkpoint cap]",
    "fix": "Make the bound the design's own named rule instead of a citation: add an appid digit cap (the 20-digit figure `steam_common.rs` already uses, restated as this design's budget) to §The store pull's record rules and to the snapshot loader's named refusals beside the existing \"not a decimal integer\" refusal it shares, then keep step 5's \"≤ 26 octets\" as arithmetic over that stated cap. This replaces the phrase \"under the closed key grammar's appid bound\" with one integer compare on a field the pull and the load already inspect; no new stage. Equal-size alternative: drop the numeric claim and make step 5's terminal case \"an id still over `MAX_SLUG_OCTETS` after substitution is the named `503`\", which is sound with no appid bound at all."
  },
  {
    "severity": "minor",
    "location": "§Issued entries — \"Retired ids (`027-L2`)\" — \"**Every** delete — issued or corpus — is two persists under one mutation-lock hold, retired first\"",
    "problem": "new (introduced by this revision; r0 said \"in one mutation-lock persist\"). This is the only place in the design where on-disk state can be half-applied, and the resulting state is unnamed: if the process dies between the two renames the id sits in `retired-ids-v1.json` while still standing in its catalog document, so `games ∩ retired ≠ ∅` — a condition no rule governs, since the mint's steps 4/6 test \"held **or** retired by a different record\" and the loader's named refusals list neither the overlap nor its absence. The recovery path is precisely the named-`503` retry: re-issuing the refused delete re-appends an id the list already holds, and the document never says whether that is a no-op or a second entry in a file it defines as strictly \"in byte order\" with binary-searched membership. Test strategy #5 pins the append ordering and the cap refusal but not the half-applied state.",
    "fix": "One sentence plus one branch: the retired-list insert is a set insert (id already present ⇒ no-op, since a duplicate breaks the byte-order invariant the same paragraph fixes), and `games ∩ retired` is defined as the legal, recoverable state it is — the catalog record keeps answering, the id is held-and-retired, and the operator's retried delete completes the removal. Replaces the silence about the retry; adds no file and no mechanism."
  },
  {
    "severity": "nit",
    "location": "§Budgets — \"`STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS = 900`\"",
    "problem": "new. The worst-case attempt grid those three knobs describe — 12 pages × (3 × 30 s of retry gaps + 32 s of connect/read timeout) = 912 s — overshoots the deadline, so a fully degraded page sequence can never execute the retry plan the other two constants state. Nothing is broken (the outcome is the named refusal the section wants, and the r0 hour-long startup coupling is genuinely gone), but this family is the design's single readable statement of the pull's worst case and it does not close.",
    "fix": "Either raise the deadline to cover the grid (960) or add a half clause to §The store pull: \"the deadline may cut a page's remaining attempts\"."
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
Load the `design-review` skill and apply the complexity checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c2.md. That file is the only path this session
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
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "§The steam snapshot resolver — \"`dropped_appids` is the ascending, duplicate-free list of the `appid`s the pull dropped for the `013-B4` name bound; normally empty\" (+ §The hosted lookup policy step 5 — \"The key's appid is listed in `dropped_appids`: the named `503` `steam snapshot incomplete`\")", "problem": "new. The r2 answer to the over-bound-name problem adds a bookkeeping subsystem where the repo already ships the rule for this data class, and its answer to \"one store title is too long\" is to make that game permanently unanswerable. r2 adds: a field in a contract document the ADR fixes (027-C3's shape {version, fetched_at, apps}), costing a Rollout-1 `amended-by` on the ADR this run implements; a new constant; set-merge/self-clear/reset semantics across deltas; four new loader refusals plus two Test-strategy rows; and a new policy branch whose served consequence, per §Compatibility matrix (\"a bound CLI fails the whole scan closed\"), is that a user owning a long-named title can never complete `discovery list` — the appid is not in `apps`, so the mint path cannot fire and the answer is `503` forever, against 027-S1's \"every other steam key answers from the snapshot on day one\".", "fix": "Use the shipped rule: `crates/discovery/src/game.rs::friendly_name` already clamps a store display name to \"the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`\" (\"a truncated display string costs a shorter label, never a different identity\"), and `apps/registry` already depends on `discovery` (Cargo.toml:14), so it is an import, not a dependency. Normalize an over-bound (or control-byte) name at the pull seam into the 013-B4-legal form instead of dropping it: the appid stays in the snapshot, no miss needs un-certifying, 027-C3 ships verbatim with no amendment, and the mint's existing steps absorb the rest (the fold never exceeds the name's octets; two titles sharing a 128-octet prefix collide at step 4 and fall to step 5's `steam-<appid>`; 027-L2 plus policy step 2 mean derivation is paid once per key). Replaces by name: the drop rule and `dropped_appids` in §The store pull, §The steam snapshot resolver, policy step 5's third bullet, §Risks row 7, §Deferred's fourth bullet, Rollout 1's 027-C3 line, Rollout 3, Test #2/#3/#6/#13, `MAX_DROPPED_APPIDS`, and the pull's \"prints it\" line — one normalization sentence in their place. Counter stated honestly: `friendly_name`'s comment rests on \"nothing parses the result\", false here since the mint derives from this name; 027-L2 + policy step 2 + steps 4–5 are what make the clamp admissible in the registry."}, {"severity": "major", "location": "§Budgets — \"`MAX_DROPPED_APPIDS = 1_024` (its breach is the named pull failure, §The store pull)\" (+ §The store pull — \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\", \"the list merges as a set on delta … and resets only on a full-pull replace\")", "problem": "new. The cap's number and its breach consequence together cancel the property the mechanism was added for — the same mid-estimate error this revision just corrected one line earlier for `MAX_STEAM_PAGE_OCTETS`. One page the design calls legal carries 50,000 dropped records: a 50,000-record page of 129-octet names is ~207 octets/record ≈ 10.4 MB, inside `MAX_STEAM_PAGE_OCTETS = 12_582_912` and inside `max_results=50000` — so a single page exceeds the cap ~49×, and a full pull's worst legal drop count is 12 × 50k = 600k, ~590×. In steady state the list only grows: the refresh thread delta-pulls and never full-pulls, and a restart with a fresh loadable snapshot loads rather than pulls, so a reset needs the operator to delete the file. On breach the design returns to the r0 posture the Revision log rejected — a named pull failure, i.e. a stale snapshot aging into `503` for every steam miss, and a startup refusal to bind — reached by remote-chosen content, which the same section says the mechanism exists to prevent. §Risks row 7's \"impact Low\" holds only on the unstated premise that the cap is never breached.", "fix": "Adopt the clamp above and the constant disappears; if the list stays, make saturation a posture and not a pull failure — while `dropped_appids` is full, no appid miss is certifiable (exactly the r1 `incomplete` posture, as the overflow case) — and size the cap from the page budget rather than a guess. Either change replaces the named pull failure and adds no mechanism."}, {"severity": "minor", "location": "§Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k *worst-legal* records at ~240 octets (A17's four fields, `name` at `MAX_NATIVE_NAME_OCTETS`), because the pinned `max_results` can answer that page, not the ~150-octet mid-estimate\"", "problem": "new. The re-sizing is right for the mid-estimate reason and wrong for the \"worst-legal\" claim: its premise (`name` at 128 octets) is exactly what §The store pull's drop case denies — names over the bound are the content the design expects to arrive, so no finite page cap is worst-legal and a page heavy with over-bound names is legal content that trips a named page refusal and fails the pull. The constant is a read budget wearing a worst-case label, and the label is what a reader trusts when the budgets are re-derived.", "fix": "One clause: name it the response read budget it is, record that over-bound names make the true worst case unbounded so an over-cap page is the residual §Risks carries (the same posture as the previous finding's fix), and drop the word \"worst-legal\". A relabel plus one risk clause, no new mechanism."}, {"severity": "nit", "location": "§Issued entries — \"safe because the append is a set insert, so an already-present id is a no-op\" beside \"A delete whose append would breach the cap is the named refusal `retired id list full` and mutates nothing\"", "problem": "new. Read literally, \"would breach\" tests the candidate entry, so a no-op append against an at-cap list refuses the very retry the same paragraph promises completes the removal.", "fix": "Three words: the cap is checked against \"the buffer the write renames\" — the check the paragraph already names for the other documents."}]}
```
