# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T01:59:14.656Z
- ended: 2026-11-19T02:04:40.180Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":69801,"output":27909,"reasoning":0,"cache.read":1024101,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 2efd6adee0318436891ae32ccd6be5b3d7dab059 4461833195ec56a1f702f869cb3c83db3f95c017` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Rollout item 3 + Risks row \"`024`'s identity fixtures and tests re-pin a second time\" + Test strategy #4 (\"The plugin-side skip case at the same bound is asserted separately\")",
    "problem": "The plan accounts for the new integration tests (#11-#13) and for named deletions, but not for the pre-existing apps/discovery-cli subprocess suites the new step 1 breaks. Verified: cli_tests.rs, cli_output_polish_tests.rs and cli_slug_refusal_tests.rs all run the built binary (CARGO_BIN_EXE_discovery-cli) against fixture/temp Steam roots and assert a rendered table or JSON array; with the probe gate landing before any plugin is constructed, every such run now exits non-zero with `error: catalog transport unavailable` and empty stdout, so each needs the loopback fake — harness work that appears in no Option, Rollout item, or Risk. Separately cli_slug_refusal_tests.rs pins the emission boundary end-to-end off a fixture whose display name is over-bound (`const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;`), which the design makes unproducible: a `steam/<appid>` key is at most 26 octets under its own 1-20-digit appid rule. As written the gates cannot pass without unplanned re-tool… [elided by the checkpoint cap]",
    "fix": "In Rollout item 3, name the migration: every existing apps/discovery-cli subprocess test runs behind a TcpListener fake (or an injected CatalogClient); and state that the CLI emit-boundary bound refusal moves to the in-source #[cfg(test)] hand-built-row pin in src/main.rs, deleting the fixture route in cli_slug_refusal_tests.rs rather than leaving it asserting an unreachable condition."
  },
  {
    "severity": "minor",
    "location": "Test strategy #14 (retirement) — \"`derive_slug`, `Namespace`, `namespace`, and `resolve_collisions` appear in no source under `crates/discovery` or `apps/discovery-cli`\"",
    "problem": "The textual-absence scope is far wider than the scheduled changes: grep finds 90 `namespace` occurrences in crates/discovery and 32 in apps/discovery-cli, across plugin_tests.rs, game_tests.rs, steam_linux_tests.rs, steam_windows_tests.rs, emission_tests.rs, zz_api_probe.rs, cli_dual_plugin_tests.rs, cli_tests.rs, cli_output_polish_tests.rs and cli_slug_refusal_tests.rs — none named for change by Rollout 1-5, which names only `slug.rs`, `slug_tests.rs`, `collision_tests.rs`, `resolve_collisions` and `Namespace`. It also catches prose, e.g. the doc comment above COLUMNS: \"(ADR-022, widened by ADR-024's namespace)\".",
    "fix": "Pin the compiled surface instead of the string — the `Namespace` type, the `namespace` field, `derive_slug` and `resolve_collisions` resolve nowhere in src/ or in any assertion — and list the re-pinned suites in Rollout 1 and 3."
  },
  {
    "severity": "minor",
    "location": "Corpus boundary and ledger — \"`!is_slug_name(slug)`, `!is_slug_name(friendly_name)`, and the slug is inside `025-B1` — the structural proof that a source key cannot become a product id, because the grammar refuses it (A5)\"",
    "problem": "The claim is structural only for the slug (the `/` decides it). For `friendly_name` the normalization rule (trim, collapse whitespace, clamp at a char boundary) leaves an all-lowercase single-word name verbatim, and display_name falls back to installdir; every current fixture happens to carry a capital or a space (\"Example Game\", \"Secondary Title\", \"BrokenMarker900202\"), so the pin passes by fixture accident and will fail confusingly for a future fixture named e.g. `doom`.",
    "fix": "Restrict \"the structural proof\" to the slug, and either state the friendly-name assertion as a fixture-contingent pin or replace it with the property 025-L5 actually needs — a friendly name is never used as a manifest/profile id or a path component."
  },
  {
    "severity": "minor",
    "location": "Scan flow and output, step 6 — \"A chosen candidate writes the record `(slug, game_id, the row's friendly name)`\" together with \"The answer applies to every row sharing the slug\"",
    "problem": "When several rows share one slug — the case the Emission section relies on (\"keeps the same game in two libraries as two rows\") — \"the row's friendly name\" names no row, and the store, \"keyed by `slug` alone\", holds one record where 025-C3 says \"one record per confirmed install\". The design acknowledges slug-sharing only for `local` (\"Two install directories differing only in case collapse to one `local/<slug>`\"), leaving the general case unstated.",
    "fix": "State that the store holds one record per distinct slug, and name the row whose friendly name is stored (the first in the existing (slug, install_path) order)."
  },
  {
    "severity": "minor",
    "location": "Assumption register A10 — \"The workspace's external deps are `serde`, `winreg`, `clap`, `serde_json`\" marked verified on basis \"Cargo.toml:20-24\"",
    "problem": "The cited basis does not support the stated set: [workspace.dependencies] lists bindforge-core, serde and winreg, and comments that clap is among the deps that \"enter when their consumer lands\"; clap and serde_json are per-package deps of apps/discovery-cli (`clap = { version = \"4\", features = [\"derive\"] }`, `serde_json = \"1.0\"`). The conclusion is safe — the CLI already has everything the new modules need, so no Cargo.lock change follows — but the row is marked verified against a basis that contradicts its wording.",
    "fix": "Restate A10 as \"the CLI's deps are `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`\", citing apps/discovery-cli/Cargo.toml:6-13 beside Cargo.toml:20-24."
  },
  {
    "severity": "minor",
    "location": "Bound — \"`schemas/README.md` gains a sibling bullet in the exported budget family: ADR-025 adds `MAX_DISCOVERY_SLUG_OCTETS = 128`\"",
    "problem": "The new bullet covers only the new constant, while the design also gives the existing MAX_NATIVE_NAME_OCTETS a second, non-schema use as the friendly_name bound (025-C2's borrowed sibling bound). Its stated homes stay narrow — the README bullet says \"one game-native action name\", 013-B4 the same — so nothing states the discovery row's display bound in the budget family.",
    "fix": "Add one clause to the new bullet: ADR-025 also borrows `MAX_NATIVE_NAME_OCTETS` as the `friendly_name` display bound, a non-schema use beside the lookup-key cap — leaving both existing bullets' text intact."
  },
  {
    "severity": "nit",
    "location": "Test strategy #4 — \"The plugin-side skip case at the same bound is asserted separately\"",
    "problem": "Through any plugin entry point that case is unreachable: the 1-20-digit appid rule caps a composed key at 26 octets, so only an in-module construction of AppManifest with a 130-digit appid can reach the MAX_DISCOVERY_SLUG_OCTETS skip in `discovered_game`.",
    "fix": "Say so in one line and pin it as an in-module AppManifest construction (defence in depth), or drop it and keep the refuse_invalid_slugs hand-built-row boundary pin."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/025-discovery-identity-and-catalog-matching.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Rollout 3 — \"including the migration that keeps the gate from breaking the pre-existing subprocess suites: `cli_tests`, `cli_output_polish_tests`, `cli_dual_plugin_tests` and `cli_slug_refusal_tests` run behind a loopback `TcpListener` fake\"","problem":"The named migration covers the gate only. Every one of those runs also loads the match store — and Test #12's confirmation round-trip writes it — through the subprocess's own XDG_DATA_HOME/HOME. Verified: apps/discovery-cli/tests/cli_tests.rs:24 and cli_output_polish_tests.rs:21 build the command with no .env call, so they inherit the developer's HOME and read (and the round-trip test would write) the real ~/.local/share/bindforge/matches.json; only cli_dual_plugin_tests.rs:35-36 clears the environment. The suite is therefore not hermetic and the store assertion in #12 has no owned target.","fix":"Extend the clause: each subprocess run pins XDG_DATA_HOME and HOME to a per-test temp dir beside the loopback fake, and Test #12 asserts the one file under that temp dir."},{"severity":"minor","location":"Match store, Save — \"`matches.json.tmp` (opened exclusively, never through a pre-existing entry) + `std::fs::rename`, after `create_dir_all` on the parent\"","problem":"Load's every failure is a named refusal, but a save that cannot create the parent or rename (read-only or missing data dir — the route cli_dual_plugin_tests' HOME=/bindforge-no-such-home already walks) has no stated outcome: no refusal name, no exit code, and no case in Test #7. A confirmed match can silently fail to persist.","fix":"Name one refusal (`error: match store unwritable: <path>`) with ExitCode::FAILURE, or state that the confirmation is lost and named; pin it as a Test #7 case with an unwritable parent."},{"severity":"minor","location":"Match store, Load — \"a `slug` past `025-B1` or not spelled `<token>/<external-id>` with `<token>` in `Source`'s closed set\", read with Scan flow steps 4 and 6","problem":"Step 4's emit boundary checks \"the `025-B1` bound alone\" and step 6 writes \"the one record for the slug\" with no shape check, so a row whose slug did not come from `lookup_key` — the case refuse_invalid_slugs' own doc comment names (\"from any `dyn DiscoveryPlugin`, including a future third-party one\", apps/discovery-cli/src/main.rs:105-111) — clears the boundary, gets confirmed, is written, and then makes the next scan refuse the whole store, whose stated remedy (\"remove the file\") destroys every unrelated match.","fix":"Apply Load's slug-shape rule at the confirmation write (the write refuses with the named error), so a record Load would reject can never enter the file."},{"severity":"minor","location":"Match store, Load — \"a file over a `65_536`-octet read budget\"","problem":"The budget is unowned arithmetic: one record is at most ~426 octets (the two 128 bounds plus the 128-octet friendly bound plus JSON punctuation) and typically ~95, so the store starts refusing between roughly 150 and ~690 confirmed matches — inside a large Steam library — and the only stated remedy is deleting every match. No record count is capped and no derivation or headroom is stated.","fix":"State the budget as records x worst-case record size, or replace the byte cap with a named \"more than N records\" refusal so an over-cap store reports the rule it broke."},{"severity":"minor","location":"Catalog client, `probe()` — \"after refusing unless every returned `SocketAddr` `IpAddr::is_loopback()` — the property is established by address, not by name\"","problem":"The address check sits only on the probe, while scan-flow step 5 makes one candidates() call per distinct unresolved slug and nothing says those requests reuse the validated address instead of re-resolving the name. The guarantee the Risks row claims (\"This build can only reach a loopback authority\") is thus established before enumeration but not on the requests that carry the installed-game inventory.","fix":"One clause: HttpCatalog connects to the SocketAddr the probe validated, or candidates() repeats the same is_loopback() refusal on every resolved address."},{"severity":"minor","location":"Scan flow step 8 — \"the two ACF-sourced cells — `Display Name`, `Install Path` — render through `escape_and_truncate` in place of today's raw `row_cells` pass-through\"","problem":"escape_and_truncate is the structured-error value renderer (crates/core/src/error.rs:190-204): it cuts at MESSAGE_VALUE_CHARS = 80 characters with a `…` and doubles every backslash. Adopting it for table cells silently clips long display names and install paths and renders Windows paths with doubled backslashes — a render-contract change the design does not state, while Test #12 pins only the ESC-byte case.","fix":"State both consequences in step 8 and pin them in Test #12, or escape control bytes without the 80-character cut if clipping is unwanted."},{"severity":"minor","location":"Assumption register A10 — \"those are the workspace's only external deps (`clap` and `serde_json` per-package, not workspace-level)\"","problem":"The row is marked verified against a basis that contradicts the wording: [workspace.dependencies] in Cargo.toml carries bindforge-core, serde and winreg and comments that clap enters \"when their consumer lands\", so the workspace's external deps are serde, winreg, clap, serde_json. The five names A10 lists include two path deps (discovery, bindforge-core) and omit winreg. (The conclusion, no Cargo.lock change, is safe.)","fix":"Drop the clause, or restate: \"the workspace's external deps are `serde`, `winreg`, `clap`, `serde_json`; the CLI's are `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`\"."},{"severity":"nit","location":"Catalog client, endpoint policy — \"an authority that is non-empty, carries no `@`, and runs to the first `/`, `?`, or `#`, with an optional `:` and digits-only port; and a host that is exactly `127.0.0.1`, `[::1]`, or `localhost`\"","problem":"With `[::1]` admitted, the \"optional `:`\" split is ambiguous because that host is itself colon-bearing, and Test #9 pins acceptance only for http://127.0.0.1:PORT, so the IPv6 case the prose admits is unpinned and could be refused by a first-match-on-colon parser.","fix":"Mirror RFC 3986 §3.2.2 in one clause — a leading `[` host runs to its `]` before the optional port colon — and add `http://[::1]:PORT` to Test #9's accepted list."},{"severity":"nit","location":"Catalog client — \"`fn candidates_url(base: &str, slug: &str) -> String` is pure\" beside the new \"`fn parse_endpoint(base: &str) -> Result<Endpoint, CatalogError>`\"","problem":"The same thing appears in two shapes: it is undefined whether candidates_url receives the raw flag/env string (so `http://127.0.0.1:8080/prefix` yields `/prefix/v1/candidates`) or the parsed Endpoint (so a path prefix is silently dropped).","fix":"Name in one clause which value candidates_url receives and what happens to a base path."},{"severity":"nit","location":"Scan flow step 6 — \"writes the one record for the slug, `(slug, game_id, friendly name of the first sharing row in `(slug, install_path)` order)`\"","problem":"The nested inline code spans close the first span mid-sentence, so the record tuple this rule now hinges on renders mangled.","fix":"Use one span with no inner backticks: `(slug, game_id, friendly_name of the first sharing row in (slug, install_path) order)`."}]}
```
