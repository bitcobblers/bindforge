# security review r1 (panel pass c0, first pass for this seat)

verdict: request-changes

Scope: whole document — `docs/designs/009-canonical-profile-and-manifest-model/02-proposal.md`,
against `docs/adrs/009-canonical-profile-and-manifest-model.md` and the proposal's Revision log.
Checklist applied: `references/security.md` (trust boundaries, untrusted parsing, path
handling, secrets/PII, dependency advisories, least privilege, fail-closed).

Checklist status:
1. Trust boundaries — adequate. `schemas/`+`data/` documents are parsed as untrusted at every
   entry point (inherited from `schemas/README.md` and `crates/core/src/error.rs:3-5`); reviewed
   seed content is gated by the authoring bar with a named reviewer; provenance "labels the
   chain and gates nothing". No finding.
2. Parsing untrusted data — adequate: reuse of `parse`/`parse_file`/`parse_reader` (1 MiB
   pre-read cap, 16-level nesting cap, closed entry sets), version gate never defaults,
   per-format refused-construct fixtures. TOML is text: no code evaluation, no compression.
3. Path handling — the profile side is strong (id→file index, ids string-compared only,
   escape-carrying id fixture stays valid, Test #10). Two gaps: F1, F3.
4. Secrets/PII — largely inherited and adequate: bounded one-line rejections, quoted
   document strings pass through `one_line`, "no rejection message carries a local path".
   F4 covers the one field a shared profile carries that is unconstrained.
5. Dependencies — the design adds none (`bindforge-core` stays depends-on-nothing,
   `bindforge-transforms` depends on the path crate only). Checked RustSec for the crates the
   document names: no advisory entries for `serde`, `toml_edit`, `toml` (package pages 404,
   recent advisory feed contains none), and none is in this design's build graph
   (`serde` is staged in `[workspace.dependencies]`, used by no member; `toml_edit` is named
   only as ADR-012's consumer path). No advisory blocker.
6. Least privilege — adequate: reads and writes only `schemas/` and `data/` documents, no
   game-config I/O, no network, no device access, no input injection (Non-goals); the index
   walk is depth- and entry-count-bounded.
7. Fail-closed — mostly: every new condition names a `CoreError` variant pinned by a fixture,
   transforms no-op on precondition failure, merge has no implicit winner. Gaps: F1, F2, F3, F5.

## Findings

- [major][new] Chosen design → Manifests state game facts / Profiles are chains —
  "content `data/manifests/<game>/manifest.toml` (fixture-name rule for `<game>`)" +
  "its halves validated against the game-id grammar and a dotted `<major>.<minor>` revision,
  else `BadPin`" + "Ids never appear in a path" — the design's canonical path rule is scoped to
  profile ids ("Id → file, never id → path", `index.rs` walks `data/profiles/`), and manifest
  resolution is left unstated while its layout puts a game id in a path component. The drift
  mechanism ("resolves every profile in `data/profiles/` against the manifest revision it
  pins") therefore has no specified lookup path, and the natural implementation — join the
  pin's game half into `data/manifests/{game}/manifest.toml` — interpolates a string from a
  shared, untrusted profile into a filesystem path, the exact class A18 exists to stop. The
  mitigation the design leans on is the grammar check, and "the game-id grammar" is named
  nowhere in the document: `Encoding rules` defines only the taxonomy-id and input-id shapes,
  so `BadPin`'s acceptance set is untestable as written. This does not re-open the Revision
  log's rejected "id charset narrowing" for profile ids — the escape-carrying id fixture stays
  valid under this fix, and profile ids keep resolving by index. Fix: name the game-id grammar
  as the shipped fixture-name rule (lowercase ASCII words joined by `-`; never a dot, a `/`, a
  `\`, or a leading `.`) in `Encoding rules`, and state in the Manifests section that a pin
  resolves through the same bounded walked index over `data/manifests/`, keyed by the
  document's declared `game` and `revision` — no path is built from a pin. Add a pin carrying
  `..` and an absolute-path pin to Test strategy #10's list so the rejection is pinned.

- [major][new] Chosen design → Transforms produce bases — "each is a non-empty array of
  strings, first element the transform name" and Test #11 "an unknown transform name is
  rejected" — the policy `stack` is validated by transform *name* only. Nothing pins the
  arity or argument grammar of an invocation: `["relocate-cluster", "move.steer-*"]` (no hand,
  no placement id), `["swap-mouse-buttons", "extra"]`, a hand value outside `left|right`, a
  placement id that is not a placement-id-shaped string, a pattern with `*` anywhere but
  trailing. A registry that reads `invocation[1]`/`invocation[2]` after checking only the name
  panics on those documents, and the shipped contract is that every refusal is a typed variant,
  "never a panic, never a bare string" (`crates/core/src/error.rs:5-6`), with policy documents
  untrusted at every entry point. Fix: state in `policy/v1/spec.md` that each invocation must
  match its registry entry's declared arity and the grammars of its arguments (hand ∈
  `left|right`, placement id, trailing-`*` action pattern), rejected as the shipped
  `InvalidValue { key, message }` — no new variant needed — one `fixtures/invalid/` case per
  shape, and extend Test strategy #11 to assert each is rejected rather than reaching
  `apply_policy`.

- [minor][new] Chosen design → Profiles are chains / index walk — "walks `data/profiles/`
  (bounded depth and entry count, each file size-checked by the shipped budget)" — the shipped
  budget check is metadata-based: `read_file` compares `fs::metadata(path).len()` to
  `MAX_DOCUMENT_BYTES` and metadata follows symlinks (`crates/core/src/toml.rs:54-61`). For a
  directory tree, an entry that is a symlink to `/dev/zero` (metadata length 0) or a FIFO
  passes the pre-read cap and then reads unbounded, so "size-checked by the shipped budget"
  does not bound what the walk reads, and the guard fails open (hang/OOM) instead of rejecting.
  In this design the tree is reviewed repo content, so this is hardening of the guard the model
  contributes to the hub path, not an in-scope live exploit — hence minor, not major.
  Fix: one clause — the walk takes a directory entry's own type
  (`DirEntry::file_type().is_file()`; symlinks are not followed) and rejects non-regular
  entries, and the cap applies to bytes read from the opened file (the `read_reader` take-N
  shape), not to metadata length.

- [minor][new] Chosen design → Profiles are chains — "`generated-by` (free-form string id,
  e.g. `policy:lefty-mouse-relocate`)" — `generated-by` is the one profile field with no
  grammar, and a profile is the artifact that leaves the machine when bindings are shared, so a
  free-form string is where an author's name, handle, or email ends up published in a hub
  listing. (The phrase is also self-contradictory: free-form and id are opposite constraints.)
  Fix: constrain the value to the existing `<namespace>:<part>` input-id shape the example
  already follows (`policy:lefty-mouse-relocate`), say so in `profile/v1/spec.md`, and reject a
  value that is not id-shaped as `InvalidValue` — one mechanism, no new grammar.

- [nit][new] Chosen design → Encoding rules / Transforms — "`*` appears only in a policy action
  pattern, as a trailing suffix (`move.steer-*`)" — a bare `*` is a trailing suffix on an empty
  prefix, so a shared policy can address the entire taxonomy; the only thing that limits the
  blast radius is each transform's precondition. Fix: require a non-empty literal prefix before
  `*`, and make a pattern that matches zero actions a named `Skipped` entry rather than a quiet
  no-op, so an adopter sees that a policy does nothing.

## FYI (no severity)

- Chosen design → Transforms: `apply_policy(...) -> PolicyOutcome` is a plain-return signature,
  while the prose says "`apply_policy` returns `Ok` with `skipped` populated" — signature and
  prose disagree about whether the function is fallible (correctness seat).
- Chosen design → Encoding rules: "**No string-embedded mini-languages.**" sits in the same list
  as the reach-row value `"<class> <row> <col>"` (`"home 1 1"`), which is a space-separated
  triple parsed out of a string — the rule and the reach-classes extension need to acknowledge
  each other (consistency/compatibility seat).
- Chosen design → Profiles are chains: `resolve(index, root, rung)` pins `rung = 1` and
  `rung = chain.len()`, and names no rejection for `rung = 0` or `rung > chain.len()`
  (correctness seat).
- Deferred section: the items I looked at as security concerns — config-fingerprint recipe
  (A21), wire-level budget enforcement, signatures/checksums behind `generated-by` (A22),
  path canonicalization — are each handed to ADR-012/ADR-014, which ADR-009's
  "Not decided here" clause names explicitly. Consistent deferrals; not re-opened here.
