# Code review — security seat — round 2, pass c0

- Diff scope: `git diff 7b036617dfa2d7895888b73c9b9412387b1d7e08...HEAD -- ':(exclude)docs/designs'`
- Inputs used: that diff, `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`, the runner's gate record, my re-run of `cargo test --workspace --locked`.
- Specialty applied: `references/security.md` only.
- Verdict: **approve** (zero blocker, zero major; two minor, one nit).

## Gate evidence

The runner's record is green on all four legs. The single quoted
`test result: ok. 0 passed; ...` line matches the workspace's `Doc-tests`
legs; my own re-run of `cargo test --workspace --locked` shows every real
suite running and passing (among them `cli_catalog_gate_tests` 7,
`cli_confirmation_flow_tests` 6, `cli_match_store_tests` 12,
`boundary_tests` 4, `retirement_tests` 5, plus the in-module unit suites).
The green record is therefore consistent evidence for the behavior claims
cited below.

## Checklist walk

1. **External input validated at the boundary** — pass.
   Store load (`matches.rs:176-245`) budgets the file at 65 536 octets,
   pins `version == 1`, validates every record's `slug` against
   `025-B1` + `Source::spells_lookup_key` (closed token set, game.rs:84-89)
   and every `game_id` against `is_slug_name` + `MAX_SLUG_OCTETS`
   (`matches.rs:354-356`); refusals rewrite nothing and name the path
   through `echo_path`. The response parse (`catalog.rs:283-403`) admits
   only a `200`, refuses any `Transfer-Encoding` case-insensitively,
   requires exactly one 1–5-digit `Content-Length` ≤ 65 536, caps the
   header block at the same budget before allocating, and caps the body
   at 256 candidates; a hostile `game_id` is dropped by
   `partition_candidates` (`catalog.rs:452-466`). The ACF appid is 1–20
   ASCII digits (`steam_common.rs:305-309`) and an unusable one is the
   count-only skip, never a fallback spelling. See nit on the read-budget
   race below.
2. **Path handling** — pass. No untrusted value names a path component:
   the store paths are the constants `bindforge/matches.json` and
   `matches.json.tmp` (`matches.rs:104,335`); `game_id` — the only
   downstream path component — is grammar-validated at candidate parse,
   confirmation write (`matches.rs:281-300`), and store load. The ACF
   `installdir` traversal rules (`steam_common.rs:319-327`: no `.`, `..`,
   absolute, `/`, interior NUL, Windows `\`) are preserved from the base
   commit. Env data-dir values must be absolute to count as set
   (`matches.rs:69-99`).
3. **Injection** — pass with one minor (finding S1). `percent_encode`
   (`catalog.rs:174-185`) encodes every byte outside RFC 3986 unreserved,
   so the request target cannot carry CR/LF or spaces; no shell, no
   regex in the diff. Two stderr sites interpolate the slug raw.
4. **Secrets and PII** — pass. The confirmed inventory
   (`matches.json`) is created owner-only (dir 0700 via
   `DirBuilder::mode(0o700)` + reassert; both files 0600 via
   `create_new` mode 0600 + reassert, `matches.rs:363-411`), so the
   disclosure is not a umask accident. The client cannot leave the host
   (endpoint policy + probe loopback check), so no inventory egress
   before ADR-026. No secrets in fixtures; the new invalid fixture
   carries only `id = "steam/900001"`.
5. **New dependencies** — none. The Cargo.toml hunks add only
   workspace-local dev-dependencies (`bindforge-core`, `discovery`,
   `serde_json` for CLI tests); no `Cargo.lock` change in the diff, as
   the proposal (Rollout 7, A10) required. The HTTP client is std-only.
6. **Filesystem writes** — pass. Save is temp-name + `create_new`
   (exclusive, refuses a pre-existing `matches.json.tmp`, symlink-safe as
   an entry check) + `fs::rename` (atomically replaces the destination,
   including a symlinked `matches.json`, rather than writing through it),
   only after at least one confirmation, never in JSON mode
   (`main.rs:235-247`).
7. **Clean rejection** — pass. Every failure is one named single-line
   `error:` to stderr with `ExitCode::FAILURE`; a mid-scan query failure
   fails closed with no partial table (`main.rs:192-196`); malformed JSON,
   over-budget files, unknown versions, and grammar-invalid ids reject
   without partial application.

## Findings

### S1 — minor — `apps/discovery-cli/src/catalog.rs:460` and `apps/discovery-cli/src/main.rs:295`: the lookup key is interpolated into stderr without `escape_and_truncate`

The design's escape invariant states that "every value a refusal
interpolates — a slug, a candidate `name`, a rejected `game_id`, the
store `<path>`, the connect `<reason>` — goes through
`bindforge_core::escape_and_truncate`, so no outside string forges a
line" (02-proposal.md, "Catalog client and gate"), and core's own
contract for the renderer says a "third-party plugin's slug" can "never
put a second line … on an operator's terminal whatever it carries"
(`crates/core/src/error.rs:183-188`). Two new sites violate it:
`partition_candidates` formats `` "dropped candidate for `{slug}` with
an invalid game id: …" `` with the slug raw (only the `game_id` is
escaped), and `print_prompt` formats `` "Discovered game `{slug}` has
catalog candidates:" `` raw. The emit boundary is deliberately
bound-only (`main.rs:277-290`), so the content of a third-party plugin's
slug is unvalidated by the time these lines print; a slug carrying a
newline plus text can forge a fake `error:` line on stderr. Shipped
emitters cannot reach it (appid is digits-only), which is why this is
minor and not major; the table path already sets the right precedent by
passing the slug through `cell()` (`main.rs:402`).
Fix: `escape_and_truncate(&slug)` inside both format strings (the value
is display-only in these lines), and pin the one-line property on the
partition line the way `main.rs`'s in-module tests pin the bound-refusal
line.

### S2 — minor — `apps/discovery-cli/src/catalog.rs:408-431`: `candidates()` connects to freshly resolved addresses without re-asserting loopback

`probe()` establishes the loopback property "by address, not by name"
(02-proposal.md, "Catalog client and gate"): it refuses unless every
resolved `SocketAddr` `is_loopback()` (`catalog.rs:247-254`). The
mid-scan `candidates()` path re-resolves the host on every query
(`catalog.rs:410-415`) and connects to whatever comes back
(`catalog.rs:418-428`) with no loopback check. For the `127.0.0.1` and
`[::1]` literals that is equivalent, but for the admitted `localhost`
spelling the name→address binding can change between the gate (step 1)
and a query (step 5) — a hosts/NSS edit mid-run sends the lookup key
(the installed-game inventory's id) to a non-loopback address while the
run believes the gate holds, contradicting "this build … cannot
disclose an installed-game inventory to a remote host"
(02-proposal.md, Options considered / Risks). Narrow window and an
attitude-hostile name service make this minor, not major.
Fix: factor `probe()`'s "resolve, refuse unless every address is
loopback" into one helper shared by `probe()` and `candidates()`, so the
address property is re-asserted at connect time on both paths (and pin
it with a unit case, mirroring the `cli_catalog_gate_tests` refusals).

### S3 — nit — `apps/discovery-cli/src/matches.rs:194-211`: the read budget is checked on a stale `metadata` size before an uncapped `fs::read`

`fs::metadata` decides "over budget" and then `fs::read` allocates the
whole file, with the second length check (`matches.rs:204-211`) running
only after the read has already allocated. A file swapped/grown between
the two calls is fully buffered before the refusal. The budgeted object
is the user's own data-directory file, so this is defense-in-depth, not
an exploitable path. Fix: `File::open` the store and `Read::take(
STORE_READ_BUDGET_OCTETS + 1)` so allocation is capped by construction
and the "over budget" refusal no longer depends on `metadata`.

## Irrelevant checklist items

None marked not-applicable; all seven were evaluated above.

## FYI (no severity, out of security seat)

`crates/discovery/tests/.rm-probe` is a stray zero-byte file committed by
this branch; and `tests/slug_tests.rs` / `tests/collision_tests.rs`
remain at the live `tests/` root, disabled via a `required-features`
gate in `crates/discovery/Cargo.toml`, while `tests/retired/README.md`
describes them as kept "here" under `retired/` — the retirement record
and the tree disagree on placement.
