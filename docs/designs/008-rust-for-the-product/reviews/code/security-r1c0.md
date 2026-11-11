# Code review — security (round 1, pass c0)

Diff under review: `git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'`
(32 files, +1918; the whole change is new: cargo workspace, `crates/core`, `schemas/` scaffold, two CI workflows, `deny.toml`, docs).

Checklist applied: `references/security.md` (7 items) only.

## Checklist walk

1. **External input validated at the boundary** — mostly satisfied, one gap. The
   untrusted path is `document::parse` → `toml::read_document`: a 1 MiB cap is
   applied before any parsing (`crates/core/src/toml.rs:39-43`), nesting is
   bounded at 16 levels (`toml.rs:229-233`), the grammar refuses table headers
   and dotted keys (`toml.rs:52-58`), duplicate keys are refused
   (`toml.rs:75-80`), control characters inside strings are refused
   (`toml.rs:176-178`), and nothing evaluates or interprets input. Gap: the
   per-format grammar is not closed — unknown entries pass (finding 2).
2. **Path handling** — clean. Every path the diff builds is from fixed
   components: `schemas_root()` derives from `env!("CARGO_MANIFEST_DIR")`
   (`tests/common/mod.rs`), `load_reach_tables` joins `{left,right}.toml` from
   a closed enum (`reach.rs:78-79`), and the harness walks directories rather
   than opening names from document content. No `..`, no absolute path taken
   from data, no shell string anywhere in the diff. Fixture-name rules
   (lowercase ASCII, case-unique) are pinned by a test, which also removes the
   case-collision class on Windows filesystems. Path canonicalization of
   profile-supplied paths is explicitly ADR-012's (Deferred), so its absence
   here is by design, not a gap.
3. **Injection** — no shell, no regex engine, no template sink. The one live
   sink is the error text that ends up in CLI/hub log lines; untrusted strings
   reach it unbounded and newline-capable (finding 3). CI `run:` blocks contain
   no interpolated input.
4. **Secrets and PII** — none in source, fixtures, fixtures' comments, docs, or
   workflows. No tokens are read by either workflow; no manifest field that
   identifies a user or host is implemented yet (ADR-009's, Deferred).
5. **New dependencies** — verified against live data, no unfixed advisory:
   `Cargo.lock` gained exactly one entry (`bindforge-core`, path member), and
   `crates/core/Cargo.toml` declares no dependencies, so the workspace pulls no
   third-party Rust code. `serde` in `[workspace.dependencies]` is declared but
   unused, hence not in the lock. The one tool the diff installs is
   `cargo-deny 0.20.2`: not yanked, checksum published, crates.io MSRV metadata
   `1.88.0`, and OSV/GHSA report **no** advisory for `cargo-deny`,
   `actions/checkout`, or `actions/setup-node`. Both third-party actions are
   pinned to full commit SHAs, and both SHAs resolve to the tag in the comment
   (`11bd7190…` → `v4.2.2`, `49933ea5…` → `v4.4.0`). The pin is nevertheless
   broken at build time — finding 1.
6. **Filesystem writes** — vacuous: nothing in the diff writes. `serialize` has
   no writer beside it, and write policy/atomicity is ADR-012's (Deferred).
   Nothing to evaluate, so nothing to grade.
7. **Clean rejection** — good shape: no `unwrap`/`expect`/`panic!`/`unsafe`
   anywhere in `crates/core/src`, every refusal is a typed `CoreError`, the
   parser's recursion is bounded by the depth guard, and the invalid-fixture
   corpus is asserted to reject. Not verified: nothing pins the size and
   nesting budget (finding 5).

## Findings

### 1. major — the repo's only dependency-advisory control cannot run in this checkout

`.github/workflows/advisories.yml:27-30`

```yaml
      - name: Install cargo-deny (pinned)
        run: cargo install cargo-deny --version 0.20.2 --locked
      - name: Scan Cargo.lock for advisories
        run: cargo deny check advisories
```

`rust-toolchain.toml:7` pins `channel = "1.85.0"`, and rustup applies the
toolchain file from the working directory — which after `actions/checkout` is
the repo root, where that file sits. So the `cargo install` step runs under
rustc 1.85.0. crates.io reports `rust_version: 1.88.0` for cargo-deny 0.20.2
(and for every release from 0.18.4 through 0.20.2); cargo refuses to build a
package whose declared `rust-version` exceeds the active toolchain. The install
step therefore fails and `cargo deny check advisories` never executes.

This is the security-relevant part, not just a red job: per Chosen design →
Gates this workflow *is* the commitment ("the only recurring look at the
lockfile"), and it is also the named trigger for the `rust-toolchain.toml`
security-advisory bump. Because no PR depends on it, a permanently-failing
schedule silently removes all dependency-advisory coverage — precisely when
`axum`/`sqlx`/`toml_edit` arrive and `Cargo.lock` stops being one local crate.
The design's A25 MSRV sanity check was applied to the workspace dependencies
but never to the scanner tool itself.

Fix: keep the product pin and run the tool on its own toolchain —
`rustup toolchain install 1.88.0` then
`RUSTUP_TOOLCHAIN=1.88.0 cargo install cargo-deny --version 0.20.2 --locked`
and `RUSTUP_TOOLCHAIN=1.88.0 cargo deny check advisories` (or the `cargo +1.88.0`
form), with a header comment recording that the tool's toolchain is deliberately
independent of the product pin. Alternative: pin `0.18.3`, the newest release
whose MSRV is 1.85.0, and state the staleness bound next to it. Either way,
add a step that fails loudly if no scan report was produced, so a dead control
cannot idle unnoticed.

### 2. major — the grammar is not closed: untrusted documents carry entries the contract says must be rejected

`crates/core/src/document.rs:77-109`, `144-157`, `159-201`

`parse` reads `format`, `version`, then per format only `id`/`derives`
(`parse_profile`) or `hand`/`classes` (`parse_reach_classes`); every other
entry that `toml::read_document` collected is silently dropped, and parse
returns `Ok`. The contract shipped in the same diff says the opposite, in both
formats: `schemas/profile/v1/spec.md:22` and `schemas/reach-classes/v1/spec.md:30`
— "Any other entry is rejected."

Why this is a security finding rather than a style one:

- The whole point of the adversarial guard (Chosen design → Adversarial guard)
  is that `bindforge-core` is the validator the hub runs on uploads with the
  same compiled types the client uses. A validator that accepts entries outside
  its grammar admits unvalidated data into the trust boundary: an uploader can
  attach arbitrary keys, and whatever component later reads the document — a
  future codec, a diff view, a manifest writer — decides what they mean.
- It fails open on the guard's own input. `derives` misspelled or moved under
  an attacker-chosen alias (`derives_ = [...]`, `Derives = [...]`) parses as
  *no ancestors at all*: the profile is accepted with an empty chain, so
  `Cycle`/`ChainTooDeep` have nothing to reject and a preset-derived profile is
  silently reinterpreted as a standalone one.
- No fixture covers it. `conformance.rs` asserts only that shipped invalid
  fixtures are rejected; none has an unknown entry, so the corpus that is
  "the contract for that behavior" cannot catch this, and third-party readers
  written from `spec.md` will reject documents the reference implementation
  accepts — the conformance divergence ADR-008 clause 3 exists to prevent.

Fix: give each format a closed entry set alongside its grammar (profile:
`format`, `version`, `id`, `derives`; reach-classes: `format`, `version`,
`hand`, `classes`), and after the required entries are read reject any key
outside it with a typed refusal — e.g. `CoreError::InvalidValue { key, message:
"not part of the profile/v1 grammar" }` or a dedicated `UnknownEntry` variant.
Then pin it in the corpus with
`schemas/profile/v1/fixtures/invalid/unknown-entry.toml` and
`schemas/reach-classes/v1/fixtures/invalid/unknown-entry.toml`, which the
existing walk will pick up with no harness change.

### 3. minor — rejection text echoes untrusted strings verbatim: newline-forgeable and length-unbounded

`crates/core/src/error.rs:63-85`, `94`; with `crates/core/src/toml.rs:160-167`,
`crates/core/src/document.rs:104-107`, `174-181`, `crates/core/src/reach.rs:80-83`, `96-99`

The `Display` impl interpolates attacker-controlled content directly: the
declared `format`, the declared `version`, entry key names, the offending
`hand` value, and `Io.path`. Nothing bounds those strings, and the grammar
actively decodes escapes — `toml.rs:163` turns `"a\nb"` into a real newline —
so one hostile upload yields a rejection message that spans lines:

```toml
format = "profile\n2026-11-11T03:17:00Z INFO hub: upload accepted id=fake"
```

The same arm can emit up to the full 1 MiB budget in one message, and
`CoreError::Io` puts absolute local filesystem paths into the string that the
hub or CLI shows or logs. Net effect on the "same reason for the same document"
promise: log-line forging, unbounded log volume per request, and local path
disclosure to whoever receives the error.

Fix: route the untrusted interpolations through one helper that truncates to a
short bound (~80 chars, with a `…`) and escapes control characters and
backslashes before writing, and keep `Io.path` out of messages returned to
uploaders (log it server-side, return the reason). Add an invalid fixture whose
`format`/`version` contains a `\n` escape and assert the rendered message is a
single line within the bound.

### 4. minor — the input budget the contract documents is not reachable by the entry points that must enforce it

`crates/core/src/lib.rs:21-28`; `crates/core/src/toml.rs:23-26`;
`crates/core/src/reach.rs:80`

`MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH` are `pub`, but `mod toml;` is
private and `lib.rs`'s re-export list does not include them, so a caller
outside the crate cannot see the limit it must respect — while `schemas/README.md`
advertises the budget as the thing third-party implementers conform to, and
`MAX_CHAIN_DEPTH` (the sibling guard) *is* exported. The consequence is an
enforcement-order problem on the untrusted paths: the cap is applied inside
`read_document`, after the caller has already materialized the whole document,
so a hub upload handler has no constant to check a `Content-Length` against and
`load_reach_tables` itself calls `fs::read_to_string` (`reach.rs:80`) with no
size check at all — an oversized file in a served or synced tree is read fully
into memory before `parse` can refuse it.

Fix: re-export both constants from `lib.rs` next to `MAX_CHAIN_DEPTH`, and give
callers a capped entry point (e.g. `parse_reader(Read)` that takes
`MAX_DOCUMENT_BYTES + 1` bytes and refuses if more is available, and a
`parse_file` that metadata-checks length first) so the budget binds *before*
the read rather than after it. Note the pre-read check in the hub's upload
policy (ADR-014) in `schemas/README.md`'s budget section so the requirement
travels with the number.

### 5. minor — nothing pins the size and nesting budget, so the guard can silently vanish

`schemas/README.md:44-49`, `schemas/profile/v1/spec.md:44-46`;
`crates/core/src/toml.rs:39-43`, `227-233`; absent from
`crates/core/tests/`

The budget is stated three times in the contract tree as a rejection every
implementer must perform, and the design makes the invalid-fixture corpus the
contract for rejection behavior. Yet no test in the diff — unit or fixture —
ever produces a document over 1 MiB or nested past 16 levels. `chain_guard.rs`
covers the chain half; the two parse-budget guards have zero coverage, so
deleting the `text.len()` check or raising the depth bound would leave the gate
green while making the advertised limit a lie. A checklist item I cannot
confirm working counts against it, and here the code that implements it is
untested.

Fix: add a `crates/core/tests/parse_budget.rs` unit suite generating inputs
in-memory (no repo fixture bloat): acceptance at `MAX_DOCUMENT_BYTES` and
`MAX_NESTING_DEPTH`, refusal with `DocumentTooLarge` at `+1` bytes, refusal with
`NestingTooDeep` at `+1` level — asserting the variant and the `limit` field,
against the constants rather than literals, mirroring how `chain_guard.rs`
handles `MAX_CHAIN_DEPTH`. Optionally one small nesting invalid fixture so the
language-neutral corpus carries the rule too.

### 6. nit — neither CI job has a timeout, and the scan's usefulness is not asserted

`.github/workflows/check.yml:19-21`, `27-30`; `.github/workflows/advisories.yml:20-24`

`timeout-minutes` is absent from both jobs. On the PR gate that is a runner-hog
risk (a hostile PR is arbitrary code on the runner); on the weekly job it means
a hung `cargo install` — which compiles the whole cargo-deny graph from source,
hundreds of crates — yields no scan report at all that week. Add
`timeout-minutes` (e.g. 15 for `check`, 30 for `advisories`) and, per finding
1's fix, a final step that fails if the scan produced no report.

### 7. nit — `deny.toml` configures policies the job never runs

`deny.toml:14-36`; `.github/workflows/advisories.yml:30`

The config sets an eleven-license allow-list, `[bans] multiple-versions =
"warn"`, and `[sources] unknown-registry = "deny"` / `unknown-git = "deny"`,
but the workflow invokes only `cargo deny check advisories`. License drift,
duplicate-version drift, and a dependency arriving from an unknown registry or
git source are all configured-as-denied yet never checked — the config reads as
a wider control surface than exists. (The `ignore`-vs-`skip` correction in the
header comment is right, and matches cargo-deny's own key spelling.) Fix: run
`cargo deny check` (advisories + bans + licenses + sources) in the same weekly
job, or drop the sections that stay unrun; both stay non-blocking per Gates.

## What this change gets right (security)

- One untrusted entry point (`parse`) with the budget applied before any value
  is built, a closed recursion depth, and no `unwrap`/`panic!`/`unsafe` in
  `crates/core/src` — hostile input returns a typed variant instead of a
  crash, and the `Cycle`/`ChainTooDeep` guard is iterative with the bound
  applied before traversal (`crates/core/src/chain.rs:26-38`).
- Chain resolution is O(derives) with a `HashSet` seen-check, so the
  self-deriving and over-deep cases are refused rather than recursed into —
  exactly the precondition the ADR names for hub-side upload validation.
- Zero third-party Rust dependencies in the lockfile, and CI pins actions to
  verified commit SHAs with job-level `permissions: contents: read`, `on:
  pull_request` (not `pull_request_target`) so fork code runs without secrets,
  and keeps advisory scanning off the PR path for the reason Gates gives.
- Fixture-name constraints (`conformance.rs`) close the case-collision class on
  case-insensitive filesystems; the version gate fails closed on a missing
  `version` (`document.rs:82-88`) rather than defaulting to the fixture's major.

## FYI (outside this seat)

`Cargo.toml:7` declares `members = ["crates/*", "apps/*"]`, and no `apps/`
directory exists in the tree — worth a correctness/CI seat's eye, since it is
the first argument every gate command reads.

Verdict: **request-changes** (2 major, 3 minor, 2 nit).
