# Transcript: design-review:compatibility r1c5

## Provenance
- stage: design-review:compatibility (round r1c5)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T04:23:01.636Z
- ended: 2026-11-22T04:35:29.725Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":63500,"output":26956,"reasoning":0,"cache.read":995607,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/compatibility-r1c5.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "[new] §Issued entries, the mint rule, steps 4/6 × §Test strategy #1 — step 4: the suffixed id \"earns the same held ∪ retired check before it is served\"; step 6: \"Any spelling the rule lands on that is held or retired by a different record refuses the mint with the named `503` `mint identity held`\" vs #1: \"a suffixed id already held by a curated record takes `steam-<appid>`, and a held `steam-<appid>` refuses `mint identity held`\"",
    "problem": "The rule text refuses the mint when the suffixed id is held, but the test suite pins a fallback to `steam-<appid>` in that case. Implementations match suites, so the shipped rule silently becomes a three-spelling cascade the rule text does not state; Test strategy #5's \"an id retired under both spellings refuses `mint identity held`\" is stale under either reading (three spellings can all be held)."
  },
  {
    "severity": "minor",
    "location": "[new] §Test strategy #1 — \"including the over-bound cases the arithmetic allows: a 128-octet ASCII name, a 128-octet digit-leading name, and a 128-octet *colliding* name all mint `steam-<appid>`\"",
    "problem": "The first fixture's expectation contradicts mint step 5: the fold never grows (single-base-letter folds are one octet per code point; hyphen collapse and trim only shrink; `g-` applies only to a leading digit), so a 128-octet digit-leading-free, non-colliding ASCII name folds to at most exactly 128 octets — `MAX_SLUG_OCTETS` is endpoint-inclusive (crates/core/src/placement.rs:83) — is in budget, and mints its own slug, not `steam-<appid>`. Only the digit-leading (+2) and colliding (+7+digits) fixtures are over-bound. Implementing to this expectation would fallback a legal at-bound slug."
  },
  {
    "severity": "minor",
    "location": "[new] §The steam snapshot resolver / §The hosted lookup policy step 5 — \"`incomplete` is how many records the pull dropped … Non-zero, the file is a list that knows it is short, so no steam non-hit is certifiable against it\"",
    "problem": "One dropped record is a whole-instance certification cliff: a single attacker-chosen over-128-octet rename (A11 records renames to existing appids; A10 notes the bound has one octet of margin) permanently flips every steam miss — including non-decimal external ids — from `200 []` to `503 steam snapshot incomplete`, and by this document's own client row and key-revocation risk row that fails every bound CLI scan closed, immediately and until the store name shrinks (the mark re-publishes on every pull; re-sizing is Deferred). The Revision log's r0→r1 rebuttal rejects failing the whole pull but never weighs localizing the non-certification."
  },
  {
    "severity": "nit",
    "location": "[new] §Assumption register A16 — \"cross-`cargo check`s the *whole workspace* for `x86_64-pc-windows-msvc` — legitimate there only because `cargo check` generates no code\"",
    "problem": "The parenthetical contradicts the design's own mechanism: `cargo check` does run build scripts, and `ring`'s compiles C/asm for the target — exactly why §Options drops `apps/registry` from the leg. The cross-check is legitimate today because no dependency runs a target-native build script."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/compatibility-r1c5.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "[new] §The store pull — \"`discovery::game::friendly_name` through that existing `discovery` dependency (A7: the shipped rule is whitespace and clamping only, which is why the filter is ours)\"", "problem": "The revision makes a shipped display-convention function load-bearing for identity, against its own stated contract. `025-C2` says the friendly name \"is never end-user visible ..., participates in no join, and carries no derivation law\", and the function's doc-comment says the clamp is admissible \"here and only here\" because \"nothing parses the result, it is never joined to anything, and it is never user-visible\" (crates/discovery/src/game.rs:96-100). This design inverts all three: the output is persisted into a contract document, served to bound CLIs as the candidate `name`, and fed to mint steps 1-5, whose output is immortal under `027-L2`. Because `025-C2` deliberately gives `friendly_name` no derivation law, nothing pins it: a display-side change in `crates/discovery` (whitespace set, clamp rule) silently changes every identity minted afterwards, with no gate and no ledger annotation.", "fix": "Have `apps/registry` own the two-line rule (a `char::is_control()` strip, whitespace collapse, longest `char`-boundary prefix) as its own named function pinned by Test strategy #6, replacing the cross-crate call with the copy it pins. If the call is kept instead, the smallest fix is a registry suite asserting the two rules byte-exactly plus a Rollout-1 line making a `friendly_name` change a mint-stability event."}, {"severity": "minor", "location": "[re-raised] §Issued entries, mint steps 4/6 × §Test strategy #1 — step 6: \"Any spelling the rule lands on that is held or retired by a different record refuses the mint with the named `503` `mint identity held`\" vs #1: \"a suffixed id already held takes `steam-<appid>`, a held `steam-<appid>` refuses `mint identity held`\"", "problem": "The contradiction I filed in r1 is still open and the revision widened it: r1's fixture said \"already held by a curated record\", the new text says \"already held\", so the unstated plain → suffixed → `steam-<appid>` cascade now covers every holder. Step 5's fallback triggers on length only, never on held-ness, so the rule text refuses exactly what the suite pins as a fallback, and implementations match suites.", "fix": "State the cascade in step 4's own sentence (\"a suffixed id that is itself held or retired takes `steam-<appid>`; if that is held or retired, step 6 refuses\") and reduce step 6's \"any spelling\" breadth to the three named spellings."}, {"severity": "minor", "location": "[re-raised] §Test strategy #1 — \"including the over-bound cases the arithmetic allows: a 128-octet ASCII name, a 128-octet digit-leading name, and a 128-octet *colliding* name all mint `steam-<appid>`\"", "problem": "Still contradicts §Issued entries step 5, which this revision restated (\"The fold never exceeds the name's octets, but steps 2 and 4 add them\"): a 128-octet ASCII name with no leading digit and no collision folds to at most exactly 128 octets, and `MAX_SLUG_OCTETS` is endpoint-inclusive (crates/core/src/placement.rs:83), so it mints its own slug. The new sentence compounds it in the other direction: \"an id still over `MAX_SLUG_OCTETS` after substitution refuses `mint identity over bound`\" is unreachable through any admissible key, since keys.rs caps the whole joined key at 128 and `steam-` costs 6. The suite thus pins a legal at-bound slug as a fallback and pins a production-impossible refusal.", "fix": "Drop \"a 128-octet ASCII name\" from that fixture list (the digit-leading and colliding names are the over-bound cases) and mark the `mint identity over bound` case a unit-only defensive branch, or delete it."}, {"severity": "nit", "location": "[new] §Options — \"Nothing else a legal pull is handed makes the snapshot short of the games-only set (§The store pull normalizes, never drops)\" and §Risks — \"The pull seam normalizes the `name` to the `013-B4`-legal form and keeps the record (§The store pull owns the rule)\"", "problem": "Both revisions cross-reference section titles that do not exist; the section is \"The store pull (traces `027-C3`, `027-B4`, `027-L1`, `027-L4`)\". The document's tracing discipline reads those anchors as pointers.", "fix": "Point both at §The store pull, as §Deferred already does."}, {"severity": "nit", "location": "[new] §Budgets — \"`MAX_STEAM_PAGE_OCTETS = 12_582_912` — ... above the 50k-records-at-~240-octets estimate ... the pinned `max_results` can return\"", "problem": "The revision introduces a legitimate-input estimate of ~240 octets per record to size the page budget, while leaving the snapshot's `027-B1` ceiling (33,554,432) unstated against it: at 200k records that ceiling admits an average record of only ~167 octets, so a catalog at the per-record figure this section justifies would breach the publish-time check — which, after this revision, has no fallback posture at all since the `incomplete` machinery is deleted. Real catalogs sit far inside it; the finding is the missing arithmetic, not a predicted outage.", "fix": "Add the per-record average the 32 MiB ceiling admits at 200k records beside the page estimate, and name a publish-time breach's posture in one clause (the same `027-B1` refusal the loader applies), so both budgets come off one table."}, {"severity": "nit", "location": "[new] §Risks — \"the cost is a shorter label, never a different identity\"", "problem": "A clamp can change the identity, not only the label: a title whose first 128 octets are all non-`[a-z0-9]` (an emoji or CJK run) with its Latin title past the bound folds empty and takes step 3's `steam-<appid>` where the unclamped name would have minted the slug. Harmless (the id stays unique and stable), but this is the mitigation row's own claim.", "fix": "Qualify it: \"a shorter label, and a different identity only where the surviving prefix folds empty, which is step 3's fallback\"."}, {"severity": "nit", "location": "[re-raised] §Assumption register A16 — \"legitimate there only because `cargo check` generates no code\"", "problem": "Unchanged from r1 and still in tension with §Options' own correction (\"`ring` compiles C and assembly for the *target* through a build script\"), which is precisely why Rollout 3 drops `apps/registry` from that leg.", "fix": "Reword the parenthetical: the cross-check is legitimate today because no other dependency runs a target-native build script."}]}
```
