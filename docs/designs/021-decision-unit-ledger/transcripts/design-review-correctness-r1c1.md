# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-14T21:38:18.476Z
- ended: 2026-11-14T21:51:23.306Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":50569,"output":50275,"reasoning":0,"cache.read":1021026,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/021-decision-unit-ledger/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 67e7b5740b46b427129312d4aa93e403e2ef5d2e 94fdca92af8afeb042234edb2d53f4b6edd00143` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/021-decision-unit-ledger.md,
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
verify what the design claims about the code, and the report at docs/designs/021-decision-unit-ledger/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design ¶2 (Unit grammar, 021-C1) — \"anchored with a look-behind and a trailing guard so it cannot match inside a longer token\"",
    "problem": "[new] The guard as written, `(?![\\d])`, blocks digits only, so the canonical id grammar matches inside `013-C1x`: `\\d{1,2}` takes `1`, the next character `x` is not a digit, so the assertion passes and `013-C1` is returned. Test strategy #1 pins the opposite (\"refuses `1234-C1`, `013-C1x`, `013-C123`\") and ¶2's own stated purpose. `013-C123` and `v013-C1` are genuinely refused; `013-C1x` is not. Impact: a typo id like `**013-C1x**` reads as a valid unit id, so the uniqueness and strictly-increasing checks consume a phantom unit instead of reporting `unit-malformed`, and pin #1 fails as written.",
    "fix": "Widen the guard in ¶2 to `(?![\\dA-Za-z])` and keep #1's two refusal cases as the pin. Nothing in the corpus depends on the narrower guard: cell-final ids are followed by a backtick, multi-id spans by a space, and `013-B1–B4` by an en dash."
  },
  {
    "severity": "major",
    "location": "Test strategy #5 (reservations) — \"`owner-missing` for `021-R1`'s pre-run `unassigned` text with no marker\"",
    "problem": "[new] The shipped text is an owner marker: docs/adrs/021-decision-unit-ledger.md:101 reads \"Owner: `unassigned` — a rollout task, not an ADR slot.\", which assumption A2 itself records (\"`021-R1`'s owner is `unassigned`\"). Under ¶2's own taxonomy `owner-missing` is the no-owner-at-all code (013-R5's shape) and an unparseable owner string is `owner-unknown`, so the pin is wrong either way the parser resolves the form. The deeper gap: ¶2 never states what may follow the owner marker, and the shipped ledgers use three different suffixes — a backticked value plus trailing prose, a bare period (\"Owner: this ADR's run.\"), and a `(reserved)` suffix — none of which #5 pins.",
    "fix": "Change #5 to pin `owner-unknown` for the shipped 021-R1 form and reserve `owner-missing` for a reservation bullet carrying no `Owner:` (013-R5), and state the deciding rule in ¶2: the owner value runs to the end of the line with backticks stripped, so `ADR-017 (reserved)`, `` `unassigned` `` and `this ADR's run.` all parse."
  },
  {
    "severity": "minor",
    "location": "Assumption register A2 with ¶2 — \"021 declares ten units (C1–C3, L1–L5, R1–R2) and none carries a `→` pointer\"",
    "problem": "[new] The cited basis contains a `→`: 021-C1's bullet, line 56, \"plus a `→` pointer to its target\". ¶2's rule — \"The pointer is the last `→` in the bullet and must be followed by non-space text\" — counts that code-span arrow (it is followed by a backtick), and \"or another prose target\" accepts the residue, so the gate sees 021-C1 as pointer-bearing today and `pointer-missing` can never fire for a unit that merely quotes the arrow convention. The conformance pass survives only because ¶8's appended `→` happens to be the last one.",
    "fix": "¶2: \"the last `→` outside a code span\"; A2: \"no unit carries a pointer; `021-C1` quotes the `→` glyph in a code span\"."
  },
  {
    "severity": "minor",
    "location": "Assumption register A5 vs A6 — \"Unit-id text in the corpus lives in exactly three kinds of place: a trailing code span at end of line, a trailing code span before a cell's ` |`, and two mid-sentence examples (`spec.md:17`, `021-…:59`)\"",
    "problem": "[new] A5's third kind is two lines, but A6 records three more handle lines that are neither line-final nor cell-final (\"Three spec lines carry handles mid-paragraph (`spec.md:27,168,183`)\"; verified, spec.md:27 reads \"and a geometry. `013-C2 013-L5` The id grammar is lowercase-only\"). The register therefore states an exhaustive taxonomy that omits three shipped lines, and ¶4's claim that the rule \"excludes the mid-sentence examples (A5) without a special case\" rests on that count.",
    "fix": "Make A5 four kinds — add \"a handle span in the middle of a wrapped paragraph line (`spec.md:27,168,183`)\" — and cite A6 from it."
  },
  {
    "severity": "minor",
    "location": "Test strategy #7 against ¶6 — \"a ledger-bearing ADR with no `ledger:` key of its own still gets the check\"",
    "problem": "[new] ¶6 defines the opposite: \"A ledger is `docs/adrs/NNN-*.md` whose frontmatter carries `ledger: yes`\", so \"a ledger-bearing ADR with no `ledger:` key\" is an empty class and the fixture cannot be built as written, even though ¶7's intent is clear (\"checked on every ADR that carries the key, not only ledger-bearing ones\").",
    "fix": "Rewrite the clause: \"a non-ledger ADR that carries `depends-on` (013's shape without the key) still gets `depends-order`\"."
  },
  {
    "severity": "minor",
    "location": "¶2 against ¶8 — \"every top-level bullet under one must be a unit\" / \"the Rollout paragraph re-set as a numbered list whose items 1–3 end with `021-C3`, `021-R1`, `021-R2`\"",
    "problem": "[new] In the shipped file the Rollout paragraph sits inside `### Reservations` (docs/adrs/021-decision-unit-ledger.md:110–112, before `## Alternatives considered`). An implementation that reads \"top-level bullet\" as \"top-level list item\" reports `unit-malformed` on this run's own conformance text and Test #11 cannot turn green. The numbered list is probably deliberate, but the document never says ordered items are prose.",
    "fix": "One clause in ¶2: only `- ` bullets are units; ordered items and paragraphs under a kind section are prose."
  },
  {
    "severity": "nit",
    "location": "Assumption register A3 — basis \"docs/adrs/013-game-manifests.md:128–158\" for \"013's ledger carries 24 units in four kind sections\"",
    "problem": "[new] The cited range is Budgets and Reservations only and cannot support the row's first claim (7 C + 8 L + 4 B + 5 R, Contracts beginning at line 50). The claim checks out; the basis does not reach it.",
    "fix": "Cite `:50–158`."
  },
  {
    "severity": "nit",
    "location": "¶7 (the checks, the taxonomy) — \"denies with the first twelve findings as `file:line code detail` lines plus `+N more`\"",
    "problem": "[new] Immediately after the sixteen-code list, \"the first twelve\" reads as twelve of the sixteen codes, which would drop every `amends-*` finding from the denial reason; Test #10 (\"lists at most twelve findings\") shows a count cap is meant.",
    "fix": "\"denies with at most twelve findings, `file:line code detail` each, plus `+N more`\"."
  },
  {
    "severity": "nit",
    "location": "¶8 (corpus conformance, spec.md) — \"line 14's \\\"ADR-016, ADR-017\\\" is corrected to \\\"ADR-017, ADR-019\\\" to match `013-L2`/`013-R4`\"",
    "problem": "[new] It does match 013-L2, but both texts then read \"the seed set's and the codec's (ADR-017, ADR-019)\", whose positional pairing is the reverse of 013-R1/R4 and spec.md:125 (codec → ADR-017, seed set → ADR-019), so the shipped pair stays ambiguous in both files.",
    "fix": "Write \"the seed set's (ADR-019) and the codec's (ADR-017)\" in the same edit."
  },
  {
    "severity": "nit",
    "location": "¶1 (Components) — \"`parseSource`'s `/^---\\r?\\n([\\s\\S]*?)\\r?\\n---/` is extracted verbatim into an exported `frontmatterOf(content)` that `parseSource` itself now calls\"",
    "problem": "[new] `parseSource` uses the whole match, not only the capture: `body = content.slice(front[0].length)` (gates.ts:55–57). A `frontmatterOf` returning the frontmatter slice cannot serve that line, so \"extracted verbatim\" and \"parseSource itself now calls\" cannot both hold with the shape implied.",
    "fix": "State the shape: `frontmatterOf(content): { front: string; body: string } | undefined` (or return the match), so the shared-regex claim is implementable as written."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/021-decision-unit-ledger.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/021-decision-unit-ledger/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"nit","location":"Chosen design ¶2 (Unit grammar) — \"an owner matching neither `ADR-\\d{3}`, `unassigned` nor `this ADR's run` is `owner-unknown`\"","problem":"[new] Every owner the design itself accepts carries a suffix beyond the shape name: the shipped forms are `ADR-017 (reserved).` (docs/adrs/013-game-manifests.md:147,150,153,155), `this ADR's run.` (docs/adrs/021-decision-unit-ledger.md:108) and, for 021-R1, `unassigned — a rollout task, not an ADR slot.` (:101). Exact equality can accept none of them, yet ¶2 says they \"all parse\" and Test #5 pins them clean, so the rule must mean *begins with*; \"matching\" also admits a search reading, under which `Owner: see ADR-017` would parse clean. Test #5 pins the intent but the rule's one word is ambiguous.","fix":"One word: \"an owner not beginning with `ADR-\\d{3}`, `unassigned` nor `this ADR's run` is `owner-unknown`\" — replaces \"matching\", keeps the three shapes."},{"severity":"nit","location":"Chosen design ¶8 (spec.md bullet) — \"line 14's \\\"ADR-016, ADR-017\\\" is corrected to the pairing `013-L2`/`013-R4` state gives\"","problem":"[re-raised] The spec.md half of my r1 nit is closed (the corrected pairing matches 013-R1 codec→ADR-017 and 013-R4 seed set→ADR-019); the other half is not: 013-L2 does not state that pairing positionally — docs/adrs/013-game-manifests.md:96–97 reads \"the seed set's and the codec's — ADR-017, ADR-019\", positionally seed set→ADR-017, codec→ADR-019, the reverse of both the corrected spec.md:14 and its own R1/R4. After ¶8 lands, spec.md:14 is the only half of the shipped pair that reads right, and ¶8 cites L2 as a source of the very pairing L2 positionally contradicts.","fix":"Extend the same editorial edit to group L2's parenthetical (\"the seed set's (ADR-019) and the codec's (ADR-017)\"), or attribute the pairing to `013-R1`/`013-R4` alone and note L2's flat order inside the sixth `Backfill findings` entry."},{"severity":"nit","location":"Goals / Non-goals — \"no Rust-side re-implementation, no CommonMark dependency (Deferred)\"","problem":"[new] Deferred / out of scope lists neither item; they are rejections recorded in Options (\"A CommonMark parse … Rejected\") and the Compatibility matrix (\"Rust product … out of scope\"). With ¶9 — the old second pointer — deleted, \"(Deferred)\" is the only citation and it under-cites two of the items it covers.","fix":"Point the parenthetical at \"(Deferred; Options and matrix)\", or add the two one-line entries the heading (\"Deferred / out of scope\") already admits."}]}
```
