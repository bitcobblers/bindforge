# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-13T14:14:41.292Z
- ended: 2026-11-13T14:16:29.876Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/014-profiles-as-chains/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff ac9cb8acf87eeb9634dab42010276b843f39e219 ec0ea4774b9da2f62f24e139b0b9617a34e4c145` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/014-profiles-as-chains.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chain operations: `fork`, `diff`, `merge` — fork bullet: \"empty delta lists for the caller to fill\"; contradicts The contract per-link rule \"A `patch` has non-empty `derives` and at least one element across `set`/`clear`/`swap`\" and invalid fixture `patch-empty-deltas.toml`; also Test strategy #5 \"`resolve(fork(a) + diff, manifest)` equals `resolve(b, manifest)`\"",
    "problem": "fork's output is always a patch with zero delta elements, which the v2 grammar refuses at parse, so the one chain operation ADR-014 Decision Three names never returns a document that parses, re-parses, or loads; the round-trip property test composes that illegal document.",
    "fix": "Either state that fork returns a not-yet-loadable shell and pin it (Test #5: fork's output refuses to load until the caller adds a delta; the round-trip property appends `diff` to `a` alone, dropping `fork(a) +`), or give fork the caller's first delta elements so its output is a valid patch."
  },
  {
    "severity": "major",
    "location": "Chain operations: `merge` — \"When no such patch exists — one side clears an action the other sets, an order the fixed list order cannot express — `merge` refuses the whole result\"; Test strategy #5 \"conflicting patches (clear vs set on one action) refuse the whole merge\"",
    "problem": "Clear-vs-set on one action is expressible as one patch under the design's own determinism rules (fixed order set → clear → swap, last write wins): upper patch clears → emit `clear X`, upper patch sets → emit `set X i`. The stated predicate therefore refuses merges the same paragraph promises are fine (\"Compatible overlaps are not conflicts\") and contradicts the Options criterion \"a conflict is representable iff the composed effect cannot be written as one patch\"; Test #5 pins the wrong case.",
    "fix": "State the predicate as per-action non-representability under the fixed order and use an example that has no one-patch form — lower patch `clear X`, upper patch `swap X Y` with Y bound: the net \"X takes Y's binding, Y unbound\" requires a swap before a clear, which the order forbids — and re-word Test #5 to that pair."
  },
  {
    "severity": "minor",
    "location": "Chain operations: `diff` bullet \"Its output is a v2 patch document (the diff *is* the grammar) that `resolve`s to exactly what it must\"; same shape in the `merge` bullet",
    "problem": "When the two chains resolve to the same layout (b is a, or a plus a self-cancelling patch) diff emits zero elements, which the `patch-empty-deltas.toml` rule refuses; merge of two mutually inverse swap patches composes to the identity, same illegal shape. Two of the three operations are undefined on legal input.",
    "fix": "One clause each reusing the existing refusal: diff refuses when the two resolved layouts are equal, naming both heads; merge refuses an identity composition."
  },
  {
    "severity": "minor",
    "location": "Resolution — \"Mount at any rung\" paragraph: \"a consumer takes a prefix, a suffix, or the documents singly. The chain is only meaningful whole, so a suffix without its base is refused by `load_chain` at load\"",
    "problem": "The sentence grants a suffix and the very next sentence refuses it, so the paragraph both allows and forbids the same read, leaving the ADR Decision Three mount-at-any-rung requirement ambiguous.",
    "fix": "Delete \"a suffix\" (a prefix and single documents are what \"take its base and patch independently\" needs) and let the refusal sentence stand as the rule."
  },
  {
    "severity": "minor",
    "location": "Cross-file loading step 3 — \"more than `MAX_CHAIN_DEPTH` links is `ChainTooDeep` — both written against the constant\"; Goals \"the same guard rules the shipped single-document guard states\"; Test strategy #3 \"a chain of exactly `MAX_CHAIN_DEPTH` links resolves and one link more refuses\"",
    "problem": "The shipped predicate is `derives.len() > MAX_CHAIN_DEPTH` (chain.rs:27), so today a head with 32 ancestors (33 resolved links) passes, while the new cross-file boundary test refuses 33. The two layers disagree at exactly one link, so the re-issued `chain-past-max-depth.toml` and the `chain_ops` boundary test can pin different documents.",
    "fix": "Name the counted unit once: either \"more than `MAX_CHAIN_DEPTH` ancestor links (the shipped `derives.len() > MAX_CHAIN_DEPTH` predicate)\", or keep the resolved-links unit and state that the cross-file guard refuses one link earlier than the single-document guard, then write the fixture and the boundary test to that unit."
  },
  {
    "severity": "minor",
    "location": "Chain operations signatures — \"pub fn merge(p1: &Profile, p2: &Profile) -> Result<Profile, CoreError>\" and the diff bullet",
    "problem": "The documents diff and merge emit have unspecified `id`, `derives`, and `for`, although those entries are the chain lookup key and file name and must be slug-grammar and collision-free; only fork states the collision rule, so merge/diff output may be unloadable by construction.",
    "fix": "One clause each: diff emits `derives = [<a's head id>]` and the joined manifest's pin; merge inherits `id`, `derives`, `for` from p1 and takes a fresh caller-supplied id with fork's collision rule (or refuses when p1 and p2 name the same id)."
  },
  {
    "severity": "minor",
    "location": "The contract: `schemas/profile/v2/` invalid fixture list — \"chain-past-max-depth.toml  self-deriving-chain.toml  table-header.toml  dotted-key.toml …\" (no `forged-log-line.toml`); Test strategy #7 \"every invalid fixture is refused with a single-line, control-free, truncating message\"",
    "problem": "The v2 corpus re-issues every v1 grammar fixture except the log-injection one, while v2 refusals echo far more untrusted strings than v1's did (id, derives elements, both pin halves, delta elements, seeded-from, generated-by, ChainLinkMissing's id); `generated-by-control-char.toml` pins the entry's own grammar, not the escape-and-truncate path #7 asserts, so no v2 fixture exercises it.",
    "fix": "Re-issue `forged-log-line.toml` at v2 with a `\\n`-escaped `generated-by` (or `id`) — the existing fixture mechanism, no new rule."
  },
  {
    "severity": "minor",
    "location": "Resolution — \"pub struct Chain { pub links: Vec<Profile> }   // deepest base first … head last\" and `pub fn resolve(chain: &Chain, manifest: &Manifest)`",
    "problem": "A public field makes Chain consumer-buildable: an empty chain (no rule defines its output), a suffix, and a chain whose `kind` positions were never checked all reach resolve/fork/diff, since the completeness and position rules live only inside load_chain.",
    "fix": "State that Chain is only ever produced by load_chain (keep `links` private or give a non-public constructor), and state that resolve refuses an empty chain."
  },
  {
    "severity": "minor",
    "location": "The model: `document.rs` — \"`serialize` writes the ten entries in the fixed order\"; Test strategy #6 \"a hand-built document carrying all ten entries\"",
    "problem": "The entry table and the Goals define eleven entries (v1's four plus for, kind, set, clear, swap, seeded-from, generated-by), and the order the sentence itself enumerates names eleven; a wrong count in the section that fixes the closed set propagates to PROFILE_V2_ENTRIES and the grammar_closure.rs expectations.",
    "fix": "Change both occurrences to \"eleven entries\"."
  },
  {
    "severity": "nit",
    "location": "Cross-file loading / Resolution — \"pub fn resolve(chain: &Chain, manifest: &Manifest) -> Result<Vec<ResolvedBinding>, CoreError>\" alongside the untouched public `resolve_chain`",
    "problem": "The crate will export `resolve_chain` (id-level validation) and `resolve` (delta application), two public names one token apart doing different jobs, and no sentence maps ADR-014 Decision Two's \"Resolve the chain with `resolve_chain`\" onto the split a reader will notice.",
    "fix": "Add one sentence: resolve_chain keeps the id-level validation the ADR names, load_chain walks the files, resolve applies the deltas; consider naming the new one `resolve_layout`."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/014-profiles-as-chains.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/014-profiles-as-chains/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Chain operations — merge bullet: \"When some action's composed effect has no one-patch form under the fixed list order — lower `clear X`, upper `swap X Y` with `Y` bound: the net needs the `swap` before the `clear`\"; Test strategy #5 \"a non-representable pair (a `clear X` under a `swap X Y`) refuses the whole merge\"","problem":"The pinned example is a representable pair under the design's own rules. The fixed list order is set → clear → swap, so the clear precedes the swap — exactly the composition order of lower-then-upper. If swap exchanges bindings (entry-table reading), the single patch clear=[\"X\"] + swap=[\"X Y\"] reproduces the composed effect on every base layout; if instead \"an unbound side stays unbound\" makes a half-bound swap a no-op, the composition collapses to clearing X, whose one-patch form is clear=[\"X\"]. Under either reading merge must emit a patch, so the text and Test #5 refuse something the stated criterion (Options: \"a conflict is representable iff the composed effect cannot be written as one patch\") composes. The predicate and the set-under-clear clause are now correct; only the exemplar (which my own r1 fix supplied, backwards) and the test case remain wrong.","fix":"Swap the pair: lower `swap X Y`, upper `clear X` (both actions bound in the base) — the net is X unbound and Y bound to X's former, base-dependent input, which needs the clear after the swap (forbidden by the fixed order) and cannot be named by `set`; mirror the same flipped pair in Test #5."},{"severity":"minor","location":"Chain operations — merge bullet: \"emits one patch producing that composition on the base\" (no identity case)","problem":"The identity half of the r1 empty-output edge is still open: two identical swap patches (`swap X Y` under `swap X Y`) compose to the identity, whose only one-patch form is the empty patch that `patch-empty-deltas` refuses. diff gained \"chains resolving to equal layouts refuse too\" but merge states no refusal and Test #5 has no case, so merge can return a document the grammar will not accept.","fix":"One clause: a composition that cancels to the identity refuses as the empty-patch case; add the matching clause to Test #5."},{"severity":"minor","location":"Cross-file loading — \"pub struct Chain { links: Vec<Profile> }  // only load_chain builds one\" vs Resolution \"a consumer takes a prefix or the documents singly\" and Test strategy #5 \"appending its output to `a` resolves to `resolve(b, manifest)`\"","problem":"The revised private `links` with no public constructor closes the buildability hole but strands the promised consumer access: with no accessor named anywhere, neither a consumer taking a prefix/singles from a returned Chain nor the integration test building the chain `a + diff` is expressible through the public surface.","fix":"Either add a read-only accessor (`pub fn links(&self) -> &[Profile]`, construction still only via `load_chain`, and name it in the `zz_api_probe` list), or restate the mount route and Test #5 as file-level moves (prefix = `load_chain` of the lower link as head, singly = `parse_file`, round-trip = write diff's output into the directory and `load_chain` it as the new head)."},{"severity":"nit","location":"The contract — `generated-by` row: \"no ASCII control, default-ignorable, or bidi-control characters\"","problem":"The rule widened to two new refused classes while the per-link promise \"each with a named fixture above\" is backed only by `generated-by-control-char.toml` for the ASCII-control branch.","fix":"Extend that fixture or name one more (e.g. a bidi-bearing label) so each refused class has a fixture."},{"severity":"nit","location":"Chain operations — diff and merge bullets: \"(`fork`'s collision rule)\"","problem":"fork's collision rule is \"refused if it collides with any link id in the chain\", but merge takes no chain and diff takes two; the set of ids the caller's `id` must not hit is unspecified.","fix":"One parenthetical each naming the set: merge — `p1` and `p2` ids; diff — every link id of `a` (and optionally of `b`)."}]}
```
