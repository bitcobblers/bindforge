# security review r1 (dissent pass c1)

verdict: approve

Scope: the c1 diff (`2efd6ad..4461833` of `02-proposal.md`) plus the full current
document. Inputs: `02-proposal.md`, `docs/adrs/025-discovery-identity-and-catalog-matching.md`,
the Revision log, my r1 findings. No other run record opened.

All seven r1 findings are closed by the changed text, and the closures hold
against the code as it stands (verified read-only: `apps/discovery-cli/Cargo.toml:6-13`
and the workspace `Cargo.toml:20-24` match A10's revised claim, and `Cargo.lock`'s
external names are the `clap`, `serde`, `serde_json`, `winreg` trees plus their
transitives — so the new std-only modules really need no `Cargo.lock` change, and
checklist item 5 (dependency advisories) has nothing new to review; `crates/core/src/error.rs:176-204`
confirms `escape_and_truncate` escapes control characters and caps at 80 chars, which
is what the new table-cell rule buys).

- r1-1 (loopback string split) — **closed**: `parse_endpoint` now spells the accepted
  grammar (`http://` literal, authority non-empty, no `@`, ends at the first `/`, `?`
  or `#`, digits-only port, three host literals, "no other IP form parses"), and Test
  strategy 9 pins userinfo, octal-IPv4, empty-host and bad-port spellings.
- r1-2 (name vs address) — **closed** at `probe()`: "refusing unless every returned
  `SocketAddr` `IpAddr::is_loopback()`". Residual for the querying path is below, as a
  new minor, not a re-raise.
- r1-3 (raw ACF table cells) — **closed**: step 8 routes `Display Name`/`Install Path`
  through `escape_and_truncate`, Test 12 pins an ESC byte in a fixture `name`.
- r1-4 (response-parse rules) — **closed**: case-insensitive header names, exactly one
  1–5-digit `Content-Length`, header-block budget, 256-candidate cap; Test 10 pins the
  spellings. The Deferred "candidate count or pagination contract" clause is the
  *server* contract (ADR-026), so the client-side read cap here does not contradict
  ADR-025.
- r1-5 (store modes, read bound, tmp) — **closed** for the umask, budget and
  symlink-following halves; the shared-ancestor case is unaddressed and unrecorded
  (nit below).
- r1-6 (store slug shape) — **closed**: load refuses a slug "not spelled
  `<token>/<external-id>` with `<token>` in `Source`'s closed set", pinned in Test 6.
- r1-7 (interpolated refusal values) — **closed**: the escape clause now names the
  store `<path>` and the connect `<reason>`, Test 6 adds newline-in-`$XDG_DATA_HOME`.

## Findings

- [minor][new] Scan flow and output — "`Display Name`, `Install Path` — render through
  `escape_and_truncate` in place of today's raw `row_cells` pass-through" — the escaped
  set is exactly the two ACF cells, but the same revision retires the only thing that
  kept the `Slug` column clean: step 4 makes `refuse_invalid_slugs` "the `025-B1` bound
  alone", and the retired grammar check is what refused control bytes in a slug today
  (`apps/discovery-cli/src/main.rs:115-129`, whose in-module test pins a newline-in-slug
  reaching stderr as `\u000a`, never as a second line). `DiscoveredGame.slug` is a `pub`
  field (`crates/discovery/src/game.rs:44-51`), so the Row identity claim that "a plugin
  cannot spell the key differently without using another function" does not bound a
  third-party row, and `Source::lookup_key` validates nothing about `external_id`: a
  row from any `dyn DiscoveryPlugin` — the exact population today's boundary doc comment
  says it exists for — can now carry 128 arbitrary octets straight into the raw `Slug`
  cell. Fix: add `Slug` (and `Game Id`, already grammar-validated, for symmetry) to the
  escaped-cell set in step 8 rather than enumerating only the ACF-sourced two, and extend
  Test 12's no-raw-control-byte assertion to every column.
- [minor][new] Scan flow and output, step 6 — "one prompt on **stderr** — stdout
  carries only the table" — the prompt is the one remaining render of a server-sourced
  string, and the revision's escape clause is scoped to "every value a refusal
  interpolates", so a loopback peer's candidate `name` (reachable by any local process
  that binds the port) reaches the terminal outside `escape_and_truncate`, exactly the
  control-sequence path step 8 and Test 12 just closed for the table. Fix: widen the
  clause from refusals to every out-of-band terminal write — the prompt's candidate
  names and slug — and pin a control byte in a fake's candidate `name` in Test 12.
- [minor][new] Catalog client and gate — "`probe()` asks the authority ... after
  refusing unless every returned `SocketAddr` `IpAddr::is_loopback()` — the property is
  established by address, not by name, so a `hosts` entry for `localhost` cannot send
  the query outward" — the check is stated inside `probe()` only, and the sentence
  claims it protects *the query*, which `candidates()` sends. The text gives
  `candidates()` a raw `base: &str` (`candidates_url(base: &str, slug: &str)`) and a
  plain `TcpStream`, so as written the query path re-resolves the name and dials whatever
  comes back, and a resolver answer that differs between the gate and the query — or an
  implementation that puts the check only where the document does — puts the slug on a
  remote socket. Fix: state that the gate resolves once and `Endpoint` carries the
  validated `Vec<SocketAddr>` that `candidates()` dials (so the check applies to every
  connection the client makes, not to the probe alone); note in the same sentence that a
  non-numeric or out-of-range port is the endpoint policy's `TransportUnavailable`, not a
  later `Unreachable`.
- [minor][new] Catalog client and gate — "admits exactly: a literal `http://` prefix; an
  authority that is non-empty, carries no `@`, and runs to the first `/`, `?`, or `#`" —
  the grammar bounds the authority and says nothing about the bytes after it, while
  `candidates_url` is still specified over the raw string (`<base>/v1/candidates?slug=<encoded>`)
  and the request is `GET <path> HTTP/1.1`. The document itself now classifies
  `--catalog-url`/`BINDFORGE_CATALOG_URL` as "both untrusted config sources", so a CR or
  LF in the path portion of an admitted base — which the grammar accepts — goes verbatim
  into the request line and lets that value forge extra request lines or headers to
  whatever loopback service holds the port. Fix: one clause stating that everything after
  the authority in an admitted base is restricted to printable ASCII (no control bytes,
  no space) and that the request target is built from the parsed `Endpoint`, not the raw
  base, plus a `http://127.0.0.1:8080/x\r\n` rejection in Test strategy 9.
- [nit][re-raised] Match store — "the `bindforge/` directory and both files are created
  owner-only — 0700/0600 where unix modes apply — so the confirmed inventory is never a
  `umask` accident" — closes the half of r1-5 about modes this process sets; it leaves
  the case where `<data>` or `bindforge/` already exists and is group/world-writable (a
  shared `XDG_DATA_HOME`, which the path rule accepts because it only checks absoluteness)
  unaddressed, and the store is the trust root for records that are "re-trusted on every
  scan". Fix: one sentence beside the inventory-disclosure row in Risks accepting a
  user-configured shared data directory as residual, or tightening the mode on load.
- [nit][new] Match store, Save — "`matches.json.tmp` (opened exclusively, never through
  a pre-existing entry)" — the exclusive open adds a failure mode the design names no
  refusal for: a `matches.json.tmp` left by an interrupted run makes every later save
  fail, and nothing in the named-refusal set covers a failed save, so the run reports a
  confirmation it never persisted. Fix: name it — `error: match store unwritable:
  <path>`, stderr, non-zero exit, no record claimed — and pin the pre-existing-tmp case
  in Test strategy 7.

## FYI (no severity)

- `escape_and_truncate` caps at 80 characters (`crates/core/src/error.rs:30`), so the
  `Display Name`/`Install Path` cells are now truncated with `…` and Windows paths render
  with doubled backslashes; that changes the T-003-pinned table contract beyond the
  escaping this seat asked for — compatibility and correctness seats may want a word.
- The store record's `friendly_name` is the one field with no load-time check; it is
  bounded by the 65 536-octet file budget and never rendered or joined, so it is inert —
  the correctness seat may still want it pinned for symmetry with `slug`/`game_id`.
- Test strategy 14's rewording ("resolve nowhere in ... `src/` or in any assertion of any
  suite") and the `cli_slug_refusal_tests` fixture-route deletion in Rollout 3 are
  coverage questions, not security ones; the bound refusal itself stays in place at step 4.
