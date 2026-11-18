# security review r1 c2 (ratification)

verdict: approve

Scope: the blob diff `3012370e..c326ff4a` of `02-proposal.md`, then the full
current document. Ratification lens only: whether the post-r1 revisions broke
what this seat approved in round 1 or made a section this seat cited false.

## Checklist walk over the changed text

1. **Trust boundaries (plugin output = untrusted).** "Emission, refusal and
   collisions" now makes plugin-side refusals count-only and confines the named
   per-game echo to the CLI boundary "alone". This *reduces* echo surfaces for
   hostile plugin-supplied values: one escaped renderer (`escape_and_truncate`),
   one boundary. The r1-approved property (every row from any plugin checked
   with `is_slug_name` + bound before leaving the plugin or the CLI) is intact.
   Approved.
2. **Verified the new code citation.** `crates/discovery/src/steam_common.rs:101`
   gates the "unreadable or malformed" message on `games.is_empty() &&
   skipped > 0 && !roots.is_empty()`, so the proposal's claim that the
   diagnostic "fires only when a plugin emitted nothing at all" is accurate —
   and the message itself carries roots and a count, "nothing else — no
   contents" (steam_common.rs:99-100), so no hostile value can leak through the
   counter path either. Count-only refusals echo nothing. Approved.
3. **Path handling / fail-closed.** `derive_slug` reordering (Latin fold before
   ASCII case-fold) keeps the output pure ASCII in `[a-z0-9-]` with the `g-`
   prefix rule; no path separator, `.` or `..` shape is reachable, and refusal
   still means the row is never constructed. No new path surface. Approved.
4. **Length bound.** 128 stays a single exported bound with fixtures re-pinned
   at 129; the action-ids example amendment ("a segment past the shared
   bound") matches `schemas/action-ids/v1/spec.md:54,58` as A3 now states.
   Relaxation-only compatibility claim is arithmetically true (63-valid ⊆
   128-valid). Approved.
5. **Compatibility matrix / Test strategy #11.** Windows moves to
   compile-verification and macOS to no CI leg; the security-relevant logic
   (grammar check, escape-and-truncate, traversal exclusion) is platform-
   independent pure Rust, and the `con` reserved-name case stays pinned and
   Deferred consistently with ADR-024's scope. No fail-closed weakening.
   Approved.
6. **`tools/orchestration` row.** `SLUG_RE` verified at
   `tools/orchestration/src/pipeline/types.ts:287` (`/^[a-z0-9][a-z0-9-]*$/`) —
   a distinct pipeline-internal grammar, as claimed; no product slug surface.
   Not a security surface change.
7. **Dependencies.** Still no new crates (std-only fold, no
   `unicode-normalization`); the one new edge is workspace-internal
   (`discovery → bindforge-core`). No advisories to check. N/A beyond that.
8. **Secrets/PII.** Unchanged by the diff; the only stderr echo remains the
   escaped, bounded slug value — no new logging of paths or values.

## Findings

None. No revision in this diff broke a security property this seat approved in
r1, and no section this seat cited was made false.

## FYI (no severity)

- This seat's r1 nit on the CLI-boundary clause ("and no path", overstates the
  echo: `escape_and_truncate` passes `/` and `.` through) survives byte-for-byte
  unchanged at "CLI boundary and output". It is not a regression introduced by
  this revision, is not listed as rejected in the Revision log, and remains a
  wording preference only — carried here as status, not re-raised as a finding
  in this ratification pass. Relatedly, the revised "count-only" text is now
  *more* conservative than r1's wording, which had the widened skip diagnostic
  naming un-derivable names; the r2 version removes that potential echo site.
- Observability note for the correctness seat: with `skipped` surfaced only in
  the all-games-empty error path (verified), a refused row among otherwise-
  healthy results leaves no operator-visible trace; the text is candid about
  this, but the "refusals increment the existing counter" language promises a
  count that is usually unreported.
