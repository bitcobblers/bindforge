# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 01c3857a43fc
- started: 2026-11-11T18:32:29.693Z
- ended: 2026-11-11T18:37:54.669Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/008-rust-for-the-product/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 04a4ae1a2d2f75dc291f14e573660a483f51bbaa 6e4ea214d807aebc3faf5a2e28068a7b30b54411` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/008-rust-for-the-product.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one,
even when a named path lives under a directory you must not open.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Workspace shape, root Cargo.toml block: `license = \"MIT\"          # matches LICENSE + tools/orchestration (A23)`",
    "problem": "The claim is false. The repo's LICENSE file is the Apache License 2.0 text, while tools/orchestration/package.json:29 declares \"license\": \"MIT\" — the repo is already split, and A23's recorded basis (\"repo listing 2026-11-11; tools/orchestration/package.json:23\") covers only the absence of the product tree and the npm check script, so nothing cited supports the license half of this row. Rollout step 1 copies the block verbatim, so the design commits every crate to MIT against an Apache-2.0 LICENSE.",
    "fix": "Set `license = \"Apache-2.0\"` to match LICENSE (or record an explicit relicensing decision) and replace the comment with a citation of the two license files, or a new register row, instead of A23; record the package.json-MIT vs LICENSE-Apache-2.0 mismatch as a separate repo fix rather than papering over it."
  },
  {
    "severity": "major",
    "location": "Chosen design → \"Adversarial guard\" vs Non-goals (\"The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009\") and Deferred (\"chain/patch/policy/manifest semantics and their fixture cases — owned by ADR-009\")",
    "problem": "This design simultaneously defers chain semantics to ADR-009 and decides them: it fixes `ChainTooDeep` \"(chain longer than `MAX_CHAIN_DEPTH = 32`)\", puts the constant in `schemas/README.md` \"so third-party readers can conform\", and Test strategy #3 places chain-boundary fixtures under `schemas/profile/v1/fixtures/invalid/` in the scaffold. An implementer cannot tell which document owns the depth bound, its error variants, and their fixtures — the guard is the one piece of chain semantics that must be a public cross-language contract here.",
    "fix": "One sentence in the guard paragraph: the bound and the `Cycle`/`ChainTooDeep` variants are decided by this design and are a constraint ADR-009 must adopt; then narrow the Deferred/Non-goals clause to \"chain semantics other than the depth guard and its boundary fixtures\"."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Workspace shape, root Cargo.toml: `members = [\"crates/*\", \"apps/*\"]` with no `resolver`",
    "problem": "The block is a virtual manifest, and the Cargo page A7 cites says a virtual workspace must set `resolver` explicitly because it has no `package.edition` to infer it from. As committed, cargo warns on every command and falls back to the edition-independent resolver, silently splitting feature unification across core/transforms — against the stated goal that workspace policy is \"declared exactly once\".",
    "fix": "Add `resolver = \"3\"` to `[workspace]` in the block (it is copied verbatim in Rollout step 1)."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Workspace shape, `[workspace.dependencies]`: `serde = { features = [\"derive\"] }   # exact version pinned at merge`",
    "problem": "A workspace dependency entry needs a source, and the cited Cargo reference states that members inheriting a workspace dependency \"cannot use any other dependency key (such as `version`)\" — no member can supply the pin later, so the committed block is not a resolvable manifest.",
    "fix": "Give the floor now (`serde = { version = \"1\", features = [\"derive\"] }`), or drop the entry and say it lands with the first crate that needs serde."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Workspace shape members note: \"(the `crates/*` and `apps/*` globs follow A7's flat-members recommendation)\" vs table row `apps/desktop` → \"`src-tauri` package `bindforge-desktop`\"",
    "problem": "`apps/*` matches one path segment, so a crate at `apps/desktop/src-tauri` is never matched by the committed globs; the workspace silently drops a member when the desktop era lands. The TypeScript boundary defers the exact `members` path but never says the glob cannot reach it.",
    "fix": "Add a clause to the members note: the desktop era adds an explicit `members` entry for the `src-tauri` path, since the flat globs will not match it."
  },
  {
    "severity": "minor",
    "location": "Formats contract → harness paragraph: \"walks every `schemas/*/v*/fixtures/{valid,invalid}` directory, and asserts: every valid fixture parses and re-serializes to a semantically equal value\"",
    "problem": "The tree gives `reach-classes/v1/` only \"tables as TOML data + fixtures/\" with no valid/invalid split, and Test strategy #4 tests reach tables through a separate loader contract. The glob either sweeps reach-class data into a round-trip assertion that is undefined for it, or matches nothing and the directory goes unexercised — an unevaluated empty/absent-input edge case.",
    "fix": "Give `reach-classes/v1/` the same `fixtures/valid` + `fixtures/invalid` layout and state that table data is asserted by load + per-hand distinctness (Test #4), not by re-serialization."
  },
  {
    "severity": "minor",
    "location": "Formats contract → versioning rule: \"a file's presence in `vN/fixtures/valid/` is the definition of \\\"supported\\\"\" next to rejection \"with a structured error naming the supported majors\"",
    "problem": "Two definitions of \"supported\" — repo-side fixture directories and a compiled-in list in bindforge-core — with nothing in Test strategy keeping them equal, and the rule never says what happens to a document that declares no version at all.",
    "fix": "Add a test asserting core's declared supported majors equal the majors present under `schemas/*/v*`, and state that an absent version declaration is a structured rejection, with a fixture under `schemas/profile/v1/fixtures/invalid/`."
  },
  {
    "severity": "nit",
    "location": "Assumption register A2: \"axum 0.8.9 ships, MSRV 1.80, breaking 0.9 in dev, consumes `tower` | verified\"",
    "problem": "The cited README supports MSRV 1.80, 0.9-in-dev and tower, but only states that the `0.8.x` branch is what is released to crates.io — it does not support the exact patch 0.8.9.",
    "fix": "Cite crates.io/docs.rs for the patch number, or write \"0.8.x\"."
  },
  {
    "severity": "nit",
    "location": "Assumption register A16: \"WebView2 is preinstalled on Windows 10 (1803+) and 11 but \\\"some devices might not\\\" have it ... | verified | https://v2.tauri.app/start/prerequisites/\"",
    "problem": "The quoted phrase is not on the cited Tauri prerequisites page (it says WebView2 \"is already installed on Windows 10 (from version 1803 onward)\"); the sentence appears on the WebView2 distribution page cited by A17.",
    "fix": "Attribute the quote to the A17 URL instead of the prerequisites page."
  },
  {
    "severity": "nit",
    "location": "Gates → canonical sqlx rule: \"every gate job builds with `SQLX_OFFLINE=true` and no `DATABASE_URL`; a dedicated hub-path CI job with a Postgres service runs `cargo sqlx prepare --check --workspace`\"",
    "problem": "The sqlx-cli README states all sqlx commands require a database URL, so the hub-path `prepare --check` job cannot be one of the jobs described as building with no `DATABASE_URL`; the two sentences overlap as written.",
    "fix": "Say \"the two PR gate jobs build with `SQLX_OFFLINE=true`\", and note the hub-path job runs with `DATABASE_URL` set and `SQLX_OFFLINE` unset."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix, macOS row: \"unknown | BindForge commits nothing\"",
    "problem": "The phrasing is ambiguous (reads as \"contributes nothing\" as easily as \"commits to nothing\"), obscuring the single thing the row decides.",
    "fix": "Reword as \"no macOS support committed for v1\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/008-rust-for-the-product.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/008-rust-for-the-product/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Test strategy #1 vs Formats contract harness paragraph — harness: \"Reach-class table fixtures are asserted by load + distinctness (Test strategy #4), not re-serialization.\" vs Test #1: \"for every directory under `schemas/*/v*/fixtures/valid/`, `bindforge-core` parses the fixture and re-serializes to a semantically equal value\"","problem":"The r1 closure gave reach-classes/v1 a fixtures/valid/ directory, so Test #1's unqualified glob now sweeps reach-class valid fixtures into the re-serialization assertion that the harness paragraph explicitly says does not apply to them. The two sentences directly contradict for that one directory; an implementer cannot tell whether the conformance harness must round-trip reach tables. New, created by the revised text.","fix":"Scope Test #1 to the document formats: \"for every `profile`/`patch`/`manifest`/`policy` directory under `schemas/*/v*/fixtures/valid/`…\", and append \"reach-class fixtures follow #4\", mirroring the harness sentence."},{"severity":"nit","location":"Goals / Non-goals — \"The model's semantics (taxonomy, reach tables, chains, policies, manifest contents) — owned by ADR-009\"","problem":"Re-raised residue of my r1 major (which is substantively closed): the guard paragraph and the Deferred clause now carve out the depth guard, error variant names, and input budget as decided by this design, but the Non-goals bullet still lists `chains` unqualified, restating the ownership the guard paragraph just partially reserved. My r1 fix asked to narrow \"Deferred/Non-goals\"; only Deferred was narrowed. Residual wording ambiguity only — the specific guard sentence resolves the ownership question.","fix":"Mirror the Deferred phrasing in the Non-goals bullet: \"chains, except the depth guard and its boundary fixtures owned by this design\"."},{"severity":"nit","location":"Rollout step 5 — \"the cargo gate commands, the Node floor, the capability-file rule (own commands only; no `fs`/`shell`/`dialog` plugins), tests-under-each-crate\"","problem":"New in the revised text: the design never states a Node floor anywhere; the only Node statement is \"Node version pinned explicitly in a `setup-node` step\" (Gates). Rollout 5 instructs the repo-practices Rust section to record a value this design has not decided.","fix":"Either state the Node floor explicitly (e.g. the LTS line matching tools/orchestration), or rewrite the Rollout 5 item as \"the Node version pinned in the `orchestration` CI job\"."}]}
```
