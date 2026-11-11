# Complexity review r1 (dissent pass c1)

verdict: approve

Scope reviewed: `git diff 04a4ae1a 6e4ea214` on `02-proposal.md`, then the full current
document, plus `docs/adrs/008-rust-for-the-product.md` and the Revision log.

## Closure check of my r1 findings

- **[closed] Formats-contract scope (was major).** The harness paragraph now says "The
  scaffold ships fixtures for `profile/v1` and `reach-classes/v1` only; the `patch`,
  `manifest`, and `policy` directories arrive with ADR-009 and the same walk picks them
  up with no harness change", and Rollout 3 and Test #1 match. No new mechanism was
  invented to do it; the existing glob walk carries it.
- **[closed] Config knobs / deps with no consumer (minor).** `[profile.dev.package.sqlx-macros]`,
  `clap` and `toml_edit` are gone from root `Cargo.toml`, replaced by a comment naming
  ADR-012/hub era as their entry point.
- **[closed] Hub-era sqlx mechanism written down here (minor).** The canonical-sqlx
  paragraph, its Risks row and Test #7 are gone; the decision stays in Options
  considered, and Deferred now names "the sqlx offline-cache mechanism and the hub-path
  `cargo sqlx prepare --check` job". A4/A5 survive only because the Options entry cites
  them.
- **[partly closed] Packaging surface outside the Goals (minor).** Flatpak option entry
  and the Windows packaging detail are gone, A6/A17/A18 deleted, A16 narrowed, and every
  surviving assumption is cited by live text. One new row was added in the same
  paragraph — see finding 5.
- **[closed] `MAX_CHAIN_DEPTH` dual ownership (minor).** The guard paragraph now says
  "This design decides that input budget, the two error variant names, and the depth
  bound — constraints ADR-009 must adopt, with 32 an initial value it may tune but not
  remove; all other chain semantics are ADR-009's", Test #3 is pinned to the constant,
  and Deferred says "chain semantics other than the depth guard and its boundary
  fixtures". One ownership question answered; the added budget wording brings a new gap
  — finding 2.
- **[closed] Empty lint tables (nit).** Tables deleted; the inheritance sentence now
  claims only edition, `rust-version` and dependency versions.
- **[closed] `crates/transforms` as an empty member (nit).** Creation moved to ADR-009;
  the `crates/*` glob needs no change when it lands.

## Findings

- [minor][new] **Gates (ADR-008 clause 4) + Rollout 1/6/7 + Test strategy #5** —
  "`cargo deny check advisories` … `deny.toml` is committed at the workspace root" —
  a fourth gate command plus a config file that traces to no Goal and to no clause the
  design is implementing: ADR-008 clause 4 fixes the gate as "`cargo clippy` +
  `cargo fmt` as the `check` gate (mirroring `npm run check`'s role)". The `deny.toml`
  contents are unspecified (advisory allow-list? license set? `[sources]` restrictions?),
  so it is a config knob whose content no text defines (checklist 6); the tool itself is
  a new contributor/CI dependency, yet Rollout 6's prerequisite list names only "rustup
  one-liner and the Linux system-deps list" and Rollout 5 documents "the cargo gate
  commands" without an install step. Coverage is inconsistent too: Test #5's one-gate
  promise still enumerates only `fmt`, `clippy`, `test`, and Rollout 7 verifies exactly
  that test, so the added command is the one gate command nothing checks. At merge the
  dependency graph is `bindforge-core` (path) plus `serde`, so the advisories check has
  almost nothing to bite on. Fix: drop the command and `deny.toml` from this design and
  let the slice that adds real third-party dependencies (ADR-012 codec deps, or ADR-014)
  introduce the gate, the config content and the install step together; if the gate
  stays, state `deny.toml`'s initial content, add the tool install to Rollout 6, and add
  the command to Test #5.
- [minor][new] **Chosen design → Formats contract, Adversarial guard** — "parsing
  rejects documents larger than 1 MiB or nested deeper than 16 table/array levels with
  typed errors; the constants and the budget are documented in `schemas/README.md` so
  third-party readers can conform … the invalid-fixture corpus is the contract for that
  behavior" — the r1 revision added two more normative constants to the public
  conformance contract, but Goal 6 hardens *chain resolution*, not the parser, and
  nothing in the slice exercises them: Rollout 3 ships one valid + one invalid fixture
  for `profile/v1`, and Test #1–#4 never mention size or nesting depth. So the sentence
  asserts the fixture corpus is the contract for behavior no fixture in this design
  pins, while the versioning rule already makes fixture presence the definition of
  "supported" (checklist items 2 and 5 — a second half that can ride with ADR-009).
  Fix: keep the untrusted-input premise and the chain guard, and add one clause — "the
  size and nesting limits arrive with ADR-009's fixture cases" — extending the existing
  Deferred bullet ("… their fixture cases — owned by ADR-009") to cover parse limits;
  alternatively, if the budget must be decided here, add one oversized and one
  over-nested fixture under `schemas/profile/v1/fixtures/invalid/` and one clause to
  Test #1, which replaces nothing and must instead delete the "the invalid-fixture
  corpus is the contract for that behavior" claim about this slice.
- [minor][new] **Rollout step 5** — "the cargo gate commands, the Node floor, the
  capability-file rule (own commands only; no `fs`/`shell`/`dialog` plugins),
  tests-under-each-crate" — two documented items have no referent in this design.
  "the Node floor" is never stated anywhere in the document: the Gates paragraph only
  says the `orchestration` job has a "Node version pinned explicitly in a `setup-node`
  step", so the practice doc is asked to record a number that does not exist (a knob
  with no value, checklist 6). The capability-file rule governs
  `apps/desktop/src-tauri/capabilities/`, which per the members table is created in the
  hub era by the desktop design, so `repo-practices` documents a rule for a tree this
  slice does not create. Fix: cut ", the Node floor, the capability-file rule (own
  commands only; no `fs`/`shell`/`dialog` plugins)" from Rollout 5 and let the desktop
  design add it (its Deferred bullet already owns webview toolchain and packaging
  details); if the capability wording stays in TypeScript boundary as a forward
  constraint, say explicitly that the practice-doc entry ships with `apps/desktop`.
- [nit][new] **Chosen design → Formats contract, tree block vs Harness** — the block
  lists "`patch/v1/spec.md  fixtures/valid/  fixtures/invalid/`" (and manifest, policy)
  while the same section says the scaffold ships `profile/v1` and `reach-classes/v1`
  only and that "a `schemas/*/v*/` directory yielding zero valid or zero invalid
  fixtures fails the run". A contributor who creates the tree as drawn gets a red gate
  from empty directories. Fix: one parenthetical on the block — "end state; Rollout 3
  creates `profile/v1` and `reach-classes/v1` only" — no change to the zero-fixture
  rule, which usefully prevents a vacuous pass.
- [nit][new] **Compatibility matrix** — "| Steam Deck / SteamOS | no (v1) | no packaged
  artifact targets the read-only image; scanner work assumes standard prefixes (A19) |"
  — the r1 collapse removed packaging detail from the other desktop rows but added this
  new v1 exclusion decision for surfaces this design hands to the desktop design and to
  ADR-012's scanner, and unlike Flatpak, macOS and mobile it has no matching Deferred
  bullet, so the decision is stated but not tracked. Fix: add "Steam Deck / SteamOS
  packaging" to the existing Deferred bullet "Flatpak viability, macOS support, Tauri
  mobile targets — no follow-up yet", or drop the row into the existing
  "Flatpak, Android / iOS" row's cell.

## Value check (checklist 7)

Proportionate. The revision is net-negative in complexity: the four-format fixture
obligation, three dependency entries, a build-profile block, two empty lint tables, an
empty workspace member, the hub-era sqlx mechanism, the Flatpak option entry and three
orphaned assumptions are all gone, and each closure reused an existing mechanism (the
`schemas/*/v*/` walk, the `crates/*` glob, the Deferred list) rather than adding a
replacement. The four residue items above are one CI tool, two parse-limit constants,
two practice-doc bullets and one matrix row — worth trimming, none of them structural.

## FYI (no severity)

- Test #2's new assertion — "the majors `bindforge-core` declares equal the majors
  present under `schemas/*/v*/`" — only works if the supported majors are data `core`
  exposes rather than match arms inside `resolve`/parse; worth a clause for the
  correctness seat, and it slightly overlaps the "presence in `vN/fixtures/valid/` is
  the definition of 'supported'" rule.
- `license = "Apache-2.0"` is now justified by the root `LICENSE` file while A23 records
  `tools/orchestration/package.json` as MIT; the design calls that a repo fix outside
  its scope — correct that the ADR does not cover licensing, but a non-license-carrying
  workspace member is a compatibility/conformance loose end.
- `resolver = "3"` on a virtual workspace is correct for edition 2024 at 1.85; only the
  correctness seat can confirm the A7 citation covers resolver inheritance.
