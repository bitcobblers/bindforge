# correctness review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 2efd6ad… 4461833…` of `02-proposal.md`, then the full current
document. Inputs: the proposal, ADR-025, its Revision log, and this seat's r1
findings. No review report or transcript was opened.

Checklist disposition: (1) assumption register — one `verified` row still
over-reads its basis (finding 7); (2) external spot-check — re-checked RFC 3986
§3.2/§3.2.2/§3.4 live (authority terminated by `/`, `?`, `#` or end; bracketed
IP-literal host; `/` legal in a query value; §7.4 rare address formats): the
proposal's URL/encoding claims hold, and `0177.0.0.1` refusal is grounded; the
count of exported core constants is 12 (`crates/core/src/lib.rs:102-121`), so
"thirteenth" and the twelve-to-thirteen sentence are right; (3)/(4)/(5)
internal agreement and edge cases — findings 1–6, 8–10; (6) the design decides
what ADR-025 asked (contracts, laws, `025-B1`, Rollout items 1-6 all landed);
(7) r1 blocker/major closure — the r1 major is closed for real, not reworded.

## r1 closure check

- r1 #1 (major, CLI harness migration + unproducible fixture route) — **closed**:
  Rollout 3 now names the four subprocess suites behind a loopback
  `TcpListener` fake and deletes the fixture route in favour of an in-module
  `main.rs` pin; Revision log r1 records both. Verified against the code: the
  suites do drive `CARGO_BIN_EXE_discovery-cli` and `cli_slug_refusal_tests`'
  `SLUG_BOUND`-shaped names are unproducible once a slug is `steam/<appid>`.
  Residual half: the store env, reported new as finding 1.
- r1 #2 (minor, retirement scope pinned to the string) — **closed**: #14 now
  pins resolution in `src/` and in assertions, and Rollout 1 names the five
  re-pinned suites. Verified: `[Nn]amespace` and `derive_slug` appear in exactly
  `plugin_tests`, `game_tests`, `emission_tests`, `steam_linux_tests`,
  `steam_windows_tests`, `zz_api_probe` (all named) plus the deleted
  `slug_tests`/`collision_tests`; `tests/common/mod.rs` touches neither.
- r1 #3 (minor, friendly-name "structural proof") — **closed**: the boundary
  bullet is now source-key-only and states that no grammar assertion is made
  about friendly names, which is the property `025-L5` needs.
- r1 #4 (minor, one record per slug and which row) — **closed** in the store
  bullet and step 6.
- r1 #5 (minor, A10 basis) — **partially closed**, the CLI restatement is right
  and now cites `apps/discovery-cli/Cargo.toml:6-13`; the added "workspace's
  only external deps" clause is false as worded. Finding 7.
- r1 #6 (minor, README bullet borrowing clause) — **closed**; both existing
  bullets stay intact (`schemas/README.md:99-141` unchanged by the design).
- r1 #7 (nit, unreachable plugin-side bound case) — **closed** as an in-module
  `AppManifest` pin, correctly labelled defence in depth.

## Findings

- [minor][new] Rollout 3 — "including the migration that keeps the gate from
  breaking the pre-existing subprocess suites: `cli_tests`,
  `cli_output_polish_tests`, `cli_dual_plugin_tests` and `cli_slug_refusal_tests`
  run behind a loopback `TcpListener` fake" — the migration covers the gate
  only, but every one of those runs also loads the match store (and Test #12's
  round-trip writes it) through the *subprocess's* `XDG_DATA_HOME`/`HOME`.
  Verified: `cli_tests.rs:24` and `cli_output_polish_tests.rs:21` build the
  command with no `.env` call, so they inherit the developer's `HOME` and read
  (and Test #12 would write) the real `~/.local/share/bindforge/matches.json`;
  only `cli_dual_plugin_tests.rs:35-36` clears the environment. Fix: extend the
  clause — each subprocess run pins `XDG_DATA_HOME` and `HOME` to a per-test
  temp dir beside the fake, and Test #12 asserts the one file under that temp
  dir.
- [minor][new] Match store, Save — "`matches.json.tmp` (opened exclusively,
  never through a pre-existing entry) + `std::fs::rename`, after
  `create_dir_all` on the parent" — Load's every failure is a named refusal,
  but a save that cannot create the parent or rename (read-only or missing data
  dir — the route `cli_dual_plugin_tests`' `HOME=/bindforge-no-such-home`
  already walks) has no stated outcome: no refusal name, no exit code, no case
  in Test #7. Fix: name one refusal (`error: match store unwritable: <path>`)
  with `ExitCode::FAILURE`, or state that the confirmation is lost and named;
  pin it as a #7 case with an unwritable parent.
- [minor][new] Match store, Load, read with Scan flow steps 4 and 6 — Load
  refuses "a `slug` past `025-B1` or not spelled `<token>/<external-id>` with
  `<token>` in `Source`'s closed set", step 4 checks "the `025-B1` bound alone",
  and step 6 writes "the one record for the slug" with no shape check. A row
  whose slug did not come from `lookup_key` — the case `refuse_invalid_slugs`'
  own doc comment names, "from any `dyn DiscoveryPlugin`, including a future
  third-party one" (`apps/discovery-cli/src/main.rs:105-111`) — clears the
  boundary, gets confirmed, is written, and then makes the next scan refuse the
  whole store, whose stated remedy ("remove the file") destroys every unrelated
  match. Fix: apply Load's slug-shape rule at the confirmation write, so a
  record Load would reject can never enter the file.
- [minor][new] Match store, Load — "a file over a `65_536`-octet read budget" —
  the budget is unowned arithmetic: one record is at most ~426 octets (the two
  128 bounds plus the 128-octet friendly bound plus JSON punctuation) and
  typically ~95, so the store begins refusing between roughly 150 and ~690
  confirmed matches — inside a large Steam library — and the only remedy is
  deleting every match. Nothing states the derivation or a headroom claim, and
  no record count is capped. Fix: state the budget as records × worst-case
  record size, or replace the byte cap with a named "more than N records"
  refusal so an over-cap store reports the rule it broke.
- [minor][new] Catalog client, `probe()` — "after refusing unless every returned
  `SocketAddr` `IpAddr::is_loopback()` — the property is established by address,
  not by name" — that check sits only on the probe, while step 5 makes one
  `candidates()` call per unresolved slug and nothing says those requests reuse
  the validated address rather than re-resolving the name. The guarantee the
  Risks row claims ("This build can only reach a loopback authority") is then
  established before enumeration and not on the requests that actually carry
  the inventory. Fix: one clause — `HttpCatalog` connects to the `SocketAddr`
  the probe validated, or `candidates()` repeats the same `is_loopback()`
  refusal per resolved address.
- [minor][new] Scan flow step 8 — "the two ACF-sourced cells — `Display Name`,
  `Install Path` — render through `escape_and_truncate` in place of today's raw
  `row_cells` pass-through" — that function is the structured-error value
  renderer (`crates/core/src/error.rs:190-204`): it cuts at
  `MESSAGE_VALUE_CHARS = 80` characters with a `…` and doubles every `\`.
  Adopting it for table cells silently clips long display names and install
  paths, and renders a Windows path with doubled backslashes — a render-contract
  change the design never states, while Test #12 pins only an ESC byte. Fix:
  state both consequences in step 8 and pin them in #12, or escape control
  bytes without the 80-character cut if clipping is unwanted.
- [minor][re-raised] Assumption register A10 — "those are the workspace's only
  external deps" — the row's own basis contradicts the wording: `Cargo.toml`
  `[workspace.dependencies]` carries `bindforge-core`, `serde` and `winreg`, and
  comments that `clap` "enter[s] when their consumer lands", so the workspace's
  external deps are `serde`, `winreg`, `clap`, `serde_json`. The five names in
  A10 include two path deps (`discovery`, `bindforge-core`) and omit `winreg`.
  (The conclusion — no `Cargo.lock` change — is safe: the CLI's five deps are
  exactly as listed and cover the new modules.) Fix: drop the clause, or restate
  "the workspace's external deps are `serde`, `winreg`, `clap`, `serde_json`;
  the CLI's are `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`".
- [nit][new] Catalog client, endpoint policy — "an authority that is non-empty,
  carries no `@`, and runs to the first `/`, `?`, or `#`, with an optional `:`
  and digits-only port; and a host that is exactly `127.0.0.1`, `[::1]`, or
  `localhost`" — with `[::1]` admitted, the "optional `:`" split is ambiguous,
  since that host is itself colon-bearing, and Test #9 pins acceptance only for
  `http://127.0.0.1:PORT`. Fix: mirror RFC 3986 §3.2.2 in one clause — a leading
  `[` host runs to its `]` before the optional port colon — and add
  `http://[::1]:PORT` to #9's accepted list.
- [nit][new] Catalog client — "`fn candidates_url(base: &str, slug: &str) ->
  String` is pure" next to the new "`fn parse_endpoint(base: &str) ->
  Result<Endpoint, CatalogError>`" — one thing, two shapes: it is undefined
  whether `candidates_url` receives the raw flag/env string (so
  `http://127.0.0.1:8080/prefix` yields `/prefix/v1/candidates`) or the parsed
  `Endpoint` (so a path prefix is silently dropped). Fix: name which one, in
  one clause, and say what happens to a base path.
- [nit][new] Scan flow step 6 — "writes the one record for the slug, `(slug,
  game_id, friendly name of the first sharing row in `(slug, install_path)`
  order)`" — the nested inline code spans close the first span mid-sentence, so
  the tuple this rule now hinges on renders mangled. Fix: one span without inner
  backticks.

## FYI (no severity)

- Performance: step 5 is one blocking GET per distinct unresolved slug with a
  2 s connect and 5 s read timeout, and a mid-scan failure discards the whole
  table — a first scan of a 500-game library is bounded by 500 × 5 s before any
  output.
- Security: step 8 escapes the two ACF-sourced cells but leaves the `Slug` cell
  raw, which is safe only for rows built through `lookup_key`; and the
  owner-only 0700/0600 store is still plaintext installed-game inventory.
- Compatibility: "the store holds one record per distinct slug" reads against
  `025-C3`'s "one record per confirmed install" for two installs that share one
  slug (one appid in two libraries); the rule is coherent, but the ADR's wording
  is not the one the design ships.
