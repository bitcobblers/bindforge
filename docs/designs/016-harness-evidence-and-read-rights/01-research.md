# Research: Harness evidence and read rights (ADR-016)

Scope: the caller's source is `docs/adrs/016-harness-evidence-and-read-rights.md`
(status `proposed`, 2026-11-13). It is a harness self-change in the line of
ADR-001/007/010 — no `crates/`, no `schemas/` — with four decisions: exempt a
review seat's own report from the read side of the independence audit; capture
the check gate's per-leg exit status and a bounded output tail as run evidence;
give the document families a read-only shell allowlist, both review panels the
project's check commands, and deny the run-record directories in the permission
layer; record per-stage token usage and budget document sizes (source:
docs/adrs/016-harness-evidence-and-read-rights.md). Lenses applied: **Prior
art** (under Findings/Prior art), **Security landscape** (under Prior art),
**Performance baselines** (under Platform notes).

## Summary

All four mechanisms the ADR describes are exactly as stated in the shipped
harness: the check gate spawns every leg with `stdio: 'ignore'` and persists
one boolean; the deeds audit's own-report exemption is keyed on the
`reviews/`-segment report identity but is wired only to writes; the document
families carry bare `shell → deny` with no `grep`/`glob`/`subagent` rules; and
transcripts print `parameters: {"note":"not exposed by endpoint"}` even though
the client reply already carries `tokens` and `cost` (source:
tools/orchestration/src/pipeline/gates.ts; source:
tools/orchestration/src/pipeline/independence.ts; source:
.opencode/agents/research/researcher.md; source:
tools/orchestration/src/pipeline/prompts.ts; source:
tools/orchestration/src/opencode-runner.ts). This very run's retry prompt —
"Previous attempt's gate failure … independence violation: grep accessed run
records", injected by the ADR-007 retry-feedback mechanism — corroborates the
ADR's cost claim and shows the permission layer did not stop the grep, only
the after-the-fact audit caught it (source: tools/orchestration/src/pipeline/
runner.ts; source: docs/adrs/007-audit-by-deeds.md). OpenCode's permission
layer already exposes every knob decision 3 names, and its own docs state the
ADR's caveat: shell resources are raw command text and a `grep` rule matches
the regex, not the search path — so run-record denies are tripwires, with the
deeds audit staying the backstop (source: https://opencode.ai/docs/
permissions/). Comparable harnesses hand bounded, machine-collected output
onward as the run's public record — GitHub Actions caps a step summary at 1
MiB for exactly this reason — and put the weight-bearing fences below text
matching, with OS-enforced sandboxes and write-denied configuration files
(source: https://docs.github.com/en/actions/using-workflows/
workflow-commands-for-github-actions; source:
https://code.claude.com/docs/en/sandboxing). The ADR's empirical motivating
numbers (prompt sizes, per-attempt minutes, review-finding counts) exist only
in the ADR text, and decision 4 is the change that would make them checkable
(source: docs/adrs/016-harness-evidence-and-read-rights.md).

## Findings

### Decision 1 — the read/write asymmetry as shipped

- The audit's read side fails a review-stage attempt on any *completed*
  `read`/`list`/`glob`/`grep` whose access fields match the `reviews` path
  segment (`.diff` excepted), and any stage's on the `transcripts` segment;
  there is no own-report exemption among the read rules (source:
  tools/orchestration/src/pipeline/independence.ts).
- The identity machinery decision 1 wants already exists and is write-only:
  `ownReportPath` is normalized by `reportKey` — "everything from the `reviews`
  segment on," so an absolute path and the repo-relative path the prompt
  handed the seat name the same report — and only `write`/`edit`/`patch`
  consult it (source: tools/orchestration/src/pipeline/independence.ts).
- "Anything broader than one report fails" falls out of the same key:
  `reportKey` returns undefined for paths that do not spell a concrete
  `reviews/<path>` report, and a completed glob is judged on its pattern
  because "the pattern is the access" — so `reviews/design/*` cannot
  identity-match a seat's report key (source:
  tools/orchestration/src/pipeline/independence.ts).
- Audit scope rules the exemption must not disturb: only completed tool parts
  count; `grep` is judged on its `path`/`include` fields, never its query
  pattern; payload text is never an access (`subagent` carries no access
  fields at all) (source: tools/orchestration/src/pipeline/independence.ts;
  source: docs/adrs/007-audit-by-deeds.md).
- The exemption's blast radius is automatic: the runner builds one
  `IndependenceAuditOptions` (with `ownReportPath` taken from the prompt var
  `reportPath`) and audited child sessions go through the same options object,
  so any read-side exemption added there propagates to every descendant of
  the seat (source: tools/orchestration/src/pipeline/runner.ts).
- The cost the ADR prices is mechanically real and observed in this run: a
  violation fails the attempt with the same one-retry-then-escalate budget,
  and attempt 2's prompt is amended with "Previous attempt's gate failure
  (log excerpt): … Do not repeat the behavior" — this researcher received
  exactly that amendment, over a `grep` that completed because no permission
  rule blocks `grep` on run-record paths (source:
  tools/orchestration/src/pipeline/independence.ts; source:
  tools/orchestration/src/pipeline/runner.ts; source: this stage's retry
  prompt; claim of cost per run: docs/adrs/016-harness-evidence-and-read-rights.md).

### Decision 2 — the gate's output is discarded today

- `testsGreen` returns a boolean only: `npm run check` in every probed package
  and each of the three `CARGO_GATE` legs (`fmt --all -- --check`,
  `clippy --workspace --all-targets -- -D warnings`,
  `test --workspace --locked`) spawn with `stdio: 'ignore'`, so the harness
  holds the streams and drops them (source:
  tools/orchestration/src/pipeline/gates.ts).
- The runner runs the gate after `implement`, `fix`, and `test-update` and
  persists only the boolean plus a two-word gate-log reason ("npm run check +
  cargo gate green/red") — no per-leg detail reaches the checkpoint (source:
  tools/orchestration/src/pipeline/runner.ts; source:
  tools/orchestration/src/pipeline/state-store.ts).
- The capture has an in-repo shape precedent: model-authored seat findings
  pass through `boundedFindings` with `MAX_FINDING_FIELD_CHARS = 1_000`
  before touching the checkpoint, because the seat board is cloned into every
  later checkpoint and re-injected into prompts (source:
  tools/orchestration/src/pipeline/runner.ts; source:
  tools/orchestration/src/pipeline/state-store.ts).
- The checkpoint grammar decision 2 must obey is stated in the schema header:
  every field added after run 1 is optional with a behavior-safe default
  (`findings`, `flow`, `planStop` are the shipped precedents), nothing is ever
  removed or retyped, and the loader sizes the file against a
  `MAX_CHECKPOINT_BYTES = 8 MiB` ceiling before parsing (source:
  tools/orchestration/src/pipeline/state-store.ts).
- The claim's artifact shape is a real, stable format: `cargo test` ends with
  a per-target summary line `test result: ok. N passed; 0 failed; …` — exactly
  the "`test result:` line" the ADR wants behind a behavior claim (source:
  https://doc.rust-lang.org/book/ch11-01-writing-tests.html; source:
  docs/adrs/016-harness-evidence-and-read-rights.md).
- Decision 2 adds no worktree file: the runner-reserved path set is the
  checkpoint, its save-tmp shape, the escalation marker, and the run lock —
  transcripts and reviews are not in it, and the capture lives in the
  checkpoint plus documents stages already own (source:
  tools/orchestration/src/pipeline/types.ts; source:
  docs/adrs/016-harness-evidence-and-read-rights.md).
- Demand is visible in the shipped prompts: `fix.md` hands the coder open
  findings and a findings-file list only — no gate output — and
  `code-review.md` states "Your inputs are exactly" diff, proposal, and ADR,
  so a seat has nothing to cite a green tail from (source:
  tools/orchestration/prompts/fix.md; source:
  tools/orchestration/prompts/code-review.md).

### Decision 3 — current reach per family, and the permission layer's knobs

- The document families' shell dimension today is a bare deny: researcher,
  designer, and planner carry `shell: * → deny` and nothing else; doc-sync
  additionally allows `git diff*`; all carry `read` denies over
  `docs/designs/**/transcripts/**` and none carries a `grep`, `glob`, or
  `subagent` rule (source: .opencode/agents/research/researcher.md; source:
  .opencode/agents/design/designer.md; source: .opencode/agents/design/planner.md;
  source: .opencode/agents/ops/doc-sync.md).
- The five design-review seats hold exactly two shell rules — `shell *` deny
  plus `git diff*` allow — and already read-deny `docs/designs/**/reviews/**`
  except the editorial seat's named `editorial-r*.diff` scope (source:
  .opencode/agents/review/design-security.md).
- The coder and the code-review seats already hold exactly the allowlist
  decision 3 extends: `cargo *`, `rustfmt *`, `npm run *`, `npm test *`,
  `npx vitest *`, the inspection set (`cd`, `ls`, `cat`, `head`, `tail`, `wc`,
  `grep`, `rg`, `find`, `git status/diff/log/show`), and trailing write-leg
  denies (`cargo fmt` denied while `cargo fmt *--check*` is re-allowed,
  `cargo fix`, `clippy --fix`, `npm run format`, `git commit/push/…`, `rm`,
  `mv`, publishes) ordered last — "the last matching rule wins" (source:
  .opencode/agents/code/coder.md; source:
  .opencode/agents/codereview/code-security.md).
- The shipped configs already depend on the scanner semantics decision 3
  leans on: "A compound command is scanned into subcommands and every leg
  must pass, so `cd <dir> && cargo test` needs this one" (source:
  .opencode/agents/code/coder.md).
- The wording decision 3 rewords is verbatim in the design-review prompt:
  "Your inputs are exactly … Do not open review reports or transcripts …
  Listing, globbing, or grepping those directories counts as opening them …
  never browse to verify" (source: tools/orchestration/prompts/design-review.md).
- The permission layer offers precisely the rule types decision 3 names —
  built-in actions include `glob` (resource = the requested glob pattern),
  `grep` (resource = the requested regular expression, *not* the search
  path), `shell` (resource = scanner-produced command text; compound commands
  produce several), and `subagent` (resource = target agent ID); rules are
  action/resource/effect with last-match-wins, agent rules appended after
  global ones, and any `deny` across an operation's resources denies it
  (source: https://opencode.ai/docs/permissions/).
- The ADR's "knowingly partial" caveat is the vendor's own posture: "Shell
  resources remain raw command text," the scanner refuses to run commands it
  cannot safely analyze, and the docs advise a narrow shell allowlist "instead
  of patterns intended to recognize every dangerous command"; a separate
  `policies` layer can hard-deny after rules and approvals and "never grants
  access," overriding project config (source: https://opencode.ai/docs/
  permissions/).
- The deferred delegation hole is structural in the same docs: every agent
  starts from an ordered base policy beginning `{action: *, resource: *,
  effect: allow}`, the shipped `general` subagent denies only questions and
  launching subagents, and "a custom subagent uses its own permissions, not a
  subset of its parent's" — so a shell-less family can reach any command
  through a child unless a `subagent` deny says otherwise (source:
  https://opencode.ai/docs/permissions/; claim that run 012's research stage
  did exactly this: docs/adrs/016-harness-evidence-and-read-rights.md).
- This run demonstrates the gap decision 3 closes: the researcher config
  denies `read` on `transcripts/**` but has no `grep` rule, so the base
  allow-all let a `grep` complete — and the segment-matching audit failed the
  attempt after the fact, which is precisely "isolation rests only on the
  after-the-fact audit" (source: .opencode/agents/research/researcher.md;
  source: https://opencode.ai/docs/permissions/; source:
  tools/orchestration/src/pipeline/independence.ts).

### Decision 4 — measurement gap, and what already exists

- `writeTranscript` prints `- parameters: <json>` defaulting to
  `{"note":"not exposed by endpoint"}`; a grep of `parameters` across
  `tools/orchestration/src` finds only the field declaration and that print
  site — no pipeline caller ever populates it (source:
  tools/orchestration/src/pipeline/prompts.ts).
- The data is already in hand one layer up: `runPrompt` returns a
  `PromptResult` whose reply carries `tokens?: TokenUsageInfo` and
  `cost?: number` from `@opencode/client ^2.1.4` — decision 4's transcript
  change needs no server or SDK change (source:
  tools/orchestration/src/opencode-runner.ts; source:
  tools/orchestration/package.json).
- A shape caveat for the designer: the poller takes the *first completed
  assistant message* of the session as the reply, so its `tokens` is that
  message's usage, not a per-session total across a tool loop (source:
  tools/orchestration/src/opencode-runner.ts).
- The "prompted in 113 bytes" claim verifies in order of magnitude:
  `prompts/research.md` is two lines naming no read set and no ceiling; the
  sent prompt is that template with vars substituted plus a generated
  reply-format JSON Schema appended by `runPrompt` (source:
  tools/orchestration/prompts/research.md; source:
  tools/orchestration/src/opencode-runner.ts; claim of 113 bytes:
  docs/adrs/016-harness-evidence-and-read-rights.md).
- Precedent for a flow-level document ceiling exists end to end:
  `planOutgrewFlow` gates a metered line count against `cfg.planMaxLines`
  (the light registry sets 80); absence is the full flow's "measure and warn
  without gating"; the runner already records `docLines` per stage result, so
  the meter decision 4's new ceiling reuses is running (source:
  tools/orchestration/src/pipeline/gates.ts; source:
  tools/orchestration/src/pipeline/flows.ts; source:
  tools/orchestration/src/pipeline/runner.ts; claim that the new ceiling is
  the sibling: docs/adrs/016-harness-evidence-and-read-rights.md).
- Refusing to raise `--max-parallel` is consistent with shipped operator
  guidance: `maxParallelReviews` defaults to 1, `--max-parallel` /
  `$OPENCODE_MAX_PARALLEL` resolve through one validator, and `docs/workflows.md`
  already warns that N "must be kept at or below the inference slots your
  server actually exposes: the runner cannot verify the slot count" (source:
  tools/orchestration/src/pipeline/cli.ts; source: docs/workflows.md; source:
  tools/orchestration/src/pipeline/stages.ts).
- The server facts the Context cites (`--parallel 1`, 262,144-token context,
  32,768 resident KV, 59/62 GiB RAM, 189,788-token largest prompt) appear
  only in the ADR; nothing in the repo spawns or configures the inference
  server. The named knobs do belong to the llama.cpp server family an
  OpenAI-compatible local endpoint implies: `-np/--parallel N` is the number
  of server slots, `-c/--ctx-size` the context, `--keep N` the prompt tokens
  kept, `--cache-prompt` is enabled by default, `--cache-reuse N` (default 0)
  enables prefix reuse via KV shifting, and `--slots`/`--metrics` are the
  monitoring endpoints — which supports the mechanism claim that concurrent
  long sessions against one slot contend for the cached prefix (source:
  https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md;
  claim: docs/adrs/016-harness-evidence-and-read-rights.md).

## Prior art

How comparable harnesses and CI systems handle the same three problems —
bounded machine evidence, read-only reach, and protecting the run's own
records:

| System | Bounded machine-collected evidence | Read-only reach class | Record/config protection |
|---|---|---|---|
| GitHub Actions | step/job summary shown on the run page "so someone viewing the result … doesn't need to go into the logs," hard-capped at 1 MiB per step; `::error file=…,line=…` annotations (source: https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions) | job/step-scoped logs | summaries and logs are runner-owned; a step cannot rewrite another step's summary (source: https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions) |
| OpenCode built-ins | n/a | the shipped `explore` agent "denies everything except shell, reads, globs, grep, web fetches, web searches, and external directories" (source: https://opencode.ai/docs/permissions/) | a `policies` layer that "turns allow or ask into deny and never grants access," overriding project config (source: https://opencode.ai/docs/permissions/) |
| Claude Code | sandbox names the denied host in the command's result so the model sees what was blocked (source: https://code.claude.com/docs/en/sandboxing) | read-only command classes and permission rules (source: https://code.claude.com/docs/en/sandboxing) | OS-enforced Bash sandbox (Seatbelt on macOS; bubblewrap + socat on Linux/WSL2) plus "protected paths": the agent's own settings/skills/hooks files stay write-denied *inside* the writable tree, because "a command that could edit those files could grant itself permissions" (source: https://code.claude.com/docs/en/sandboxing) |
| BindForge ADR-016 (proposed) | per-leg exit status + bounded output tail in the checkpoint, handed to seats, fix/implement, and the build note (source: docs/adrs/016-harness-evidence-and-read-rights.md) | read-only shell allowlist for doc families; check commands for review panels (source: docs/adrs/016-harness-evidence-and-read-rights.md) | run-record denies in the permission layer + deeds audit as backstop (source: docs/adrs/016-harness-evidence-and-read-rights.md) |

What wins by convention:

- **Bounded run summaries are the standard evidence artifact, with an explicit
  cap.** GitHub Actions gives every step a bounded summary channel (1 MiB)
  plus file/line-anchored annotations — the same "bounded tail in the record,
  not raw logs" posture decision 2 takes (source:
  https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions).
  The in-repo analogue for a cap-as-tested-constant is `boundedFindings` /
  `MAX_FINDING_FIELD_CHARS = 1_000` (source:
  tools/orchestration/src/pipeline/runner.ts).
- **A read-only reach class is a normal shipped thing.** OpenCode's `explore`
  agent is a standing deny-everything-except-reads shape; decision 3's
  allowlist is the same shape with the shell narrowed to named commands —
  and that narrower shell allowlist is what OpenCode's docs recommend over
  deny patterns (source: https://opencode.ai/docs/permissions/).
- **The fence that carries weight lives below the agent's pen.** Claude Code
  write-denies its own configuration files even inside the writable working
  directory, and enforces read/write boundaries at the OS level rather than
  by command-text rules; the BindForge audit header reached the same
  conclusion from its own runs — text matching over recorded command lines is
  defense-in-depth that obfuscation routes around (source:
  https://code.claude.com/docs/en/sandboxing; source:
  tools/orchestration/src/pipeline/independence.ts).

Security landscape — the failure modes these projects record for exactly the
boundaries ADR-016 is drawing:

- **Pattern denies over interpreters and workspace files escalate:** a Claude
  Code exclusion pattern covering an interpreter or a tool that acts on a
  workspace file lets the agent "write that file and then run it outside the
  sandbox," and with filesystem isolation off a sandboxed command "can write
  files that later commands run or read, such as shell startup files … and
  use them to widen its own access" (source:
  https://code.claude.com/docs/en/sandboxing).
- **Command-text matching misses indirect calls:** an excluded-command
  pattern does not match "a script or `make` target that calls `docker`
  internally"; the BindForge audit logs the same limit — a shell built by
  obfuscation reaches run records without spelling the segment (source:
  https://code.claude.com/docs/en/sandboxing; source:
  tools/orchestration/src/pipeline/independence.ts).
- **Delegation escapes per-agent policy:** a subagent "uses its own
  permissions, not a subset of its parent's" and starts from an allow-all
  base, which is how a shell-less family reaches any command — the hole
  ADR-016 defers with an explicitly named fix shape (`subagent` deny per
  family) (source: https://opencode.ai/docs/permissions/; source:
  docs/adrs/016-harness-evidence-and-read-rights.md).

## Platform notes

- The harness is Node/TypeScript (`engines.node >= 22.23.4`) driving a running
  OpenCode server through `@opencode/client ^2.1.4`; token usage and cost are
  already typed on the SDK reply, so decision 4's measurement is local to the
  harness (source: tools/orchestration/package.json; source:
  tools/orchestration/src/opencode-runner.ts).
- Capture is mechanical at four spawn sites: `npm run check` per probed
  package (root, one level down, `tools/`/`packages/`) plus the three legs,
  and the legs are already enumerated as data in `CARGO_GATE`, which keeps
  "per leg" bookkeeping a constant change (source:
  tools/orchestration/src/pipeline/gates.ts).
- The shell commands decision 3's doc allowlist names (`cd`, `ls`, `cat`,
  `head`, `tail`, `wc`, `grep`, `rg`, `find`, `git status/diff/log/show`) are
  byte-identical to the allowlist already shipped for coder and code-review
  seats, so the rule text exists to copy and the compound-command split-and-
  check-every-leg semantics are already depended on (source:
  .opencode/agents/code/coder.md).
- Performance baseline, single-slot economics: llama.cpp caches prompts by
  default (`--cache-prompt` enabled), reuses cached prefixes only at
  `--cache-reuse` ≥ chunk size (default 0 = off), pins `--keep N` prompt
  tokens, and sizes slots with `-np/--parallel`; against one slot, two long
  sessions evict each other's prefix rather than overlap — the mechanism
  behind the ADR's refusal to raise `--max-parallel` before prompts shrink
  (source: https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md;
  source: docs/adrs/016-harness-evidence-and-read-rights.md).
- The evidence block the gate capture would inject is itself per-seat context
  spend; the GitHub Actions 1 MiB per-step summary cap and the repo's own
  `MAX_FINDING_FIELD_CHARS = 1_000` are the two shipped budget conventions it
  will be priced against (source: https://docs.github.com/en/actions/
  using-workflows/workflow-commands-for-github-actions; source:
  tools/orchestration/src/pipeline/runner.ts).
- If a non-text fence for run records is ever wanted (explicitly out of this
  ADR's scope), the field's platform tools are Seatbelt on macOS and
  bubblewrap + socat on Linux/WSL2 (source:
  https://code.claude.com/docs/en/sandboxing).

## Risks & unknowns

- **Circular pricing between decisions 2 and 4.** The evidence block grows
  every review prompt — the exact quantity only decision 4 will measure — so
  until token recording ships, the block's cost is another operator estimate;
  the capture-budget test the ADR promises is load-bearing (source:
  docs/adrs/016-harness-evidence-and-read-rights.md).
- **The `grep` deny may not express what decision 3 asks for.** OpenCode's
  grep resource is "requested regular expression, not the search path," so a
  grep rule over run-record *directories* can only match patterns that
  literally spell the path; the researcher's completed grep this run confirms
  the current configs leave grep wholly unruled until the audit fires
  (source: https://opencode.ai/docs/permissions/; source:
  .opencode/agents/research/researcher.md).
- **Own-report exemption propagates to children automatically.** The child
  walk reuses one `IndependenceAuditOptions` object, so an exemption keyed on
  the seat's report identity extends to that seat's descendants — acceptable
  only if the designer intends a child to be able to read the parent's own
  report, which the ADR does not state (source:
  tools/orchestration/src/pipeline/runner.ts; source:
  docs/adrs/016-harness-evidence-and-read-rights.md).
- **Checkpoint growth is governed, not free.** New fields must be
  optional-with-default to keep 015-era checkpoints resumable, the seat-board
  clone-into-every-snapshot problem is why `boundedFindings` exists, and the
  loader refuses checkpoints over 8 MiB — a gate-evidence field inherits all
  three constraints (source: tools/orchestration/src/pipeline/state-store.ts).
- **The empirical baseline is operator-observed, not repo-checkable.** The
  counts that price the ADR (unverifiable-finding mentions across 011–013,
  fix rounds spent, minutes per burned attempt, the 189,788/262,144/32,768
  token figures, host RAM) appear only in the ADR text; the transcripts that
  would corroborate them are fenced from the researcher by its own permission
  config, so no run-record number can be re-derived in this research pass —
  decision 4's recorded numbers are the first checkable baseline (source:
  docs/adrs/016-harness-evidence-and-read-rights.md; source:
  .opencode/agents/research/researcher.md).
- **Coarse run-record denies can misfire on legitimate commands** that merely
  name the directories (the audit's own segment regex matches any command
  text spelling `transcripts` or `reviews`); the ADR accepts this because no
  stage prompt has a reason to name them, and the acceptance only holds while
  that stays true (source: tools/orchestration/src/pipeline/independence.ts;
  source: docs/adrs/016-harness-evidence-and-read-rights.md).
- **Widening the design panel's reach changes what its prompt must say.** The
  panel's prompt currently forbids browsing outright; decision 3 rewords it,
  and the audit's review-stage rule (`reviews` reads fail for review stages
  regardless of shell rights) must stay consistent with what the new wording
  permits (source: tools/orchestration/prompts/design-review.md; source:
  tools/orchestration/src/pipeline/independence.ts).
- **The deferred delegation hole undercuts decision 3 for one day.** Until a
  `subagent` deny per family ships, every family widened by decision 3 can
  still delegate around its own allowlist through an allow-all child (source:
  https://opencode.ai/docs/permissions/; source:
  docs/adrs/016-harness-evidence-and-read-rights.md).

## Open questions

- What is the capture's exact shape and budget: per-leg `{command, exit,
  tail}`? tail in bytes or lines, stdout+stderr merged or separate, taken
  from the end — and which named constant pins it, in the house pattern of
  `MAX_FINDING_FIELD_CHARS = 1_000` and `brief()`'s 140-char line cap (source:
  tools/orchestration/src/pipeline/runner.ts; source:
  tools/orchestration/src/pipeline/independence.ts)?
- Which checkpoint field carries the capture, and is it written beside the
  `testsGreen` boolean at the same drive-loop point so a red gate's record
  survives a crash exactly like the boolean does today (source:
  tools/orchestration/src/pipeline/runner.ts)?
- Which prompts grow an evidence block — `code-review.md`,
  `design-review.md`, `fix.md`, `implement.md` — and does each prompt's
  "your inputs are exactly …" enumeration name the block so seats are allowed
  to cite it (source: tools/orchestration/prompts/code-review.md; source:
  tools/orchestration/prompts/design-review.md)?
- What rules are actually writable for the run-record denies: a `glob` deny
  matches the pattern string and a `grep` rule matches the regex — given the
  grep resource is not the search path, is the grep-side deny only a
  plain-spelling tripwire, and is the non-granting `policies` overlay the
  right home for it (source: https://opencode.ai/docs/permissions/)?
- For decision 1: does the exemption live in `IndependenceAuditOptions`
  (auto-covering the child walk) or in the matcher's read branch, and do the
  review prompts gain an own-report carve-out sentence so a seat obeying the
  letter of its prompt may now re-read its own report (source:
  tools/orchestration/src/pipeline/runner.ts; source:
  tools/orchestration/prompts/design-review.md; source:
  docs/adrs/016-harness-evidence-and-read-rights.md)?
- For decision 4: is the recorded token quantity the final reply's `tokens`
  (what `PromptResult` exposes today) or a sum over the session's message
  list, and does the transcript add a `tokens` line beside `parameters` or
  fold usage into it (source: tools/orchestration/src/opencode-runner.ts;
  source: tools/orchestration/src/pipeline/prompts.ts)?
- Is the new document ceiling a second call beside `planOutgrewFlow` gated in
  both flows or measured like the full flow already measures (`docLines`
  today), and does the research prompt's named read set become machine-checked
  or stay a stated budget the deeds audit enforces (source:
  tools/orchestration/src/pipeline/gates.ts; source:
  tools/orchestration/src/pipeline/runner.ts)?
- Does any stage today depend on delegation (so a family-level `subagent`
  deny would break it), and what audit option shape would a child of an
  exempt seat inherit — deferred, but decision 3's real-world effect waits on
  it (source: https://opencode.ai/docs/permissions/; source:
  tools/orchestration/src/pipeline/runner.ts).
