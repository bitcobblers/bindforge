# Code review — correctness seat, round 1 dissent pass (c1)

- Run: `024-game-slug-identity`
- Design: `docs/designs/024-game-slug-identity/02-proposal.md` (read at HEAD), source ADR `docs/adrs/024-game-slug-identity.md`
- Scope reviewed: `git diff 0ee2eeeb7992f470561763e45e56261f503e64ae..HEAD -- ':(exclude)docs/designs'`
  (8 files: `crates/core/src/{document,placement,policy}.rs`, `crates/core/tests/{chain_ops,placement,policy}.rs`,
  `schemas/README.md`, `schemas/policy/v1/spec.md`), read against the full current proposal and the HEAD state of
  the surfaces the revision claims to describe.
- Gate record supplied by the runner (2026-11-18T19:38:10Z): `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked` all exit 0.
- Inputs used: the diff, `02-proposal.md`, ADR-024, the gate record, my round-1 findings. No review report or
  transcript was opened. `03-build-notes.md` was not in this seat's allowed inputs, so deviation-declaration
  claims could not be verified (stated per finding where it matters).

## Checklist results

1. *Components/interfaces named in the proposal exist and are wired*: pass for the surfaces this revision
   touches (the exported `MAX_SLUG_OCTETS`, `is_slug_name`, and the load-seam checks are all present and wired).
2. *Assumption register honored*: **fail** — finding C1-c1-2. The revision silently narrows what `024-B1`
   covers relative to ADR-024's own enumeration.
3. *Declarations in `03-build-notes.md`*: **not verifiable by this seat** (file outside allowed inputs).
   Findings C1-c1-2 and C1-c1-4 are deviations from the proposal's mandate whose declaration status this seat
   cannot read; each states the escalation.
4. *Scope creep*: findings C1-c1-2 (a behavior pin the proposal never asked for) and C1-c1-4 (the
   refusal-message restructure, wider than the `format!` mandate).
5. *Build-notes claims*: n/a (not readable here).
6. *Error paths from the design exist and fail closed*: pass. Both new bound arms (`crates/core/src/policy.rs:57-65`,
   `crates/core/src/document.rs:1235-1243`) refuse before any file is read or path is joined; the new pins in
   `crates/core/tests/policy.rs:874-919` and `crates/core/tests/chain_ops.rs:692-716` demonstrate it. The gate
   shows the whole suite green, so the pre-existing 128-accept/129-refuse re-pins of Test strategy #6 hold.
7. *Previous round's blocker/major addressed*: **no** — findings C1-c1-1 and C1-c1-3 are the round-1 blocker and
   round-1 major, both still open at HEAD.

## Findings

### C1-c1-1 — blocker (carried from round 1, not addressed)

- **Location**: `docs/adrs/013-game-manifests.md:114-116`
- **Problem**: The proposal requires the ledger amendment three times — Goals: "Record the ledger amendment
  (`013-L5` gains `amended-by 024-B1`)" (02-proposal.md:31); "Ledger: the `013-L5` unit line in
  `docs/adrs/013-game-manifests.md` gains its `amended-by 024-B1` annotation per `021-L4`" (:226-228); Rollout 3
  (:322-323) — and ADR-024 Rollout 5 (:142-143) makes it a rollout item of this run. At HEAD the unit line still
  reads "- **013-L5** Name agreement: … → spec: Table data, Load-time validation." with no annotation, and
  `docs/adrs/013-game-manifests.md` appears in no commit of this branch (`git diff --stat main...HEAD` lists 45
  files, none of them an ADR). The revision under review did not touch it either. A branch that raises the shared
  bound and amends four published contracts would therefore merge with the amended unit's ledger row un-annotated;
  no gate test reads the ledger, so nothing else forces it to land.
- **Fix**: Append the `amended-by 024-B1` annotation to the `013-L5` unit line (013's prose and `013-R1` stay
  untouched, per proposal :227-228), as part of this run — the runner owns these commits per proposal :228. If
  `docs/adrs/` is outside the build stage's path ownership, the runner must land the edit before completion; it
  cannot be satisfied by prose in a design artifact.

### C1-c1-2 — major (new in this revision)

- **Location**: `schemas/README.md:126-135`; `crates/core/src/placement.rs:67-81` (doc comment of the exported
  `MAX_SLUG_OCTETS`); propagated into `schemas/policy/v1/spec.md:92-94`; pinned by
  `crates/core/tests/placement.rs:320-357`
- **Problem**: This revision removes the keyboard geometry name from the bound's coverage. The README bullet —
  which `024-L3` and the proposal (:206-212) make the bound's *one stated home*, and which `placement.rs`'s doc
  comment copies verbatim — now says: "A keyboard geometry name rides the grammar but states no length in its own
  entry table, so no length is enforced on it either; the bound stops where a format's spec stops." ADR-024
  `024-B1` (docs/adrs/024-game-slug-identity.md:97-100) states the opposite enumeration: `MAX_SLUG_OCTETS = 128`
  is "the single shared bound every slug-grammar id rides: a manifest `id`, `codec` or preset id, a profile `id`
  and each `derives` element, a policy `id`, **a geometry name**, and an action-id segment." Nothing in the
  proposal authorizes the narrower scope: it has no assumption-register row for the premise ADR-024's Context
  (:22-26 "a geometry name … capped at 63 octets") gets wrong, no revision-log entry, and no test-strategy item
  for the new pin; yet `crates/core/tests/placement.rs:332` now pins `a_geometry_name_past_the_shared_slug_bound_loads_as_known_current_behaviour`,
  converting the gap into asserted behavior. I verified the descriptive half is accurate (`load_geometry` at
  `placement.rs:155-161` checks only `is_slug_name`; the document `geometry` check at `document.rs:1681` likewise;
  `schemas/keyboard-geometry/v1/spec.md` states no length) — which is exactly the problem: the one stated home of
  the bound now publishes a scope that contradicts the budget unit this run implements, with no ledger trace and
  no ADR-024 note. It also leaves `placement.rs:83-89` self-contradicting in the same file: the grammar comment
  still says the bound "is applied beside it by each entry's own rule" for the three entry kinds that share the
  grammar, one of which the new text says applies nothing.
- **Fix**: Reconcile the ADR with the corpus rather than restating the budget two ways. Either (a) enforce the
  exported bound on the geometry name at the two load sites and keep the bullet's original enumeration — noting
  that, as `024-L3` puts it, that is a *tightening* and so by the ADR's own rule rides a schema major, not this
  in-place amendment — or (b) keep the code as is and record the divergence where it is authoritative: state in
  the README/placement text that this is a scope narrower than `024-B1`'s enumeration and why (a geometry format
  that states no length), name the follow-up that would cap it at a major, and have the ADR owner annotate `024-B1`
  (the run already touches the ledger for `013-L5` per C1-c1-1). If (b) is chosen, the divergence must be declared
  in `03-build-notes.md`; this seat may not read that file, so if it is not declared there this finding is a
  blocker under checklist item 3 (undeclared deviation), and the placement.rs test pin should say which ADR unit
  licenses it instead of "the accepted code-review r1 resolution".

### C1-c1-3 — major (carried from round 1, not addressed)

- **Location**: `schemas/manifests/v1/spec.md:54`; `schemas/profile/v2/spec.md:38` and `:71`;
  `schemas/policy/v1/spec.md:40` and `:92-93`; `schemas/action-ids/v1/spec.md:54-56`
- **Problem**: The proposal (:213-219) mandates that in the four amended specs "the number and the RFC 1035
  derivation become a cross-reference to the README bullet", and `024-L3` says "each format spec cross-references
  it instead of restating the number". All six bound sentences still publish the literal — "at most
  `MAX_SLUG_OCTETS` = 128 octets" (manifests:54, profile:38, policy:40), "capped at `MAX_SLUG_OCTETS` = 128 octets"
  (profile:71), "past the shared `MAX_SLUG_OCTETS` = 128-octet bound" (policy:92-93), "is at most
  `MAX_SLUG_OCTETS` = 128 octets" (action-ids:54) — and `action-ids/v1:55-56` still restates the superseded RFC
  1035 §2.3.1 derivation. Meanwhile the new README bullet (:133-136) and the constant's doc comment
  (`placement.rs:74-75`) assert that "each of the four now cross-references this bullet instead of carrying a
  number of its own", a claim false against the shipped corpus. The only spec edit in this revision
  (`schemas/policy/v1/spec.md:92-94`) dropped a co-rider phrase and left the number in place. A future bound
  amendment therefore still leaves four published `128` literals to rot — the drift `024-B1`'s one-stated-home rule
  exists to prevent.
- **Fix**: Drop the `= 128` gloss from all six sentences, e.g. "at most `MAX_SLUG_OCTETS` octets — the shared
  bound `schemas/README.md` states once (`024-B1`)", and drop the RFC 1035 clause from `action-ids/v1:55-56` in
  favor of the cross-reference. The number-free form the same corpus already uses at `profile/v2:39` and
  `manifests/v1:56` ("at most the same shared `MAX_SLUG_OCTETS` bound") is the model.

### C1-c1-4 — minor (carried from round 1, extended by this revision)

- **Location**: `crates/core/src/document.rs:1232-1243` (`check_pin`); `crates/core/src/policy.rs:50-65`
  (`load_policy`); on top of the round-1 sites `crates/core/src/action.rs:38-56` and `:257-277`,
  `crates/core/src/document.rs:1262-1281`, `crates/core/src/manifest.rs:529-546`
- **Problem**: The proposal's only mandate for these refusal strings was that a message embedding the number be
  built with `format!` from the exported constant (:125-127). The implementation keeps splitting fault kinds and
  reordering messages: this revision adds two more bound arms whose messages have shapes no contract asks for —
  `policy.rs` "a policy id is at most 128 octets: `<id>`" and, in `check_pin`, a bound arm that bypasses the
  local `fault` wrapper entirely, so a past-bound `for` head is refused as "a manifest id is at most 128 octets:
  `<head>`" and no longer names the value as a manifest *pin* or echoes the whole pin (the sibling arms keep the
  "`<text>` is not a manifest pin: …" frame). Accepted/rejected sets are unchanged (both arms refuse the same
  inputs, grammar before bound, before any file read) and the new shapes are pinned by
  `crates/core/tests/policy.rs:874-919` and `crates/core/tests/chain_ops.rs:692-716`, so this is a widened
  rejection-text surface, not a behavior break.
- **Fix**: No code change sought. Record the refusal-message restructure (now across `action.rs`, `manifest.rs`,
  `document.rs` ×2 and `policy.rs`, including `check_pin`'s loss of the pin-level framing) as a declared deviation
  in `03-build-notes.md`, or restore the "`<text>` is not a manifest pin: …" frame around the new bound message so
  the refusal still names the entry the way its siblings do.

## Notes

- Gate evidence used: the runner's green `cargo test --workspace --locked` leg (exit 0) establishes that the
  128-accept/129-refuse re-pins, the two new bound-arm pins, and the new geometry-name pin all hold at HEAD; the
  single `test result: ok. 0 passed` line the renderer captured is one leg's tail, not the suite total, and
  nothing in this report turns on it.
- FYI (out of seat, no severity): `schemas/keyboard-geometry/v1/spec.md` is the only slug-grammar format whose
  corpus states no length bound at all; whichever way C1-c1-2 is resolved, that asymmetry is the thing a future
  reader trips over.

**Verdict: request-changes** (1 blocker, 2 major, 1 minor; two of round 1's blocker/major remain open).
