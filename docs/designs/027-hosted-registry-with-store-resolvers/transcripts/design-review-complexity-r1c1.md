# Transcript: design-review:complexity r1c1

## Provenance
- stage: design-review:complexity (round r1c1)
- agent: review/design-complexity
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:31:30.885Z
- ended: 2026-11-22T03:38:27.920Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":53501,"output":15191,"reasoning":0,"cache.read":1038104,"cache.write":0}
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (complexity seat):
[
  {
    "severity": "major",
    "location": "Chosen design §Issued entries — \"The mint rule\", step 3: \"Truncation is **not** part of the rule: … a derived slug is grammar-valid and within `MAX_SLUG_OCTETS` by construction\"",
    "problem": "The by-construction bound omits the rule's own octet expansions: step 2's `g-` prefix adds 2 octets and step 5's `-steam-<appid>` suffix adds 7+. The snapshot loader admits 128-octet names (MAX_NATIVE_NAME_OCTETS = 128, crates/core/src/placement.rs), so an ASCII digit-leading name of 127–128 octets mints a 129–130-octet id, and a collision suffix onto any slug over ~121 octets breaches MAX_SLUG_OCTETS = 128 — which the reused 026-C3 loader enforces (corpus.rs::game_id_valid). The mechanism removal (\"no new rule\") rests on unsound arithmetic, and Test strategy #1's \"every output satisfies is_slug_name ∧ ≤ MAX_SLUG_OCTETS\" cannot hold as pinned; a record the loader refuses turns the issued document into a load failure or a per-mint 503.",
    "fix": "Add one final mint step: if the final id exceeds MAX_SLUG_OCTETS, substitute `steam-<appid>` (≤26 octets under the closed key grammar's appid bound), held-set-checked like any id, and delete the \"by construction\" claim — replacing text, adding no stage beyond one length compare."
  },
  {
    "severity": "major",
    "location": "Chosen design §Issued entries — \"The mint rule\", step 5: \"A suffix-vs-suffix collision is arithmetically impossible (the suffix is a function of the appid)\"",
    "problem": "The held set is games ∪ retired across both catalog documents, and corpus ids are operator-chosen grammar strings, not mint outputs: a curated or retired record whose free-chosen id already spells `<slug>-steam-<appid>` for the same slug/appid pair is reachable, and as written the mint emits a suffixed id already held — no refusal fires, and 027-L2 uniqueness breaks across documents. The impossibility argument only holds if every suffixed id came from a mint, which 026-C4's upsert freedom contradicts.",
    "fix": "Run the existing held ∪ retired check on the suffixed id before serving, routing to the same named 503 `mint identity held`; this replaces the impossibility sentence with one lookup the rule's machinery already performs."
  },
  {
    "severity": "minor",
    "location": "§The store pull / §Budgets: \"Per page, up to `STEAM_PAGE_ATTEMPTS` (3) attempts separated by `STEAM_RETRY_SECONDS`\"",
    "problem": "One 3,600 s constant governs two different loop tempos: the refresh thread's between-pull retry and the within-pull page-attempt gap. A single flaky page at startup defers the named startup refusal by up to 2+ hours inside the pre-bind full pull; no Risks row, Rollout item, or runbook mention acknowledges this coupling or the bind stall.",
    "fix": "Split the knob — a short seconds-scale backoff for page attempts, STEAM_RETRY_SECONDS reserved for the refresh thread — or pin the startup cadence explicitly in §The store pull and add a Risks row; name what the single number replaces."
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
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/complexity-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§Issued entries — \"The mint rule (027-C5, restated in full)\", step 5 — \"A mint whose chosen id exceeds `MAX_SLUG_OCTETS` mints `steam-<appid>`: appid-unique, ≤ 26 octets under the closed key grammar's appid bound\"","problem":"re-raised (narrowed from r0 #1). The bound this clause rests on is not stated anywhere in the design or the repo. The registry's whole key rule is apps/registry/src/keys.rs `key_is_admissible`: a 128-octet cap on the joined key plus `Source::spells_lookup_key` (crates/discovery/src/game.rs:84), documented as \"shape only\" requiring merely a non-empty external id — no decimal rule, no appid-length rule; a 122-octet external id is admissible (the shipped test pins `steam/` + 122 `a`s as admitted). The only appid-digit bound in the tree is `MAX_APPID_DIGITS = 20` (crates/discovery/src/steam_common.rs:26), a private const gating the client-side ACF emission seam, not the grammar and not visible to apps/registry. The snapshot's own rules refuse only \"an `appid` that is not a decimal integer\", so a decimal appid of any digit count fits a 32 MiB file and loads; `steam-<appid>` is 7 + digits octets, so the rule's terminal case is not provably ≤ MAX_SLUG_OCTETS = 128 (a 122-digit appid gives 129), and Test strategy #1's pinned invariant \"every output satisfies `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS`\" is still not guaranteed by the rule. r1 fixed the rule's shape — the bound step exists, the \"by construction\" claim is gone, the suffixed id is held ∪ retired-checked — but its one remaining length claim cites a nonexistent bound, and an over-bound minted id is refused by the reused 026-C3 loader (corpus.rs:390 `game_id_valid`), i.e. a load refusal of the issued document or a per-mint 503, the same failure as r0.","fix":"Make the bound the design's own named rule instead of a citation: add an appid digit cap (the 20-digit figure `steam_common.rs` already uses, restated as this design's budget) to §The store pull's record rules and to the snapshot loader's named refusals beside the existing \"not a decimal integer\" refusal it shares, then keep step 5's \"≤ 26 octets\" as arithmetic over that stated cap. This replaces the phrase \"under the closed key grammar's appid bound\" with one integer compare on a field the pull and the load already inspect; no new stage. Equal-size alternative: drop the numeric claim and make step 5's terminal case \"an id still over `MAX_SLUG_OCTETS` after substitution is the named `503`\", which is sound with no appid bound at all."},{"severity":"minor","location":"§Issued entries — \"Retired ids (`027-L2`)\" — \"**Every** delete — issued or corpus — is two persists under one mutation-lock hold, retired first\"","problem":"new (introduced by this revision; r0 said \"in one mutation-lock persist\"). This is the only place in the design where on-disk state can be half-applied, and the resulting state is unnamed: if the process dies between the two renames the id sits in `retired-ids-v1.json` while still standing in its catalog document, so `games ∩ retired ≠ ∅` — a condition no rule governs, since the mint's steps 4/6 test \"held **or** retired by a different record\" and the loader's named refusals list neither the overlap nor its absence. The recovery path is precisely the named-`503` retry: re-issuing the refused delete re-appends an id the list already holds, and the document never says whether that is a no-op or a second entry in a file it defines as strictly \"in byte order\" with binary-searched membership. Test strategy #5 pins the append ordering and the cap refusal but not the half-applied state.","fix":"One sentence plus one branch: the retired-list insert is a set insert (id already present ⇒ no-op, since a duplicate breaks the byte-order invariant the same paragraph fixes), and `games ∩ retired` is defined as the legal, recoverable state it is — the catalog record keeps answering, the id is held-and-retired, and the operator's retried delete completes the removal. Replaces the silence about the retry; adds no file and no mechanism."},{"severity":"nit","location":"§Budgets — \"`STEAM_PAGE_ATTEMPTS = 3` separated by `STEAM_PAGE_RETRY_SECONDS = 30`, inside `STEAM_PULL_DEADLINE_SECONDS = 900`\"","problem":"new. The worst-case attempt grid those three knobs describe — 12 pages × (3 × 30 s of retry gaps + 32 s of connect/read timeout) = 912 s — overshoots the deadline, so a fully degraded page sequence can never execute the retry plan the other two constants state. Nothing is broken (the outcome is the named refusal the section wants, and the r0 hour-long startup coupling is genuinely gone), but this family is the design's single readable statement of the pull's worst case and it does not close.","fix":"Either raise the deadline to cover the grid (960) or add a half clause to §The store pull: \"the deadline may cut a page's remaining attempts\"."}]}
```
