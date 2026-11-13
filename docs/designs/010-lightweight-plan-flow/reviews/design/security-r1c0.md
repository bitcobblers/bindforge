# security review r1

verdict: request-changes

Scope: whole document (panel pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/010-lightweight-plan-flow.md`, the Revision log. No prior findings for this seat.

## Findings

- [major][new] Options considered ("Seats in `PipelineConfig`") + Chosen design, Budget
  resolution — "`funnelSpecialties(phase, cfg)` and `states.ts`'s `specialtiesFor(phase, cfg)`
  read `cfg.reviewSeats` and fall back to the five specialties when absent" — the review panel,
  which is the pipeline's audit gate, becomes unvalidated config data. `configFor` spreads
  `base` (A4's `opts.cfg` / CLI seam) **before** `FLOWS[flow].config`, and `FLOWS.full.config`
  is documented as "an empty partial", so for a full run a caller-supplied `reviewSeats` *is*
  the effective panel, with no validation anywhere in the design: `reviewSeats.code: []` makes
  `seatsConverged` vacuously true (A2 lists it as a consumer of the seat list), so
  `case 'code-review'` converged → `doc-sync`/`done` with **zero** review sessions, no
  escalation, and a board that prints no dissent; `reviewSeats.code: ['corectness']` reaches
  seat launch with a name no agent backs. The empty-list form is one this design itself
  canonicalises (`reviewSeats.design: []`), so "empty list = this phase has nobody" is now a
  legal value in the same field that carries the code panel. Nothing bounds `planMaxLines` /
  `planMaxFiles` either. Fix: put the validation in the documented `configFor` contract and
  replace the bare "fall back to the five specialties when absent" clause with a
  presence-based read plus rules: every seat name must be one of the five known specialties;
  the seat list for every phase the flow runs must be non-empty (`design: []` is legal only
  because the registry's `stages` declares no design phase for `light`, not as a tunable);
  `planMaxLines`/`planMaxFiles` must be positive integers; a violation throws at the same
  pre-lock posture the bad `flow` value gets. Add Test strategy entries: `configFor` rejects an
  empty code panel for a flow that runs code review, rejects an unknown specialty name, and
  `flow: light` + a `base.reviewSeats` keeps the registry's two seats (registry wins over base).

- [minor][new] The `plan` stage — "`mode: primary`, edits `docs/designs/**`, denies
  `.pipeline.json`" — the plan's write surface is specified only as agent permissions, and the
  design's own reasoning says that is not the gate: "the gate is code — agent permissions and
  skill text are advisory (A13)" (Options, second entry). The Ownership section's deltas name
  `implement`/`fix`, "test-engineer stages and document stages", and `stagePathAllowed` keys on
  the stage **name** (A8), so the new name `plan` appears in no stated code rule; Test strategy
  #18 pins only `plan` writing `src/**`, and Test strategy #7's reserved-name enumeration
  ("`implement`, `fix`, `test-authoring`, `test-update` and document stages") does not name
  `plan`, whose designated directory is exactly where `.pipeline.json` sits. Fix: one clause in
  Ownership — `plan` is governed by the document-stage rule in **both** flows (writes confined
  to the run's `docs/designs/NNN-<slug>/` document paths, `namesReservedSegment` first) — and
  add `plan` to Test strategy #7's reserved-name list (`.pipeline.json`, the lock,
  `ESCALATION.md`, a `saveState` tmp) for both flows.

- [minor][new] The `plan` stage (Oversized-plan gate) + Risks — "a missing section counts as 0
  items (a faithfulness concern for the seats, not a gate)" — the file gate fails open in
  precisely the case it was added for. The gate is an exact heading string plus a bullet count,
  so `## Files to Touch`, `## Files to touch:`, a nested/indented list, or a numbered list all
  count 0 and "80 lines naming 20 files passes a line count with room to spare" also passes the
  file count silently; the section the gate reads is the only signal that section carries, and
  the two remaining seats are `correctness` and `testing`, not design reviewers, so the
  faithfulness hand-off does not cover it. The mitigation row "planMaxFiles catches
  files-spanned" is then not true as written. Fix: make the matcher canonical (case-insensitive
  exact heading after trimming; items = `-`/`*`/numbered entries at any indent under the
  section) and treat a missing section as a **denial** with its own escalation string
  (`plan names no Files to touch section: set flow: full and re-run with --force`), replacing
  "counts as 0 items" — `prompts/plan.md` renders the section, so its absence is a contract
  violation, never a legal plan.

- [nit][new] Selecting the flow — "`state.flow` is set in `runPipeline` immediately after
  `freshState`, so **the entry save carries it**" — the section describes two reads (the
  pre-lock validation "reads the source text right after `deriveIdentity`", then the entry
  save), and never says the stamped value is the value that passed validation. If the save
  re-parses, an edit to the source document between the two reads changes the effective flow
  after the pre-lock usage-error gate has already approved a different one. Fix: state that the
  value validated pre-lock is carried on the `deriveIdentity` result and is the sole input to
  `state.flow` — one frontmatter read per entry, no re-parse at save.

## FYI (no severity)

- Ownership is keyed on `(stage, flow)`, so the plan's `Files to touch` bullets grant no write
  paths — good: keep it that way if a later revision ever scopes `implement` from that list,
  which would turn document text into a path-authority.
- The default parameter in `stagePathAllowed(stage, path, flow = 'full')` fails **closed** for
  un-threaded call sites (a light run's legitimate test writes get reverted — cost, not a
  widening). The design names two call sites (`runner.ts:279`, `moves.ts:149-162`); the full
  call-site enumeration is a correctness-seat concern.
- `docs/workflows.md` "names the class that must not go light" is prose guidance, and the
  guardrail for wide-in-consequences work is Deferred consistently with ADR-010's Consequences
  ("nothing in the tool stops an operator from picking `light`") — not a finding under the
  deferral rule.

## Checklist coverage

1. **Trust boundaries** — classified: source document (operator), model-authored artifacts
   (`02-proposal.md`, review reports), `.pipeline.json` (runner-reserved), and the `opts.cfg`
   / CLI config seam (A4) — the last is the only unbounded input in the design (finding 1).
2. **Parsing untrusted data** — CRLF regex frontmatter scan, `z.enum` on `flow`, a line count
   and a bullet count; no code evaluation, no decompression, no schema-less structured parse of
   document text; caps live in the registry. Bounded.
3. **Path handling** — `namesReservedSegment` first and path-keyed, ownership keyed on
   `(stage, flow)`; no path is interpolated from document text, and none reaches a shell string.
4. **Secrets and PII** — unchanged surface: brief/plan text goes to model sessions as today;
   `statusLines`/`escalationMarkdown` print the flow name and effective numbers only;
   transcripts unchanged; `planner` denies reading `transcripts/**`.
5. **Dependencies** — no new dependency is named anywhere in the design (a stage, a seat list,
   a config record; `engines` floor and zod unchanged), so there is no advisory to fetch and no
   blocker of this kind.
6. **Privileges** — the coder's test-path widening is the ADR's stated accepted necessity
   (no test engineer in the flow); `planner` has no shell; the reserved-name precedence is
   asserted for both flows.
7. **Fail closed** — the pre-lock `flow` throw is a clean reject with "no lock, no design
   directory, no session" (Test strategy #17), and `version: 1` + golden fixtures pin the
   legacy read. Findings 1 and 3 are the fail-open gaps.
