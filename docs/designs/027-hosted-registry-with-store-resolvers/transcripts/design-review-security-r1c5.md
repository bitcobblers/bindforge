# Transcript: design-review:security r1c5

## Provenance
- stage: design-review:security (round r1c5)
- agent: review/design-security
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:23:01.636Z
- ended: 2026-11-22T04:30:47.922Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":67592,"output":14740,"reasoning":0,"cache.read":748388,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff fcb66a83bd31cc82493e01e93f55b745bd37e25c d105d78caaf272e507760abf17dedfdd966810b6` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/security-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "minor",
    "location": "§The steam snapshot resolver (loader refusals) / §The store pull (response rules) — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound `027-C3` names — the pull never writes one, §The store pull)\"",
    "problem": "The r1 parenthetical \"the pull never writes one\" is unenforced for the control-byte half of the loader's name rule: `013-B4` is only the 128-octet budget (docs/adrs/013-game-manifests.md), and the pull's sole name disposition is the drop for a record \"whose `name` breaks the `013-B4` bound\". The shipped native-name grammar (crates/core/src/manifest.rs::check_native_name) additionally bans ASCII control characters, empty, and leading/trailing-space names. A store record with, say, a control character in its attacker-chosen display name (A11) passes every pull rule and is published, and the snapshot's own loader then refuses the file: every later boot takes the forced full-pull path, and once the non-rotatable key ends (Risks row on key termination) the unloadable snapshot converts to a startup refusal. The same class reaches minted issued records: a name the `026-C3` loader would refuse contaminates the issued document at mint time.",
    "fix": "Extend the pull's single drop-and-bump-`incomplete` clause from \"breaks the `013-B4` bound\" to \"breaks the loader's name rule\" (length, control bytes, empty, leading/trailing space — the `check_native_name` family), keep it the only dropped case, and pin one fixture per clause in Test strategy #6's drop case."
  },
  {
    "severity": "minor",
    "location": "§The store pull (merge sentence) vs §The steam snapshot resolver (definition) — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound; `0` marks a worldlist\" and \"a delta merges by `appid` … and re-persists; a drop for the `013-B4` bound bumps `incomplete` rather than failing the page\"",
    "problem": "The definition is worded per-pull while the merge rule only says drops \"bump\" the field, leaving delta carry-forward unpinned. Under the per-pull reading, a delta with no fresh drops republishes `incomplete: 0` over a merged `apps` list that still lacks the record an earlier pull dropped — a silently incomplete worldlist regains the right to certify `200 []` for exactly the appid it is missing, reintroducing the dishonest miss certification the r1 posture exists to prevent; an attacker needs only one planted over-long name plus the passage of one delta.",
    "fix": "One sentence in §The store pull: a delta's published `incomplete` is the loaded snapshot's value plus its own drops (dropped records never enter the merged `apps` list, so nothing decrements it); only a full pull can publish `0` — and pin it in Test strategy #6's delta case."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/security-r1c5.md. That file is the only path this session
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
{"verdict":"approve","findings":[]}
```
