# security review r1 (dissent pass c1)

verdict: request-changes

Scope: `git diff cc78a21 7b2ddc6` on `02-proposal.md`, then the full current document, against
`docs/adrs/009-canonical-profile-and-manifest-model.md` and the Revision log. Inputs limited to the
proposal, the ADR, the Revision log, and my own r1 findings.

## Closure check on my r1 findings

- **major — game-id grammar + manifest resolution (A18 class): closed.** Encoding rules now name the
  game-id grammar ("Game ids: the same shape with no dot (`remnant2`), which excludes `..`, `/`, `\`
  and a leading `.` — this grammar, not a path rule, is what gates a pin's halves"), §3 adds the
  manifest index keyed on declared `game` + `revision` with "no path is ever built from a document
  string", §5 states the layout is cosmetic, and Test #9 carries a `..` pin and an absolute pin. The
  escape-carrying id fixture is untouched, so the Revision log's rejected charset narrowing stays
  closed. The rebuttal holds; not re-raised.
- **minor — walk guard fails open on symlink/FIFO: closed, verbatim.** "an entry is taken by its own
  type (`DirEntry::file_type().is_file()`, symlinks not followed) with the size cap applied to bytes
  read from the opened file (`parse_reader`'s take-N shape, A6), not to `metadata` length" — matches
  `crates/core/src/toml.rs:56` (metadata shape) vs `:79-90` (take-N), and the limits are now named,
  public, and pinned by fixtures + Test #1. A25 makes the aggregate bound (entries × per-file cap)
  explicit. Closed.
- **nit — bare `*`: closed.** Non-empty literal prefix in Encoding rules + "a pattern matching zero
  actions is a reported `Skipped`", asserted in Test #10.
- **minor — `generated-by` free-form: mechanism closed, guarantee sentence overreaches** (finding 3).
- **major — invocation arity/argument grammar: content closed, owner unstated** (finding 1).

## Findings

- [major][new] Chosen design → Transforms produce bases / Adversarial posture / Test strategy #1 /
  Rollout 3 — "Each invocation must match its registry entry's declared arity and the grammars of its
  arguments (hand ∈ `left|right`, a placement id, a trailing-`*` action pattern); a mismatch is the
  shipped `InvalidValue { key, message }`, one `fixtures/invalid/` case per shape" — the guard my r1
  finding asked for is fully specified in content (every shape enumerated, shipped `InvalidValue`
  reused, one fixture per shape, "before `apply_policy` runs") but its **enforcing component is
  unnamed, and the two candidate crates cannot both satisfy it as written.** The policy format is
  "parsed by `bindforge-core`, because the harness requires a reader for every format directory", and
  Test #1 requires "each `fixtures/invalid/` case is rejected with the pinned `CoreError` variant" —
  which the shipped harness does through core's reader (`crates/core/tests/conformance.rs:138-150`,
  `:223-232`). So the arity cases in `schemas/policy/v1/fixtures/invalid/` must be refused by core.
  But the knowledge needed to refuse them — transform names, arity, per-argument grammars — is stated
  to live only in `bindforge-transforms`'s registry (§4 "The v1 registry holds exactly"; Rollout 3
  "the registry — each entry carrying its declared arity and argument grammars"). `policy/v1/spec.md`
  is never told to carry that table, and Rollout 1 (write the spec files) does not list it, so core's
  reader has an undeclared dependency on a table in a crate that depends on it. Either the arity
  fixture set is untestable through the harness path (Test #1 cannot pass), or the table gets
  duplicated in two places and drifts. Consequence for this checklist: the fail-closed boundary for
  untrusted policy documents has no stated owner. Fix: state the closed invocation table (name,
  arity, per-argument grammar) in `schemas/policy/v1/spec.md` as part of the format contract, enforced
  by core's policy reader, and add one clause that `bindforge-transforms`'s registry carries exactly
  that table, with a test asserting registry == spec — one canonical statement, no new variant, no new
  mechanism.

- [minor][new] Chosen design → Profiles are chains (index walk) / Manifests state game facts —
  "`data/manifests/` gets the same treatment keyed on each document's declared `game` + `revision`, so
  a `for` pin is an index lookup and no path is ever built from a document string" — the index's key
  is declared content, and the design names no collision rule for it. Profile ids have `DuplicateId`;
  a manifest carries no `id` (its entry set is `format`, `version`, `game`, `revision`, …), so nothing
  rejects two documents under `data/manifests/` declaring one `game` + `revision`. A pin then resolves
  to one of them by walk order, which is exactly the path the drift gate (contract 5) and
  `seeded-from`/axis-kind cross-checks run against — a shadowing manifest is a determinism and
  pin-integrity gap, and the checklist's fail-closed item wants a rejection, not an order. Fix: one
  clause — the manifest index rejects two documents declaring one `game` + `revision` as `DuplicateId`
  on that declared key (no new variant), and Test #9 adds a shadowing-manifest case alongside the
  "valid pin resolves to a manifest by index lookup on declared `game` + `revision`" case.

- [minor][re-raised] Chosen design → Profiles are chains — "`generated-by` (an id in the
  `<namespace>:<part>` shape … a value outside the shape is `InvalidValue`, so a shared profile
  publishes no author name, handle, or email)" — the mechanism adopted my r1 fix and the free-text
  case is gone, but the justification sentence in the new text is stronger than the shape enforces.
  `generated-by`'s namespace is explicitly open ("there the namespace is a producer label, never an
  input namespace") and `<part>` is any lowercase word string, so `author:alice-smith`,
  `mike:handle` or `studio:alicegmail-com` are in-shape and `InvalidValue` cannot refuse them. That is
  the checklist's "what leaves the machine when bindings are shared" case, and a profile is the
  artifact that travels, so the absolute claim ("publishes no author name, handle, or email") is false
  as written. Fix: make the producer namespace a closed list in `profile/v1/spec.md` (e.g.
  `policy|preset|generator`), mirroring the closed input-namespace set, and soften the sentence to what
  the shape enforces ("publishes no free-text attribution"); ADR-014 keeps identity binding to a key
  (Deferred, A22).

## Checklist coverage

1. Trust boundaries — classified; hub is `no (v1) — out of scope`, consistent with ADR "Not decided
   here" (Deferred not re-opened). Pass.
2. Parsing untrusted data — new formats ride the shipped `parse`/`parse_file`/`parse_reader` path;
   nested-array rows stay at nesting level 2; walk bounds named + `IndexTooLarge`/`IndexTooDeep` pinned
   by fixture and Test #1; aggregate bytes = entries × per-file cap (A25). Pass, except finding 1 for
   the policy invocation set.
3. Path handling — no id becomes a path, pins are string lookups, grammar-gated halves, `..`/absolute
   pin fixtures present. Pass.
4. Secrets/PII — finding 3.
5. Dependencies — design adds none (`bindforge-core` depends on nothing, transforms → core path dep);
   `serde`/`toml_edit` are named only as a future ADR-012 consumer's. RustSec sweep found no advisory
   against serde or toml_edit (the hits were unrelated crates: basic-toml unmaintained, zstd-safe,
   actix-files). Pass.
6. Privileges — read-only over `schemas/` and `data/`, no network, no cwd dependence, symlinks not
   followed. Pass.
7. Fail closed — typed variants throughout, `InvalidValue` reuse confirmed shipped
   (`crates/core/src/error.rs:53`), no panic path left in the transform invocation contract; finding 2
   covers the one unspecified collision case.

## FYI (no severity)

- Compatibility matrix Windows row cites "exercised leg (Test strategy #13)" for the platform leg; the
  r1 renumbering moved Platform to #14 (Rollout 7 correctly says #14, Test #13 is harness pickup).
  Renumbering artifact — correctness seat.
- The graded set is defined twice: `schemas/README.md`'s "admitted keyboard-name list — the single
  enumeration the graded set below and Test strategy #3 iterate", vs the authoring bar's "the graded
  set — every key the shipped placements and the seed manifest's `default` rows reference, extended
  only by review". Two enumerations, one name — which one Test #3 iterates decides whether
  unreferenced admitted keys are graded at all. Correctness seat.
- `resolve` checks `seeded-from` against "the pinned manifest's `presets`"; with `for` absent there is
  no manifest, so the check is silently vacuous. Provenance "gates nothing" (ADR Decision 3) makes that
  acceptable; noted for the correctness seat.
