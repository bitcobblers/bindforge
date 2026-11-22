# Transcript: design-review:complexity r1c3

## Provenance
- stage: design-review:complexity (round r1c3)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:13:47.113Z
- ended: 2026-11-22T04:16:40.253Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":57081,"output":18347,"reasoning":0,"cache.read":504423,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c3: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 19dcdcb170ba3f06749f7a84641e94b5839d1f77 1d28899313549497baa7bbad8c1ecb0c53979c5c` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c3.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "§The steam snapshot resolver — \"`dropped_appids` is the ascending, duplicate-free list of the `appid`s the pull dropped for the `013-B4` name bound; normally empty\" (+ §The hosted lookup policy step 5 — \"The key's appid is listed in `dropped_appids`: the named `503` `steam snapshot incomplete`\")",
    "problem": "new. The r2 answer to the over-bound-name problem adds a bookkeeping subsystem where the repo already ships the rule for this data class, and its answer to \"one store title is too long\" is to make that game permanently unanswerable. r2 adds: a field in a contract document the ADR fixes (027-C3's shape {version, fetched_at, apps}), costing a Rollout-1 `amended-by` on the ADR this run implements; a new constant; set-merge/self-clear/reset semantics across deltas; four new loader refusals plus two Test-strategy rows; and a new policy branch whose served consequence, per §Compatibility matrix (\"a bound CLI fails the whole scan closed\"), is that a user owning a long-named title can never complete `discovery list` — the appid is not in `apps`, so the mint path cannot fire and the answer is `503` forever, against 027-S1's \"every other steam key answers from the snapshot on day one\".",
    "fix": "Use the shipped rule: `crates/discovery/src/game.rs::friendly_name` already clamps a store display name to \"the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`\" (\"a truncated display string costs a shorter label, never a different identity\"), and `apps/registry` already depends on `discovery` (Cargo.toml:14), so it is an import, not a dependency. Normalize an over-bound (or control-byte) name at the pull seam into the 013-B4-legal form instead of dropping it: the appid stays in the snapshot, no miss needs un-certifying, 027-C3 ships verbatim with no amendment, and the mint's existing steps absorb the rest (the fold never exceeds the name's octets; two titles sharing a 128-octet prefix collide at step 4 and fall to step 5's `steam-<appid>`; 027-L2 plus policy step 2 mean derivation is paid once per key). Replaces by name: the drop rule and `dropped_appids` in §The store pull, §The steam snapshot resolver, policy step 5's third bullet, §Risks row 7, §Deferred's fourth bull… [elided by the checkpoint cap]"
  },
  {
    "severity": "major",
    "location": "§Budgets — \"`MAX_DROPPED_APPIDS = 1_024` (its breach is the named pull failure, §The store pull)\" (+ §The store pull — \"a list that would exceed `MAX_DROPPED_APPIDS` is itself a named pull failure\", \"the list merges as a set on delta … and resets only on a full-pull replace\")",
    "problem": "new. The cap's number and its breach consequence together cancel the property the mechanism was added for — the same mid-estimate error this revision just corrected one line earlier for `MAX_STEAM_PAGE_OCTETS`. One page the design calls legal carries 50,000 dropped records: a 50,000-record page of 129-octet names is ~207 octets/record ≈ 10.4 MB, inside `MAX_STEAM_PAGE_OCTETS = 12_582_912` and inside `max_results=50000` — so a single page exceeds the cap ~49×, and a full pull's worst legal drop count is 12 × 50k = 600k, ~590×. In steady state the list only grows: the refresh thread delta-pulls and never full-pulls, and a restart with a fresh loadable snapshot loads rather than pulls, so a reset needs the operator to delete the file. On breach the design returns to the r0 posture the Revision log rejected — a named pull failure, i.e. a stale snapshot aging into `503` for every steam miss, and a startup refusal to bind — reached by remote-chosen content, which the same section says the me… [elided by the checkpoint cap]",
    "fix": "Adopt the clamp above and the constant disappears; if the list stays, make saturation a posture and not a pull failure — while `dropped_appids` is full, no appid miss is certifiable (exactly the r1 `incomplete` posture, as the overflow case) — and size the cap from the page budget rather than a guess. Either change replaces the named pull failure and adds no mechanism."
  },
  {
    "severity": "minor",
    "location": "§Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — 50k *worst-legal* records at ~240 octets (A17's four fields, `name` at `MAX_NATIVE_NAME_OCTETS`), because the pinned `max_results` can answer that page, not the ~150-octet mid-estimate\"",
    "problem": "new. The re-sizing is right for the mid-estimate reason and wrong for the \"worst-legal\" claim: its premise (`name` at 128 octets) is exactly what §The store pull's drop case denies — names over the bound are the content the design expects to arrive, so no finite page cap is worst-legal and a page heavy with over-bound names is legal content that trips a named page refusal and fails the pull. The constant is a read budget wearing a worst-case label, and the label is what a reader trusts when the budgets are re-derived.",
    "fix": "One clause: name it the response read budget it is, record that over-bound names make the true worst case unbounded so an over-cap page is the residual §Risks carries (the same posture as the previous finding's fix), and drop the word \"worst-legal\". A relabel plus one risk clause, no new mechanism."
  },
  {
    "severity": "nit",
    "location": "§Issued entries — \"safe because the append is a set insert, so an already-present id is a no-op\" beside \"A delete whose append would breach the cap is the named refusal `retired id list full` and mutates nothing\"",
    "problem": "new. Read literally, \"would breach\" tests the candidate entry, so a no-op append against an at-cap list refuses the very retry the same paragraph promises completes the removal.",
    "fix": "Three words: the cap is checked against \"the buffer the write renames\" — the check the paragraph already names for the other documents."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c3.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§The store pull — \"The rule is the shipped `discovery::friendly_name` one, through the existing `discovery` dependency (A7) — whitespace collapse then the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`, with control bytes dropped in the same pass\" (also A7's new clause, §Options \"an over-bound name is normalized, not dropped\", §The hosted lookup policy step 5 \"every record the pull was handed is in it\", §The steam snapshot resolver's loader refusal, §Test strategy #6, Rollout 3)","problem":"new. The clause is false against the code A7 marks `verified`: `crates/discovery/src/game.rs:102-113` is `split_whitespace().join(\" \")` then an `is_char_boundary` clamp — those two operations only. `char::is_whitespace` covers `\\t \\n \\u{b} \\u{c} \\r` and U+001C–U+001F/U+0085, so NUL, `\\u{1}`–`\\u{8}`, `\\u{e}`–`\\u{1b}` (ESC), DEL and C1 survive into the snapshot name. The design's own loader refuses exactly that content — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (… the pull normalizes one before writing it)\" — so with the shipped rule as cited, the instance publishes a document its own loader will not read: every restart takes the \"else pull in full\" branch, and a restart during a store/key outage (the §Risks A6 row) is a named startup refusal instead of serving the last snapshot — the remote-chosen denial of supply r3 exists to delete, back through the loader. §Test strategy #6 (\"a bound-breaking or control-byte `name` is clamped to the `013-B4`-legal form\") also pins behavior `friendly_name` does not have, so the suite fails or the implementer adds the strip step the document says is already shipped. The r3 deletion's premise covers the name-bound half and not the control-byte half; A7's own description of the shipped rule (\"whitespace collapse plus the longest `char`-boundary prefix\") is accurate and contradicts the body clause.","fix":"Delete \"with control bytes dropped in the same pass\" and name the one line the registry owns instead: the pull seam drops `char::is_control()` characters before the `friendly_name` clamp (a std filter — no constant, no document, no policy branch, no ADR amendment), mirrored in A7 and Rollout 3's \"`friendly_name` name normalization\". Keep the loader's control-byte refusal as the contract-document discipline it is. One clause plus one named filter replaces the false reuse; nothing else r3 removed needs restoring."},{"severity":"nit","location":"§Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — the response *read* budget for one page, 50k records at ~240 octets\"","problem":"new. The relabel resolves my r1 \"worst-legal\" finding, but the derivation still reads as a product that does not produce the constant: 50,000 × ~240 = 12,000,000, while 12,582,912 is 12 MiB (≈252 octets per record). A reader re-deriving the budget gets a different number than the shipped one.","fix":"Word it as the sum it is: \"— 12 MiB, which the 50k-records-at-~240-octets estimate sits inside —\"."},{"severity":"nit","location":"§The store pull (and §Test strategy #6) — \"whitespace collapse then the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`\"","problem":"new. No disposition for a name that normalizes to empty: a whitespace-only title (and, once the control strip is owned, an all-control title) clamps to `\"\"`, which r2's pass-through could not produce. The identity is safe — mint step 3 mints `steam-<appid>` — but the issued record and the served candidate carry `name: \"\"`, where the shipped ingest rule treats empty as a fault (\"`name` is outside the 1-128-octet bound\", apps/registry/src/upstream.rs:247) while the record gate the issued document reuses checks only the upper bound (apps/registry/src/corpus.rs:424), so nothing refuses it.","fix":"One clause: \"a name that normalizes to empty is stored empty — its label renders blank and its identity is step 3's `steam-<appid>`\" — or substitute a single space. No mechanism either way."},{"severity":"nit","location":"A7 / §The store pull / Rollout 3 — \"`discovery::friendly_name`\"","problem":"new. Not the shipped path: `crates/discovery/src/lib.rs` publishes `pub mod game;` with no crate-root re-export, and every shipped caller writes `use discovery::game::{… friendly_name}`. The design the code review pins misnames the import.","fix":"Spell it `discovery::game::friendly_name` in A7's basis and Rollout 3."}]}
```
