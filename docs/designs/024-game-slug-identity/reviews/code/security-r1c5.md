# Code review — security seat, round r1c5 (ratification)

- **Seat:** security (code review)
- **Pass:** ratification — checks only that the revisions landed after the
  earlier approval did not break what this seat approved.
- **Scope:** `git diff 0ee2eeeb7992f470561763e45e56261f503e64ae..HEAD --
  ':(exclude)docs/designs'` — 17 files, +713/−69.
- **Inputs used:** that diff, `docs/designs/024-game-slug-identity/02-proposal.md`
  (full current document), `docs/adrs/024-game-slug-identity.md`, the runner's
  gate record, and my own previous findings (none).
- **Verdict:** **approve** — zero open blocker and major findings; no new
  findings at any severity.

## What the range changes (production code)

The run record's "Escalation repair" revision-log entry is the shape of this
range. Behavior-bearing edits are four:

1. `apps/discovery-cli/src/main.rs:81-129` — the inline emit-boundary `retain`
   closure is hoisted into `fn refuse_invalid_slugs(&mut Vec<DiscoveredGame>)
   -> Vec<String>`; `main` prints the returned lines. Predicate, ordering and
   echo renderer are byte-identical to what r1 approved.
2. `crates/core/src/placement.rs:161-181` — `load_geometry` gains a
   `MAX_SLUG_OCTETS` arm *beside* the grammar check, before `dir.join`; doc
   comments restate the `024-B1` rider set.
3. `crates/core/src/policy.rs:49-66` — `load_policy`'s single combined slug
   refusal splits into a grammar arm and a bound arm.
4. `crates/core/src/document.rs:1232-1243` — `check_pin`'s combined arm splits
   the same way for the `for` pin head.

Everything else in the range is tests (`crates/core/tests/{placement,policy,
chain_ops,ledger_annotations}.rs`, `crates/discovery/src/steam_common.rs`
in-source module, `crates/discovery/tests/collision_tests.rs` header,
`apps/discovery-cli/tests/cli_slug_refusal_tests.rs` and its in-source module),
plus the `013-L5` ledger annotation and the spec-literal removals in
`schemas/`.

## Security checklist, item by item

**1. External input validated at the boundary — holds, strengthened.**
The CLI seam's condition is unchanged (`is_slug_name(&game.slug) && game.slug
.len() <= bindforge_core::MAX_SLUG_OCTETS`, `main.rs:117-119`) and still runs
*before* both output paths: `main.rs:82-84` precedes `serde_json::to_string`
(`main.rs:89`) and `render_table` (`main.rs:97`), so neither the JSON row nor a
table cell can carry an out-of-grammar or over-bound slug from a third-party
`dyn DiscoveryPlugin`. The extraction adds no bypass: the return value is the
only thing `main` prints for refusals. Reproduced:
`cargo test -p discovery-cli --bin discovery-cli` → 5 passed.

**2. Path handling — holds, strengthened; no new traversal surface.**
`load_geometry` keeps `is_slug_name` first (`placement.rs:162-167`), so `..`, a
separator, an uppercase or absolute spelling is still refused without asking the
filesystem anything; the new bound arm (`placement.rs:168-179`) sits behind it
and *before* `dir.join(&file_name)` (`placement.rs:180-181`). An arm placed
after the grammar check can only add a refusal, never widen a path. Neither new
message mentions `dir`, and the re-pinned test
(`crates/core/tests/placement.rs:330-395`) asserts the scratch path and the
document body stay out of the refusal, and that a name at exactly 128 still
loads. `load_policy`'s split keeps the same property (`policy.rs:49-66`): both
arms precede the join. Reproduced:
`cargo test -p bindforge-core --test placement a_geometry_name_past_the_shared_slug_bound`
→ 1 passed.

**3. Injection (shell / regex / echo) — holds.**
No shell, no command construction, no regex compiled from untrusted data
anywhere in the range. Every value the new arms interpolate into a message —
`{name}`, `{id}`, `{head}` — is by construction already `[a-z][a-z0-9-]*`: each
bound arm runs only after its grammar arm passed, so no control character,
separator, quote or case variant can reach an error string. Message rendering
stays behind `CoreError::InvalidValue`'s Display, which runs *both* key and
message through `escape_and_truncate` (`crates/core/src/error.rs:147-154`,
`MESSAGE_VALUE_CHARS = 80` at `error.rs:30`); the new "bound leads, offender
follows" wording is what keeps the bound number inside that 80-character cut
instead of truncating it away. The one new text-processing routine,
`ledger_annotations.rs`'s `sentences`/`has_number` pair, is a byte scan over a
checked-in document with `checked_sub` and `slice::get` bounds handling — no
panic path, no untrusted input.

**4. Secrets and PII — holds, newly pinned.**
The refusal line names the slug and no install path. The new in-source CLI
tests put a deliberate canary (`paths-must-not-leak-here`) in `install_path`
and assert both that no refusal line contains it and that no line contains a
newline (`main.rs:216-232`, `main.rs:245-256` region of the test module).
Nothing in the range adds a credential, host identity, or real user path to
source, fixtures or log lines.

**5. New dependencies — none.** No `Cargo.toml` or `Cargo.lock` line in the
range. The new `const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;`
(`apps/discovery-cli/tests/cli_slug_refusal_tests.rs:33-37`) rides the
`bindforge-core = { workspace = true }` entry already present at
`apps/discovery-cli/Cargo.toml:10`, and it *removes* a second literal home for
the bound rather than adding one — the direction `024-B1` asks for. No advisory
surface to check.

**6. Filesystem writes — none in production code.** The one new file-reading
suite, `crates/core/tests/ledger_annotations.rs`, is read-only: `repo_root()`
resolves at compile time from `env!("CARGO_MANIFEST_DIR")`
(`crates/core/tests/common/mod.rs:50-57`) and `fixture_text` is
`fs::read_to_string` (`common/mod.rs:171-173`); no writes, no network. Scratch
fixtures use the existing cleanup-on-drop harness.

**7. Clean rejection of hostile input — holds.** The two splits keep the
variant and key unchanged: `check_pin`'s `fault` closure is itself a
`CoreError::InvalidValue { key: "for", .. }` (`document.rs:1220-1223`), and the
new arms produce the same variant with the same `key` as the arms they replace,
so no caller-visible or fixture-corpus fault-kind drift, no panic, no stack
trace, and no partial application — the geometry load refuses before opening a
file (the test writes a *valid* document at the past-bound name precisely to
prove it was never opened). At the discovery collision seam a refused name is
removed and counted, and the new unit module pins `skipped` growing by exactly
the refused row count (`cargo test -p discovery --lib` → 2 passed).

## Regression check against what r1 approved

The complete behavior delta of this range is: (a) one *additional* refusal at a
load seam that previously accepted a past-bound geometry name; (b) two refusal
messages re-worded inside unchanged error variants and keys; (c) one
behavior-preserving function extraction at the CLI emit boundary; (d) test,
fixture, README/spec-sentence and ledger-annotation edits. None of these moves
a check later in a path, weakens an echo, adds an unvalidated entry point, or
drops a rider from the bound's enforcement — the direction of every change is
toward more validation. The `024-L1` echo discipline, the `024-B1`
one-constant rule and the grammar-before-filesystem discipline are all intact
and now better pinned.

## Evidence

Runner's gate record (2026-11-18T21:37:49.945Z, green): `npm run check`,
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo test --workspace --locked` all exit 0.

Run myself in this pass (all green, consistent with the gate):

- `cargo test -p bindforge-core --test placement a_geometry_name_past_the_shared_slug_bound`
  → `test result: ok. 1 passed; 0 failed`
- `cargo test -p bindforge-core --test ledger_annotations` → `ok. 4 passed; 0 failed`
- `cargo test -p discovery-cli --bin discovery-cli` → `ok. 5 passed; 0 failed`
- `cargo test -p discovery --lib` → `ok. 2 passed; 0 failed`

## Findings

None at any severity.

## FYI (no severity, outside this ratification pass's scope)

- `escape_and_truncate` (`crates/core/src/error.rs:190-204`) escapes backslashes
  and Unicode `Cc` control characters and caps length, but Unicode *format*
  characters (e.g. U+202E bidi override) in a hostile plugin-supplied slug pass
  through verbatim into the CLI's refusal line and could visually reorder
  terminal text. The renderer is untouched by this range and predates it, so it
  is recorded for a future renderer/grammar pass, not against this branch.
- Out of seat: `schemas/policy/v1/spec.md:89-92` drops "a geometry name" from
  that sentence's rider list while the gated one-stated-home bullet in
  `schemas/README.md:127-135` still names it.

## Method note

One `git diff` invocation in this pass used a `docs` pathspec and therefore
printed run-record transcript text from `docs/designs/`. That content was not
used as review input; this verdict rests on the code and test diff, the full
current `02-proposal.md`, ADR-024, the runner's gate record, and the read-only
check commands I ran myself.
