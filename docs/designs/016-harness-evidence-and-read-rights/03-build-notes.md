# Build notes — 016 harness evidence and read rights

Implemented against `02-proposal.md` (Decisions 1–4) as merged, plus the
Rollout items the proposal names, plus the fixes code review round 2 asked for.
The package gate ran as its own legs; the posture is below, then the deviations.

## Gate posture

Run by hand in this stage's shell, leg by leg, because the runner's own record
for this stage lands only at the stage's commit — the legs and exits here are
the same commands `checkGate` records, and the tails are quoted selectively
(their last lines, dropping absolute paths):

| Leg | cwd | exit |
| --- | --- | --- |
| `npm run check` (lint → format:check → test) | `tools/orchestration` | 0 |
| `cargo fmt --all -- --check` | repo root | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | repo root | 0 |
| `cargo test --workspace --locked` | repo root | 0 |

Green legs, one citable summary line each:

- `npm run check` → `Test Files  51 passed (51)`,
  `Tests  557 passed | 2 skipped (559)`
- `cargo fmt --all -- --check` → no output (nothing to reformat)
- `cargo clippy --workspace --all-targets -- -D warnings` → finished the dev
  profile with no warning to deny
- `cargo test --workspace --locked` → every target green; the workspace gate
  contract alone reads `test result: ok. 8 passed; 0 failed; 0 ignored;
  0 measured; 0 filtered out; finished in 0.00s`
  (`bindforge-core --test gate_contract`)

These four legs were re-run after the last edit in this tree (the npm leg at
14:31 with the workspace cached for the cargo legs). The table supersedes the one
the first implementation pass committed: that table recorded the npm leg at exit
1 on one fixture, and the failure was inside a test file — the diagnosis is kept
below for the record, and the test-update commit fixed that fixture, so the npm
leg has been green at every commit since.

### Round 1's red leg (kept for the record)

The red leg was one fixture, and the failure sat inside a test file, so it
routed to the test engineer rather than to a source fix:

```
Test Files  1 failed | 50 passed | 1 skipped (52)
Tests  1 failed | 556 passed | 3 skipped (560)
FAIL tests/pipeline/document-cap-integration.test.ts > … > fails, retries and
escalates a research document over the cap
AssertionError: expected '…' to contain 'document outgrew'
+ unhandled pipeline error after stage design-review: ENOENT: no such file or
directory, open '…/docs/designs/016-oversized/02-proposal.md'
```

Why it was a fixture fault, not an implementation one: the fixture resolved the
design directory **before** the run created it —
`researchStub(dir, designDirRel(dir), CAP + 100)` at line 170, where
`designDirRel` returned `''` while `docs/designs/` did not exist yet — so the
stub's `writeFileSync(join(dir, designDirRel, "01-research.md"))` landed the
oversized document at the **repo root**, never in the design directory. The meter
(`documentSize(repoDir, designDir, docFileFor(stage))`, the signature Decision 4
names) then correctly found no `01-research.md` to measure, the run walked on
past research with no document at all, and the gate log's reasons were the
funnel's and an `ENOENT` — never the cap. The same `''` also explains the
`02-proposal.md` ENOENT line above. The fixture fix is one line: resolve the
design-dir-relative path inside the stub's `prompt()` handler (or create
`docs/designs/016-oversized` before the stub is built).

## Deviations from the proposal

1. **`gateEvidence` default lives in `prompts.ts`, not only in
   `stageVars`/`reviewVars`.** Decision 2 says the var is "defaulted in
   `stageVars`/`reviewVars` exactly as `planMaxLines` is". It is defaulted
   there, **and** added to `TEMPLATE_DEFAULTS` (prompts.ts:37) — the file's
   existing home for "values a template may ask for that are NOT flow config",
   where `testOwnership` already sits. Without it, three pre-existing
   fixtures that render the real templates with hand-built vars
   (`fix-trigger.test.ts` renders `fix-tests.md` with only `{{designDir}}`;
   `reply-contract.test.ts` and `moves-stage.test.ts` call `executeStage` with
   `vars: { designDir: … }`) throw `unresolved template placeholder:
   {{gateEvidence}}` on the new placeholder. Both prompt suites are committed,
   so the strict render needs a default at the render boundary. The runner
   still always supplies the real block; the default is the fallback, never the
   authority — the same sentence that already covers `testOwnership`.
2. **`MAX_GATE_TAIL_CHARS` tail carries an explicit elision marker.** The
   proposal describes the tail as "the last `MAX_GATE_TAIL_CHARS` characters".
   `boundedTail` stores `[<n> characters elided by the gate tail cap]` followed
   by the kept last lines, with the marker's own length counted against the
   budget (`note.length + keep ≤ 2_000`). A truncated tail that reads like a
   whole transcript is the failure the capture test pins
    (`tail.startsWith('xxxx') === false`, `tail.length ≤ 2_000`), and the marker
    is what makes the truncation legible in a prompt. See 13 for how `<n>` is
    now sized.
3. **CI `node-version` pin moved with the Node floor.** Rollout 2 names only
   `tools/orchestration/package.json` `engines.node → >=22.23.5`. The repo's
   own gate contract (`crates/core/tests/gate_contract.rs`,
   `the_orchestration_job_pins_the_node_floor_the_package_declares`) pins
   `.github/workflows/check.yml`'s `setup-node` version to that floor, so
   raising the floor without the workflow pin turns `cargo test` red.
   `.github/workflows/check.yml:48` is now `"22.23.5"`, with a comment saying
   the two move together. This is the proposal's Rollout 2 change completed,
   not an extra one — but it is a file the proposal did not name.
4. **Gate-log reason wording.** The proposal's example reason is
   `document outgrew the flow cap (<N> lines over cap <M>)`; the shipped string
   is `document outgrew the flow cap (<N> lines, cap <M>)`. The committed
   integration test asserts the substrings `document outgrew` and the cap
   number; the shorter parenthetical reads better next to the plan-stop line
   the same log prints.
5. **`gateEvidenceText(state, cfg)` keeps `cfg` unread** (`void cfg` plus a
   comment). Decision 2 names the resolved flow config at that seam so a
   per-flow evidence budget would be registry config, but Rollout 2 ships one
   constant (`MAX_EVIDENCE_BLOCK_CHARS = 6_000`) for both flows, so nothing
   reads it yet. The parameter is kept at the named seam rather than dropped.
6. **`checkGate` returns a named `GateRun` (`{ green, legs }`)** rather than an
   inline object type; shape unchanged. `testsGreen` is the thin
   `(await checkGate(repoDir)).green` wrapper as specified, so its callers and
   tests keep their signature.
7. **Leftover diagnostic file on a test path.**
   `tools/orchestration/tests/pipeline/zz-scratch-dbg.test.ts` was an inert
   `describe.skip` stub holding a temporary end-to-end probe of the document-cap
   meter. It contained no assertions and must not survive the branch; it is
   absent from the tree at this commit (the round-2 correctness review confirms
   the same).

The remaining deviations (8–14) are the code-review round-2 fixes. Each extends
a Decision rather than adding one; each is named because it touches something
the proposal text did not.

8. **The run-record fence now covers the checkpoint too** (security r2, finding
   2). Decision 3's run-record block names `*transcripts*` and `*reviews*`; the
   same argument now applies to `.pipeline.json`, which carries every seat's
   verdict and findings, the gate record, and — per ADR-016's Context — is a
   violation for a stage to read. So all seventeen shell-holding agents now also
   deny `shell *pipeline.json*`, `read` and `glob` on
   `docs/designs/**/.pipeline.json` (same block, same coarse shape, before the
   per-seat re-open), and `auditSessionIndependence` tests the `STATE_FILE` name
   as a whole path segment — case-folded, built from `STATE_FILE` so the audit
   and the writer cannot drift — for every stage, on the shell branch and on the
   access-value branch. Nothing in the proposal's §3 rule set named the
   checkpoint; the repo already denied stages *editing* it, and this closes the
   read and shell sides. The coarse `*pipeline.json*` shell rule can deny a
   command that merely spells the name — A23's accepted cost, exactly the shape
   of the `*reviews*` deny — and no stage prompt gives a reason to spell it: a
   suite about the checkpoint runs by file path
   (`npm test -- tests/pipeline/gate-evidence-checkpoint.test.ts`), not by
   naming the state file.
9. **The line-selective citation rule runs where the record is WRITTEN**
   (security r2, finding 1). The proposal's Risks row rests tail hygiene on "the
   checkpoint and transcripts stay local"; that premise is false in this repo —
   the stage commit's `git add -A` commits `docs/designs/**/.pipeline.json`
   (no `.gitignore` entry covers it) and the transcripts are committed too. So
   `boundedGateEvidence` now applies the build note's rule before persistence:
   lines matching credential shapes (`_auth…=`, `token=`/`secret=`/`password=`
   style assignments, `scheme://<user>:<pass>@host`, an `Authorization:` header)
   and lines carrying an absolute path are dropped, with a stated
   `[<n> tail line(s) dropped by the record's citation rule]` marker rather than
   a silent cut, and each leg's `cwd` is stored repo-relatively
   (`boundedGateEvidence(evidence, repoDir)` — one new optional parameter; with
   no repo to judge against, an absolute cwd is withheld outright). Capture
   stays raw: `checkGate` still records the absolute package dir and the whole
   decoded tail, which is what the capture tests pin, and every persisted and
   rendered copy goes through the write seam.
10. **The rendered evidence block owns its own syntax** (security r2, finding 3;
    correctness r2, finding 2). `$ ` opens a leg line and `| ` opens captured
    output, so captured text cannot forge a leg-shaped line inside its own
    block; a run of three or more backticks inside captured text gains a leading
    backslash, so a captured ``` line cannot close the fence early; the closing
    fence is reserved inside `MAX_EVIDENCE_BLOCK_CHARS`, so the cap's final
    slice trims captured text and never unbalances the block. And a green leg's
    citable line is now the **last** `test result:` clause, ending at cargo's
    ` finished in ` terminator (or a 160-character window when it is absent) —
    the old `[^\n]*` match ran from the first summary to the end of a
    `printable`-flattened tail, which is not the one-liner §2 promises.
11. **`StageResult.docName` (additive-optional checkpoint field) keeps the
    growth aggregate on the design document** (correctness r2, finding 3).
    Putting `research` in `docStages` let `01-research.md` counts flow into
    `proposalGrowth`, so the operator line summed two documents under one
    `proposal:` label and counted research as a design stage. The meter now
    names the document it measured (`types.ts` `PROPOSAL_DOC`/`RESEARCH_DOC`,
    recorded on the `StageResult` and in `StateSchema.results`), and
    `proposalGrowth` skips records whose `docName` is `01-research.md` —
    absence reads as the design document, so pre-016 checkpoints and the golden
    fixtures are unchanged (A6). The runner's own stage line prints the metered
    file name instead of the word `proposal`.
12. **`sessionTokens` resolves open assumption A13 by flattening one nesting
    level.** The pinned client types a message's `tokens` as
    `TokenUsageInfo = { input, output, reasoning, cache: { read, write } }`; the
    original per-key sum walked only the top-level keys, so `cache.read` and
    `cache.write` — the largest terms for a session whose prefix is re-prefilled
    — were dropped silently, which is exactly the quantity Decision 4 exists to
    make checkable. Nested numeric objects now contribute dotted keys
    (`cache.read`, `cache.write`); non-numeric values and nesting deeper than
    one level are still skipped, so an unexpected shape can only add a key,
    never lose one. The existing flat fixture keeps passing; the real-shape
    fixture the review asks for is test-engineer work (below).
13. **`boundedTail`'s elision marker states the true elision** (correctness r2,
    finding 4). The marker's text is deviation 2's; what changed is the
    arithmetic. The kept span and the stated count are solved together
    (`keep = MAX_GATE_TAIL_CHARS − note.length`, `elided = chars.length − keep`)
    instead of sizing the span against a fixed 64-character reservation, which
    over-reported the elision by however far the note fell short of that
    reservation (≈ 20 characters). The budget holds exactly:
    `note.length + keep = 2 000`.
14. **`prompts/design-review.md` scopes the sentence it keeps** (correctness r2,
    finding 5). §3 asked for "never browse to verify" to be replaced; the first
    pass kept it verbatim and added the allowance after it. It is still kept —
    `prompts.test.ts` pins the wording, and the prohibition is real — but it now
    says what it prohibits: walking a directory to locate a path or confirm that
    a named path exists. The next sentence says plainly that running the
    project's read-only check legs is not browsing, so the two cannot read as
    contradicting each other.

## Tests this round needs from the test engineer (`testsNeedUpdate: true`)

No test file was edited here. The fixes above change pinned behaviour and open
new behaviour nothing yet pins:

1. `tests/pipeline/agent-shell.test.ts` — the run-record pair for the
   checkpoint: every shell-holding agent denies `cat
   docs/designs/002-x/.pipeline.json` and `git show
   HEAD:docs/designs/002-x/.pipeline.json`, and carries the
   `read`/`glob` denies over `docs/designs/**/.pipeline.json`, the way the
   `transcripts`/`reviews` pair is pinned today.
2. `tests/pipeline/independence.test.ts` — a `read`/`glob`/`grep`/`write` whose
   access value, or a shell command whose text, names `.pipeline.json` is a
   violation for any stage (including `git show HEAD:…`-style spellings); a
   `pipeline-notes.md`-shaped name is not.
3. A write-side capture test: a fixture leg whose output carries a line of the
   `//registry.npmjs.org/:_authToken=…` shape and a line holding an absolute
   path — after `boundedGateEvidence`, neither reaches the persisted record, and
   the drop is stated by the marker; with a `repoDir` handed in, `cwd` comes
   back repo-relative (`.` for the root package, `tools/a` for a nested one).
   `checkGate`'s raw capture posture stays as it is pinned today.
4. `tests/pipeline/gate-evidence-render.test.ts` — a tail containing a ``` run
   cannot close the evidence fence and the block stays fence-balanced at the cap;
   a green leg whose flattened tail holds two `test result:` summaries emits the
   last one only, as one line.
5. `tests/pipeline/transcript-tokens.test.ts` — a fixture in the real
   `TokenUsageInfo` shape (`{ input, output, reasoning, cache: { read, write } }`)
   shows `cache.read`/`cache.write` in the totals; a deeper-nested or
   non-numeric group stays skipped.
6. `tests/pipeline/proposal-size.test.ts` — a research `StageResult` carrying
   `docName: '01-research.md'` is excluded from `proposalGrowth`, while an
   unnamed record (the pre-016 shape) still counts as the design document.

## Notes on what landed where (no deviation, for the reader's orientation)

- `gates.ts`: `checkGate` (piped stdio, decoded once with replacement
  characters, tail kept from the end so `cargo test`'s `test result:` summary
  survives, `exit: null` for spawn failure/signal kill), `boundedLegs`,
  `boundedTail` (deviation 13's arithmetic), `documentOutgrewFlow`.
- `types.ts` / `state-store.ts`: `GateLeg`, `GateEvidence`,
  `PipelineState.gateEvidence?`, `StageResult.docName?`, the metered-document
  names (`PROPOSAL_DOC`, `RESEARCH_DOC`), and the matching optional zod schemas —
  additive-optional per A6, so every golden checkpoint fixture still loads with
  no new key materialised.
- `runner.ts`: gate record at the one existing `implement`/`fix`/`test-update`
  site, before the same `checkpoint()` call; `boundedGateEvidence` at write
  (deviation 9's citation rule, the repo-relative `cwd`), `gateEvidenceText`
  re-bounding at render (deviation 10's line prefixes, fence neutralization,
  reserved closing fence, last-`test result:` summary); `gateEvidence` in
  `stageVars` and `reviewVars`; `documentSize`/`docFileFor` generalizing the
  proposal meter, naming the metered document; over-cap as a normal stage
  failure at the meter point (not the sticky plan stop, per Options);
  `sessionTokens` into the transcript's `- tokens:` line.
- `report.ts`: `proposalGrowth` aggregates only records whose metered document
  is the design one, so the `proposal:` line counts what it labels.
- `flows.ts`/`stages.ts`: `FLOWS.full.docStages` gains `research`,
  `FLOWS.full.config.docMaxLines = 900`, `validateDocMaxLines` mirroring
  `validatePlanMaxLines`; the light flow leaves it absent.
- `independence.ts`: one own-report identity (`OwnReport` = design directory +
  report key, canonicalized separators) shared by the read branch and the
  shell branch; transcripts access is never exempted; a command naming own
  report and a sibling fails closed; a prefix-less `reviews/…` spelling fails
  closed; the checkpoint name is a run-record segment for every stage
  (deviation 8).
- `.opencode/agents/`: run-record block (3 shell denies — `*transcripts*`,
  `*reviews*`, `*pipeline.json*` — plus 3 `glob`, 2 `grep` and 1 `read` deny,
  deviation 8) on all seventeen shell-holding agents, per-seat `git diff
  *reviews/<phase>/<specialty>-*` re-open last on the ten review seats, the code
  seats' check block copied to the five design seats, and the star-form
  inspection set (no toolchain allow) on the five document families.
- Prompts: `code-review.md`, `implement.md`, `fix.md`, `fix-tests.md` carry
  `{{gateEvidence}}` and the "Gate posture" build-note sentence;
  `design-review.md` keeps the "never browse to verify one" and
  listing/globbing/grepping sentences with the browse bar scoped to locating
  paths (deviation 14) while adding the read-only check legs and the own-report
  re-read; `research.md` names its read set and the 25-read ceiling.
- `docs/workflows.md`: the artifact-table row for the build note's "Gate
  posture" section, the line on the bounded runner gate block in review
  prompts, the line on the permission-layer run-record denies (now naming the
  checkpoint), the widened `docMaxLines` row in the per-flow budget table, and
  the shell-rights bullet updated to name the design seats and doc families.

## As-built addendum (2026-11-13, after code review round 2 closed)

Round 2 closed with five approvals (security r2c1, correctness r2c1, testing
r2c2, compatibility r2c2, performance r2c2 — no open blocker or major). This
addendum finishes this note where later commits made it incomplete: it closes
the test round, corrects two sentences that went stale, and lists what stands
open at the branch tip. It supersedes nothing else above.

1. **The test round shipped.** The six asks of "Tests this round needs from the
   test engineer" were taken up by the test-update round, commit `c2fd15e`
   (`test: pin gate-record hygiene, token cache groups and the checkpoint
   fence`). Ask 1 → `agent-shell.test.ts`: the checkpoint run-record block —
   `cat`/`wc`/`git show`/`git log` spellings denied on every shell-holding
   agent, the `read`/`glob` denies pinned per agent, and the reverse direction
   (the run's own documents still open) pinned. Ask 2 → `independence.test.ts`:
   access values and shell text naming `.pipeline.json` are violations for
   every stage (the `git show HEAD:` spelling included); `pipeline-notes.md`
   does not match; write *content* merely naming the checkpoint is not an
   access. Ask 3 → `gate-evidence-integration.test.ts` `dirty` mode: planted
   credential lines (`_authToken=`, basic-auth URL, `token=`) and an absolute
   host path reach neither the persisted record nor the next `fix` prompt, the
   citable `test result: FAILED` line survives, and with a `repoDir` handed in
   the `cwd` comes back repo-relative. Ask 4 → `gate-evidence-render.test.ts`:
   a planted ``` run cannot close the evidence fence (exactly two fence lines)
   and the cap keeps the block balanced; two summaries in a flattened tail
   emit the last clause only, as one line. Ask 5 → `transcript-tokens.test.ts`:
   the real `TokenUsageInfo` shape contributes `cache.read`/`cache.write` to
   the totals and they reach the printed `- tokens:` line; deeper nesting
   stays skipped. **Ask 6 did not ship**: no fixture anywhere carries a
   `proposalGrowth` input with `docName: RESEARCH_DOC` (grep: `docName`
   appears in the suite only in a comment), and the research-meter test
   asserts `docLines` but not the stamped `docName` — deleting the
   `r.docName !== RESEARCH_DOC` filter keeps the whole suite green. Carried
   open as the testing seat's ratification finding 1 (minor).
2. **Deviation 14's pin sentence is stale (correctness r2c1, finding N1).**
   "`prompts.test.ts` pins the wording" was true when deviation 14 was written
   (`c3c085f`) and was made false by `c2fd15e`: the verbatim
   `toContain('never browse to verify one')` assertion was removed. What the
   test pins now is what §3 keeps — `Do not open review reports or
   transcripts` and the listing/globbing/grepping sentence — with its own
   comment stating the pre-016 phrase is deliberately not pinned verbatim.
   The prompt itself is as deviation 14 describes (kept, scoped, followed by
   the running-checks allowance); only the pin claim needed this correction.
3. **Gate posture refresh.** The last source edit in this tree is `c2fd15e`
   (14:39), after the 14:31 re-run the posture section cites. The runner's own
   gate record for the test-update stage (2026-11-13T19:39:13Z) shows all four
   legs green with the table's commands and exits — so the table itself
   stands. The npm leg's citable line has since grown: two review seats
   re-ran `npm run check` independently during the ratification passes (testing r2c2: `Start at 14:48:51`) and both report
   `Test Files  51 passed (51)`, `Tests  570 passed | 2 skipped (572)`; the
   `557 passed | 2 skipped (559)` quoted above predates the test-update
   commit. (Recorded from `reviews/code/testing-r2c2.md` and
   `reviews/code/performance-r2c2.md`; this recording pass runs under the
   repo's permission layer, whose shell allow does not cover re-running the
   legs by hand.)
4. **Open at the branch tip** — minors and nits only, each accepted by the
   approving seats, so the shipped state is legible:
   - security r2c1 1 (minor): `boundedTail` concatenates the elision marker to
     the kept span with no separator, so when the cut lands exactly at a
     line-initial absolute path the citation rule's anchored branch can miss
     that one line (host-layout disclosure only); security r2c1 2 (nit): the
     checkpoint block carries no `grep` deny though its comment claims the
     directories' rule shape.
   - correctness r2c1 (FYI): a stored tail can exceed 2,000 chars by the
     citation marker's own length (the cap is enforced at capture, the marker
     added after); the save-state tmp name (`.pipeline.json.<hex>.tmp`) sits
     outside the audit's segment regex, covered by the shell permission rule.
   - performance r2c2 1 (minor, accepted): the red-gate wall-clock cost of
     keeping all legs is still comment-priced — no `duration` on `GateLeg`;
     :2 (nit): the "nothing here may backtrack" comment overstates the
     credential patterns (worst case bounded by the capture caps).
   - compatibility r2c2 1 (minor): on a real `cargo test --workspace` tail the
     `greenSummary` last-clause rule cites the doc-test target's
     `test result: ok. 0 passed` line rather than a crate total — the citable
     artifact Test strategy #10/A18 promise, pinned only against a fixture
     whose last summary is the meaningful one; :2 (minor, carried from r2):
     `README.md:24` and `.opencode/skills/repo-practices/SKILL.md` still state
     the Node floor as `22.23.4` while `engines.node` and the CI pin read
     `22.23.5` (the gate-contract test keeps only that pair in lockstep);
     :3–6: the strategy #14 gap (below) plus the `citableCwd`
     case-sensitivity fallthrough, the 160-char slice bounding by UTF-16
     units, and the unguarded POSIX-only stub-cargo fixture legs (nits).
   - testing r2c2 1–10: the ask-6 gap above (:1), the elision-marker text
     asserted nowhere (:2), `greenSummary`'s last-summary choice cites cargo's
     least informative doc-test line on a real `cargo test --workspace` (:3),
     three sanitizer branches untested (:4), strategy #14's real-cargo claim
     vs the PATH-stubbed suite (:5), `MAX_TAIL_BYTES` eviction never
     exercised (:6), the glob/grep run-record pin loop covers 10 of the 17
     agents (:7), no self-kill signal-tail fixture (:8), the dead
     `tokens.length === 0` guard uncommented (:9), and the render-time array
     re-bound unasserted (:10).
