# Security review — design 015 code, round 1 pass c0

- Seat: security
- Range reviewed: `git diff 1af12026893971772c5ac8384ceae348d63cd064...HEAD -- ':(exclude)docs/designs'` (53 files, ~4.7k insertions: `schemas/policy/v1/` corpus, `document.rs` policy arms, `policy.rs` loader, `crates/transforms` crate, suites)
- Inputs: that diff, `02-proposal.md`, `docs/adrs/015-transforms-as-preconditioned-policies.md`. First pass for this seat; no prior findings.

## Checklist walk

1. **External input validated at the boundary** — pass. `policy/v1` rides the shared `parse_file` path, so the 1 MiB `MAX_DOCUMENT_BYTES` pre-read bound and nesting cap apply unchanged. The closed entry set (`POLICY_V1_ENTRIES`), fixed row field counts, closed three-kind set, tag grammar (exact id or one `-`-anchored prefix, byte-exact — no general wildcard), hand pair, `0..=MAX_GRADE_BOUND` bound (digit-by-digit, no `+` coercion), closed-vocabulary targets, `MAX_POLICY_TRANSFORMS`/`MAX_RELOCATE_TARGETS` caps, and target-repeat refusal all land at parse, each pinned by its invalid fixture. No code evaluation anywhere in the diff.
2. **Path handling** — pass. `load_policy` slug-checks `id` *before* any filesystem question (`policy.rs:55-61`), so `..`, separators, and absolute-path shapes refuse naming nothing but the grammar; the joined component is `format!("{id}.toml")` from a grammar-checked slug. No shell anywhere. Refusal messages carry no path (`source_path` posture kept).
3. **Injection** — pass. No regex is used at all; tag matching is byte-exact string prefixing with the prefix forced to close on `-`, against a registry whose ids are grammar-capped at two segments — no catastrophic backtracking surface, no path interpolation from document text.
4. **Secrets and PII** — pass. The only log-shaped string in the corpus is the deliberately hostile `forged-log-line.toml` fixture (fake hub line, refused by the slug grammar); no credentials or personal data in source, fixtures, or messages.
5. **New dependencies** — pass. `Cargo.lock` gains exactly one package, `bindforge-transforms`, depending on `bindforge-core` only — workspace-local, no external crates, no advisory surface. Justified by A16.
6. **Filesystem writes** — pass (vacuously). The new code reads documents and builds values; it writes nothing. Scratch-dir use is test-only.
7. **Clean rejection of malformed/hostile input** — **one major finding** (below). The refusal-message posture itself is sound: every echoed document string goes through `one_line` (escape + 80-char truncate) in `CoreError::Display`; every report `offender` embeds only grammar-checked typed ids (bounded charset, ≤63-octet segments, ≤16 joined per message), so the bounded-echo posture holds without a `CoreError` in sight. Entry refusals fail closed (`kind` absent → refused). No panic path on hostile input was found (`fields[0]` is safe because `split(' ')` yields ≥1 field; index accesses are length-guarded; `rook_neighbours` uses checked `try_from`).

## Findings

### major — `crates/transforms/src/apply.rs:276-309, 436-449, 512-519, 552-553` — first-match `Layout` semantics let a hostile-but-legal base defeat the guards

`Layout::new` copies `base.set` verbatim; `input_of`/`action_on` answer with the *first* matching element and `rebind` writes the *first* match. But profile/v2 determinism rule 3 (`schemas/profile/v2/spec.md:175`) is **last-writer-wins**, the grammar does not forbid it (`parse_set` has no cross-element check; `set-twice-last-element.toml` pins it valid), and the consumer of the produced base is `resolve`, which is last-wins (`chain_ops.rs:169-171`). Bases are untrusted documents. Two exploitable shapes, each reported `Applied` and producing a document that passes every downstream validation:

- **Silent defeat of a row.** Base `set = ["move.steer-left keyboard:a", "move.steer-right keyboard:s", "move.steer-left keyboard:z", "move.steer-right keyboard:x"]`: every precondition (contiguity, not-keyboard, guard, predicate) is checked against the *first* copies' keys, the row reports `Applied`, and the writes land on the first copies — while `resolve`'s last-wins keeps the untouched trailing copies (`z`/`x`) as the effective bindings. The transform demonstrably changed nothing the player will ever feel, contradicting "never a silent no-op" (proposal, Options: Tags) and "a report reads back as the row it came from".
- **`occupied-target` guard bypass.** A target input bound by two actions in the base, the first holder *inside* the match set and a foreign one later, passes the occupancy loop (`apply.rs:436-449` sees only the first holder via `action_on`) — the row lands a matched action onto a key a never-named foreign action also holds, the exact displacement the check exists to prevent ("a target already bound to an action outside the match set → `occupied-target` naming it", proposal §The stack). The same first-match semantics also make `swap_polarity` (512-519) and `swap_mouse` (552-553) rewrite only the first copies, so on a duplicated binding a "swap" can be a no-op reported as applied.

This is the diff's core security surface — the guards and the atomicity guarantee the ADR calls structural — failing on a document the shipped grammar accepts.

**Fix:** make `apply` see one binding per action, the same answer `resolve` gives. Cheapest and posture-consistent: in `refuse_entry`, refuse a base whose `set` names one action twice or binds one input twice (`InvalidValue` naming the duplicate — no new variant, A9), which keeps `apply`'s one-binding-per-action assumption explicit and reject-clean per checklist 7. (The alternative, folding `base.set` through rule 3 before `Layout::new`, must also decide what "preserves the source base's element order" means for dropped elements — the refusal keeps that question out.) Whichever shape, pin both hostile bases in `crates/transforms/tests/apply.rs`: duplicate-action base → refused (or last-wins-folded, with the predicate seeing the *effective* keys), and duplicated-input `occupied-target` refusal naming the input.

## FYI (no severity, out of specialty)

- `relocate` runs the untouchable guard before contiguity and occupancy, while the proposal enumerates it first and contiguity/occupancy before the predicate; both orders are write-free verdicts, so only *which* reason a multiply-faulted row reports differs — worth a coder note if another seat wants the enumeration order pinned literally.

## Verdict

request-changes — one major open finding (hostile-input guard gap above); checklist items 1-6 pass.
