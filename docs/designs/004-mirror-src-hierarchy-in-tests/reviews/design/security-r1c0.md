# security review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`), inputs limited to the proposal, its
Revision log, and `docs/adrs/004-mirror-src-hierarchy-in-tests.md`. Revision log
(r1) records no prior rejections, so every finding here is `new`.

## Findings

- [major][new] Test strategy #4 (with Chosen design ¶3) — "pass unmodified,
  demonstrating `TEST_PATH_RE` and ownership checks treat nested paths
  identically" — the acceptance evidence cannot support the security-relevant
  half of this claim. In this pipeline the test-path gate and the ownership
  check are the mechanism that decides which files count as tests and which
  stage may touch them, i.e. a privilege boundary, and the proposal's case that
  nesting does not move that boundary rests on one quoted regex (A7, covering
  `TEST_PATH_RE` at two call sites) plus these tests passing. The tests are the
  weak part: A14 states the gate/ownership tests' path strings are fixtures
  ("other in-test path mentions are inert fixture data"), and Deferred keeps
  them verbatim in the old flat form ("Refreshing fixture path strings that
  mimic the old naming (e.g. `pipeline-lane.test.ts` at
  `pipeline-independence.test.ts:145`)"). After the move those tests receive
  byte-identical inputs, so a green run is a no-op signal about nested-path
  classification — it proves the unchanged inputs still behave, not that
  `tests/pipeline/gates.test.ts` is classified as a test path and owned by the
  same stage as before. Nothing in the assumption register covers any path
  pattern other than `TEST_PATH_RE`: A7 is the only path-literal assumption, A8
  covers import specifiers, A11–A13 cover agent globs and prompt/skill prose. A
  pattern keyed to the flat `pipeline-*` test filename would stop matching
  silently, and `npm run check` (Test strategy #5) would stay green — fail open.
  Fix: replace the demonstrative claim in #4 with an assertion over a real
  nested path (classify the moved file's own path, e.g.
  `tools/orchestration/tests/pipeline/gates.test.ts`, through the same gate and
  ownership call paths A7 cites), and add one assumption row recording the
  static enumeration of every path pattern/literal in the gate and ownership
  code checked against the nested form. Fixture refresh stays deferred; the
  claim must not depend on it.

- [minor][new] Chosen design ¶3 — "No config change (Decision, bullet 3)" —
  the document contains no trust-boundary statement anywhere, so checklist
  items 1, 2 and 4 cannot be evaluated from it: nothing classifies the inputs
  the change moves or leaves in place (repo files the pipeline reads, git
  metadata, agent permission grants), nothing says no untrusted data is parsed,
  and nothing says nothing leaves the machine. For a mechanical move the
  correct closure is one sentence, not a mechanism; its absence is why a later
  reader cannot tell whether the design considered that the paths it relocates
  are exactly the paths a gate and an agent grant match against. Fix: add one
  line to ¶3 (or Non-goals): the change moves only repo-local, trusted files;
  no untrusted/network/shared-binding input is parsed by any part of it; the
  only security-relevant consumers of the moved paths are the in-repo path gate
  and the test-engineer agent's existing `**/tests/**` grant, and both keep the
  same scope before and after.

- [minor][new] Chosen design ¶2 / Rollout 2 — "rewrite every `../src/...`
  specifier to `../../src/...` (A8)" — the mechanism is unspecified, and the
  evidence behind it is narrower than the instruction. A8's basis is an
  import-scoped grep (`from ['"]\.\.?/`), which says nothing about
  `../src/`-shaped substrings appearing in fixture path strings — which is
  precisely what A14 and Deferred confirm gate and ownership tests contain. A
  content-wide substring rewrite therefore can change gate test *inputs* while
  `npm run check` stays green; the only guard the design offers is the manual
  diff read in Risk row 3 ("review checks the diff contains only renames,
  import-depth lines, and that one comment line"). Fix: state the rewrite as
  specifier-scoped using the same pattern A8 greps (`from '../src/...`), naming
  the substring substitution it replaces, and extend the ¶1 diff check in Test
  strategy to assert that no line outside import specifiers and the one
  carve-out comment changed.

- [minor][new] Rollout 1 — "Confirm no orchestration run is mid-branch against
  `tools/orchestration/tests/`" — the pre-condition that keeps the tree from
  ending up half-mirrored is a human confirmation with no mechanical check, and
  A18 marks it unverified ("must be checked at rollout"). If it slips, the
  post-commit tree carries renames plus newly authored old-form files: a
  partially applied move, which is the state the path gate and ownership check
  classify inconsistently, and neither Test strategy #2 nor #3 detects it (both
  count files and would pass at any total they are told to expect). Fix: add a
  fail-closed post-condition rather than relying on the check — one static Test
  strategy item asserting `grep` finds zero remaining `tests/pipeline-*.test.ts`
  paths in `tools/orchestration/tests/` after the commit.

## Checklist coverage

1. Trust boundaries — not evaluable from the document (finding 2).
2. Parsing untrusted data — not evaluable; no parsing is described (finding 2).
3. Path handling — `git mv` over 23 known-lowercase ASCII names is canonical,
   no `..`/absolute input, no shell interpolation of untrusted values; the
   open half is the unspecified rewrite scope (finding 3).
4. Secrets/PII — no logging or egress change described; nothing to store or
   redact. Not evaluable as a positive statement (finding 2).
5. Dependencies — checked against OSV: `vitest` — GHSA-5xrq-8626-4rwp
   (CVE-2026-47429) affects <3.2.6 and 4.0.0–<4.1.0; GHSA-82fw-gwwq-j7x9
   (CVE-2026-84373) affects 2.1.0–<4.1.11 and 5.0.0-beta.1–<5.0.0-rc.2;
   GHSA-9crc-q9x8-hgqq (CVE-2025-24964) affects 1.x–3.0.4. The pinned
   `vitest ^5.0.5` (Compatibility matrix) is above every fixed bound, so no
   unfixed advisory. `typescript` and `@trivago/prettier-plugin-sort-imports`
   return no OSV advisories. Node core advisories are outside OSV/npm, and the
   design changes no Node API use. No blocker; the design adds and bumps no
   dependency.
6. Privileges — least privilege holds: nesting stays inside the existing
   `**/tests/**` grant (A11); no new filesystem or device access.
7. Fail closed — atomic single commit is the right shape, but the in-flight-run
   pre-condition has no mechanical guard (finding 4) and the gate-evidence gap
   in finding 1 hides a silent misclassification.

## FYI (no severity)

- Chosen design ¶2's carve-out edits a comment inside a moved file, while
  ADR-004 Decision bullet 2 says "The only content change permitted is the
  import depth". The carve-out is well argued in Options considered, but as
  written the proposal contradicts the ADR clause it cites for that part; the
  Deferred entry declines to touch the ADR only for the "14 modules" typo. ADR
  conformance, not security.
- `TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/` as quoted in A7 has an unanchored
  `\.test\.ts$` alternative, so any `*.test.ts` anywhere (including under
  `src/`) matches it. That breadth predates this change and the move neither
  widens nor narrows it; it belongs in a separate hardening note, not this
  design.
- Compatibility matrix pins the acceptance toolchain to vitest ^5.0.5, whose
  known critical advisories are in the Vitest UI / API server. Rollout 3's
  `npm run check` and `vitest run` start no API server; worth keeping the
  acceptance steps free of `--ui`/`api.host` so the gate itself never opens
  that surface.
