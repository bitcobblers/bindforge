# correctness review r4

verdict: approve

Scope: the caller named this as the single editorial confirmation pass of round 4,
scoped to `reviews/design/editorial-r4.diff`. Inputs used: `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log inside the proposal,
the diff, and my own prior findings (none — the editorial phase is one language
pass per round, and this seat has no earlier correctness report in this phase).
No review reports, transcripts, or unnamed paths were opened.

## Findings

None. No blocker, major, minor, or nit finding in the diff.

Per-hunk check (all four changes are language-only; none changes a decision,
name, number, or obligation):

1. **Goals** — `CI running both toolchains' gates, and a Rust section in
   repo-practices.` Parallelism fix to match the surrounding noun-phrase goals.
   Still matches ADR-008's "CI must run both" consequence and clause 4, the
   Gates section (`check.yml` `rust` + `orchestration` jobs), and Rollout 5
   (`repo-practices` Rust section). No drift.
2. **Gates** — `it commits to the only recurring look at the lockfile: a second
   workflow, .github/workflows/advisories.yml (Rollout 4) …` Replaces `it keeps
   the only recurring look at the lockfile it commits`. The obligation stays
   owned here (not deferred), stays out of the per-PR gate, and still lands in
   Rollout 4 with `advisories.yml` + root `deny.toml`; the `deny.toml` `skip`,
   maintainer triage window, the toolchain-advisory-bump trigger, and the A27
   caveat are untouched. Matches Revision log r1c3. Nothing newly false.
3. **Risks** — `Codec conformance fixtures pin toml_edit's known non-preservation
   gaps (A21) when crates/adapters lands (ADR-012 takes the obligation)`.
   Passive → active; agent, timing (`crates/adapters` lands), and obligation
   owner (ADR-012) are unchanged, and the A21 basis checks out (docs.rs shows
   `toml_edit` 0.25.19+spec-1.1.0 and lists exactly the two non-preservation
   limitations cited). Still consistent with the Deferred entry handing codec
   conformance obligations to ADR-012.
4. **Rollout 5** — `…, and tests-under-each-crate.` Serial-comma/`and` fix; the
   bullet's content still mirrors ADR-008 clause 4 ("tests under each crate")
   and the earlier `tests/`-under-each-crate statement in Rollout 2.

## Checklist status

1. Assumption register — no `verified`/`unverified` row is added, removed, or
   reworded by the diff; the rows the changed text cites (A11, A14, A21, A23,
   A27, A28) are unchanged from the previously reviewed text.
2. URL spot-check (anti-hallucination) — the diff adds no citation; three
   sources behind diff-adjacent claims were re-checked live: A21
   (docs.rs/toml_edit — version, spec, and limitation list match), A14
   (v2.tauri.app/security/capabilities — "What does it **not** protect against:
   Malicious or insecure Rust code" matches the Risks row and the TypeScript
   boundary clause), A12 (rust-lang/cargo#7760 — still open, feature request for
   a `core`-name warning). All live and supporting. A28's basis is a repo file
   not named by the caller, so it was not re-opened.
3. Internal contradictions — none introduced; the four touched passages still
   agree with Gates, Formats contract, TypeScript boundary, Risks, Test
   strategy, Rollout, and the ADR.
4. Missing edge cases — the diff removes no guard (advisory scan, depth/size
   budget, fixture-name rule, capability rule all intact).
5. Interfaces/naming — no symbol, path, or constant renamed; `advisories.yml`,
   `deny.toml`, `check.yml`, `22.23.4`, `MAX_CHAIN_DEPTH` appear once each with
   their previous spelling.
6. Decides what the source ADR asked — unchanged: clauses 1–4 remain decided in
   the same sections.
7. Prior-round blocker/major closures — the r1/r1c3/r2 closures this diff sits
   on (advisory obligation owned here, Node floor single-valued, ADR-014
   handoffs) are still present verbatim in the surviving text; no closure was
   re-opened or reworded away.

## FYI (no severity)

- Gates: `it commits to the only recurring look at the lockfile` uses "commits"
  (pledge) two clauses before `the committed Cargo.lock` (git sense) in the same
  sentence. Meaning is recoverable; whether the word choice is worth another
  pass is the editorial seat's call, not a correctness problem.
- Risks: `Codec conformance fixtures` names ADR-012's fixtures, while Formats
  contract uses `conformance harness` for the `schemas/` harness. The term
  predates this diff; if it ever reads as ambiguous, the disambiguator belongs
  with complexity/clarity, not with a correctness change here.
