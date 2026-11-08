# Transcript: design-review:correctness:attempt2 r2

## Provenance
- stage: design-review:correctness:attempt2 (round 2)
- agent: review/design-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/design-review.md @ 143f0d087e11
- started: 2026-11-08T05:33:12.071Z
- ended: 2026-11-08T05:35:19.112Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/002-add-timeout-flag/02-proposal.md.
Apply only this change set: the whole document.
Load the `design-review` skill and apply the correctness checklist.
Write your report to docs/designs/002-add-timeout-flag/reviews/design/correctness-r2.md.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"Assumption register A6 (+ Options considered → Value semantics)","problem":"A6 is marked verified for 'plain integer seconds' but its first cited basis contradicts the integer part: the curl man page's `-m, --max-time <seconds>` section says 'This option accepts decimal values' and shows `curl --max-time 2.92`. Only libpq connect_timeout ('decimal integer') and CURLOPT_TIMEOUT (long seconds) back the integer claim, and the fraction-rejection rationale names coreutils as the sole floating-point outlier when curl is a second one. The decision still stands on libpq plus 250 ms poll granularity.","fix":"Reword A6 to 'seconds-valued human-facing interfaces (integer-typed in libpq connect_timeout and CURLOPT_TIMEOUT; --max-time is seconds-valued but accepts decimals)' and acknowledge curl as a second decimal-valued outlier in the Options rationale."},{"severity":"minor","location":"Assumption register A10, basis","problem":"The anchor `tools/orchestration/src/pipeline/runner.ts:326` does not land on the claimed pattern: line 326 is the `title:` line of the session.create call; the `start + timeout` wall-clock deadline is at runner.ts:338 (`const deadline = startedAt + cfg.stageTimeoutMs`, tested at :378). The claim itself is true and the stages.ts:31,39 anchors are exact.","fix":"Cite runner.ts:338 (and :378) instead of :326."},{"severity":"minor","location":"Chosen design → src/cli.ts, resolver bullet 1 (+ Test strategy #8)","problem":"The set-but-empty env case (OPENCODE_REPLY_TIMEOUT_SECONDS=\"\", common in container images) is decided only implicitly: process.env yields \"\", which is !== undefined and fails the ^\\d{1,5}$ gate, so the run hard-fails even with a valid --timeout. Consistent with the design's fail-closed stance, but nowhere does the spec or the Test #8 matrix state empty-vs-unset — exactly the 'empty input' edge case the checklist names.","fix":"State the empty-string case explicitly in the resolver bullet and add `\"\"` to Test #8's rejection matrix."},{"severity":"nit","location":"Chosen design → src/cli.ts, resolver bullet 1 (+ Data flow and input trust)","problem":"The strictness rationale for the format gate (Number() is looser than integer seconds: \"0x2d\" → 45, \"1e4\" → 10000) is argued only for env; the flag path has the identical silent coercion (yargs type:'number' turns `--timeout 0x2d` into 45, which passes the range check) and the resolver's number signature makes a format gate impossible there. The stated rationale reads as if only env is exposed.","fix":"Add one sentence recording the flag path's known coercion as an accepted limitation, or thread the raw argv token into the resolver for a symmetric gate."},{"severity":"nit","location":"Test strategy #5","problem":"'real timers, since the call must reject synchronously' is wrong in kind: runPrompt is an async function, so an invalid timeoutMs produces a rejected promise, never a synchronous throw, and the immediate rejection follows from validation preceding any timer scheduling — fake vs real timers is irrelevant.","fix":"Drop the parenthetical or say 'no fake timers needed: the promise rejects before any timer is scheduled'."},{"severity":"nit","location":"Revision log, r3 entry (02-proposal.md:160)","problem":"The r3 bullet contains the 'Deliberately not changed this round' paragraph twice verbatim in the same line (grep confirms the duplicate pair inside line 160); only the second copy is followed by the register-status sentence. Log hygiene only — no design content is contradicted.","fix":"Delete the duplicated paragraph."}]}
```
