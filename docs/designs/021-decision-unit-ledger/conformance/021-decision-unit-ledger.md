---
status: proposed
date: 2026-11-14
ledger: yes
---

# ADR 021: Define the decision-unit ledger as the ADR system of record

An ADR's `Decision` section is a **decision-unit ledger**: typed, numbered
units, each a bounded choice with a pointer to the normative text that
carries it. The ADR documents the decision — why, when, in what order, on
whose authority — and the spec documents the design, always showing current
state. Reconstruction is the join: read the specs for content, walk the
ledgers for chronology, and the coverage check is the proof that nothing is
unaccounted for.

## Context

Two problems, both observed in this repo, not hypothetical.

**Chronology collides with application order.** ADR numbering is one axis
that has had to serve both record order and application order. The series
paid for it twice: 003/004 and 005/006 were *swapped* to match
implementation order (`docs: renumber`), and 012–015 were *rewritten* as
deltas against the shipped core. The series already behaves like a replay
log — but it is a replay log only a human can replay, because the normative
detail lives in `schemas/*/spec.md` and code comments while the ADR's own
`Decision` is one undifferentiated prose blob.

**The record is not a record.** The ADR-013 backfill (merged with this
ADR) re-sliced one product ADR into typed units and tagged the spec's
claims with their governing units. It found drift no reader had flagged:
the source text's "+ axis kind" clause contradicts the shipped spec; seven
decisions — the action-row grammar, the loader contract, the uniqueness
law, all four budgets — exist only in the spec, made in the run and never
written back; one reservation (manifest authoring) is deferred to nobody.
A system that cannot see its own gaps is a record of decisions, not a
record of the system.

## Decision

The `Decision` section of a product ADR is a **decision-unit ledger**. The
narrative sections (`Context`, `Alternatives considered`, `Consequences`)
stay narrative: they record *why* and are outside the reconstruction. The
ledger records *what was chosen, in what order*, and each unit points at
the normative text that carries it.

### Contracts

- **021-C1** The unit ledger. A `Decision` section groups its units into
  kind sections: `C` contract (a format, path, or API the ADR adds),
  `L` law (a behaviour that holds forever), `B` budget (a named numeric
  cap), `R` reservation (a deferred slot, naming its owner), `S` seed (data
  content the ADR ships or schedules). Unit ids are `NNN-K<n>` — the ADR's
  own number, the kind letter, a per-ADR sequence — with the grep form
  `[0-9]{3}-[CLBRS][0-9]{1,2}`. Each unit is one bounded choice plus a
  `→` pointer to its target: a spec section, or the owning ADR number for
  a reservation. → template: Decision kind sections.
- **021-C2** Unit handles. A backticked unit id at the end of a spec line
  (e.g. `013-B1`) names the unit that governs that line's claim; a line
  may carry several, space-separated. A line without a handle is
  descriptive or is governed by another ADR's unit. The spec states the
  convention in a `Unit handles` note. → template: Unit handles note.
- **021-C3** Self-dogfooding. This ADR's own `Decision` is a ledger, and
  its units point to this ADR's sections or to the rollout artifacts.
  `docs/adrs/template.md` adopts the ledger sections when this run
  completes. This ADR's narrative sections keep the legacy form —
  harness ADRs adopt the full convention at their next amendment
  (021-R1). → Rollout, item 1.

### Laws

- **021-L1** The spec is the contract; the ADR is the decision record. A
  unit records a bounded choice — a reader can know what was chosen and
  its bounds without opening the spec — and never re-specifies. The test:
  an entry is a unit only if deleting it from the spec loses no normative
  content; otherwise it belongs in the spec. → template: Unit handles note.
- **021-L2** The join is total and mechanical. Every handle in a spec
  resolves to a unit; every unit carries at least one handle, except a
  reservation marked `unassigned`. Coverage is checked by pattern, not by
  reading. → Rollout, item 3.
- **021-L3** Replay order is file-number order. Where a `depends-on` key
  is present, `depends-on ⊆ smaller numbers`; a unit may reference only
  what earlier units defined. Chronology and application order coincide by
  construction — a violation is a gate failure, not a renumbering. This
  retires the failure mode the 003/004 and 005/006 swaps paid for.
  → template: Unit handles note.
- **021-L4** Amendments never renumber. A later ADR adds a unit with
  `amends NNN-K<n>`; the target unit gains `amended-by`; unit ids are
  never reused. The spec always reads as current state; the ledger is the
  per-claim sequence. → template: Unit handles note.
- **021-L5** Every reservation names an owning ANNN — an existing or
  reserved ADR number, or this ADR's own run — or carries an explicit
  `unassigned` marker. 013-R5 is the failure mode: a deferral to nobody.
  → template: Unit handles note.

### Reservations

- **021-R1** Backfill scope: the product series is backfilled (011, 012,
  014, 015; 013 is done, on this ADR's branch) — editorial-only
  re-slices, each with a `Backfill findings` section recording any
  reconciliation against its spec, per the 013 precedent. Harness ADRs
  (001–010, 016, 020) adopt forward-only: ledger at their next
  amendment. Owner: `unassigned` — a rollout task, not an ADR slot.
  → Rollout, item 2.
- **021-R2** The gate. The pipeline's deterministic source step parses
  the ledger and checks the join in both directions before any inference
  runs — dangling handles, unregistered units, a `depends-on` that
  violates 021-L3, a reservation with neither owner nor `unassigned` —
  and finalize asserts the run's own units all resolve in the merged
  tree: the ADR is a work order with machine-verified acceptance.
  Owner: this ADR's run. → Rollout, item 3.

**Unit handles.** The handles in this ADR sit at the end of the Rollout
list items: one trailing code span of unit ids governs the line's claim, an
id anywhere else governs nothing — the convention the template states
canonically (021-C2).

Rollout:

1. the template adopts the ledger sections. `021-C3`
2. the product backfills land per 021-R1. `021-R1`
3. the gate lands per 021-R2. `021-R2`
4. 017–019 adopt at landing.

## Alternatives considered

**Fold the spec into the ADR.** The `Decision` section carries the full
contract; `spec.md` becomes a derived view. Loses: the spec is read by
humans and machines alike and is always current, while an ADR that
carries the contract must be rewritten at every amendment — destroying
the very chronological record the ledger is meant to keep. Separation
keeps both current: the spec is the state, the ledger is the history.

**Heading anchors instead of unit handles.** A unit points at a spec
heading. Fragile to editorial churn: every rename is a compliance event.
The repo already leans the other way — budgets are named beside their
adding ADR, and ADR-011's id philosophy is stable names over positional
references.

**Per-claim version tags.** A spec clause carries the version that
established it. Loses the per-claim amendment chain (021-L4) and still
needs an external index for authority; a handle carries the authority —
the ADR number — on the line itself.

**Do nothing.** The 012–015 delta rewrite is the status quo, and the 013
backfill shows its ceiling: the drift was found only because the units
were extracted, and the ADR still contradicts its own spec.

## Consequences

Becomes easy: reconstruction is provable — coverage is a grep; the ADR is
a machine-checkable work order — the source gate and the finalize check;
amendments are per-claim — nothing is renumbered; the backfill is a
mechanical, reviewable diff, and any drift it finds is recorded, not
silently absorbed.

Costs: every product ADR carries a ledger at writing time (the template
enforces it); runs must write decisions back into the source ADR — the
seven-units finding of the 013 backfill is the failure mode the finalize
assert closes; two files must agree, so the join check is a standing CI
cost.

Not decided here: the order in which the 011/012/014/015 backfills land
(rollout detail, 021-R1); the gate's exact error taxonomy (021-R2's
run).
