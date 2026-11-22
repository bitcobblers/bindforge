# compatibility review r1 (c1 dissent pass)

verdict: approve

Scope: the diff `7b5cec4 → fcb66a8` of `02-proposal.md`, then the full current
document, checked against ADR-027 and the Revision log. This pass first
re-tests my five round-1 findings against the changed text; everything else
reported here arises in the changed text itself.

## Previous findings — disposition

1. **[major, mint bound step] CLOSED.** The false "within `MAX_SLUG_OCTETS` by
   construction" claim is retracted and step 5 ("Bound") is the bound test I
   asked for: an over-`MAX_SLUG_OCTETS` id in either spelling mints
   `steam-<appid>`, the suffix is held ∪ retired-checked, and the fallback is
   record-stable. Arithmetic re-verified: `MAX_NATIVE_NAME_OCTETS = 128`
   (`crates/core/src/manifest.rs:58`), `MAX_SLUG_OCTETS = 128` endpoint-
   inclusive (`crates/core/src/placement.rs:83`), and the "≤ 26 octets" claim
   holds — `MAX_APPID_DIGITS = 20` (`crates/discovery/src/steam_common.rs:26`)
   plus the 6-octet `steam-` prefix.
2. **[major, native-dependency packaging] CLOSED.** "Pure-Rust TLS stack" is
   gone (§Options now says "`ring` compiles C and assembly for the *target*
   through a build script — not the 'pure-Rust' stack this section claimed");
   the fix names both halves I asked for — the `check.yml` Windows-leg scope
   split with `apps/registry` compile-checked on the host, and the `README.md`
   C-toolchain prerequisite (Rollout 3) — plus version floors and a RUSTSEC
   screen in place of the false "versions pinned; CI needs no edit".
3. **[minor, platform rows] CLOSED.** Windows is now "Yes (compile-only)" and
   names the shipped `x86_64-pc-windows-msvc` leg and its narrowing; macOS is
   "Dev only" with a named deferral; §Test strategy #16 names a platform test
   for every row the matrix still claims supported. Checklist items 1 and 7
   now pass.
4. **[minor, convergence count] CLOSED.** §Options now reads "five of the six
   … the seed's `portal-two` being the exception, which the rule derives as
   `portal-2`" — verified against `data/catalog/v1/catalog.json` (line 31) —
   and Rollout 6 carries the re-mint cost into the cap arithmetic.
5. **[minor, fail-closed client consequence] CLOSED.** The client row, the
   key-revocation risk row, and Rollout 6 all now name `025-C4`'s fail-closed
   rendering and read snapshot age as user-visible availability.

## Findings

- [minor][new] §Issued entries, the mint rule, steps 4/6 × §Test strategy #1 — step 4 says the suffixed id "earns the same held ∪ retired check before it is served" and step 6 says "Any spelling the rule lands on that is held or retired by a different record refuses the mint with the named `503` `mint identity held`" — i.e. a suffixed id held by a curated record refuses — but Test strategy #1 pins the opposite: "a suffixed id already held by a curated record takes `steam-<appid>`, and a held `steam-<appid>` refuses `mint identity held`". Implementations match suites, so the shipped rule would silently become the three-spelling cascade the rule text does not state, and #5's "an id retired under both spellings refuses `mint identity held`" is stale under either reading (three spellings can all be held). Fix: extend step 5's sentence to "A mint whose chosen id exceeds `MAX_SLUG_OCTETS`, **or whose suffixed id is held or retired**, mints `steam-<appid>`", and restrict step 6's refusal to `steam-<appid>` itself held or retired — replacing the current "Any spelling the rule lands on" phrasing — with #5's "both spellings" restated as "every spelling the rule can land on".
- [minor][new] §Test strategy #1 — "including the over-bound cases the arithmetic allows: a 128-octet ASCII name, a 128-octet digit-leading name, and a 128-octet *colliding* name all mint `steam-<appid>`" — the first fixture's expectation contradicts step 5: the fold never *grows* (single-base-letter folds are one octet per code point; hyphen collapse and trim only shrink; `g-` applies only to a leading digit), so a 128-octet digit-leading-free, non-colliding ASCII name folds to at most exactly 128 octets — `MAX_SLUG_OCTETS` is an endpoint-inclusive cap (`crates/core/src/placement.rs:83`) — and is in budget, minting its own slug, not `steam-<appid>`. Only the digit-leading (+2) and colliding (+7+digits) fixtures are over-bound. Fix: expect that fixture's own 128-octet slug — pinning the exactly-at-bound case as in-budget, which is the boundary the step-5 test actually needs — and keep the `steam-<appid>` expectations for the digit-leading and colliding fixtures.
- [minor][new] §The steam snapshot resolver / §The hosted lookup policy step 5 — "`incomplete` is how many records the pull dropped … Non-zero, the file is a list that knows it is short, so no steam non-hit is certifiable against it" — the changed text makes one dropped record a whole-instance certification cliff: a single attacker-chosen over-128-octet rename (A11 records renames to existing appids; A10 says the bound has one octet of margin over the longest surveyed title) permanently flips *every* steam miss — including non-decimal external ids — from `200 []` to `503 steam snapshot incomplete`, and by this document's own client row and key-revocation risk row that fails every bound CLI scan closed, immediately and until the store name shrinks (the mark re-publishes on every pull; re-sizing is Deferred). The Revision log's r0→r1 rebuttal correctly rejects failing the whole pull, but never weighs localizing the non-certification. Fix: carry the dropped appids in the snapshot document beside the count (same loader discipline, bounded by `027-B1`), and restate policy step 5's non-certifiable set as "a fresh snapshot, no such appid, and the appid is not one the pull dropped" — replacing the global "`incomplete` > 0 → every steam non-hit is `503`" rule — or, if the global rule is kept deliberately, record in §Risks that a single rename is a remote, permanent trigger whose blast radius is every bound scan, not only the operator's availability dial.
- [nit][new] §Assumption register A16 — "cross-`cargo check`s the *whole workspace* for `x86_64-pc-windows-msvc` — legitimate there only because `cargo check` generates no code" — the parenthetical contradicts the design's own mechanism: `cargo check` *does* run build scripts, and `ring`'s compiles C/asm for the target — exactly why §Options says the leg must drop `apps/registry` ("never asked of a runner without a target C toolchain"). The cross-check is legitimate today because no dependency runs a target-native build script. Fix: reword the A16 parenthetical so the assumption's rationale and §Options' mechanism agree.

## Checklist status

1. Matrix rows: all five carry statuses; no `unknown`, no empty cell. Pass.
2. OS input APIs: not applicable — server-side design, no input surface.
3. Node version floor: not applicable — `tools/orchestration` untouched (#17).
4. Native modules: `ring`'s C/asm dependency is now named with its packaging story (README prerequisite, CI-leg split, floors + advisory screen). Pass.
5. Keyboard reality: not applicable.
6. File paths: all four new documents plus `write.lock` live under the shipped `DataLayout`-resolved `<data>` branch (A2); Windows owner paths named; no string-concatenation of separators claimed. Pass.
7. Platform tests named for every supported row (Linux trio; Windows cross-check leg + `DataLayout`/env seams; client contract byte-exact suite #12); macOS/Other-Unix claim no support. Pass.

## FYI (no severity)

- §Budgets: `STEAM_PULL_DEADLINE_SECONDS = 900` is shorter than the worst page-pace arithmetic the same section funds (12 pages × (3 attempts × (2 s + 30 s) + 2 × 30 s gaps) ≈ 1,872 s), so a slow-but-healthy store trips the named deadline refusal before `STEAM_PAGE_ATTEMPTS` is exercised — a performance-seat question about which budget is load-bearing.
- A17 carries the response envelope unverified while §The store pull's page refusals are keyed to it; the named malformed-envelope refusal makes the failure loud, but the delta/`last_modified` story rests on an unverified shape — correctness seat.
- §Hosted mode's health fact (`{games, fetched_at, incomplete}`) and the snapshot document (`version, fetched_at, incomplete, apps`) add a field to what `027-C3` pins as `{games, fetched_at}`/`{version, fetched_at, apps}`; no shipped consumer breaks (the design names the retiring gauge as health's only in-repo reader), but Rollout 1's ledger lines name amendments to `026`/`024` units without a `027-C3` line — correctness/ledger seat.
