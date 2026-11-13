# correctness review r2

verdict: approve

Scope: `docs/designs/016-harness-evidence-and-read-rights/reviews/design/editorial-r2.diff`
(the single editorial language pass of round 2), read against `02-proposal.md`
and `docs/adrs/016-harness-evidence-and-read-rights.md`. Previous correctness
findings: none.

## Checklist application

1. **Assumption register** — untouched by the diff; the assumptions anchoring
   the changed passages (A16/A17 permission grammar, A18 cargo-test summary
   line) are cited with bases I re-checked this pass (see 2).
2. **Source spot-check (anti-hallucination gate)** — fetched
   `https://opencode.ai/docs/permissions/`: live V2 page; confirms last-match-wins,
   `*` spanning `/`, a trailing ` *` matching the bare command, `grep` matching
   the requested regex not the search path, `glob` matching the requested
   pattern, `read` resources Location-relative or canonical absolute, `\`→`/`
   normalization with case-insensitive Windows matching, and scanner-produced
   raw shell text with compound commands scanned leg by leg — exactly A16/A17.
   Fetched `https://doc.rust-lang.org/book/ch11-01-writing-tests.html`: live;
   shows the `test result: ok. N passed; …` trailing summary — supports A18 and
   §2's tail design. No dead or contradicting source.
3. **Internal contradictions** — none introduced. Every hunk is grammar,
   punctuation, or word choice with the technical claim held: "spawning failed",
   "except that", "wrapper, so", "exactly as the boolean does today", "going
   over length", "become replacement characters", "pick up the gate record".
   The one substantive edit, "the deeds audit" → "the deed audit" (§3), removes
   a name variant and now quotes ADR-007/ADR-016 verbatim ("The deed audit
   stays the backstop, exactly as ADR-007 says").
4. **Edge cases / interfaces** — the diff changes no mechanism, name, or shape
   (§1 identity rule, `GateEvidence`/`GateLeg`, the run-record block, the
   per-seat `git diff` allow all textually intact); nothing to re-derive.
5. **Decides what the ADR asked** — unchanged; the Context rewording now reads
   closer to ADR Decision 3's two-clause split.
6. **Prior rounds closed for real** — the diff does not touch or dilute any
   r1/r2 revision-log entry (whole-token shell operand, line-selective
   build-note citation, byte-identical star-form allowlists, `not exercised`
   platform rows, `>=22.23.5`, 900-line cap); the editorial pass left the
   r2 wording intact while fixing its grammar.
7. **Deferred section** — not touched by the diff; no deferral contradicts the
   ADR's Decision or Consequences.

## Findings

None. The editorial diff is language-only, introduces no contradiction, and
breaks no previously approved text.

## FYI (no severity)

- The diff's two added `Editor notes:` entries ("seat-board clone problem A7
  exists for" in Options; "a sibling names the child session" in Test
  strategy #3) flag genuine ambiguity in pre-existing text and correctly leave
  it to the author rather than guessing technical meaning — language
  clarification for the editorial/author loop, not a correctness defect:
  neither passage is made false by current text (#3's semicolon split keeps
  both assertions consistent with §1's identity rule and A3).
