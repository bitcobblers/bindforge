# Code review — security — round 1, pass c0

Range: `git diff 4cfcd8db4477247e8e6ecb26ecade40451e0ad91...HEAD -- ':(exclude)docs/designs'`
Inputs: that diff, `02-proposal.md`, `docs/adrs/014-profiles-as-chains.md`. No other
review seat's material was read.
Verdict: **approve** (0 blocker, 0 major, 3 minor).

## Checklist walk

1. **External input validated at the boundary** — pass. `format_majors("profile")`
   widened to `[v1, v2]` only (`document.rs:194-199`), the entry set is selected from
   the declared version (`profile_entries`, `document.rs:423-433`), and the v2 set is
   closed — `derives_` aliasing is refused naming the entry and `v2`
   (`tests/grammar_closure.rs:412-450`). Every chain file is opened through
   `parse_file` → `toml::read_file`, so `MAX_DOCUMENT_BYTES` is checked before the
   read (A6). No code evaluation anywhere in the diff.
2. **Path handling** — pass on the traversal half. Every `derives` element is
   slug-grammar checked at parse (`parse_profile_link`, `document.rs:801-803`) *before*
   any path is built, and `is_slug_name` (`placement.rs:74-84`) is strict ASCII
   `[a-z][a-z0-9-]*`, so `..`, `/`, an absolute spelling and an empty element cannot
   reach `directory.join(format!("{ancestor}.toml"))` (`chain_ops.rs:131`). Pinned by
   `derives-element-uppercase.toml` and by
   `refuses_a_derives_element_that_breaks_the_slug_grammar_naming_the_element`
   (`tests/chain_ops.rs:717-723`), and the single-major-directory rule is pinned by
   `resolves_no_link_outside_the_head_major_directory` (`tests/chain_ops.rs:1384`).
   No shell command is built anywhere. The id↔file-name rule has a hole on
   case-insensitive filesystems — FYI below, no severity.
3. **Injection** — mostly pass, one gap (finding 1). No regex in the diff, so no
   backtracking surface; user-controlled values echoed into refusals go through
   `delta_fault`/`name_the_link`/`name_the_element` and out `one_line`, which escapes
   and truncates (`error.rs:164-178`); the pin, id, delta element and label refusals
   all name the offender within that path. `check_generated_by` deliberately does *not*
   echo the label (`document.rs:1074-1078`).
4. **Secrets and PII** — pass. Nothing in source, fixtures, READMEs or test fixtures
   carries a credential, path or personal identifier; `forged-log-line.toml` is the
   adversarial fixture, not data. Provenance labels are capped at 128 octets and are
   never consumed as trust (`resolve` joins `seeded-from` referentially and stops —
   `chain_ops.rs:508-522`).
5. **New dependencies** — none. `Cargo.toml`, `Cargo.lock` and both workflows are
   byte-unchanged in the range; the module is std-only. Nothing to check against
   advisories.
6. **Filesystem writes** — none added. `chain_ops` opens files (`try_exists` +
   `parse_file`) and builds `Profile` values; `fork`/`diff`/`merge` never touch the
   filesystem, matching the design's "core builds documents, not filesystems".
7. **Clean rejection** — pass on the hostile-walk side: the loop opens at most
   `MAX_CHAIN_DEPTH` documents (`chain_ops.rs:128-130`), refuses `Cycle` before
   descending (`:120-122`), `ChainLinkMissing` before reading anything else
   (`:348-352`), and a v1-declaring link is refused by `check_link_version` before
   `link_below` could read its non-slug-checked `derives` (`:362`). `resolve` runs all
   three joins before any layout is computed — no partial application
   (`:151`). `merge` refuses the whole composition and re-checks its own emission
   (`:746-789`); a non-reciprocal 3-cycle falls into the refusal rather than emitting a
   wrong patch. No `unwrap`/indexing on untrusted data in the new code (the four
   `unwrap_or*` sites are defaults). `cargo test -p bindforge-core` is green, including
   the single-line-message and truncation assertions over the new corpus.

## Findings

### minor — Unicode line separators survive the label policy and `one_line`
- location: `crates/core/src/document.rs:1101-1116` (`is_label_char`); reached from
  `check_generated_by` (`document.rs:1088`) and from any refusal that echoes a decoded
  value; escape side at `crates/core/src/error.rs:173`
- problem: `is_label_char` excludes `char::is_control()` (C0+C1 only) plus an explicit
  default-ignorable/bidi list, but not `U+2028` LINE SEPARATOR or `U+2029` PARAGRAPH
  SEPARATOR. Those are line terminators for JSON/JS-flavoured log consumers and for
  several terminal emulators, and the grammar can decode them — `toml.rs:230`
  accepts `\uXXXX`, and only a *raw* sub-0x20 byte is refused inline (`toml.rs:239`).
  `one_line` does not escape them either (it tests `c.is_control()`), and the corpus
  assertion only checks `!contains('\n') && !contains('\r')`
  (`tests/conformance.rs:295`). So a `generated-by` label carrying `U+2028` parses and
  ships, and a refusal for `kind`/`format`/a delta element can carry one into the
  "single bounded line" the contract fixes. The helper's own comment claims the
  excluded set is "the one that can steer what a reader *sees*", which these are.
- fix: add `'\u{2028}' | '\u{2029}'` to the `matches!` list in `is_label_char` (spec
  sentence in `schemas/profile/v2/spec.md` line 46 widened to say so), and pin it with
  an invalid fixture (`generated-by-line-separator.toml`) beside the bidi one. If the
  log-line invariant is meant to hold for echoed values generally, also escape these
  two code points in `one_line`.

### minor — `merge` stamps a provenance label one operand never asserted
- location: `crates/core/src/chain_ops.rs:650-656` (`inherit_provenance`), applied at
  `:334-335`
- problem: the function's doc says a label is inherited "only when the two operands
  agree on it, and a disagreement is dropped rather than guessed at", but
  `(Some(one), None) => Some(one)` copies a one-sided label onto the merged document.
  Merging a patch that says `generated-by = "human"` with a silent one produces a
  document — authored by `merge` — that asserts a producer claim neither operand's
  author made for it, which is the provenance-drift class the contract refuses
  elsewhere. Impact is bounded by ADR Decision Four (nothing consumes provenance as
  trust), which is why this is minor, not major.
- fix: inherit only when both operands carry equal labels (`(Some(a), Some(b)) if a == b`)
  and drop otherwise, or state the one-sided rule in `spec.md`'s provenance paragraph
  and correct the comment; pin the chosen rule with a `merge` test.

### minor — `merge` can emit lists past `MAX_PATCH_DELTA_ELEMENTS`
- location: `crates/core/src/chain_ops.rs:315-333` (`merge`'s emit path), with
  `merge_touch_order` (`:679-690`) and `emit_composition` (`:746-789`)
- problem: the delta cap is a parse rule only (`delta_elements`,
  `document.rs:914-928`), and nothing bounds what `merge` writes out. Touched actions
  per operand reach 256 `set` + 256 `clear` + 2×256 `swap` halves = 1024, and the union
  of two operands reaches 2048; `emit_composition` writes one element per touched
  action, so the merged patch's `set` (or `clear`) can hold far more than
  `MAX_PATCH_DELTA_ELEMENTS` (256). The result is a `Profile` value that no writer's
  own reader accepts — a grammar-valid pair of operands yields an unloadable artifact,
  and the failure surfaces at the next `load_chain` instead of at the operation.
- fix: after `emit_composition`, refuse with the same `InvalidValue` shape
  `delta_elements` uses when any emitted list exceeds `MAX_PATCH_DELTA_ELEMENTS`
  ("`merge`'s composition is past the delta budget"), and pin it with a test; this
  also keeps the composite-cost statement in `spec.md` honest for merge output.

## FYI (out of specialty, no severity)

- `open_link` (`chain_ops.rs:363`) compares `link.id` against the **requested** stem
  (`link_stem(&path)`, `:379-386`), never against the name on disk. On a
  case-insensitive filesystem, a bundle containing `Game-Base`-shaped `A.toml` with
  `id = "a"` satisfies the id↔name rule when the walk asks for `a.toml`, so the
  Compatibility matrix's "profile-id↔file-name equality is case-unique" claim is not
  what the code enforces. No ambiguity follows for chain lookup on such a filesystem;
  flagging for the compatibility seat.
- The diff changes the shipped `CoreError::Cycle` `Display` text
  (`error.rs:141`: `chain derives from itself` → `chain derives from itself (`Cycle`)`)
  while the proposal states "No `CoreError` variant is removed or changed".
  Compatibility ground, not security.
- Chain loading follows symlinks (`try_exists` + `parse_file` both follow). Core never
  creates the directory, so this belongs to whoever unpacks a hub bundle (ADR-018);
  noted so it is not mistaken for a core-side guarantee.
