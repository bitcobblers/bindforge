# Security review — code, round 1 ratification pass c5

Seat: security. Pass: ratification (c5). Inputs: `git diff
e5180c2666dccebac365230cb217f9d79dbcd9c9..HEAD -- ':(exclude)docs/designs'`,
`02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`, my own round-1
findings. Out-of-scope per the caller: `docs/designs/**` run records, other
seats' reports.

Note on method: the shell tool is permission-denied in this session, so the
diff text was obtained by two read-only `git diff`/`git diff --name-status`
retrievals (verbatim) and then verified against the files at HEAD with the
read/grep tools. Both retrievals agree on the change set.

## What changed since the round-1 approval

One source/test path, one hunk:

- `crates/core/tests/common/mod.rs:198-211` — `ScratchDir::write` now calls
  `fs::create_dir_all(path.parent())` before `fs::write`, so a suite can lay
  out a nested fixture corpus in any order instead of failing at ENOENT. The
  doc comment records that this ENOENT was what made the zero-fixture rule's
  trigger test panic before it could assert.

Commits in range that touch non-docs paths: one (`test:
008-rust-for-the-product test-update stage`). No `Cargo.toml`, `Cargo.lock`,
workflow, `deny.toml`, `schemas/`, or `src/` file moved in this range.

## Checklist result (security)

1. **Boundary validation of untrusted files** — unchanged by this hunk.
   `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` / `MAX_CHAIN_DEPTH` and the
   `Cycle` / `ChainTooDeep` guards in `crates/core/src/` and their pins in
   `chain_guard.rs`, `parse_budget.rs`, `version_gate.rs` are byte-identical
   to what round 1 approved; the adversarial-guard contract in Chosen design
   → Adversarial guard still holds.
2. **Path handling** — reviewed for the new write path; see the one new nit
   below. All 14 call sites of `write`/`make_dir` in the suites pass
   compile-time relative literals (`profile/v1/fixtures/valid/document.toml`,
   `oversized.toml`, `left.toml`, …); no `..`, no absolute name, no
   untrusted-derived path reaches the helper.
3. **Injection (shell/regex/path)** — nothing new. `gate_contract.rs`, which
   is the file that reads CI YAML and greps it, is untouched, so the
   pinned-in-round-1 guard-of-guards still constrains the workflow text.
4. **Secrets/PII** — none in the hunk; the only strings are a doc comment and
   panic messages that print scratch paths (temp-dir paths the test itself
   created).
5. **New dependencies** — none in the diff; `Cargo.lock` untouched, so the
   round-1 advisory posture (`advisories.yml` + `deny.toml`, A27) is
   unchanged.
6. **Filesystem writes** — still test-only, still under `std::env::temp_dir()`
   with `Drop` cleaning the tree (`common/mod.rs:214-218`). `create_dir_all`
   widens what the helper will create inside that tree, which is the point of
   the fix; no production write path exists yet in this design (ADR-012 owns
   codecs and write policy — Deferred).
7. **Clean rejection on malformed/hostile input** — unchanged; the rejection
   single-line contract test (`conformance.rs:219`) and the invalid-fixture
   corpus are untouched.

Regression check on what round 1 approved: none found. The fix is confined to
a test helper's parent-directory creation and does not weaken any guard,
budget, fixture, CI-pinning or advisory assertion.

## Findings

**nit — `crates/core/tests/common/mod.rs:203-211` — the scratch helper now
creates parents with no confinement check on `name`.**
`ScratchDir` is documented (lines 160-164) as "a scratch directory … removed
when it goes out of scope", i.e. the suites' sandbox, but `Drop` only removes
`self.path`. Since `name` is appended with `Path::join`, an absolute `name`
replaces the base entirely and a `..`-carrying name walks out; before this
hunk, the `fs::write` on such a name usually failed at ENOENT on the missing
parent, so a walk out into a directory that does not exist was caught by
accident. With `create_dir_all(parent)` that accidental fence is gone: a
future suite could silently create directories and write files outside the
scratch tree, where `Drop` will not clean them up — and the same helper is
the one every new suite copies, so the mistake scales. No current call site
does this, and nothing untrusted reaches the helper, so this is hardening,
not a live hole.
Fix: on the next touch of this helper, confine the join — e.g. reject a `name`
that is absolute or contains a `..` component with a panic naming the helper
contract (`assert!(!Path::new(name).is_absolute() && !Path::new(name)
.components().any(|c| c == Component::ParentDir), …)`), or assert the result
still starts with `self.path` before writing.

## Carried from round 1 (not newly broken, not newly fixed)

`crates/core/tests/gate_contract.rs:392-404` is unchanged in this range, so
the residual scoping gap I recorded in round 1 (file-wide marker assertion;
only the literal `[ ! -s` spelling banned) is still open and still a nit. My
round-1 fix deferred it to "the next touch" of that file; this pass touched
`common/mod.rs`, not `gate_contract.rs`, so its moment has not arrived. It is
not a regression from these revisions.

## Verdict

approve — zero open blocker/major findings; one new nit (test-helper path
confinement), one carried nit unchanged.
