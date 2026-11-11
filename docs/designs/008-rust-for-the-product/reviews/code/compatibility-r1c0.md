# Code review — compatibility (round 1, pass c0)

Scope: `git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'`
Reviewed paths: root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`,
`.gitignore`, `.github/workflows/check.yml`, `.github/workflows/advisories.yml`,
`crates/core/**` (`Cargo.toml`, `src/{lib,document,toml,reach,chain,error}.rs`,
`tests/{conformance,version_gate,chain_guard,reach_tables}.rs`, `tests/common/mod.rs`),
`schemas/**` (`README.md`, `profile/v1`, `reach-classes/v1`), `README.md`,
`.opencode/skills/repo-practices/SKILL.md`.

Inputs: the diff, `02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`.
Previous verdict held no findings, so this is a first read.

## Checklist walk

1. **Platform code vs the Compatibility matrix** — matrix commits: CI runner (Linux) yes,
   contributor dev env Linux + Windows yes, macOS no (v1), desktop packaging deferred.
   Nothing in the diff is OS-specific: no `cfg(unix)`/`cfg(windows)`, no OS API call, no
   packaging step, and the cargo gate is plain portable std. CI is Linux-only
   (`ubuntu-latest`), matching the matrix; the Windows leg stays the manual Rollout 7
   verification, as Test strategy #7 says. The toolchain floor is coherent
   (`edition = "2024"`, `rust-version = "1.85"`, `channel = "1.85.0"`, `resolver = "3"`,
   which is stable since Cargo 1.84), so the pinned floor has a code path on every
   platform it claims. No finding, except the un-pinned line endings in finding 4.
2. **Node floor** — no API above the floor is introduced (no TS source changed).
   `.github/workflows/check.yml:42` pins `node-version: "22.23.4"`, which equals
   `tools/orchestration/package.json` `engines.node` (`>=22.23.4`, A28) and the value
   documented in `repo-practices/SKILL.md:58` and `README.md:24`. One floor, one value,
   three places, all consistent. Pass.
3. **Paths** — every filesystem path is built with `Path::join`
   (`tests/common/mod.rs:37-43,71,92`; `src/reach.rs:79`); the only `/`-joined strings
   (`conformance.rs:108,177`) are assertion labels, not paths. Case sensitivity is
   explicitly handled for the Windows filesystem: the harness asserts fixture names are
   case-unique (`conformance.rs:127-167`). The fixture-root walk through
   `env!("CARGO_MANIFEST_DIR")` is cwd-independent (A26) and identical on both claimed
   contributor platforms. Pass, with finding 4 (CRLF reality un-pinned) and finding 5
   (the case-collision guard is unreachable).
4. **Guarded platform imports** — no platform-only module is imported anywhere; nothing to
   guard. Pass.
5. **Locale/layout reality** — non-ASCII ids reach the model as TOML strings written
   literally and survive parse → `serialize` → re-parse (`src/toml.rs:175-178` accepts
   non-ASCII; `src/document.rs:253` writes it back raw), so non-QWERTY / non-ASCII binding
   names work in the BindForge→file direction, and fixture *file* names are pinned to
   lowercase ASCII per the proposal's versioning rule. They do **not** survive the
   third-party→BindForge direction (finding 2), and control-char escapes break the
   round-trip the contract promises (finding 1).
6. **Native dependencies** — none introduced: `Cargo.lock` holds exactly one package
   (`bindforge-core`, no sources/checksums) and `crates/core/Cargo.toml` declares zero
   dependencies. No packaging story, no CI matrix extension, no system library is needed
   for the gate; the README's apt list is explicitly labeled desktop-era and not gate
   required. Pass.
7. **Tests run where the plan claims** — no test is `#[ignore]`d or `cfg`-gated, so the
   harness runs on every platform that runs the gate, not just Linux CI; the conformance
   walk finds the contract tree identically for a contributor and in the gate
   (`CARGO_MANIFEST_DIR`, no cwd convention). Test strategy #5's code-level precondition
   is genuinely exercised; #6/#7 are CI/rollout legs that no test file can run, and the
   harness says so. The only gap is the toolchain-profile dependency in finding 3's
   neighbourhood (finding 6).

## Findings

### 1. minor — `crates/core/src/document.rs:243-258` — `serialize` writes control characters raw, breaking the round-trip the contract promises
`quoted()` escapes only `"`, `\`, `\n`, `\r`, `\t`; every other character is pushed
verbatim. The reader decodes `\b` → U+0008 and `\f` → U+000C (`src/toml.rs:161-162`), so an
id containing one of those parses fine, and `serialize` then emits the raw control byte —
text that the crate's own reader rejects (`src/toml.rs:175`: "control character inside a
string") and that no TOML v1.1 reader accepts (C0 controls other than tab must be escaped).
That is exactly the failure mode of the control-char asymmetry: it silently invalidates the
"valid fixture re-serializes to a semantically equal value" promise (`schemas/README.md`,
`schemas/profile/v1/spec.md:50-51`) for any document whose id carries such a character.
No shipped fixture triggers it, which is why the gate stays green today.
Fix: in `quoted()`, encode any remaining `char < 0x20` (and `0x7F`) as `\u00XX`, and pin the
behavior with a `profile/v1/fixtures/valid/` fixture whose id uses a `\u`/`\b` escape so the
round trip is covered by the corpus, not just by the in-memory test.

### 2. minor — `crates/core/src/toml.rs:155-173` — the v1 grammar never pins the string-escape set, and the reader rejects TOML's `\uXXXX`/`\UXXXXXXXX`
`parse_string` accepts only `\b \f \n \r \t \" \\`; `\uXXXX` and `\UXXXXXXXX`, which TOML
v1.1.0 (A20) allows, are refused with "not an escape this grammar allows". Neither
`schemas/README.md` nor either `spec.md` enumerates the permitted escapes — they say only
"quoted string" — so a third-party implementer cannot tell that escaping is illegal, and a
community tool that escapes non-ASCII when writing (common serializer behavior) produces a
profile `bindforge-core` rejects, while `bindforge-core` itself writes non-ASCII literally.
One half of the interop the contract exists to guarantee is untested in both directions.
Fix: accept `\uXXXX`/`\UXXXXXXXX` (rejecting lone surrogates and code points > U+10FFFF),
or state the exact escape set in `schemas/README.md` + both `spec.md` files and add an
`invalid` fixture for `\u` so the asymmetry is at least part of the contract.

### 3. minor — `.github/workflows/check.yml:25-30` — the `--locked` promise this change writes into the docs is only honored by one of the three gate commands
`README.md:21-23` and `.opencode/skills/repo-practices/SKILL.md:53-58` state "Cargo.lock is
committed; CI uses `--locked`", and the proposal's Workspace shape repeats it (A8, A11), but
only the test step passes `--locked`. `cargo clippy --workspace --all-targets` runs before
it without `--locked`, so a resolution change (a yank, a new patch release, a mirror moving)
rewrites `Cargo.lock` inside the job, and the later `cargo test --locked` then verifies the
already-mutated lock rather than the committed one. The gate can therefore go green on a
dependency graph no contributor has — the "every contributor and CI job builds identically"
floor this design is built on, on every platform. The Gates block lists the three commands
verbatim, so this reads as a design-text gap carried into CI; the CI fix costs one flag.
Fix: `cargo clippy --workspace --all-targets --locked -- -D warnings` (fmt resolves no
graph, so it needs none), or record in Gates + `repo-practices` that only `cargo test` is
locked and say why.

### 4. nit — repo root, no `.gitattributes` — line endings are unpinned on a platform the matrix commits to
The Compatibility matrix commits to Windows contributor dev env, and this change makes
`schemas/**/*.toml` a byte-level-ish contract read by an in-tree parser and by third-party
implementers. Nothing pins line endings, so a Windows contributor with `core.autocrlf=true`
checks out the contract tree and the Rust sources with CRLF. It happens to work — the reader
tolerates CRLF (`str::lines()` + `trim()` in `src/toml.rs:46-48`) — but that tolerance is
nowhere in the contract, and rustfmt's default `newline_style = "Auto"` is platform-native,
so a Windows contributor running `cargo fmt` (not `--check`) rewrites every `.rs` file to
CRLF. Cheap to pin rather than to re-discover during the Rollout 7 Windows leg.
Fix: add `.gitattributes` (`*.rs text eol=lf`, `*.toml text eol=lf`, or `* text=auto
eol=lf`) and one line in `schemas/README.md` stating the line-ending rule implementers must
accept.

### 5. nit — `crates/core/tests/conformance.rs:144-164` — the Windows case-collision guard can never fire
The fixture-name assertion at `conformance.rs:137-141` already requires every segment to be
lowercase ASCII (`tests/common/mod.rs:191-205`), so no file name can survive the first
assert and then collide case-insensitively; the `lowered.sort()/dedup()` check that exists
to keep the corpus green on a case-insensitive filesystem is unreachable. Not harmful, but
the one guard aimed at platform case sensitivity is decoration.
Fix: either drop the dedup check and say the lowercase rule is the collision guard, or let
the name rule accept case-unique mixed-case names so the collision check does real work.

### 6. nit — `rust-toolchain.toml:6-7` — the gate's "nothing beyond the pinned toolchain" promise rides on rustup's default profile
The gate needs `cargo fmt` and `cargo clippy`, which are components, not part of the
`minimal` profile. The toolchain file lists only `channel`, so the one-gate promise (Test
strategy #5) holds only where rustup installs with the default `recommended` profile; a
contributor or image that installs with `--profile minimal` gets "cargo-fmt is not
installed" on an unchanged PR.
Fix: add `components = ["rustfmt", "clippy"]` to `[toolchain]`.

## FYI (out of specialty, no severity)

`schemas/profile/v1/spec.md:22` and `schemas/reach-classes/v1/spec.md:30` say "Any other
entry is rejected", but `parse`/`read_document` ignores unknown top-level entries
(`src/document.rs:144-201`), and no `invalid` fixture covers the case — a third-party
implementer following the spec rejects documents `bindforge-core` accepts. Also
`tests/reach_tables.rs:81` (`!std::ptr::eq(left, right)`) is vacuously true for two
elements of the same `Vec`, and the `reach-classes` valid fixture is parsed but never
load-asserted, so the fixture corpus's "load" half of Test strategy #4 rests on
`left.toml`/`right.toml`.

## Verdict

approve — no blocker and no major finding. The change matches the proposal's
platform/toolchain story: one Node floor stated identically in three places, zero native or
external dependencies in the lockfile, no platform-specific code, no skipped or
platform-guarded tests, and path handling done with joins plus an explicit
case-uniqueness rule for the fixture corpus. The open findings are locale/escape gaps in
the format contract, an under-applied `--locked`, and three cheap platform hardenings.
