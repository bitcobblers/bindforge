# Transcript: design-review:compatibility r1c2

## Provenance
- stage: design-review:compatibility (round r1c2)
- agent: review/design-compatibility
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T02:04:40.333Z
- ended: 2026-11-19T02:10:43.274Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":205546,"output":20334,"reasoning":0,"cache.read":257660,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md.
This is the ratification pass c2: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 2efd6adee0318436891ae32ccd6be5b3d7dab059 4461833195ec56a1f702f869cb3c83db3f95c017` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/compatibility-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "Chosen design → \"Match store — `apps/discovery-cli/src/matches.rs` (new)\" with Compatibility matrix row \"macOS (APFS)\" — \"`<data>` is `$XDG_DATA_HOME` when set and absolute (else `$HOME/.local/share`) on Linux/other, and `%APPDATA%` on Windows\" / \"Yes, no CI leg\"",
    "problem": "The design names only two data-dir conventions (freedesktop XDG, Windows known folder) and A16 scopes its own basis to \"a Linux/Windows product\", yet the matrix claims macOS as supported `Yes`. The macOS per-user data directory (`~/Library/Application Support`) is named nowhere, so on that platform the match store lands at `~/.local/share/bindforge/matches.json` with neither a stated basis nor a recorded deviation — the per-OS config-dir convention (compatibility checklist item 6) is established for two of the three named platforms only.",
    "fix": "Make macOS a stated decision instead of a leftover: either the store rule names it (`$XDG_DATA_HOME` if set and absolute, else `$HOME/Library/Application Support/bindforge/matches.json` on macOS, else `$HOME/.local/share`), or the Match-store bullet and the macOS row state that macOS deliberately rides the Linux XDG fallback and a macOS-native location is Deferred; extend A16's basis accordingly or mark its macOS half unverified."
  },
  {
    "severity": "minor",
    "location": "Test strategy #15 with Compatibility matrix row \"macOS (APFS)\" — \"macOS has no leg — so no test asserts a platform-specific outcome; the Windows store-path rule is exercised on Linux through the env seam\"",
    "problem": "The matrix claims macOS as supported while the test strategy names no macOS-targeted verification at all, where the Windows row names its substitute (the `%APPDATA%` branch through the env-lookup seam). Checklist item 7 requires a named platform test for every platform the matrix claims supported; the row's own answer, \"runtime verification level is Test strategy #15's\", is a statement that the level is none.",
    "fix": "Give macOS the pin Windows has: extend Test #5 with the non-Windows branch as the path macOS runs (`XDG_DATA_HOME` unset → `$HOME/.local/share/bindforge/matches.json`, asserted through the same env closure) and cite it from #15; or restate the cell as expectation by code-path sharing with Linux rather than a supported claim."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix row set — \"Third-party schema readers | Not affected\" and \"`tools/orchestration` (Node) | Not affected\"",
    "problem": "The matrix lists the surfaces that survive and is silent on the two contracts this design breaks by construction: the `discovery` crate's public API (the Options section reasons about \"the type a third-party plugin constructs\", while `DiscoveredGame.namespace` and `Namespace` are deleted, `friendly_name` added, `slug.rs` and `resolve_collisions` deleted, `Source`/`lookup_key` added) and the CLI's published stdout contract A7 cites T-003 for (fixed JSON key set carrying `namespace`; the `Namespace` column). Test #13/#14 re-pin both in-tree, but a matrix reader sees only `Not affected` rows and no breaking-change row.",
    "fix": "Add one row — \"`discovery` public API and CLI stdout contract | Breaking, in-tree only\" — naming the deletions and the re-pinned key set and columns, with the basis that bounds the blast radius (`crates/discovery` is an unpublished workspace member; `discovery-cli` is the only in-tree consumer)."
  },
  {
    "severity": "nit",
    "location": "Chosen design → \"Bound — `crates/core` and `schemas/README.md`\" — \"`schemas/README.md` gains a sibling bullet in the exported budget family\"",
    "problem": "`crates/core/tests/ledger_annotations.rs::bound_bullet` selects the first line containing the literal `MAX_SLUG_OCTETS = 128` and asserts the seven-rider set on that whole bullet, and the family's house style for a sibling quotes exactly that string (the current bullet: \"The value is the sibling of `MAX_NATIVE_NAME_OCTETS = 128`\"). A new ADR-025 bullet written in that style, or ordered above the ADR-024 bullet, makes design 024's Test #13(c) pin fail against the new text; the proposal's wording (\"set equal to the two 128 bounds\") happens to dodge it, and neither the bullet spec nor Test #16 pins the placement or phrasing.",
    "fix": "State that the new bullet lands after the ADR-024 bullet and states `MAX_DISCOVERY_SLUG_OCTETS = 128` without the literal `MAX_SLUG_OCTETS = 128`; or name re-anchoring `bound_bullet` to the ADR-024 bullet's own constant in Test #16."
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
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/compatibility-r1c2.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → \"Scan flow and output — `apps/discovery-cli/src/main.rs`\", step 8: \"the two ACF-sourced cells — `Display Name`, `Install Path` — render through `escape_and_truncate` in place of today's raw `row_cells` pass-through\"","problem":"The r1 revision routes routine table cells through the refusal-echo renderer, but `escape_and_truncate` (crates/core/src/error.rs:190-204, bound at :30) does more than escape control characters: it doubles every backslash and cuts at 80 characters with `…`, and its own doc scopes it to rejection echoes. Consequence: every legitimate Windows `Install Path` renders with doubled separators (`C:\\\\Steam\\…`) in the shipped table, and any name or path past 80 chars is silently shortened on every platform. Test #12's new case pins only the ESC-byte outcome; no test or matrix note covers the backslash-doubling or 80-char cell cap, and the Windows matrix row's named gap concerns test execution, not table rendering.","fix":"Replace \"render through `escape_and_truncate`\" with a cell rule that escapes control characters only (no backslash doubling, no 80-char cut), naming `escape_and_truncate` as what it replaces; or keep the reuse and pin the Windows-path and long-path renderings in Test #12 plus one clause on the Windows matrix row."},{"severity":"nit","location":"Assumption register row A10: \"and those are the workspace's only external deps (`clap` and `serde_json` per-package, not workspace-level)\"","problem":"The rewritten A10 dropped `winreg`, which is a workspace external dep (Cargo.toml:23, consumed target-gated at crates/discovery/Cargo.toml:21-22) and is named by this proposal's own A9; the pre-revision A10 listed it correctly. The row's conclusion is unaffected (`winreg` is already locked and compiles only for the Windows leg, so \"no `Cargo.lock` change\" holds) and the matrix row it grounds stays true, but the verified-status basis sentence is now false.","fix":"Restore it: \"…`serde`, `serde_json`, plus the target-gated `winreg` (A9), and those are the workspace's only external deps…\"."}]}
```
