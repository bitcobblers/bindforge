# Transcript: code-review:performance r1c1

## Provenance
- stage: code-review:performance (round r1c1)
- agent: codereview/code-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T10:07:36.180Z
- ended: 2026-11-22T10:19:02.384Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":94400,"output":24381,"reasoning":0,"cache.read":2949729,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/027-hosted-registry-with-store-resolvers.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r1c1.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-22T09:51:31.853Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (performance seat):
[
  {
    "severity": "major",
    "location": "apps/registry/src/state.rs:417-423 (deepening into apps/registry/src/corpus.rs:361-365, 280-291)",
    "problem": "Every mint deep-clones the whole issued Catalog (records and postings) and then Catalog::upsert runs rebuild_postings() — a full rebuild of the key→postings index — followed by a full ~30 MB re-serialization, all under the global `mutations` mutex that serializes mints and admin mutations for up to 64 request threads. The design's §Concurrency names this exact thing as forbidden: \"a mint inserts the new key's posting rather than rebuilding the index\". At 027-B3's 200k-record cap each novel-key mint pays the whole-model O(n) clone + O(n) index rebuild + O(n) serialize three times over, making the cloned postings copy pure waste; 027-S1 makes first-time steam keys the day-one ordinary path. Scale evidence: the suite's own 200k startup-load leg (one O(n) parse + postings build) takes 4.09s in debug (test result: ok. 1 passed; 0 failed; ... finished in 4.09s), so a per-mint hold of the one mutex near the cap is multi-hundred-ms-to-seconds. The design's §Risks write-amplification row prices… [elided by the checkpoint cap]",
    "fix": "Give Catalog a mint seam that inserts the record and pushes the new key's single posting into the existing postings in game_id order (one mint adds exactly one posting) instead of calling rebuild_postings, and avoid the deep postings clone (mutate the clone's postings in place, or swap the model through a single-slot Arc swap). No mint may rebuild the index. Then land the Rollout-4 measurement of the issued rewrite at 100k and 200k records beside the §Budgets figure."
  },
  {
    "severity": "minor",
    "location": "apps/registry/src/steam.rs:721-747, 787-857 (deadline checks only at steam.rs:572-580 and 627-631)",
    "problem": "STEAM_PULL_DEADLINE_SECONDS is not enforced while a page's header or body is being read. A peer that dribbles ≤8 KiB per <30 s never trips the per-connect socket timeout and never reaches a deadline check, so one page can run for hours and the startup path holds bind far past the named 1,920 s guarantee instead of failing as `steam pull deadline`. The suite's stall test only pins the total-silence leg, where the connect-time socket timeout actually saves it.",
    "fix": "Pass the pull deadline into read_header_block/read_body (or recompute the per-read socket timeout from deadline − now before each fill) and return the named `steam pull deadline` refusal when it is spent mid-page."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:247",
    "problem": "SteamSnapshot::publish serializes with serde_json::to_vec_pretty, inflating the daily snapshot write, the metadata-gated file size, and the next boot's parse by roughly half against the same MAX_STEAM_SNAPSHOT_OCTETS cap, for bytes that carry no content.",
    "fix": "Publish with serde_json::to_vec (compact), or if the pretty spelling is deliberate for operator inspection of the file, record that reason in a comment."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r1c1.md, and write no other file: an audit checks
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "apps/registry/src/corpus.rs:355-378 and apps/registry/src/state.rs:461, against 02-proposal.md Rollout item 4 (:730-731) and §Budgets:535-537", "problem": "Rollout 4's measured issued-document rewrite at 100k and 200k records is still not produced by anything in the reviewed paths, and §Budgets still states the figure prospectively. After this revision a novel-key mint at 027-B3's cap costs exactly two things — the Catalog::to_bytes_with serialization and one persist::save with sync_all, both under `mutations` — and §Deferred's deferral of batched issued writes is supposed to defer against that number. No leg yields it: the suite's only 200k-record leg (issued_entries.rs:355-411) refuses the novel key at the cap check (state.rs:439) before serializing, so its 3.88s debug runtime (measured this pass: `test result: ok. 1 passed; 0 failed; ... finished in 3.88s`) is document load, not rewrite. Recorded as missing evidence rather than a defect: the run's build notes are not an input to this seat, so if the figure was recorded there only the §Budgets line needs it.", "fix": "Add one #[ignore]d timing leg that boots against a 100k-record issued document and a 200k-record one and asks a single novel key, printing the to_bytes_with and persist::save elapsed times, and carry the two figures into §Budgets beside the memory figure."}, {"severity": "nit", "location": "apps/registry/src/steam.rs:750 and 793-814", "problem": "Deferring resolution to connect is what keeps a fresh-snapshot boot off the resolver, but to_socket_addrs is now asked once per page attempt (up to STEAM_REFRESH_MAX_PAGES × STEAM_PAGE_ATTEMPTS = 36 per pull) and is the one blocking network act on the pull path with no bound: connect checks the deadline at :743-746 and then resolves without a timeout. A host whose resolver is black-holed rather than failing can spend the resolver's own retry grid 36 times, giving the named 1,920s STEAM_PULL_DEADLINE_SECONDS guarantee an unaccounted tail — pre-bind, that is the bind held past what constants.rs:262-268 promises. With a cached resolver this is noise; it is a boundary the r1 fix moved, not a defect.", "fix": "Resolve once per pull and reuse the answer across the page loop's attempts (still never at startup), or state in dial_addresses that the resolver's own timeouts, not STEAM_PULL_DEADLINE_SECONDS, bound this leg."}, {"severity": "nit", "location": "apps/registry/src/steam.rs:884 and :1015 (the within_pull_deadline calls); apps/registry/tests/steam.rs:929 is the only deadline leg", "problem": "a_stalled_pull_is_cut_off_by_its_own_deadline pins the total-silence peer, where the per-connect socket read timeout would catch it anyway. No leg dials a fake that dribbles a header block or a body under the read timeout, which is exactly the peer this revision added the mid-read deadline guard for, so dropping either call site later stays green.", "fix": "Add a loopback leg with a short pull_deadline whose fake writes a few bytes every 50 ms after the headers, asserting the refusal names `steam pull deadline` and that the elapsed time is the deadline rather than a socket-timeout grid."}]}
```
