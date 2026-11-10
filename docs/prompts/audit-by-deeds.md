# Audit by deeds: make the independence matcher stop eating its own words

Three harness defects in `tools/orchestration`, each with receipts from run
005 (`docs/designs/005-runner-executed-file-moves/.pipeline.json` gate log,
readable in the `pipeline/005-runner-executed-file-moves` branch). Keep every
change inside `tools/orchestration`; existing tests must stay green.

## 1. The independence audit must judge acts, not words

`independence.ts` scans completed tool parts of the stage session. For tools
not in `ACCESS_FIELDS` — which includes `subagent` and `execute` — it falls
back to a full scan of every string in the tool input. Run 005 paid for that
three different ways:

- **implement (attempt 1, 977s lost):** the coder delegated shell probes to
  subagents and recited the rule in the delegation prompt ("NEVER read
  anything under any `transcripts` directory"). A prohibition reads identically
  to an instruction; the attempt failed on its own compliance reminder.
- **fix:** same flavor — a delegation prompt matched on quoted prohibition text.
- **test-update:** the engineer used `execute` to generate lines of test code,
  and forbidden segments inside the *string literals of the code it was
  writing* (tests about run records) matched. Content, not access.

The perverse feedback loop: prompt hardening makes agents quote the rule more,
which raises the false-positive rate of the audit.

Required change:

- **Delegation text is not an access.** For `subagent` (and any delegation-like
  tool), never treat the prompt text in the parent's tool input as evidence.
  Instead enumerate the attempt session's child sessions (the session schema
  carries `parentID`) created at or after the attempt's `since` stamp, fetch
  their message lists, and run THOSE through the same completed-parts matcher.
  The child's deeds are the violation. This does not weaken run 003's lesson —
  it strengthens it: today the parent's words are punished while the child's
  actual reads are invisible to the audit. Run 005's own security seat filed
  this as a design major ("no text names anything that enumerates the attempt's
  child sessions and feeds their completed tool parts into the matcher").
- **Content-carrying inputs are judged on paths only.** `execute` (and any
  tool whose input embeds program text) must be handled like `write` already
  is — scope note 1 of `independence.ts`: code ABOUT run records is not a read
  of run records. Extend that discipline to unknown tools generally: the scan
  judges where a tool looks, never what text it carries. If a hidden-path
  concern remains for unknown tools, resolve it by effect (what the tool
  touched, what children it spawned), not by string-scanning payloads.
- **Test pins, all three flavors:** (a) a `subagent` part whose delegation
  prompt recites the prohibition passes; (b) an `execute` part whose code
  contains `transcripts/` string literals passes; (c) a child session that
  actually reads a `transcripts/` file or globs `reviews/**` FAILS the parent
  attempt through its deeds — the new detection must be pinned, not just the
  new exemption.

## 2. A retry must be told why the first attempt failed

`runWithRetry` in `runner.ts`: on a fresh-session attempt 2 (the non-resumable
path), the stage gets the identical template prompt with nothing about what
attempt 1 did wrong. Recovery so far has been re-roll luck; the one
deterministic habit — run 005's editorial-security seat globbing
`reviews/design/*` to verify its handed path — failed twice in a row precisely
because attempt 2 knew nothing attempt 1 didn't.

Required change: the fresh attempt-2 prompt carries the attempt-1 failure
reason (the gate-failure text, already sanitized through `printable()`), and
for independence violations the rule that was broken. The retry budget and
escalation flow stay exactly as they are — only the prompt content changes, and
it must be pinned by a test asserting attempt 2's prompt contains the failure
reason.

## 3. Two prompt lines that run 005 proved necessary, this time with test pins

An operator edit of the first of these was reverted by the review panel on the
005 branch as an out-of-scope change with no test pin. Re-land both here, as
reviewed artifacts, each pinned like `tests/pipeline/fix-trigger.test.ts`
pins prompt templates (load from `prompts/`, assert on rendered content):

- `prompts/design-review.md`: after the existing "Do not open review reports
  or transcripts" sentence, state the bright line — listing, globbing, or
  grepping those directories counts as opening them; every path a seat needs
  is named in the prompt, so never browse to verify one, even when a named
  path lives under a directory the seat must not open. Run 005 evidence: the
  compatibility ratifier globbed `reviews/**/*` and the editorial-security
  seat globbed `reviews/design/*` — both failed their attempts.
- `prompts/test-authoring.md` and `prompts/test-update.md`: the test engineer
  owns the test files it writes, and must run the package formatter
  (`npx prettier --write` on the files it touched) BEFORE replying. Run 005
  escalated at "tests not green at the code-review budget limit" with 5/5 fix
  rounds spent and lint/tests otherwise fine: the only red check was prettier
  drift in two test files, ping-ponging across three test rounds because every
  new test batch reintroduced it.

## Notes for the stages of this very run

This run executes under the CURRENT (buggy) matcher. Stage agents delegating
shell probes should name the allowed paths and commands directly in the
delegation; do not copy prohibition sentences from stage prompts into
subagent prompts verbatim.
