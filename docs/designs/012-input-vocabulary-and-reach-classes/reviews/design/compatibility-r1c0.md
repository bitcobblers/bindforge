# Compatibility review r1 (pass c0, scope: whole document)

verdict: request-changes

Inputs used: `02-proposal.md` (whole), `docs/adrs/012-input-vocabulary-and-reach-classes.md`, the
Revision log (empty — no prior dispositions, so every finding here is `new`). No previous
compatibility verdict existed.

## Findings

- [major][new] Compatibility matrix — "| Third-party reader, non-Rust | unknown | Corpora are language-neutral TOML; no external runner contract exists (Deferred) |" — the checklist forbids an `unknown` support status in the matrix: every platform row must commit to a decision, and `unknown` must be reported as a major. The row currently expresses an unmade decision rather than a justified one. Fix: change the status cell to a committed value and keep the note as the justification, e.g. `yes (data only)` — this stage ships no code, no binary, and no runner contract for any language, and the corpora are plain TOML readable by any parser — while the runner-contract gap stays where it already is, in Deferred ("An external runner contract so non-Rust readers run the corpora — no follow-up yet (same as design 011)"). That closes the matrix cell without introducing a mechanism; the deferral itself does not contradict ADR-012 (its Consequences only assert the corpus is language-neutral), so the finding is about the undecided cell, not the deferred contract.

## Checklist walk (for the record)

1. Matrix statuses: every row carries one; the single `unknown` is the major above; no empty cells.
2. OS input APIs: the design targets none and justifies the abstraction — it is vocabulary plus
   table data, the matrix states "This stage ships no binary", and A4 anchors the naming to the
   common denominator of the surveyed registries (HID, evdev, SDL, W3C `code`, kVK). Pass.
3. Version floor: the repo's floor is Rust, not Node; the matrix row names it (1.85.0, edition
   2024) and the design is std-only with `Cargo.lock` byte-unchanged, so no above-floor API is
   introduced. Pass.
4. Native modules: none; "std-only, no new dependency" plus the no-`Cargo.lock`-change rollout
   rule closes the packaging story. Pass.
5. Keyboard reality: non-QWERTY is handled by the stated ANSI-US geometry assumption, the AZERTY
   risk row, and the geometry extension point (ADR-consistent: "Tables are ANSI-US at first");
   non-ASCII binding names are structurally excluded by the closed ASCII member sets; dead keys
   and IME are moot for a physical-key vocabulary because the spec pins glyph meaning to position
   ("the US legend at that position under ANSI geometry"). Pass.
6. File paths: `load_geometry` uses `dir.join(format!("{name}.toml"))` — std `Path` joining, not
   string concatenation; the Windows case-collision story is stated in the matrix and pinned by
   Test strategy #8; per-OS config-dir conventions are untouched (no installed artifact), and the
   locating-`schemas/` deferral is consistent with design 011 and does not contradict ADR-012.
   Pass.
7. Platform tests for supported rows: Linux — gate commands in Test strategy #8; Windows — the
   once-per-contributor Windows leg in #8; toolchain floor — the three gate commands plus the
   byte-unchanged `Cargo.lock` rule; third-party v1-only reader — pinned by the `keys-entry.toml`
   fixture, the byte-unchanged `reach_tables.rs` v1 expectations, and the `supported_majors()`
   equality check (#2, #6, #7). Pass.

## FYI (no severity)

- No document (this proposal, its Deferred list, or ADR-012) assigns the OS event-code → input-id
  mapping (evdev/XKB, Windows Raw Input/virtual keys, kVK, HID usage pages) as a consumer concern;
  nothing contradicts the ADR, but whoever builds a capture layer will need an owner for it.
- The name `duplicate-input.toml` appears in both new invalid-fixture sets
  (`reach-classes/v2/fixtures/invalid/` and `keyboard-geometry/v1/fixtures/invalid/`). Test
  strategy #6 asserts the case-unique fixture-name rule "covers both new directories"; if that
  rule turns out to be global across the `schemas/*/v*/` walk rather than per-directory, one of
  the two names collides. The rule's scope is in `schemas/README.md`, which is outside this seat's
  allowed inputs — flagging for a seat that can check it.
- Item 5 note: the spec states ids name physical positions, not produced characters; one explicit
  sentence ("dead keys and IME do not affect id resolution") would make the keyboard-reality
  answer unmissable, but the requirement is already met by the glyph-meaning text.
