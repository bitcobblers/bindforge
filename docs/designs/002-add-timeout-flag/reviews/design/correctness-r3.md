# correctness review r3

verdict: approve

Scope: the change set `reviews/design/editorial-r3.diff` applied to
`02-proposal.md` (proposed file already contains the applied hunks). Checklist:
`references/correctness.md`. Previous correctness round: `correctness-r2.md`.

Verification performed for this round:

- Every diff hunk read line by line against the pre/post text. All seven hunks
  are language-only: the `honor→honour` spellings (Context, Goals), two comma/
  relative-clause fixes (Value semantics, Test #10), the grammar fix in the
  resolver-bullet bounds sentence, the inserted "that" in Test #6, and the
  revision-log r3 fix that deletes the verbatim-duplicated "Deliberately not
  changed this round" paragraph and completes the truncated tail
  "fallback-triggering condition". No decision, contract, anchor, or number
  changes in any hunk.
- Checklist #7 (closure): correctness-r2 raised no blocker/major. Its `[nit]`
  about the duplicated r3 log paragraph (02-proposal.md:160) is now closed for
  real — the applied file has exactly one copy, followed by the register-status
  sentence, matching the review's prescribed fix.
- Checklist #4/#5 against the changed lines: Test #6's "exactly two
  `message.list` calls" at `timeoutMs = 100` was re-traced by hand against the
  real loop at `tools/orchestration/src/opencode-runner.ts:259–271` (immediate
  check → deadline tested only after each reply check → single 250 ms sleep →
  second check → deadline hit): two calls, ≤ one-interval overshoot — the
  edited sentence is correct. `timeoutMs`/`timeoutSeconds` naming is unchanged
  and still consistent across sections. Post-diff, the document's spelling is
  internally consistent (`honouring`/`honours` align with the pre-existing
  en-GB `behaviour`/`labelled`; no `honor`/`behavior`/`hardcoded` remnants).
- Checklist #2 (anti-hallucination gate): four citation URLs spot-checked live
  this round — https://vitest.dev/blog/vitest-5.html ("Vitest 5 requires Vite
  >= 6.4.0 and Node.js >= 22.12.0", verbatim, supports A21),
  https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNECT-CONNECT-TIMEOUT
  ("in seconds (write as a decimal integer…); Zero, negative, or not specified
  means wait indefinitely", supports A6/A14/research line 23),
  https://curl.se/libcurl/c/CURLOPT_TIMEOUT.html (long seconds, last-set-wins,
  default 0, supports research line 22),
  https://www.gnu.org/software/coreutils/manual/html_node/timeout-invocation.html
  (floating-point duration + optional unit, 0 disables, exit 124, supports
  research line 25). No dead sources. One contradicting citation found —
  finding 1.
- Checklist #1/#6: no hunk states an unverified assumption as fact (the r3 log
  sentence it rewrites explicitly keeps A8/A14/A17/A19 `unverified`, matching
  the register); the source document's asks are untouched and still decided.

## Findings

- [minor] 02-proposal.md:55 (Options considered → Value semantics, changed by
  diff hunk @@ -52) — the edited sentence asserts "the prior-art human-facing
  interfaces are integer seconds (A6…)" and names "coreutils' floating-point
  durations" as *the* outlier, but the first citation of A6, the curl man
  page's `-m, --max-time <seconds>` section (re-fetched live this round), says
  "This option accepts decimal values" and gives `curl --max-time 2.92` — curl
  is a second decimal-valued prior art, so both the blanket "integer seconds"
  claim (marked `verified` in A6) and "the outlier" are contradicted by the
  proposal's own cited basis. The decision (reject fractional values) still
  stands on libpq ("write as a decimal integer"), `CURLOPT_TIMEOUT` (long) and
  250 ms poll granularity, so this weakens the citation, not the outcome.
  Carried through unchanged by the editorial diff, which touched this exact
  sentence (this was correctness-r2's first `[minor]`; it remains open).
  Fix: reword the rationale (and A6) to "seconds-valued human-facing
  interfaces (integer-typed in libpq `connect_timeout` and `CURLOPT_TIMEOUT`;
  `--max-time` is seconds-valued but accepts decimals)" and acknowledge curl
  as a second floating-point outlier.
- [nit] 02-proposal.md:126 (Test strategy #6, changed by diff hunk @@ -123) —
  the edited clause "pins the behaviour that research line 43 asks the
  designer to fix" misstates the source: research line 43 asks the designer to
  "pin down" the bounded sub-poll behaviour, and the design deliberately
  documents rather than fixes the loop ("documented rather than 'fixed'" in
  the r2/r3/r4 log entries). The design's decision is correct and pinned; only
  the attribution verb is wrong. Fix: say "asks the designer to pin down".

## FYI (no severity)

- correctness-r2's other open items (A10 basis anchor `runner.ts:326` →
  should be `:338`/`:378`; the set-but-empty env string `""` case not pinned
  in the resolver bullet or Test #8's matrix; the flag path's identical silent
  yargs numeric coercion; Test #5's "reject synchronously" parenthetical for
  an `async` function) are technical content outside this editorial change
  set — none of them is touched by the diff, so they stay for the proposal
  editor, not this review.
- The diff's `honor→honour` and `hardcoded→hard-coded` changes move the
  document toward its prevailing en-GB style; no US-spelling remnants were
  found, so no two-names-for-one-thing inconsistency was introduced (checklist
  #5 is satisfied on the prose side as well as the interface side).
- The A21 quote checked live this round is on the published Vitest 5.0 post
  dated September 2026, matching the register's "re-quoted verbatim this
  round" claim.
