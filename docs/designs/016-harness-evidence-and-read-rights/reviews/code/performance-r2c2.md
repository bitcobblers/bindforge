# Code review — performance seat, round 2 ratification (c2)

Pass: ratification c2. Scope: `git diff 6a5c454827e1ee63b910e3c937b114129784d3c8..HEAD -- ':(exclude)docs/designs'`, read against the full current 02-proposal.md and ADR-016. Question: did the post-approval fixes (`c3c085f` sanitize gate evidence / fence the checkpoint / name metered docs; `c2fd15e` pin gate-record hygiene / token cache groups / checkpoint fence) break anything this seat approved at r2? Not a fresh full review.

## Verdict: approve

## What the fixes change on the performance surface

Checked each changed hunk against the six checklist items:

- **gates.ts `boundedTail` (gates.ts:335-370)** — the only gates.ts commit in scope; `checkGate` (gates.ts:479-495) is untouched, so the r2-approved cost profile (no cargo short-circuit, serial expensive legs, parallel npm legs) is exactly what I approved. The fixed-point loop is bounded at 8 iterations, O(1) per iteration; the `Array.from(text)` scan existed identically pre-fix. Caps unchanged: `MAX_GATE_TAIL_CHARS = 2_000`, `MAX_GATE_LEGS = 32` (gates.ts:309-310) — the r2 static verification holds.
- **runner.ts write-time sanitization (`citableTail`, `citableCwd`, unshipped)** — one split + ≤5 bounded regexes per line per leg, at gate write (once per gate run) and re-run inside `gateEvidenceText` per prompt render. Upper bound: 32 legs × 2,000 chars ≈ 66 KB of regex scanning per render — negligible, no hot path.
- **runner.ts render changes (`greenSummary`, `defuseFences`, fence reservation)** — strictly cheaper for prompts than the approved r2 state: a green leg now contributes at most a 160-char clause instead of a `[^\n]*` match that could span the whole 2,000-char flattened tail; block cap stays 6,000 with the fence reserved (reserve 96→99, backstop slices around the fence). Token cost per render goes down, never up.
- **prompts.ts `sessionTokens` one-level flattening** — bounded by session message count; adds dotted keys (`cache.read`, `cache.write`) to the transcript tokens line. This strengthens cost observability (Decision 4) rather than taxing it.
- **report.ts `proposalGrowth` docName filter, state-store `docName` field, independence.ts `STATE_FILE_SEGMENT`, runner `meteredDoc`** — O(#results) / one extra linear regex per audited part / one `docFileFor` call instead of an inline literal. No added fs or git calls (the "one read, no second `git` call" comment is accurate — verified at runner.ts:1722-1730).
- No new caches, registries, arrays, or boot-path reads. No O(n²)-unbounded structures.

## Checklist item 6 — timing notes run this pass (r2 gap closed)

At r2 I could not run anything; this pass the seat shell accepted the check commands, so the capture-budget claims are now measured, not just statically pinned:

- `npx vitest run tests/pipeline/gate-evidence-render.test.ts gate-evidence-integration.test.ts transcript-tokens.test.ts` → **3 files, 19 tests passed, Duration 1.52s** (this seat's run, 2026-11-13).
- `npm run check` (full orchestration suite) → **51 files, 570 passed | 2 skipped, Duration 7.27s** — matches the runner gate record's green leg.
- Cargo legs from the gate record (2026-11-13T19:39:13Z, green): test-leg summaries `finished in 0.00s`; a warm re-run of `cargo test --workspace --locked` returned in seconds (exact wall time not citable — composite timing commands are outside this seat's shell allow).

## Findings

1. **minor — carried, unchanged: per-leg gate durations still not recorded** (tools/orchestration/src/pipeline/gates.ts:479-495). The r2 finding stands verbatim: the red-gate wall-clock cost of dropping the cargo short-circuit is deliberate (Decision 2) but still priced only by comment, not by a `duration` on `GateLeg` or timestamps in the render header. Nothing in c3c085f/c2fd15e addressed it, and nothing in them makes it worse — `checkGate` is byte-identical to what I approved. Accept the cost explicitly with one measured line (the green-path numbers above are now citable; only the red path needs the duration field). Do not re-add the short-circuit.
2. **nit — "linear patterns only" comment slightly overstates** (tools/orchestration/src/pipeline/runner.ts:412-421 region, `CREDENTIAL_LINE_PATTERNS` + `unshipped`). The comment says "nothing here may backtrack." The credential-assignment pattern (`[\w.-]*(?:token|…)[\w.-]*\s*[:=]`) does backtrack — the trailing `[:=]` fails at every split of a long word-char run, worst case quadratic in the line. The cost is bounded by the capture caps the same module owns (tail ≤ 2,000 chars × 32 legs, once per write + once per render → milliseconds worst case, no user-facing path), so it is performance-acceptable as written. Fix (docs only): say "no unbounded-cascade patterns; worst case is bounded by the capture caps" instead of implying no backtracking. No code change requested.

## R2 findings disposition

- F1 (red-gate wall-clock, minor): **carried unchanged** — see finding 1; still approve-with-acceptance.
- F2 (capture-budget tests unevaluated by this seat, minor verification gap): **closed this pass** — the vitest suites ran green with the numbers quoted above; caps verified live as well as statically.

No blocker or major findings. The fixes improve the prompt-token posture I approved at r2 (bounded green-leg clauses, fence-safe rendering, flattened cache-usage terms) and touch none of the cost decisions that pass rested on.
