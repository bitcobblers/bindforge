# complexity review r3c0

verdict: request-changes

Scope: whole document, round 3 full-panel pass c0. Inputs used: `02-proposal.md`
(incl. its Revision log), `docs/tasks/T-001-steam-linux-discovery.md`, and
read-only inspection of the code the proposal cites (`crates/discovery/src/*`,
`apps/discovery-cli/*`, `crates/core/src/toml.rs`, `Cargo.lock`,
`rust-toolchain.toml`, `.github/workflows/check.yml`, `schemas/*/spec.md` budget
clauses) plus the pinned-toolchain std docs. No other run record opened.

## Findings

- [major][new] *Assumption register* A9 + *Chosen design §2 Root selection* —
  "`std::env::home_dir()` yields the home directory (on Unix `$HOME`, else the
  passwd entry)" / "`home` comes from `std::env::home_dir()` (A9)" — on the
  toolchain this repo pins, that function is deprecated, so the design fails
  its own gate. `rust-toolchain.toml` pins `channel = "1.85.0"`, and the std
  docs for exactly that version carry the stability banner "Deprecated since
  1.29.0: This function's behavior may be unexpected on Windows. Consider using
  a crate from crates.io instead." (the attribute was removed only in 1.87.0).
  A call in `from_host()` therefore emits the warn-by-default `deprecated`
  lint, which `.github/workflows/check.yml:31`
  (`cargo clippy --workspace --all-targets -- -D warnings`) turns into an
  error. So §10's "`Cargo.lock` stays byte-identical", *Test strategy* #10 and
  *Rollout* #3 cannot all hold with §2's chosen home-directory source: either
  the gate breaks, or the zero-dependency posture breaks, or an undeclared
  escape hatch appears in the implementation. A9's evidence is today's std doc
  page, which post-dates the undeprecation and so does not describe the pinned
  floor. Fix: in §2/A9 name `std::env::var("HOME")` as the source of `home` —
  `host_candidates(home, env)` already models exactly that shape, so no new
  mechanism appears, `Cargo.lock` stays byte-identical, and §7's existing
  "no home directory → `Ok(vec![])`" branch already covers the unset case (the
  passwd fallback is the only thing given up) — or, if the passwd fallback is
  judged necessary, say so here and state that the single call site carries
  `#[allow(deprecated)]` with a comment naming the 1.87 floor that retires it.
  Adding `dirs` is a third option but then §10, A15, Test #10 and Rollout #3's
  "no lock diff" check must be amended in the same edit.

- [minor][new] *§4 KeyValues reader* + *§10 Dependencies and gates* — "Files
  larger than 1 MiB are skipped as malformed (A16)" and *Risks*' "Both caps are
  single named constants" — these two caps are not this design's own numbers:
  they are the repo's documented shared parse budget (`schemas/reach-classes/v1
  /spec.md` "larger than 1 MiB or nested deeper than 16 levels are rejected
  before parsing", and the same clause in every other `spec.md`), already
  implemented as named constants `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` in
  `crates/core/src/toml.rs:36-39`. The proposal re-declares the pair as
  plugin-local numbers with no cross-reference and no word on why
  `crates/discovery` does not take the `bindforge-core` edge to reuse them, so
  a reader cannot tell whether the match is intentional, and the next budget
  change has two unlinked copies. Fix: one sentence in §4 tying both caps to
  the documented budget ("the `schemas/README.md` parse budget restated as
  plugin-local constants, because `crates/discovery` deliberately has no
  `bindforge-core` dependency"), which also documents the reuse decision §10's
  no-new-dependency posture rests on. Do not add the dependency for this: it
  would break §10's byte-identical-lock claim for two constants.

- [minor][new] *§1 Modules and public surface* — "Every `pub` item above is
  exercised by a test in *Test strategy*; the rest (scan loop, composition,
  comparison key) stays private" — that sentence is the design's justification
  for 13 public items, and it is false for one of them:
  `SteamLinuxPlugin::from_host()` is `pub` but no test calls it — #5 tests the
  pure `host_candidates`, #7/#8 use `from_roots`, and #10 states "no test reads
  the developer's home directory". The only production entry point (`§8`'s
  default path, *Rollout* #4's "Default on merge") is thus outside the suite
  while being cited as covered. Fix: amend the sentence to name the exception —
  "every `pub` item except `from_host()`, whose candidate list #5 pins and whose
  host probe *Rollout* #5 checks manually" — rather than widening the public
  surface on a claim the test list does not support.

- [nit][new] *§1* ("`keyvalues.rs` — platform-neutral, no Steam knowledge") +
  *Compatibility matrix* Windows row — "`keyvalues.rs` is written
  platform-neutral to be reusable" — the reusability rationale points at a
  consumer that does not exist (*Deferred*: "`SteamWindowsPlugin` … no task
  filed"), which is speculative justification for public surface. The real
  reason the module must be `pub` is the crate's existing convention of external
  integration tests (`crates/discovery/tests/{slug,registry,plugin,game}
  _tests.rs`). Fix: state that reason in §1 and keep the platform-neutral
  wording as a property of the code, not as a promised cross-plugin API.

## FYI (no severity)

- *§3 Library selection* first bullet, "`<root>/steamapps` itself, always",
  cannot be squared with *§5*'s `install_path = l.join("steamapps").join
  ("common")`, with *§3*'s "entries directly inside a selected library's
  `steamapps/`", or with *§2*'s rule that scanned directories are "dropped … if
  they lack the required `steamapps` directory": as written, the always-scan
  entry either composes to `<root>/steamapps/steamapps/common/…` or is dropped
  by its own gate, and the fixture's `"0" → "."` "root/library de-duping" only
  de-dupes if relative VDF `path` values join to the root rather than to "the
  directory holding the VDF". The three passages need one consistent unit for
  "library". — correctness seat.
- Same section pair: on 1.85.0 `home_dir()` returns `Some("")` when `$HOME` is
  set but empty (the 1.85.0 doc says "including to an empty string"; today's doc
  says "and not an empty string"), so §2's "`None` … the candidate list is
  empty" branch does not cover that case; the canonical path rule drops the
  resulting relative candidates, so the outcome is safe but untested. —
  correctness seat.
- *§5*: "because it is appended unvalidated to the emitted path" contradicts
  the rejection list in the same bullet. — correctness seat.
- *Options considered*: the hand-rolled-reader decision cites only external
  precedent; `crates/core/src/toml.rs` is the in-repo precedent for the same
  posture (hand-rolled grammar subset, positioned errors, the same two caps) and
  strengthens the argument. Not a reuse violation — its TOML-subset grammar and
  `CoreError` model do not parse KeyValues — so nothing is reusable beyond the
  constants in the minor finding above.

## Value check (checklist #7)

Proportionate. Moving parts trace to Goals or to the task's Done-when: the
plugin and its two constructors, the pure candidate list, the `--steam-root`
seam, the subset reader, the single `Err` path. `tools/` was checked for reuse
(it is a Node CLI — nothing Rust), and `crates/core/src/toml.rs` is the only
adjacent Rust code, whose grammar is not reusable here. A 30–50% smaller design
was considered and rejected: (a) key-scraping the two files instead of a section
parser loses the positioned errors and the traversal validation §5 depends on,
and cuts against the repo's existing reader precedent; (b) trimming the
candidate list to native layouts only re-opens r0 q1, whose rebuttal (A8's
coexisting layouts; the task's "a Linux machine with Steam") is sound. *Rollout*
#1–#2 already defers the CLI half behind a thin crate-only first slice, and the
filter, diagnostics, error-taxonomy and budget-harness additions are deferred
rather than built. The only removable volume found is the duplicated budget
constants in the minor finding above.
