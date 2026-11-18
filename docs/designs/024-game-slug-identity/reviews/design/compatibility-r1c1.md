# Compatibility review r1 — dissent pass c1

verdict: approve

Scope: `git diff 3012370e c326ff4a` on `02-proposal.md` (the revision written to
answer this seat's round-1 findings), then the full current document. Checklist:
`references/compatibility.md`. All row/leg claims re-verified against the repo
(read-only) rather than against any other review record.

## Findings

None open. No blockers, no majors, no minors.

## Round-1 finding closures (verified, not re-raised)

1. **Major — third-party schema readers row read `Unknown`.** Closed. The row
   now commits to "Yes, with a known behaviour change" and states the committed
   basis (relaxation-only per 024-L3; an old-text reader refuses only 64–128-octet
   ids, none of which ship at merge, A7). Verified: ADR-024's Consequences
   concedes exactly this cost and calls it acceptable on the same ground; 024-L3
   in the ADR defines the relaxation-only in-place amendment the row cites.
   Checklist item 1 now passes for every row — no `unknown`, no empty cell.

2. **Major — Test strategy #11 / Windows + macOS matrix claims.** Closed via
   option (b) of the named fix. #11 now states the real gate; verified against
   `.github/workflows/check.yml`: both jobs `runs-on: ubuntu-latest`, the Windows
   leg is `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`
   (the workflow's own comment states cargo check does no codegen or linking), and
   the test leg is Linux-only. The Windows row's status ("Compile-verified, with
   a named gap") and the macOS row's ("Yes, no CI leg") now match that
   verification level, and #11's platform-independence assertion holds for the
   design's logic: `derive_slug`, the collision pass and the grammar/bound checks
   are pure string and set operations with no OS surface. The `con`-style
   reserved-name pin runs in the Linux suite.

3. **Minor — orchestration row basis ("No slug-grammar surface there").** Closed
   as proposed: the row now names `SLUG_RE` at
   `tools/orchestration/src/pipeline/types.ts:287` — verified: `export const
   SLUG_RE = /^[a-z0-9][a-z0-9-]*$/`, the run-identity grammar, which admits a
   leading digit and rides no product slug bound. The "Not affected" conclusion
   stands on the corrected basis.

## Regression sweep of the changed text

Checked the other revisions for newly-false compatibility claims; none found:

- The new `steam_common.rs:101` citation is accurate: the "skipped as unreadable
  or malformed" diagnostic is gated on `games.is_empty()`, so it indeed fires only
  when a plugin emitted nothing at all — consistent with count-only plugin-side
  refusals and with 024-L1's named per-game refusal living solely at the CLI.
- The Identity-shape divergence claim is accurate: `Platform` (game.rs) carries
  no serde attribute and `apps/discovery-cli/tests/cli_tests.rs:189` pins
  `"SteamLinux"`; the kebab-case `"unknown"` for `Namespace` is a disclosed,
  committed decision, not a drift.
- The action-ids amendment now covers both spec surfaces A3 cites: `spec.md:54`
  (the 63/RFC 1035 sentence) and `:58` ("a 64-octet segment ... refusal") — both
  verified to exist as quoted; the renamed fixture and its `action_ids.rs` re-pin
  site close the rename's compatibility tail consistently with ADR Rollout 4.
- The revised fold order (Latin table → ASCII case-fold, lower-case ASCII bases)
  is stated consistently across Options, Derivation and Test strategy #1; A12/A15
  claims unaffected.
- Commit-granularity sentences (items 1+2, 4+5) are compatible with 024-L3's
  code-and-spec-atomicity requirement and with dependency order; gate item 7
  unchanged and still matches the real workflows.

## Checklist dispositions

- Item 1 (matrix statuses): pass — every row definite.
- Items 2, 4, 5, 6 (OS input APIs; native modules; keyboard reality; path
  handling): not applicable — the design touches no input API, adds no native
  module or Node surface, and changes no path handling.
- Item 3 (Node floor): not applicable — no Node code changes; floor is
  `engines.node >= 22.23.5` (verified) and the npm gate leg is unchanged.
- Item 7 (a platform test per supported-platform claim): pass as scoped —
  Linux (the only plain "Yes" OS row) has the full suite named; Windows claims
  only compile verification and names it; macOS's qualified claim and #11's
  "no CI leg" are the same committed statement (see FYI).

## FYI (no severity)

- The macOS row's `Yes` has no named runtime test; the row itself and #11 both
  disclose this, and workspace-wide `cargo check --all-targets` covers its
  compilation from the Linux leg, so the claim is committed rather than false —
  flagged for the next ADR that adds a macOS runner, not for this run.
- The corrected `Å`→`a`, `É`→`e` fold order belongs to the correctness seat;
  the compatibility surface (A5's std-only constraint) is untouched by it.
