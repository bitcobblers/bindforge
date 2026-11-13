# complexity review r1

verdict: approve

Value check (checklist #7): the complexity looks proportionate. Every moving part
maps to one of ADR-016's four Decision clauses, and almost all of it reuses
house machinery already in `tools/orchestration` rather than adding new: the
audit exemption rides the existing `reportKey` identity (no new options flag),
the gate bounding mirrors `boundedFindings`/`printable` (A7), `testsGreen`
becomes a wrapper over `checkGate` instead of a second gate, and the document
ceiling generalizes `proposalSize`/`planMaxLines`/`validatePlanMaxLines` rather
than minting a new gate. No new dependencies are introduced (checklist #4), the
four constants (`MAX_GATE_TAIL_BYTES`, `MAX_GATE_LEGS`, `MAX_EVIDENCE_BLOCK_CHARS`,
`docMaxLines`) each have a stated consumer (checklist #6), the Deferred section
hands the genuinely heavy items (subagent denies, machine-checked read ceiling,
per-run permission templating, status display) to future documents, and the
Options section rejects the heavier alternatives with reasons. A 30–50% smaller
design meeting the same Goals does not suggest itself (checklist #3); the
smaller variants the ADR itself considered are explicitly rejected there.
Single-branch rollout is acceptable here because every change is additive and
the parts are mutually load-bearing (§1's route needs §3's per-seat re-open).

## Findings

- [minor][new] Chosen design §3 — document-family bullet: "then trailing denies:
  `*transcripts*` and `*reviews*` over shell, `glob` and `grep` rules over
  `docs/designs/**/transcripts/**`" — this inline deny list overlaps but does not
  match the "Run-record block, appended last to every shell-holding agent (the
  five design seats, the five code seats, coder, test-engineer, the five
  document families)", which also covers the doc families and specifies glob
  denies over `docs/designs/**/reviews/**` in addition to transcripts, and grep
  denies of `*transcripts*`/`*reviews*` rather than path-scoped grep rules. Two
  overlapping, differently-worded rule lists for the same agents means the
  implementer must choose which one wins, and duplicated rule text is exactly
  what drifts — §3 elsewhere holds rule text to byte-identical reuse. Fix: end
  the document-family bullet with "...then the run-record block below" and
  delete its inline trailing-deny clause, making the run-record block the single
  statement of the denies; this removes text rather than adding any.
- [nit][new] Chosen design §2 — "Canonical shape, defined once here: `GateEvidence
  = { at: number; green: boolean; legs: GateLeg[] }`" — `at` has no stated
  consumer: the `gateEvidenceText` spec (failing legs first, green one-liners,
  the "(no gate record for this run yet)" fallback), the build-note sentence, and
  the test strategy never reference it. Every other field of the shape traces to
  a named use; this one is record weight with no reader (checklist #6's knob
  with no consumer, applied to a record field). Fix: drop the field, or state
  its one consumer in a clause — e.g. `gateEvidenceText` prints `at` as a
  "gate run at" header line, which also disambiguates evidence in a multi-attempt
  run. Either direction is one line.

## FYI (no severity)

- §2: the elision leg when `MAX_GATE_LEGS` is exceeded ("an elision note leg is
  appended like `boundedFindings` does") leaves unspecified what `command`/`exit`/
  `tail` an elision leg carries — correctness seat.
- A13 (`TokenUsageInfo` field names unverified): the per-key summation without
  naming fields is the complexity-cheap choice, but the assumption is
  load-bearing for §4 — correctness seat.
- Decision 1's realized benefit rides unverified A21 (own-report reads reached
  the audit via the `git diff*` shell route); if A21 is wrong, §3's per-seat
  re-open opens a route nothing used — correctness seat.
- Deferred entries (subagent denies, middle flow tier, served slots/KV,
  `--max-parallel`) restate ADR-016's "Not decided here" and Decision 4's
  refusal verbatim in substance — consistent with the source, not re-opened.
