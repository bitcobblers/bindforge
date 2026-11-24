# Code review — testing seat — round 1, pass c2 (ratification)

**Run:** `017-keybinding-extraction-codecs` · **Design:** `02-proposal.md` (+ `docs/adrs/017-keybinding-extraction-codecs.md`)
**Scope reviewed:** `git diff a8fd09ec6d6b8210574447f9b68e7cc199f73153..HEAD -- ':(exclude)docs/designs'` (15 files, +504/−44), read against the full current proposal.
**Pass type:** ratification — checking only that the revisions that landed after this seat approved did not break what was approved. Round-1 minors that were never scheduled as gating stay carried, re-raised unchanged, marked as such.

## Gate evidence

Supplied by the runner (gate at 2026-11-24T07:31:52.450Z, green, exit 0 on every leg): `npm run check`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`. The captured `test result: ok.` line is the last leg's; no leg failed. This seat's shell allow is `git diff*`, so **no check was executed here**; the diff was taken from the same commits the on-disk sources carry (the working tree contains every hunk in the range, e.g. `read::decode_owned`, `locate`'s `opened` counter, `bindings.rs`'s `claim_route`), so the gate is read as current for the reviewed content. No failing leg to cite.

## Ratification checks — what round-1 approval rested on, re-verified

| # | Approved property | Result | Basis |
|---|---|---|---|
| 1 | Test strategy #10 — the loader refuses by name; the invalid corpus is the conformance currency | **Holds, extended** | `snapshot.rs:305-323` adds a `files` spelling rule; the new fixture `files-traversal-files.json` names its key per the loop's `expected_key` rule (`tests/snapshot.rs:289-310`), so `every_invalid_schema_fixture_is_refused_naming_its_key` picks it up; `tests/snapshot.rs:177-223` adds six escaping spellings inline |
| 2 | Test strategy #11 — extraction and contract cannot disagree | **Holds** | `tree_relative` (`unreal_ini.rs:548-555`) is now a `/`-join of components — octet-identical on Linux, so the loader round-trip at `tests/snapshot.rs:228-248` still covers both shipping trees; no valid fixture carries `\` or `:`, and the empty-`files`-entry refusal (`snapshot.rs:299`) is untouched |
| 3 | Test strategy #9 — appid pass before probe pass, first claim in id order, evidence tokens | **Holds** | `registry.rs:39-60` only skips re-running a `detect` pass 1 already ran; return shape and evidence unchanged; the new `tests/registry.rs:215-253` pins detect-at-most-once, and its `AtomicUsize` is touched by no other test in that binary, so it is deterministic |
| 4 | Test strategies #4/#6 — the open cap, and the walk's stop-and-count | **Holds** | `locate` counts opens (`unreal_ini.rs:188-190,245-248`), matching §The first family's wording; `tests/unreal_ini.rs:426-474` pins both sides of the cap (past → no claim, inside → located) |
| 5 | §Determinism / Test strategy #12 — byte-identity | **Holds** | no ordering law changed; the `Fold::remove` fast path, `decode_owned` and the CLI memo are all output-neutral; `tests/determinism.rs` is outside the diff and green in the gate |
| 6 | §Read discipline / Test strategy #5 — `decode` semantics | **Holds** | `decode` is now a wrapper over `decode_owned` (`read.rs:57-80`); BOM, UTF-16 and strict-UTF-8 paths unchanged; `zz_api_probe.rs:76-79` resolves both spellings, so the new public name is on the pinned surface |
| 7 | Test strategy #7 — the merge fold | **Holds** | `Fold` keeps `seen` and `pairs` in lockstep (`add` inserts both, `remove` removes both, `clear` clears both — `unreal_ini.rs:342-375`), so the early return cannot strand a pair; the minus/bang/clear tests still run green, and `tests/unreal_ini.rs:573-594` pins the new no-op path |
| 8 | `017-L1` never-write, and Test strategy #13's bytes-unchanged check | **Holds** | the diff adds no write, no `OpenOptions` change and no new file creation outside test temp roots; `cli_bindings_tests.rs`'s byte-identity check is untouched |
| 9 | `017-L5` — `list` untouched | **Holds** | no `main.rs`, `COLUMNS`, `json_row` or `discover()` hunk in the range |
| 10 | §Platform #17 — Windows leg stays compile-only | **Holds** | the three new `#[cfg(unix)]` tests each carry a `#[cfg(not(unix))]` arm (`tests/unreal_ini.rs:225-231,320-324,405-410`) |
| 11 | §Read discipline's aggregate sentence (README + spec) | **Holds, restated** | README's bullet and `schemas/bindings/v1/spec.md:854-864` both now state `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES × MAX_BINDING_FILE_OCTETS` = 1,088 MiB; the arithmetic matches shipped behavior (locate reads the bytes for the content test, `read_and_decode` re-reads each located file) |

No previously approved guarantee was broken by this pass. Out-of-specialty FYI (no severity): the `locate` budget is tested both at the top of the queue loop and inside the entry loop, so the top-of-loop `break` is reachable only through an empty queued directory.

## Findings

### 1. minor — the broken-prefix degradation is now *asserted* in a source comment, and still pinned by no test (re-raised from c0, changed by this pass)

**Location:** `crates/codecs/src/paths.rs:130-141` (revised comment); `crates/codecs/tests/fixtures/unreal/` (fixture set unchanged: steam-userdata, proton-prefix, bom-utf16, defaults-only, bindings-absent, broken-tuple — no `symlinked-prefix`); `apps/discovery-cli/tests/cli_bindings_status_tests.rs` (no `probe` string in the file).

**Problem:** §The first family names a `symlinked-prefix` fixture tree and Test strategy #6 names "a symlinked *tree root* in `symlinked-prefix`"; the tree still does not ship, and the resolver-level pin (`tests/paths.rs:140-175`) stops at "no `ProtonPrefix` tree" — no test reaches the CLI status the Compatibility matrix row promises ("reads as `family-confirmed-files-absent` (exit 4)", Risks row 3). This pass resolved the ambiguity in a **comment**: `paths.rs` now states the answer is `claim-refused` (exit 3) and that exit 4 "belongs to appid-table claims". The design document the change is reviewed against still says exit 4, so the two now disagree on paper with nothing in the suite on either side; worse, the comment's rationale misdescribes the shipped code, where `bindings.rs:269-271` sets `family-confirmed-files-absent` for *any* claim — probe-claimed included — whose documents all have empty entries, which is how the existing exit-4 test reaches that status. A future edit that makes the comment true (exit 4 only via appid tables) or the matrix true (exit 4 for a linked prefix) keeps the suite green either way.

**Fix:** one `#[cfg(unix)]` fixture-or-temp-root scenario plus one CLI subprocess test: a `bindings <slug>` run against a Proton prefix whose `steamuser` root is a link, asserting the single `status:` line and its exit code — and make the matrix row, `paths.rs`'s comment and the exit-4 route in `bindings.rs:269-271` state the same rule.

### 2. minor — `UnreadableFile` is still reached by no test, and this pass added a new silent branch that consumes it (new half is this pass's)

**Location:** `crates/codecs/src/families/unreal_ini.rs:257-265` (the split `Err` arms), `:178-180` (the doc that now promises "an `UnreadableFile` … is skipped and counted, never a candidate"), `crates/codecs/src/read.rs:27,36,45`, `crates/codecs/src/codec.rs:196-200`.

**Problem:** the round-1 split of the `Err` arm (rightly made to stop an invented path becoming a candidate) also turned the IO-error case into an unobserved skip. `locate`'s new `Err(_) => {}` arm is reached by no test — a symlink is already skipped at `:228-230`, so only a permission-denied or raced `*.ini` gets there, and the suite has neither — and §The first family's R1 posture ("a match the reader refuses is located on path grammar alone and `extract` refuses it by name, never unseen") is now only true for two of the three read refusals. A workspace grep for `UnreadableFile` still returns src/ hits only: the variant stays unreached, and its new `Display` echo (`codec.rs:196-200`) is untested; of the four arms this pass routed through `escape_and_truncate`, only `UnparsableEntry` is pinned (`tests/unreal_ini.rs:371-410`).

**Fix:** two cheap tests — (a) a `chmod 000` (or mode-`000`-guarded, `#[cfg(unix)]`) grammar-shaped `.ini` asserting the tree still claims whatever else it located and that the unreadable file is neither a candidate nor a refusal, which is what the new arm promises; (b) the round-1 ask, `UnrealIni.extract` on a hand-built `Detection` whose `tree_root` spells neither resolver template, asserting `ExtractionRefusal::UnreadableFile { .. }` and that its printed line names the variant.

### 3. minor — the `schemas/README.md` statements Test strategy #15 names as contract-tree pins are still read by no test, and this pass rewrote the pinned text again (re-raised from c0)

**Location:** `schemas/README.md:200-297` region (the seven-constant budget family and the rewritten aggregate bullet), `schemas/bindings/v1/spec.md:854-864`; no test in the diff (or anywhere) reads `schemas/README.md` for codecs' budget family or the two-JSON-formats passage.

**Problem:** the repo pins comparable prose elsewhere (`crates/core/tests/ledger_annotations.rs:130-160,231-247` asserts README bullets against drift), and #15 lists exactly these README statements as pins. Reverting the passage to one JSON format, dropping the codecs budget family, or changing the aggregate in only one of the two places that now restate it — this pass moved 544 MiB → `2 × …` = 1,088 MiB in both — leaves `JSON_FIXTURE_FORMATS`, the conformance walk, the boundary sweep and every Rust suite green. This pass edited the unpinned text rather than pinning it.

**Fix:** one `ledger_annotations`-style test: read `schemas/README.md`, assert the two-format passage names `catalog` and `bindings`, assert the budget bullet names the seven `MAX_*` constants with their values and states the `2 × (MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` product (comparing the number against the constants, not a literal), and assert the hygiene sentence names `schemas/bindings/`.

### 4. minor — the CLI claim memo added by this pass is a branch no test can reach, and its key omits fields `detect` may read (new)

**Location:** `apps/discovery-cli/src/bindings.rs:222-237`.

**Problem:** the memo is keyed on `(appid, library, roots)`. Two `ScanRecord`s that share all three would need the same slug (`steam/<appid>`) and the same owning library holding the same appid twice — a shape `SteamCore`'s per-root library ownership and dedupe cannot produce, so the `or_insert_with` hit path never runs and no test can run it (checklist 7: untestable additions are findings). Its comment names the case it does *not* cover: "two records of one game in two libraries" differ in `library`, so both walks are still paid — which is also the shape `tests/determinism.rs:94-126` and `cli_bindings_tests.rs:403-404` pin. Separately, the key drops `install_path`, `platform` and `lookup_key`; that is safe for `unreal-ini` only because `detect` reads just `roots`/`appid`/`library` (`unreal_ini.rs:147-168` + `paths.rs`), and nothing — comment, invariant, or test — holds that for the next family, one of which (`017-R3`'s Source lineage) is plausibly install-tree-reading.

**Fix:** either drop the memo (the walk is capped at 17 × 32 opens anyway) or key it on the whole `GameContext` and pin the hit path in a unit test; in either case fix the comment's two-libraries claim, or state the invariant ("no shipped `ScanRecord` pair shares the key") the code now depends on.

### 5. minor — probe-mode statuses and the two defensive arms stay unpinned (re-raised from c0, unchanged by this pass)

**Location:** `apps/discovery-cli/src/bindings.rs:249-254`, `:262-265`, `:282-284`; `apps/discovery-cli/tests/cli_bindings_status_tests.rs` (contains no `probe` string).

**Problem:** statuses 0/3/4/5 are pinned through the non-probe route only; `bindings <slug> --probe` on a game no family claims — and its `{"status":"claim-refused","claims":[]}` document, Test strategy #13's "four statuses each producing one named stderr `status:` line and its own exit code … with `--json` printing the status document" read against `--probe` — has no test. The `registry::family()` `None` arm and the `status == Ok && documents.is_empty()` arm remain unreachable through the shipped registry.

**Fix:** one probe-mode status test per failure kind (table mode: exit 3 with one `status:` line; `--probe --json`: parses to `status`/`claims`), and either drop the two unreachable arms or comment the invariant that makes each defensive-only.

### 6. nit — "skipped and counted" is still a fact the tree cannot show (re-raised from c0)

**Location:** `crates/codecs/src/families/unreal_ini.rs:188-190,219-221,245-246,263-265`; `crates/codecs/src/paths.rs:130-141`; `crates/codecs/src/lib.rs:51-58`.

**Problem:** Test strategy #3 and #6 both say skipped entries are "skipped and counted", and this pass added a third skip-and-count site (`opened` past `MAX_CANDIDATE_FILES`) plus a comment that leans on the count. No count is observable anywhere — not in a refusal, not in `--probe`, not in a log — so the tests pin only that the walk stops and the hidden candidate is never located. `lib.rs`'s revised wording discloses that one `read_dir` listing is materialized *before* the budget trims it, which is another unobservable quantity.

**Fix:** surface the counts (a probe column or a budget-named line makes them testable), or record the wording divergence in the Revision log so the operator ledger does not carry a fact the tree cannot show.

### 7. nit — dead harness helpers survive the pass untouched (re-raised from c0)

**Location:** `crates/codecs/tests/common/mod.rs:101` (`TempRoot::files_below`, no callers), `crates/codecs/tests/honesty.rs:37` (`honest_games`, `#[allow(dead_code)]`, its fixture list naming `broken-tuple`, which only ever refuses).

**Fix:** delete both, or use `files_below` for a never-write byte check in the codec suites (it is exactly the helper the `017-L1` byte check needs) and drive the honesty loops from `honest_games()`.

### 8. nit — a new test cites the wrong Test strategy number

**Location:** `crates/codecs/tests/unreal_ini.rs:325-327` — `// proves: Test strategy #7 and #13` on `a_control_byte_in_an_echoed_path_is_escaped_and_stays_one_line`.

**Problem:** #13 is the CLI subprocess integration item (tables, JSON arrays, statuses); this is a `crates/codecs` unit test of `ExtractionRefusal::Display`, which is §Read discipline's echo rule under #7 (and #10's "single-line, control-free" message rule). Mis-citation makes the coverage map read as if the CLI face pinned it.

**Fix:** cite #7 (and #10's message rule), leaving #13 to the CLI suites.

## Verdict

**approve.** No blocker or major finding: every hunk in the range preserves the properties this seat approved in c0 — the loader's refusals and corpora, the registry's two-pass ordering, the fold's semantics, byte-level determinism, the never-write posture, `list`'s untouchedness, and the four statuses' exit-code table. What remains is eight carried-or-new test-coverage items, all of which the fix pass could close without touching a design section; findings 1 and 4 are the two where the revisions themselves left the sharpest gap between what the sources now claim and what the suite can show.
