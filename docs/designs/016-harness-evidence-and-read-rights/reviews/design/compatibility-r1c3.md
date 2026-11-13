# Compatibility review r1c3 (ratification pass)

verdict: approve

Scope: the blob diff 9b6b5ed → 4e18f4f (r2 revisions), then the full current
document. This pass reports only regressions against what r1 approved.

## Checklist walk (changed text only)

1. **Matrix statuses** — the compatibility matrix is untouched by the diff;
   every row still carries a status, no `unknown`, no empty cell. The r2
   changes (Context wording, §1 shell branch, §2 build-note citation, the
   rolling-tail Risks row, Test #3, Revision log) do not touch any platform
   claim, and the macOS / Windows-WSL2 `not exercised` rows still agree with
   Rollout #2 and §3. No row was made false.
2. **OS input APIs** — no input-API surface in the design; the diff adds none.
   The r2 §2/§1 wording is harness-side parsing and prompting, host-neutral.
3. **Node floor** — `engines.node >= 22.23.5` (matrix + Rollout #2) unchanged;
   the revised §2 tail/citation mechanics use the same stdio-pipe and string
   machinery already approved in r1. Nothing in the changed text exceeds the floor.
4. **Native modules** — the diff introduces none; the "no worktree file / no
   new file" posture is untouched.
5. **Keyboard reality** — not engaged by the changed sections.
6. **File paths** — §1's revised shell branch ("whole command token ... prefix
   included ... a prefix-less token resolves outside the attempt's design
   directory and fails closed") is consistent with §1's identity rule, which
   canonicalizes per A17 (`\`→`/`, case folding); no new string-concatenation
   path handling appears. Test #3's new pinned spelling
   `git diff docs/designs/<own>/reviews/design/<specialty>-r1c1.md` does match
   §3's per-seat re-open `git diff *reviews/design/<specialty>-*` (A16's `*`
   spans `/`), and the "same path under another run ... fails" pin matches the
   Risks-table seam (permission allow wider than the audit's identity rule)
   that r1 approved — consistent, not contradicted.
7. **Platform tests vs matrix** — the only `yes` platform row remains Linux,
   still covered by Test #14; the r2 diff adds no platform claim.

## Regression check against r1-cited sections

- §3 Read-only reach and Test strategy #4 (my r1 citation) do not appear in
  the diff; their text is byte-identical to what r1 saw. Nothing in the r2
  changes makes that text false: the new §1 shell rule, §2 citation rule, and
  Test #3 pin all reference §1/§3 spellings without contradicting §3's
  byte-for-byte coder.md posture or the matrix's "§3's inspection set" notes.
- §1 and §2 (approved in r1 theme) — the r2 rewordings tighten, not invert:
  fail-closed token matching in §1 preserves Decision 1's exemption route the
  matrix and Risks rows depend on; §2's line-selective citation keeps the
  committed-carrier story the Risks mitigation and Rollout #3 (build-note
  "Gate posture" row) state consistently.

## Findings

None. No r2 revision broke a previously approved section or made a
r1-cited section false.

## FYI (no severity)

- My r1 minor (§3 inspection set defined only by reference to `code/coder.md`;
  Test #4 pin narrowed to `wc <file>`/`cat <path>`, so ADR-016 Decision 3's
  `git status`/`git diff`/`git log`/`git show` reach is unpinned) is carried in
  unchanged text. The Revision log's r1 entry records the byte-identical
  coder.md posture as the author's decision, the r2 round left the text alone,
  and the rebuttal is not unsound — A8 verifies coder.md's set by file ref —
  so it is not re-raised on this pass; drift risk remains as stated in r1.
