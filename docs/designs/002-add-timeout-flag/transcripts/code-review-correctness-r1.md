# Transcript: code-review:correctness r1

## Provenance
- stage: code-review:correctness (round 1)
- agent: codereview/code-correctness
- model: strata/qwen3.8-flash-next-iq3_s
- template: tools/orchestration/prompts/code-review.md @ 0710f55a4e99
- started: 2026-11-08T06:09:48.932Z
- ended: 2026-11-08T06:12:46.373Z
- parameters: {"note":"not exposed by endpoint"}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/002-add-timeout-flag/02-proposal.md. Diff range: git diff fdfd2bc280482e9a878429f197619d5e794a075d...HEAD.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/002-add-timeout-flag/reviews/code/correctness-r1.md.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{
  "verdict": "request-changes",
  "findings": [
    {
      "severity": "major",
      "location": "docs/designs/002-add-timeout-flag/03-build-notes.md:52-60 (Test strategy #12 / Rollout #1)",
      "problem": "The design's only acceptance gate, `npm run check` (lint + format check + tests) in tools/orchestration, was never executed — deviation 1 records it could not run, and nothing on the branch shows it ran afterwards. Both new suites (tests/cli.test.ts, tests/opencode-runner-timeout.test.ts) were authored before the implementation existed, so Goal 4 ('existing tests pass unchanged') and the merge gate are unevidenced; this review's environment also refused to run them, so the item could not be evaluated (which counts as a finding, not a pass). My static trace found no source/test mismatch, so this is unverified rather than known-broken.",
      "fix": "Run `npm run check` in tools/orchestration (or at minimum `npx vitest run` plus `npm run typecheck` and `npm run lint`), fix any fallout, and record the run summary (test counts, lint/format status, Node version) in 03-build-notes.md before merge."
    },
    {
      "severity": "minor",
      "location": "tools/orchestration/src/cli.ts:125-134",
      "problem": "The env range check sits behind the flag branch, so `--timeout 5` with `OPENCODE_REPLY_TIMEOUT_SECONDS=3601` silently resolves to 5 and the broken environment is never reported. That matches the letter of the proposal (only the format gate is pinned as precedence-independent) and is declared as build-notes deviation 2, but it contradicts the design's own rationale next to it ('a broken environment is a configuration error even when a valid flag overrides it', proposal:82) and the Risks row promising an env typo hard-fails the run.",
      "fix": "Validate the env value fully (format gate and `1..maxSeconds` range) before the `flagValue !== undefined` branch, then apply precedence; or keep the current behaviour and pin it with a test such as `expect(resolveReplyTimeoutSeconds(5, '3601')).toBe(5)`."
    },
    {
      "severity": "minor",
      "location": "docs/designs/002-add-timeout-flag/03-build-notes.md:46-48",
      "problem": "The notes' coverage claim ('No test files were edited; the three existing suites … were treated as the contract') does not describe the whole reviewed diff range, which also contains src/pipeline/gates.ts, src/pipeline/prompts.ts, src/pipeline/runner.ts, tests/pipeline-gates.test.ts and tests/pipeline-prompts.test.ts from commits f0a2e16 and 891567b — a subsystem the proposal lists as an explicit non-goal (proposal:17). Provenance indicates operator infrastructure rather than this run's output (both commits are the oldest in the range, authored right after the base pin fdfd2bc and before the run's first docs commit), so this is range hygiene rather than scope creep.",
      "fix": "Add one build-notes line recording the two out-of-scope pipeline commits, or re-pin the review base to 891567b so the review diff contains only the design's change set. If those commits turn out to be the run's own output, escalate this to a major undeclared scope-creep finding."
    },
    {
      "severity": "nit",
      "location": "tools/orchestration/src/cli.ts:110-111",
      "problem": "The env format-gate error message states the accepted range ('must be an integer number of seconds in 1..3600 (plain digits only)') even though the failure it reports is a format violation; the proposal deliberately keeps format rejection separate from range rejection, and a user who sets `OPENCODE_REPLY_TIMEOUT_SECONDS=30s` is told about a range they never violated.",
      "fix": "Word the format error as format-only, e.g. `$OPENCODE_REPLY_TIMEOUT_SECONDS must be plain decimal digits (integer seconds), got …`, leaving the range wording to the range rejection at cli.ts:129-130."
    },
    {
      "severity": "nit",
      "location": "tools/orchestration/src/cli.ts:182,188",
      "problem": "`configFromEnv()` is called twice per run — once inside `parseArgs()` and again for `replyTimeoutSeconds` — so the resolver validates a separate read of the environment from the bag yargs used for the other option defaults, which is slightly at odds with the design's 'env reads centralized in configFromEnv()' intent (A3).",
      "fix": "Have `parseArgs(argv)` accept or return the `EnvValues` bag and pass that same snapshot into `resolveReplyTimeoutSeconds`."
    }
  ]
}
```
