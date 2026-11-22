# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T02:44:51.072Z
- ended: 2026-11-22T02:59:02.009Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":61680,"output":29238,"reasoning":0,"cache.read":1240457,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/compatibility-r1c0.md
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
re-open one only if it contradicts docs/adrs/027-hosted-registry-with-store-resolvers.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"[new] §Issued entries, the mint, and identity bookkeeping → \"The mint rule (`027-C5`, restated in full)\", step 3: \"Truncation is **not** part of the rule: … a derived slug is grammar-valid and within `MAX_SLUG_OCTETS` by construction\"","problem":"The claim is false by arithmetic: MAX_NATIVE_NAME_OCTETS = 128 (crates/core/src/manifest.rs:58) and MAX_SLUG_OCTETS = 128 (crates/core/src/placement.rs:83) are the same number, and the rule adds octets — step 2's `g-` prefix (+2) and `027-C5`'s collision suffix `-steam-<appid>` (+7+digits). A legal 128-octet store name opening with a digit mints 130 octets; any name whose derived slug is ≥ ~115 octets mints over 128 once it collides. The rule it claims to re-pin ends \"prefix `g-` a leading digit; … cap at the shared bound\" (ADR-024 `024-C2`) and `027-C5` spells the mint \"bounded to the slug budget\", so the step was dropped rather than re-pinned. Concrete failure: an over-bound game_id fails the registry's own issued-document gate (apps/registry/src/corpus.rs, game_id_is_valid inside record_from_value, adopted verbatim via \"the same `corpus.rs::Catalog` type\"), so the mint writes an issued-v1.json its next load refuses by name; every bound client drops the candidate at the `025-L5` gate (apps/discovery-cli/src/catalog.rs:456-470) and the match store refuses to confirm it (apps/discovery-cli/src/matches.rs:284, :354), and `027-L2` makes the unusable id permanent. Test strategy #1's assertion that \"every output satisfies `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS`\" is a property this rule cannot deliver.","fix":"Replace the \"by construction\" sentence with a bound test routed to the existing step-4 fallback: \"empty result **or** a derived slug that does not fit `MAX_SLUG_OCTETS` with room for the `-steam-<appid>` suffix (and the `g-` prefix)\" mints `steam-<appid>`. No truncation rule and no new mechanism; the fallback stays a function of the appid, so it is record-stable per `027-L2`."},{"severity":"major","location":"[new] §Options considered: \"**In-process HTTPS pull over a pure-Rust TLS stack** (chosen)\", with §Test strategy #17 \"CI needs no edit\" and the matrix's Linux \"`rustls`/`ring` build in CI\" / Windows \"not run in CI\"","problem":"The chosen stack is not pure Rust: the `ring` provider compiles C and hand-written assembly for the target in a build script, and build scripts run even under `cargo check`. The shipped gate already cross-compiles this workspace for Windows from Linux — .github/workflows/check.yml:26-32 adds x86_64-pc-windows-msvc and runs `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` on ubuntu-latest, a runner with no MSVC C toolchain for a foreign-target build script. The design's only mitigations are \"versions pinned; cargo trio covers it\" and \"CI needs no edit\", and it never names the host-toolchain prerequisite the crate's std-only posture let the project disclaim (README.md:13-15 documents the cargo gate as needing none of the listed system dependencies).","fix":"Settle the packaging/toolchain question in the document before implementation: either record the evidence that the pinned rustls/ring versions keep the existing x86_64-pc-windows-msvc leg green, or name the workflow change (compile-check apps/registry on the host, keep the cross-target leg for the rest of the workspace) plus the README contributor-prerequisite line that now names a C toolchain — and stop calling a C/asm build-script dependency a \"pure-Rust TLS stack\"."},{"severity":"minor","location":"[new] Compatibility matrix: \"Windows … not run in CI\" and \"macOS … not a macOS CI runner\", both \"Supported? Yes\", with §Test strategy #16 \"proven through the `DataLayout`/env-injection seams and compile coverage, not a runner\"","problem":"Two rows claim support that no repository gate produces, and the Windows note is factually wrong: check.yml already runs a whole-workspace Windows compile leg that includes apps/registry — exactly where the native-dependency finding lands. macOS has no leg of any kind, so \"compile coverage\" names something nobody runs; checklist item 7 wants a platform test named for every platform the matrix claims to support.","fix":"Correct the Windows note to the existing x86_64-pc-windows-msvc compile leg and make it the place the dependency question is proven; either add the same-style compile-only macOS-target leg from the Linux runner (the workflow already justifies that technique) or change the macOS/Windows rows to the honest \"dev/one-shot posture, not a hosting target\" status."},{"severity":"minor","location":"[new] §Options considered (mint basis): \"the six curated ids are exactly what the rule would mint from those names — is what makes mint-then-promote cohere rather than collide\"","problem":"The shipped seed contradicts it: data/catalog/v1/catalog.json spells the record named \"Portal 2\" as `portal-two`, which this rule derives as `portal-2`; five of six converge, not six. Under `027-L2` the consequence is not cosmetic — if that curated record is ever deleted, appid 620 re-mints as `portal-2` and both ids live forever, the older one already held by any match store that confirmed it.","fix":"Restate the convergence evidence with the real count and name `portal-two` as the exception, and add the sentence on what re-minting a curated appid costs under `027-L2`, in the cap/promotion arithmetic Rollout 6 carries."},{"severity":"minor","location":"[new] Compatibility matrix, \"Bound CLI clients (any platform)\" row: \"the new `503`s render as the shipped status-line-only refusal\"","problem":"The rendering is right but the consequence is understated: the shipped client fails the whole scan on any non-200 (apps/discovery-cli/src/main.rs, step 4: \"A mid-scan failure fails closed: no partial table (`025-C4`)\"), so once a snapshot passes `027-B2`'s age bound the first novel steam key turns a user's `discovery list` into a refused run; the Risks table's \"the honest degraded posture while corpus+issued keep serving\" is a server-side claim no bound client can render. The `027-L3` posture itself is not re-opened.","fix":"Add one clause to that row, echoed in the key-revocation risk row, naming the client's fail-closed rendering, so the runbook treats snapshot age as a user-visible availability property rather than an operator-only detail."}]}
```
