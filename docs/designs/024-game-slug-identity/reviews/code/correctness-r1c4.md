# Code review — correctness seat — round r1, dissent pass c4

**Verdict: approve** (0 open blockers, 0 open majors; round 1's blocker and
minor are both discharged; 1 new minor and 1 new nit, introduced by this
revision's prose, neither blocking.)

## Scope and inputs

- Diff under review: `git diff 2d0bfff1892322ca8bae27013ea0ae88413f9cb8..HEAD --
  ':(exclude)docs/designs'` — 6 files: `crates/core/src/placement.rs` (bound
  check in `load_geometry` + three doc comments rewritten),
  `crates/core/tests/ledger_annotations.rs` (new, 250 lines),
  `crates/core/tests/placement.rs` (geometry pin flipped from load to refusal),
  `crates/core/tests/policy.rs` (comment), `docs/adrs/013-game-manifests.md`
  (+1 line), `schemas/README.md` (bound bullet rewritten).
- Inputs read: that diff; the full current `02-proposal.md` (including its
  Revision log "Escalation repair" entry and Test strategies #13/#14);
  `docs/adrs/024-game-slug-identity.md`; `docs/adrs/021-decision-unit-ledger.md`
  (021-L4) and `docs/adrs/template.md` for the annotation format; the runner's
  gate record; this seat's round-1 findings. Surrounding source opened only to
  verify this revision's claims: `crates/core/src/{manifest.rs,document.rs,
  action.rs}`, `schemas/keyboard-geometry/v1/spec.md`, `schemas/README.md:126-145`.
  No review report or transcript was read, listed, or grepped.
- Gate record (2026-11-18T21:37:49Z, green) plus this seat's own runs at HEAD:
  `cargo test -p bindforge-core --test ledger_annotations` → 4 passed, 0 failed;
  `--test placement` → 21 passed, 0 failed. The new ledger gates are green with
  the annotation in the tree.

## Findings from round 1 — status

| Round-1 finding | Status at this revision |
|---|---|
| F1 (blocker) — `013-L5` ledger amendment never lands | **Discharged.** The annotation landed at `docs/adrs/013-game-manifests.md:117` (`amended-by 024-B1 (ADR-024, per 021-L4).`) as a continuation line inside the `013-L5` unit entry — 013's prose and `013-R1` untouched, as the proposal requires (`02-proposal.md:239-241`). Format matches `021-L4` and `template.md:53` ("the target unit gains `amended-by`"), and it names `024-B1` per ADR-024 Rollout 5. It is now gate-pinned, not convention: `crates/core/tests/ledger_annotations.rs::the_013_l5_unit_line_carries_the_amendment_annotation` fails the workspace suite without the string; this seat ran it green at HEAD. The runner landed the commit (`c4c849b8`, "operator escalation repair"), exactly the stage-boundary action this seat prescribed. |
| F2 (minor) — accepted `024-B1` and the corpus stated two scopes | **Discharged by reversal.** The escalation repair chose the opposite reconciliation from this seat's fix — instead of annotating `024-B1` down, the design (current `02-proposal.md:121-128`, Revision log :410-415) ordered the narrowing reversed: `load_geometry` now checks the name against `MAX_SLUG_OCTETS` beside the grammar check, before the join (`crates/core/src/placement.rs:168-179`), and the README bullet and constant doc state the full rider set with no narrowing clause. The accepted unit's enumeration and the enforced scope are now one scope. Withdrawn; one prose overreach this rewrite introduced carries on as F-new-1 below, at minor. |
| Nit — test comment lacked the `DF-024-1` handle | **Moot.** The pin was rewritten to the refusal (`crates/core/tests/placement.rs:323-395`); `DF-024-1` is realized, no citation of it remains in `crates/` or `schemas/` (whole-tree pattern search at HEAD returns none). |

## Checklist

| # | Item | Result |
|---|---|---|
| 1 | Named components/interfaces exist and match | Pass. `load_geometry` bound check (proposal :121-123, Rollout 1), `ledger_annotations.rs` (Test #13, ledger section, Rollout 3), the `013-L5` annotation (Rollout 3 / ADR Rollout 5), and the re-pinned geometry case (Test #14) all exist and match the proposal's names and places. |
| 2 | Assumption register honored | Pass. A1 (`is_slug_name` carries no length bound internally) still true — the predicate body is unchanged; the bound rides beside it at the call sites, as every rider site now does (`manifest.rs:532-551`, `document.rs:1232-1243` and `:1262-1281`, `policy.rs:57-65`, `action.rs:298,312`, `placement.rs:168-179`). |
| 3 | Deviations declared | Pass within the inputs allowed this pass: the reversal is declared in the proposal's Revision log and Chosen design. (`03-build-notes.md` is not in this pass's input list, so its registers were not re-verified — evidence limit noted under item 5, not a finding.) |
| 4 | Scope creep | Pass. Every hunk in the diff answers a named proposal item (Test #13/#14, Chosen design "Shared grammar and bound" / "Specs, fixtures, ledger", Rollout 1/3, and the Revision-log escalation entry); the `policy.rs` hunk is comment alignment with the same rider set. |
| 5 | Build-note claims true | Not re-verified this pass (file outside the caller's input list); nothing in the diff contradicts the claims quoted into this seat's prior findings. |
| 6 | Error paths fail closed | Pass. The bound refusal precedes the join — no filesystem access, no path named, `InvalidValue` with the number leading (`placement.rs:168-179`); pinned by the re-written test's no-path, no-content, one-bounded-printable-line assertions (A11), and the at-bound name still loads. |
| 7 | Prior blocker/major addressed | Pass. The third-pass blocker landed and is now gate-enforced; no prior major was open. |

## F-new-1 — MINOR — the rewritten `is_slug_name` doc states a universal the corpus does not support

- **Location:** `crates/core/src/placement.rs:90-93` (with `:87-88` and
  `crates/core/src/document.rs:1680-1686`; related prose at
  `schemas/README.md:131-135` and `schemas/keyboard-geometry/v1/spec.md:45,93`).
- **Problem.** The rewritten comment closes "…so the bound applies wherever the
  grammar does", but the same comment two lines earlier names an application of
  the grammar where the bound demonstrably does not ride: "applied to a
  document's `geometry` entry" — `parse_keyboard_geometry` checks that entry with
  `is_slug_name` only (`document.rs:1681`), through the public `parse`/
  `parse_file` seam. That asymmetry is deliberate and right — the un-amended
  `keyboard-geometry/v1` entry table (`spec.md:45`) states no length for the
  `geometry` entry, ADR-024's amendment list (`024-B1`) names the other four
  specs, and `024-L3` rides tightenings on a schema major — and via
  `load_geometry` it is behaviorally sound (the name is capped, the entry must
  equal the capped stem). But the universal clause is false as stated, and its
  mirror in the README bullet ("The bound rides beside the grammar at every one
  of these ids", `schemas/README.md:131`) is carried only by its own grounding
  clause about the pre-join check. This is the same class of claim-vs-scope
  finding as round 1's F2, in the opposite direction: the previous comment
  stated the narrower scope explicitly; this revision universalized it.
- **Fix.** Prose only: qualify the clause to the load seams the design actually
  mandates — e.g. "so the bound rides beside the grammar at every load seam that
  takes one of these ids — the document's `geometry` entry is guarded by the
  grammar and held to the capped name by the file-stem agreement" — or simply
  drop the universal clause; the sentence the proposal mandates (rider set
  exactly as `024-B1` enumerates, no narrowing) is unaffected either way. No
  code change is sought: bounding the entry check inside `parse` would newly
  refuse documents `keyboard-geometry/v1`'s published table accepts, the exact
  `024-L3` violation the earlier deferral was built to avoid.

## F-new-2 — NIT — the README bullet and the constant's doc comment are no longer verbatim

- **Location:** `schemas/README.md:131-135` vs `crates/core/src/placement.rs:71-74`.
- **Problem.** The proposal requires the bullet's rationale text to be copied
  into the constant's doc comment verbatim (`02-proposal.md:214-215`); this
  seat verified that identity in round 1 and c3. This revision rewrote the same
  sentence in both places with a deliberate lexical split — "the reference
  implementation checks a geometry name against it beside the grammar check" vs
  "`load_geometry` checks a geometry name against it beside the `is_slug_name`
  check". Semantically identical, and a Rust doc naming the concrete function is
  natural, but the one-copy property now holds only up to this sentence.
- **Fix.** Align one copy to the other in the next edit that touches either (or
  have the README adopt the function-named spelling), restoring the verbatim
  property the design states.

## Why this pass approves

The blocker every prior pass withheld on has landed in the ledger, in the
`021-L4` shape, and is now enforced by the workspace suite itself — a future
branch that amends `013-L5` and forgets the annotation fails the gate. The
geometry-name rider is enforced exactly as the accepted `024-B1` enumerates it,
before the join, with the refusal shape A11 pins; both suites this seat re-ran
are green at HEAD. What remains is one overreaching sentence in a doc comment
and a one-clause copy drift — prose for the next touch, not gates for this
branch.
