# Transcript: code-review:compatibility r1c2

## Provenance
- stage: code-review:compatibility (round r1c2)
- agent: codereview/code-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-19T23:13:26.245Z
- ended: 2026-11-19T23:19:19.532Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":82293,"output":27453,"reasoning":0,"cache.read":1959437,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/026-registry-service/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff 06d528febd4bd7d8b875553668b4ebab6a3bd097..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/026-registry-service.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/026-registry-service/reviews/code/compatibility-r1c2.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-19T23:01:03.401Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "apps/registry/src/main.rs:199-208",
    "problem": "The missing-`<data>` startup refusal hard-codes \"set `XDG_DATA_HOME` to an absolute path\" even though the layout is resolved via `DataLayout::host()`; on the Windows branch (where `<data>` = absolute `%APPDATA%` per A6 and the matrix's Windows row) the message names a variable the platform rule never reads, and the unit test at main.rs:383-393 pins the Unix-only wording unconditionally.",
    "fix": "Branch the refusal on the resolved layout: name `%APPDATA%` on the Windows branch (or name both spellings), and assert the wording that matches `DataLayout::host()` in the test."
  },
  {
    "severity": "minor",
    "location": "apps/registry/src/upstream.rs:72-73,102-105",
    "problem": "`authority_of` (endpoint.rs:126) strips brackets from an IPv6 host and `fetch` rebuilds the authority as `format!(\"{host}:{port}\")`, so an admitted bracketed IPv6 upstream (endpoint.rs:44-53, unit-pinned at endpoint.rs:146 for `http://[::1]:8080`) sends an unbracketed `Host: ::1:8080` — not the bracketed form RFC 9110 requires. It works only against this registry's own Host-blind reader; a standards-conforming front (proxy/vhost edge) can reject or misroute it.",
    "fix": "Bracket the host when it contains ':' when building the Host header and authority string, e.g. `if host.contains(':') { format!(\"[{host}]:{port}\") } else { format!(\"{host}:{port}\") }`."
  },
  {
    "severity": "minor",
    "location": "apps/registry/tests/common/mod.rs:462-476",
    "problem": "`Registry::caching` relocates the instance's documents via `XDG_DATA_HOME` alone while `Registry::start` scrubs `XDG_DATA_HOME`/`HOME`/`APPDATA` from every child (common/mod.rs:671-677). On a Windows host the child resolves `<data>` from the scrubbed `APPDATA`, finds none, and refuses to start — every caching-mode subprocess suite (cache_document, caching_policy, upstream_client, upstream_endpoint_gate, data_layout, cli_end_to_end caching legs) fails on the platform the Compatibility matrix marks supported. Test strategy #13 and the harness header (common/mod.rs:13-15) both name `APPDATA` as part of this same child-environment seam; Linux-only CI hides it.",
    "fix": "In `Registry::caching`, set `XDG_DATA_HOME`, `HOME`, and `APPDATA` all to the sandbox directory (each layout ignores the other layouts' variables), making the helper portable and the #13 claim true for the whole suite."
  },
  {
    "severity": "nit",
    "location": "schemas/catalog/v1/fixtures/valid/ + apps/registry/tests/slug_decoding.rs:124-147",
    "problem": "Every fixture and suite key/name is pure ASCII: the widened alphabet is pinned for dots, `%`, and spaces, but survival of a non-ASCII lookup key or display name (byte-wise percent-encode, octet-counted caps, UTF-8-validated decode) is only structurally true, never asserted.",
    "fix": "Add one valid fixture with a non-ASCII external id (e.g. `local/Jörn`) plus a candidate-query row that round-trips it, and let the fixture-naming rule govern it."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/026-registry-service/reviews/code/compatibility-r1c2.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"apps/registry/src/main.rs:222-228 (test at main.rs:440-452)","problem":"Carried from round 1, unchanged by this increment: the missing-`<data>` startup refusal hard-codes \"set `XDG_DATA_HOME` to an absolute path\" although the layout is resolved via `DataLayout::host()`; on the Windows branch (a `Yes` row of the Compatibility matrix, `<data>` = absolute `%APPDATA%` per A6) the message names a variable the platform rule never reads, and the unit test pins the Unix-only wording unconditionally.","fix":"Branch the refusal on the resolved layout (name `%APPDATA%` on the Windows branch, or name both spellings) and assert the wording that matches `DataLayout::host()` in the test."},{"severity":"minor","location":"apps/registry/src/upstream.rs:72-73,102-105","problem":"Carried from round 1, unchanged by this increment: `authority_of` strips brackets from an IPv6 host and `fetch` rebuilds the authority as `format!(\"{host}:{port}\")`, so an admitted bracketed IPv6 upstream (endpoint.rs, pinned for `http://[::1]:8080`) sends an unbracketed `Host: ::1:8080`, not the bracketed form RFC 9110 requires. It works only against this registry's own Host-blind reader; a standards-conforming front (proxy/vhost edge) can reject or misroute it.","fix":"Bracket the host when it contains ':' when building the Host header and authority string, e.g. `if host.contains(':') { format!(\"[{host}]:{port}\") } else { format!(\"{host}:{port}\") }`."},{"severity":"minor","location":"apps/registry/tests/common/mod.rs:462-476 (with AMBIENT_ENV_VARS at 671-677)","problem":"Carried from round 1, unchanged by this increment: `Registry::caching` relocates an instance's documents via `XDG_DATA_HOME` alone while `Registry::start` scrubs `XDG_DATA_HOME`/`HOME`/`APPDATA` from every child, so on a Windows host the child resolves `<data>` from the scrubbed `APPDATA`, finds none, and refuses to start — every caching-mode subprocess suite (cache_document, caching_policy, upstream_client, upstream_endpoint_gate, data_layout, cli_end_to_end caching legs) fails on a platform the Compatibility matrix marks supported. Test strategy #13 and the harness header (common/mod.rs:13-15) name `APPDATA` as part of this same seam; Linux-only CI hides it. This increment reshaped legs on this seam but added no new caching-mode suite, so exposure is unchanged.","fix":"In `Registry::caching`, set `XDG_DATA_HOME`, `HOME`, and `APPDATA` all to the sandbox directory (each layout ignores the other layouts' variables), making the helper portable and the #13 claim true for the whole suite."},{"severity":"nit","location":"schemas/catalog/v1/fixtures/valid/ + apps/registry/tests/slug_decoding.rs:124-147","problem":"Carried from round 1, unchanged by this increment: every fixture and suite key/name is pure ASCII, so the widened alphabet is pinned for dots, `%`, and spaces while survival of a non-ASCII lookup key or display name (byte-wise percent-encode, octet-counted caps, UTF-8-validated decode) remains only structurally true — including now for the new octet-bounded answer window, which is byte arithmetic over serialized names.","fix":"Add one valid fixture with a non-ASCII external id (e.g. `local/Jörn`) plus a candidate-query row that round-trips it, and let the fixture-naming rule govern it."},{"severity":"nit","location":"apps/registry/tests/cli_end_to_end.rs:512-556 (helper at apps/registry/tests/common/mod.rs:1048-1054)","problem":"New in this increment: the explicit-`--port` leg takes the port it binds from `closed_loopback_port()`, a helper whose stated contract is \"the 'no reachable upstream' of `026-L4`\" — it binds a loopback listener, reads the assigned port, and drops it. Used as a port to re-bind in a child, the value is free only until something else takes it: with the test binary's parallel threads, a concurrent `FakeUpstream`/`SteamRoot` `bind(\"127.0.0.1:0\")` can be handed the just-learned port before the child binds it, and the leg then dies at startup (common/mod.rs:512-517 panics on the child exiting before printing a port). Linux's rotating ephemeral allocator makes the coincidence rare — hence nit, and the gate is green — but reuse of a recently closed port is the documented behavior on the matrix's Windows row, where this leg is claimed supported and never run.","fix":"Stop borrowing the unreachable-upstream helper for a bind: hold the probe listener until just before `Registry::start` and retry the child start once with a freshly learned port if the bind is taken, or drive the leg from a per-test port in a range no `bind(:0)` in this binary can be handed."}]}
```
