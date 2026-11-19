# Transcript: design-review:correctness:attempt2 r1c0

## Provenance
- stage: design-review:correctness:attempt2 (round r1c0)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T01:49:17.553Z
- ended: 2026-11-19T01:53:52.986Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":68346,"output":32935,"reasoning":0,"cache.read":1008450,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/025-discovery-identity-and-catalog-matching.md,
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
verify what the design claims about the code, and the report at docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/correctness-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/025-discovery-identity-and-catalog-matching.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/correctness-r1c0.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Previous attempt's gate failure (log excerpt): stage-timeout.
Do not repeat the behavior that caused that failure.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"request-changes","findings":[{"severity":"major","location":"Rollout item 3 + Risks row \"`024`'s identity fixtures and tests re-pin a second time\" + Test strategy #4 (\"The plugin-side skip case at the same bound is asserted separately\")","problem":"The plan accounts for the new integration tests (#11-#13) and for named deletions, but not for the pre-existing apps/discovery-cli subprocess suites the new step 1 breaks. Verified: cli_tests.rs, cli_output_polish_tests.rs and cli_slug_refusal_tests.rs all run the built binary (CARGO_BIN_EXE_discovery-cli) against fixture/temp Steam roots and assert a rendered table or JSON array; with the probe gate landing before any plugin is constructed, every such run now exits non-zero with `error: catalog transport unavailable` and empty stdout, so each needs the loopback fake — harness work that appears in no Option, Rollout item, or Risk. Separately cli_slug_refusal_tests.rs pins the emission boundary end-to-end off a fixture whose display name is over-bound (`const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;`), which the design makes unproducible: a `steam/<appid>` key is at most 26 octets under its own 1-20-digit appid rule. As written the gates cannot pass without unplanned re-tooling of suites the design never names.","fix":"In Rollout item 3, name the migration: every existing apps/discovery-cli subprocess test runs behind a TcpListener fake (or an injected CatalogClient); and state that the CLI emit-boundary bound refusal moves to the in-source #[cfg(test)] hand-built-row pin in src/main.rs, deleting the fixture route in cli_slug_refusal_tests.rs rather than leaving it asserting an unreachable condition."},{"severity":"minor","location":"Test strategy #14 (retirement) — \"`derive_slug`, `Namespace`, `namespace`, and `resolve_collisions` appear in no source under `crates/discovery` or `apps/discovery-cli`\"","problem":"The textual-absence scope is far wider than the scheduled changes: grep finds 90 `namespace` occurrences in crates/discovery and 32 in apps/discovery-cli, across plugin_tests.rs, game_tests.rs, steam_linux_tests.rs, steam_windows_tests.rs, emission_tests.rs, zz_api_probe.rs, cli_dual_plugin_tests.rs, cli_tests.rs, cli_output_polish_tests.rs and cli_slug_refusal_tests.rs — none named for change by Rollout 1-5, which names only `slug.rs`, `slug_tests.rs`, `collision_tests.rs`, `resolve_collisions` and `Namespace`. It also catches prose, e.g. the doc comment above COLUMNS: \"(ADR-022, widened by ADR-024's namespace)\".","fix":"Pin the compiled surface instead of the string — the `Namespace` type, the `namespace` field, `derive_slug` and `resolve_collisions` resolve nowhere in src/ or in any assertion — and list the re-pinned suites in Rollout 1 and 3."},{"severity":"minor","location":"Corpus boundary and ledger — \"`!is_slug_name(slug)`, `!is_slug_name(friendly_name)`, and the slug is inside `025-B1` — the structural proof that a source key cannot become a product id, because the grammar refuses it (A5)\"","problem":"The claim is structural only for the slug (the `/` decides it). For `friendly_name` the normalization rule (trim, collapse whitespace, clamp at a char boundary) leaves an all-lowercase single-word name verbatim, and display_name falls back to installdir; every current fixture happens to carry a capital or a space (\"Example Game\", \"Secondary Title\", \"BrokenMarker900202\"), so the pin passes by fixture accident and will fail confusingly for a future fixture named e.g. `doom`.","fix":"Restrict \"the structural proof\" to the slug, and either state the friendly-name assertion as a fixture-contingent pin or replace it with the property 025-L5 actually needs — a friendly name is never used as a manifest/profile id or a path component."},{"severity":"minor","location":"Scan flow and output, step 6 — \"A chosen candidate writes the record `(slug, game_id, the row's friendly name)`\" together with \"The answer applies to every row sharing the slug\"","problem":"When several rows share one slug — the case the Emission section relies on (\"keeps the same game in two libraries as two rows\") — \"the row's friendly name\" names no row, and the store, \"keyed by `slug` alone\", holds one record where 025-C3 says \"one record per confirmed install\". The design acknowledges slug-sharing only for `local` (\"Two install directories differing only in case collapse to one `local/<slug>`\"), leaving the general case unstated.","fix":"State that the store holds one record per distinct slug, and name the row whose friendly name is stored (the first in the existing (slug, install_path) order)."},{"severity":"minor","location":"Assumption register A10 — \"The workspace's external deps are `serde`, `winreg`, `clap`, `serde_json`\" marked verified on basis \"Cargo.toml:20-24\"","problem":"The cited basis does not support the stated set: [workspace.dependencies] lists bindforge-core, serde and winreg, and comments that clap is among the deps that \"enter when their consumer lands\"; clap and serde_json are per-package deps of apps/discovery-cli (`clap = { version = \"4\", features = [\"derive\"] }`, `serde_json = \"1.0\"`). The conclusion is safe — the CLI already has everything the new modules need, so no Cargo.lock change follows — but the row is marked verified against a basis that contradicts its wording.","fix":"Restate A10 as \"the CLI's deps are `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`\", citing apps/discovery-cli/Cargo.toml:6-13 beside Cargo.toml:20-24."},{"severity":"minor","location":"Bound — \"`schemas/README.md` gains a sibling bullet in the exported budget family: ADR-025 adds `MAX_DISCOVERY_SLUG_OCTETS = 128`\"","problem":"The new bullet covers only the new constant, while the design also gives the existing MAX_NATIVE_NAME_OCTETS a second, non-schema use as the friendly_name bound (025-C2's borrowed sibling bound). Its stated homes stay narrow — the README bullet says \"one game-native action name\", 013-B4 the same — so nothing states the discovery row's display bound in the budget family.","fix":"Add one clause to the new bullet: ADR-025 also borrows `MAX_NATIVE_NAME_OCTETS` as the `friendly_name` display bound, a non-schema use beside the lookup-key cap — leaving both existing bullets' text intact."},{"severity":"nit","location":"Test strategy #4 — \"The plugin-side skip case at the same bound is asserted separately\"","problem":"Through any plugin entry point that case is unreachable: the 1-20-digit appid rule caps a composed key at 26 octets, so only an in-module construction of AppManifest with a 130-digit appid can reach the MAX_DISCOVERY_SLUG_OCTETS skip in `discovered_game`.","fix":"Say so in one line and pin it as an in-module AppManifest construction (defence in depth), or drop it and keep the refuse_invalid_slugs hand-built-row boundary pin."}]}
```
