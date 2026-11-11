# complexity review r2

scope: `docs/designs/006-self-update-checkpoint-compatibility/reviews/design/editorial-r2.diff`
(r2 editorial confirmation pass; language-only diff over `02-proposal.md`)

verdict: approve

## Checklist walk (against the diff, not the whole document)

1. **New package/module/interface** — the diff adds none. It touches one Goals
   bullet, one §2 sentence, one §3 bullet's word order, four §4 articles, and
   Test #12's clause. No new symbol is introduced, so no `tools/` reuse question
   opens.
2. **Moving parts trace to a Goal** — unchanged by the diff. §4's rewording
   ("a live holder throws with today's message … a dead holder is cleared with
   today's warning and one `'wx'` retry, and a racing loser re-reads and
   throws") restates the same three branches that support the §1 single-flight
   premise; no branch was added or implied.
3. **A 30–50% smaller design** — no change to evaluate: the diff removes nothing
   and adds no mechanism, so the size profile I had no objection to in the
   round-1 shape stands (§2 is explicitly "rule set + fixtures, no mechanism").
4. **New dependencies** — none; the only primitives named in touched text are
   Node's `'wx'` flag and `rename` (stdlib), same as before the edit.
5. **Deferred second half** — untouched by the diff; the Deferred section still
   hands the lock-ID nonce, schema `version: 2`, multi-run concurrency and
   Windows verification elsewhere.
6. **Config knobs / single-implementation abstractions** — none introduced. The
   one naming change ("or its tmp shape, for `.pipeline.json.<8 hex>.tmp`")
   aligns the Goals wording with §1's existing "matches the `saveState` tmp
   shape", which narrows rather than broadens the matcher's described surface.
7. **Value check** — proportionate. The diff is pure editorial cost: no new
   state, no new knob, no new class. `CheckpointWriteFailure` is the only new
   type the design carries, it exists to answer ADR-006 Q3 ("what the runner
   does when the checkpoint cannot be written"), and the diff neither adds to it
   nor widens where it is caught.

## Findings

None. The five hunks preserve meaning:

- Goals Q2: "tmp pattern" → "tmp shape" — terminology convergence with §1, no
  change to what is fenced.
- §2 May refuse: "gains that sentence" → "gains that wording" — still one
  message naming "the supported version plus the ADR's own two interim
  postures"; "wording" is in fact the safer word, since the ADR postures are two
  alternatives rather than one sentence.
- §3 loadState bullet: front-loading "For a directory at the checkpoint path"
  keeps the same errno set (`EISDIR`/`ENOTDIR`/`ENOTEMPTY` alongside
  `ENOENT`) and the same seam (`state-store.ts:117–125`).
- §4: articles and commas only; the stale-clear race admission and the Deferred
  lock-ID nonce are byte-identical.
- Test #12: active → passive ("the checkpoint is never written outside the
  runner's own write path (fence untouched)") asserts the same observable, and
  the parenthetical still pins the fence.

No section cited by the complexity seat in round 1 exists (editorial phase runs
one language pass per round), so there is no approved text this revision could
have invalidated.

## FYI (no severity)

- §3, "For a directory at the checkpoint path, `loadState` names that repair
  instead of a raw errno" — the antecedent of "that repair" now sits two
  sentences earlier in the `finalizeEscalate` bullet; readability only, the
  editorial seat's call.
- Test #12's passive clause drops "the run" as agent; the test author may want
  the assertion's subject explicit (which write path is observed). Not a
  complexity matter.
