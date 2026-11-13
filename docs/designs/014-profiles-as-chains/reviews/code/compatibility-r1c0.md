# Code review — compatibility seat, round 1 pass c0

Scope: `git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'`
(57 files: `crates/core/src/{chain,chain_ops,document,error,lib}.rs`, five core test
suites, `schemas/profile/v2/` spec + 43 fixtures, `schemas/README.md`,
`data/profiles/README.md`). Judged against
`docs/designs/014-profiles-as-chains/02-proposal.md` (Compatibility matrix, Chosen
design, Test strategy #9) and `docs/adrs/014-profiles-as-chains.md`. Previous
findings from this seat: none (first pass).

## Checklist evaluation

1. **Platform code vs. the Compatibility matrix** — evaluated, one gap. No
   OS-specific code was added, and none is needed: the loader is
   filesystem-portable in its API use, no `cfg`-gated paths, no new job in
   `.github/workflows/check.yml` (matrix row "Linux CI runner … no CI change this
   run" holds — the workflow files are untouched). The three gate commands were
   run here on Linux: `cargo fmt --all -- --check` clean, `cargo clippy --workspace
   --all-targets -- -D warnings` clean, `cargo test --workspace --locked` green,
   including the 71 new `chain_ops` tests and the 9 `conformance` cases that now
   cover `profile/v2`. The Windows row's stated basis — "profile-id ↔ file-name
   equality is case-unique" — is not what the loader enforces: finding 1.
2. **Toolchain floor (1.85.0, edition 2024, std-only, no new dependency, `Cargo.lock`
   byte-unchanged)** — holds. `Cargo.toml`/`Cargo.lock` and both workflows are absent
   from the diff; `bindforge-core` still depends on nothing. Scanned every added line
   for APIs newer than the floor: the newest used are `std::iter::repeat_n` (1.82,
   `crates/core/tests/chain_ops.rs:864,876`) and `Path::try_exists` (1.54,
   `chain_ops.rs:348`); no let-chains (1.88), no `LazyLock`/`extract_if`/`is_none_or`,
   no unstable calls. Caveat: no 1.85 toolchain is installed here, so this is a static
   scan, not a floor build.
3. **Paths** — `path` joins throughout: `directory.join(format!("{ancestor}.toml"))`
   (`chain_ops.rs:131`), `head.parent()` (`:112`), tests join relative components
   (`chain_ops.rs` tests `:359,1293,1305,1394,1419`). No string-built paths: every
   `format!` produces a single file-name *component* from a slug-validated id, and the
   parse-time slug check (`document.rs:1051`) refuses `..`, `/`, and `\` before any
   path is built. Case-sensitivity assumption is the real gap: finding 1.
4. **Guarded platform imports** — nothing platform-only is imported; no change needed.
5. **Locale/layout reality** — ids stay ASCII by construction (`is_slug_name`, plus
   the closed ASCII input and action grammars), so the pair encoding
   (`pair_fields`, `document.rs`) cannot be split by a locale glyph, and non-ASCII ids
   refuse with a named `InvalidValue` (`id-uppercase.toml`,
   `derives-element-uppercase.toml`). The one-`@` pin check rejects non-ASCII digits
   (`tail != revision.to_string()`, `document.rs:1036-1041`). The provenance-label
   character rule, the one field that *does* admit non-ASCII, is under-specified in the
   third-party contract and unpinned on both sides: finding 2.
6. **Native dependencies** — none introduced; packaging story unchanged.
7. **Tests run where claimed** — holds. The new suite is an ordinary `tests/` target
   with no `#[ignore]` and no `cfg(unix)`/`cfg(windows)` gating, so the Linux CI leg
   runs it (verified by execution above). The scratch-tree tests use the pre-existing
   portable `ScratchDir` (`std::env::temp_dir()`, process-id + nonce, `Drop` cleanup)
   and only relative `/`-joined paths, which Windows accepts; nothing asserts a POSIX
   separator or a case-sensitive lookup, and the depth test's paths stay far under
   `MAX_PATH`. The Windows leg of Test strategy #9 is the human step the proposal says
   it is (Rollout 7); it is exactly the leg that finding 1 would catch.

## Findings

### 1. minor — the id↔file-name rule is enforced against the requested name, not the on-disk name, so a mis-cased link loads on Windows and refuses on Linux

**Location:** `crates/core/src/chain_ops.rs:347-375` (`open_link`: `try_exists` at
`:348`, id check at `:363-373`), reached from the constructed lookup at `:131` via
`link_stem` at `:379`.

The lookup builds `<major dir>/<slug>.toml` and then compares the parsed document's
`id` with `expected`, the stem **of the path this module constructed** — never with
the name the filesystem actually matched. On a case-insensitive filesystem (the
matrix's Windows row: "Supported? yes"), a directory holding `Game-Base.toml` whose
document carries `id = "game-base"` satisfies a `derives = ["game-base"]` lookup and
loads; the same directory refuses `ChainLinkMissing` on the Linux CI runner, and git
on Windows will happily commit the mis-cased file. The compatibility matrix rests the
Windows row on "v2 ids and pin slugs are lowercase-ASCII slugs, so profile-id ↔
file-name equality is case-unique" — the lowercase ids are what makes the *requested*
name case-unique; the check does not make the *stored* name match it. Consequence is
portability, not corruption (chain identity is contents, and the divergence is loud on
Linux): a hub bundle (ADR-018's directory shape) authored on a Windows contributor
machine can pass locally and fail everywhere else, and nothing in the suite pins
either direction — `refuses_a_link_whose_id_disagrees_with_its_file_name`
(`tests/chain_ops.rs:1354`) differs by more than case, so it passes on both platforms.

**Fix:** pin the byte-exact name before opening the link, e.g. in `open_link` require
that `fs::read_dir(directory)` contains an entry whose `file_name()` equals
`{expected}.toml` byte-for-byte (a one-directory scan per link, inside the existing
`MAX_CHAIN_DEPTH` document budget), and add the case-only mismatch to the
`load_chain` guard cases in `tests/chain_ops.rs` (Test strategy #3's "a file whose
`id` disagrees with its name refuses"). If the panel decides the guard is not worth
an extra directory read, the honest alternative is to state the caveat where the
contract lives — one sentence in `schemas/profile/v2/spec.md` §"Loading a chain" rule
3 ("the reference reader compares the declared id with the name it asked for; a
case-insensitive filesystem may answer a differently-cased file, so a chain directory
must be committed with byte-exact names") — and mirror it in the matrix's Windows
row. Either way the divergence must be written down or refused, not left platform-dependent.

### 2. minor — the `generated-by` character rule the reference reader enforces is narrower in `spec.md` than in code, and neither side of the non-ASCII boundary is pinned

**Location:** `schemas/profile/v2/spec.md:46` (entry table) and §Per-link rules; the
enforcing code is `crates/core/src/document.rs:1074` (`check_generated_by`) with
`is_label_char` at `:1101`; corpus: `schemas/profile/v2/fixtures/invalid/`.

The proposal states the rule as "1–128 octets, no ASCII control, **default-ignorable**,
or bidi-control characters" and `check_generated_by` implements exactly that
(soft hyphen, U+034F, U+061C, U+180E–180F, U+200B–200F, U+202A–202E, U+2060–206F,
U+FEFF, U+FFF9–FFFB). `spec.md`'s row drops the default-ignorable half: it says only
"no ASCII control or bidi-control characters". The corpus pins only the two halves the
spec states — `generated-by-control-char.toml` (`\u0007`) and
`generated-by-bidi-char.toml` (`U+202E`, whose *comment* claims "Default-ignorable and
joiner controls are refused the same way", which is not contract text) — and no test or
fixture covers a soft-hyphen or zero-width label. Meanwhile every valid fixture's label
is ASCII (`human`, `southpaw`, `bindforge-cli 0.1`), so the accept side of the same
boundary — a non-ASCII label like a tool name with an accent or a Cyrillic policy name
— is unpinned too. Under the matrix's "Third-party reader, non-Rust — spec.md +
fixtures are the contract", a conforming third-party reader therefore accepts documents
`bindforge-core` refuses, and no fixture can catch it.

**Fix:** restore the proposal's three-way wording in `spec.md:46` (and state the
excluded set once, as `is_label_char`'s comment already enumerates it), then pin the
boundary in `crates/core/tests/chain_ops.rs`' grammar cases — one refusal for a
`U+00AD`-bearing and one for a `U+200B`-bearing label, and one accept case for a
non-ASCII label. Keep the corpus as the design's fixture list states it; if the panel
would rather pin with a fixture (`generated-by-soft-hyphen.toml`, a valid
`generated-by-unicode-label.toml`), the design's fixture list gets the same edit in the
same PR, since that list is the contract's index.

### 3. minor — a v2 id may be a Windows reserved device name; the compatibility matrix's Windows row does not price it

**Location:** `crates/core/src/document.rs:1051` (`check_profile_slug`, the one rule a
v2 `id`, each `derives` element, the pin's manifest half, and `fork`/`diff`/`merge`'s
caller-supplied fresh id all pass through — `chain_ops.rs:581`), with the id used as a
file name at `crates/core/src/chain_ops.rs:131` and `:348`.

The slug grammar `[a-z][a-z0-9-]*` admits `con`, `aux`, `prn`, `nul`, `com1`–`com9`,
`lpt1`–`lpt9`, which on Windows are device names, not file names: the file cannot be
created, `git` refuses to check it out, and `Path::try_exists` answers through the
device rather than the directory entry. So a chain whose base is `id = "con"` is
loadable on Linux and unbuildable/unfetchable by the Windows contributor the matrix
marks "Supported? yes", and a profile bundle from the hub (ADR-018's directory) can
carry one. Nothing in the diff mentions it. Note the precedent: `placement::is_slug_name`
already turns manifest ids into directory names and geometry names into file names with
the same grammar and no reserved-name guard, so this is a widening of an existing
exposure rather than a new invention — which is why it is minor and why the fix belongs
to one shared place.

**Fix:** decide it once where the grammar is defined rather than three times: either
reject a reserved Windows basename inside `check_profile_slug` (naming the rule, as the
other refusals do) with a `chain_ops`/grammar case pinning it — and open the same
question for `placement::is_slug_name` as its own follow-up — or record it under
Deferred/out of scope with its owner, plus one sentence in `schemas/profile/v2/spec.md`
(§Per-link rules, "Id grammars") and the matrix's Windows row that the id grammar does
not exclude platform-reserved names. Silence is the only unacceptable answer, because a
third-party implementer following `spec.md` has no way to know a valid id can be an
unusable file name.

## FYI (no severity, out of specialty)

- `Profile` is a public struct with eight new public fields and no
  `#[non_exhaustive]`, so any out-of-tree consumer that builds it with a struct literal
  breaks at compile time. The proposal's matrix covers document compatibility, not the
  Rust API, and nothing outside this workspace consumes the crate yet; noting it
  because the next seat's `zz_api_probe` reading is where such a promise would live.
- The 63-octet id bound is stated in `spec.md` and enforced in code but is not one of
  the exported budget constants `schemas/README.md` says the reference implementation
  publishes (its "nine constants" count is accurate as written). Flagging only in case
  the docs seat wants it named beside the other eight.

## Verdict

approve — three minor findings, no blocker or major. The Linux matrix row was verified
by running all three gate commands; the floor, dependency, path-construction,
platform-import, native-dependency, and test-portability items all hold as stated.
