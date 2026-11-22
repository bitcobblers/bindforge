# Transcript: code-review:performance r1c0

## Provenance
- stage: code-review:performance (round r1c0)
- agent: codereview/code-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T07:32:52.226Z
- ended: 2026-11-22T07:44:16.889Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":120499,"output":20931,"reasoning":0,"cache.read":2032754,"cache.write":0}
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
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r1c0.md is yours: re-reading it is allowed, through your
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
Your own previous findings (performance seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r1c0.md, and write no other file: an audit checks
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"apps/registry/src/state.rs:417-423 (deepening into apps/registry/src/corpus.rs:361-365, 280-291)","problem":"Every mint deep-clones the whole issued Catalog (records and postings) and then Catalog::upsert runs rebuild_postings() — a full rebuild of the key→postings index — followed by a full ~30 MB re-serialization, all under the global `mutations` mutex that serializes mints and admin mutations for up to 64 request threads. The design's §Concurrency names this exact thing as forbidden: \"a mint inserts the new key's posting rather than rebuilding the index\". At 027-B3's 200k-record cap each novel-key mint pays the whole-model O(n) clone + O(n) index rebuild + O(n) serialize three times over, making the cloned postings copy pure waste; 027-S1 makes first-time steam keys the day-one ordinary path. Scale evidence: the suite's own 200k startup-load leg (one O(n) parse + postings build) takes 4.09s in debug (test result: ok. 1 passed; 0 failed; ... finished in 4.09s), so a per-mint hold of the one mutex near the cap is multi-hundred-ms-to-seconds. The design's §Risks write-amplification row prices only the whole-document rewrite and names the posting-insert as the in-memory mitigation; the clone+rebuild delta is unpriced, and the Rollout-4 deliverable (\"issued-document rewrite measured at 100k and 200k records, recorded in §Budgets\") has no artifact in the reviewable diff.","fix":"Give Catalog a mint seam that inserts the record and pushes the new key's single posting into the existing postings in game_id order (one mint adds exactly one posting) instead of calling rebuild_postings, and avoid the deep postings clone (mutate the clone's postings in place, or swap the model through a single-slot Arc swap). No mint may rebuild the index. Then land the Rollout-4 measurement of the issued rewrite at 100k and 200k records beside the §Budgets figure."},{"severity":"minor","location":"apps/registry/src/steam.rs:721-747, 787-857 (deadline checks only at steam.rs:572-580 and 627-631)","problem":"STEAM_PULL_DEADLINE_SECONDS is not enforced while a page's header or body is being read. A peer that dribbles ≤8 KiB per <30 s never trips the per-connect socket timeout and never reaches a deadline check, so one page can run for hours and the startup path holds bind far past the named 1,920 s guarantee instead of failing as `steam pull deadline`. The suite's stall test only pins the total-silence leg, where the connect-time socket timeout actually saves it.","fix":"Pass the pull deadline into read_header_block/read_body (or recompute the per-read socket timeout from deadline − now before each fill) and return the named `steam pull deadline` refusal when it is spent mid-page."},{"severity":"nit","location":"apps/registry/src/steam.rs:247","problem":"SteamSnapshot::publish serializes with serde_json::to_vec_pretty, inflating the daily snapshot write, the metadata-gated file size, and the next boot's parse by roughly half against the same MAX_STEAM_SNAPSHOT_OCTETS cap, for bytes that carry no content.","fix":"Publish with serde_json::to_vec (compact), or if the pretty spelling is deliberate for operator inspection of the file, record that reason in a comment."}]}
```
