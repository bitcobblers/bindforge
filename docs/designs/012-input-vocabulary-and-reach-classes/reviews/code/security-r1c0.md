# Security review — ADR-012 implementation (round 1, pass c0)

Seat: security. Diff: `c26bf7f0...HEAD -- ':(exclude)docs/designs'` (source +
tests + contract corpora). Judged against `02-proposal.md` and
`docs/adrs/012-input-vocabulary-and-reach-classes.md`. This seat's first pass.

## Verdict: approve

Open blockers: 0. Open majors: 0. Open minors: 0. Nits: 0.

## Checklist walk

1. **External input validated at the boundary.** Every new untrusted surface is
   schema-gated before use: the `reach-classes` arm picks its closed entry set
   from the *declared* version (`document.rs` `reach_classes_entries`), so
   `keys` in a v1 doc is a refusal naming the entry — pinned by
   `schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml`. `keys` pairs and
   geometry triples are validated element-by-element (grammar-valid
   `keyboard:*` id, class listed in the same document, id once, coordinates
   `0..=999` through checked i64→u32 conversion with an explicit
   `MAX_GEOMETRY_COORDINATE` — no wrapping). The `classes` array is pinned to
   the exact five-id taxonomy in order (`check_taxonomy`), so no document can
   smuggle a private severity scale. All loads go through
   `toml::read_file`, which enforces `MAX_DOCUMENT_BYTES` against file metadata
   *before* allocating, and the nesting budget is unchanged. No code
   evaluation anywhere in the diff.

2. **Path handling.** `load_geometry(dir, name)` validates `name` against
   `[a-z][a-z0-9-]*` (`placement.rs` `is_geometry_name`) *before* the
   `dir.join` — no `..`, no separator, and an absolute argument is refused
   (its first byte fails the grammar, which also blocks `Path::join`'s
   absolute-replaces-base hazard). `tests/placement.rs
   a_geometry_name_outside_the_grammar_is_refused_before_any_file_is_read`
   pins traversal, separator, `.`/`..`, empty, case, space, and absolute names
   against a scratch tree where the traversed file is a *valid* document, and
   asserts the refusal names no resolved path and no opened file. The
   document-side `geometry` entry takes the same grammar at parse
   (`parse_keyboard_geometry`), and nothing builds a path from document data.
   `load_reach_tables` builds file names only from the `Hand` enum. No shell
   anywhere in the diff.

3. **Injection.** No regex (no catastrophic-backtracking surface at all); the
   pair/triple encodings use single-`split_once`/bounded `split(' ')` on ids
   whose closed member sets structurally exclude spaces (verified:
   `is_keyboard_member` admits only ASCII letters/digits/11 glyphs/`space`).
   Echoed hostile strings (`invalid_id`, `pair_fault`, `coordinate_fault`,
   `check_dir_version`, `reject_unknown_entry`) all flow into `CoreError`
   variants whose `Display` runs every carried string through `one_line`
   (`error.rs`: control chars escaped, 80-char truncation), so no log-line
   forging or log inflation; `tests/input.rs
   a_refusal_echoes_the_id_escaped_to_printable_characters` pins it. Non-ASCII
   glyph homoglyphs (`keyboard:é`) are refused — the member test is
   ASCII-only.

4. **Secrets and PII.** None in new source, fixtures, or messages; rejection
   messages keep local paths in `CoreError::Io.source_path` only, never in
   `Display` — and the traversal tests assert that.

5. **New dependencies.** None: `Cargo.toml`/`Cargo.lock` are untouched (not in
   the diff), the diff is std-only, and `git diff --name-only` over the range
   shows no dependency-manifest change. Nothing to check against advisories.

6. **Filesystem writes.** No write path added in `src` at all —
   `load_geometry`/`load_reach_tables` are read-only; the only `fs::write`
   hits are the pre-existing `ScratchDir` helpers in `tests/common`, outside
   the diff's additions.

7. **Clean rejection.** No `unwrap`/`expect`/`panic!`/`unreachable!` anywhere
   in the non-test part of the diff (grep over the diff returns none); every
   hostile-input path returns a typed `CoreError` (`InvalidValue`,
   `MissingEntry`, `UnsupportedVersion` — no new variant, honoring A11). The
   loaders are all-or-nothing: `load_reach_tables` returns `Err` before
   assembling any `ReachTables`, `load_geometry` before returning any model,
   so a half-validated directory never reaches a caller. Version/directory
   mismatch (v2-in-v1 *and* v1-in-v2) is refused at load for every reach
   table, pinned by `tests/reach_v2.rs
   a_reach_table_declaring_a_version_that_disagrees_with_its_directory_is_refused`.
   The definedness rules in `placement_is_legal` answer a structured refusal,
   not a bool verdict, so an ungraded/uncoordinated id can never be silently
   admitted into a "legal" answer.

## Notes on deliberate design choices reviewed and accepted

- `check_dir_version` no-ops when the loading directory's last segment is not
  `v<digits>`. This is the spec'd stand-in carve-out
  (`reach-classes/v2/spec.md` "Fixtures"; the harness scratch directories now
  carry the `v<major>/` segment so shipped-corpus loads still take the check).
  A caller that points a loader outside the `v<N>` tree opts out of the
  directory binding, but the version gate and per-document grammar still
  apply, and a v1 half can never produce grades to poison a placement — it
  refuses as `ungraded`. Acceptable as documented.
- The closed member grammar caps valid `keys` elements at 48 per document
  (49th is a grammar or duplicate refusal), so the linear duplicate scans are
  not a CPU-amplification vector within the 1 MiB budget.
- The placement-test suite explicitly pins the absolute-name hazard the
  proposal's r2 revision recorded ("the hazard the accepted finding names"),
  and the implementation's grammar-check-before-join satisfies it.

## FYI (not findings, no severity)

- The diff adds a fourth test target, `crates/core/tests/zz_api_probe.rs`,
  which the proposal's delta inventory does not list (it names three).
  Behaviorally it is a public-surface compile probe with no new rules; flag
  for the scope/correctness seats to reconcile with the inventory.
- `serialize`'s reach arm keys-writes when `version == v1 && !keys.is_empty()`
  is impossible from parsed input (v1 parses with empty `keys`), but a
  hand-constructed `ReachClassTable` with pub fields could serialize a v1
  document carrying `keys`, which would not re-parse — model-API honesty, not
  an untrusted-input path.
