# correctness review r2

verdict: approve

Scope: ratification pass over `reviews/design/editorial-r2.diff` (single editorial
language pass, round 2). Inputs: `02-proposal.md` (post-diff text confirmed to
contain all five hunks), `docs/adrs/006-self-update-checkpoint-compatibility.md`,
the Revision log, and my previous findings (none — editorial phase).

## Findings

None.

## Checklist evaluation

1. Assumption register — untouched by the diff; no changed text contradicts
   A1–A12. The hunks' code anchors (`state-store.ts:147`, `state-store.ts:117–125`,
   `state-store.ts:186`, `runner.ts:383–404`) still match the register's cited
   bases (A8, A10, r1 tmp-shape closure).
2. Anti-hallucination spot-check — fetched the two URLs cited in the register
   (A5/A6/A12): POSIX `rename(2)` (pubs.opengroup.org) is live and lists
   `[EISDIR]` (new path is a directory, old is not) and `[EEXIST]/[ENOTEMPTY]`
   (new is a non-empty directory), supporting A5's "rename fails
   EISDIR/ENOTEMPTY when a directory sits at the target"; Node `fs` docs are live
   and state `'wx'` "fails if the path exists", the `O_EXCL` mapping on Windows,
   and that "the exclusive flag might not work with network file systems",
   supporting A6's verified/unverified split and A12's Windows-flag and
   rename-defers-to-`rename(2)` claims (fs docs link rename to POSIX `rename(2)`).
   No dead or contradicting source.
3. Internal contradictions — none introduced. Hunk 1 (`tmp pattern` → `tmp
   shape`) aligns the Goals bullet with §1's "saveState tmp shape" and Test #1/#2
   wording. Hunk 3 (`with` → `alongside`) removes the reading that the new
   errnos are handled *as* ENOENT, which would have contradicted Test #5 and the
   Risks row (the directory case must name the `rm -r` + `git checkout` repair,
   not the missing-file path); the revised text is the more faithful one and its
   "that repair" antecedent is the adjacent bullet's repair commands. Hunks 4–5
   are article/voice changes with identical behavior claims.
4. Edge cases — the errno set `EISDIR`/`ENOTDIR`/`ENOTEMPTY` plus today's
   `ENOENT` is preserved verbatim in hunk 3; no case dropped.
5. Interfaces — no mechanism renamed; the diff only unifies "tmp shape" naming.
6. ADR coverage — the diff touches no decision text; the three ADR questions
   remain answered and the refusal-message postures still mirror ADR-006:55–57;
   nothing re-opens ADR-006's exclusions.
7. Prior closures — Revision log entries (five r1 closures) stand; the diff
   reopens none of the deliberately-rejected items (lock-ID nonce, digest
   option, relocation, v2 ladder) and breaks nothing the closures established.

## FYI (no severity)
- None.
