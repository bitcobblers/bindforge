# Code review — security (round 2, dissent pass c1)

Scope: `git diff 6a5c4548..HEAD -- ':(exclude)docs/designs'`, read against the
full 02-proposal.md and ADR-016. Dissent pass: this pass checks that the last
revision answers my round-2 findings; new findings only where the changed code
makes them true.

## My round-2 findings — verification

### Finding 1 (major) — raw gate tails + absolute cwds reach committed history
**Closed.** `boundedGateEvidence(evidence, repoDir)` now applies the design's
§2 line-selective citation rule where the record is WRITTEN
(runner.ts:3057-3063): `citableTail` drops lines matching credential shapes
(`_auth*`, `token|secret|passwd|password|api_key|access_key|private_key`
assignments, basic-auth URLs, `Authorization: bearer|basic`) and absolute-path
lines (POSIX + Windows), stating the drop with an explicit marker instead of
dropping silently; `citableCwd` stores each leg's package dir repo-relative
(`.` / relative / `(cwd withheld)` for an absolute dir with no repo to judge
against). The only write path for `state.gateEvidence` is the sanitized call at
runner.ts:2882; `testsGreen()` discards legs; the render path re-runs the same
bounds before anything reaches a prompt or transcript. `.gitignore` has no
entry for `docs/designs/**/.pipeline.json`, so the committed-checkpoint premise
the fix rests on is current, and the drop now precedes the commit — the
channel my round-2 finding opened is shut at capture. The e2e fixture
(`gate-evidence-integration.test.ts`, `dirty` mode: `_authToken=`,
`https://ci-bot:hunter2pw@`, `token=DEADBEEF`, `/home/ci/work/bindforge`)
asserts neither the persisted record nor the fix prompt carries any planted
string while the citable `test result: FAILED` line survives. Re-ran the four
relevant suites locally: 62/62 pass. Regexes are linear (checklist 3 — the
nested quantifier in `ABSOLUTE_PATH_LINE` has no post-group failure point to
backtrack into). One narrow residual gap, reported below as minor.

### Finding 2 (major) — the checkpoint outside the run-record fence
**Closed.** All 17 shell-holding agent configs (coder, test-engineer, 5
code-review, 5 design-review, researcher, designer, planner, editor, doc-sync
— matches the test's `SHELL_HOLDING`) gained the block: shell deny
`*pipeline.json*`, read + glob deny `docs/designs/**/.pipeline.json`. The
rules sit after every shell allow and before the seat's own-report
`git diff *reviews/code/<specialty>-*` allow, so the last-match-wins order
holds (the exception cannot re-open the checkpoint). `independence.ts` builds
`STATE_FILE_SEGMENT` from the `STATE_FILE` constant (no writer/auditor drift),
judges it on shell commands and on tool access values for *every* stage, and
its dot-escaping keeps `pipeline-notes.md` out of the net. `agent-shell.test.ts`
pins both directions: four spellings (`cat`/`wc`/`git show`/`git log`) deny on
every shell-holding agent, read/glob denies pinned per agent, and the reverse
direction pins the run's own documents (`01-research.md`, `02-proposal.md`,
`03-build-notes.md`) still allow. `independence.test.ts` adds the audit
backstop plus the design-007 §1 pin that write *content* merely naming the
checkpoint is not an access. My prescribed fix is implemented in full. One
shape asymmetry with the block's own comment and the new workflows.md sentence
— reported as nit.

### Finding 3 (minor) — fence escape / leg impersonation / cap cutting the fence
**Closed.** The renderer owns the only line starts: `$ ` for legs, `| ` for
captured text, stated in the header itself; every captured tail line is
`trim()`-ed, prefix-marked, and passed through `defuseFences` (a run of three
or more backticks gains a backslash). A smuggled ` ``` ` block can no longer
close the region (fixture asserts exactly two fence lines), and a
`$ <cmd> (exit 0)`-shaped tail renders as `| $ ...` — captured, never a leg.
The cap reserves the closing fence (line loop: `reserve = 96 + FENCE.length`;
backstop slice budget `MAX − FENCE.length − 1` then re-appends `\n` + fence),
so the hard slice can never leave an unbalanced block; the fixture checks both
`length ≤ 6000` and the last line being the closing fence.

## New findings (made true by the changed code)

1. **minor** — `gates.ts:366` + `runner.ts:3090-3105`: the tail-cap note is
   concatenated with no separator (`${note}${kept}`), so when the kept span
   begins exactly at a line-initial absolute path (rustc/cargo print such
   lines), the citation rule's one-line view starts `[…]` + `/home/...` and
   the anchored POSIX branch of `ABSOLUTE_PATH_LINE` (needs `^` or
   `[\s'"(=,]` before the slash) misses it — one absolute host path survives
   into the committed record. Credential patterns are unanchored and survive
   the glue, so this is host-layout disclosure only. Fix: separate the
   boundedTail note from the kept span with a newline (or add `]` to the
   leading class, or have citableTail strip a leading elision note before
   judging the line), and extend the dirty fixture with a >`MAX_GATE_TAIL_CHARS`
   output whose cut lands at a line-initial path.
2. **nit** — run-record block shape/docs: the transcripts/reviews pair carries
   a `grep` deny; the checkpoint block omits one, though its agent-file comment
   claims "same coarse rule shape as the two directories above" and the new
   workflows.md sentence says "matching `read`/`glob`/`grep` denies". Per A17 a
   grep resource matches the requested regex, so the gap is only the
   regex-spelling route (path-targeted grep is audit-caught via
   `accessValues.grep`), but the fix is two lines: `- action: grep /
   resource: "*pipeline.json*" / deny` in each block, or reword the sentence.

## Checklist

1 (boundary validation): sanitizer at the write boundary, tested. 2 (paths):
leg cwds repo-relative, no traversal introduced. 3 (injection): fence +
prefix hardening, linear regexes. 4 (secrets): dropped at write; no secrets in
source or fixtures (fixture secrets are the negative controls). 5 (deps): none
added by the diff. 6 (fs writes): unchanged tmp+rename checkpoint write.
7 (hostile output): record survives dirty legs cleanly (fixture leg exit 1,
record persists, citable line kept).

## Verdict

**approve.** All three round-2 findings closed with tests pinning both
directions; the two residuals are minor/nit hardening, not open majors.

FYI (no severity, out of diff scope): the proposal's Risks row still says "the
checkpoint and transcripts stay local" — the fix's own comments (runner.ts:3050,
agent configs) correctly record that this repo commits the checkpoint; the
document's premise is stale even though the code now defends it.
