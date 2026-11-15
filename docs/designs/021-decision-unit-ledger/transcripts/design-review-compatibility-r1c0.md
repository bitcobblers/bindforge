# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-nvfp4-nvfp4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-14T20:50:09.670Z
- ended: 2026-11-14T21:01:35.916Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":44453,"output":39721,"reasoning":0,"cache.read":782475,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/021-decision-unit-ledger/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
verify what the design claims about the code, and the report at docs/designs/021-decision-unit-ledger/reviews/design/compatibility-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/021-decision-unit-ledger.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/021-decision-unit-ledger/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Compatibility matrix, row \"ADRs without `ledger:` (001–012, 014–016, 020)\" — \"Yes — unchecked | Absent key means no ledger checks (¶6); their prose and handles behave as today\"","problem":"The backward-compatibility statement for the unconverted corpus is contradicted by ¶5 and ¶7. ¶5 binds every corpus file regardless of the key (\"A document that carries a handle must state the convention in a `**Unit handles.**` note\"), and ¶4 makes any trailing id span in any docs/adrs or schemas file a handle — so a unit-shaped trailing span added to ADR-005 or schemas/profile/v2/spec.md becomes handle-dangling / note-missing though that file never carries ledger:. ¶7 adds a code that needs no key: \"depends-order … checked on every ADR that carries the key, not only ledger-bearing ones\", which Test #7 restates as \"a ledger-bearing ADR with no `ledger:` key of its own still gets the check\" — self-contradictory, so the test that pins the exception cannot be written as stated. The Risks table already concedes the wider reach (\"A red corpus blocks every new run at entry, including runs unrelated to the ledger\"). Nothing is red today (grep confirms A17/A8: unit-shaped text only in 013, 021 and manifests/v1/spec.md), so this is a wrong compatibility contract rather than a broken tree — a maintainer would believe a 011/012/014/015 backfill run (Rollout 6) enters a tree that does not judge that file.","fix":"Split that row's Notes into what the absent key exempts (unit, pointer, owner, amendment checks) and what it does not (the ¶4–¶5 handle and note rule on every corpus file; depends-order on every ADR carrying depends-on), and re-word Test #7 to \"an ADR that carries `depends-on` but no `ledger:` key still gets the check\", replacing the self-contradictory phrase."},{"severity":"major","location":"Test strategy #10 and Rollout 1 — \"the corpus fixture set (temp dirs under `tests/pipeline/fixtures/ledgers/`)\" / \"fixtures under `tests/pipeline/fixtures/ledgers/`\"","problem":"Committed markdown under tools/orchestration/tests/ lands inside the standing format leg: .prettierignore lists only dist, node_modules and package-lock.json, format:check is `prettier --check .`, and `npm run check` chains lint, format:check and test (tools/orchestration/package.json:23) — the same command the harness runs as checkGate after every implement/fix/test-update stage (runner.ts:3065–3088; states.ts:11 \"executing `npm run check` after each mutating stage\") and CI runs on every PR (.github/workflows/check.yml:54–56). The fixtures must be malformed by design (a unit bullet outside a kind section, a C unit under ### Laws, an id with a trailing word, a preamble-inheriting pointer), which is exactly what prettier's markdown formatter canonicalizes, so a non-canonical fixture turns format:check red on Rollout 1 — the commit the design calls its first landing — and `npm run format` then rewrites the pinned bytes; ¶9 states the opposite rule for the corpus (\"nothing rewrites a ledger or a spec, so no check can be satisfied by a formatter\") without applying it to its own fixtures. Separately, Test #3's CRLF corpus cannot be a committed file at all (.prettierrc sets no endOfLine, so prettier's lf default fails a CRLF markdown file), and #10 (\"temp dirs under\") and Rollout 1 (\"fixtures under\") disagree about whether the tree is committed.","fix":"State the suite's existing posture: build fixture text in the test module and write it into the scratch repo the vitest.config.ts header describes (\"each file builds a scratch repo under /tmp\"), synthesizing the CRLF variant in memory — which replaces the committed tests/pipeline/fixtures/ledgers/ tree, so Rollout 1 lands only .ts files (already leg-clean). If committed .md fixtures are kept instead, name the tree in .prettierignore, which replaces format:check's coverage of it, and record there that #3's CRLF case is generated, not stored."},{"severity":"minor","location":"Assumption register A24 — \"the standing CI leg is what keeps the entry hook from blocking unrelated work | unverified | … no CI workflow run was observed\"","problem":"The leg is verifiable in this repository and true: .github/workflows/check.yml triggers on pull_request, runs `npm run check` with working-directory tools/orchestration on runs-on: ubuntu-latest, with node-version pinned \"22.23.5\" to the engines floor. The design leans on it twice as load-bearing — Risks row 1's mitigation (\"Test strategy #11 makes the join an `npm run check` leg, so the tree is green before a run starts (A24)\") and Rollout 4's acceptance (\"#11 turns the join into a standing CI leg\") — while the register marks it unverified, and the matrix's \"Linux (dev and CI host)\" row states the CI fact with no citation.","fix":"Set A24's status to verified with .github/workflows/check.yml (on: pull_request; `npm run check` in tools/orchestration; ubuntu-latest; node 22.23.5) as the basis, replacing \"no CI workflow run was observed\", and cite the same file in the matrix's Linux row."},{"severity":"minor","location":"Chosen design ¶1 — \"`ledger.ts` reads files with `node:fs`, never writes, and parses no path it did not list itself\"","problem":"No path library is named, yet the corpus is written as forward-slash patterns (\"docs/adrs/*.md + schemas/**/*.md\"), ¶2 tests an owner by whether \"docs/adrs/NNN-*.md is present\", and ¶6 needs \"template.md is corpus but never a ledger: it matches no `NNN-`\" — all deriving names, and name prefixes, from listed paths. The repo's \"no path parsing\" precedent does not cover this: stagePathAllowed's comment (gates.ts:589–592) rests on paths arriving \"only as git plumbing output — repo-root-relative, POSIX separators\" (moves.ts imports posix for it), whereas a gate that builds paths from fs listings gets platform separators, which are backslashes on the Windows checkout the matrix expects. The macOS/Windows row lists only case and CRLF as the portability axes considered, leaving checklist item 6's separator half open.","fix":"Add one clause in ¶1 naming node:path for the two corpus roots (join) and for the NNN-prefix and template.md tests (basename), replacing the bare \"reads files with `node:fs`\", and add \"separators via `node:path`\" to the macOS/Windows row's axis list."},{"severity":"nit","location":"Compatibility matrix, row \"macOS / Windows checkout\" — \"Expected — assumption\"","problem":"The cell is a hedge rather than a status; the matrix vocabulary is Yes / No / unknown, and the row's own note (\"not executed there\") is what unknown means. Not scored higher because the row states its mitigations and Test #14 names the fallback, so nothing is hidden from a reader.","fix":"One word: \"Not tested — expectation only\", or `unknown` with #14 named as the sole evidence."}]}
```
