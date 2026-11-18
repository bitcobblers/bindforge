# Code review — correctness seat, round 1 pass c0

Design: `docs/designs/024-game-slug-identity/02-proposal.md` (ADR-024)
Scope: `git diff e786de8f6820d71dbe919300e247f72bd59ef7f3...HEAD -- ':(exclude)docs/designs'` (46 files, +2733/−287)
Inputs: the diff, 02-proposal.md, docs/adrs/024-game-slug-identity.md, the runner's gate record (green: npm check, fmt, clippy, cargo test).
Checklist: `references/correctness.md` only. Out-of-specialty observations go to the FYI line.

## Checklist walk

1. **Components and interfaces named in the proposal.** Present and wired:
   `pub const MAX_SLUG_OCTETS: usize = 128` + `pub fn is_slug_name` in
   `crates/core/src/placement.rs`, both re-exported via `lib.rs` together with
   `pub fn escape_and_truncate` (`error.rs`, renamed from `one_line`); the four
   private copies (policy.rs, manifest.rs, document.rs, action.rs — A2) deleted
   and replaced by the imported constant, with the action.rs bound refusal built
   from the constant via `format!`; `pub enum Namespace { Unknown }` with
   kebab-case serde + `Display` and `namespace` between `slug` and
   `display_name`; `pub fn derive_slug(name: &str) -> Option<String>` with the
   ordered fold → case-fold → dash-map → collapse → trim → `g-` pipeline, no
   truncation anywhere, `None` on empty; the display-name → install-dir fallback
   and construction-gate in `steam_common::discovered_game`; the per-plugin
   collision pass (byte-order distinct names, first-claim, `-2`/`-3` skipping
   claimed, out-of-bound refusal counted into `skipped`) with the
   `(slug, install_path)` sort re-applied after assignment
   (`steam_common.rs:100` pass, `:122` sort); the CLI-boundary
   `is_slug_name` + bound retain before both renderings with the exact
   `error: refused game with invalid slug: <escape_and_truncate>` line and no
   path; the five-column `COLUMNS`/`row_cells`; README bullet + "twelve"
   sentence; the four specs amended; fixtures re-pinned to exactly 129 octets
   (verified by exact-length match on all four) and `segment-past-cap.toml`
   renamed with the `action_ids.rs` roster re-pinned. **One named component is
   missing: the ledger amendment — Finding 1.**
2. **Assumption register.** Honored: A2's four sites are the ones deleted; A9's
   sentence updated to twelve; A13's single renderer kept (rename only,
   truncation/escape budget untouched); A15's boundary is implemented as the
   Options-(c) table — spot-checked the ranges (Latin-1 `À–Å→a …`, `× ÷ Æ Ø Ð
   Þ ß` excluded; Latin Ext-A strokes/ligatures `Đ Ħ ı ĸ ŉ Ŋ Œ Ŀ Ł Ŧ ſ`
   excluded, `İ→i`, `Ÿ→y` included) and `ḗ` outside the table folds to `-`;
   A12 reserved names stay reachable and are pinned as known current behaviour
   (Test strategy #11) without a platform-specific assertion; A1:
   `is_slug_name` body unchanged and length-free. No silently changed
   assumption.
3. **Deviations.** Findings 2 and 3. Whether they are declared in
   `03-build-notes.md` is unverifiable from this seat's allowed inputs — see
   Missing evidence.
4. **Scope creep.** Nothing behaviorally unasked beyond Finding 3. The
   `discovery` `serde_json` addition is `[dev-dependencies]` only, needed by the
   Test strategy #5 pin. The diff's +3 lines in `docs/adrs/024-game-slug-identity.md`
   (Rollout item 6) come from `04bc88e6 docs: give 024-R1 its ledger join handle
   in the rollout list`, a pre-proposal docs commit the proposal's Rollout 6
   already reflects — noted, not a build-stage finding (FYI line).
5. **Build-notes claims.** Not checkable — see Missing evidence.
6. **Error paths fail closed.** Verified: un-derivable → fallback → refuse
   before any `DiscoveredGame` is constructed (count-only, never echoed);
   collision candidates stop at the bound and the name is removed and counted;
   a root whose every row is refused surfaces through the existing §7
   zero-games diagnostic without echoing the row (pinned end-to-end); the CLI
   seam drops invalid rows with an escaped echo while valid rows render.
7. **Revision rounds.** First pass; no prior findings.

The derivation semantics were checked line by line (fold-before-case-fold so
accented capitals survive, `g-` prefix, collapse/trim, pure-ASCII output); the
collision pass's skip-claimed rule was checked for order cases (a name whose
base equals an already-claimed suffixed slug moves to the next free candidate,
so no two distinct names can be assigned one slug); the gate record is green
and is consistent with the suite as read.

## Findings

### 1. Blocker — the `013-L5` ledger amendment never lands

- **Location:** `docs/adrs/013-game-manifests.md:114` (file absent from the diff)
- **Problem:** The proposal's Goals include "Record the ledger amendment
  (`013-L5` gains `amended-by 024-B1`)", its "Specs, fixtures, ledger" section
  names the exact site — "the `013-L5` unit line in `docs/adrs/013-game-manifests.md`
  gains its `amended-by 024-B1` annotation per `021-L4`" — and ADR-024 Rollout 5
  carries `024-B1` for it. The diff range touches only
  `docs/adrs/024-game-slug-identity.md` inside `docs/adrs/`; at HEAD the 013-L5
  unit line carries no `amended-by` annotation anywhere (grep over `docs/adrs/`
  finds `amended-by` only in 024, 021 and the template). The gate record cannot
  catch this: no test reads the ledger. A branch that raises the shared bound
  and amends four published contracts would merge with the amended unit's ledger
  row still stating the un-annotated `013-L5`, which `021-L4` makes mandatory —
  the run's own Rollout-3/ADR-Rollout-5 completion criterion is unmet in the
  reviewed change.
- **Fix:** Land the annotation so it is inside the run's diff range: append the
  `amended-by 024-B1` note to the `013-L5` unit line in
  `docs/adrs/013-game-manifests.md`, leaving 013's prose and `013-R1`
  untouched. If `docs/adrs/` is outside the build stage's stage ownership, the
  runner (which owns these commits per the proposal's own note) must make the
  edit part of this run before completion.

### 2. Major — the four amended specs still publish the bound's literal number

- **Location:** `schemas/action-ids/v1/spec.md:54`; `schemas/policy/v1/spec.md:40`
  and the paragraph at ~:92; `schemas/manifests/v1/spec.md:54`;
  `schemas/profile/v2/spec.md:38` and `:71`
- **Problem:** The proposal (Chosen design → "Specs, fixtures, ledger") states
  the four spec sentences are amended such that "the number and the RFC 1035
  derivation become a cross-reference to the README bullet", and the bullet the
  branch itself writes into `schemas/README.md` says the four "now
  cross-reference this bullet instead of carrying a number of its own"
  (024-L3: "cross-references it instead of restating the number"). As shipped,
  all four still restate the number inline — "at most `MAX_SLUG_OCTETS` = 128
  octets". Two consequences: (a) the README bullet's and `placement.rs`'s
  shared doc sentence is false against the shipped corpus; (b) the next bound
  amendment updates the README bullet and one constant but leaves four published
  "128" literals to rot — the constant-drift failure 024-B1's one-stated-home
  rule exists to prevent.
- **Fix:** Drop the `= 128` gloss from all six sentences, e.g. "at most
  `MAX_SLUG_OCTETS` octets — the shared bound `schemas/README.md` states once
  (`024-B1`)". The `profile/v2` `derives` row already uses exactly this
  number-free form ("at most the same shared `MAX_SLUG_OCTETS` bound"), so the
  pattern is in the diff.

### 3. Minor — refusal-message restructuring beyond the proposal's mandate

- **Location:** `crates/core/src/action.rs:38-56`, `:257-277`
  (`SEGMENT_EDGE_RULE` folded into a reworded `SEGMENT_SHAPE_RULE`;
  `invalid_id`'s echo moved from leading to trailing; new
  `segment_bound_rule`/`invalid_segment` split) and the message-order swaps at
  `crates/core/src/document.rs:1259-1265` and `crates/core/src/manifest.rs:538-543`
- **Problem:** The proposal's only mandate for these sites was that refusal
  strings embedding the number "be built with `format!` from the constant".
  The implementation also reshapes the public `CoreError::InvalidValue` message
  order (rule-first vs offender-first per fault class). The accepted/rejected
  sets are unchanged, the rationale (an offender already past the bound would
  push the named number past the renderer's 80-character cut) is real, and the
  new shape is pinned (`crates/core/tests/action.rs` bound-message test,
  `slug_bound_corpus.rs`), so this is not a behavior defect — but it is a wider
  change to core's rejection-text surface than the design asked for.
- **Fix:** If `03-build-notes.md` does not already list the message-order
  restructure as a deviation under the bound amendment, record it there; no
  code change sought.

## Missing evidence (recorded explicitly)

`03-build-notes.md` is outside this pass's allowed inputs, so checklist items 3
and 5 could not be evaluated against it. Finding 2 is a concrete
proposal-vs-corpus gap and Finding 3 a concrete wider-than-mandated change;
neither is blocking on this seat for want of the declaration check — Finding 1
blocks on its own, and Finding 2 on the violated stated design text with the
drift scenario above.

## FYI

- `apps/discovery-cli/tests/cli_slug_refusal_tests.rs` keeps its own `SLUG_BOUND
  = 128` literal (no core dev-dep) by stated design, pinning the README's
  published number rather than the constant — deliberate, noted only so a later
  seat doesn't read it as drift.

## Verdict

request-changes — 1 blocker (ledger amendment absent), 1 major (specs still
restate the bound literal the design moved to one home), 1 minor.
