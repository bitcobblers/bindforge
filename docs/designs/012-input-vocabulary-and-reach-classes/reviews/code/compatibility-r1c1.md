# Code review — compatibility seat, round 1 dissent pass (c1)

Scope: `git diff 2682fb1f740770fdb588db1a3ff69d66e9d1f3d4..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md`, `docs/adrs/012-input-vocabulary-and-reach-classes.md`,
the contract tree, and `crates/core`. Inputs: that diff, the proposal, the ADR, and my
round-1 findings. Verdict basis: my five r1 findings re-tested against the fix commits
(`eba75fc`, `9c1f601`), plus anything the changed code newly makes true.

## My round-1 findings, re-checked

### 1. major — version gate format-blind — **closed**

`crates/core/src/document.rs` now carries a single per-format gate: `format_majors(format)`
(document.rs:125-131: profile/action-ids/keyboard-geometry → `["v1"]`, reach-classes →
`["v1","v2"]`, unknown → `None`) and `check_format_major` (document.rs:137-148), called in
`parse` immediately after the crate-wide gate (document.rs:238-241). A `profile` or
`action-ids` document declaring `v2` now passes the union gate and is refused by
`check_format_major` with `UnsupportedVersion` naming *that format's* majors — the
fail-open my r1 finding described (v2-declaring profile parsed with the v1 grammar via
`parse_profile`/`registry_from_entries`) no longer exists; I confirmed by reading the
call order and by the pinned cases. The geometry arm's private guard
(`parse_keyboard_geometry`'s `version != MAJOR_V1` block) is deleted and subsumed by the
per-format gate, and the `keyboard-geometry` v2 case stays pinned in the new
`version_gate.rs::rejects_a_document_declaring_a_major_that_only_another_format_ships`
(profile, action-ids, keyboard-geometry, each asserted naming `format_majors(format)`,
with a guard that the tree genuinely carries no v2 for those formats). The conformance
currency is closed too: `schemas/profile/v1/fixtures/invalid/major-not-shipped.toml`
(v2-declaring profile) is in the invalid corpus, is refused by the walk, and
`the_invalid_corpus_ships_a_fixture_for_a_major_the_format_does_not_ship` pins it by name
naming profile's own majors. The contract states the rule in all three places a
third-party reader reads it: `schemas/README.md:27-34`, `schemas/profile/v1/spec.md:37-44`,
`schemas/reach-classes/v2/spec.md:147-151`. The majors-equality case keeps the union
against the tree (A2 unchanged). The `action.rs` loader-side directory/version equality
is untouched and still fires through the shared helper. Exactly the fix my finding asked
for, in one place, pinned at test and corpus level.

### 2. minor — ASCII half of the id grammar only in Rust — **closed**

`schemas/reach-classes/v2/spec.md:24-43` now says "lowercase ASCII", states "The set is
ASCII, not Unicode-lowercase", names `keyboard:é` as refused-naming-the-member and never
folded onto `keyboard:e`, and points at the corpus case. `schemas/reach-classes/v2/
fixtures/invalid/non-ascii-member.toml` ships it (one fault: `keys = ["keyboard:é home"]`;
the documented string grammar admits non-ASCII characters in basic strings — only control
characters are refused, `toml.rs:240` — so the fixture is refused for the member, not for
the string). `crates/core/tests/input.rs:124-127` adds the `keyboard:é` unit case;
`crates/core/tests/reach_v2.rs::the_v2_corpus_shipps...` (name as shipped) pins the fixture
by path and asserts the refusal names the member — I confirmed `one_line` (error.rs:155-169)
keeps non-control printable characters verbatim, so the message contains `é`. The Rust
rule text (`KEYBOARD_MEMBER_RULE`, `is_keyboard_member`, module docs) matches. All three
arms of my fix are in place.

### 3. minor — stand-in-directory parenthetical contradicted the harness — **closed**

`schemas/reach-classes/v2/spec.md:159-166` now describes the shipped layout: stand-in
directory named for the fixture's own major, completing table from the same major
directory, "a fixture never answers beside another major's table". This matches
`crates/core/tests/conformance.rs:121-128` (`{major}/{other}.toml`, load from
`scratch.path().join(major)`) and the doc-comment at conformance.rs:93-108. The contract
text and the reference harness now say the same thing.

### 4. nit — "directory is named for a major" defined twice — **closed**

`major_dir_segment` (document.rs:692-700, `pub(crate)`) is the single definition; both
private `is_major_dir` copies in `document.rs` and `action.rs` are deleted,
`load_action_registry` (action.rs:229) and `check_dir_version` (document.rs:675) both
consume it, and each loader still decides what `None` means (registry: refusal; reach
tables: pass-through for harness stand-ins). The remaining `is_major_dir` at
`tests/common/mod.rs:412` is in the external test crate and cannot reach a `pub(crate)`
src helper — unavoidable across the crate boundary, outside what my fix asked for.

### 5. nit — `zz_api_probe.rs` outside the delta inventory — **closed via the doc-sync leg**

The target itself stays (my fix offered recording as the alternative to folding), and it
is now declared: `03-build-notes.md` deviation 1 names `crates/core/tests/zz_api_probe.rs`
and `common::quoted_array` as beyond the proposal's three-test-target inventory, deviation
2 corrects the stale stand-in justification, and the fix commit's message records the
nit as notes-leg bookkeeping. The proposal's delta inventory still names three targets —
the proposal is the approved design record and the build-notes deviation register is the
place deviations are declared, which is what the doc-sync leg did. I consider the
accountability closed; the divergence is now visible in the run record rather than hidden.

## New finding from the changed code

1. **nit — the per-format contract sentences and the union gate's message for
   majors outside every format.** `schemas/README.md:27-34` states flatly that "a document
   declaring a major its format does not ship is refused naming that format's majors", and
   `crates/core/tests/action.rs:16-22` says action-ids refusals name "the majors the
   `action-ids` corpus carries rather than `bindforge-core`'s crate-wide union". But the
   union gate still binds first (document.rs:232-237): a profile or action-ids document
   declaring `v99` is refused naming `["v1", "v2"]` — including the `v2` that format
   never ships — while the same class of document declaring `v2` is refused naming
   `["v1"]`. The module doc states this layering honestly (document.rs:11-19: "a major no
   format ships is refused naming every major the crate declares"), the README's first
   bullet covers it, and the direction is safe (naming a superset is not fail-open; the
   suite's own assertions are containment, not set equality, and no fixture pins exact
   message text), so conformance is not at risk — but the two narrowed sentences the diff
   adds read as "only the format's majors", which the reference implementation does not
   honor for majors no format ships. Fix: add the module doc's half to `schemas/README.md`
   ("a major no format ships at all is refused naming the majors the crate declares"), and
   soften the `action.rs` header the same way — or run `check_format_major` before the
   union gate so every version refusal names exactly the format's majors.

## Compatibility checklist

1. **Platform matrix:** no OS-specific code anywhere in the diff; matrix rows hold —
   Linux CI (all corpora run under the three gate commands: I ran
   `cargo test --workspace --locked` independently — 14 targets, 122 passed, 0 failed),
   Windows (new fixture names `major-not-shipped`, `non-ascii-member` are lowercase,
   hyphen-joined, case-unique per README:39-40).
2. **Toolchain floor:** std-only; `Path::file_name`, `strip_prefix`, byte checks — all
   well inside 1.85/edition-2024. No `Cargo.toml`/lock change in the diff.
3. **Paths:** `major_dir_segment` uses `Path::file_name`; no new string-concatenated
   filesystem joins in src. Case sensitivity: unchanged handling, new names case-unique.
4. **Guarded imports:** none needed; no platform-only modules.
5. **Locale/layout reality:** the seat's core item is now pinned three ways — the spec
   says ASCII, the corpus fixture refuses `keyboard:é` with the member echoed un-folded,
   and the loader compares byte-exact. The UTF-8 fixture content itself survives the
   documented string grammar on every platform; é is not a control character, so the
   single-line echo keeps it (error.rs:155-169).
6. **Native dependencies:** none introduced.
7. **Tests run where claimed:** the new cases are content-based (fixture text, in-test
   document strings) — no cwd or platform assumptions; they ran green in the suite walk
   on this Linux runner; nothing platform-skipped.

## Verdict

All five of my round-1 findings are closed by the fix: the per-format gate landed exactly
as prescribed and is pinned at unit, version-gate, and corpus level; the ASCII grammar is
now contract text, fixture, and unit case; the stand-in parenthetical matches the harness;
the major-dir predicate is single-sourced; the probe's inventory divergence is declared.
The one new item is a nit about wording alignment between the README's per-format sentence
and the union gate's message for majors no format ships — safe direction, no conformance
currency at risk. No open blocker or major.

**approve**
