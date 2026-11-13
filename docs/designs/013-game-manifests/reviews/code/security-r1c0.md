# Code review — security — round 1, pass c0

Scope: `git diff fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'`
(34 files: `crates/core/src/{manifest,document,lib,placement}.rs`, the five
`crates/core/tests/*` suites, `schemas/manifests/v1/` spec + 22 fixtures,
`schemas/README.md`, `data/manifests/README.md`).
Judged against `docs/designs/013-game-manifests/02-proposal.md` and
`docs/adrs/013-game-manifests.md`. Previous verdict held no findings; this is an
independent first read.

## Verdict: approve

Four minor/nit findings, no blocker and no major. The trust-boundary work the
design promises is present and pinned: closed entry set, named budgets applied
before any per-row work, grammar-checked ids, path-free single-line refusals.

## Checklist walk

1. **External input validated at the boundary** — met. `manifests` joins the
   shared parse path: closed entry set (`document.rs:104-116` `MANIFEST_ENTRIES`,
   applied at `document.rs:291` before any value is read), per-format version
   gate (`document.rs:149`, `format_majors`), the 1 MiB budget enforced *before*
   the read (`toml.rs:56`) and again on text (`toml.rs:99`), and the four new
   budgets (`manifest.rs:45-57`) applied by `check_row_count` at
   `manifest.rs:284,436,463,505` — each *before* the row loop, so a hostile array
   is refused before its rows are built or registry-joined. No evaluation of
   document content anywhere in the diff. `presets`, the array the proposal's r1
   revision added to the cap, is capped in code and pinned by
   `fixtures/invalid/presets-past-cap.toml` and `tests/manifest.rs:1109`.
2. **Path handling** — met. No path is built from document data:
   `load_manifest` joins the constant `"manifest.toml"` (`manifest.rs:222`) and
   *compares* the directory segment to `id` rather than composing from it
   (`manifest.rs:230-246`), so `..`/absolute/separator-bearing ids can never
   reach the filesystem — and an `id` outside `[a-z][a-z0-9-]*` is refused at
   parse (`manifest.rs:537`), which is what keeps the comparison meaningful.
   The `is_geometry_name` → `is_slug_name` rename did not move the predicate
   that guards `load_geometry`'s join (`placement.rs:139`), and
   `tests/grammar_closure.rs` pins the refusal on both sides of the rename.
   Non-UTF-8 directory names fall to the path-free refusal at
   `manifest.rs:230-235`.
3. **Injection** — met, with finding 1. No shell, no regex; splitting is on
   literal characters (`split_once(' ')`, `split(' ')`), which cannot backtrack.
   The only interpolation into file content is `serialize`'s `quoted`
   (`document.rs:463-471`, `manifest_row` at `document.rs:515`), and the
   escaped-glyph round trip is pinned by `fixtures/valid/spaced-native-names.toml`
   and `tests/manifest.rs:1302` (a name carrying `"`, `\`, `#`). The `#` case is
   genuinely safe: `toml.rs:206-245` consumes a quoted string to its closing
   quote and `finish()` only accepts `#` *after* a complete entry
   (`toml.rs:348-355`), so a `#` in a native name cannot comment out the rest of
   the line — no parser-differential truncation of the constraint entries.
4. **Secrets and PII** — met. No credential, token, path, or personal data in
   source, fixtures, or test literals; the fixture fingerprints are fixed fake
   digests and nothing logs them.
5. **New dependencies** — met: the diff adds none (`Cargo.toml`/`Cargo.lock`
   are not in the range, consistent with the compatibility matrix's
   "std-only, no new dependency"), so the base64 decoder at `manifest.rs:586`
   is hand-rolled rather than a supply-chain addition — nothing to check against
   advisories. Hand-rolled is acceptable for grammar-only validation, which is
   what this is (see finding 2 for the one place it under-checks).
6. **Filesystem writes** — met: the diff writes nothing at runtime; `serialize`
   returns a `String`. Atomicity is therefore not a concern this run.
7. **Clean rejection on malformed/hostile input** — met, with findings 3 and 4.
   Every new refusal is `InvalidValue`/`MissingEntry`/the gate — no new
   `CoreError` variant, matching the proposal's error-shape rule. No panic path
   found: `decode_base64`'s indexing is guarded by the `% 4` check and the
   `significant` loop bound, id grammars own their parsing, and out-of-`i64`
   revisions are refused by `toml.rs:308` as `Malformed`, not overflow. Messages
   are one bounded line, escaped, and path-free — pinned for the location rule at
   `tests/manifest.rs:1057-1065`.

## Findings

- **[minor]** `crates/core/src/manifest.rs:360` (`check_native_name`, control
  check) with `crates/core/src/error.rs:155` (`one_line`) — the format's one
  free-text column can carry an unescaped line terminator into a refusal message
  and into every serialized copy of the manifest. `check_native_name` bans only
  ASCII controls (`bytes().any(|b| b.is_ascii_control())`), while
  `one_line`'s escaper is Unicode-category-based (`c.is_control()` = category
  `Cc`), so the two guards do not meet: `U+2028`/`U+2029` are neither ASCII
  control nor `Cc`, so a name like `Attack\u{2028}2026-…` parses, is stored, is
  re-serialized raw by `quoted` (`document.rs:875` has the same `Cc` blind spot),
  and is echoed raw by the escape-and-truncate path this design names as its own
  mitigation (Risks: "every echoed native name goes through the escape-and-truncate
  error path"). `error.rs:11-16` states the property being protected — "one
  hostile upload cannot forge extra log lines" — and JS/JSON-adjacent log
  pipelines, several log shippers, and some terminals do break on `U+2028`, so
  the forged line is a downstream-dependent but real hole in a *new* attack
  surface: every earlier format's echoed strings are grammar-constrained ids
  (`PROFILE_ENTRIES`, `ACTION_IDS_ENTRIES`, `GEOMETRY_ENTRIES` carry no free
  text), so this diff is the first time arbitrary Unicode reaches that path. It
  also weakens the "two documents differing only by an invisible character are
  never two facts about one label" rationale the trailing-space rule rests on
  (`U+00A0`/`U+200B` differ invisibly and pass). The design chose "no ASCII
  control character", so the code conforms; the fix belongs in the code and one
  fixture. Fix: tighten the clause to the printable set —
  `name.chars().any(|c| c != ' ' && (c.is_control() || c.is_whitespace()))` is
  refused (that covers `U+0085`, `U+2028`, `U+2029`, NBSP), plus refuse the
  `Cf` format range that reorders a reviewer's terminal
  (`U+200B..=U+200F`, `U+2060..=U+2064`, `U+2066..=U+206F`, `U+FEFF`), add
  `fixtures/invalid/name-format-char.toml`, and mirror the one clause in
  `schemas/manifests/v1/spec.md:79` so a third-party reader refuses the same
  documents.

- **[minor]** `crates/core/src/manifest.rs:586-627` (`decode_base64`, used by
  `check_fingerprint:559`) — the decoder accepts non-canonical base64,
  contradicting its own doc comment at `manifest.rs:582-585` ("A body outside
  this never decodes through a permissive fallback, which would let two spellings
  claim one digest") and the `fingerprint-not-base64.toml` fixture comment that
  says the same thing about a different case. In the final quartet the decoder
  takes `significant` from the trailing `=` and pushes only the high bits, so the
  unused low bits of the last data sextet are discarded silently: a 32-octet
  digest's 44-char body has 16 spellings that decode identically (`AA==` and
  `AB==` both yield `0x00`). RFC 4648 §3.5 requires those bits be zero, and
  `spec.md:116-117` ("standard base64 … `=` padding at the end of the final
  quartet only") does not say which side of that a reader takes — so the corpus,
  which "is the contract" here, pins neither, and a third-party reader written to
  RFC 4648 refuses documents this reader accepts. Impact is contained today
  because nothing consumes the digest in this run (proposal: "nothing consumes
  the value in this run"), but 017/018's whole use of the field is comparing it,
  and then two spellings of one digest are two byte-different fingerprints in two
  `manifest.toml` files that a string comparison calls a mismatch. Fix: in the
  final quartet, reject non-zero discarded bits (`significant == 2` →
  `sextets[1] & 0x0f == 0`; `significant == 3` → `sextets[2] & 0x03 == 0`), state
  "canonical spelling per RFC 4648 §3.5" in `spec.md`'s Fingerprint grammar, and
  pin it with `fixtures/invalid/fingerprint-non-canonical.toml` plus a case
  beside `tests/manifest.rs:894`.

- **[minor]** `schemas/manifests/v1/spec.md:129` — the shipped, language-neutral
  contract claims a guard the reference implementation does not have: "The loader
  enforces monotonicity forward". `load_manifest` (`manifest.rs:221-273`) applies
  exactly two cross-file rules — location and registry join — and has no prior
  state to compare a `revision` against; nothing else in the diff reads history,
  and the proposal itself says "core cannot pin a file's history, A10" and puts
  the bump rule in manifest-PR review plus ADR-014's deferred pin matrix. Because
  this sentence sits in the spec an out-of-repo implementer implements verbatim,
  it both invites invented history-dependent behavior and lets a consumer treat
  `revision` as rollback/tamper evidence ("a manifest re-spinned at an older
  revision under the same number") on the strength of a loader check that does
  not exist — the design's actual posture is that no such check exists until
  ADR-014's matrix lands. Fix: reword to match the design — "monotonicity is
  enforced in manifest review, and by the dependent-pin matrix once ADR-014
  ships; `load_manifest` enforces only the location rule and the registry join".

- **[nit]** `crates/core/src/manifest.rs:559-579` (`check_fingerprint`) — the one
  string in the entry table with no named size bound: a legal value is at most
  51 characters, but nothing states or checks that, and the body is fully
  allocated and decoded (`decode_base64`'s `Vec::with_capacity(quartets * 3)`,
  `manifest.rs:592`) before the 32-octet length refusal, so a 1 MiB fingerprint
  element still buys ~0.75 MB of allocation and a full scan before the refusal.
  Everything else in this format has its cap named in the budgets table; this is
  the gap the table's own rationale ("each cap sits well above the baseline it was
  surveyed for") implies. Fix: `if body.len() != 44 { return Err(fault(…)) }`
  before decoding (which also settles the canonical-shape question above), or
  name the bound in `spec.md`'s budgets table.

## FYI (no severity)

`string_array` (`document.rs:840`) materializes the whole array before
`check_row_count` runs, so a hostile array inside the 1 MiB budget transiently
allocates on the order of 10^5 `String`s before the cap fires — pre-existing
pattern across every format, bounded by the 1 MiB budget, not a new exposure; the
`presets` cardinality the design's r1 added is now capped and pinned. `Manifest`
and its row types expose public fields with no constructor validation, so a
consumer can build an in-memory `Manifest` its own `serialize` writes as a
document the grammar refuses — same shape as `Profile`/`KeyboardGeometryTable`
today, an API-design note rather than a security one.
