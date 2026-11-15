---
status: proposed        # proposed | accepted | superseded
date: YYYY-MM-DD
deciders:               # optional, humans accountable for the decision
  - <name>
supersedes: —           # optional, ADR number
ledger: yes
---

# ADR NNN: <short imperative title>

Numbering is shared with `docs/designs/NNN-<slug>/`: an ADR that commissions a
pipeline run and the run's design artifacts carry the same NNN.

## Context

Why this decision is needed: the forces at play, constraints, prior state of
the system. One or two paragraphs.

## Decision

What we are doing, stated in the active voice ("We will ..."). Include the
concrete contract: file layouts, interfaces, configuration, and rules that
follow from the decision.

The decision is recorded as a **decision-unit ledger**: each unit is one
bounded choice plus a `→` pointer to the normative text that carries it.
Units group into kind sections below; the commented skeleton in each starts
a section.

**Unit handles.** The convention this template states, one claim per line,
each line ending with the handle of the unit that governs it:

- The `Decision` section of a product ADR is a decision-unit ledger: units
  grouped in kind sections — `C` contract, `L` law, `B` budget, `R`
  reservation, `S` seed — ids `NNN-K<n>`, each one bounded choice plus a
  `→` pointer to its target. `021-C1`
- A handle is one trailing code span on a line, holding the unit ids that
  govern that line's claim, separated by single spaces; an id anywhere else
  on the line governs nothing. `021-C2`
- Harness ADRs adopt the full convention at their next amendment; a product
  ADR is ledger-bearing at birth from this template. `021-C3`
- A unit never re-specifies: an entry is a unit only if deleting it from
  the spec loses no normative content; otherwise it belongs in the spec.
  `021-L1`
- The join is total: every handle resolves to a unit, and every unit
  carries at least one handle, except a reservation marked `unassigned`.
  `021-L2`
- Replay order is file-number order: where a `depends-on` key is present,
  `depends-on ⊆ smaller numbers`. `021-L3`
- Amendments never renumber: a later ADR adds a unit with
  `amends NNN-K<n>`; the target unit gains `amended-by`. `021-L4`
- Every reservation names an owning ADR — existing or reserved — or this
  ADR's own run, or carries an explicit `unassigned` marker. `021-L5`

### Contracts

One unit per contract the ADR adds (a format, path, or API). The skeleton:

<!-- - **NNN-C1** The bounded choice, stated with its bounds. → spec: <section>. -->

### Laws

One unit per behaviour that holds forever. The skeleton:

<!-- - **NNN-L1** The law, stated so a violation is a named refusal. → spec: <section>. -->

### Budgets

One unit per named numeric cap. The skeleton:

<!-- - **NNN-B1** `MAX_<NAME> = <n>` — what the cap bounds. → spec: Budgets. -->

### Reservations

One unit per deferred slot, naming its owner. The skeleton:

<!-- - **NNN-R1** The deferred slot. Owner: ADR-NNN (reserved). → ADR-NNN -->

### Seeds

One unit per data content the ADR ships or schedules. The skeleton:

<!-- - **NNN-S1** The seed content, and where it lands. → spec: <section>. -->

## Alternatives considered

What else was on the table and why it lost. Keep each to a paragraph.

## Consequences

What becomes easy or hard once this decision is in force: costs, maintenance
burdens, follow-on work this enables or forecloses.
