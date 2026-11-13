# Transcript: code-review:security r1c0

## Provenance
- stage: code-review:security (round r1c0)
- agent: codereview/code-security
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ ba9304249d45
- started: 2026-11-13T13:41:48.150Z
- ended: 2026-11-13T13:44:23.404Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/013-game-manifests/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/013-game-manifests.md, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the security checklist.
Write your report to docs/designs/013-game-manifests/reviews/code/security-r1c0.md, and write no other file: an audit checks
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
{"verdict":"approve","findings":[{"severity":"minor","location":"crates/core/src/manifest.rs:360 (check_native_name control check) with crates/core/src/error.rs:155 (one_line)","problem":"The format's one free-text column can carry an unescaped line terminator into a refusal message and into every serialized copy of the manifest. check_native_name bans only ASCII controls (bytes().any(|b| b.is_ascii_control())), while one_line's escaper is category-based (c.is_control() = Cc), so the two guards do not meet: U+2028/U+2029 are neither ASCII control nor Cc, so such a name parses, is stored, is re-serialized raw by quoted (document.rs:875 has the same Cc blind spot), and is echoed raw by the escape-and-truncate path the design names as its own mitigation against hostile echoed names. error.rs:11-16 states the property being protected (\"one hostile upload cannot forge extra log lines\"), and JS/JSON-adjacent log pipelines and some shippers/terminals break on U+2028. This is a new surface: every earlier format's echoed strings are grammar-constrained ids (PROFILE_ENTRIES, ACTION_IDS_ENTRIES, GEOMETRY_ENTRIES carry no free text). It also undercuts the trailing-space rationale, since U+00A0/U+200B differ invisibly and pass.","fix":"Tighten the clause to the printable set: refuse name.chars().any(|c| c != ' ' && (c.is_control() || c.is_whitespace())) — that covers U+0085, U+2028, U+2029, NBSP — and refuse the Cf format range that reorders a reviewer's terminal (U+200B..=U+200F, U+2060..=U+2064, U+2066..=U+206F, U+FEFF); add fixtures/invalid/name-format-char.toml and mirror the one clause in schemas/manifests/v1/spec.md:79 so a third-party reader refuses the same documents."},{"severity":"minor","location":"crates/core/src/manifest.rs:586-627 (decode_base64, via check_fingerprint:559)","problem":"The decoder accepts non-canonical base64, contradicting its own doc comment at manifest.rs:582-585 (\"A body outside this never decodes through a permissive fallback, which would let two spellings claim one digest\") and the fingerprint-not-base64.toml fixture comment. In the final quartet the decoder takes significant from the trailing '=' and pushes only the high bits, so the unused low bits of the last data sextet are discarded silently: a 32-octet digest's 44-char body has 16 spellings decoding identically (AA== and AB== both yield 0x00). RFC 4648 §3.5 requires those bits be zero, and spec.md:116-117 does not say which side a reader takes, so the corpus pins neither and a reader written to RFC 4648 refuses documents this one accepts. Contained today (nothing consumes the digest in this run), but 017/018's use of the field is comparing it, where two spellings of one digest become byte-different fingerprints a string comparison calls a mismatch.","fix":"In the final quartet reject non-zero discarded bits (significant == 2 -> sextets[1] & 0x0f == 0; significant == 3 -> sextets[2] & 0x03 == 0), state \"canonical spelling per RFC 4648 §3.5\" in spec.md's Fingerprint grammar, and pin it with fixtures/invalid/fingerprint-non-canonical.toml plus a unit case beside tests/manifest.rs:894."},{"severity":"minor","location":"schemas/manifests/v1/spec.md:129 with crates/core/src/manifest.rs:221-273 (load_manifest)","problem":"The shipped language-neutral contract claims a guard the reference implementation does not have: \"The loader enforces monotonicity forward\". load_manifest applies exactly two cross-file rules (location, registry join) and has no prior state to compare a revision against; nothing in the diff reads history, and the proposal itself says core cannot pin a file's history (A10) and puts the bump rule in manifest-PR review plus ADR-014's deferred pin matrix. Because an out-of-repo implementer implements the spec verbatim, the sentence invites invented history-dependent behavior and lets a consumer treat revision as rollback/tamper evidence on the strength of a loader check that does not exist.","fix":"Reword to match the design: monotonicity is enforced in manifest review, and by the dependent-pin matrix once ADR-014 ships; load_manifest enforces only the location rule and the registry join."},{"severity":"nit","location":"crates/core/src/manifest.rs:559-579 (check_fingerprint) and manifest.rs:592 (decode_base64 allocation)","problem":"The fingerprint is the one entry-table string with no named size bound: a legal value is at most 51 characters, but nothing states or checks it, and the body is fully allocated and decoded (Vec::with_capacity(quartets * 3)) before the 32-octet length refusal, so a 1 MiB fingerprint element buys ~0.75 MB of allocation and a full scan before it is refused. Every other shape in this format has its cap named in the budgets table.","fix":"Reject a body whose length is not 44 before decoding (which also settles the canonical-shape question), or name the bound in spec.md's budgets table."}]}
```
