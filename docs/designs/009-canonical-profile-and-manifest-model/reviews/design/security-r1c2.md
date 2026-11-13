# security review r1 — dissent pass c2

verdict: request-changes

Scope as given: `git diff 7b2ddc6d 5bbcc09d` over `02-proposal.md`, then the full
current proposal, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the
Revision log, and my own r1 findings.

## r1 findings re-checked

- **r1 #1 (major, invocation guard's enforcing component unnamed) — closed.** The
  changed §4 text puts the closed invocation table in `policy/v1/spec.md` as
  format contract, names `apply_policy` as the enforcer, splits the fixture
  surface honestly (`schemas/policy/v1/fixtures/invalid/` "covers exactly what
  it can see"), and Test #10 asserts `registry == spec table`; Rollout 1 and 3
  carry the table as spec content. The one deviation from my suggested fix —
  enforcing in `apply_policy` rather than core's reader — is sound and stated:
  the r2 log entry says a parse-time check "sat on the far side of the one-way
  dependency edge, so core had to duplicate the registry", which is true
  (`bindforge-transforms → bindforge-core` is one-directional). Fail-closed
  behavior is intact: the whole stack is validated before any transform runs, so
  no partial application, and because names/grammars are validated first, a
  `Skipped { transform, reason }` can only ever carry a table-approved name and
  a grammar-valid id — which, with the shipped `one_line` sanitizer
  (`crates/core/src/error.rs:132-134,155-169`), closes the log-injection surface
  in `Skipped.reason` as a side effect.
- **r1 #2 (minor, manifest index collision rule) — closed.** `DuplicateId` now
  fires "on two manifest documents declaring one `game` + `revision`", with the
  walk-order rationale stated, and Test #9 carries the shadowing-manifest case.
  No new variant, as proposed.
- **r1 #3 (minor, open `generated-by` namespace) — closed.** The namespace is
  now the closed producer list `policy|preset|generator`, the claim is softened
  to "publishes no free-text attribution", person-binding is pointed at
  ADR-014 (A22), and Test #6 rejects a namespace outside the list.

## Findings

- [major][new] Chosen design → Profiles are chains (Resolution, `resolve`
  bullet) / Profiles are chains → Id → file / Test strategy #9 — "`resolve` also
  rejects a `seeded-from` the pinned manifest does not declare, naming the
  profile id … `for` is required at parse, so the check always has a manifest."
  — the r2 clause is true of the *pin*, not of the *manifest*. `for` required at
  parse guarantees a well-formed `game` + `revision` string; the manifest is
  found by index lookup, and the design names no rejection when that lookup
  misses. `UnresolvedAncestor` is defined narrowly — "`UnresolvedAncestor` on a
  `derives` entry with no document" — `BadPin` is defined as a grammar failure,
  and Test #9's pin cases are only "a `for` pin whose game half is `..`,
  absolute, dotted, or dotted-revision is `BadPin`" and "a valid pin resolves to
  a manifest by index lookup". A profile pinning a deleted, renamed, or never
  authored manifest revision is therefore an unruled state, and
  `Resolved { manifest: Option<ManifestPin> }` offers the silent reading
  (`manifest: None`, proceed, preset check skipped) — which is exactly the
  vacuously-green condition the r2 revision says it closed ("unpinned, the check
  was vacuously green"), re-opened one level up, and it also makes the drift
  gate's "resolves every profile … against the manifest revision it pins" pass
  on a pin nothing satisfies. Fix: one clause in §3 — a `for` pin with no
  manifest document in the index is `UnresolvedAncestor`, that variant's rule
  stated once as "a document reference with no document" (no new `CoreError`
  variant, no new mechanism; the choice cannot be left to implementation,
  because the variant list is the shipped compatibility promise) — and one
  Test #9 case beside the valid-pin lookup: a well-formed pin naming no manifest
  is rejected, so `resolve`'s `seeded-from` check and the drift gate are never
  vacuous.

- [minor][new] Chosen design → Profiles are chains → Id → file (changed text) —
  "`crates/core/src/index.rs` walks `data/profiles/` and parses every document,
  indexing by its **declared** `id`; `data/manifests/` gets the same treatment
  keyed on each document's declared `game` + `revision`" — the keying presumes
  every document in a walked tree declares the format that tree holds, and no
  such rule is stated anywhere. Readers dispatch on the document's own `format`
  (the shipped `UnknownFormat` covers only formats the crate does not parse), so
  a document under `data/manifests/` declaring `format = "policy"`, or one under
  `data/profiles/` declaring `format = "manifest"`, parses cleanly and reaches
  the index with the key entry the tree needs absent (`id`, or `game` +
  `revision`) — the behavior on hostile or misfiled content is left to the
  implementer, in the same fail-closed family as the manifest-key collision
  closed this round. Fix: one clause — each walked tree accepts only the
  format(s) it holds (`data/profiles/` → `profile`/`patch`, `data/manifests/` →
  `manifest`), a document declaring another is refused at index time — and a
  Test #9 case planting a manifest-format document in `data/profiles/`.

- [nit][new] Chosen design → Adversarial posture (changed text) — "New
  `CoreError` variants, each pinned by a fixture in this slice except
  `MergeConflict` … and the two index limits, pinned by Test strategy #1's
  synthetic walk, not by a fixture document: … `DuplicateDelta`, `DuplicateId`,
  …" — the exception list is one variant short: `DuplicateId` is an
  index-level, two-document rejection, and the shipped harness asserts per
  fixture file (`crates/core/tests/conformance.rs:138-154`), so no single
  `fixtures/invalid/` document can pin it; Test strategy #9 is what pins it.
  Fix: add `DuplicateId` to the same exception clause, with Test #9 named as its
  pin.

## Checklist coverage (security)

1. Trust boundaries — pass. Every new format is treated as untrusted data on the
   shipped read path; hub-side trust is Deferred to ADR-014, consistent with the
   ADR's "Provenance labels the chain; it never substitutes for validation".
2. Parsing untrusted data — pass. 1 MiB pre-read cap, 16-level nesting cap,
   closed entry sets, refused-TOML constructs per format, no code evaluation, no
   decompression; the new formats add no per-document budget.
3. Path handling — pass. Ids are string-compared only, never path components;
   pin halves gated by grammar; `DirEntry::file_type()` with symlinks not
   followed and the cap on bytes read; `..`/absolute/control-character ids stay
   valid *as ids* and reach no file.
4. Secrets and PII — pass. `generated-by` is now a closed-namespace id, so no
   free-text attribution travels in a shared profile; rejection messages carry no
   local path (`source_path` stays out of `Display`).
5. Dependencies — pass. No new third-party dependency is named
   (`bindforge-core` stays depends-on-nothing, `bindforge-transforms` takes the
   path crate only), and the ADR's serde parenthetical is deviated from
   deliberately, so there is no advisory surface to webfetch.
6. Privileges — pass. Reads `schemas/` and `data/` only; no game config I/O, no
   network, no device access, no cwd dependence (Test #14).
7. Fail closed — the open item is the major finding above: a dangling `for` pin
   has no named rejection, and the walked-tree format rule is unstated.

## FYI (no severity)

- Chosen design → Input vocabulary — "`schemas/README.md` carries the admitted
  keyboard-name list: the admitted `keyboard:` vocabulary the reach loader
  enforces … No second key-name table is invented here." The enforcer is named
  and the rule is testable (Test #3), but a loader-enforced enumeration needs a
  machine-readable home, so "the README carries it" and "no second table is
  invented" describe the same list as both canonical and copied. Which artifact
  is canonical, and what asserts loader-list == README-list, is a
  spec-canonicity question for the correctness/compatibility seats.
- Chosen design → Manifests — `dead-keys` is a required entry whose only stated
  consumer is `validate`'s "valid input id" check, i.e. it is data with no v1
  reader; the proposal's own stated rule for that case (a required entry no
  consumer reads costs a major bump to remove) was applied to the policy `hand`
  this round. For the complexity seat.
