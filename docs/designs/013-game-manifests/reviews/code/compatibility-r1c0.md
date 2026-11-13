# Code review — compatibility — round 1, pass c0

- Design: `docs/designs/013-game-manifests/02-proposal.md` (r1), `docs/adrs/013-game-manifests.md`
- Diff under review: `git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'`
- Prior findings from this seat: none (first pass)

## Verdict: approve

No blocker or major compatibility finding. Three open items: one minor, two nits.

## Checklist walk

1. **Platform code vs. the Compatibility matrix** — pass. The diff adds no
   OS-specific code and no OS guard, which is what the matrix claims (Linux CI:
   the corpus runs under the three existing gate commands; Windows/macOS
   contributor: "nothing here adds a macOS-specific requirement"; Steam Deck:
   none). No workflow file is in the diff, matching "no CI change in this run
   (A10)". The supported legs have real code paths: `load_manifest`
   (`crates/core/src/manifest.rs:221`) is the loader the matrix names, and
   `crates/core/tests/conformance.rs:405` exercises the new corpus through the
   unchanged `schemas/*/v*/` walk. Verified on the Linux leg: `cargo fmt --all
   --check` clean, `cargo clippy --workspace --all-targets -- -D warnings`
   clean, `cargo test --workspace --locked` green (`tests/manifest.rs`: 49
   passed; `conformance.rs` 9; `grammar_closure.rs` 11; `version_gate.rs` 8).
2. **Toolchain floor** — pass. The floor is `rust-version = "1.85"`
   (`Cargo.toml:17`) and `rust-toolchain.toml` channel `1.85.0`, the matrix's
   "1.85.0, edition 2024". The new code uses nothing above it: `let-else`,
   `split_once`, `strip_prefix`, `u8::is_ascii_control`, `char::is_control`,
   `Chunks`, array `IntoIterator` — all well inside 1.85. `bindforge-core` still
   declares no dependencies (`crates/core/Cargo.toml`), and neither
   `Cargo.toml` nor `Cargo.lock` is in the diff, so "std-only, no new
   dependency, `Cargo.lock` byte-unchanged" holds.
3. **Paths** — pass with a nit. Every path in `src/` is built with `Path::join`
   (`manifest.rs:222` `game_dir.join("manifest.toml")`; test helpers
   `scratch.path().join(...)`), never string concatenation. Case-sensitivity
   assumptions are flagged where they matter: `schemas/manifests/v1/spec.md:19-22`
   and the proposal both state that byte-equality is case-unique because the id
   grammar is lowercase-only, and the grammar itself (`placement.rs::is_slug_name`)
   is ASCII-byte-based, so `id`/`codec`/preset ids cannot carry a case or a
   non-ASCII lookalike. `game_dir.file_name().and_then(|n| n.to_str())`
   (`manifest.rs:230`) fails closed on a non-UTF-8 segment rather than guessing.
   The posture matches the shipped precedents (`load_geometry`,
   `load_reach_tables`), which also compare the caller-supplied name — so the
   matrix's Windows claim ("case collisions visible") means exactly what it
   meant for the other formats, no regression.
4. **Guarded platform imports** — pass. No platform-only module is imported; the
   module graph gains `manifest` only (`lib.rs:70`), std-only.
5. **Locale/layout reality** — behavior holds, coverage does not: see finding 1.
   Non-ASCII game-native names do survive the format: the reference reader
   pushes every decoded char ≥ U+0020 raw (`toml.rs:239-242`),
   `check_native_name` (`manifest.rs:341-364`) refuses only ASCII control bytes,
   the cap counts octets (`name.len()`), and `quoted` (`document.rs:863-881`)
   writes non-control chars back raw, so a localized name round-trips
   byte-exactly. Nothing in the corpus or the suite pins that.
6. **Native dependencies** — pass, and loudly so: no dependency edge is added
   anywhere in the diff, so the packaging/CI-matrix question does not arise.
7. **Tests run where they claim** — pass. The 49 new cases in
   `crates/core/tests/manifest.rs` carry no `cfg(unix)`/`cfg(windows)` guard and
   no `#[ignore]`, so nothing is silently skipped everywhere; they are
   platform-neutral: unique scratch roots (`common/mod.rs:185-199`, pid+nonce+
   nanos, so the `Remnant-2` and `other` cases at `tests/manifest.rs:1042` never
   collide on a case-insensitive filesystem), no permission or symlink
   assumptions, no path near `MAX_PATH`, and the fixture reader tolerates a
   CRLF checkout because `toml.rs:106` splits with `str::lines()` (strips the
   trailing `\r`) even though the repo ships no `.gitattributes`. The Windows
   leg of Test strategy #8 is manual by design (Rollout 7), and nothing in this
   diff makes it more fragile than the suites that shipped before it.

## Findings

### minor — the format's only free-text column has no non-ASCII case anywhere in the corpus or the suite

- Location: `crates/core/tests/manifest.rs:428` (`accepts_a_native_name_containing_a_hash_a_quote_and_a_backslash`)
  and `schemas/manifests/v1/fixtures/valid/`
- Problem: the proposal's basis for the row encoding (A13) is *localized* game
  names — "Steam IGA 'Move'/'Camera' with localized text", Elite's multi-word
  control names — yet every native name in the new corpus and in all 49 tests is
  pure ASCII. `grep -P '[^\x00-\x7F]'` over `schemas/manifests/`,
  `src/manifest.rs` and `tests/manifest.rs` finds only prose em-dashes. So the
  two locale-relevant properties of this format are unpinned and unproven
  against a second implementation: (a) a multi-byte name parses and round-trips
  byte-exactly — `quoted` writes it raw rather than as `\uXXXX`, which is a
  third-party-visible serialization choice nothing asserts; (b) the 128-octet
  cap is an *octet* cap, so a 60-character CJK name (180 octets) refuses while a
  character-counting implementer would accept it — the spec says "octets" once
  (`spec.md:78`) and no case demonstrates it. `reach-classes/v2` set the
  precedent of pinning the non-ASCII posture in the corpus
  (`the_v2_corpus_ships_the_fixture_that_refuses_a_non_ascii_member`); the
  format whose whole point is the game's own spelling of a name ships none.
- Fix: pin both halves. Minimum: a case in `tests/manifest.rs` beside the
  punctuation test that parses a row whose name is multi-byte (say
  `aim.camera Caméra "retour"`), asserts the value byte-for-byte and
  `parse(serialize(parse(text))) == parse(text)`; plus a boundary case whose
  name is under 128 *characters* and over 128 *octets*, refused for the octet
  budget. Better: also add one `fixtures/valid/` document carrying a localized
  name, which needs one line added to the proposal's fixture list and to
  `spec.md`'s "Spaces, `#`, and every glyph…" sentence so the language-neutral
  contract says non-ASCII is data.

### nit — the location refusal interpolates a directory-name segment from the caller's filesystem, and the test only rules out the full path

- Location: `crates/core/src/manifest.rs:237-245`, asserted at
  `crates/core/tests/manifest.rs:1058-1061`
- Problem: the proposal states the mismatch is an `InvalidValue` that "never
  echoes a local path (the error contract's channel for paths is
  `source_path`)", and `error.rs` keeps that promise everywhere else by never
  rendering `CoreError::Io { path }`. This message interpolates `{segment}` — a
  component of the caller's local path — into the rendered line. It is harmless
  today (it is bounded and escaped by `one_line`, and it is the only spelling
  that makes the refusal actionable), but the test that claims the promise
  asserts only `!detail.contains(scratch.path().display())`, so a later edit
  that echoed `game_dir.display()` minus the temp root, or a parent segment,
  would keep the suite green while quietly contradicting the sentence.
- Fix: either narrow the message to name only the document's own `id` (the
  caller already knows which directory it passed), or keep the segment and
  strengthen the assertion to reject any `display()` suffix or path separator in
  the rendered message, so the contract sentence is what is pinned rather than
  one scratch root.

### nit — no non-ASCII/lookalike case for the slug grammar, though the test's own rationale names that risk

- Location: `crates/core/tests/manifest.rs:787` (`refuses_a_manifest_id_that_breaks_the_slug_grammar_naming_the_offender`)
- Problem: the doc comment two lines above says the rule exists to catch "a
  spelling that merely *looks* lowercase", but the loop only tests an uppercase
  ASCII spelling, a dot, a leading hyphen, and a 64-octet id. A non-ASCII id —
  `remnantе-2` with a Cyrillic `е`, or an accented `joueur-2`, or a U+2011
  non-breaking hyphen — is the realistic contributor mistake on a non-English
  machine, and it is also the case where a directory name and an `id` can look
  equal in a file browser while being unequal byte-for-byte, i.e. the exact
  name↔location collision the loader's rule exists to make visible.
- Fix: add one non-ASCII spelling (and one Unicode-hyphen spelling) to that
  loop's list; the byte-based `is_slug_name` refuses them, so it is a one-line
  addition that pins the claim the comment already makes.

## FYI (outside this seat)

`decode_base64` (`manifest.rs:586-627`) accepts non-canonical final-quartet bits
— for a 44-char body the last character carries two unused bits — so contrary to
its doc comment, several spellings do decode to one digest, and a canonical
third-party encoder would rewrite such a fingerprint on round-trip. Relatedly,
`Document` gains a variant with no `#[non_exhaustive]` (same posture as the 011
and 012 arms, crate is 0.1.0), and a native name may still carry Unicode
format/bidi characters (U+202E) because only ASCII control characters are refused.
