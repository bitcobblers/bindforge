# Code review — correctness seat — design 026 (round 1, dissent pass c1)

**Verdict: approve** (0 blocker, 0 major, 2 minor, 3 nit)

## Scope and inputs

Diff under review: `git diff 06d528febd4bd7d8b875553668b4ebab6a3bd097..HEAD -- ':(exclude)docs/designs'`
— 20 files, +1,777/−155: `apps/registry/src/{admin,cache,constants,corpus,http,keys,main,persist,serve,state,wire}.rs`,
`apps/registry/tests/{admin_auth,admin_mutations,caching_policy,candidates_join,cli_end_to_end,persist_and_concurrency}.rs`,
`data/catalog/README.md`, `schemas/catalog/v1/spec.md`, and one new fixture
`schemas/catalog/v1/fixtures/invalid/record-entry-outside-closed-set.json`.

Inputs, exactly as the caller set them: that diff, `docs/designs/026-registry-service/02-proposal.md`
(full current document), `docs/adrs/026-registry-service.md`, the runner's gate record, and my own
round-1 findings as quoted by the caller. No review report or transcript was opened. `03-build-notes.md`
is outside this pass's inputs, so build-notes claims could not be verified first-hand; where a fix depends
on that document I say so rather than assume either way. Gate commands were taken from the runner's record
(all green, including `cargo test --workspace --locked`); I additionally read the code the gate ran.

This pass checked my round-1 findings first, as instructed, and reports other matters only where this
round's changed code touches them.

## Round-1 findings — disposition

### 1. major — answer capped by count but not by octets — **fixed**

- `corpus.rs:84-100` `response_bounded` takes the largest prefix of the `game_id`-ordered postings that
  fits `MAX_RESPONSE_OCTETS`, inside the same `take(MAX_CANDIDATES)` window the count cap uses;
  `corpus.rs:102-110` `candidates_body` serializes exactly that prefix, and `serve.rs:294-301` is now the
  only place the `GET /v1/candidates` body is built. The arithmetic is exact, not estimated:
  `serde_json::to_vec` on a `Value::Array` is compact, so `2 + Σ(object + separator)` is the body length,
  and the cap test `total + cost > MAX` cannot overshoot.
- I verified the "never empties a non-empty join" claim rather than trusting the comment: a candidate is
  `{"game_id":…,"name":…}` with `game_id` grammar-restricted (`corpus.rs:game_id_is_valid`) and `name ≤ 128`
  octets (`013-B4`, enforced at load, at the admin gate, at cache load and at upstream ingest), so worst-case
  escaping is ≈2·128 + 24 ≈ 280–408 octets — three orders of magnitude below 65,536, so `kept ≥ 1` whenever
  the join is non-empty. No path can now answer `200 []` by trimming.
- The same budget is carried at the cache seams the finding named: `cache.rs:141-152` refuses a stored entry
  over `MAX_CANDIDATES` at load and `cache.rs:244-250` caps the staged list at the write seam; `Catalog::to_bytes`
  (`corpus.rs:339-357`) emits only `version`/`revision`/`games` and three-key records, so the closed-set gate
  added this round cannot make the instance refuse its own writes.
- Pinned three ways, and at the seam that matters: `corpus.rs` unit test (asserts the unbounded body really
  exceeds the cap before asserting the bounded one does not), `wire.rs:605-655` (one `Content-Length`, equal to
  the bytes sent, `kept < MAX_CANDIDATES`, stable across requests), and `tests/candidates_join.rs:184-273`
  (256 records at both field bounds over the wire: `200`, body ≤ 65,536, `Content-Length` equal to the body,
  served window is the `game_id`-ordered prefix, identical bytes on the second request).
- Contract side closed too: `schemas/catalog/v1/spec.md:110-133` ("The answer's budget") states the cap in both
  directions, the ≈283-octet-per-candidate arithmetic, the ~231-candidate crossover, and that the trim is a
  shorter answer and never the `026-L4` `503`. My fix asked for the code bound **or** the spec record; both landed.

### 2. major — 026-L1 / seed shipped without the ledger annotations Rollout 6 says this run ships — **partly fixed; re-opened at minor**

- Fixed half: `data/catalog/README.md:12-22` no longer asserts that `026-S1` carries a partially-satisfied line.
  It now states the unit's wording is ahead of the file, names the annotation as owed to design 026's Rollout
  step 6, and points at the run's build notes for the record. That was the concrete falsehood in the shipped tree.
- Unfixed half: `docs/adrs/` is untouched anywhere in the branch (`git diff --stat main...HEAD -- docs/adrs/ docs/tasks/`
  is empty), and `grep -n 026 docs/adrs/025-discovery-identity-and-catalog-matching.md` still returns only the
  pre-existing ADR-026 reservations (lines 147, 184, 246) — no `amended-by 026-L1` on `025-C4`/`025-L1`, no
  "filled by ADR-026" on `025-R1`, and no partially-satisfied line on `026-S1` (`docs/adrs/026-registry-service.md:207-211`),
  while the seed that ships in this branch mints provisional ids. ADR-026's own Rollout item 6 (`:241-244`) and the
  proposal's step 6 (`02-proposal.md:598-602`) make these this run's deliverables.
- I am correcting my own round-1 premise: `grep loopback` over ADR-025 returns **no** matches, so
  `025-C4`/`025-L1` never stated a loopback-only endpoint policy in words. The live staleness is narrower than I
  wrote: `025-C4` (`:96-101`) still says "The server-side contract is `025-R1`" and ADR-025's Rollout item 6
  (`:182-185`) still records the lookup contract as "the client-side provisional spelling until the reserved
  ADR-026 states the service contract", which this branch's registry and CLI amendment overtake.
- Why minor, not blocking: nothing in the code is wrong, the deviation is declared in a file this branch ships,
  the remedy is a `docs/adrs/` edit that the runner's stage-ownership rule may put outside this stage's write
  rights, and the caller's pass scope is source and test paths. It still needs landing somewhere.

### 3. minor — spec's "entry set is closed" vs a loader that ignored extra entries — **fixed**

- `corpus.rs:31-36` names the two closed sets; `corpus.rs:215-227` refuses a fourth top-level entry and
  `corpus.rs:396-409` refuses a fourth record entry, both named, escaped, single-line. `record_from_value` is
  the gate the admin upsert shares, so `POST /v1/admin/games` with a `gameId` typo is refused as the record it
  is not — the sentence in `spec.md:56-60` is now true at both levels, and the refusal table row
  (`spec.md:79`) names the new fixture and says the document-level twin is refused by the same gate.
- `schemas/catalog/v1/fixtures/invalid/record-entry-outside-closed-set.json` covers the record level; the
  document level is pinned in-suite (`corpus.rs:553-574`, `games_extra` and `gameId`, with the one-line-refusal
  assertion). That division is stated rather than silent, which is what the table row is for.

### 4. minor — a root with a nonexistent `--corpus` answered `200 []` — **fixed**

- `corpus.rs:165-196` splits the read: `load` keeps absent-is-empty, `load_required` names the refusal;
  `state.rs:170-181` puts `load_required` behind `State::root` (which is also what `registry query` in root mode
  builds) while `state.rs:210` keeps absent-is-empty for the caching instance's local-entries document — the
  asymmetry the contract actually states. Pinned by `wire.rs:826-845` (single named line) and stated at
  `spec.md:138-142`.

### 5. minor — disabled admin face answered 405+Allow for non-carried methods — **fixed**

- `serve.rs:212-224`: `admin_gate` consults enablement *first* and returns the one flat `404` for every dispatch
  outcome; `405`/`Allow` survives for an enabled face (`serve.rs:225-235`), and `http.rs:428-432` writes the list
  when there is one to write. Pinned at all three levels, and the subprocess leg
  (`tests/admin_auth.rs:228-277`) asserts the status line is *byte-equal* to an unknown route's and that no answer
  carries `Allow` — exactly the "one flat 404, publishing nothing" property, and the non-carried-method case
  (`PUT /v1/admin/games`) that I found routed around is now in the matrix. `admin.rs:22-26` states the rule as a
  decision. The routing/credential ordering for an *enabled* face is unchanged, so `026-C4`'s `405`+`Allow` rule
  and the framing refusals of Test-strategy #11 (`tests/persist_and_concurrency.rs:393-478`, loopback-open admin)
  still run on the paths they ran on before.

### 6. minor — the run's own scope guard says no product code; the branch ships it — **not addressed; held at minor**

Unchanged by this round and unaddressable by it: the proposal still reads "This run ships the contract and the
ledger annotations and **no product code**: the binary, schema, and CLI amendment land as tasks `T-004`…`T-008`"
(`02-proposal.md:13-14`, repeated as the code-review scope guard at `:30-32`, in Rollout 7 at `:603-604`, and in
ADR-026 at `:245-248`); `git diff --stat main...HEAD` shows the whole `apps/registry` crate and the
`apps/discovery-cli` amendment in this branch and nothing under `docs/tasks/` (only `T-001`…`T-003` exist), so the
"contract ahead of code" window that §Rollout step 5 (`:592-597`) and the Risks row (`:454`) present as open and
accepted is in fact closed here. I deliberately did **not** escalate to blocking as my round-1 fix contemplated:
`03-build-notes.md` is outside this pass's inputs, so I cannot establish that the deviation is undeclared, and
several of this round's tests cite a build-notes *Deviations* section by name
(`tests/admin_mutations.rs:191-199`, `tests/candidates_join.rs:184-191`). What I cannot establish is a record
wording, not a required behavior of the code, so it does not block proceeding. Whoever holds the run record should
make the Rollout text and the task ledger say what happened.

### 7. nit — parameterized path answers 404 where the collection path answers 405 — **fixed as a stated decision**

`admin.rs:13-26` now spells both asymmetries (parameterized paths are method-specific; a disabled face is flatly
`404`) as decisions, with the reason for each — a `405` on `/v1/admin/games/<seg>` would publish which segments the
face parses as `game_id`s to an unadmitted caller — and `dispatch`'s doc comment (`admin.rs:82-86`) points at it.
Pinned in `serve.rs:449-456` (`PUT /v1/admin/games/example-game` is the named `404`). The rationale is better than
the `405` I offered.

### 8. nit — extra modules and `keys.rs` reconstructing the lookup-key rule — **half fixed**

`keys.rs:79-90` now calls `Source::spells_lookup_key` and keeps only the path-safety half after it; I checked the
shipped predicate (`crates/discovery/src/game.rs:84-89`: `split_once('/')`, non-empty id, closed token set) against
the code it replaces and the two are equivalent, with the same separator. The extra modules are still extra — see
finding N1.

## New findings from this round's code

**N1 (nit) — four modules beyond the proposal's "concrete file plan".** `02-proposal.md:123-143` names nine
modules; the crate ships `keys.rs`, `persist.rs`, `state.rs`, and `wire.rs`, the last a `#[cfg(test)]`-only
~830-line in-process wire suite inside `src/` (it grew by 242 lines this round). Behavior-equivalent, and the
splitting is good engineering, but the file plan is the artifact a test engineer reads. Either record the four
beside the module list (or in the deviation record) or move `wire.rs` under `tests/`.

**N2 (nit) — new refusals, budgets and CLI syntax that the design text never names.** This round adds
`400` for a carried admin request that carries `Origin` (`serve.rs:236-247`) and `415` for an upsert whose
declared media type is not JSON (`serve.rs:253-275`, `media_type_spells_json` at `:273-279`), two private
budgets `UNREAD_BODY_DRAIN_OCTETS`/`UNREAD_BODY_DRAIN_TIMEOUT` (`constants.rs:41-53`), and the `--flag=value`
CLI form with the credential-echo rule behind it (`main.rs:129-160,181-190`). They are coherent, they are
documented at the code's own surface (`admin.rs:27-40`), and they are pinned at three levels; as a group they
close the browser-forgery and credential-echo exposures on a face the shipped client never calls, and the tests
describe them as an accepted round-1 security closure. The residual issue is the design text: §The admin face and
auth (`02-proposal.md:277-326`) still lists only `404`/`405`/`401`/`422` and §Budgets (`:400-418`) names three
request budgets, so the next reader of the design cannot see a `415` or a drain bound. No code change asked for;
make the deviation record (or the registry's own route documentation) name them, the same way
`schemas/catalog/v1/spec.md` was amended for the response bound in finding 1.

**N3 (nit) — a doc comment this round made false.** `persist.rs:235-237` still says the `None` a capped read
returns is what "`026-C7` and `026-C3` **both** read as an empty document". After `corpus.rs:165-196` that is true
of the cache and the local-entries document only: for `--corpus`, absent is now the named startup refusal. The
helper's two callers are `read_capped` and the cache, so nothing behaves wrongly; the sentence is the last place
the old rule is asserted. Same drift, one line, in the module doc at `corpus.rs:11-16`.

## Correctness checklist walk

1. **Components/interfaces named in the proposal exist and match.** Yes for every named item I could reach from
   this diff: the two-mode CLI surface, `GET /v1/candidates` / `/v1/health`, the three admin routes,
   `record_from_value` shared by load and upsert, `Source::spells_lookup_key` reuse (now a direct call, finding 8),
   the copied persist discipline (now with `sync_all` before the rename, `persist.rs:196-233`, in the documented
   write → flush → sync → chmod → rename order). The module-plan gap is N1.
2. **Assumption register honored.** A1 is the row this round was about, and both of its caps are now enforced
   outbound as well as inbound — the register is not silently re-read, it is applied in the direction A1 implies.
   A4/A5 bounds are what make the trim non-empty; A3/A16/A19 unchanged; A7/A22 extended (not weakened) by
   `sync_all`; A23's one-request-per-connection still holds with the head/body split. No row was silently changed.
3. **Deviations declared.** Not verifiable in this pass (`03-build-notes.md` is out of scope); N2/N1 and finding 6
   are the deviations I can see from the branch, and two of them are cited as build-notes records by the tests
   themselves. No undeclared deviation is *established*, so nothing is blocking on this item.
4. **Scope creep.** The branch-level scope fact is finding 6; within this round, the additions are hardening and
   budget closures against findings the panel accepted, with N1/N2 as the residue.
5. **Build-notes claims true.** Not checked — out of this pass's inputs (recorded here as the missing evidence).
6. **Error paths exist and fail closed.** Yes, and this round moved two of them earlier without weakening them:
   an admin request the header block condemns is refused before a body octet is buffered, with the declared body
   discarded rather than read (`http.rs:370-400`, `serve.rs:176-207`), and both refusals I checked keep the
   fail-closed properties the design pins (no mutation, no tmp, no `Allow`/`WWW-Authenticate` leak beyond the
   face's own rule). Framing refusals for a body are still reached on the admin paths the suite exercises, because
   those legs run against a loopback-open face.
7. **Previous round's blocker/major findings addressed.** Major 1: yes, verified at the seam and at three test
   levels. Major 2: partly — the shipped-tree falsehood is fixed, the ledger itself is not, re-opened at minor with
   a corrected premise (finding above). No blocker was open.

## FYI (no severity, not this seat)

`serve.rs:116-135` gates only `/v1/admin/**` from the header block, so a non-admin request still buys the
`vec![0u8; octets]` allocation in `http.rs:321` from a declared `Content-Length` alone — up to
`MAX_REQUEST_BODY_OCTETS` (16,777,216, the design's own figure at `02-proposal.md:402-404`) per admitted
connection, including on `GET`/`/v1/health`. Pre-existing and design-sanctioned, but it makes the new comment at
`http.rs:28-34` ("`MAX_REQUEST_BODY_OCTETS` is … not an unauthenticated peer's memory allowance") true for the
admin face only. Left to the security/performance seats.
