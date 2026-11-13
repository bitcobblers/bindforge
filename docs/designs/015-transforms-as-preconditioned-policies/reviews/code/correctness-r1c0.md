# Code review — correctness, round 1 pass c0

Seat: correctness. Diff: `1af12026893971772c5ac8384ceae348d63cd064...HEAD`,
`docs/designs/` excluded. Judged against `02-proposal.md` and
`docs/adrs/015-transforms-as-preconditioned-policies.md`. No previous-round
findings to re-check (first pass of the code panel).

## Verdict

**approve** — no open blocker or major finding. The implementation is faithful
to the design where faithfulness matters: the closed kind set, the row and tag
grammars, the two budget constants, the entry-refusal/verdict channel split, the
match rule, the sequential-and-atomic stack, the untouchable guard as loaded
data, and the produced-base rules all exist, are wired in, and carry the
proposal's names and signatures. Findings below are one contract-wording
mismatch and three small behavioral gaps at legal-but-unusual inputs, plus
nits.

## Checklist

**1. Every component and interface named in the proposal exists and matches.**
Pass. `SUPPORTED_FORMATS` gains `"policy"` (`document.rs:79-86`);
`format_majors("policy")` → `[v1]` (`document.rs:242-248`) and
`supported_majors()` is untouched (`document.rs:231-233`), so the majors-equality
invariant holds; `POLICY_V1_ENTRIES` (`document.rs:166`); `Policy` / `Transform`
/ `TagPattern` are the proposal's types with the proposal's fields
(`document.rs:366-398`, `452-459`), `TagPattern::matches` does byte-exact
prefixing (`document.rs:464-474`), `serialize` re-spells rows from typed fields
and preserves array order (`document.rs:806-818`);
`MAX_POLICY_TRANSFORMS = 64` / `MAX_RELOCATE_TARGETS = 16` are exported
(`document.rs:208`, `214`) and re-exported at the crate root
(`lib.rs:102-103`, `114`). `load_policy(dir, id)` matches the proposal's
signature and its five load rules (`policy.rs:54-85`). `apply` matches the
proposal's sextuple and returns `Applied { base, reports }`
(`apply.rs:181-223`); `TransformReport` / `Outcome` / `NotApplicable` /
`NotApplicableReason` are the proposal's report shape with the ten closed
spellings (`apply.rs:70-164`). `crates/transforms` depends on
`bindforge-core` alone (`Cargo.toml:12-13`) and core still depends on nothing.
Test tree mirrors `src/` (ADR-004) and the probes name the new exports in both
crates.

**2. Assumption register honored.** Pass on the rows the code can break.
A9: no new `CoreError` variant — `error.rs:32-77` is the shipped family, and
every policy refusal is `InvalidValue` / `MissingEntry` / `Io` /
`UnsupportedVersion` / `NestingTooDeep`. A1: an inapplicable row is always a
verdict with a report, never a partial write and never a refusal, and the entry
refusals are exactly the three the proposal names (`apply.rs:231-262`). A4/A5:
hands are the row's inputs, the predicate is the only legality oracle, and a
`keyboard:` id the grid or tables cannot answer propagates the predicate's
`InvalidValue` rather than becoming a verdict (`apply.rs:456`, `617-643`).
A6/A23: the guard is unconditional and built only from `dead-keys`,
`docked-actions`, and `exclusive-groups` (`apply.rs:327-339`). A11: a fourth
kind and a `policy` `v2` both stay refusals. A15: slug-before-filesystem, with
the octet cap applied before the join (`policy.rs:55-61`). A13: the bound is
derived from the taxonomy length, not a literal 4 (`document.rs:180`).
A19: the mouse pair is read through `InputId::parse`, not trusted as a literal
(`apply.rs:705-709`).

**3. Deviations declared.** Not evaluable at this seat — `03-build-notes.md` is
under `docs/designs/` and is outside both the diff and this seat's stated
inputs. The gaps I found are listed as findings so the editor can check them
against the build notes (F2, F3, F4 below are the ones that need a line there
if they are intentional).

**4. Scope creep.** None found. Everything the diff adds traces to a proposal
sentence: the cluster-connectivity computation (`apply.rs:617-643`) is A21's
"built here"; `Transform::kind()` (`document.rs:405-411`) and
`TagPattern::spelling()` (`document.rs:480-485`) are what `serialize` and
`TransformReport.kind` need. `tests/common/mod.rs` is the same helper shape
`crates/core/tests` already ships. `seeded_from` is the one output field with no
proposal sentence — F4.

**5. Build-notes claims.** Not evaluable (see 3).

**6. Error paths exist and fail closed.** Pass. Every grammar fault has its
fixture and its refusal names the entry, row, or field
(`document.rs:1346-1498`, `policy.rs`), the corpus is 6 valid / 30 invalid and
`policy.rs:999-1020` pins that the named files exist, the unchanged walk picks
the directory up (`conformance.rs:404-426`), and the `forged-log-line` case is
answered by the escape-and-truncate posture (`error.rs:146-148`, `172-177`).
Nothing in `apply` writes before every check for that row has passed, so
"never mangles" is structural, as claimed.

**7. Previous round's blocker/major findings.** N/A — first code pass.

## Findings

### F1 — minor — `NotApplicable.offender` carries prose, but the proposal and the shipped `spec.md` say it carries an id

Location: `crates/transforms/src/apply.rs:94-100` (field and its doc comment),
used at `apply.rs:401-404`, `411-417`, `445-448`, `566-606`, `676-679`,
`732-739`; contract text at `schemas/policy/v1/spec.md:208-211`.

The proposal declares `pub offender: Option<String>, // a grammar-checked id,
never raw text` and asserts "every string a report carries is a grammar-checked
id". `spec.md`, shipped in this diff, states a report carries "an offender that
is always a grammar-checked id — never raw document text". The implementation
instead fills the field with a composed sentence — e.g. "`keyboard:d` is bound
to `combat.fire`, which the row never named" — and the field's own doc comment
still says "the id, tag, or key the row tripped over". The ids inside are all
grammar-checked, so the bounded-echo posture is intact, but the field is not an
id, and a third-party implementer that follows `spec.md` emits a different
report for the same fault. No fixture or test pins report text, so nothing in
the gate catches the drift; the suite even disclaims it
(`crates/transforms/tests/common/mod.rs:346-355`, "which sentence names it is
not part of the contract").

Fix: pick one side and make it consistent. Cheapest: amend the sentence in
`spec.md` to say the offender names the grammar-checked id, tag, or key at
fault (plus a short bounded reason line), and correct the field's doc comment
at `apply.rs:96`. Otherwise: put the checked id in `offender` and carry the
explanation in a separate field (which would also need the proposal updated).

### F2 — minor — a base whose `set` repeats an action resolves first-wins here and last-wins in core

Location: `crates/transforms/src/apply.rs:276-309` (`Layout::new`,
`input_of`, `action_on`, `rebind`).

`profile/v2` states "duplicates are data: a repeated action id inside one link
is legal; the resolution rules decide what it means", and `resolve` implements
last-wins (`crates/core/src/chain_ops.rs:167-170` — `bound.insert` over the
`set` list). Nothing refuses it for a base either, and the transforms crate
builds its working layout straight from `base.set`, taking the **first** element
for both reads and writes. So for `set = ["move.steer-left keyboard:a",
"move.steer-left keyboard:b"]`, a relocate of that tag rebinds the stale first
element, leaves the effective binding (last) alone, and reports `Applied` — a
row that did nothing while looking as though it did something, which is exactly
the shape this design refuses elsewhere (it is the reason the closed entry set
exists, `document.rs:158-165`).

Fix: make the working layout agree with `resolve` — collapse `elements` by
last-wins when `Layout::new` builds it (keeping the element's position in the
order-preserving output), or refuse at `refuse_entry` a base whose `set` binds
one action twice, naming the action. Either way the report stops lying.

### F3 — minor — the guard misses a matched-but-unbound docked action in `swap-polarity`

Location: `crates/transforms/src/apply.rs:500-510`.

The guard is "a row that touches an action named by the manifest's
`docked-actions` … is NotApplicable `reserved-input`" (`spec.md:176-179`).
`swap_polarity` builds `moves` with `filter_map` over bound matched actions
only, so `guard_fault`'s `docked_at(action)` leg never sees a matched action
that the base leaves unbound. With `swap-polarity move.steer-*`, `steer-left`
hard-docked by the manifest and unbound in the base, the row falls through the
`(Some, Some)` test and reports `Applied` while writing nothing, instead of
`reserved-input`. `relocate-cluster` cannot hit this because `unbound-action`
fires first. No mangling results, but the row's answer is the wrong one of the
ten.

Fix: run the `docked_at` leg over `matched` rather than over the bound subset
(a small loop before the `filter_map`), so the guard sees every action the row
names.

### F4 — minor — the produced base copies `seeded_from`, which the proposal's output rules do not state

Location: `crates/transforms/src/apply.rs:665`.

The proposal enumerates the produced document: `kind`, `id = policy.id`, empty
`derives`, copied `for`, `set` = working layout, `generated-by =
"policy:<policy-id>"` — `seeded_from` is not in the list, and the choice to
carry it forward is left implicit. It is harmless under pin equality (a
`seeded-from` must name a preset the pinned manifest ships,
`crates/core/src/chain_ops.rs:161`, and the pin is copied), but a produced base
now asserts a seeding provenance its layout no longer has, and a future
consumer cannot tell whether carrying it was intended.

Fix: either drop it (leave `seeded_from: None`) or state the carry-forward in
`spec.md` → "The output" and in the build notes, with the reason.

### F5 — nit — the non-base entry refusal always says `patch`

Location: `crates/transforms/src/apply.rs:232-238`.

The condition is `!matches!(base.kind, Some(ProfileKind::Base))`, which also
covers a `profile/v1` document with no `kind` at all, but the message is the
fixed string "`patch` is not a base: …". Naming what the document actually
carried (or `none`) keeps the refusal about the document rather than about one
of the two ways to fail it.

### F6 — nit — off-by-vocabulary comment

Location: `crates/transforms/src/apply.rs:104-105`: "A fourteenth reason is a
`policy/v2` question" — the set is ten (`apply.rs:107-143`, pinned by
`crates/transforms/tests/zz_api_probe.rs:85-96`), so the next reason is the
eleventh.

### F7 — nit — fixture comment names an id the fixture does not use

Location: `schemas/policy/v1/fixtures/valid/exact-tag-policy.toml:1-6`. The
comment reasons about `move.strafe-left` "never reaching
`move.strafe-left-extra`", but the exact-tag rows are `swap-polarity
move.steer-left` and `relocate-cluster move.strafe-right …`. The corpus is the
contract for a third-party reader, so the comment should name the ids on the
rows.

## FYI (out of specialty, no severity)

`placement_detail` (`apply.rs:744-774`) keeps the **last** move that exceeds the
bound when several do; first-would-read better against the "which clause
failed" framing, and either is deterministic today.
