# Code review — correctness seat, round 1 dissent pass c2

**Design:** `docs/designs/024-game-slug-identity/02-proposal.md` (ADR-024)
**Diff under review:** `git diff 818ca62094aacfed640ec7320fe3d293a235646d..HEAD -- ':(exclude)docs/designs'`
(11 files: `apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_slug_refusal_tests.rs`,
`crates/core/src/placement.rs`, `crates/core/tests/placement.rs`, `crates/discovery/src/steam_common.rs`,
`crates/discovery/tests/collision_tests.rs`, `schemas/README.md`, `schemas/action-ids/v1/spec.md`,
`schemas/manifests/v1/spec.md`, `schemas/policy/v1/spec.md`, `schemas/profile/v2/spec.md`)
**Inputs used:** the diff above, `02-proposal.md`, `docs/adrs/024-game-slug-identity.md`, the runner's
gate record, this seat's round-1 findings. `docs/adrs/013-game-manifests.md` was read only to
adjudicate a round-1 finding that names it. No `reviews/` or `transcripts/` file other than this
seat's own report was opened; `03-build-notes.md` was not read (outside this seat's input set), so
checklist item 3's declaration test is recorded below as not directly checkable.

**Gate record (citable output):** `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked` all
exit 0 (green) at 2026-11-18T19:51:47Z. No failing leg to quote.

**Verdict: request-changes** — 1 open blocker (carried, unaddressed), 0 open majors, 2 minors.

## Disposition of this seat's round-1 findings

| r1 finding | Status in c2 |
|---|---|
| Blocker — `013-L5` ledger amendment never lands | **Still open.** Not addressed: no ADR is in the branch diff and `docs/adrs/013-game-manifests.md` carries no `amended-by` string at HEAD. See F1. |
| Major — bound's coverage of the keyboard-geometry name restated two ways | **Largely addressed** via the second option this seat offered (keep the code, record the divergence where it is authoritative). README bullet (`schemas/README.md:131-139`) and the constant's doc comment (`crates/core/src/placement.rs:71-79`) now state the narrower *enforced* scope, give the reason, and name the follow-up and the schema-major ride; the pin at `crates/core/tests/placement.rs:323-333` now cites `024-B1` and the bullet's clause instead of "the accepted code-review r1 resolution". Two residual legs remain — see F2, downgraded to minor. |
| Major — the four amended specs still publish the literal number, and action-ids still restates the RFC 1035 derivation | **Addressed.** `grep` over the four `spec.md` files finds no literal length for the slug bound and no `RFC 1035` / `63` anywhere: `schemas/manifests/v1/spec.md:54,56`; `schemas/profile/v2/spec.md:38,39,71`; `schemas/policy/v1/spec.md:40,92-93`; `schemas/action-ids/v1/spec.md:54-56`. The action-ids paragraph lost its DNS-label derivation and keeps its own shape rules in prose (`:52-53`), and its refusal example now names `segment-past-cap.toml`'s "segment one octet past the shared bound" (`:63-64`). The bullet's claim that "each of the four now cross-references this bullet instead of carrying a number of its own" (`schemas/README.md:141-142`) is now true against the corpus. |
| Minor — widened refusal-text surface in `check_pin` and `load_policy` | **Not touched by this revision; unchanged, still open as a minor.** See F3. |

## Checklist

1. **Components and interfaces named in the proposal exist and are wired** — **fails on one item.**
   The ledger deliverable (Goals `:31`; "Specs, fixtures, ledger" `:226-228`; Rollout 3
   `:322-323`; ADR-024 Rollout 5 at `docs/adrs/024-game-slug-identity.md:142-143`) is absent (F1).
   Everything in this revision that the proposal names is present and wired: the README bullet is the
   bound's one stated home and the constant's doc comment is its verbatim copy (`placement.rs:67-87`
   against `schemas/README.md:126-145`, with only the README's lead-in and the doc comment's closing
   "one exported home" sentence differing); `is_slug_name` and `MAX_SLUG_OCTETS` are exported from
   `lib.rs:116`; `refuse_invalid_slugs` (`apps/discovery-cli/src/main.rs:113-127`) keeps the
   `024-L1` boundary semantics, called ahead of both render paths (`:81-84`, before the `cli.json`
   branch).
2. **Assumption register honored** — pass for this revision. No row is silently changed. The
   geometry-name carve-out the bullet now states is descriptively accurate: `is_slug_name` carries no
   length test (`placement.rs:100-110`) and the three geometry sites check only grammar —
   `placement.rs:118` (`geometry_matches_file_name`), `placement.rs:165` (`load_geometry`, pre-join),
   `document.rs:1681` (the `geometry` entry) — while every id kind that *is* bounded applies
   `MAX_SLUG_OCTETS` beside it (`policy.rs:57`, `manifest.rs:540`, `document.rs:1235,1270`,
   `action.rs:298,312`). `schemas/keyboard-geometry/v1/spec.md` states no octet bound for its
   `geometry` entry, as the bullet claims.
3. **Deviations declared in `03-build-notes.md`** — **not directly checkable by this seat** (the
   file is outside the caller's input set). The one deviation this revision turns on (the narrower
   enforced scope) is declared in the published corpus itself, in the bound's one stated home, with
   reason and follow-up, so this seat does not escalate an unverifiable declaration; recorded here as
   the one property this pass could not itself establish.
4. **Scope creep** — pass. The revision's non-spec content is text and tests: the `refuse_invalid_slugs`
   extraction is behaviour-preserving (same predicate, same retain order, same one line per refusal,
   emitted at the same point), and the new `#[cfg(test)]` modules in `apps/discovery-cli/src/main.rs`
   and `crates/discovery/src/steam_common.rs` plus the `cli_slug_refusal_tests.rs` re-pin of
   `SLUG_BOUND` to `bindforge_core::MAX_SLUG_OCTETS` sit inside Test strategy #4 and #9's mandates
   (`discovery-cli` carries `bindforge-core` in `[dependencies]`, so the integration test can read
   it). No new refusal, acceptance, or output shape appears.
5. **Build-note claims true** — not checkable (input closed, see item 3). The claims the diff *adds*
   to the corpus were checked instead, and one overreaches: see F2.
6. **Error paths exist and fail closed** — pass. Grammar is checked before the bound and both before
   any path is built or any row renders; a past-bound slug is refused, never truncated
   (`main.rs:116-118`; pinned by `a_slug_one_octet_past_the_shared_bound_is_refused_not_truncated_into_a_row`);
   the collision pass refuses a name with no in-bound candidate and counts each of its rows into
   `skipped` (pinned by `a_name_with_no_in_bound_candidate_grows_skipped_by_its_own_row_count`).
7. **Previous round's blocker/majors actually addressed** — majors yes (one fully, one
   near-completely); the blocker no.

## Findings

### F1 — Blocker: the `013-L5` ledger amendment this design mandates has still not landed

- **Severity:** blocker
- **Location:** `docs/adrs/013-game-manifests.md:114-116` — the file appears in neither the
  source/test pathspec of `main...HEAD` (44 changed paths, no ADR among them) nor this revision's 11
- **Problem:** The design requires this deliverable three times — Goals (`02-proposal.md:31`),
  "Specs, fixtures, ledger" (`:226-228`), and Rollout 3 (`:322-323`) — and ADR-024 makes it a rollout
  item of this run (`docs/adrs/024-game-slug-identity.md:142-143`, `013-L5` gains its `amended-by`
  annotation naming `024-B1`, per `021-L4`). At HEAD, `grep -n amended-by
  docs/adrs/013-game-manifests.md` returns no match and the `013-L5` unit line still reads only "Name
  agreement: the game directory's name must equal the `id` entry byte-for-byte … → spec: Table data,
  Load-time validation." The c2 revision did not touch it either. The branch that raises the shared
  bound and amends four published contracts — the `id` row of `manifests/v1` is now annotated
  `013-L5 024-B1` (`schemas/manifests/v1/spec.md:54`) — would complete with the amended ledger unit
  un-annotated, and no gate test reads the ledger, so nothing else forces it: the green gate record
  does not cover it. This is checklist item 1 (a named deliverable that does not exist) and item 7
  (a prior-round blocker not addressed).
- **Fix:** Land the `amended-by 024-B1` annotation on the `013-L5` unit line in
  `docs/adrs/013-game-manifests.md` before completion, leaving `013`'s prose and `013-R1` untouched as
  history (`02-proposal.md:227-228`). Per `:228` and ADR-005 the commits are the runner's, so if
  `docs/adrs/` is outside the build stage's path ownership the runner applies the edit at a stage
  boundary; prose in a design artifact does not discharge it, and the branch should not be recorded
  complete until the line carries the annotation.

### F2 — Minor: the geometry-name carve-out is declared, but its "named follow-up" is registered nowhere and `024-B1` is left unannotated

- **Severity:** minor
- **Location:** `schemas/README.md:131-139`; `crates/core/src/placement.rs:71-79` and `:95-98`
  (`is_slug_name`'s doc comment); residual of `docs/adrs/024-game-slug-identity.md:97-100`
- **Problem:** This revision states the divergence well: the bullet and the copied doc comment say
  plainly that "`024-B1` enumerates a keyboard geometry name among these ids too, but … the bound's
  *enforced* scope is narrower than `024-B1`'s enumeration for this one format", why (capping would
  newly refuse documents `keyboard-geometry/v1` has always accepted; a tightening rides a schema major
  per `024-L3`), and that until a follow-up lands only the grammar guards a geometry name. Two legs
  of that sentence are not backed by anything the tree carries, though: the bullet calls capping
  geometry names "that format's named follow-up" and `is_slug_name`'s doc comment calls it "the
  deferred follow-up it states", but no follow-up register names it — the proposal's Deferred section
  names only the Windows reserved-device-name refusal (`02-proposal.md:336-353`, ": no follow-up yet"),
  and no ADR in `docs/adrs/` and no entry in `docs/tasks/` mentions a geometry-name length cap. A
  conformance reader is pointed at a follow-up that has no owner or handle. Second leg: ADR-024's
  `024-B1` still enumerates "a geometry name" among "the single shared bound every slug-grammar id
  rides", unqualified, and this run's completion flips that ADR `proposed` → `accepted` with the
  enumeration intact, so the accepted normative unit overstates what the reference implementation and
  now the README enforce. Not a major: the divergence is visible at the bound's stated home, is
  descriptively accurate against the code, and the behaviour is pinned as known current behaviour by
  `crates/core/tests/placement.rs:323-333`.
- **Fix:** Give the follow-up a referent — one line in `03-build-notes.md`'s deferred list (or a
  `docs/tasks/` stub) naming "cap `keyboard-geometry/v1` geometry names at `MAX_SLUG_OCTETS` at that
  format's next major" as the follow-up the bullet cites, and cite that handle from the README bullet
  instead of the self-referential "that format's named follow-up". Ask the ADR owner to annotate
  `024-B1` (or record the divergence in its acceptance note) so the accepted unit and the corpus state
  one scope.

### F3 — Minor: widened refusal-text surface in `check_pin` and `load_policy` remains, framing still not restored

- **Severity:** minor
- **Location:** `crates/core/src/document.rs:1232-1243` (`check_pin`), `crates/core/src/policy.rs:50-65`
  (`load_policy`) — unchanged by this revision
- **Problem:** Carried forward unchanged from round 1: the proposal's only mandate for these strings
  was that a message embedding the number be built with `format!` from the exported constant
  (`02-proposal.md:125-127`), which both sites honor (`document.rs:1241`, `policy.rs:61`), but the
  implementation also splits fault kinds and reorders message shapes that no contract asks for. A
  past-bound `for` head is still refused as "a manifest id is at most 128 octets: `<head>`", bypassing
  the local `fault` wrapper its sibling arms use, so it no longer names the value as a manifest pin or
  echoes the whole pin. Accepted/rejected sets are unchanged (both arms refuse the same inputs,
  grammar before bound, before any file read) and the shapes are pinned by
  `crates/core/tests/policy.rs:874-919` and `crates/core/tests/chain_ops.rs:692-716`, so this is a
  widened rejection-text surface, not a behavior break. Whether it is declared as a deviation could
  not be checked by this seat (checklist item 3, input closed).
- **Fix:** No code change sought: either confirm the refusal-message restructure (now across
  `action.rs`, `manifest.rs`, `document.rs` ×2 and `policy.rs`, including `check_pin`'s loss of the
  pin-level frame) as a declared deviation in `03-build-notes.md`, or restore the
  "`<text>` is not a manifest pin: …" frame around the new bound message so it names the entry like
  its siblings.

## FYI (no severity, out of specialty)

`schemas/action-ids/v1/spec.md:59-62` now hard-wraps mid-sentence ("…canonicalization of / any kind,
so / two spellings never name one action.") — renders fine, reads oddly in the source.
