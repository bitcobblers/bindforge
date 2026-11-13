# Code review — correctness, round 1 pass c0

Diff: `4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'`
Inputs used: that diff, `02-proposal.md`, `docs/adrs/014-profiles-as-chains.md`.
`03-build-notes.md` is a run record excluded from this pass's inputs, so
checklist rows 3 and 5 were judged on the proposal's own text; where the code
diverges from a sentence the proposal states explicitly, that is reported as a
finding rather than excused.

Verdict: **request-changes** — 1 blocker, 1 major, 3 minor, 2 nits.

## Checklist

**1. Components and interfaces named by the proposal exist and match.** Pass.
`Chain { links }` (private field, only `load_chain` builds it), `load_chain`,
`ResolvedBinding`, `resolve`, `fork`, `diff`, `merge` match the proposal's
signatures (`chain_ops.rs:61-64,111,150,196,217,285`); `ProfileKind`, the
`Profile` v2 fields, `CoreError::ChainLinkMissing { id }` with a `one_line`
`Display` arm (`error.rs:64,148-151`), `MAX_PATCH_DELTA_ELEMENTS` /
`MAX_GENERATED_BY_OCTETS` (`document.rs:165,170`), the
`PROFILE_V1_ENTRIES`/`PROFILE_V2_ENTRIES` rename and the `profile_entries(version)`
selector beside `reach_classes_entries` (`document.rs:76,84,428`), the
version-branched `serialize`, `chain.rs`'s doc-pointer-only header, the
`lib.rs` module + re-exports + public-surface doc, the fixture corpus (7 valid /
37 invalid, exactly the proposal's list), `schemas/README.md`,
`data/profiles/README.md`, `version_gate.rs`, `grammar_closure.rs`,
`conformance.rs`'s expected-directory list, and `zz_api_probe.rs` are all there.

**2. Assumption register.** Honored except two rows: A15 / the compatibility
matrix's "existing … refusal byte-unchanged" (finding 2) and A5's
`check_dir_version` (finding 3). A1/A2 (iterative walk, both refusals before
descent, documents-opened unit) are implemented correctly — `load_chain`
refuses `Cycle` on a repeated id and `ChainTooDeep` when a 33rd document would
be opened, and the exactly-32 case loads (`chain_ops.rs:120-131`). A13 holds for
the new refusals: no path enters a message. A18's named refusal exists. A14's
slug grammar plus the 63-octet bound is applied to `id`, every `derives`
element, `seeded-from`, and the pin's head, matching manifest.rs's own 63-octet
slug bound, so no pin can name a manifest a v2 document cannot express. A19
holds: every delta list is a one-line string array.

**3. Declared deviations.** Not evaluable within this pass's inputs (build notes
excluded). Two divergences from sentences the proposal states are reported below
(findings 2 and 3) so the editor can decide whether they are declared or need a
proposal/spec amendment.

**4. Scope creep.** `merge`'s provenance inheritance is behavior the proposal
never asked for (finding 4). `Chain::links()`/`head()` and `ProfileKind::name()`
/`from_name()` are necessary support for a private-field `Chain` — not creep.

**5. Build-notes claims.** Not evaluable (see 3).

**6. Error paths fail closed.** Pass. Every v2 fault is typed; a v2 document
sitting in a `v1/` directory is refused; a v1 head is refused rather than
defaulted (`chain_ops.rs:437-443`); a `derives` element cannot escape the
directory because the slug grammar runs before any path is built; `merge`
re-applies its own emitted lists over the same symbols and refuses on any
mismatch (`chain_ops.rs:773-787`), so it cannot ship a patch that means
something else. The blocker below is the mirror image: it refuses too much.

**7. Previous round's blockers/majors.** None — first pass.

## Findings

### 1. Blocker — `merge` can never emit a `swap`: every merge that keeps an exchange is a false conflict

`crates/core/src/chain_ops.rs:763` (in `emit_composition`, 746-789)

```rust
if !reciprocal || exchanged.contains(other) {
    return Err(composition_fault(action, other));
}
exchanged.insert(action.clone());
```

The partner of an emitted pair is looked up under the wrong key. The first
action of a reciprocal pair emits `swap(action, other)` and inserts `action`;
when the loop reaches `other` (it always does — `touched_actions` puts both
halves of every `swap` into `touched`), `other` of *that* iteration is the first
action, which is in `exchanged`, so the branch fires and the whole merge
refuses. `emitted.swap` therefore can only ever end empty, and any composition
whose net effect keeps an exchange is refused as a conflict that does not exist.

Smallest case: `p1 = swap a b`, `p2 = set c k` — the two patches touch disjoint
actions, the proposal's own definition of a compatible overlap ("Compatible
overlaps are not conflicts"; "Output element order is deterministic —
first-patch touch order, then second-patch-only actions"), and the answer is one
patch `set c k` + `swap a b`. `merge` returns
`InvalidValue("a and b compose to an effect no one patch can write …")`. So
ADR-014 Decision Three's `merge` is broken for a whole class of inputs, and the
suite does not see it: no merge test asserts a non-empty `swaps_of(&merged)`
(`merge_composes_compatible_patches_into_one_patch` asserts it *empty*,
`chain_ops.rs:1984`), and the two swap-carrying merge tests
(`:2017`, `:2046`) both expect a refusal, so they pass either way.

Fix: skip the partner instead of faulting it, keeping the genuine refusal for a
non-reciprocal composition —

```rust
if !reciprocal {
    return Err(composition_fault(action, other));
}
if exchanged.contains(other) {
    continue; // this pair was already emitted from its other half
}
exchanged.insert(action.clone());
emitted.swap.push((action.clone(), other.clone()));
```

and pin it with the missing test: `p1` = `swap a b`, `p2` = `set c k` (or
`clear d`) must merge into one patch carrying `swap ["a b"]` plus the second
patch's element, and resolve over the base to the same layout as the two-patch
chain. Keep the `swap X Y` under `clear X` case refusing — with this fix it
still refuses through the `!reciprocal` branch, as it does today.

### 2. Major — a shipped refusal string was edited to satisfy a new test

`crates/core/src/error.rs:141`

```rust
CoreError::Cycle => write!(f, "chain derives from itself (`Cycle`)"),
```

The proposal's model section says "No `CoreError` variant is removed or
changed; one is added", the compatibility matrix pins "Existing v1 documents,
fixtures, `resolve_chain` | yes (untouched)", and A15 pins the shipped chain
guard as delivered. `Cycle`'s message is design 008's, and its only reason to
change is the new assertion `assert_refusal_names(&err, &["Cycle", "link-a",
"link-b"])` at `crates/core/tests/chain_ops.rs:1281` — a needle list the
original message cannot satisfy (and that no message can satisfy for `link-a`
and `link-b`, since `Cycle` carries no payload; the test passes only on the
`"Cycle"` needle, via `assert_refusal_names`' `any()`). Editing a frozen
contract string to make a test green is the wrong direction of dependency.

Fix: revert the `Display` arm to `write!(f, "chain derives from itself")` and
drop `"Cycle"` from that needle list — the variant is already asserted two lines
above (`chain_ops.rs:1277-1280`), which is what the proposal's Test strategy #3
actually pins.

### 3. Minor — the directory-major rule got a second spelling and a different refusal variant

`crates/core/src/chain_ops.rs:394-408`, `crates/core/tests/chain_ops.rs:1404-1427`

The proposal's Options says the one-directory rule "reuses `check_dir_version`
(A5)". `chain_ops` instead re-implements it and answers
`CoreError::UnsupportedVersion` naming `profile`'s majors, while every shipped
loader that applies this rule answers `InvalidValue`
(`document.rs:1337-1352`; `reach.rs:160`, `placement.rs:160`, `action.rs:235-241`)
— the same rule the codebase deliberately keeps as one definition so loaders
"cannot drift into two spellings of it" (`document.rs:1354-1358`). The new test's
doc comment then states the chain refuses "exactly as the shipped loaders
already refuse it (`reach_v2.rs`)", which is not the case: `reach_v2.rs:626-657`
pins `InvalidValue` for the identical fault. Behaviorally the chain path still
fails closed, so this is a contract-consistency and comment-accuracy problem,
not a hole.

Fix: call `check_dir_version(directory, &link.version, &format!("{expected}.toml"))`
and drop `check_link_version`; or, if `UnsupportedVersion` is the intended chain
answer (which is what `schemas/profile/v2/spec.md:120-123` states), say so in the
proposal's Options and correct the test comment to name the divergence instead of
denying it.

### 4. Minor — `merge` carries provenance labels onto a document they do not describe

`crates/core/src/chain_ops.rs:334-335`, `inherit_provenance` at `:650-656`

The proposal defines `merge`'s output as the caller's `id` plus inherited
`derives`/`for` plus the composed delta lists; it never asks for the provenance
labels to be carried. `generated-by` means "a label naming what produced the
document" (entry table), so putting `p1`'s label on the merged document asserts
that whatever produced `p1` also produced the merge result — a wrong fact in a
label the design insists is "never displayed as authority" and "never secret".
(`seeded-from` is less wrong — it describes the chain's base — but the
proposal's merge contract does not cover it either.)

Fix: leave both labels absent on `merge` output, or record the inheritance rule
in the proposal and pin it with a test that says which operand wins and why.

### 5. Minor — a missing head is reported as `ChainLinkMissing`, with no path and an unvalidated "id"

`crates/core/src/chain_ops.rs:118-119` → `open_link` at `:347-352`

`load_chain` runs the missing-link rule over the head too, so
`load_chain("…/v2/nope.toml")` answers
`ChainLinkMissing { id: "nope" }`. The proposal's step 1 is `parse_file(head)`,
and A13 assigns read failures to `Io`, whose variant carries the path for
`source_path`; `ChainLinkMissing` deliberately omits it, so a consumer cannot log
*which* head was missing, and the value it does echo is a raw file stem, never
validated as a profile id. The ancestor rule — the one A18 is about — is
implemented correctly and pinned (`chain_ops.rs:348-351`,
`tests/chain_ops.rs:1322-1346`).

Fix: resolve the head with `parse_file` (its `Io` already carries the path) and
apply `ChainLinkMissing` only to links the walk itself constructed.

### 6. Nit — `merge`'s symbolic model assumes the base binds both sides of every `swap`

`crates/core/src/chain_ops.rs:714-738` (`apply_lists`)

`Binding::Base` is never unbound, so a `swap` always moves in the symbol model
while `resolve` rules it a no-op when a side is unbound (rule 3). The
consequence is over-refusal only — e.g. `p1 = set X a`, `p2 = swap X Y` composes
to a non-representable symbol composition, yet over a base leaving `Y` unbound
the real composition is just `set X a` — so the code stays sound, and the comment
says the assumption is deliberate. The proposal does not state it. A sentence in
the merge section ("the base is assumed to bind both sides of an exchange; a
merge that would need otherwise refuses") would make the refusal predictable to
a third-party implementer.

### 7. Nit — `serialize` writes a v2 document's `kind`/`for` conditionally

`crates/core/src/document.rs:639`

The proposal says "the writer branches on `version` alone", but the v2 branch
skips `kind` and `for` when `kind` is `None`. That cannot happen for a parsed v2
document, but `Profile`'s fields are all public, so a consumer can build a v2
model with `kind: None` and get text that will not re-parse — the one case the
function's own doc comment promises not to produce. Either write `kind`/`for`
unconditionally for a v2 version and document that a v2 model must carry a kind,
or state the precondition in the doc comment.

## FYI (out of specialty, no severity)

- `MAX_SLUG_OCTETS` + the slug refusal sentence now exist twice —
  `manifest.rs:63,538-547` and `document.rs:176,1051-1067` — with `SLUG_RULE`
  duplicating `placement.rs`'s spelling. One shared helper would keep the two
  caps from drifting.
- `check_joins` scans `manifest.actions` linearly per touched action
  (`chain_ops.rs:523-537`), which is not the "one per-resolution lookup" the
  proposal's cost paragraph claims; the bound is the same, the constant factor
  is not.
