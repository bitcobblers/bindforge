# Code review — compatibility, round 1 pass c5 (ratification)

- Design: `docs/designs/008-rust-for-the-product/02-proposal.md`, read against
  `docs/adrs/008-rust-for-the-product.md`.
- Change under review: `git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD`
  with `docs/designs` excluded — source, tests, `schemas/`, workflows, root
  cargo/toolchain/deny files.
- Question for this pass: did the revisions that landed after the r1 approval
  break what this seat approved?
- Verdict: **approve**. No blocker or major finding. Three findings below, all
  minor/nit, and all of them are consequences of code the fix commits *added* —
  nothing the r1 approval covered was broken.

## r1 items re-checked (nothing regressed)

| r1 finding | State at HEAD |
|---|---|
| minor — `quoted()` pushed control chars the reader rejects (round-1 `crates/core/src/document.rs:243-258`) | **closed.** `document.rs:298-316` now emits `\b`, `\f`, and any other control char or `\u{7f}` as `\uXXXX`, and the reader decodes them (`toml.rs:222-237`, `250-265`). `schemas/profile/v1/fixtures/valid/escaped-strings.toml` pins the `\b`/`\f` round trip in the corpus. The `\uXXXX` writer branch is itself still uncovered — finding 1. |
| minor — escape set unpinned; `\uXXXX`/`\UXXXXXXXX` refused while TOML v1.1 allows them | **closed in direction, still unpinned.** The reader now accepts `\uXXXX`/`\UXXXXXXXX` and refuses surrogates and past-`U+10FFFF` (`toml.rs:230-231`, `247-265`), and both `spec.md` files now state the exact permitted set and close it ("and no other", `profile/v1/spec.md:16-17`, `reach-classes/v1/spec.md:26-27`). No fixture or test exercises that family — finding 1. |
| minor — `cargo clippy` runs without `--locked` while README/repo-practices promise CI uses `--locked` | **unchanged, and design-faithful.** `check.yml:28-33` matches the proposal's Gates block verbatim, and `tests/gate_contract.rs:195-205` pins those three strings, so this can only be repaired from the proposal, not from the workflow. Not a regression; not re-raised. |
| nit — no `.gitattributes` line-ending policy | **unchanged**; carried over, slightly more load-bearing now — finding 3. |
| nit — unreachable case-collision check (`conformance.rs:291-312`) | **unchanged.** `common/mod.rs:352-366` still forces lowercase, so the dedup is a belt-and-braces guard; `conformance.rs:269-273` now says the lowercase rule is the collision guard. Acceptable. |
| nit — `rust-toolchain.toml` pins `channel` only, no `components` | **unchanged**; `rust-toolchain.toml:6-7`. Not a regression. |

The grammar closure the fix commits added is, from this seat, a *net*
compatibility improvement: the subset is now stated in the contract tree rather
than only in the reader (`schemas/README.md:31-39`, both `spec.md` grammar
sections), and each excluded construct has an invalid fixture pinned by
`tests/grammar_closure.rs:125-152`. Accepted documents remain a subset of legal
TOML v1.1.0, so the README's promise still holds.

## Checklist walk

1. **Platform code vs the Compatibility matrix** — no `cfg`/`target_os`/
   `target_family` anywhere in `crates/`; the only environment access is
   `std::env::temp_dir()` for scratch directories (`tests/common/mod.rs:177`),
   which is correct on both claimed contributor platforms. CI is
   `ubuntu-latest` only, matching the matrix's "CI runner (Linux) yes"; the
   Windows leg is the proposal's own dated rollout check (Test #7), not a
   silently-skipped test.
2. **Node floor** — `check.yml:48` installs `22.23.4`, and
   `tests/gate_contract.rs:296-314` pins it to `tools/orchestration`'s
   `engines.node` read from the package, so the one floor this design sets
   cannot drift. Nothing else introduces a Node version.
3. **Paths** — every path is built with `Path::join`
   (`common/mod.rs:60-61`, `126-135`, `reach.rs:108`); no string concatenation
   of separators. Case-sensitivity: fixture names are constrained to lowercase
   ASCII (`common/mod.rs:352-366`, asserted in `conformance.rs:274-313`), which
   is what keeps the corpus identical on a case-insensitive Windows checkout.
   The one remaining case assumption is the reach-table file-name rule —
   finding 2.
4. **Guarded platform imports** — nothing platform-only is imported;
   `bindforge-core` depends on nothing (`crates/core/Cargo.toml:9-10`).
5. **Locale reality** — the reader takes non-ASCII literally
   (`toml.rs:242`) and the writer emits it literally (`document.rs:311`), so an
   id like a non-Latin binding name survives both directions, and rejections
   truncate by `char` rather than by byte (`error.rs:141-155`), so a non-ASCII
   value cannot panic or split mid-sequence in a log line. What is missing is
   any contract document that proves it — finding 1.
6. **Native dependencies** — none introduced: `Cargo.lock` holds exactly
   `bindforge-core`, so `--locked` is trivially satisfiable and there is no
   packaging story to state. `cargo-deny` is a CI tool on the scheduled job
   only, built on its own `SCAN_TOOLCHAIN` (`advisories.yml:45`, `61-65`),
   which keeps the product's 1.85 pin out of the scan and does not add anything
   to the PR gate or to a contributor's machine.
7. **Tests on claimed platforms** — no `#[ignore]`, no `cfg`-gated test, no
   subprocess, no network; every suite reaches the contract tree through
   `env!("CARGO_MANIFEST_DIR")` (`common/mod.rs:50-57`), so the corpus runs
   identically from a Windows contributor checkout and the Linux runner.
   Caveat on my ability to evaluate: this review environment had no shell, so
   I could not run `cargo fmt`/`clippy`/`test` myself and make no claim about
   gate greenness — that stays what the proposal assigns to Rollout 7.

## Findings

### 1. minor — `crates/core/src/toml.rs:230-231, 247-265` + `crates/core/src/document.rs:310` + `schemas/profile/v1/fixtures/valid/escaped-strings.toml`: the escape family the fix added to the contract has no corpus or test coverage

The fix commits widened the *contract*: both `spec.md` grammar sections now
promise nine escapes — `\b \t \n \f \r \" \\ \uXXXX \UXXXXXXXX` "and no other"
(`profile/v1/spec.md:16-17`, `reach-classes/v1/spec.md:26-27`) — and the reader
and writer implement it. But `schemas/README.md:41-43` says the fixtures, not
the Rust, are the conformance currency every implementer runs, and the corpus
pins five of the nine: `escaped-strings.toml` carries `\"`, `\\`, `\t`, `\b`,
`\f` and nothing else. Nothing anywhere (corpus or `crates/core/tests/`)
exercises `\uXXXX`/`\UXXXXXXXX` decode, the surrogate and past-`U+10FFFF`
refusals in `parse_hex_chars`, the "no other escape" half of the rule, or the
writer's `\u{:04x}` branch at `document.rs:310` — grep for `\u`/`\U` across
`crates/core/tests` and `schemas/**` returns nothing. No contract document of
any kind carries a non-ASCII value either: every fixture id, and both shipped
reach tables, are pure ASCII. So a third-party reader that gets `\u00e9` or
`\ud800` wrong passes the conformance suite, and the "non-ASCII names survive
the format" property this seat's checklist asks about remains unproven in the
one artifact the contract makes authoritative.

Fix: extend the corpus in the same directories — a `profile/v1/fixtures/valid/`
document whose `id` mixes a literal non-ASCII name, a `\u00e9` escape, and a
`\u0001` control escape (which round-trips through the writer's `\uXXXX`
branch), plus `fixtures/invalid/` cases for `"\ud800"`, `"\U00110000"` and a
`"\_"` escape; mirror the decode cases with one `grammar_closure.rs` unit test
so the reference implementation is pinned too.

### 2. nit — `crates/core/src/reach.rs:103-131` (with `55-57`): the table file-name rule is unenforced on the one platform where the case distinction disappears

The fix commits added `hand_matches_file_name` and its truth table
(`reach_tables.rs:138-155`), whose comment argues the exact match matters
because "on a case-insensitive filesystem the two names are the same file".
That argument holds for the pure function only: `load_reach_tables` never calls
it. It builds `{hand}.toml` itself and compares only the document's declared
`hand`, so a version directory that holds `Left.toml` instead of `left.toml` —
legal for git to contain, and what a Linux contributor can commit — loads
successfully on Windows (the open resolves case-insensitively, the declared
hand matches) and fails on Linux with an `Io` refusal. Same contract tree,
opposite result on the two platforms the Compatibility matrix commits to, and
the `spec.md:11-13` file-name rule is the thing that goes unenforced exactly
there.

Fix: have the loader read the directory and pass the on-disk stem through
`hand_matches_file_name` before parsing it (refusing any `*.toml` in the
version directory whose stem is not exactly `left`/`right`), or state in
`reach-classes/v1/spec.md` that the name check is the caller's job, so a
third-party loader is not silently more permissive on Windows.

### 3. nit — `.gitattributes` (still absent at the repo root): the contract is now explicitly line-oriented, and the CR tolerance is still nowhere in the contract

Carried over unchanged from r1, so recorded as a carry-over rather than a new
break. The fix commits made it a little more load-bearing: both grammar
sections now declare "Line-oriented entries, one per line"
(`profile/v1/spec.md:12`, `reach-classes/v1/spec.md:22-24`), and the corpus is
the contract every implementer runs. A Windows contributor checkout with
`core.autocrlf=true` therefore hands the reader CRLF contract trees, and it
works only because `str::lines()` strips a trailing `\r` and each line is then
`trim`ed (`toml.rs:106-108`) — a tolerance no contract document states, and one
a third-party implementer has no obligation to reproduce. `cargo fmt`'s default
`newline_style = "Auto"` is platform-native on top of that.

Fix: add a root `.gitattributes` (`*.rs text eol=lf`, `*.toml text eol=lf`) and
one sentence in `schemas/README.md` stating whether a reader must accept CRLF
line terminators.

## FYI (out of specialty, no severity)

`Cargo.toml:12` declares `members = ["crates/*"]` where the proposal's Workspace
shape writes `["crates/*", "apps/*"]`, and `advisories.yml:83` runs the whole
`cargo deny check` where Gates says `cargo deny check advisories`
(`deny.toml:8-10` also corrects the proposal's `skip` to cargo-deny's actual
`ignore` key). Both look like deliberate corrections of the design text with
the tests pinning the new form (`gate_contract.rs:504-540`, `370-378`); they
belong to the design-conformance seat, not here.
