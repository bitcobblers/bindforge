# Complexity review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (the round-2 editorial language pass over
`02-proposal.md`). Inputs used: the diff, `02-proposal.md`,
`docs/adrs/015-transforms-as-preconditioned-policies.md`, the Revision log inside
the proposal. No earlier specialty reports were opened; this seat has no prior
findings (the editorial phase is one language pass per round).

## Findings

None. No blocker or major findings are open against this scope.

## Checklist walked (diff scope)

1. **New package / module / interface reuse** — the diff adds none. Its only
   text-touching surface is wording in Goals/Non-goals, Options considered, the
   stack checks, Deferred, plus a trailing editor note. No new moving part to
   compare against `tools/` or `crates/core`.
2. **Every moving part traces to a Goal** — nothing added, so nothing orphaned.
   The diff-adjacent parts it touches (closed kind set, per-kind verdicts, pair
   encoding, the `policy/v2` deferrals) all trace to the stated Goals or are
   listed under Deferred / out of scope.
3. **A 30–50% smaller design** — the diff does not change design size. Evaluated
   against the diff: vacuous. Nothing in the changed text makes the design larger.
4. **New dependencies** — none introduced by the diff; the crate/`Cargo.lock`
   surface it touches is unchanged prose ("std-only; `crates/transforms` depends
   only on `bindforge-core`").
5. **Deferred second half** — unchanged by the diff, and still thin: "This run
   ships no policy data"; seed collection, CLI, hub all deferred to ADR-016/017/018,
   consistent with the ADR's "Not decided here" clause. No contradiction with
   `docs/adrs/015-transforms-as-preconditioned-policies.md` found in the changed
   text — the deferral of "Preconditions composed with and/or/not, and any fourth
   transform kind" matches the ADR's fixed-vocabulary posture, and
   "`supported_majors()` is **unchanged**" is consistent with the ADR's cost that
   the format "joins the conformance corpus and `supported_majors`" because the
   shipped union is already `["v1","v2"]`.
6. **Config knobs / single-implementation abstractions** — the diff neither adds
   nor renames a knob. The two budget constants and the closed reason enum it
   paraphrases are untouched.
7. **Value check** — proportionate. This pass changes six sentences and adds one
   note; it removes more hedging than it adds ("exactly as … do today" → "as …
   already do", "answer-why" → "answer"), and no complexity is introduced. The
   design's overall weight (one new format directory, one core module, one second
   crate, two constants, one ten-variant verdict vocabulary) still traces to the
   ADR's Decision, and the editorial edits left every one of those parts sized as
   previously reviewed.

## Regression check

Each hunk was checked for text it could falsify outside its own sentence:

- "Precondition outcomes are **verdict data** on `PlacementCheck`'s channel" —
  still matches A9 and the Unchanged note "no new `CoreError` variant".
- "(Options considered)" x2 — label now names the real section; no design claim
  changes.
- "mapped one-to-one to a named verdict" — still matches the ten-variant
  `NotApplicableReason` list in The report.
- "with wrong-field-count refusals and fixtures, as `set`/`swap`/manifest rows
  already do" — dropping "exactly" weakens, not strengthens, the claim; the
  fixture list (e.g. `relocate-few-fields.toml`, `swap-polarity-extra-field.toml`)
  still supports it.
- "exactly the answer a bool cannot give (A4)" — A4's basis ("a bool cannot say
  why a placement is illegal") still holds the sentence up.
- Revision log: unchanged entries remain current.

Nothing the revisions broke, and no previously cited section was made false.

## FYI (no severity)

- Non-goals / Deferred — "both ride a future `policy/v2` (Options considered)" and
  the matching Deferred bullet now point at a section that analyses the closed
  kind set ("Growing the set rides `policy/v2`") but contains no option about
  precondition composition (and/or/not); that idea appears only as a deferral.
  Cross-reference accuracy, not complexity.
- Trailing "Editor notes: Test strategy — … is ambiguous; author to clarify" leaves
  an open author action inside the final document, and the round-2 wording changes
  are not recorded as Revision log entries. Editorial/process surface.
