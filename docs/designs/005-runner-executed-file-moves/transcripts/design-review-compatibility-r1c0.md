# Transcript: design-review:compatibility r1c0

## Provenance
- stage: design-review:compatibility (round r1c0)
- agent: review/design-compatibility
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 8d0282dcf042
- started: 2026-11-10T17:02:00.465Z
- ended: 2026-11-10T17:05:53.966Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/005-runner-executed-file-moves/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/005-runner-executed-file-moves.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/005-runner-executed-file-moves/reviews/design/compatibility-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"Compatibility matrix — \"| macOS (APFS, case-insensitive default) | unknown | suite expected to pass; case-only moves rejected by the disk check — unverified (A10) |\"","problem":"The row carries no decision: `unknown` is not a status an implementer or operator can act on, and the Notes cell hedges twice (\"expected to pass\", \"unverified\"). The compatibility checklist requires a status on every platform row.","fix":"Replace `unknown` with a committed status and back it with a named check: either \"yes (unverified)\" with Test strategy #12 made concrete for macOS (run one move round trip plus one case-only move and record the observed rejection as the A10 evidence), or \"not exercised\" stating the runner is not run there."},{"severity":"major","location":"Compatibility matrix — \"| Windows (NTFS, `core.protectNTFS`) | unknown | never exercised in this repo's pipeline; name-legality rules untested |\"","problem":"Same missing decision, and the Notes cell names a gap (\"name-legality rules untested\") that no validator rule and no Test strategy case addresses, so the row cannot resolve itself during implementation.","fix":"Pick the status as for macOS and make Test strategy #12 name the two Windows observations this design depends on — a move round trip, and the behavior of a `\\`-separated or Windows-illegal `to`. If no one will run the pipeline on Windows, say so in the Notes cell instead of leaving `unknown`."},{"severity":"minor","location":"Chosen design / Validator rule 1 — \"repo-relative POSIX paths, normalized, no absolute path, no `.` or `..` segments\"","problem":"\"normalized\" is undefined while every downstream check assumes `/`: rule 4 builds the revspec `<headBefore>:<from>`, the new `.git` rule is the string test \"path === '.git' || path.startsWith('.git/'\", and `stagePathAllowed` is regex-on-`/` (gates.ts:253,260). A reply emitting `src\\old.ts` clears rule 1 (no `/`-separated `..`, no absolute marker) and then fails rule 4 with a confusing \"not tracked at headBefore\"; a `.git\\config` endpoint escapes the `.git` string test and is caught only accidentally. The repo already normalizes the other way (`relative(repoDir, doc).split(/[\\/]/).join('/')`, runner.ts:134).","fix":"Define normalization in one rule-1 clause: reject any `\\` in either endpoint and compare with `path.posix` segment semantics; add one rejection case to Test strategy #2."},{"severity":"minor","location":"Chosen design / Validator rule 1 — \"Shape: repo-relative POSIX paths, normalized, no absolute path\"","problem":"The shape list covers `.`/`..`/absolute but no per-OS name legality, while the Windows row itself says \"name-legality rules untested\". A Windows-reserved component (`CON`, `NUL`, trailing dot/space) validates and commits cleanly on the Linux reference platform; a `:` in a path additionally corrupts the `<headBefore>:<from>` revspec in rule 4, so it fails as an apparent git error rather than a validator message naming the pair.","fix":"Extend the same rule-1 sentence with a reserved/illegal-name check on each segment (one regex) plus one Test strategy #2 case — or state in the Windows Notes cell that name legality is operator-checked; today the text says untested without deciding which."},{"severity":"minor","location":"Chosen design / Validator rule 6 — \"all `from`s distinct, all `to`s distinct\" … \"so the list is a matching on disjoint paths and apply order is irrelevant\"","problem":"Distinctness is string-distinct, which is not disjointness on a case-insensitive checkout — the platform class the macOS/Windows rows name. Two `to`s differing only in case pass rules 5 and 6, then the second pair's per-pair \"re-check `to` absent\" trips inside `applyMoves`: the list rolls back as `moves-apply-failed` (environmental class, fresh session) instead of the cheap in-attempt nudge, and the \"apply order is irrelevant\" claim is false as written.","fix":"In rule 6, compare endpoint sets case-folded when the checkout is case-insensitive (reusing the `existsSync` probe rule 5 already uses), or state the unconditional rule that case-insensitively colliding endpoints are rejected everywhere."},{"severity":"nit","location":"Compatibility matrix — \"| Repository spanning mount points | partial | `rename(2)` fails `EXDEV` (A8): the pair fails at apply, the list rolls back, the stage fails with `moves-apply-failed` |\"","problem":"The row describes a specific path through `applyMoves` that no test names; Test strategy #5 and #11 exercise a generic mid-list failure, not the cross-mount case, so the \"partial\" status has no observation behind it.","fix":"Name it in #5: a pair whose `to` parent sits on a different mount (e.g. a tmpfs under the tmpRepo worktree)."},{"severity":"nit","location":"Compatibility matrix — \"| Node ≥ 22.12 (tools/orchestration/package.json:30-32) | yes | no new runtime dependency: `fs.renameSync` plus the existing git helper |\"","problem":"The matrix pins the runtime floor but not the git binary floor the design newly leans on: `git status --no-optional-locks*` needs git ≥ 2.19, and rename detection defaults and `core.ignoreCase` behavior are version-dependent.","fix":"Add one matrix row \"git ≥ <floor>\", or one clause in the Node row naming the assumed git floor."}]}
```
