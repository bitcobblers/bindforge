# Code review — testing seat, round 1, dissent pass c2

- Run: `027-hosted-registry-with-store-resolvers`
- Seat: testing (checklist: `.opencode/skills/code-review/references/testing.md`)
- Scope as given: `git diff f52686fecdc794062dc72f6d71c58262d2029fb9..HEAD -- ':(exclude)docs/designs'`,
  read against the full current `02-proposal.md` and `docs/adrs/027-…`.
- Diff contents (the whole range): `apps/registry/tests/admin_auth.rs` (+38/-6),
  `apps/registry/tests/retired_ids_and_curation.rs` (+95/-1/-3), `apps/registry/tests/steam.rs` (+14).
  Three commits: `f6d24bca` (tests), plus two `docs:` commits inside the excluded `docs/designs/` path.
- Inputs used: that diff, `02-proposal.md`, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
  the runner's gate record, and this seat's r1 findings. No review report or transcript was opened.

## Gate evidence (checklist 6)

Runner record at `2026-11-22T11:13:49.574Z`, all four legs green (exit 0):
`npm run check`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`.
The only `test result:` line the record shows is `ok. 0 passed; 0 failed; 0 ignored`, which is one
binary's tail and says nothing about the three files this diff touched — it is not evidence that the
new legs ran.

Checks I executed myself (source is unchanged in `apps/registry/src` in this range, so the prebuilt
target is the code under review):

- `cargo test --locked -p bindforge-registry --test retired_ids_and_curation -- an_issued_delete_segment_answers_exactly_as_the_corpus_route_answers_it --exact`
  → `test result: ok. 1 passed; 0 failed; … 11 filtered out`.
- `cargo test --locked -p bindforge-registry --test admin_auth`
  → `test result: ok. 12 passed; 0 failed; 0 ignored`.

So the new and re-pinned legs are green and deterministic (ephemeral loopback ports, injected
fixtures, no wall-clock assertions, no outbound resolution — checklist 4 holds for the changed code).
I did not re-run the workspace suite or the npm leg; the supplied record covers them.

## Status of this seat's r1 findings

| r1 finding | Severity | c2 status |
|---|---|---|
| #6 transport-assurance fixture (`tests/steam.rs`) | major | **open** — one header section added; no test; the contradicting claims it was supposed to replace are all still in the file |
| mint-path octet cap unexercised | minor | open — unchanged code, unchanged coverage |
| `startup_snapshot` arms unpinned + docstring claim | minor | open — still zero call sites under `apps/registry/tests` |
| canonical external-id filter unpinned | minor | open — no suite queries `steam/+730` / `steam/00730` |
| four named refusals in `src/steam.rs` without legs | minor | open — `tests/steam.rs` gained only prose |
| retiring half mass-`#[ignore]`, three undeclared modules | minor | open — byte-identical to r1 |
| `mint_rule.rs` header contradicts `src/mint.rs` unit | nit | open |
| budget constants re-spelled as literals | nit | open |
| bare `proves: Test strategy #N` markers | nit | open |

The two legs this revision actually added (`admin_auth.rs` issued-route matrices,
`retired_ids_and_curation.rs:364` segment-rule leg) answer the issued-route coverage asks from other
seats: they are well-built, deterministic, and non-vacuous in their stated purpose (the route's
*posture*). They do not touch any of this seat's open items except #1, where the response was
documentation.

## Findings

### 1. major — Test strategy #6's transport-assurance clause still has no test, and the suite still claims it

Location: `apps/registry/tests/steam.rs:58-70` (new section) against `:3`, `:23`, `:1067-1072`,
`:1136-1143`; design `02-proposal.md:619-629`.

Problem. `#6` requires the production `StoreTransport` "driven against a loopback TLS listener
presenting a self-signed cert *for the name being verified* and one for another name; it refuses both
handshakes, so the first case fails on the chain, not the hostname. Without it, a verifier relaxed to
accept the fake passes the whole suite with the key on an unauthenticated link." The diff added no
test and no fixture: `apps/registry/Cargo.toml` `[dev-dependencies]` is still `serde_json` only (I
read it), `find` over the repo returns no `.pem`/`.crt`/`.der`/`.key` file, `grep` finds no `rustls`
or `Certificate` reference in any suite, and `src/steam.rs` has no `#[cfg(test)]` module. The shipped
legs at `:1074` and `:1151` still dial a plain `TcpListener` that presents nothing.

The r1 fix offered a documentation branch — "correct the suite's header and leg comments to pin only
what is actually asserted, and record #6's assurance clause as unmet". Only part of that landed. The
new `:58-70` section is accurate (I verified each of its claims) and does record the gap, but every
claim it was meant to correct is still on the page and still contradicts it:

- `:3` still lists "the production transport's assurance half" as one of the four things the suite
  covers; `:23` still closes the file's own `proves:` block with "…and its transport half".
- The leg doc at `:1067-1072` still claims the leg proves "Test strategy #6's transport-assurance
  clauses" and that the transport "refuses a loopback listener that *cannot chain to the pinned
  authority*" — the listener presents no chain to judge at all.
- The inner comment at `:1136-1138` still says "The first case is the chain, not the hostname: a
  self-signed listener for the very name being verified must still be refused", and the assert under
  it is still `assert!(!text.is_empty(), …)`, which no reachable failure can fail. A verifier
  relaxed to accept any chain, or `webpki-roots` swapped for a permissive anchor set, changes no
  assertion in this file and lands the key on an impersonated link.

Checklist 1 (a test citing each numbered strategy item's clauses) and checklist 3 (each previously
rejected major now has a test that would have caught it) are therefore both still unmet, and
checklist 7's discipline about claims is met only in the one direction.

Fix. Preferred: add `rustls` (no default features) plus either a checked-in self-signed cert/key pair
or a minting dev-dependency, and drive both named cases through `with_production_transport` against a
loopback TLS acceptor dialing `127.0.0.1` — asserting both handshakes refuse and that the
verified-name case's refusal names a certificate/chain condition, replacing the `!text.is_empty()`
stub. If the fixture stays out of scope for this run, finish the documentation branch instead: delete
"the production transport's assurance half" from `:3` and "and its transport half" from `:23`,
rewrite `:1067-1072` and `:1136-1143` to state that the fake presents no chain and that the
chain-vs-hostname discrimination is *not* pinned, and carry `#6`'s assurance clause in
`03-build-notes.md` as an outstanding item so the ledger does not read it as satisfied.

### 2. minor — the mint-path octet cap is still exercised by no test

Location: `apps/registry/src/state.rs:461-466` (`issued.to_bytes_with(&record)` measured against
`MAX_ISSUED_DOCUMENT_OCTETS`); `apps/registry/tests/issued_entries.rs:7-11` header claim, legs
`:350-396` (count cap) and `:510-555` (load-side metadata gate).

Problem. Unchanged in this range and still open. `grep MAX_ISSUED_DOCUMENT_OCTETS` shows the only
exercise is the load-side gate leg at `:510-555`. The suite header still promises "the octet cap
refuses the same way" (`issued_entries.rs:11`), and `#4` (`02-proposal.md:590-596`) names it: "at
`MAX_ISSUED_ENTRIES` a novel key is `503 mint cap reached` while a held key still answers; **the
octet cap refuses the same way**". The c1 refactor moved this measurement into the new
`Catalog::to_bytes_with` spliced buffer, so a wrong buffer size or `revision + 1` arithmetic in that
function lands entirely on this unexercised branch.

Fix. Add one leg beside `at_the_issued_cap…`: plant a document under the record cap whose
serialization sits within one record of `MAX_ISSUED_DOCUMENT_OCTETS` (fat 128-octet ids/names,
streamed the way the count-cap leg already streams), then assert the novel key gets the named `503`
and the file, model, and `revision` are untouched.

### 3. minor — `startup_snapshot`'s three arms are unpinned and its docstring still promises tests

Location: `apps/registry/src/steam.rs:426-429` (docstring) and `:437-469`; zero call sites under
`apps/registry/tests` (`grep -rn startup_snapshot apps/registry` → `main.rs:349,362` and
`steam.rs:164,437` only).

Problem. Unchanged in this range. The `027-C3` startup decision — load only present + loadable +
fresh; stale/absent/unloadable → full pull; failed pull → the key-free named refusal
(`02-proposal.md:269-272`) — was moved to the library face on the stated rationale that "a test can
point a `SteamStore` at a loopback fake and watch which arm the decision takes". No test does, and
the stale → pulls-in-full arm is the posture change that revision made. Meanwhile
`hosted_lookup_policy.rs`, `health_route.rs` and `cli_end_to_end.rs` each justified relocating their
stale legs by citing this startup rule.

Fix. Three short in-process legs over `startup_snapshot(&SteamStore::for_test(fake, KEY), path, now)`:
a fresh plant returns the loaded snapshot with the fake unqueried; a stale plant pulls in full and
publishes; an absent plant plus a failing fake yields the key-free named refusal. Otherwise delete
the docstring's test claim and record the arms as unpinned.

### 4. minor — the canonical external-id filter ships with zero citations

Location: `apps/registry/src/state.rs:369-387` (`.filter(|appid| appid.to_string() == external_id)`).

Problem. Unchanged in this range. `steam/+730` and `steam/00730` now answer `200 []` on a fresh
snapshot instead of parsing to appid 730, and the comment grounds that in the "appid-unique" property
the mint's held-set probes and `027-B3`'s cap arithmetic stand on. The only non-decimal cell in the
suite is `hosted_lookup_policy.rs:309-335`, which queries `steam/not-an-appid` — the alphabetic case
that `u64::from_str` already rejected before this filter existed. The double-identity hazard the
guard exists to close is unpinned.

Fix. One leg over the in-process `policy()` helper already in `hosted_lookup_policy.rs`:
`state.candidates("steam/+9100801", now)` and `"steam/009100801"` answer `200 []` with provenance
`none` while the canonical spelling answers the snapshot's record.

### 5. minor — four named refusals added in c1 still have no legs

Location: `apps/registry/src/steam.rs:620-625` (full pull of an empty worldlist), `:631-637`
(a continuation page carrying no records that then claims finished), `:340-345` (`load_snapshot`
refuses `"apps": []`), `:707-714` (any non-200 status, not just non-2xx);
`apps/registry/tests/steam.rs:643-693` unchanged.

Problem. `tests/steam.rs` gained only prose in this range. `every_unhealthy_store_answer_fails_the_pull_by_name`
still sends `500`, `429`, `302`, `Transfer-Encoding`, non-JSON, wrong envelope, no-`apps`-list and a
non-decimal `appid` — no `204`, never an ok envelope with zero records, and no continuation page that
comes back empty. No `#2` loader leg plants an `"apps": []` file (`grep` for an empty apps array in
the suite returns nothing). Each rule is production-visible — a key without the `IStoreService`
grant, a lying pagination, a hand-edited snapshot — and each can now fail a whole pull or a boot,
asserted only by its own comment.

Fix. Add rows to `every_unhealthy_store_answer_fails_the_pull_by_name` (`204`; empty-envelope full
pull; empty continuation after `have_more_results: true`, plus the delta no-change first page that
must still publish) and one `#2` loader row for an empty-apps snapshot file.

### 6. minor — the retiring half of ADR-027 is still mass-`#[ignore]`, not deleted

Location: `apps/registry/tests/{cache_document.rs,caching_policy.rs,upstream_client.rs,upstream_endpoint_gate.rs}`
(27 `#[ignore]`d legs across the four retiring suites — cache_document 7, caching_policy 11,
upstream_client 6, upstream_endpoint_gate 3 — plus the two single-leg retires at
`admin_mutations.rs:598` and `candidates_join.rs:331`); `apps/registry/src/{cache.rs,upstream.rs,endpoint.rs}`,
declared by neither `main.rs:24-35` nor `lib.rs:28-35`.

Problem. Byte-identical to r1. `grep '#\[ignore'` still shows 29 ignored legs, and the four reason
strings still end "suite retired in place pending the runner's deletion". The three retired modules
still sit in `src/` undeclared, so they are not compiled by any target: a green
`cargo test --workspace --locked` says nothing about them, and the ledger's "retired" quietly means
"ignored" in the run record. I still cannot verify from this pass's allowed inputs whether
`03-build-notes.md` records the deletion as an outstanding runner action — the caller's input list
excludes it.

Fix. The runner deletes the three modules and the four suites as the recorded completion of
Rollout 2/3, keeping the two single-leg retires ignored-with-reason. If deletion cannot happen in
this run, ensure it is recorded as an outstanding runner action in `03-build-notes.md`.

### 7. nit — `mint_rule.rs`'s header still contradicts the in-crate unit that pins the last rung

Location: `apps/registry/tests/mint_rule.rs:15-23` vs `apps/registry/src/mint.rs:316-330`;
`apps/registry/src/lib.rs:31` (`pub mod mint`).

Problem. Unchanged. The header still asserts that `mint identity over bound` is "**not** pinned here
or anywhere else in this package" and that the rung "needs a mint seam of its own (`mint.rs::mint_id`)
to be tested at all", while `an_appid_spelling_over_bound_is_named_not_truncated` at `mint.rs:316`
does exactly that (200-digit external id → `IdentityOverBound`, held-plain and over-bound suffix the
same) and carries no `proves:` marker. `#1`'s last rung is traceable only through an unmarked test
the header contradicts.

Fix. Correct the header to cite the `src/mint.rs` unit, and mark that unit
`// proves: design 027's Test strategy #1, last rung` (or move the two assertions into
`tests/mint_rule.rs` over the published face).

### 8. nit — the budget family is still re-spelled as literals in the suites

Location: `apps/registry/tests/steam.rs:82-98`; `tests/issued_entries.rs:37-38`;
`tests/retired_ids_and_curation.rs:35-36` vs `apps/registry/src/constants.rs:98,104,114,121,130,133`
(re-exported through `lib.rs:28`).

Problem. Unchanged. c1 centralized the store-resolver budget family in `constants.rs` and published
it, which is the import surface r1 asked for, but every suite still re-declares its own copies, so a
budget change moves one side only and surfaces as a confusing fixture assertion.

Fix. Import the published constants where the crate exports them; keep a local literal only where a
suite deliberately pins the design's number, marked as such on the spot.

### 9. nit — inline markers still share one numbering space across designs

Location: `apps/registry/tests/response_shape.rs:221`; `upstream_client.rs:59,165,204,252`;
`upstream_endpoint_gate.rs:48,79,138`; and the rest of the 136 `// proves: Test strategy #N` markers
that sit in the bare form across the suite set.

Problem. Unchanged. A grep for `Test strategy #16` still mixes a design-026 pin with design 027's
platform item; only file headers disambiguate.

Fix. Prefix the markers in the re-pinned legacy suites with their design (`proves: design 026's Test
strategy #16 …`), matching the headers' style.

### 10. nit — the new segment leg pins equality only, and mis-attributes two of its rows

Location: `apps/registry/tests/retired_ids_and_curation.rs:349-436` (new in `f6d24bca`);
`apps/registry/src/admin.rs:109-135` and `:288-321`; `apps/registry/tests/slug_decoding.rs:219-254`.

Problem. The leg asserts, per row, `status_code(issued) == status_code(corpus)` and
`!matches!(code, Some(200..=299))` — nothing names the expected code. Because both handlers share the
same `game_id_is_valid` → `422` / `Absent` → `404` shape, a single shared edit (drop the grammar
check from both, or route both to `NotARoute`) turns every row into `404 == 404` and the leg stays
green while the clause it cites — "`validate[s] the `game_id` grammar (A10)`"
(`02-proposal.md:487-489`) — stops being enforced anywhere. The corpus side's twin at
`slug_decoding.rs:243` is no firmer (it asserts only "not `200`/`204`"), so no leg in the suite names
the grammar refusal. Relatedly, two of the ten rows do not test the grammar at all: for
`"../../etc/passwd"` and `"steam/9455001"` the encoded `/` makes the decoded segment contain a `/`,
so `admin::dispatch`'s `!segment.contains('/')` guard answers `404` before `is_slug_name` is ever
consulted — the docstring's "outside A10's grammar … in every spelling the wire can carry (dot-dot,
an encoded separator, …)" credits those rows to the wrong rule.

Fix. Attach the expected status to each label (`422` for uppercase/underscore/dot/leading-digit/
control-byte/over-bound, `404` for the two slash spellings and the empty segment, `404` for
`grammar-valid-but-unheld`) alongside the equality assert, and move the slash rows' labels to name
the route matcher rather than the grammar.

## Checklist verdict

1. Numbered strategy items cited — partially met: every item has citing legs, but `#6`'s
   transport-assurance sub-clause (finding 1), `#4`'s octet-cap clause (2), `#3`'s canonical-spelling
   cell (4) and `#1`'s last rung (7, by contradiction) are not covered by what cites them.
2. Unit + integration layers where promised — met, subject to findings 3 and 5.
3. Previously rejected majors now tested — **unmet**: finding 1.
4. Determinism — met (see gate section; both re-runs green and fast; loopback fakes, injected clocks).
5. Names describe behavior — met; the two new legs are named for the posture they pin.
6. Gate evidence — green as supplied; two targeted re-runs by me, both green; the supplied
   `test result: ok. 0 passed` tail is not evidence about the changed files.
7. Untestable/unexercised additions flagged — met as findings 1, 2, 3, 4, 5, 6, 10.

Verdict: **request-changes** — one open major (finding 1), five minors, four nits.
