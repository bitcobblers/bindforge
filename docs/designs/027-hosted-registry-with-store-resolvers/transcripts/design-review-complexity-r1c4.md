# Transcript: design-review:complexity r1c4

## Provenance
- stage: design-review:complexity (round r1c4)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:20:18.633Z
- ended: 2026-11-22T04:23:01.488Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":43857,"output":17073,"reasoning":0,"cache.read":583894,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c4: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 1d28899313549497baa7bbad8c1ecb0c53979c5c d105d78caaf272e507760abf17dedfdd966810b6` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c4.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "§The store pull — \"The rule is the shipped `discovery::friendly_name` one, through the existing `discovery` dependency (A7) — whitespace collapse then the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`, with control bytes dropped in the same pass\" (also A7's new clause, §Options \"an over-bound name is normalized, not dropped\", §The hosted lookup policy step 5 \"every record the pull was handed is in it\", §The steam snapshot resolver's loader refusal, §Test strategy #6, Rollout 3)",
    "problem": "new. The clause is false against the code A7 marks `verified`: `crates/discovery/src/game.rs:102-113` is `split_whitespace().join(\" \")` then an `is_char_boundary` clamp — those two operations only. `char::is_whitespace` covers `\\t \\n \\u{b} \\u{c} \\r` and U+001C–U+001F/U+0085, so NUL, `\\u{1}`–`\\u{8}`, `\\u{e}`–`\\u{1b}` (ESC), DEL and C1 survive into the snapshot name. The design's own loader refuses exactly that content — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (… the pull normalizes one before writing it)\" — so with the shipped rule as cited, the instance publishes a document its own loader will not read: every restart takes the \"else pull in full\" branch, and a restart during a store/key outage (the §Risks A6 row) is a named startup refusal instead of serving the last snapshot — the remote-chosen denial of supply r3 exists to delete, back through the loader. §Test strategy #6 (\"a bound-breaking or control-byte `name` is clamped to the `013-B4`-legal form\") also… [elided by the checkpoint cap]",
    "fix": "Delete \"with control bytes dropped in the same pass\" and name the one line the registry owns instead: the pull seam drops `char::is_control()` characters before the `friendly_name` clamp (a std filter — no constant, no document, no policy branch, no ADR amendment), mirrored in A7 and Rollout 3's \"`friendly_name` name normalization\". Keep the loader's control-byte refusal as the contract-document discipline it is. One clause plus one named filter replaces the false reuse; nothing else r3 removed needs restoring."
  },
  {
    "severity": "nit",
    "location": "§Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — the response *read* budget for one page, 50k records at ~240 octets\"",
    "problem": "new. The relabel resolves my r1 \"worst-legal\" finding, but the derivation still reads as a product that does not produce the constant: 50,000 × ~240 = 12,000,000, while 12,582,912 is 12 MiB (≈252 octets per record). A reader re-deriving the budget gets a different number than the shipped one.",
    "fix": "Word it as the sum it is: \"— 12 MiB, which the 50k-records-at-~240-octets estimate sits inside —\"."
  },
  {
    "severity": "nit",
    "location": "§The store pull (and §Test strategy #6) — \"whitespace collapse then the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`\"",
    "problem": "new. No disposition for a name that normalizes to empty: a whitespace-only title (and, once the control strip is owned, an all-control title) clamps to `\"\"`, which r2's pass-through could not produce. The identity is safe — mint step 3 mints `steam-<appid>` — but the issued record and the served candidate carry `name: \"\"`, where the shipped ingest rule treats empty as a fault (\"`name` is outside the 1-128-octet bound\", apps/registry/src/upstream.rs:247) while the record gate the issued document reuses checks only the upper bound (apps/registry/src/corpus.rs:424), so nothing refuses it.",
    "fix": "One clause: \"a name that normalizes to empty is stored empty — its label renders blank and its identity is step 3's `steam-<appid>`\" — or substitute a single space. No mechanism either way."
  },
  {
    "severity": "nit",
    "location": "A7 / §The store pull / Rollout 3 — \"`discovery::friendly_name`\"",
    "problem": "new. Not the shipped path: `crates/discovery/src/lib.rs` publishes `pub mod game;` with no crate-root re-export, and every shipped caller writes `use discovery::game::{… friendly_name}`. The design the code review pins misnames the import.",
    "fix": "Spell it `discovery::game::friendly_name` in A7's basis and Rollout 3."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c4.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "nit", "location": "§The store pull — \"The publish line counts the names it clamped (§Risks, §Deferred)\"", "problem": "new (from the revised text). The revision made the registry own a second normalization — the `char::is_control()` strip — but the only count anywhere is still the clamp count, and §Risks (\"the pull's clamp count is the signal\") and §Deferred (\"the pull's clamp count is the operator's signal\") both name that count as the sole operator signal feeding the deferred `013-B4`/`027-C3` bound re-sizing decision. A store answering with control-byte-laden titles is therefore normalized invisibly, and a name that becomes in-budget because it was stripped never reaches the count that decides whether the bound needs re-sizing.", "fix": "One word: \"The publish line counts the names it normalized (clamped or control-stripped)\" — or, if the split is deliberate, one clause here saying control-strip events are deliberately not counted as bound pressure. No new counter, constant, or log line either way."}, {"severity": "nit", "location": "§Options \"Partial snapshot serving\" — \"of the games-only set (§The store pull normalizes, never drops)\"; §Risks `013-B4` row — \"keeps the record (§The store pull owns the rule)\"", "problem": "new (introduced by this revision). Two parentheticals where the `§` sigil binds to a clause instead of a heading: no section is named \"The store pull normalizes, never drops\" or \"The store pull owns the rule\", so the cross-reference resolves only by guesswork, unlike every other `§`-ref in the document.", "fix": "Put the assertion outside the sigil: \"(§The store pull: it normalizes, never drops)\" and \"(the rule is §The store pull's)\". No text deleted."}]}
```
