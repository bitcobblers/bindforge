# compatibility review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (single editorial language pass, round 2).
Inputs: `02-proposal.md`, `docs/adrs/016-harness-evidence-and-read-rights.md`, the
Revision log, this seat's prior findings (none — editorial is one language pass per round).

## Checklist walk (compatibility)

1. **Compatibility matrix** — untouched by this diff. Every row carries a status
   (`yes` / `not exercised`); no row is empty and no row says `unknown`. The
   `not exercised` posture for macOS and Windows/WSL2 is the r1 revision already
   logged ("Compatibility matrix + Rollout (r1)"). The only diff-adjacent matrix
   claim — the Context summary's reworded "(3) the document families get a
   read-only shell allowlist and both review panels get the project's check
   commands" — matches the matrix's Linux-only posture and ADR-016 Decision 3.
2. **OS input APIs** — not applicable to this harness-only change (no product,
   no `crates/`); the design targets Linux and justifies non-claims per platform
   in the matrix Notes. Diff changes nothing here.
3. **Node version floor** — unchanged (`engines.node >= 22.23.5`, Rollout 2).
   The diff touches §2's spawn wording ("null when spawning failed or a signal
   killed it") and test #1's "bytes that are not valid UTF-8 become replacement
   characters" — both describe behavior available well below the floor
   (`stdio` pipe arrays, non-fatal UTF-8 decode); no API above the floor is
   introduced.
4. **Native modules** — none; the diff introduces none. The "deeds audit" →
   "deed audit" and "appended like" → "appended as" edits are terminology/grammar
   only; the "deed audit" spelling now also matches ADR-016 Decision 3 verbatim
   ("The deed audit stays the backstop, exactly as ADR-007 says"), a consistency
   improvement, not a semantic change.
5. **Keyboard reality** — not applicable (no input capture in this design).
6. **File paths** — §1's identity rule keeps its "normalized the way the
   permission layer normalizes paths (A17: `\` to `/`, case folded)" wording; the
   diff hunk there changes only "fails as today" → "fails as it does today".
   No string-concatenation path handling is introduced.
7. **Platform test per supported platform** — the matrix claims `yes` only for
   Linux as a platform; test strategy #14 ("Platform — exercise everything
   above on Linux") still names it. The diff's edits inside #1/#3 (grammar, a
   comma splice fixed to a semicolon) do not narrow what #14 pins.

## Findings

None. Every hunk in the diff is language-only: no changed sentence alters a
platform claim, the Node floor, the packaging story, path handling, or the
matrix/test pairing. The two revision-log Editor notes appended by the diff flag
ambiguities for the author but change no shipped text.

## FYI (no severity)

- Editor note on Options considered — "the seat-board clone problem A7 exists
  for" — is about checkpoint bounding posture; that is performance/complexity
  territory, not compatibility.
- Editor note on Test strategy #3 — "a sibling names the child session" — the
  semicolon in the diff fixes the splice but the clause's expected behavior is
  still unstated; that reads as a correctness/clarity concern for another seat,
  and it is already flagged for the author in the revision log.
