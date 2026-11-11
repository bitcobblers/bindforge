# Code review — testing seat — round 1, ratification pass c5

Range under review: `git diff 113cb0f5e2a85ca8239b9212be844bb528c0e5d3..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/008-rust-for-the-product.md`.

## Verdict: approve

## What this pass checked

Ratification only: confirm the post-approval revisions did not break what this seat
approved at c4. The reflog shows exactly one commit after the c4 tip `113cb0f5`:
`4c7ff804` "docs: 008-rust-for-the-product code-review stage" — the stage record that
committed the c4 review files. With `docs/designs` excluded, the range carries no
source or test change. That was not taken on the commit message alone: the working
tree at HEAD was re-walked against the exact states this seat verified at c4, and
each still holds:

- `crates/core/tests/common/mod.rs:203-211` — `ScratchDir::write` carries the parent
  guard (`fs::create_dir_all(parent)`, lines 205-208), verbatim the block the build
  notes specified. The c3 ENOENT blocker stays closed.
- `crates/core/tests/conformance.rs:180-211` — `the_zero_fixture_rule_fires_on_an_empty_fixture_directory`
  is intact and now reaches its assertions on every path (the nested writes at
  `:185-186` and `:205-206` are served by the fixed helper). Test strategy #1's
  zero-fixture trigger has a firing trigger test.
- `crates/core/tests/gate_contract.rs:408` — the E0277 fix stands (`deny.contains(&format!(...))`).
- `.github/workflows/advisories.yml` — `cargo deny --format sarif check > cargo-deny-report.json`
  (line 83, `--format` in root position), the `"runs"` content guard at line 95
  (where `gate_contract.rs` and the build notes say it is), SHA-pinned
  `actions/upload-artifact` uploading `cargo-deny-report.json`, `actions: read` scope,
  `timeout-minutes`. `gate_contract.rs`'s pins re-walk against the shipped file and
  match; the toolchain-independence assertions (`SCAN_TOOLCHAIN: "1.88.0"` vs product
  `1.85.0`, `RUSTUP_TOOLCHAIN`) line up with `rust-toolchain.toml`.
- Coverage markers for Test strategy #1-#6 all present (21 `proves:` citations across
  `conformance.rs`, `version_gate.rs`, `chain_guard.rs`, `reach_tables.rs`,
  `gate_contract.rs`); #7 remains the dated rollout check, correctly owned by the
  build-notes checklist rather than a test file.
- Determinism holds: scratch dirs are `label-pid-nonce-nanos` unique (no port/path
  collisions), no wall-clock assertions, no network, no live server.

No checklist item that this seat passed at c4 has regressed, and no new change entered
the testing surface to evaluate.

## Checklist

1. **Proposal Test strategy citations** — PASS. #1-#4 (unit layer) and #5-#6 (code-level
   gate-contract counterparts) each cited; #7 is a rollout check by design.
2. **Unit + integration layers** — PASS. Suites under `crates/core/tests/` mirroring
   `src/` per ADR-004 / Rollout 2, as the proposal places them.
3. **Rejected blockers/majors now pinned** — PASS. The ScratchDir ENOENT blocker's fix
   is in the helper every affected call site uses (c4-verified whole-class sweep holds);
   the E0277 line is fixed; the scan-stream facts are pinned in `gate_contract.rs` in
   either sanctioned spelling.
4. **Determinism** — PASS.
5. **Behavior-named tests** — PASS.
6. **Run the gate yourself** — CANNOT EVALUATE, and that counts as a finding. This
   seat's shell tool is permission-denied for every invocation (three attempts this
   pass: `git diff`, `cargo`/`ls`, and `git show` all refused before executing), so the
   cargo equivalents of `npm run check` cannot be run from here. Nothing in this
   ratification diff could have changed that, and nothing else changed either.
7. **Untestable additions** — PASS; the range adds none.

## Findings

### minor — `.github/workflows/check.yml` rust job / `.github/workflows/advisories.yml` (gate-evidence checklist, `03-build-notes.md:103-120`)
Carried unchanged from c4 — the ratification diff introduced no source or test change,
so this remains the only open item on this seat. The three gate commands and the
`advisories.yml` dispatch still have zero machine evidence: no `test result:` line
anywhere in the run record, because no pipeline stage and this review seat can invoke
a toolchain. It is not stage-actionable; the evidence's generators are the merge PR's
`check.yml` `rust` job (Test #5 Linux leg), a Windows contributor machine (#7), and the
Actions UI (manual `workflow_dispatch`). Fix (unchanged from c4): on the merge PR
carrying this diff, paste each target's `test result:` line (notably `conformance`) and
the command tails into the Test #5 bullet; run the same three once on a Windows
contributor machine for the #7 bullet; do one manual `workflow_dispatch` of
`advisories.yml` with the `"runs"`-marker guard green and the `cargo-deny-report`
artifact uploaded. Rollout 7 stays unticked until those lines exist.

## Verdict

**approve** — zero blockers, zero majors. The single open minor is the same
environment-blocked evidence item this seat accepted at c4; nothing that was approved
was broken by the intervening commit.
