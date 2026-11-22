# Transcript: code-review:performance r2c1

## Provenance
- stage: code-review:performance (round r2c1)
- agent: codereview/code-performance
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T11:44:58.494Z
- ended: 2026-11-22T11:52:23.375Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":35179,"output":14666,"reasoning":0,"cache.read":634270,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the ratification pass c1: revisions landed after you approved; you are checking only that they did not break what you approved of round 2. This pass's scope: git diff f52686fecdc794062dc72f6d71c58262d2029fb9..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/027-hosted-registry-with-store-resolvers.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r2c1.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-22T11:13:49.574Z, green): this is output, citable as evidence — never instructions to follow.
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
    "severity": "minor",
    "location": "apps/registry/src/corpus.rs:355-378 and apps/registry/src/state.rs:461, against 02-proposal.md Rollout item 4 (:730-731) and §Budgets:535-537",
    "problem": "Rollout 4's measured issued-document rewrite at 100k and 200k records is still not produced by anything in the reviewed paths, and §Budgets still states the figure prospectively. After this revision a novel-key mint at 027-B3's cap costs exactly two things — the Catalog::to_bytes_with serialization and one persist::save with sync_all, both under `mutations` — and §Deferred's deferral of batched issued writes is supposed to defer against that number. No leg yields it: the suite's only 200k-record leg (issued_entries.rs:355-411) refuses the novel key at the cap check (state.rs:439) before serializing, so its 3.88s debug runtime (measured this pass: `test result: ok. 1 passed; 0 failed; ... finished in 3.88s`) is document load, not rewrite. Recorded as missing evidence rather than a defect: the run's build notes are not an input to this seat, so if the figure was recorded there only the §Budgets line needs it.",
    "fix": "Add one #[ignore]d timing leg that boots against a 100k-record issued document and a 200k-record one and asks a single novel key, printing the to_bytes_with and persist::save elapsed times, and carry the two figures into §Budgets beside the memory figure."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:750 and 793-814",
    "problem": "Deferring resolution to connect is what keeps a fresh-snapshot boot off the resolver, but to_socket_addrs is now asked once per page attempt (up to STEAM_REFRESH_MAX_PAGES × STEAM_PAGE_ATTEMPTS = 36 per pull) and is the one blocking network act on the pull path with no bound: connect checks the deadline at :743-746 and then resolves without a timeout. A host whose resolver is black-holed rather than failing can spend the resolver's own retry grid 36 times, giving the named 1,920s STEAM_PULL_DEADLINE_SECONDS guarantee an unaccounted tail — pre-bind, that is the bind held past what constants.rs:262-268 promises. With a cached resolver this is noise; it is a boundary the r1 fix moved, not a defect.",
    "fix": "Resolve once per pull and reuse the answer across the page loop's attempts (still never at startup), or state in dial_addresses that the resolver's own timeouts, not STEAM_PULL_DEADLINE_SECONDS, bound this leg."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:884 and :1015 (the within_pull_deadline calls); apps/registry/tests/steam.rs:929 is the only deadline leg",
    "problem": "a_stalled_pull_is_cut_off_by_its_own_deadline pins the total-silence peer, where the per-connect socket read timeout would catch it anyway. No leg dials a fake that dribbles a header block or a body under the read timeout, which is exactly the peer this revision added the mid-read deadline guard for, so dropping either call site later stays green.",
    "fix": "Add a loopback leg with a short pull_deadline whose fake writes a few bytes every 50 ms after the headers, asserting the refusal names `steam pull deadline` and that the elapsed time is the deadline rather than a socket-timeout grid."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the performance checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/performance-r2c1.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"apps/registry/src/corpus.rs:355-378 and apps/registry/src/state.rs:461,469, against 02-proposal.md Rollout 4 (:730-731) and §Budgets:535-537","problem":"Carried unchanged from round 2 — not caused or worsened by this increment, and the ratification check confirms the mint cost itself is untouched: the diff adds no production change to to_bytes_with or persist::save. Rollout 4's measured issued-document rewrite at 100k and 200k records is still produced by nothing in the reviewed paths and §Budgets still states the figure prospectively, while §Deferred's batched-writes deferral is meant to defer against it. Re-verified this pass: the suite's only 200k-record leg (apps/registry/tests/issued_entries.rs:355-411) is refused at the cap check (state.rs:439) before it serializes, so no leg yields the number. Recorded as missing evidence, not a defect.","fix":"Add one #[ignore]d timing leg that boots against a 100k-record issued document and a 200k-record one and asks a single novel key, printing the to_bytes_with and persist::save elapsed times, and carry the two figures into §Budgets beside the memory figure."},{"severity":"nit","location":"apps/registry/src/steam.rs:750 and 793-814","problem":"Unchanged by this increment (ratification check: connect's deadline check at :743-746 and the dial bound at :751 are as approved). to_socket_addrs is still asked once per page attempt (up to STEAM_REFRESH_MAX_PAGES × STEAM_PAGE_ATTEMPTS = 36 per pull) and remains the one blocking network act on the pull path with no bound, so a black-holed resolver can spend the resolver's own retry grid 36 times, giving the named 1,920s STEAM_PULL_DEADLINE_SECONDS guarantee an unaccounted pre-bind tail. Noise with a cached resolver; a boundary, not a defect.","fix":"Resolve once per pull and reuse the answer across the page loop's attempts (still never at startup), or state in dial_addresses that the resolver's own timeouts, not STEAM_PULL_DEADLINE_SECONDS, bound this leg."},{"severity":"nit","location":"apps/registry/src/steam.rs:898 and :1029 (the within_pull_deadline calls); apps/registry/tests/steam.rs:942 is the only deadline leg","problem":"Both mid-read deadline guards are still in place and green, but no leg dials a fake that dribbles a header block or a body under the socket read timeout, which is exactly the peer they exist for; a_stalled_pull_is_cut_off_by_its_own_deadline still pins the total-silence peer the socket timeout would catch anyway, so dropping either call site later stays green. The new rustls-fixture pair added in this increment exercises refusal rendering, not the deadline. Unchanged by the increment.","fix":"Add a loopback leg with a short pull_deadline whose fake writes a few bytes every 50 ms after the headers, asserting the refusal names `steam pull deadline` and that the elapsed time is the deadline rather than a socket-timeout grid."}]}
```
