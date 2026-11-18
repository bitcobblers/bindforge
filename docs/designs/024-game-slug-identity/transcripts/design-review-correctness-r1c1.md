# Transcript: design-review:correctness r1c1

## Provenance
- stage: design-review:correctness (round r1c1)
- agent: review/design-correctness
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T18:02:27.469Z
- ended: 2026-11-18T18:05:31.640Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":34903,"output":19753,"reasoning":0,"cache.read":383783,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/024-game-slug-identity/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 3012370eafcd855aa268a91062c7f6a6b2c05128 c326ff4ae38c21d4e72f1bd2bc2f7c164c913292` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/024-game-slug-identity.md,
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
verify what the design claims about the code, and the report at docs/designs/024-game-slug-identity/reviews/design/correctness-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (correctness seat):
[
  {
    "severity": "major",
    "location": "Chosen design → Derivation — \"fold precomposed Latin via a private `fold_latin_base(char) -> Option<char>` match table covering Latin-1 Supplement and Latin Extended-A single-base-letter decompositions, both cases\"",
    "problem": "The steps are stated \"in order\" with ASCII case-fold first and the Latin fold second, and the Options section says each covered letter \"maps to that base\", meaning its Unicode canonical-decomposition base. For every uppercase precomposed letter that base is ASCII uppercase (Å → A, É → E, İ → I), and the ASCII case-fold has already run behind it, so the next clause — \"map every remaining character outside `[a-z0-9]` to `-`\" — dashes it out. `Åpen` derives `pen`; an accented all-caps title loses every accented letter. That is precisely the accented-folds-to-`-` loss the Options section cites as the reason to keep the diacritic clause (\"\\\"Pokémon\\\" → `pok-mon`\"). Test strategy #1 cannot catch it: its diacritic cases (é, ü, ñ) are all lowercase, and A15's registered claim is about the fold reproducing NFKD, not about case.",
    "fix": "One clause: the table maps each covered letter to its lower-case ASCII base (or the ASCII case-fold is re-applied after the diacritic fold); add an upper-case accented case (É → e, Å → a) to Test strategy #1."
  },
  {
    "severity": "major",
    "location": "Chosen design → Specs, fixtures, ledger — \"the four `spec.md` sentences (A3) are amended in place: the number and the RFC 1035 derivation are replaced by a cross-reference\"",
    "problem": "A3's action-ids citation is `schemas/action-ids/v1/spec.md:54` only, so line 58 of the same sentence is left unamended: \"`move..steer`, `steer-left`, `move.steer.left`, and a 64-octet segment are each a refusal, and each has a fixture under `fixtures/invalid/` naming it\". Under `MAX_SLUG_OCTETS = 128` a 64-octet segment is legal, so the amended published spec refuses something two lines above it now permits, and the fixture it promises to name is the file this run renames to `segment-past-cap.toml` with a 129-octet segment.",
    "fix": "Extend the amendment list and A3's citation to that example clause: replace \"a 64-octet segment\" with \"a segment past the shared bound\" so it names the renamed fixture instead of a stale length."
  },
  {
    "severity": "minor",
    "location": "Test strategy #6 — \"`crates/core/tests/{manifest.rs, policy.rs, chain_ops.rs, action.rs}` accept 128 and refuse 129\"",
    "problem": "Those four are exactly the files pinning the 63/64 boundary, but the file pinning the action-ids invalid fixture's *name* is a fifth: `crates/core/tests/action_ids.rs:124` lists `\"segment-past-63-octets.toml\"` in its required-fixture set. Neither Test strategy #6 nor #10 names it, so the fixture rename this design orders has no named re-pin site.",
    "fix": "Add `action_ids.rs` (with the new literal `segment-past-cap.toml`) to #6's file list and to Rollout item 1's re-pin list."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Emission, refusal and collisions — \"refusals increment the existing `skipped` counter, and that diagnostic's wording widens from \\\"unreadable or malformed\\\" to name un-derivable names as well\"",
    "problem": "That diagnostic is `crates/discovery/src/steam_common.rs:101`, guarded by `!self.roots.is_empty() && games.is_empty() && skipped > 0`: it fires only when the plugin emitted nothing. A scan that refuses one game and emits the rest prints nothing, so the widened wording never appears in the ordinary mixed case, and 024-L1's \"a violation is a named refusal\" is user-invisible for the Steam plugins — their rows are refused inside the plugin and never reach the CLI's named `error: refused game with invalid slug:` line.",
    "fix": "Emit one named line per construction-path refusal through `escape_and_truncate` (mirroring the CLI's line, no path), or state that plugin-side refusals are count-only and that 024-L1's named refusal is the CLI-boundary line alone."
  },
  {
    "severity": "minor",
    "location": "Chosen design → Identity shape — \"`pub enum Namespace { Unknown }` with `#[serde(rename_all = \\\"kebab-case\\\")]` and a `Display` impl, mirroring `Platform`'s pattern\"",
    "problem": "The pattern cited does not exist: `Platform` (`crates/discovery/src/game.rs:5-18`) carries no serde attribute, and `apps/discovery-cli/tests/cli_tests.rs:186-189` pins `obj[\"platform\"] == \"SteamLinux\"` with the comment that \"the r1 major `#[serde(rename_all = \\\"kebab-case\\\")]` was reverted as unauthorized\". A builder following the cited precedent emits `\"Unknown\"` against a value pinned to `\"unknown\"` in three places, and the JSON row ends up mixing `\"namespace\":\"unknown\"` with `\"platform\":\"SteamLinux\"` without that being a stated, deliberate divergence.",
    "fix": "Delete \"mirroring `Platform`'s pattern\" and state the deliberate divergence from it (and that the closed-enum JSON spelling in this row is now mixed)."
  },
  {
    "severity": "nit",
    "location": "Chosen design → Shared grammar and bound (\"surveyed-title rationale per ADR-024, with A16's caveat\") vs Specs, fixtures, ledger (\"with the substituted rationale\")",
    "problem": "The same constant is given two different published rationales; the README is 024-L3's one stated home, so its wording is the contract while the code doc comment cites a clause the README drops.",
    "fix": "Write the rationale once and reuse it in both places, stating whether 024-B1's \"one octet above the longest surveyed store display title\" clause is carried (with A16's caveat) or replaced."
  },
  {
    "severity": "nit",
    "location": "Rollout item 2 — \"Same commit as the constant bump so code and spec text cannot drift (024-L3)\"",
    "problem": "Commit-level atomicity is required for constant-plus-specs, yet item 4 (discovery, including \"the eight re-pinned test files\", four of which live in `apps/discovery-cli/tests/`) is sequenced ahead of item 5 (the CLI's `Namespace` column and JSON shape), so no intermediate commit satisfies `cargo test --workspace --locked`.",
    "fix": "One clause: items 4 and 5 land as one commit, or state that the gates are branch-level and only items 1+2 are commit-atomic."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/024-game-slug-identity.md.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/024-game-slug-identity/reviews/design/correctness-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"Chosen design → Emission, refusal and collisions — \"they increment the existing `skipped` counter (whose message counts files, so a refused row counts there as a skipped row)\"","problem":"The count-only closure is accepted, but the new parenthetical bakes in a conflation that misreports in the one case the counter is ever user-visible: the counter's only surface is the steam_common.rs:101 error, whose message reads \"N file(s) skipped as unreadable or malformed\". When every game a plugin finds is refused for an un-derivable name — exactly the case ADR-024's Consequences foresee (\"a game whose display name and install directory are both non-Latin gets refused at discovery\") — the diagnostic fires and attributes readable, well-formed files to \"unreadable or malformed\", naming neither a game nor the actual cause.","fix":"In this bullet, split the fired message's counts (e.g. \"M file(s) skipped as unreadable or malformed; K game(s) refused\") or drop the \"unreadable or malformed\" attribution from the combined count."}]}
```
