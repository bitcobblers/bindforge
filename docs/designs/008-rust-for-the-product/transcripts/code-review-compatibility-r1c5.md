# Transcript: code-review:compatibility r1c5

## Provenance
- stage: code-review:compatibility (round r1c5)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-11T20:59:50.231Z
- ended: 2026-11-11T21:10:29.962Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/008-rust-for-the-product/02-proposal.md.
This is the ratification pass c5: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/008-rust-for-the-product.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "crates/core/src/document.rs:243-258",
    "problem": "`quoted()` escapes only `\"`, `\\`, `\\n`, `\\r`, `\\t` and pushes every other char raw, so an id containing a control char the reader decodes (`\\b` → U+0008, `\\f` → U+000C, src/toml.rs:161-162) re-serializes into text its own reader rejects (src/toml.rs:175) and no TOML v1.1 reader accepts. This breaks the contract's \"valid fixture re-serializes to a semantically equal value\" promise (schemas/README.md, schemas/profile/v1/spec.md:50-51); no shipped fixture triggers it, so the gate stays green.",
    "fix": "Encode remaining chars < 0x20 (and 0x7F) as `\\u00XX` in `quoted()`, and add a profile/v1/fixtures/valid/ fixture whose id uses such an escape so the round trip is covered by the corpus."
  },
  {
    "severity": "minor",
    "location": "crates/core/src/toml.rs:155-173",
    "problem": "The v1 grammar never pins the string-escape set (schemas/README.md and both spec.md say only \"quoted string\"), and the reader refuses TOML v1.1's `\\uXXXX`/`\\UXXXXXXXX`. A third-party writer that escapes non-ASCII emits a document bindforge-core rejects, while bindforge-core writes non-ASCII literally — so non-ASCII binding names survive the format in one direction only, and the asymmetry is invisible to implementers and untested by the corpus.",
    "fix": "Accept `\\uXXXX`/`\\UXXXXXXXX` (rejecting lone surrogates and code points above U+10FFFF), or state the exact permitted escape set in schemas/README.md and both spec.md files and add an invalid fixture for `\\u`."
  },
  {
    "severity": "minor",
    "location": ".github/workflows/check.yml:25-30",
    "problem": "README.md:21-23 and .opencode/skills/repo-practices/SKILL.md:53-58 (and the proposal's Workspace shape, A8/A11) promise \"Cargo.lock is committed; CI uses `--locked`\", but only the test step passes it. `cargo clippy --workspace --all-targets` runs first without `--locked`, so a resolution change rewrites Cargo.lock inside the job and the later `cargo test --locked` verifies the already-mutated lock; the gate can pass on a dependency graph no contributor has, on every platform.",
    "fix": "Use `cargo clippy --workspace --all-targets --locked -- -D warnings` (fmt resolves no graph), or record in Gates and repo-practices that only cargo test is locked and why."
  },
  {
    "severity": "nit",
    "location": ".gitattributes (absent at repo root)",
    "problem": "The Compatibility matrix commits to a Windows contributor dev env and this change makes schemas/**/*.toml a byte-level contract, but no line-ending policy is pinned: a Windows checkout with core.autocrlf=true yields CRLF contract trees and sources. It works only by accident (the reader tolerates CRLF via str::lines() + trim in src/toml.rs:46-48, and that tolerance is not stated in the contract), and rustfmt's default newline_style=\"Auto\" is platform-native, so `cargo fmt` on Windows rewrites every .rs file to CRLF.",
    "fix": "Add .gitattributes (`*.rs text eol=lf`, `*.toml text eol=lf`, or `* text=auto eol=lf`) and one line in schemas/README.md stating the line-ending rule implementers must accept."
  },
  {
    "severity": "nit",
    "location": "crates/core/tests/conformance.rs:144-164",
    "problem": "The only guard aimed at filesystem case sensitivity is unreachable: the preceding assertion (conformance.rs:137-141) already forces every fixture name segment to lowercase ASCII via is_fixture_name (tests/common/mod.rs:191-205), so no surviving name can collide case-insensitively; the lowercased sort/dedup check can never fail.",
    "fix": "Either drop the dedup check and note that the lowercase rule is itself the collision guard, or allow case-unique mixed-case names so the check does real work."
  },
  {
    "severity": "nit",
    "location": "rust-toolchain.toml:6-7",
    "problem": "The gate needs the rustfmt and clippy components, but the toolchain file pins only `channel`; the one-gate promise (Test strategy #5, \"nothing installed beyond the pinned toolchain\") therefore depends on rustup's default `recommended` profile, and a `--profile minimal` contributor or image fails on an unchanged PR.",
    "fix": "Add `components = [\"rustfmt\", \"clippy\"]` to the `[toolchain]` table."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/code/compatibility-r1c5.md, and write no other file: an audit checks
that review seats touch only their own report.
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "crates/core/src/toml.rs:230-231,247-265 + crates/core/src/document.rs:310 + schemas/profile/v1/fixtures/valid/escaped-strings.toml", "problem": "The fix commits widened the contract — both spec.md grammar sections now promise exactly nine escapes (`\\b \\t \\n \\f \\r \\\" \\\\ \\uXXXX \\UXXXXXXXX`, profile/v1/spec.md:16-17, reach-classes/v1/spec.md:26-27) — but nothing in the corpus or in crates/core/tests exercises them. escaped-strings.toml pins only five of the nine; no fixture or test covers `\\uXXXX`/`\\UXXXXXXXX` decode, the surrogate and past-U+10FFFF refusals in parse_hex_chars, the \"and no other\" half of the rule, or the writer's `\\u{:04x}` branch. No contract document anywhere carries a non-ASCII id (both reach tables and every fixture id are ASCII), so with fixtures declared the conformance currency (schemas/README.md:41-43), a third-party reader that mishandles `\\u00e9` or `\\ud800` still passes the suite, and the non-ASCII-survives-the-format property stays unproven.", "fix": "Add a profile/v1/fixtures/valid/ document whose id mixes a literal non-ASCII name, a `\\u00e9` escape, and a `\\u0001` control escape (round-tripping the writer's `\\uXXXX` branch), plus fixtures/invalid/ cases for `\"\\ud800\"`, `\"\\U00110000\"`, and `\"\\_\"`; mirror the decode/refusal cases with a grammar_closure.rs unit test."}, {"severity": "nit", "location": "crates/core/src/reach.rs:103-131 (with 55-57)", "problem": "The fix added `hand_matches_file_name` and a truth table whose comment (reach_tables.rs:131-137) argues exact matching matters because a case-insensitive filesystem collapses the two names, but `load_reach_tables` never calls it: it builds `{hand}.toml` itself and compares only the declared `hand`. A version directory holding `Left.toml` instead of `left.toml` therefore loads fine on Windows (case-insensitive open, declared hand matches) and fails on Linux with an Io refusal — the spec.md:11-13 file-name rule is unenforced on precisely the platform where the case distinction vanishes, so the same contract tree behaves differently on the two platforms the Compatibility matrix commits to.", "fix": "Enumerate the version directory in `load_reach_tables` and pass each on-disk stem through `hand_matches_file_name`, refusing any `*.toml` whose stem is not exactly `left`/`right`; or state in reach-classes/v1/spec.md that the name check is the caller's obligation."}, {"severity": "nit", "location": ".gitattributes (absent at repo root)", "problem": "Carry-over from r1, slightly more load-bearing after the fixes: both grammar sections now declare \"Line-oriented entries, one per line\" (profile/v1/spec.md:12, reach-classes/v1/spec.md:22-24) and the corpus is the contract every implementer runs, yet no line-ending policy is pinned. A Windows contributor checkout with core.autocrlf=true yields CRLF contract trees that parse only by accident — `str::lines()` strips the trailing CR and each line is trimmed (toml.rs:106-108) — a tolerance no contract document states and no third-party reader is obliged to reproduce; rustfmt's default newline_style=\"Auto\" is platform-native on top of that.", "fix": "Add a root .gitattributes (`*.rs text eol=lf`, `*.toml text eol=lf`) and one line in schemas/README.md stating whether a reader must accept CRLF terminators."}]}
```
