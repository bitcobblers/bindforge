# compatibility review r1 — dissent pass c1

verdict: request-changes

Scope: `git diff 4a75e3d7 9ff2ec78` (02-proposal.md blobs), then the full current
document. Inputs: `02-proposal.md`, `docs/adrs/026-registry-service.md`, the
Revision log, my r1c0 findings. Code claims checked with read-only inspection
(`crates/core/tests/{conformance.rs,common/mod.rs,version_gate.rs,slug_bound_corpus.rs}`,
`crates/discovery/tests/boundary_tests.rs`, `crates/discovery/src/game.rs`,
`crates/core/src/placement.rs`, `apps/discovery-cli/src/{matches.rs}`,
`apps/discovery-cli/Cargo.toml`, `data/`).

## Closures from r1c0 — both resolved, not re-raised

- **[major] exotic-Unix row carried `unknown`** — CLOSED. The row now reads
  `| Other / exotic Unix (BSD et al.) | No | Not a v1 target: … nothing exercises
  or promises it. |` and the footer names the three `Yes` rows as the rows the
  tasks owe behavior to. Every row now carries a decision; checklist #1's
  `unknown` prohibition is satisfied, and Test #19 needs nothing new because the
  row is unsupported. (Its landing spot collides with row 1's label — reported
  as finding 3, a different defect in the same table.)
- **[minor] shipped JSON fixtures parsed by nothing** — CLOSED. Test #1 now reads
  "every `fixtures/valid/*.json` loads; each of — … — ships in
  `fixtures/invalid/*.json` and is a named refusal", and the `conformance.rs`
  bullet records that `fixture_file_names_…` "keeps governing the `.json`
  fixtures". The *intent* is now in the document; the sentence that carries it is
  self-contradictory as an instruction — finding 2.

## Checklist mapping

1. Matrix rows: all four carry Yes/No — item satisfied as to *status*; one
   label collision (finding 3).
2. OS input APIs (X11/Wayland/Raw Input/HID): **not applicable** — `apps/registry`
   touches no input device; its only OS surface is the filesystem and `std::net`.
3. Node version floor: **not applicable** — no `tools/orchestration` change in
   the diff (`AGENTS.md` npm legs untouched).
4. Native modules: **not applicable**; the one dependency the revision adds to
   the design's dependency line, `serde_json`, is already a shipped-client
   dependency (verified `apps/discovery-cli/Cargo.toml:13`), so no new native or
   build-script surface.
5. Keyboard reality / non-ASCII names: satisfied — `game_id` is ASCII-only via
   `is_slug_name` (verified `crates/core/src/placement.rs:107-117`), `name` is
   octet-refused and never truncated on a serving path, and the percent-decode-
   then-validate order (#3) is right. The one real-world-alphabet defect the
   revision introduces is the lookup-key dot ban — finding 1.
6. File paths / per-OS config dirs: satisfied — the copied `<data>` rule,
   `StoreLayout`/`data_dir`/`store_path` exist as claimed
   (verified `apps/discovery-cli/src/matches.rs:46-103`); paths are named by
   join-shaped segments, never string-concatenated. The new key validator's
   path-safety rationale is disclaimed by its own paragraph (finding 1).
7. Platform test per supported platform: satisfied — #19 names Linux (cargo
   gates), Windows and macOS (#13 seam + case-unique fixture names); the `No`
   row needs none.
- A10 (`version_gate.rs`: **no change**) verified true: the invariant is a set
  equality over major names (`crates/core/tests/version_gate.rs:392-403`), and
  `catalog/v1` contributes the already-present `v1`. The other `format_dirs()`
  consumers I checked — `the_walk_picks_up_every_version_directory_the_contract_tree_ships`
  (conformance.rs:404-426) and `slug_bound_corpus.rs`'s `SLUG_BEARING_TREES`
  roster — are inclusion-only assertions, so a new tree costs them nothing. The
  file list is complete on that axis; it is incomplete on the
  `boundary_tests` axis only in coverage, not in files (finding 4).

## Findings

- [major][new] §The serving face step 1 ("after the first `/`, exactly one
  non-empty `.`-free component, no embedded `/`") + §Catalog document and its
  spec ("a `source_key` failing `spells_lookup_key` (A16) or the tightened key
  grammar of §The serving face step 1") + §Budgets ("any `slug` failing §The
  serving face step 1's validator by name") — the new validator's dot ban makes
  the registry's admitted lookup-key vocabulary **narrower than every shipped
  spelling rule for the same keys**: `Source::spells_lookup_key`
  (`crates/discovery/src/game.rs:84-89`) admits any non-empty external id after
  a closed-set token, `Source::lookup_key` (`game.rs:66-68`) — the single
  composer, `Source::ALL` including `Source::Local` — imposes nothing, and the
  shipped store validates a slug as exactly "len ≤ `MAX_DISCOVERY_SLUG_OCTETS` +
  `spells_lookup_key`" (`apps/discovery-cli/src/matches.rs:222-234`). So
  `local/Steam.SE` is a legal slug the shipped client stores and sends as
  `slug=`, and the revision refuses it at four seams at once: candidate query
  (`404`), admin upsert (refused by name — closing ADR-026's own `local/<dir>`
  curation path: "a `local/<dir>` install has a lookup key" and "giving
  `local/`-source installs a curation path no public root can own"), corpus load
  (a curated corpus can never carry such a key), and cache load (one dotted slug
  refuses the whole cache document). The rule's stated rationale is path safety —
  "the grammar that keeps a `..` or a separator from ever asking the filesystem
  for a path" — which the same paragraph disclaims ("Lookup keys are opaque
  in-memory strings, never joined onto paths"), and whose named precedent
  `is_slug_name` governs `game_id`, already bound by A4. No shipped emitter
  breaks today (`crates/discovery/src/steam_common.rs:249` composes only
  `steam/<appid>`), which is why this is a vocabulary narrowing rather than a
  today-failure — and why it must be decided now: `T-005`…`T-007` would encode
  it into the wire contract the ADR says is "untouched". Fix: in §The serving
  face step 1 replace "exactly one non-empty `.`-free component, no embedded `/`"
  with "exactly one non-empty component with no embedded `/`", and move the dot
  ban to the place its precedent actually lives — one clause naming
  `is_slug_name`/A4 as the `game_id`-only rule; carry the same substitution into
  the §Catalog document refusal list and the §Budgets cache-load clause, and
  re-word #3 and #4 so a dotted `local/…` key is an admitted spelling rather
  than a `404`/refusal. If the ban is wanted, keep it deliberately: say so there,
  name it as a narrowing of `026`'s `local/<dir>` use case, and say which task
  refuses dotted local keys loudly — replacing the path-safety rationale the
  paragraph already disclaims.

- [minor][new] §Catalog document and its spec — "`crates/core/tests/conformance.rs`:
  guard the fixture-tree loops — `every_valid_fixture_…`, both
  `every_invalid_fixture_…` message tests, and `fixture_file_names_…` (which
  keeps governing the `.json` fixtures: lowercase dash-separated names,
  case-unique) — behind the core-TOML check" — one sentence says both that
  `fixture_file_names_…` is moved behind the core-TOML check (so it does not run
  for `catalog`) and that it "keeps governing the `.json` fixtures"; the
  instruction is undecidable as written. The property at stake is a platform
  one: the shipped test
  `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique`
  (`crates/core/tests/conformance.rs:336-376`) walks `format_dirs()` — the same
  walk that picks `catalog/v1` up on its own — and its doc-comment names exactly
  what it buys: names "case-unique within a fixture directory, which is what
  keeps the same corpus green on a contributor's case-insensitive Windows
  filesystem as on the Linux CI runner". That is the only mechanism behind the
  Windows `Yes` row's corpus claim and Test #19's "case-unique fixture names".
  Mechanically related: `is_fixture_name` takes a bare file name
  (`crates/core/tests/common/mod.rs:421-435`), so "`is_fixture_name` accepts
  `.json` for that directory only" already requires the predicate to learn the
  directory — and whether this test is guarded decides whether that widening is
  exercised at all. Fix: drop `fixture_file_names_…` from the guarded list
  instead of guarding it with a parenthetical that reverses the guard, and put
  the reason in the `common/mod.rs` bullet: "`is_fixture_name` takes the format
  name (or a per-directory suffix set) so the unguarded `fixture_file_names_…`
  walk keeps its lowercase/`-`-separated and case-uniqueness rule over
  `catalog/v1`'s `.json` corpus — the Windows case-insensitive-filesystem
  claim's only coverage." Text replacement; no new mechanism.

- [minor][new] §Compatibility matrix — "| Linux & other Unix | Yes |" together
  with the new footer "the three `Yes` rows (Linux & other-Unix CI, macOS,
  Windows) are the rows the tasks owe behavior to", against "| Other / exotic
  Unix (BSD et al.) | No | Not a v1 target … nothing exercises or promises it."
  — the r1 fix decided the fourth row but landed on a label that overlaps the
  first: "other Unix" is simultaneously a `Yes` the tasks owe behavior to and
  "not a v1 target … nothing promises it". A `T-005`…`T-008` author cannot tell
  whether a BSD build is owed behavior or explicitly refused, which is the same
  failure mode as an undecided row under a different name. Fix: rename the first
  row's label to the platform it actually promises — "Linux" (its own Notes
  already say "Primary CI target") — and the footer's parenthetical to "(Linux
  CI, macOS, Windows)"; if the design means to promise the generic-Unix
  data-dir branch beyond BSD, give it its own row with its own status instead of
  leaving it inside row 1's "other Unix". Labels only; #19 already names the
  three supported platforms and needs no change.

- [minor][new] §Catalog document and its spec — "`crates/discovery/tests/boundary_tests.rs`:
  … so re-scope the sweep to `data/manifests` + `data/profiles`, with its
  doc-comment and `checked > 0` guard re-pinned" — the amendment removes a
  coverage half nobody asked it to remove. Verified shipped shape
  (`crates/discovery/tests/boundary_tests.rs:110-156`): the sweep reads every
  file under `data/` **and** every `schemas/<format>/<major>/fixtures/valid/`,
  and its doc-comment pins "no source key or friendly name reaches a manifest, a
  profile, or a path" against precisely those trees. `026-S1` requires
  `data/catalog/` and `schemas/catalog/v1/` to be exempt; re-scoping to the two
  `data/` directories also drops `schemas/manifests/v1/` and
  `schemas/profile/v{1,2}/` valid fixtures — the trees the pinned rule names —
  from the gate, permanently. Fix: replace the re-scope with a carve-out that
  moves coverage only where `026-S1` forces it — "keep both current roots; skip
  files under `data/catalog/` and `schemas/catalog/`; amend the doc-comment to
  name `catalog` as the one tree where a lookup key is legal content" — and then
  the `checked > 0` guard needs no re-pinning (`data/manifests/` and
  `data/profiles/` exist and are already counted either way).

- [nit][new] §`apps/registry` CLI surface ("relocated in tests through the same
  env seam as #13") + Test #13 ("this seam is how caching tests relocate the
  local-entries document (no dedicated CLI knob)") — dropping `--local-entries
  <path>` leaves the copied `<data>` rule's env inputs as the only relocation
  lever, and the design names the seam (the in-process `StoreLayout`/`data_dir`
  env closure) without naming the variables, while #17's caching end-to-end runs
  `registry serve --upstream` as a subprocess that writes both `catalog-v1.json`
  and `cache-v1.json`. Fix: one clause in #13 — "for the subprocess cases (#17),
  set `XDG_DATA_HOME` (Unix) / `APPDATA` (Windows) in the child environment;
  there is no path knob" — so the task author doesn't reintroduce a flag to
  compensate.

## FYI (no severity)

- §Budgets: "evicts the oldest entry by **insertion sequence** (ties by
  `fetched_at`, …)" does not implement `026-B1`'s stated rule ("over the cap the
  oldest `fetched_at` is evicted", ADR-026 Budgets): a refreshed entry keeps its
  original insertion slot, so it can be evicted first while holding the newest
  `fetched_at`. Correctness seat's call, not mine.
- Compatibility matrix Windows row names only `cache-v1.json`, though caching
  mode also writes `catalog-v1.json` (§CLI surface); naming both would make the
  row readable on its own.
