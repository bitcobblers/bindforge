# Code review — `compatibility` seat, round 1 · pass c2 (ratification)

**Inputs (exactly).** `git diff 06d528febd4bd7d8b875553668b4ebab6a3bd097..HEAD --
':(exclude)docs/designs'` (20 files, +1,777 / −155), `docs/designs/026-registry-service/02-proposal.md`,
`docs/adrs/026-registry-service.md`, the runner's gate record (green at
2026-11-19T23:01:03Z: `npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`,
all exit 0), and this seat's round-1 findings. No review report or transcript opened.

**Pass duty.** Ratification only: confirm the revisions that landed after this seat's
approval did not break what it approved. New-code findings are raised only where the
changed code itself introduces them.

**Verdict: approve.** Zero blocker, zero major. Four round-1 findings are unchanged by
this increment and carry forward (2 minor + 1 minor + 1 nit); one new nit comes from the
increment itself.

## 1. What the increment changed, read for compatibility

| Change | Compatibility reading |
|---|---|
| `keys.rs:82-97` — the key rule now **calls** `Source::spells_lookup_key` instead of reconstructing it | Verified equivalent to the rule approved in r1. The shipped predicate (`crates/discovery/src/game.rs:84-89`) is exactly "token in the closed set + non-empty external id + one `/` split"; the registry's extra path-safety refusals (`keys.rs:91-97`) are untouched, and the empty-`external-id` refusal moved *into* the shipped call rather than being dropped — `keys.rs:122-143` still refuses `steam/`. The widened alphabet this seat pinned in r1 is still admitted: `keys.rs:107-120` pins `local/Steam.SE`, `local/100%`, `local/two words`, on-cap 128. No drift against §The serving face step 1 or Test strategy #3. |
| `corpus.rs:84-110` + `serve.rs:288-302` — the answer seam trims to `MAX_RESPONSE_OCTETS` as well as `MAX_CANDIDATES` | `026-C1`'s byte-identical face is preserved because there is still exactly **one** body builder (`candidates_body`) and it is the only one `candidates_answer` uses — so root and caching answers trim identically. The arithmetic counts brackets and `,` separators, i.e. precisely what `serde_json::to_vec` writes, so `Content-Length` equals the bytes emitted and the body lands at ≤ 65,536 (inclusive, the boundary A1's reader accepts). A single projected candidate is bounded (128-octet `game_id` unescaped, 128-octet `name` escaping to ≤ 770), so the trim never empties a non-empty join: `200 []` stays the root empty join and the `503` stays the caching-only cold case (`026-L4`). `candidates_join.rs:184-273` and `wire.rs:604-655` pin stability and the `Content-Length` match. |
| `cache.rs:141-151`, `cache.rs:244-250` — per-entry candidate cap at the write seam, load refusal over the cap | Inside `026-C7`'s "refused by name outside its shape, **bounds**". No self-poisoning: the only writer of an entry is `Cache::stage`, which truncates, so the load refusal can only fire on a hand-edited or foreign document — the whole-cache refusal the proposal rejected as the *only* mechanism is not what this adds. |
| `corpus.rs:32-35`, `218-227`, `396-406` + `schemas/catalog/v1/spec.md:53-59` — closed entry set at both levels, one new invalid fixture | The spec already shipped the sentence "the entry set is closed" in r1; the increment makes the loader meet it instead of widening the contract. Refusal lines interpolate the offending entry name through `escape_and_truncate`, so a non-ASCII or control-byte entry name stays a single named line. The new fixture name is lowercase ASCII dash-separated, so core's case-unique fixture rule (`#19`) still governs it on a case-insensitive filesystem, and `tests/catalog_document.rs:125-150` walks it into a one-line startup refusal. |
| `http.rs:170-...`, `http.rs:370-399`, `constants.rs:36-52` — header-only admin gate plus bounded discard of a refused request's declared body | Framing-neutral for every client. |
| `http.rs:328-368` — header-block terminator test rewritten from a full rescan to a tail test | Equivalent, not a framing change: the loop reads one octet per iteration (`read(&mut byte)` with `[0u8; 1]`, `http.rs:330-348`), so the only terminator that can be new is the one ending at the octet just pushed. `MAX_REQUEST_HEADER_OCTETS` still bounds the block. |
| `serve.rs:212-266` (`admin_gate`, `media_type_spells_json`), `admin.rs:10-35` — flat `404` for a disabled face, `Origin` → `400`, non-JSON declared `Content-Type` → `415` on the upsert route only | Both modes share `admin_gate`, so no mode divergence is introduced. These are HTTP-surface additions outside this seat's checklist — see FYI. |
| `persist.rs:212-225` + `persist.rs:6-11` + `spec.md:148-157` — `sync_all` before the rename | Portable `std`: `File::sync_all` is implemented on the matrix's Windows row (`FlushFileBuffers`) as on Unix, so no new `cfg` guard is needed and none was added. The deliberately *un*synced parent directory is named with its platform reason, and the failure direction it leaves is the safe one. |
| `state.rs:180`, `corpus.rs:165-197` — a root's `--corpus` must exist | Documented in `spec.md` §Placement. No shipped consumer or harness leg breaks: every `Registry::root` caller writes the corpus first, and the invalid-fixture sweep copies the file before starting a root. `absent-is-empty` still belongs to the caching instance's two documents (`026-C7`), as the proposal states. |
| `main.rs:129-197` — `--flag=value` | Only `--`-prefixed tokens are split, so a positional slug holding `=` stays a slug (`main.rs:429-440`); no platform-specific argument rewriting (no `@file`, no glob expansion) is introduced, so Windows/Unix argv handling is unaffected. |
| `caching_policy.rs:146-410`, `cli_end_to_end.rs:512-556`, `admin_auth.rs`, `admin_mutations.rs`, `persist_and_concurrency.rs` | Test-side. The reshaped TTL legs stay on the existing `Registry::caching` seam — no *new* caching-mode subprocess suite was added, so this seat's round-1 harness finding did not grow. |
| `data/catalog/README.md`, `spec.md` ledger/annotation wording | Ledger and docs surface; not this seat. |

## 2. Checklist (`references/compatibility.md`)

1. **Platform code vs the Compatibility matrix** — *pass with carried findings.* The
   increment adds no platform-specific code: `sync_all`, `set_read_timeout`, chunked
   reads and `ends_with` are portable `std`, and nothing new sits behind `cfg(unix)`.
   The Windows row still has its code path (`DataLayout::Windows`, `persist.rs:64-70`)
   and its in-process unit coverage (`persist.rs:330-345`); the two Windows-facing gaps
   this seat reported in r1 (`main.rs` refusal wording, the caching harness's
   `XDG_DATA_HOME`-only relocation) are unchanged — findings C-1, C-3.
2. **Node floor** — *not applicable.* No JS/Node path in the increment
   (`tools/orchestration` untouched); the only Node leg in the gate record is
   `npm run check`, green.
3. **Paths** — *pass.* The increment builds no new path at all: documents still resolve
   through `data_dir`/`document_path` joins, and `--corpus` remains a `PathBuf::from`
   of the operator's argument. No new case-sensitivity assumption; the new fixture
   filename satisfies the case-uniqueness rule that stands in for a case-insensitive
   filesystem (`#19`).
4. **Guarded platform imports** — *not applicable / pass.* No platform-only module is
   imported or added; the crate's `winreg` transitives come from the `discovery` edge
   the proposal already names.
5. **Locale / layout reality** — *unchanged from r1.* Non-ASCII lookup keys and display
   names still have no fixture and no suite row (carried nit C-4). Nothing in the
   increment makes survival worse: the octet trim counts `serde_json`'s UTF-8 output in
   octets and emits the same bytes, so a multi-byte `name` costs its real octets and is
   never split at a non-boundary; the closed-set refusal escapes the arbitrary key name
   it quotes.
6. **Native dependencies** — *pass.* No `Cargo.toml`, build script or dependency change
   anywhere in the diff.
7. **Tests on the platforms they claim** — *pass with one new nit.* Linux is exercised by
   the green cargo gates; Windows/macOS still ride the env-closure seam and case-unique
   fixture names, and no platform-gated test was newly skipped everywhere. New nit C-5
   covers the increment's one platform-portable-idiom test.

## 3. Gate record as evidence

All four legs green at the captured gate, including `cargo test --workspace --locked`
and `cargo clippy --workspace --all-targets -- -D warnings`; the increment's unit and
integration legs are therefore running on the Linux primary target. Nothing in the
increment moves a claim onto a platform the matrix does not mark `Yes`.

## 4. Findings

### Carried from round 1 — code unchanged by this increment (still open, none blocking)

**C-1 · minor · `apps/registry/src/main.rs:222-228` (test at `main.rs:440-452`).**
The missing-`<data>` startup refusal still hard-codes "set `XDG_DATA_HOME` to an
absolute path" although the layout is resolved from `DataLayout::host()`; on the
Windows branch — a `Yes` row of the matrix, `<data>` = absolute `%APPDATA%` — the
message names a variable the platform rule never reads, and the unit test pins the
Unix-only wording unconditionally. *Fix:* branch the refusal on the resolved layout
(name `%APPDATA%` on the Windows branch, or name both spellings) and assert the wording
that matches `DataLayout::host()`.

**C-2 · minor · `apps/registry/src/upstream.rs:72-73`, `102-105`.**
`authority_of` strips the brackets from an IPv6 host and `fetch` rebuilds
`format!("{host}:{port}")`, so an admitted bracketed IPv6 upstream sends an unbracketed
`Host: ::1:8080`, not the bracketed form RFC 9110 requires. It works only against this
registry's own Host-blind reader; a standards-conforming front (proxy / vhost edge) can
reject or misroute it. *Fix:* bracket when the host contains `:`.

**C-3 · minor · `apps/registry/tests/common/mod.rs:462-476` with `671-677`.**
`Registry::caching` relocates an instance through `XDG_DATA_HOME` alone while
`Registry::start` scrubs `XDG_DATA_HOME`/`HOME`/`APPDATA` from every child, so on a
Windows host the child resolves `<data>` from the scrubbed `APPDATA`, finds none and
refuses to start — every caching-mode subprocess suite fails on a `Yes` row. Test
strategy #13 and the harness header (`common/mod.rs:13-15`) both name `APPDATA` as part
of this same seam; Linux-only CI hides it. This increment reshaped legs on this seam but
added no new caching suite, so the exposure is unchanged. *Fix:* set `XDG_DATA_HOME`,
`HOME` and `APPDATA` all to the sandbox directory in `Registry::caching`.

**C-4 · nit · `schemas/catalog/v1/fixtures/valid/`, `apps/registry/tests/slug_decoding.rs:124-147`.**
Fixtures and suite keys are pure ASCII: the widened alphabet is pinned for dots, `%`
and spaces, but survival of a non-ASCII lookup key or display name (byte-wise
percent-encode, octet-counted caps, UTF-8-validated decode) is structurally true and
never asserted — including now for the octet-capped answer window, which is
byte-arithmetic. *Fix:* one valid fixture with a non-ASCII external id plus a
candidate-query row that round-trips it.

### New in this increment

**C-5 · nit · `apps/registry/tests/cli_end_to_end.rs:512-556` (helper at
`apps/registry/tests/common/mod.rs:1048-1054`).**
The new explicit-`--port` leg takes its port from `closed_loopback_port()`, a helper
whose stated contract is "the *no reachable upstream* of `026-L4`": it binds a loopback
listener, reads the assigned port and **drops** it. Used as a port to re-bind, the value
is only free until something else takes it: with the cargo test binary's parallel
threads, a concurrent `FakeUpstream`/`SteamRoot` listener (`bind("127.0.0.1:0")`) can be
handed the just-learned port before the child's `bind`, and the leg then dies at startup
(`common/mod.rs:512-517` panics on the child exiting before printing a port). Linux's
rotating ephemeral-port allocator makes that coincidence rare — which is why this is a
nit and why the gate is green — but the reuse of a recently closed port is the documented
behavior on the matrix's Windows row, where this leg is claimed supported and never run.
*Fix:* don't borrow the "unreachable" helper for a bind: hold the probe listener until
immediately before `Registry::start`, and have the leg retry the child start once with a
freshly learned port if the bind is taken; or drive the leg from a per-test port in a
range no `bind(:0)` in this binary can be handed.

## 5. FYI (out of specialty, no severity)

- `serve.rs:249-266` adds `415` for an upsert whose declared `Content-Type` is not
  `application/json`, and `serve.rs:237-247` adds `400` for any admin request carrying
  `Origin`. Neither status appears in `026-C4`/`026-C5` or the proposal's named-refusal
  set. No shipped consumer breaks (nothing in the tree POSTs to the admin face, and the
  header stays optional), but the practical operator edge is that `curl -d '{...}'`
  — whose default declaration is `application/x-www-form-urlencoded` — now gets `415`
  unless it adds `-H 'Content-Type: application/json'`, which is worth a line wherever
  the admin face is documented.
- `serve.rs:298-301` states that "`query` prints the same projection it answers with".
  It does not: `main.rs:305-307` prints `state.candidates(...)` whole (up to
  `MAX_CANDIDATES`), not `response_bounded`'s prefix, so for a wide join `registry query`
  can list more candidates than `GET /v1/candidates` answers. The comment is the defect,
  or the seam is; either way the two surfaces of one policy should agree with
  `spec.md` §The answer's budget.
