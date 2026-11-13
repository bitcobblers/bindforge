# Code review — performance seat — round 2, pass c0

Diff under review: `22ce12bf...HEAD` excluding `docs/designs/` (47 files,
+3957/−78; the change is all `tools/orchestration` harness code, agent/prompt
config, and tests — no product `crates/`/`schemas/` path, matching the
proposal's Non-goals, so the event/keystroke-path and startup items of the
checklist have no product hot path to apply to).

Inputs: the diff, `02-proposal.md`, ADR-016, the runner's gate record (none
for this run), my own previous findings (none — first pass of the seat).

## Checklist walk

1. **Event/keystroke path** — n/a to product code. On the runner paths the
   change touches: `sessionTokens` is one O(messages) pass over a list the
   audit already holds; `documentSize` still makes exactly one `git` call and
   `documentOutgrewFlow` judges that same measurement (no second read —
   runner.ts:1725–1730 honors the meter's "one read" rule); `gateEvidenceText`
   is one bounded pass per prompt render. No synchronous fs added to any
   per-message loop.
2. **Complexity** — no O(n²) anywhere. The audit's new identity path
   (`shellReachesReviews`, independence.ts) is linear in the command string:
   one segment test, one split, per-token `reviewsKeyOf`; `boundedTail` is
   linear in the ≤64 KiB decode window; render loops are linear in legs/lines.
   Maps/sets not needed at these tiny bounded sizes.
3. **Unbounded growth** — every new accumulator is bounded and the bounds are
   pinned: per-leg byte window `MAX_TAIL_BYTES = 64 KiB` (gates.ts:318) with a
   rolling front-drop so a chatty leg cannot make the runner hold its log;
   tails bounded at 2 000 chars at capture (`boundedTail`, keeps the END,
   surrogate-safe via `Array.from`); the leg array bounded at 32 twice —
   capture (`boundedLegs`) and write (`boundedGateEvidence`) — with an
   elision note leg; the rendered block bounded at 6 000 chars at render,
   with the keep-whole-lines loop sized to leave a 96-char reserve for the
   elision note and fence (worst case ≈ 5 954, inside the cap; the
   code-point-safe final slice is a backstop). `state.gateEvidence` is
   overwritten per gate, never appended; worst record ≈ 33×2 000 chars,
   three orders under the 8 MiB loader cap. All four constants match Rollout
   2 (`2_000 / 32 / 6_000 / 900`).
4. **Startup** — no boot-path change.
5. **Cache invalidation** — no cache introduced; the bounded record's
   write-then-render double bounding (the A7 pattern this stands in for) is
   present and covered by `gate-evidence-checkpoint.test.ts`,
   `gate-evidence-capture.test.ts`, and `gate-evidence-render.test.ts`.
6. **Cheap benchmark-like check** — could NOT be exercised: the run supplied
   no gate record, and this seat's shell allow (git-diff/inspection set)
   rejects `npx vitest`/`cargo`. Per the skill, an item I could not run is a
   finding, not a pass — see F2. The bounds themselves I verified statically
   against the pinning tests (they assert tail ≤ 2 000, legs ≤ 33 with one
   elision note, block ≤ 6 000, failing-legs-first, printable-at-render).

## Findings

### F1 — minor — red-gate wall-clock: the cargo-leg short-circuit is gone, cost unpriced
`tools/orchestration/src/pipeline/gates.ts:451–467` (`checkGate`).
The boolean-era `cargoGreen` returned `false` on the first failing cargo leg,
so a red `cargo fmt --all -- --check` (seconds) skipped the expensive
`cargo clippy --workspace --all-targets` and `cargo test --workspace --locked`
legs. `checkGate` now always runs all three serially. This is deliberate —
Decision 2's whole point is that a red gate names the failing *test*, which
requires running the test leg — and the comment at 459–460 is the only record
of the choice. But the proposal's Risks table prices only the *context* cost
of the record; the fix loop's hot path (the gate re-runs after every
implement/fix/test-update, up to `maxRounds: 5`) is exactly where gates go
red, so every red round now pays full clippy + full workspace-test time the
old gate avoided. No measurement is cited anywhere.
Fix: accept the cost explicitly — add one Risks/build-note line stating the
red-gate wall-clock change — and make it checkable: the next run's transcripts
plus a per-leg duration (one field on `GateLeg`, or a timestamp in the render
header) would turn this into a cited number instead of a debate. Do not
re-add the short-circuit; it would gut Decision 2.

### F2 — minor — checklist item 6 could not be executed by this seat
The capture-budget test the ADR names as load-bearing (the tail/legs/block
caps) exists and is pinned, but this review seat could not run it (no gate
record in the run; `npx vitest` and the cargo legs are outside the seat's
permission set). The verdict therefore rests on static reading for the
timing-sensitive claims in F1.
Fix: none in the code — this is a verification gap of the review posture. If
the runner captured the check gate's own leg durations (see F1), the gate
record handed to seats would close it for the next panel without widening any
seat's shell.

## FYI (out of specialty, no severity)

- `gates.ts:340` — `boundedTail`'s elision note computes the claimed elided
  count against a 64-char reservation while the actual note is ~45 chars, so
  the stated count overstates the real elision by ~19 (total tail length is
  still exactly ≤ 2 000 — the cap holds; only the number in the note is
  off).
- A record already at the capture bound (33 legs incl. its elision note) is
  re-bounded at write into 32 legs + a *second* elision note reading "1
  further gate leg(s) elided" — bounded and harmless, cosmetically odd.

## Verdict

approve — zero open blockers or majors; the two minors are a pricing/doc gap
and a verification gap, not defects in the shipped bounds.
