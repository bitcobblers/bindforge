# Code review — security, round 1 pass c0

Scope: `git diff 4aeb57c2188fbdfc1ca68d99080e4d2f3950d945...HEAD -- ':(exclude)docs/designs'`
(30 files: `crates/core/src/{action,chain,document,lib}.rs`,
`crates/core/tests/{action,action_ids,grammar_closure}.rs`, `data/{manifests,profiles}/README.md`,
`schemas/README.md`, `schemas/action-ids/v1/**`, `schemas/profile/v1/spec.md`).
Inputs: that diff, `02-proposal.md`, `docs/adrs/011-action-taxonomy-and-core-skeleton.md`.
Specialty: security checklist only (`references/security.md`). No previous findings to re-check — first pass.

## Verdict

approve — no blocker or major open. Three findings, all minor/nit: two gaps in how a
stated trust rule and a stated message rule are pinned by the *contract* rather than
only by this crate's Rust tests, one doc-enumeration gap.

## Checklist walk

1. **External input validated at the boundary** — pass. The new format goes through the
   same untrusted path as every contract document: `load_action_registry` →
   `document::parse_file` → `toml::read_file`, which checks the file length against
   `MAX_DOCUMENT_BYTES` *before* reading (`toml.rs:54-62`), then `read_document`'s
   second length check (`toml.rs:98-103`) and the nesting cap. The entry set is closed
   (`ACTION_IDS_ENTRIES`, `document.rs:52-58`, applied at `document.rs:142` *before* any
   id is touched), each id is grammar-checked (`action.rs:113-131`), and the axis-kind
   spelling set is closed (`AxisKind::from_name`, `action.rs:82-89` — it is not used to
   widen the entry set, which is the right shape). No evaluation of any kind; no
   `serde`/dynamic dispatch on untrusted type tags.
2. **Path handling** — pass. `load_action_registry` joins a constant file name onto a
   caller-owned directory (`action.rs:221`); no path component is taken from document
   content anywhere in the diff. The only path-derived string the loader reads back is
   `dir.file_name()`, and it is validated to `v`+ASCII digits before use
   (`action.rs:229-238`, `is_major_dir` at `action.rs:289-294`). Non-UTF-8 names fall to
   `unwrap_or_default()` → refusal, not a panic. No shell anywhere.
3. **Injection** — pass on the code, see finding 1 on the contract. No regex, so no
   catastrophic backtracking; the grammar check is a single `O(len)` byte sweep
   (`action.rs:275-286`). Untrusted values reach only log-message fields, and `one_line`
   (`error.rs:155-169`) escapes `\\` and every control character and cuts at 80 chars —
   that is what makes the four echo sites in the new code safe (id in
   `action.rs:249-257`, id in the duplicate message at `action.rs:201-205`, the declared
   and directory version segments at `action.rs:239-245`). Note the case-sensitivity
   bonus: TOML keys are byte-exact, so `None = [...]` is refused as an unknown entry
   rather than accepted as `none`.
4. **Secrets and PII** — pass. Nothing in the new source, fixtures, or READMEs is a
   secret or personal data; the fixture corpora hold only ARPG action names. No logging
   calls were added, and the new echo sites are bounded as above.
5. **New dependencies** — pass, nothing to evaluate: no `Cargo.toml` or `Cargo.lock`
   hunk in this diff, the new module is std-only (`action.rs:28-33`), and the
   compatibility matrix's "no new dependency, `Cargo.lock` byte-unchanged" row holds.
6. **Filesystem writes** — pass. The diff writes nothing from library code; the only
   writes are the pre-existing `common::ScratchDir` temp-directory helpers, used here
   with fixed literal names, and they are removed on drop.
7. **Clean rejection of malformed/hostile input** — pass. Every new refusal reuses an
   existing `CoreError` variant (no new variant entered the error contract, as the
   proposal required), nothing is partially applied — `registry_from_entries` discards
   the half-built map on every fault (`action.rs:179-210`) — and the loader's `Io`
   refusal keeps the local path off the message (`toml.rs:64-71`, `error.rs:142-144`).
   The suite pins the hostile-input legs explicitly: truncation of an echoed 4 KiB id
   (`tests/action.rs:453-471`), escaped control character in an id
   (`tests/action.rs:476-491`), no path disclosure in the three loader refusals
   (`tests/action_ids.rs:540-585`, `tests/action.rs:549-553`), and the over-budget
   registry refused as `DocumentTooLarge` before parsing (`tests/action_ids.rs:591-606`).

## Findings

### 1. minor — the corpus never pins the escape/truncate rule for the new id echo site
- **Location:** `schemas/action-ids/v1/fixtures/invalid/` (missing case); rule stated at
  `schemas/action-ids/v1/spec.md:116-117` and `crates/core/src/action.rs:109-131`,
  `action.rs:194-206`.
- **Problem:** this diff adds the contract's largest new echo site of untrusted text —
  the offending id, which an attacker controls up to the whole parse budget — and the
  language-neutral corpus does not exercise it. All 14 invalid fixtures carry ids of at
  most ~69 characters with no control character, so the harness legs that every
  implementer runs (`conformance.rs:223-245`: one line, no control character) pass for a
  reader that echoes a 4 KiB id verbatim and never truncates. `profile/v1` already sets
  the precedent for pinning this (`fixtures/invalid/forged-log-line.toml`); `action-ids`
  has no counterpart, so the rule lives only in this format's `spec.md` prose and in
  Rust tests a third-party reader does not run (`tests/action.rs:453-491`).
- **Fix:** add one invalid fixture for `action-ids/v1` named after its fault (the
  `unknown-entry.toml`/`forged-log-line.toml` model), e.g.
  `fixtures/invalid/forged-log-line.toml`: a `none` entry holding one id of several
  hundred characters that contains `\u0007` and a `\n` escape, so the harness's
  one-line/no-control-character legs bind the escaping for ids, and extend
  `conformance.rs`'s bound leg (or `a_rejection_truncates_an_untrusted_value…`) with a
  fixture-driven length assertion so a reader that echoes the whole id fails. Register
  the new file in `INVALID_FIXTURES` (`tests/action_ids.rs:113-128`) and in the corpus
  list in `02-proposal.md`.

### 2. minor — the "only the shipped copy is the registry" rule is prose-only, and the
load path the tests use is the same path an untrusted tree would take
- **Location:** `crates/core/src/action.rs:212-247` (`load_action_registry`),
  `crates/core/src/lib.rs:37` (re-export), `schemas/action-ids/v1/spec.md:14-19`,
  `02-proposal.md:192-196` and its r1 revision log line for spec.md ×
  `load_action_registry` × Placeholders.
- **Problem:** the contract says a `registry.toml` from anywhere else "is data: only the
  copy shipped in this versioned directory is ever consulted for `axis_kind`", and the
  module doc repeats it (`action.rs:9-12`, `action.rs:214-219`). What the code enforces
  is internal consistency only: the file must be an `action-ids` document, its directory
  must be named `v<digits>`, and the declared version must equal that name. Nothing
  distinguishes the shipped directory from a directory the diff's own READMEs declare
  untrusted — `data/manifests/README.md:9-13` and `data/profiles/README.md:9-13` say
  whatever lands there arrived from elsewhere. So the first consumer that resolves a
  path from a synced or uploaded tree gets an attacker-chosen taxonomy, including the
  `discrete-lateral` set ADR-015 will act on, and no test in the diff would fail:
  `tests/action.rs:138-145` and `tests/action_ids.rs:541-606` deliberately load from
  scratch directories, i.e. from "elsewhere". No consumer exists yet, hence minor, but
  the rule is exactly the kind that silently rots between 011 and 013/014.
- **Fix:** make the trust classification mechanical rather than documented. Either give
  consumers a path-free entry point that resolves the shipped tree and keep the
  path-taking loader off the public surface (`lib.rs:37` exports it today) — e.g.
  `load_shipped_action_registry()` plus a `pub(crate)` path loader that the suites use —
  or, if the Deferred "how an installed artifact locates `schemas/` data" question must
  leave the path-taking loader public, say so where a consumer reads it: name
  `load_action_registry`'s caller obligation in `spec.md` ("a caller may pass only the
  directory the artifact ships with it"), and add a `03-build-notes.md` open item binding
  ADR-013/014 to prove the registry's source is the installed copy, not a synced tree.

### 3. nit — the canonical rejection-message contract does not list the two new echo sites
- **Location:** `schemas/README.md:68-73`; new sites at `crates/core/src/action.rs:249-257`
  (offending id), `action.rs:201-205` (duplicated id), `action.rs:239-245` (declared and
  directory version segments).
- **Problem:** the README is where a third-party reader reads the guard ("a message
  quoting part of the document — a declared `format`, a declared `version`, an entry
  name — escapes control characters and truncates"). An id value is now a fourth quoted
  part, and it is the only one whose length the writer controls up to 1 MiB; a reader
  implementing the README's list literally would treat it as outside the rule. The
  obligation exists, but only in the format-local spec (`action-ids/v1/spec.md:116-117`),
  which is the arrangement this contract tree otherwise avoids ("a narrowing that only
  lives in one implementation is not a contract", `schemas/README.md:39-40`).
- **Fix:** one clause in `schemas/README.md:68-71` — "…a declared `format`, a declared
  `version`, an entry name, or an id value…" — so the escape-and-truncate duty is
  stated once, canonically, for every part of a document a rejection may quote.

## FYI (out of specialty, no severity)

- `tests/action_ids.rs:592-598`: `budget + 1 - oversized.len()` underflows (panics the
  test) if `registry.toml` ever grows past ~1 MiB; deriving the padding length with
  `saturating_sub` or a fixed filler keeps the over-budget leg independent of the id set.
- `schemas/profile/v1/spec.md:6` still assigns "the taxonomy, reach tables … and chain
  semantics other than the depth bound" to ADR-009, which ADR-011 supersedes; Rollout 4
  retargets the three module docs and `schemas/README.md` but not this line, and
  `schemas/README.md:32` keeps an "(ADR-009)" citation. Docs-only, but it is the stale
  lineage this stage set out to fix.
