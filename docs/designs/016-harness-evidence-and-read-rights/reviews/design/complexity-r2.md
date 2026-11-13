# complexity review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial language pass)
against `02-proposal.md`, `docs/adrs/016-harness-evidence-and-read-rights.md`,
and the proposal's Revision log. Specialty: complexity only.

## Findings

None. The diff is language-only: every hunk is a grammar, punctuation, or
word-choice fix (`fails as today` → `fails as it does today`; `spawned failed`
→ `spawning failed`; `appended like` → `appended as`; `except` → `except that`;
`deeds audit` → `deed audit`; `come replacement characters` → `become
replacement characters`; Oxford comma in `no --max-parallel, KV, or server
change`; comma → semicolon in Test strategy #3). No hunk adds, removes,
retypes, or re-scopes a mechanism, and none makes a section I would have
approved read differently.

## Checklist walk

1. **New package/module/interface, reuse check** — the diff introduces none. The
   only interface-adjacent text it touches is §2 ("`tools/orchestration/src/
   pipeline/gates.ts`: new exported `checkGate(repoDir)`"), unchanged in
   substance, and its reuse claims verify in-repo: `printable`
   (independence.ts:311), the bounding pattern `boundedFindings` /
   `MAX_RECORDED_FINDINGS = 40` (runner.ts:2949–2978, which really does `push`
   an elision entry — so §2's "an elision note leg is appended as
   `boundedFindings` does (A7)" is grounded, not aspirational), and
   `testsGreen` (gates.ts:297) kept as a wrapper rather than forked.
2. **Every moving part traces to a Goal** — the one substantive-looking rewrite,
   Context clause (3) ("the document families get a read-only shell allowlist
   and both review panels get the project's check commands"), still maps
   one-to-one to Goals bullet 3 and ADR-016 Decision 3. No new moving part
   appears in the diff.
3. **A design 30–50% smaller** — the diff neither adds nor removes surface, and
   the existing deferrals (per-run permission templating, `subagent` deny,
   status display, machine-checked read ceiling, middle flow tier) already keep
   the first merged slice thin. Nothing smaller is available inside this diff.
4. **New dependencies** — none touched. The only runtime prerequisites the
   document names (`engines.node >= 22.23.5`, pinned `@opencode/client
   ^2.1.4`) sit in hunks this diff does not change.
5. **Second half deferrable** — unchanged. Rollout #1's "Merged runs pick up the
   gate record, the prompt blocks, and the token lines immediately" is a phrasing
   fix of the same landing story already in the Revision log.
6. **Knobs without consumers; abstraction with one implementation** — the
   constants named in edited text keep their consumers inside the same
   sentences after the edit: `MAX_GATE_TAIL_CHARS` (capture buffer),
   `MAX_GATE_LEGS = 32` (array bound + elision leg),
   `MAX_EVIDENCE_BLOCK_CHARS = 6_000` (render cap), `docMaxLines: 900`
   (`FLOWS.full.config`). `checkGate` gains the `runner.ts` call site and
   `testsGreen` keeps its signature, so the wrapper has two real callers —
   not an abstraction with one implementation. No new knob appears.
7. **Value check** — proportionate, and this diff slightly *lowers* perceived
   complexity: "fails as it does today", "the thin `(await checkGate(repoDir)).green`
   wrapper, so its existing callers and tests keep their signature", and
   "persists the record exactly as the boolean does today" each state a
   continuation of shipped behavior, which forecloses a reader inferring new
   machinery. The 8 MiB ceiling and seat-board-clone rationale behind the
   bounded-tail choice is real code (state-store.ts:31–33: "the seat board is
   cloned into every later checkpoint and re-injected into that seat's next
   prompt"), so the Options rejection of full-log persistence still rests on
   measured repo facts, not invented ones.

## Revision log check

The five current entries (§1 whole-token shell operands, §2 line-selective
build-note citation, §3 single run-record rule set, Compatibility/Rollout
platform posture, A24 cap 600 → 900) are all r1/r2 decision entries; none of
the editorial hunks contradicts one, and none of the log's deliberate
non-changes (no per-run permission templating, no `subagent` deny, macOS/
Windows left `not exercised`) is disturbed by the wording.

## Deferred check

No Deferred item is re-opened. Nothing in the diff contradicts ADR-016's
Decision clauses or its "Not decided here" list (subagent delegation, middle
flow tier, served slot count / KV / memory layout), which the Deferred section
mirrors clause-for-clause.

## FYI (no severity)

- Editor notes (appended by this diff) flag two ambiguities for the author:
  Options considered — "raw logs blow past the 8 MiB checkpoint ceiling and
  the seat-board clone problem A7 exists for", where only the verb's scope is
  unclear (the technical claim is sound and traceable to state-store.ts:31–33
  and the 8 MiB loader cap at state-store.ts:150); and Test strategy #3 — "a
  sibling names the child session". Both are language/clarity items, not
  complexity ones — they belong to the editorial seat.
- Context (3): "…allowlist **and** both review panels get the project's check
  commands, **and** the run-record directories are denied…" stacks two `and`s
  inside a four-clause numbered list; the earlier comma form scoped the list
  more cleanly. Editorial preference.
