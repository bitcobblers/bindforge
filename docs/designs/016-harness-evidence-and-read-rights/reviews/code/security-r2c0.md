# Code review — security — round 2, pass c0

Diff: `22ce12bf09a600d3983f374294d9d5844b07af54...HEAD` (docs/designs excluded).
Judged against `02-proposal.md` and ADR-016, with the security checklist.
No gate record was supplied; I verified persistence claims directly with
read-only git queries on the repo's own history.

## Checklist walk

1. **Boundary validation** — the new checkpoint shape is zod-validated
   (`state-store.ts` `GateLegSchema`/`GateEvidenceSchema`, `exit` int-nullable),
   the 8 MiB pre-parse size cap is untouched, `sessionTokens` sums only finite
   numbers and skips non-numeric usage fields. Pass.
2. **Path handling** — the §1 identity rule (`independence.ts`
   `canonicalPath`/`reviewsKeyOf`/`isOwnReport`) fails closed on traversal
   spellings, prefix-less tokens, and cross-design-directory same-named
   reports; the `dir.endsWith('/' + designDir)` test keeps the prefix match on
   a path boundary. I probed `..`, embedded-second-segment, suffix-collision
   and case/separator spellings: every wider spelling denies. Pass.
3. **Injection** — `runLeg` spawns argv arrays with no shell; no regex is
   built from input data (`/test result:[^\n]*/` is linear). Caveat: the
   rendered evidence fence (finding 3). Pass with caveat.
4. **Secrets/PII in committed artifacts** — finding 1 (major) and finding 2
   (major).
5. **Dependencies** — no new dependency; the only package.json change raises
   `engines.node` to `>=22.23.5` for the 2026-11-11 Node security release,
   matched in `.github/workflows/check.yml`. Justified; no advisory concern.
   Pass.
6. **FS writes** — no new write paths; the atomic checkpoint write and the
   same-`checkpoint()`-call persistence of boolean + evidence are per design
   §2. Pass.
7. **Malformed/hostile input** — spawn failures record `exit: null` with the
   reason spelled in `tail`; tails are code-point-sliced, control bytes
   stripped via `printable` at write and again at render; elision is stated,
   never silent. Pass.

## Findings

### 1. major — the gate record's raw tails and package cwds land in a committed file, without the line-selective drop the design makes its whole mitigation

- **Location:** `tools/orchestration/src/pipeline/runner.ts:2872` (write of
  `state.gateEvidence`), capture at `tools/orchestration/src/pipeline/gates.ts:385-387`.
- **Problem:** the design's Risks row and revision log r2 rest the tail
  hygiene entirely on one premise: "the checkpoint and transcripts stay local
  and the build note is the one committed carrier, holding only §2's
  line-selective citation with the credential-bearing and absolute-path lines
  dropped." The premise is false in this repo: the pass's `git add -A` commits
  the checkpoint (`STATE_FILE = .pipeline.json`, no .gitignore entry). I
  confirmed on this very run's history — `git log --name-only --
  'docs/designs/*/.pipeline.json'` lists it in stage commits `09bebd3` and
  `6a5c454`, and the version at `6a5c454` contains `gateEvidence` (5
  occurrences). So the run's git history now carries, verbatim and unselected,
  up to 32 tails of ≤2,000 raw output characters each (npm/cargo stderr — a
  category the design itself says can hold registry text and credentials)
  plus each leg's package `cwd` (an absolute host path once the repo was given
  as absolute, which is how the runner spawns). Transcripts, also committed,
  repeat the same block inside every later prompt. This is exactly the
  share/export leak path checklist item 4 guards, and the diff opened it while
  the design's stated drop rule reaches only the build note.
- **Fix:** apply the build note's line-selective rule at capture or write, not
  only at the prompt: in `boundedGateEvidence` (or before it, in `runLeg`/
  `boundedTail`) drop lines matching credential shapes (e.g.
  `_authToken`, `://<user>:<pass>@`, `token=`) and lines carrying absolute
  paths, or normalize `cwd` to a repo-relative spelling. A test on a fixture
  leg whose output carries a fake token and an absolute path must show neither
  survives into the persisted record.

### 2. major — the §3 run-record fence denies `transcripts/` and `reviews/` but not `.pipeline.json`, while the diff grants every widened family shell reach into that verdict-bearing file

- **Location:** the run-record block as shipped to every shell-holding agent,
  e.g. `.opencode/agents/research/researcher.md:71-93` (same block in the
  five design seats, five code seats, coder, test-engineer, doc families).
- **Problem:** the block is commented as "the single authority for these
  families' run-record denies," and Decision 3's premise is that the
  checkpoint holds what the fences protect: `PipelineState.seats` carries every
  seat's findings and verdicts — the exact anchoring ADR-002/007 independence
  guards — and the diff now adds the gate record beside it. Yet no rule
  mentions the checkpoint: `cat docs/designs/<dir>/.pipeline.json`,
  `git show HEAD:docs/designs/<dir>/.pipeline.json`, and `git log *` carry no
  `transcripts`/`reviews` token, so they are **allowed** by the new star-form
  inspection set, and the audit's segment regexes name no `pipeline.json`
  segment either. The repo itself already treats the path as a run record —
  every agent config denies **edit** on `docs/designs/**/.pipeline.json`
  (line 12 of each) — and ADR-016's Context calls a research child reading
  the checkpoint a real violation. The pre-diff `git diff*` allow left a
  narrow legacy route for review seats, but this diff grants full
  read-commands to five doc families that had none, enriches the file, and
  claims the fence authority; that makes the omission true of the change as
  shipped.
- **Fix:** extend the run-record block with `shell: *pipeline.json* → deny`
  and `read`/`glob: docs/designs/**/.pipeline.json → deny` on every
  shell-holding agent (none of them has a legitimate reason to read it; the
  runner owns it), add the `STATE_FILE` name to the audit's segment test as
  backstop, and pin both directions in `agent-shell.test.ts` the way the
  `transcripts`/`reviews` pair is pinned.

### 3. minor — captured output can break the evidence block's fence, weakening the only guard against gate-output prompt injection

- **Location:** `tools/orchestration/src/pipeline/runner.ts:3085-3123`
  (`gateEvidenceText` header and body loop).
- **Problem:** tails and command lines are emitted verbatim (modulo
  `printable`) inside a ```text fence; the "this is output … never
  instructions" header is the design's entire injection defense for leg
  output that the code under review — written by an earlier stage — controls.
  A tail containing a ``` line closes the fence early and puts subsequent
  captured text outside the quoted region; a tail line shaped like
  `$ <command> (exit 0)` also impersonates a leg line inside the block. The
  hard `MAX_EVIDENCE_BLOCK_CHARS` slice at the end can additionally drop the
  closing fence, leaving an unbalanced block.
- **Fix:** at render, neutralize fence sequences inside captured strings
  (e.g. replace runs of `` ` `` of length ≥3 with an escaped form, or prefix
  each tail line with a fixed continuation marker the renderer owns), and
  reserve the closing fence inside the cap so the slice never unbalances it.

## FYI (no severity, not security)

`boundedTail`'s elision note is sized from a fixed 64-char reservation, so the
stated "N characters elided" over-reports the actual elision whenever the
note's own length differs from 64. Also, `isOwnReport` is asymmetric across
spellings: an absolute handed `ownReportPath` makes a repo-relative access
value fail (fail-closed, and today's runner hands relative paths, so only the
safe direction bites).

## Verdict

request-changes — findings 1 and 2 must close; finding 3 should ride with
them. The identity rule, capture bounds, and dependency posture are sound.
