# complexity review r1 — ratification pass c2

verdict: approve

Scope: only the diff `3012370e → c326ff4a` of `02-proposal.md`, then the full
current document. My r1 verdict carried no findings, so the only admissible
findings here are revisions that broke what I approved or made a section I
cited false. None did.

## Checklist walk-through (against the changed text only)

1. **Reuse — does the repo already have it?** The one changed part that could
   add a mechanism is the plugin-side refusal path, and it *reuses* the existing
   `skipped` counter instead of adding a diagnostic channel: verified
   `crates/discovery/src/steam_common.rs:84` (the counter) and `:101` (the
   `"{} file(s) skipped as unreadable or malformed"` error path, which fires only
   under `!roots.is_empty() && games.is_empty() && skipped > 0`). The revised
   sentence's two claims — the message counts files, and it fires only when a
   plugin emitted nothing — are both true as written.
2. **Every moving part traces to a Goal.** The diff adds no moving part. The
   additions are: a fact correction (A3's second action-ids site), a fold-order
   and case fix, a rationale-consolidation, a diagnostics narrowing, a
   disclosed serde spelling, three Compatibility-matrix status corrections, a
   narrowed Test #11, and Rollout commit granularity. Each maps to an ADR-024
   unit already traced in r1 (`024-B1`/`024-L3` for the spec-example and README
   edits, `024-L1` for the refusal split, `024-C1` for the field). No YAGNI
   entry appears.
3. **A 30–50 % smaller design?** Not applicable to this pass — the revision
   shrank the claimed surface rather than growing it: it deleted the widened
   skip-counter wording, deleted the "suite runs unchanged on the Windows
   matrix" claim, and folded the constant's rationale into one text. Nothing
   here needs a slimmer alternative.
4. **New dependencies:** none added by the diff; still the single
   `discovery → bindforge-core` edge and std-only derivation (A5).
5. **Defer the second half?** Unchanged from r1; the resolver/`publisher` half
   stays with ADR-025 (`024-R1`).
6. **Knobs with no consumer; abstractions with one implementation:** the
   one-variant `Namespace` enum is mandated by `024-C1` (a separate closed field
   whose only v1 value is `unknown`); the diff adds no knob, no layer, no
   second implementation. The new `#[serde(rename_all = "kebab-case")]`
   divergence disclosure is documentation, not a new option.
7. **Value check:** proportionate. The changed text is nearly all accuracy
   work — citations, verification levels, commit boundaries — and buys the run a
   claims-vs-repo match at zero added product surface.

## Corrections I verified as true (the changed claims)

- `schemas/action-ids/v1/spec.md:58` really does say "…and a 64-octet segment are
  each a refusal, and each has a fixture under `fixtures/invalid/` naming it",
  so A3's widened citation and the paired example edit are needed, not extra
  churn. `crates/core/tests/action_ids.rs:124` really does carry the literal
  `"segment-past-63-octets.toml"`, so the new Test #6 / Rollout 1 re-pin site
  names a file that must change; it adds no new file to touch.
- Test #11's narrowed claim matches CI: `.github/workflows/check.yml` runs the
  `rust` job on `ubuntu-latest` only, with a `cargo check --workspace
  --all-targets --target x86_64-pc-windows-msvc` step that runs no test binary,
  and no macOS job. The Windows and macOS matrix rows now state that level
  instead of claiming a runtime matrix the repo does not have — this removes an
  unverifiable promise rather than adding scope.
- `apps/discovery-cli/tests/cli_tests.rs:186-189` pins `obj["platform"]` to
  `"SteamLinux"` with no serde attribute, so the disclosed divergence is
  accurate and the value to build (`"unknown"`) is settled rather than left to
  implementation guesswork.
- `tools/orchestration/src/pipeline/types.ts:286-287` defines
  `SLUG_RE = /^[a-z0-9][a-z0-9-]*$/` — leading digit admitted, no octet bound —
  and a search of `tools/orchestration/src` finds no `MAX_SLUG`, `is_slug`, or
  63-octet reference, which supports the "Not affected" commitment.
- A2's four private constants remain exactly four
  (`manifest.rs:63`, `policy.rs:37`, `document.rs:220`, `action.rs:38` at 63),
  and `schemas/README.md:127-129` still says "all eleven constants", so the
  twelfth-export arithmetic in the unchanged neighbouring text still holds.

## Findings

None. No blocker, major, minor, or regression: the revisions did not break
anything approved in r1, and did not make any section cited in my r1 review
false.

## FYI (no severity)

- Ratification-scope notes for other seats, raised nowhere as findings: (a)
  narrowing plugin-side refusals to count-only means a row refused at
  construction never gets the per-game named stderr line that `024-L1` pairs
  with "a violation is a named refusal"; the named line now exists only at the
  CLI boundary — that reading of the law is the correctness seat's call. It also
  means a sole refused game can trip the `games.is_empty() && skipped > 0`
  plugin error path whose message says "unreadable or malformed". (b) The JSON
  row now deliberately mixes closed-enum spellings (`"platform":"SteamLinux"`
  vs `"namespace":"unknown"`), a wire-format consistency question for the
  compatibility seat. (c) The `placement.rs` doc comment is now instructed to
  copy the README bullet verbatim, where the repo's existing habit is a
  constant's own rationale in its doc comment (e.g. `manifest.rs:54-57`) — a
  two-site text copy that the next bound amendment must move in lockstep; a
  pointer to the README bullet would be marginally smaller, but this is a
  wording preference with no product-surface cost, so it is not a finding.
