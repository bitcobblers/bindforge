# Transcript: code-review:testing r1c0

## Provenance
- stage: code-review:testing (round r1c0)
- agent: codereview/code-testing
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T07:36:44.833Z
- ended: 2026-11-22T07:48:31.277Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":277411,"output":34079,"reasoning":0,"cache.read":3484400,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/027-hosted-registry-with-store-resolvers.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/testing-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-22T07:10:57.020Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (testing seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the testing checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/testing-r1c0.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only. The entire response must be a single valid JSON object with no prose, explanation, markdown, or additional text before or after it.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict": "request-changes", "findings": [{"severity": "major", "location": "apps/registry/tests/steam.rs:1053-1131 (with :1122-1129 and src/steam.rs:494-503)", "problem": "Test strategy #6's transport-assurance clause specifies a loopback TLS listener presenting a self-signed cert for the verified name and one for another name, refused on the chain rather than the hostname, precisely so that \"a verifier relaxed to accept the fake\" cannot pass the suite with the key on an unauthenticated link. What shipped is a plain-TCP TcpListener that answers nothing: no certificate is ever presented and no chain is ever built. The named mutation is therefore invisible — relaxing the verifier still fails the handshake for I/O reasons, still returns Err, still leaks no key bytes, and every assertion still passes, while production would ship BINDFORGE_STEAM_WEB_API_KEY to any terminable link. The chain-not-hostname case is asserted vacuously as `!text.is_empty()`. The legs do pin real properties (no GET/key before handshake; SNI is the pinned authority, not the connect address), but not the assurance property the design and its revision log r1->r2 name as the reason the clause exists.", "fix": "Add rustls (no default features) as a dev-dependency plus a checked-in self-signed cert/key fixture and drive the two cases the design names — cert for the verified name, cert for another name — asserting both handshakes are refused and that the refusal names a certificate/chain condition, both dialing 127.0.0.1 so #17's no-outbound rule holds. If a TLS fixture is deliberately out of budget, correct the claims at tests/steam.rs:3-4, :53-56, :1053-1058 to state only what is pinned (TLS client + name independence) and record #6's assurance clause as unmet rather than proven."}, {"severity": "major", "location": ".github/workflows/check.yml:29-32 (unchanged) against apps/registry/Cargo.toml:21-26 and Cargo.lock", "problem": "Test strategy #16 says the shipped whole-workspace cargo check --target x86_64-pc-windows-msvc leg's scope \"Rollout 3 narrows\" so ring's target build script is never asked of a runner without a target C toolchain, with apps/registry compile-checked on the host instead, and #17 says \"The cargo job does change, per Rollout 3: the Windows leg's scope, and the advisory screen\". Neither landed: the workflow still runs the leg over the whole workspace, which now contains the rustls/ring dependency (ring 0.17.14 with its C/asm build.rs), and README.md gained no C-toolchain prerequisite (Rollout 3, proposal lines 719-727). The runner's gate record contains only host-target legs, and I could not execute the cross leg myself (target x86_64-pc-windows-msvc not installed), so no evidence establishes that the promised platform posture holds or that the leg still passes after this dependency addition.", "fix": "Land the Rollout 3 gate half with the code: narrow the x86_64-pc-windows-msvc leg to the C-free members and add a host-target `cargo check -p bindforge-registry` step, plus the README.md prerequisite line. If the split is intentionally deferred, attach a green run of the unmodified whole-workspace cross leg to the gate record and mark #16/#17's cargo-job clauses unmet before approval."}, {"severity": "minor", "location": "apps/registry/src/state.rs:424-428 (untested; nearest tests apps/registry/tests/issued_entries.rs:335-396 and :495-550)", "problem": "Test strategy #4 promises \"the octet cap refuses the same way\" beside the record-count cap. The suite pins the count cap by planting the real 200,000 records and pins the load-side metadata gate, but nothing exercises the write-side MAX_ISSUED_DOCUMENT_OCTETS check on the mint path, which is reachable in production (~150k records with 128-octet ids and names sits at 64 MiB while still under 027-B3's count cap) and is what stops persist-before-serve renaming an over-budget document.", "fix": "Add one leg beside at_the_issued_cap_…: plant a document under the record cap whose serialization sits within one record of MAX_ISSUED_DOCUMENT_OCTETS (fat 128-octet ids/names, streamed as :516-521 already does), then assert the novel key gets the named 503 and the file, model, and revision are untouched."}, {"severity": "minor", "location": "apps/registry/tests/mint_rule.rs:15-23 and apps/registry/tests/hosted_lookup_policy.rs:16-20 (src/state.rs:349-351; src/lib.rs:19-21)", "problem": "Two named refusals required as Test-strategy pins are unreachable through every tested face and are pinned nowhere: `mint identity over bound` (keys.rs caps the joined key at 128, so steam-<external id> cannot reach the rung) and `503 no steam snapshot` (startup is load-or-pull, so a live instance either has a snapshot or never binds and state.rs:349-351 cannot fire). The gaps are recorded honestly, but only in test-file headers; under checklist item 7 they are untestable additions — a 503 and a mint rung with no reachable caller — and the mint rung needs a seam the crate published for the pull but declined for mint.", "fix": "Mirror the tests/steam.rs precedent: publish `pub mod mint;` on the library face and pin mint_id's over-bound rung directly (a 128-octet name with a 123-digit external id -> MintIdentityOverBound), and pin `no steam snapshot` as a unit over the posture decision with an empty snapshot slot. If the judgment is that both are unreachable by construction, record that reachability claim in §The hosted lookup policy and §Issued entries rather than only in headers."}, {"severity": "minor", "location": "apps/registry/tests/cache_document.rs:162, caching_policy.rs:74, upstream_client.rs:62, upstream_endpoint_gate.rs:52 (29 ignored legs total) with apps/registry/src/cache.rs, upstream.rs, endpoint.rs", "problem": "ADR-027's retiring half is implemented as mass #[ignore] rather than deletion: 29 legs across four test targets (cache_document 7, caching_policy 11, upstream_client 6, upstream_endpoint_gate 3, plus one each in admin_mutations.rs:598 and candidates_join.rs:331), which in my run reported `0 passed; N ignored` in 0.00s. The retired modules themselves — cache.rs, upstream.rs, endpoint.rs — are no longer declared in main.rs:19-30 yet remain in the tree, so a green `cargo test --workspace` says nothing about ~1,900 lines that are still compiled as test targets. §Hosted mode's \"Retired code … Retired suites per A15\" means gone, and the stated reason (\"pending the runner's deletion\") names an actor that must actually act.", "fix": "Have the runner delete the four source modules and four suites as the recorded completion of Rollout 2/3, keeping the two single-leg retires ignored-with-reason; if deletion cannot happen in this run, record it in 03-build-notes.md as an outstanding runner action so the ledger's \"retired\" does not quietly mean \"ignored\"."}, {"severity": "nit", "location": "apps/registry/tests/steam.rs:69-84, tests/issued_entries.rs:36-38, tests/retired_ids_and_curation.rs:32", "problem": "Twelve budget constants (MAX_STEAM_SNAPSHOT_OCTETS, MAX_STEAM_PAGE_OCTETS, STEAM_REFRESH_MAX_PAGES, STEAM_PAGE_ATTEMPTS, STEAM_PAGE_MAX_RESULTS, MAX_NATIVE_NAME_OCTETS, REFRESH/MAX_AGE/RETRY seconds, MAX_ISSUED_ENTRIES, MAX_ISSUED_DOCUMENT_OCTETS, MAX_RETIRED_IDS_OCTETS) are re-spelled as literals in the suites although src/steam.rs:71-117 publishes most of them and src/constants.rs:98 is already on the library face; a budget change then moves one side and surfaces as a confusing fixture assertion.", "fix": "Import the published constants where the crate exports them, and keep a local literal only where a suite deliberately pins the design's number independently of the code — marked as such on the spot."}, {"severity": "nit", "location": "apps/registry/tests/response_shape.rs:221, tests/admin_auth.rs:135 and :438, tests/candidates_join.rs:48-142, tests/catalog_document.rs:199, tests/slug_decoding.rs:39", "problem": "The `proves: Test strategy #N` markers now carry two designs' numbering: these inline markers cite design 026's strategy while the new hosted suites cite design 027's with the same bare form, so the mapping checklist item 1 is read off (e.g. a grep for \"Test strategy #9\") mixes design-027 health pins with design-026 admin-auth pins. Only the file headers disambiguate.", "fix": "Prefix the inline markers in the re-pinned legacy suites with the design they cite (\"proves: design 026's Test strategy #16 …\"), matching the style the new suites' headers already use."}]}
```
