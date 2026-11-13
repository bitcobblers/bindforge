# Complexity review r1 (panel pass c0, full document)

verdict: approve

Inputs used: `02-proposal.md` (whole document, including its Revision log,
which shows r0 with no findings addressed yet — so every finding below would
be `new`), `docs/adrs/013-game-manifests.md`. No prior complexity report for
this seat.

## Checklist walkthrough

1. **Reuse vs. new packages/modules/interfaces** — evaluated. The design
   adds one format directory, one `Document` arm, one parse/serialize module
   (`manifest.rs`), and one loader, riding machinery the repo already ships:
   the closed-entry-set/version-gate/conformance-harness path (A1, A3), the
   existing `MAX_DOCUMENT_BYTES`/`MAX_NESTING_DEPTH` budgets (A6), the
   `load_action_registry` pattern (A7), and `InputId::parse` (A8). The
   `is_geometry_name` → `is_slug_name` rename is anti-duplication: it shares
   the already-shipped slug grammar (A18) rather than defining a second one.
   No pre-existing module in the cited layout duplicates `manifest.rs`'s job.
2. **Every moving part traces to a Goal / the ADR** — evaluated. The eleven
   entries map to the ADR's five named fact groups. The `fingerprint` entry —
   grammar-only and unconsumed this run — traces to the ADR Decision clause
   "game id + detected config fingerprint"; required-or-deferring it would
   contradict the ADR, and the Options section's rejection of "required" is
   sound (no config read ⇒ no bytes to hash, A15). `docked_actions`' action
   side traces to ADR-013's "hard-docked UI constraints" and ADR-015's
   refusal-explanation use. The budgets trace to the A6 gap. No orphan found.
3. **A 30–50% smaller design that still meets the Goals?** — evaluated.
   Sketching cuts: dropping `fingerprint` or the constraint entries violates
   the ADR's Decision clauses (not a legitimate shrink); the two-column row
   with first-space splitting is already the minimal encoding that survives
   spaced names (alternatives — separator chars, quoting — were considered
   and are larger or unsound per A13/A19); fixture count follows the pinned
   per-rule invalid-fixture precedent (A5). No smaller design meets the
   stated Goals.
4. **New dependencies** — none: std-only, `Cargo.toml`/`Cargo.lock` byte-
   unchanged, no CI change. Confirmed by the delta inventory and the
   compatibility matrix toolchain row.
5. **Could the second half be deferred to keep the first slice thin?** —
   evaluated. The design already defers aggressively (game data, CI matrix,
   preset contents, codec semantics, vocabulary widening, action-level
   exclusivity, hub transport). The shipped remainder is one indivisible
   contract slice: per A3 the corpus only "makes the format supported" when
   the code arm exists, so schema-plus-code must land together; constraint
   entries in v1 avoid a near-immediate `manifests/v2`.
6. **Knobs without consumers; abstractions with one implementation** —
   evaluated. The four budget constants each bind a named refusal path with
   a fixture/test; `MAX_MANIFEST_ROWS` is reused across three arrays rather
   than triplicated. `ExclusivityGroup` is a one-field wrapper, but it names
   the 2..=16-members invariant the API needs; not worth flagging. The
   `codec` entry has no implementation set until ADR-017 — but the ADR
   itself says "This ADR reserves the id space," so recording the id is the
   ADR-mandated minimum.
7. **Value check** — the complexity looks proportionate. One new format
   riding already-paid-for contract machinery, no new dependencies, no new
   error variants, no CI change, and an unusually disciplined set of
   deferrals each with a named owner. Nothing in the design exceeds what
   ADR-013's Decision requires for v1.

## Findings

(none)

## FYI (no severity)

- Chosen design → "The contract" — "each at most 63 octets — the same
  DNS-label bound RFC 1035 §2.3.1 publishes and the action-id grammar already
  borrows": this fifth cap is enforced at parse but is not one of the four
  exported budget constants, while Rollout 4 says the README budgets
  paragraph "names the four new constants"; an API/public-surface seat may
  want the slug cap named alongside the others (or pointed at the existing
  action-ids bound).
- Chosen design → example manifest: the inline note that "`ui.inventory` must
  be in `registry.toml` before the manifest merges" while the example also
  shows `docked-actions = ["ui.inventory Tab"]` using `Tab` — the example is
  internally caveated as "not shippable"; whether the caveat fully covers the
  docked row is a correctness-seat question.
- Options → "CI re-validation now, or its inputs now and its matrix later":
  the Deferred entry admits the scheduled-job side "has no follow-up yet and
  needs one before the first pinned profile ships" — tracking that gap is
  another seat's call, not a complexity finding.
