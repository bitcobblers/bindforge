# Transcript: code-review:compatibility r1c1

## Provenance
- stage: code-review:compatibility (round r1c1)
- agent: codereview/code-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T12:46:14.885Z
- ended: 2026-11-13T12:48:05.603Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/012-input-vocabulary-and-reach-classes/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/012-input-vocabulary-and-reach-classes.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "crates/core/src/document.rs:186-191 (gate fed by supported_majors() at document.rs:100); profile and action-ids arms at document.rs:194-208; parse_profile document.rs:379; registry_from_entries crates/core/src/action.rs:179",
    "problem": "The version gate is format-blind, and only the new keyboard-geometry arm checks its own majors (parse_keyboard_geometry, document.rs:531-536). Adding MAJOR_V2 to the crate-wide supported list therefore widens two formats the diff declares untouched: a document declaring format = \"profile\" (or \"action-ids\") and version = \"v2\" now passes the gate, passes reject_unknown_entry against the v1 entry list, and parses with v1 semantics, where before this diff it was refused as UnsupportedVersion. That is the fail-open tests/version_gate.rs:8-12 says the gate exists to prevent, it contradicts ADR-012's \"extending them means a new major, never widening v1\", and it is unpinned (version_gate.rs:45 only tests v99, and no fixture declares a major its directory does not carry). Every later format major in the 012-015 series inherits the hole.",
    "fix": "Make the gate per-format in one place: consult a format_majors(format) table (profile/action-ids/keyboard-geometry -> [\"v1\"], reach-classes -> [\"v1\",\"v2\"]) right after the format lookup and refuse with UnsupportedVersion naming that format's majors, or mirror the geometry arm's version guard in the other two arms. Then pin it in the contract and in a test: a profile document declaring v2 is refused as UnsupportedVersion."
  },
  {
    "severity": "minor",
    "location": "schemas/reach-classes/v2/spec.md:24 and fixtures/invalid/ (only uppercase-member.toml covers case); crates/core/src/input.rs:170-181; crates/core/tests/input.rs:110-144",
    "problem": "The ASCII-only half of the id grammar exists only in Rust. The spec says \"lowercase\", which a non-Rust implementer reads as Unicode-lowercase, so a reader could accept keyboard:é while bindforge-core refuses it; the corpus that the contract calls the conformance currency never exercises the case, and no unit case does either. This is the format whose grammar every later format reuses, so the divergence is the non-US-keyboard locale risk made concrete.",
    "fix": "Add schemas/reach-classes/v2/fixtures/invalid/non-ascii-member.toml (one fault, keys = [\"keyboard:é home\"]), a matching case in tests/input.rs::rejects_an_id_that_breaks_the_namespace_or_member_grammar, and the word ASCII in the spec line: \"a lowercase ASCII glyph of the ANSI-US set\" — the precedent is schemas/action-ids/v1/fixtures/invalid/non-ascii-id.toml."
  },
  {
    "severity": "minor",
    "location": "schemas/reach-classes/v2/spec.md:150-154 (stand-in-directory parenthetical) against crates/core/tests/conformance.rs:121-130",
    "problem": "The spec still tells implementers that \"a stand-in directory is not named for a major, so the directory/version rule above does not apply to it\", but this diff changed load_corpus_table to write the stand-in pair as {major}/{other}.toml and load from scratch.path().join(major) precisely because check_dir_version (document.rs:623) does apply. The contract text a third-party implementer mirrors now describes the layout the reference harness no longer has.",
    "fix": "Reword the parenthetical to the shipped behavior: the stand-in directory carries the fixture's own major segment, and the table completing the pair is taken from the same major directory, so the version/directory rule holds there too."
  },
  {
    "severity": "nit",
    "location": "crates/core/src/document.rs:645 (new is_major_dir) beside crates/core/src/action.rs:229-238; lenient use at document.rs:623-644",
    "problem": "The \"directory is named for a major\" rule is now defined twice, in two private helpers, with different strictness: a non-major directory is a refusal for the registry and a pass-through for reach tables and geometries. The divergence is deliberate (harness stand-ins), but nothing but two copies of the predicate carries it, so the version-gate rule can drift between loaders.",
    "fix": "Share one pub(crate) fn major_dir_segment(dir: &Path) -> Option<&str> in document, used by load_action_registry, load_reach_tables and load_geometry, with each loader deciding what None means."
  },
  {
    "severity": "nit",
    "location": "crates/core/tests/zz_api_probe.rs (new, 107 lines) against 02-proposal.md:127-146",
    "problem": "The delta inventory names exactly three new test targets (tests/input.rs, tests/placement.rs, tests/reach_v2.rs); the diff ships a fourth. The target is valuable to this seat — it resolves every name ADR-015 will compile against at the crate root, which is where a rename would break a consumer not yet written — but it is outside the inventory the build was measured against and outside ADR-004's module mirroring.",
    "fix": "Record it in the delta inventory / Rollout 3 via the doc-sync leg, or fold the probe into tests/placement.rs."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/012-input-vocabulary-and-reach-classes/reviews/code/compatibility-r1c1.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"nit","location":"schemas/README.md:27-34 against crates/core/src/document.rs:232-237 (union gate binds before check_format_major) and crates/core/tests/action.rs:16-22","problem":"The new per-format sentences state that a document declaring a major its format does not ship is refused 'naming that format's majors', but the crate-wide union gate still binds first, so a profile or action-ids document declaring `v99` is refused naming [\"v1\", \"v2\"] — including a major that format never ships — while the same formats declaring `v2` name [\"v1\"]. The module doc (document.rs:11-19) honestly states this layering and the README's first bullet covers it, and the direction is safe (a superset message is not fail-open, and no fixture pins exact message text), but an implementer mirroring only the narrowed sentence would emit a different refusal message than the reference implementation for majors no format ships.","fix":"State the layering where the per-format rule is stated: in schemas/README.md add 'a major no format ships at all is refused naming the majors the crate declares' (the sentence document.rs's module doc already carries), and soften the action.rs test header's 'rather than the crate-wide union' claim the same way — or run check_format_major before the union gate so every version refusal for a known format names exactly that format's majors."}]}
```
