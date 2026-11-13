# Design: Harness evidence and read rights (ADR-016)

## Context

ADR-016 (`docs/adrs/016-harness-evidence-and-read-rights.md`) stops the runner
from withholding facts it already holds, in four moves, each a harness
self-change in the line of ADR-001/007/010: (1) the independence audit's read
side exempts a review seat's own report, exactly as the write side already
does; (2) the check gate captures per-leg exit status and a bounded output
tail as run evidence instead of discarding the streams; (3) the document
families get a read-only shell allowlist and both review panels get the
project's check commands, and the run-record directories are denied in the
permission layer as well as by the audit; (4) per-stage token usage is recorded in the
transcript and document size becomes a budgeted flow config. This design
implements the ADR's Decision clauses 1–4 and nothing else — the exclusions
are the Non-goals.

## Goals / Non-goals

- Goals:
  - A review seat's re-reading of the report the prompt handed it is not a failed
    attempt; every other `reviews/` access still fails it (Decision 1).
  - One canonical gate record — per-leg command, exit, bounded tail —
    persisted in the checkpoint and rendered into the `code-review`, `fix`,
    `fix-tests`, and `implement` prompts and the build note (Decision 2).
  - Read-only shell reach for researcher, designer, planner, editor,
    doc-sync; check commands with write legs denied for the design panel;
    run-record shell/glob/grep denies in every widened family (Decision 3).
  - A `- tokens:` line per stage transcript, a named read set and stated read
    ceiling in the research prompt, and a `docMaxLines` ceiling for
    `01-research.md` and `02-proposal.md` (Decision 4).
- Non-goals:
  - No change to `--max-parallel`, `maxParallelReviews`, or the served slot
    count / KV layout — Decision 4 refuses this on the ADR's measured facts.
  - No `subagent` deny per family: the delegation hole is ADR-016's
    "Not decided here", not this design's.
  - No product (`crates/`, `schemas/`) change.
  - No `pipeline status` / `report.ts` display of tokens or the gate record.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Today the audit's read side fails a review-stage attempt on any completed `read`/`list`/`glob`/`grep` whose access fields carry a `reviews` path segment (`.diff` excepted) and any stage's on `transcripts`; shell is judged on command text; there is no own-report read exemption | verified | tools/orchestration/src/pipeline/independence.ts:152–203 |
| A2 | The write side already exempts the handed report by identity: `reportKey` keys "everything from the `reviews` segment on", so absolute and repo-relative spellings name one report; it is consulted only by `write`/`edit`/`patch` | verified | tools/orchestration/src/pipeline/independence.ts:191–214 |
| A3 | The runner builds one `IndependenceAuditOptions` per attempt (with `ownReportPath` from prompt var `reportPath`) and the child-session walk audits descendants through the same object, so any exemption propagates to the seat's descendants automatically | verified | tools/orchestration/src/pipeline/runner.ts:1494–1519 |
| A4 | Only completed tool parts are audited; `grep` is judged on `path`/`include`, never its query; `subagent` carries no access fields; a permission-denied call never completes and is not a violation | verified | tools/orchestration/src/pipeline/independence.ts:84–99,176–179 |
| A5 | `testsGreen` spawns every leg (`npm run check` per probed package, the three `CARGO_GATE` legs) with `stdio: 'ignore'`; only the boolean plus a two-word gate-log reason reach the checkpoint | verified | tools/orchestration/src/pipeline/gates.ts:279–316; tools/orchestration/src/pipeline/runner.ts:2831–2839 |
| A6 | Checkpoint grammar: every field added after run 1 is optional with a behavior-safe default, nothing is removed or retyped, and the loader sizes the file against `MAX_CHECKPOINT_BYTES = 8 MiB` before parsing | verified | tools/orchestration/src/pipeline/state-store.ts:59–140,150 |
| A7 | The house bounding pattern is `MAX_RECORDED_FINDINGS = 40` / `MAX_FINDING_FIELD_CHARS = 1_000` applied where records are written and again where rendered | verified | tools/orchestration/src/pipeline/runner.ts:2949–2979,3034 |
| A8 | The coder and the five code-review seats already hold the Decision 3 allowlist (`cargo *`, `rustfmt *`, `npm run *`, `npm test *`, `npx vitest *`, the inspection set, trailing write-leg/commit/publish denies, ordered last, last-match-wins) | verified | .opencode/agents/code/coder.md; .opencode/agents/codereview/code-security.md:23–151 |
| A9 | The document families carry a bare `shell * → deny` (doc-sync adds `git diff*`) and no `grep`/`glob`/`subagent` rules; their read denies cover `transcripts/**` only, leaving `reviews/**` readable — by design, mutating stages must read the reports handed them | verified | .opencode/agents/research/researcher.md; design/designer.md; design/planner.md; ops/doc-sync.md; independence.ts:61 |
| A10 | Design-review seats hold exactly `shell *` deny plus `git diff*` allow, and read-deny `docs/designs/**/reviews/**` except the editorial `editorial-r*.diff` scope | verified | .opencode/agents/review/design-security.md:17–31 |
| A11 | Seat report grammar is `<designDir>/reviews/<phase>/<specialty>-rN[cM].md`, editorial reports under `reviews/design/`; each seat is a distinct agent file naming its specialty | verified | tools/orchestration/src/pipeline/panel.ts:186–198 |
| A12 | The SDK's completed assistant message already carries `tokens` and `cost`; `runPrompt` surfaces them, and `executeInference` holds the same message lists | verified | tools/orchestration/src/opencode-runner.ts:167–177,278–287; runner.ts:1441–1457 |
| A13 | `TokenUsageInfo`'s exact field names are as `@opencode/client ^2.1.4` types them; this design sums its numeric fields per key without naming them | unverified | tools/orchestration/src/opencode-runner.ts:9 (type import only; fields not read in this repo) |
| A14 | `writeTranscript` prints `- parameters:` defaulting to `{"note":"not exposed by endpoint"}`; no pipeline caller ever populates it | verified | tools/orchestration/src/pipeline/prompts.ts:96,134 |
| A15 | A flow-level document ceiling already runs end to end: `planOutgrewFlow` gates the `docLines` meter against `cfg.planMaxLines` (light sets 80); its absence is the full flow's measure-and-warn posture | verified | tools/orchestration/src/pipeline/gates.ts:135–144; flows.ts:82–90; runner.ts:1701–1735,2775–2810 |
| A16 | `tests/pipeline/agent-shell.test.ts` pins stage shell rights against a repo-local re-implementation of OpenCode's wildcard matching (`*` spans `/`; a rule ending ` *` also matches the bare command; last match wins) — the grammar A17's permissions page documents; §3's deny-then-reallow order holds only under that grammar, so the fixture stays pinned to it | verified | tools/orchestration/tests/pipeline/agent-shell.test.ts:74–102; https://opencode.ai/docs/permissions/ |
| A17 | OpenCode's permission layer (V2 page): `grep` rules match the requested regex, not the search path; `shell` resources are scanner-produced raw command text with compound commands scanned leg by leg; `glob` resources are the requested pattern; `read` resources are Location-relative or canonical absolute paths; path matching normalizes `\` to `/` and is case-insensitive on Windows | verified | https://opencode.ai/docs/permissions/ (V2), read 2026-11-13 |
| A18 | `cargo test` ends each target with a `test result: ok. N passed; …` summary line, so a tail from the end of its output carries the artifact behind a behavior claim | verified | https://doc.rust-lang.org/book/ch11-01-writing-tests.html |
| A19 | The ADR's empirical run numbers (189,788-token prompt, 9/11/3 unverifiable-finding mentions, minutes per burned attempt, fix rounds spent) are operator-observed and exist only in the ADR text | unverified | docs/adrs/016-harness-evidence-and-read-rights.md; docs/designs/016-harness-evidence-and-read-rights/01-research.md § Risks |
| A20 | Server facts (`--parallel 1`, 262,144-token context, 32,768 resident KV, 59/62 GiB RAM) are the operator's host state; nothing in this repo spawns or configures the inference server | unverified | docs/adrs/016-harness-evidence-and-read-rights.md; 01-research.md § Decision 4 |
| A21 | The own-report reads that burned attempts in 011–013 reached the audit through the seat's `git diff*` shell allow (their `read`-tool route is permission-denied and would never complete) — the config shows the route exists; the run records are fenced from this design | unverified | .opencode/agents/review/design-security.md:20–31; A1/A4 above |
| A22 | There is no ADR index file; the `docs/adrs/` directory listing is the index the research prompt can name | verified | docs/adrs/ (16 ADRs + template, no README/index) |
| A23 | A coarse shell deny naming `*transcripts*` / `*reviews*` denies any command spelling the word, and can deny a command that merely names it — the ADR accepts this because no stage prompt has a reason to name those directories | unverified | docs/adrs/016-harness-evidence-and-read-rights.md § Consequences (acceptance holds only while no stage needs to name them; Decision 1's own-report carve is the one named exception, closed by A11's per-seat allow below) |
| A24 | Shipped full-flow document sizes the ceiling must sit above: 014's `02-proposal.md` is the largest at 733 lines (015: 728, 012: 599, 013: 557, 011: 479); this run's `01-research.md` is 440 | verified | line totals of `docs/designs/{011,012,013,014,015}/02-proposal.md` and `docs/designs/016-*/01-research.md` as read 2026-11-13 |

## Options considered

- **Exempt the own report in `IndependenceAuditOptions` (new flag) vs in the
  matcher's read branch.** Rejected (flag): the options object already carries
  `ownReportPath` and its `reportKey` identity (A2); a matcher-side branch
  reusing that identity changes one function and keeps the options shape, so
  the child-session walk needs no fork. A flag would be a second authority for
  one fact.
- **Block the exemption from the child walk.** Rejected: the walk audits a
  child's deeds *as the seat's deeds* (ADR-007 §2, A3); an access the seat
  itself may make cannot become a violation because its subagent made it. The
  exemption is report-identity-scoped, so a child still fails on every other
  report.
- **Persist the full gate output, or a worktree evidence file.** Rejected:
  raw logs blow past the 8 MiB checkpoint ceiling and the seat-board clone
  problem A7 exists for; a new worktree file would collide with ADR-006's
  reserved paths and the ownership matrix. A bounded per-leg tail in the
  checkpoint plus the documents stages already own is the shipped pattern
  (GitHub Actions' 1 MiB capped step summary is the same posture).
- **Hand reviewers only their own shell runs, skip the captured record.**
  Rejected by the ADR (the runner's record is the shared ground truth five
  independent seats and the fix loop can all cite) and kept as a complement —
  Decision 3's check commands stay.
- **Record only the reply message's `tokens`.** Rejected: a stage's session is
  a tool loop of many assistant messages; the reply's usage is one turn's, not
  the session's. Summing the attempt's assistant messages answers Decision 4's
  "the token usage of its session".
- **Gate the document ceiling like the plan stop (sticky record, escalate,
  commit window skipped).** Rejected: the plan stop exists because the light
  flow's plan is written once and the stop must outlive the escalation it
  minted; a full-flow document over length is shrinkable by the next revise,
  which the retry-then-escalate budget already models. Reusing the stage
  failure path gives the overshoot a repair loop instead of a terminal stop.
- **Blanket shell deny over `*reviews*` for review seats.** Rejected: it
  would deny the own-report re-read Decision 1 just legalized — the seat's
  only reachable route to its own report is `git diff*` (A21). The shipped
  last-match-wins grammar lets the rule be exact: deny the word, then re-allow
  `git diff` scoped to the seat's own specialty prefix (A11).

## Chosen design

### 1. Own-report identity on the audit's read side (Decision 1)

All of this lives in `auditSessionIndependence` in
`tools/orchestration/src/pipeline/independence.ts`; the
`IndependenceAuditOptions` shape does not change. The identity rule — the
canonical statement of the exemption, referenced below: *a `reviews/` access
value is the seat's own report iff its canonical spelling — normalized the way the
permission layer normalizes paths (A17: `\` to `/`, case folded), resolved against
the repo root — lies under the attempt's own design directory (the directory of
`opts.ownReportPath`) and its `reportKey` (everything from the `reviews` segment on)
equals `reportKey(opts.ownReportPath)`. Anything else — a directory, a glob, a
listing, a command naming a second report, another design directory's same-named
report — is not an identity match and fails as it does today.*

- Read-tools branch (`reviewStage && READONLY_TOOLS && REVIEWS_SEGMENT &&
  !.diff`, independence.ts:184–190): add the identity test above — skip the
  violation when it matches. A glob pattern like
  `reviews/design/*` keys as itself, never equals a concrete report key, and
  still fails; a `grep` whose `path` names the report directory or a sibling
  still fails; transcripts access is never exempted — there is no own-report
  in `transcripts/`.
- Shell branch (independence.ts:164–171): the `transcripts` test is unchanged.
  Replace the bare `REVIEWS_SEGMENT.test(m.command)` with: collect each whole
  command token containing a `reviews` segment — delimited as `REVIEWS_SEGMENT`
  delimits, prefix included, never the prefix-stripped `REVIEWS_KEY` fragment —
  exempt only if the set is non-empty and every token is an identity match to
  `own` above; a prefix-less token resolves outside the attempt's design
  directory and fails closed. A `git diff` naming own and a sibling fails.
- Prompts (`tools/orchestration/prompts/design-review.md`,
  `code-review.md`) gain one sentence each next to the existing
  `{{reportPath}}` lines: the report at `{{reportPath}}` is yours — re-reading
  it is allowed through your `git diff*` allow, since your `read` tool stays
  fenced off `reviews/` (A21); every other file under any `reviews/` directory
  stays closed. The existing prohibitions and the "Listing, globbing, or
  grepping those directories counts as opening them" sentence stay.

The permission layer keeps its shape: the seat `read` deny over
`docs/designs/**/reviews/**` (A10) stays, and the own-report re-open is §3's;
the seam between the two is the recorded risk, not a hidden gap.

### 2. The gate record (Decision 2)

Canonical shape, defined once here:
`GateEvidence = { at: number; green: boolean; legs: GateLeg[] }` with
`GateLeg = { command: string; cwd: string; exit: number | null; tail: string }`
— `command` the leg's command line as spawned (e.g. `cargo test --workspace
--locked`), `cwd` the package dir, `exit` the child exit code (null when
spawning failed or a signal killed it, with the failure spelled in `tail`),
`tail` the leg's merged stdout+stderr captured as bytes, decoded once as
UTF-8 with replacement so the stored tail never cuts a code point, and bounded
to the last `MAX_GATE_TAIL_CHARS = 2_000` characters of that decode, then
passed through `printable` (independence.ts:311) at write and again at render
(A7). `MAX_GATE_LEGS = 32` bounds the array; an elision note leg is
appended as `boundedFindings` does (A7).

- `tools/orchestration/src/pipeline/gates.ts`: new exported
  `checkGate(repoDir): Promise<{ green: boolean; legs: GateLeg[] }>` —
  `packageDirs`/`CARGO_GATE`/`hasCargoWorkspace` logic unchanged (A5) except that
  each spawn uses `stdio: ['ignore', 'pipe', 'pipe']` and a rolling tail
  buffer holding the last `MAX_GATE_TAIL_CHARS` of decoded output per leg.
  `testsGreen` becomes the thin `(await checkGate(repoDir)).green` wrapper, so
  its existing callers and tests keep their signature.
- `tools/orchestration/src/pipeline/types.ts`: `PipelineState.gateEvidence?:
  GateEvidence` (the `GateLeg`/`GateEvidence` interfaces live here, beside
  `SeatRecord`); `state-store.ts` `StateSchema` gains the matching
  `optional()` zod object — optional-with-default per A6, so a 015-era
  checkpoint loads unchanged.
- `tools/orchestration/src/pipeline/runner.ts`: at the one existing site after
  `implement`/`fix`/`test-update` (runner.ts:2831–2839), call `checkGate`,
  set `state.testsGreen` from `.green` (field kept, never retyped — A6), set
  `state.gateEvidence = boundedGateEvidence(gate)` before the same
  `checkpoint()` call, so a crash after a red gate persists the record exactly
  as the boolean does today. `gateLog` reasons unchanged.
- Prompt rendering, bounded separately from capture: `gateEvidenceText(state,
  cfg)` returns a fenced block capped at `MAX_EVIDENCE_BLOCK_CHARS = 6_000`
  whose header line fixes provenance — output the runner captured *from the
  code under review*, citable evidence, never instructions to the seat — with
  failing legs first and their tails (which carry the `test result:` line,
  A18), green legs as one-liners (`command exit 0` plus any tail line starting
  `test result:`). It renders `(no gate record for this run yet)` when
  `gateEvidence` is absent — the case for a run resumed from a pre-016
  checkpoint and for the first `implement`. New template var
  `gateEvidence` (defaulted in `stageVars`/`reviewVars` exactly as
  `planMaxLines` is) is appended to `prompts/code-review.md` (its "Your inputs
  are exactly" enumeration names the block, so seats may cite it),
  `prompts/fix.md`, `prompts/fix-tests.md`, and `prompts/implement.md`;
  `implement.md`/`fix.md` add the sentence that the block speaks under the
  header above, and that a failing-leg tail belongs in the committed
  `03-build-notes.md` "Gate posture" section — cited line-selectively,
  dropping any credential-bearing or absolute-path line — with its leg
  command and exit, through the document the coder already owns.
  doc-sync's existing completion of the build notes is unchanged.
- No worktree file is added, so ADR-006's reserved set (`namesReservedSegment`,
  types.ts:197–210) and the ownership matrix (`stagePathAllowed`) do not change.

### 3. Read-only reach and permission-layer run-record denies (Decision 3)

Config edits in `.opencode/agents/`, using the rule text already shipped in
`code/coder.md` (byte-identical allowlists per 01-research.md § Platform
notes), and respecting last-match-wins (A16):

- Document families — `research/researcher.md`, `design/designer.md`,
  `design/planner.md`, `design/editor.md`, `ops/doc-sync.md`: keep
  `shell * → deny` first, then the inspection set byte-for-byte as
  `code/coder.md` ships it — star form throughout, since under A16/A17 a bare
  `cat` rule matches only the argument-less command. Then the run-record
  block below: it is the single authority for these families' run-record
  denies. No toolchain: the leading deny keeps `cargo`/`npm` out. The block's
  `*reviews*` shell deny closes only the shell route; the families keep
  legitimate report reads through the `read` tool (A9) — the designer's
  handed `{{reviewFiles}}` still open.
- Design panel — the five `review/design-<specialty>.md`: copy the code
  seats' check block from `codereview/code-security.md` (the toolchain
  allows, the compound-command inspection set, and every trailing write-leg
  deny: `cargo fmt *` denied with `cargo fmt *--check*` re-allowed,
  `cargo fix *`, `cargo clippy --fix*`, `npm run format *`, commits, branch
  state, `rm`/`mv`/`tee`/`sed -i`/`dd`, publishes), keeping `git diff*`
  first-class. Then the run-record block below.
- Run-record block, appended after every other rule except the per-seat
  re-open below, to every shell-holding agent (the five design seats, the five
  code seats, coder, test-engineer, the five document families): shell denies
  `*transcripts*` and `*reviews*`; `glob` denies over
  `docs/designs/**/transcripts/**` and `docs/designs/**/reviews/**`; `grep`
  denies `*transcripts*` and `*reviews*` — knowingly partial: a `grep` rule
  matches the requested regex, not the search path, and a shell rule is raw
  command text (A17), so these stop the accidental plain-spelling access and
  the deed audit stays the backstop, exactly as ADR-007 says.
- Own-report re-open, per-seat agent after the deny:
  `shell git diff *reviews/<phase>/<specialty>-*` → allow, naming the agent's
  own specialty in the pattern (A11) — design seats get
  `git diff *reviews/design/<specialty>-*`, code seats
  `git diff *reviews/code/<specialty>-*`. This is the Decision 3 exception
  that keeps Decision 1's route open; the wildcard's extra reach over
  prior-round and other runs' same-named reports is the recorded seam.
- `prompts/design-review.md` rewording: replace "never browse to verify" —
  the seat may run the project's read-only checks (the cargo/rustfmt/npm
  legs, the same write legs denied) and may re-read its own `{{reportPath}}`;
  it may not open the run's records, with the existing
  listing/globbing/grepping sentence kept, so prompt, permission layer, and
  audit say the same thing. The `subagent` hole stays where Non-goals and
  Deferred place it.

### 4. Budgeted, measured context (Decision 4)

- Tokens per transcript: `executeInference` already holds the attempt's
  message list (`lastSeen`, runner.ts:1500). New helper `sessionTokens`
  (prompts.ts) sums the numeric fields of each assistant message's `tokens`
  (A12; per-key summation, field names not enumerated per A13) over the
  attempt's messages — for a reply-retry that resumed the same session, that
  is the whole session's usage, which is what Decision 4 names. A `tokens?:
  Record<string, number>` field on `TranscriptInput` (prompts.ts) prints
  `- tokens: <json>` beside the unchanged `- parameters:` line; absent when
  every message carries no usage, and the parameters note keeps its old
  meaning (endpoint exposure, not usage). Nothing is added to the checkpoint
  and no server or SDK change is needed.
- Research prompt (`tools/orchestration/prompts/research.md`): names its read
  set — the source document at `{{sourcePath}}`, the `docs/adrs/` directory
  listing (the ADR index; no index file exists, A22), the ADRs the source
  cites by number, and the immediately prior run's `01-research.md` +
  `02-proposal.md` if present — and states a ceiling: at most 25 reads of
  files outside that set. The ceiling is prompt-stated; machine-checking it
  is the Deferred entry.
- Document ceiling: `PipelineConfig.docMaxLines?: number` (stages.ts, beside
  `planMaxLines`), validated like `validatePlanMaxLines` (flows.ts);
  `FLOWS.full.config` sets `docMaxLines: 900` — headroom above the largest
  shipped full-flow document in this repo (A24) and far below run 2's
  2,696-line runaway the meter exists for (A15). The light flow leaves it
  absent: its plan stays under `planMaxLines: 80`.
- Metering `01-research.md`: `proposalSize` generalizes to take the doc name
  (`documentSize(repoDir, designDir, docName)`), `docFileFor(stage)` maps
  `research → 01-research.md`, every other measured stage keeps
  `02-proposal.md`, and `FLOWS.full.docStages` gains `research`. The
  `docLines*` fields of `StageResult` (types.ts:110–112, already optional)
  carry the counts unchanged.
- Gate: `documentOutgrewFlow(lines, cfg)` in gates.ts runs at the existing
  stage-ok point where the meter already measures (runner.ts:1701–1723); going over
  length turns the attempt into a normal stage failure —
  `document outgrew the flow cap (<N> lines over cap <M>)` — so the retry
  mechanism (same-session reply retry or attempt 2 with `failureNote`,
  runner.ts:1344) instructs the shrink, and the existing
  one-retry-then-escalate budget terminates it. Absent config is the
  unchanged measure-and-warn behavior. Not the plan-stop pre-commit: see
  Options.
- Decision 4 also names what does not change: `maxParallelReviews` stays 1
  (stages.ts:119), with no `--max-parallel`, KV, or server change — one served
  slot and A20's memory picture mean more concurrency evicts prefixes.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Evidence block grows every review prompt — the very quantity only Decision 4 measures (the ADR's circular-pricing consequence) | high | medium | Two pinned budgets: `MAX_GATE_TAIL_CHARS` (capture) and `MAX_EVIDENCE_BLOCK_CHARS` (render); Test strategy #1/#7; the new `- tokens:` line makes the cost checkable per stage from the next run on |
| Checkpoint growth: one `gateEvidence` field cloned into every later snapshot | medium | low | Bounded legs × bounded tail ≪ 8 MiB loader cap (A6); Test strategy #6 pins the caps and an oversized-tail fixture |
| Coarse `*transcripts*`/`*reviews*` shell denies misfire on a legitimate command that merely names the words (A23) | low | medium | Accepted by the ADR; the one named exception (own report, Decision 1) is re-opened by the per-seat `git diff` allow; Test strategy #5 pins both directions |
| Permission/audit seam: the per-seat wildcard `git diff *reviews/.../<specialty>-*` also opens the seat's *prior-round* reports and another design directory's same-named report (A16's `*` spans `/`), which the audit's design-directory-scoped identity rule (§1) then fails | medium | low | Accepted: the prompt names only the current pass's `{{reportPath}}`, prior findings already ride `{{prevFindings}}`, and a burned attempt for an off-prompt read is today's behavior (no regression); closing the seam needs per-run permission templating — deferred |
| Own-report exemption propagates into the child walk (A3) | low | low | Intentional (child deeds are the seat's deeds, ADR-007 §2); scoped to the one report identity, every sibling still fails — Test strategy #3 |
| A resumed or in-flight run meets the new `docMaxLines`: a document already over the cap fails its next document stage | low | medium | Registry config, not code: the operator raises `docMaxLines` or forces the stage; the failure text names the cap and the repair |
| The design panel now runs commands, so prompt wording, permission rules, and the audit must agree | medium | high | One wording for all three in §3; Test strategy #4/#5 pin the rule matrices; the review-stage audit rule (reads of `reviews/` fail regardless of shell rights) is untouched except the identity exemption |
| The delegation hole lets every widened family route around its allowlist until a `subagent` deny ships | medium | medium | Accepted as ADR-016 scope; the deed audit and the ownership sweep stay the backstop; deferred entry names it |
| `tokens` sum over a resumed session counts work a prior attempt did | low | low | Honest at session granularity, which is what Decision 4 names; the transcript's attempt label disambiguates |
| Rolling tail cuts the decoded output mid-line, so the `test result:` line could be cut; a tail is raw output of the repo's own check commands and can carry absolute paths or registry text | low | low | 2,000 characters from the end comfortably holds cargo test's multi-line summary (A18); tails are evidence, not parsed contracts; the checkpoint and transcripts stay local and the build note is the one committed carrier, holding only §2's line-selective citation with the credential-bearing and absolute-path lines dropped — no redaction pass ships |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux dev/CI host | yes | The only exercised environment: `tools/orchestration` Node at the `engines.node` floor — raised to `>=22.23.5` for the 2026-11-11 Node security release (package.json:30–31) — plus agent rule text in `.opencode/agents/` and prompt text; §3's deny order rests on the wildcard grammar A16 pins |
| macOS | not exercised | Harness change developed and tested on Linux only. It adds host-dependent surfaces — §3's POSIX inspection set (`cat`/`wc`/`find`) and the design panel's spawned `cargo`/`rustfmt`/`npm` legs — so no claim is made for this host |
| Windows / WSL2 | not exercised | Same posture as macOS: no native-shell equivalent of the §3 inspection set, and cargo/npm presence is per host, so §3's widening is not claimed there |
| Checkpoints written before this change | yes | `gateEvidence` is optional (A6); `testsGreen`, `docLines*`, and the golden fixtures in `tests/pipeline/fixtures/checkpoints/` load unchanged |
| A run in flight across the merge | yes | Pre-016 checkpoint ⇒ `gateEvidence` absent ⇒ prompts render "(no gate record for this run yet)"; transcripts without usage data keep printing no `- tokens:` line. The same merge also applies `docMaxLines` and `research` in `docStages` to that run: a document already over the cap fails its next document stage — raise the cap or force the stage (Risks) |
| Full flow | yes | Gains `docMaxLines` in the registry and `research` in `docStages` |
| Light flow | yes | Gets the doc-family/planner shell widening and the gate record; `docMaxLines` stays absent there — `planMaxLines: 80` remains the plan's authority |
| Pre-ADR-016 OpenCode server config | yes | Only standard rule types (`shell`, `glob`, `grep`, `read`) are used; no `policies` overlay is required. §3's deny-then-reallow order relies on the wildcard grammar the server the pinned `@opencode/client ^2.1.4` speaks documents — A16 pins the repo's local matcher to it |

## Test strategy

1. Unit — capture: against a fixture repo, `checkGate` records one leg per
   probed package and per `CARGO_GATE` leg with `exit` and the last
   `MAX_GATE_TAIL_CHARS` characters of the leg's decoded output (a tail from
   the end contains a `test result:` line; bytes that are not valid UTF-8 become
   replacement characters, never a split code point); a spawn failure records
   `exit: null` and names the failure in the tail; legs over `MAX_GATE_LEGS`
   collapse to an elision leg — the capture-budget test the ADR names as
   load-bearing.
2. Unit — checkpoint grammar: a state carrying `gateEvidence` parses; every
   existing golden checkpoint fixture parses without it; `boundedGateEvidence`
   strips control bytes via `printable` and bounds the array at write, and
   `gateEvidenceText` re-applies both at render (A7).
3. Unit — audit identity (extend `independence.test.ts`): read of own report
   (repo-relative, absolute, and separator- or case-variant spellings) is
   clean; a sibling report, another design directory's same-spelled report, a
   directory, a glob pattern, and a grep `path` under `reviews/` fail; shell
   `git diff docs/designs/<own>/reviews/design/<specialty>-r1c1.md` — the
   spelling §3's allow opens — is clean; the same path under another run, or
   own plus a sibling in one command, fails; `ls .../reviews` fails;
   transcripts access is never exempted; a child session reading the parent's
   own report inherits the exemption; a sibling names the child session.
4. Unit — agent shell matrix (extend `agent-shell.test.ts`): the five design
   seats and the five document families carry shell rules covering every
   compound gate command leg (design seats only — doc families deny
   `cargo`/`npm`); doc families allow `wc <file>` and `cat <path>` (with
   arguments, never the bare command names) and deny `cargo test`, and carry
   the same run-record `glob`/`grep` denies as the seats; every shell-holding
   agent denies a command naming `transcripts` or `reviews`, and each review
   seat's own-specialty `git diff` allow beats that deny while a
   sibling-specialty path stays denied; the fixture's matcher stays pinned to
   the grammar the pinned client's server documents (A16/A17).
5. Unit — design-seat posture keeps the code-seat invariants: `cargo test
   --workspace --locked` and `cargo fmt --all -- --check` allowed; `cargo fmt
   --all`, `cargo fix --all`, `cargo clippy --fix`, `npm run format`,
   commits/branch state/`rm`/`mv`/publishes denied.
6. Unit — `documentOutgrewFlow`: over-cap denies with the count and cap in
   the reason, under-cap allows, absent cap allows; `validateConfig` refuses
   a non-positive `docMaxLines`; `documentSize` measures `01-research.md` for
   the `research` stage.
7. Unit — prompt render: `code-review.md`, `fix.md`, `fix-tests.md`,
   `implement.md` render with `{{gateEvidence}}` filled (strict render), the
   absent record renders the "(no gate record for this run yet)" line, and
   the rendered block never exceeds `MAX_EVIDENCE_BLOCK_CHARS`, failing legs
   appearing first.
8. Unit — transcript tokens: `sessionTokens` sums per-key numeric usage across
   a fixture message list; `writeTranscript` prints `- tokens:` when present
   and omits the line when not; the `- parameters:` line is byte-unchanged.
9. Integration — red gate round trip: in a fixture repo with a failing leg,
   after an `implement`, the checkpoint's `gateEvidence.legs` names the failing
   command with nonzero `exit` and a tail containing the failure, and the
   next `fix` attempt's prompt contains that leg's tail.
10. Integration — green gate evidence for a seat: a run whose gate passes
    renders each green leg's `test result:` line into the
    `code-review` seat's prompt, so a behavior claim has a citable line (the
    "unevaluable" finding class gets an answer).
11. Integration — own-report seat: a staged message list in which a design
    seat writes its report, then re-reads exactly that path via `read` and
    `git diff`, passes the audit and the attempt; the same with one sibling
    path added fails it with the sibling named.
12. Integration — oversized document: a fixture where `01-research.md`
    exceeds the cap fails the research attempt with the cap named, retries
    once, and escalates with the cap in the gate log; raising `docMaxLines`
    lets the same fixture pass.
13. Integration — resumed legacy run: a run resumed from a
    `tests/pipeline/fixtures/checkpoints/`-era checkpoint completes a stage
    with no `gateEvidence` field and no prompt breakage.
14. Platform — exercise everything above on Linux (the dev/CI posture); the
    cargo legs run against the real workspace in the integration tests that
    need a green gate.

## Rollout

1. Land as one branch; no migration and no checkpoint version bump —
   `gateEvidence` and the widened agent configs are additive (A6). Merged
   runs pick up the gate record, the prompt blocks, and the token lines
   immediately.
2. Shipped constants: `MAX_GATE_TAIL_CHARS = 2_000`, `MAX_GATE_LEGS = 32`,
   `MAX_EVIDENCE_BLOCK_CHARS = 6_000`, `docMaxLines: 900` in
   `FLOWS.full.config` — config, not CLI surface, following `planMaxLines`.
   `tools/orchestration/package.json` `engines.node` rises to `>=22.23.5` for
   the 2026-11-11 Node security release — the only runtime prerequisite added.
3. `docs/workflows.md` gains: the design artifact table row for the build
   note's "Gate posture" section, one line that review prompts carry a
   bounded runner gate block, and one line that `transcripts/`/`reviews/` are
   now denied in the permission layer, not just audited.
4. First post-merge full-flow run is Decision 4's first checkable baseline:
   every transcript carries `- tokens:`, and the ADR's prompt-size claims
   become numbers the operator can re-read. If the numbers show the evidence
   block's context cost is disproportionate, shrink `MAX_EVIDENCE_BLOCK_CHARS`
   — one constant.

## Deferred / out of scope

- Whether any family may delegate to a subagent, and a `subagent` deny per
  family — ADR-016 "Not decided here"; no follow-up doc yet.
- A middle flow tier between `full` and `light` — ADR-016 "Not decided
  here".
- Served slot count, KV residency, `--max-parallel`, and the host memory
  layout — ADR-016 Decision 4 refuses; any future change owns it.
- Machine-checking the research read ceiling (counting reads per attempt in
  the deed audit) — this design states the budget in the prompt only.
- Per-run permission templating so the review seat's shell allow matches the
  audit's exact pass identity, closing the seam in Risks — no mechanism
  today; no follow-up yet.
- A non-text fence over run records (OS sandbox / `policies` overlay) —
  out of ADR-016's scope; 01-research.md § Platform notes names the field's
  tools if ever wanted.
- Displaying token totals or the gate record in `pipeline status` /
  `report.ts` — transcripts and the checkpoint carry them; no follow-up yet.
- An on-disk ADR index document — the `docs/adrs/` listing serves; creating
  one is product/docs work, no follow-up yet.

## Revision log

- Chosen design §1 Own-report identity (r2): the shell operand is whole command tokens,
  prefix included — a prefix-less spelling fails closed; #3 pins the §3 spellings.
- Chosen design §2 The gate record (r2): the build-note citation is line-selective —
  credential-bearing and absolute-path lines dropped — the build note the one committed carrier.
- Chosen design §3 Read-only reach (r1): one run-record rule set for the doc families, their
  inspection set byte-identical to `code/coder.md`'s star-form text, block before the re-open.
- Compatibility matrix + Rollout (r1): macOS and Windows/WSL2 read `not exercised` (POSIX
  inspection set, cargo/npm legs); `engines.node` → `>=22.23.5`; A16/#4 pin the matcher.
- A24 + §4 Document ceiling (r1): shipped artifact sizes measured and registered — largest
  `02-proposal.md` 733 lines — so the cap rises 600 → 900, which 600 sat under.

Editor notes: Options considered — "raw logs blow past the 8 MiB checkpoint
ceiling and the seat-board clone problem A7 exists for" is ambiguous; author to
clarify.
Editor notes: Test strategy #3 — "a sibling names the child session" is
ambiguous; author to clarify.

## As-built (2026-11-13)

The branch implemented Decisions 1–4 as approved; code review round 2 closed
with five approvals. This note records where the shipped code drifted from this
document's text — design decisions are not amended here, only the as-built
state. Full per-item detail, with the review findings that motivated each, is
in `03-build-notes.md` (deviations 1–14 and its dated addendum).

- **§2 The gate record — citation rule moved to the write seam.** Drift:
  `boundedGateEvidence` applies §2's line-selective citation rule *before*
  persistence — credential-shaped and absolute-path lines are dropped from the
  stored tail with a stated marker, and each leg's `cwd` is stored
  repo-relative (absolute with no repo to judge against is withheld). Reason:
  the Risks row's premise "the checkpoint and transcripts stay local" is false
  in this repo — the stage commit's `git add -A` commits `.pipeline.json` and
  the transcripts (security r2, finding 1). Affected: §2 (canonical shape
  unchanged; the bounding widened to the write seam), Risks row 10.
- **§2 Prompt render — the evidence block owns its own syntax.** Drift: `$ `
  opens a leg line, `| ` opens captured output, fence runs inside captured
  text are neutralized, and the closing fence is reserved inside
  `MAX_EVIDENCE_BLOCK_CHARS`; a green leg's citable line is the *last* single
  `test result:` clause (ended at cargo's ` finished in `, else a 160-char
  window), not a whole-line match. Reason: gate output is written by the code
  under review and must not be able to forge a leg-shaped line or escape its
  block (security r2, finding 3; correctness r2, finding 2). Affected: §2
  rendering bullet.
- **§2 Capture — the bounded tail is self-describing.** Drift: `boundedTail`
  prepends a line-leading `[<n> characters elided by the gate tail cap]`
  marker whose count and the kept span are solved as one equation inside the
  2,000-char budget (dev 2, 13); capture keeps a 64 KiB byte window
  (`MAX_TAIL_BYTES`) per leg before the decode-then-bound step. Reason: a
  truncated tail must not read as a whole transcript; the byte window bounds
  runner memory while preserving the tail-of-decoded-output contract.
  Affected: §2 canonical shape (the four fields are unchanged).
- **§2 plumbing names.** Drift: `gateEvidence` is also defaulted in
  `TEMPLATE_DEFAULTS`, not only `stageVars`/`reviewVars` — pre-existing
  fixtures render the real templates with hand-built vars and a strict render
  needs a default at the boundary (dev 1); `checkGate` returns a named
  `GateRun` (dev 6); the gate-log reason reads `(<N> lines, cap <M>)` (dev 4);
  `gateEvidenceText(state, cfg)` keeps `cfg` unread at the named seam (dev 5).
  Affected: §2.
- **§3 run-record block — the fence now covers the checkpoint.** Drift: the
  block carries a third shell deny (`*pipeline.json*`) plus `read` and `glob`
  denies over `docs/designs/**/.pipeline.json` on all seventeen shell-holding
  agents, and `auditSessionIndependence` tests the `STATE_FILE` name as a
  whole, case-folded segment for every stage, on the shell and access-value
  branches. Reason: the checkpoint carries every seat's verdicts and findings
  and the gate record; ADR-006 §1 reserves it to the runner (security r2,
  finding 2). Open nit at branch end: the block carries no `grep` deny for
  that name, though its comment claims the directories' rule shape. Affected:
  §3 run-record block and re-open ordering, Compatibility matrix rows.
- **§4 metering — `StageResult.docName` added.** Drift: an additive-optional
  checkpoint field names the document each size record measured
  (`PROPOSAL_DOC`/`RESEARCH_DOC`); `proposalGrowth` skips research records and
  the runner's stage line prints the metered file name. Reason: putting
  `research` in `docStages` otherwise summed two documents under the one
  `proposal:` label (correctness r2, finding 3); absence reads as the design
  document, so A6 holds and the golden fixtures are unchanged. Affected: §4
  metering bullet, `StageResult` field list.
- **§4 tokens / A13 — resolved in favor of completeness.** Drift: A13 is no
  longer unverified — the pinned client types `TokenUsageInfo` as
  `{ input, output, reasoning, cache: { read, write } }` — and `sessionTokens`
  flattens one nesting level into dotted keys (`cache.read`, `cache.write`)
  rather than summing only top-level keys. Reason: the flat sum would silently
  drop the largest terms for a prefix-re-prefilled session, the very quantity
  Decision 4 exists to make checkable (correctness r2, finding 1); non-numeric
  and deeper nesting are still skipped. Affected: §4 tokens bullet, A13.
- **Rollout 2 — the CI pin moves with the floor.** Drift:
  `.github/workflows/check.yml`'s `setup-node` version rises to `22.23.5`
  with `engines.node`; the workspace gate-contract test pins the workflow
  string to the package floor, so raising one without the other turns
  `cargo test` red (dev 3). Carried open at branch tip: two of the four
  statements of the floor still read `22.23.4` — `README.md:24` and the
  `repo-practices` skill's "Node floor for the orchestration gate" — which
  the gate-contract test does not read (compat r2c2 finding 2, minor).
  Affected: Rollout 2, Linux compatibility row.
- **Test strategy posture.** #1–#13 shipped under `tests/pipeline/`
  (`gate-evidence-*`, `independence`, `agent-shell`, `own-report-seat`,
  `document-cap*`, `proposal-size`, `transcript-tokens`, prompts/flows
  extensions). Two drifts to record: #14's "the cargo legs run against the
  real workspace in the integration tests" is not the vitest posture — cargo
  is PATH-stubbed there, and green cargo legs are evidenced by the runner's
  own gate record and the CI rust job (carried as a testing-seat minor); and
  the `proposalGrowth` `docName` filter shipped without a pinning test (open
  minor, testing r2 ratification finding 1). #10's citable line is shipped as
  the *last* summary clause, which on a real workspace is the doc-test
  `0 passed` line — the seat ratifying the fix filed that knowingly (compat
  r2c2 finding 1, minor). Affected: Test strategy #6, #10, #14.
- Shipped as written, no drift observed: §1's audit identity (one
  design-directory + report-key authority shared by the read and shell sides,
  options shape unchanged, the child-walk case resolved as §1 intends), §3's
  per-seat `git diff *reviews/<phase>/<specialty>-*` re-opens and byte-for-byte
  check/inspection blocks, §4's `docMaxLines: 900` / `validateDocMaxLines` /
  research read set with the 25-read ceiling, the prompt changes, and
  Rollout 3's `docs/workflows.md` lines.
