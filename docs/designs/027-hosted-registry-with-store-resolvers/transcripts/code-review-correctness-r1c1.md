# Transcript: code-review:correctness r1c1

## Provenance
- stage: code-review:correctness (round r1c1)
- agent: codereview/code-correctness
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T09:51:31.929Z
- ended: 2026-11-22T10:13:47.210Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":84425,"output":38128,"reasoning":0,"cache.read":4052300,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: git diff 75cbc433ed8a546734c41d4eb928395373ab73e6..HEAD -- ':(exclude)docs/designs', read against the full current document.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/027-hosted-registry-with-store-resolvers.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/correctness-r1c1.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-22T09:51:31.853Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (correctness seat):
[
  {
    "severity": "blocker",
    "location": "apps/registry/src/main.rs:368-401",
    "problem": "`load_or_pull_snapshot` accepts any loadable snapshot and pulls only when nothing loadable exists, reversing `027-C3`'s verbatim clause that the snapshot is loaded \"if present and within `027-B2`'s freshness, else pulled in full before the process binds\" (ADR-027 lines 105-108; proposal 268-273). An instance booted over a snapshot past `STEAM_SNAPSHOT_MAX_AGE_SECONDS` answers `503 steam snapshot stale` for every steam key until the refresh thread's first delta, up to 86,400 s later (main.rs:458-463 counts the startup as the last pull), and a bound CLI fails a whole scan closed on those 503s. No startup leg tests the designed direction (hosted_startup.rs has none; tests/steam.rs:210 only pins that a stale file loads). The comment at main.rs:357-367 claims a §Deviations entry in 03-build-notes.md, which is outside my permitted inputs, so the declaration could not be verified.",
    "fix": "Load only a present-and-fresh snapshot; treat past-age (and present-but-unloadable) as the pull case, keeping the named startup refusal for a failed pull; add the `027-C3`/Test-strategy-#8 startup leg. If the posture is intended instead, amend `027-C3` before shipping the code."
  },
  {
    "severity": "major",
    "location": "apps/registry/src/steam.rs:527-545",
    "problem": "A full pull that collects no records publishes an empty worldlist: `pull`/`SteamSnapshot::published` (steam.rs:265-275) accept an empty `collected`, `decode_page` (862-893) accepts `{\"apps\": [], \"have_more_results\": false}`, and `load_snapshot` (372-412) loads an empty `apps` list. The module's own doc claims the opposite (\"A published empty worldlist is a refusal of the pull that produced it\", steam.rs:190-197) and no test covers it. A 200-with-empty-list answer (key without the IStoreService grant, gateway envelope) replaces a good snapshot at startup, after which every `steam/<appid>` lookup answers `200 []` from state.rs:364-368 — the fabricated \"never heard of it\" §Options' partial-snapshot rejection refused and `027-L3` does not license.",
    "fix": "Fail the pull by name when a full pull yields zero records (and when a page yields zero records with have_more_results false), refuse an empty `apps` list at load as a malformed shape, and add the legs to tests/steam.rs."
  },
  {
    "severity": "major",
    "location": "apps/registry/src/state.rs:358",
    "problem": "`external_id.parse::<u64>()` accepts non-canonical spellings (`steam/007`, `steam/+7` both parse to 7, and keys.rs:60-85 admits them), while mint.rs:97 spells the appid ids from the queried spelling. So `steam/007` mints `my-game` and `steam/7` — same appid, same snapshot record — is pushed by the step-4 held check to `my-game-steam-7`, the spelling the design reserves for a different record with the same name (proposal 387-390). One game gets two load-bearing ids, the design's \"appid-unique\" claim (proposal 385, 399-404) is false, and one appid can consume ~120 of the 200,000 `027-B3` slots, breaking the §Risks cap arithmetic.",
    "fix": "Require canonical decimal spelling before the mint path (`app.appid.to_string() == external_id`, else answer `200 []` like any non-appid token) and spell `steam-<appid>` from the snapshot record's `appid`; extend Test strategy #1 with the `007`/`+7` rows."
  },
  {
    "severity": "major",
    "location": "apps/registry/src/store_lock.rs:116-134",
    "problem": "`refuse_or_adopt` deletes a lock whose recorded pid is absent from `/proc` and takes it over (process_is_live, store_lock.rs:171-194; the module doc at 9-18 names the narrowing). §Concurrency (proposal 471-479) chose \"finding it present is the named startup refusal\" with runbook cleanup precisely because the fixed-tmp discipline cannot refuse a second process. A pid is not a cross-namespace identity: with a shared `<data>` volume, a `query` in another container cannot see the live `serve`'s pid, deletes the lock, and becomes the second persist-before-serve writer over the live instance's `027-L2` records.",
    "fix": "Implement the designed rule (present implies refuse) and leave corpse cleanup to the Rollout 6 runbook; if stale adoption is wanted, it needs a cross-namespace-safe liveness proof (owner heartbeat rewrite or flock on the same inode) and a proposal amendment first."
  },
  {
    "severity": "major",
    "location": ".github/workflows/check.yml:29-32",
    "problem": "Rollout 3's gate work is absent from the diff: the workflow still runs `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`, which now builds `apps/registry` with `rustls`/`ring` (Cargo.toml:21-26). The proposal requires this leg's scope split twice (proposal 106-109, 722-724; §Compatibility matrix 559) precisely because of `ring`'s target build script (A16). The same Rollout item's \"RUSTSEC advisory screen added to the gate\" (proposal 721) is also absent — the pre-existing advisories.yml is a weekly scan whose findings deliberately do not fail the job. The runner's green gate record runs neither leg.",
    "fix": "Land the check.yml split (exclude bindforge-registry from the msvc leg, compile-check it on the host) plus the advisory screen, per Rollout 3."
  },
  {
    "severity": "major",
    "location": "apps/registry/Cargo.toml:21",
    "problem": "`rustls = { version = \"0.23\", ... }` (and `webpki-roots = \"1.0\"` at line 26) declares no floor, where the design names the floors as the mitigation for its largest new dependency: rustls >= 0.23.46 (RUSTSEC-2026-0346, Debug disclosing TLS secrets), rustls-webpki >= 0.103.16, webpki-roots >= 1.0.10 with the 1.x-parent note (proposal 93-101, Risks 543). Cargo.lock satisfies them today (0.23.46 / 0.103.17 / 1.0.12), but any re-resolution inside `^0.23`/`^1.0` lands on an affected version and, per the previous finding, nothing screens it.",
    "fix": "Declare the floors in the manifest (`rustls = \"0.23.46\"`, `webpki-roots = \"1.0.10\"`, rustls-webpki at its floor) or enforce them via deny.toml minimum-version/advisory policy so the floor is structural rather than incidental to the lockfile."
  },
  {
    "severity": "major",
    "location": "apps/registry/src/main.rs:338-355",
    "problem": "`Config::store` resolves `partner.steam-api.com` via `to_socket_addrs` and is called at main.rs:305, before `load_or_pull_snapshot` decides whether anything will be dialed. Every hosted suite leg that runs the real binary with a key and a planted snapshot (tests/hosted_end_to_end.rs:286, 369, 389, 414, 523, 543) therefore performs a live DNS lookup for the real store authority, which Test strategy #17 forbids explicitly (proposal 693-696); the suite's greenness depends on external DNS. It also hard-fails a networkless restart whose snapshot is fresh and needed no pull.",
    "fix": "Decide load-vs-pull first and resolve only on the branch that will dial; add a guard so no test resolves the authority (injectable resolver or an assertion that the resolution failure is unreachable with a fresh snapshot)."
  },
  {
    "severity": "minor",
    "location": "docs/adrs/026-registry-service.md:1",
    "problem": "Rollout 1's ledger annotations did not land: the diff touches only ADR-027 itself under docs/, and neither docs/adrs/026-registry-service.md nor docs/adrs/024-game-slug-identity.md mentions 027. The proposal (702-710) requires amended-by lines on `026-C1`/`026-C2`, retirement lines on `026-C6`, `026-C7`, `026-B1`-`B3`, `026-L4` amended-by, `026-L5`'s caching half, `026-R2`'s intake half, and the `024-C2` cross-reference, while this branch deletes cache.rs, upstream.rs, endpoint.rs and the 026-B1-B3 constants.",
    "fix": "Land Rollout 1's annotations per 021-L4."
  },
  {
    "severity": "minor",
    "location": "docs/registry-hosting.md:1",
    "problem": "The Rollout 6 operator runbook is absent. Everything proposal 736-756 names has no artifact: key custody and the CWE-526 stance, the no-URL-in-logs rule, custody of corpus/issued/retired-ids/snapshot, forcing a full refresh, deleting a crashed run's write.lock, the retired-list size check, the 026 document-pair cleanup with no migration, query as a stopped-instance tool, the restart policy bounding daily startup pulls, snapshot age as user-visible availability, cap arithmetic, and the Steam ToS obligations by name. Findings on write.lock recovery and startup pulls reference runbook steps that do not exist.",
    "fix": "Write docs/registry-hosting.md with the enumerated content, per Rollout 6."
  },
  {
    "severity": "minor",
    "location": "apps/registry/src/admin.rs:231-249",
    "problem": "`Answer::refused` has no 503 arm, so `AdminRefusal::Reject { status: 503 }` from state.rs:662-666 renders as `HTTP/1.1 503 Bad Request` with `{\"error\":\"bad_request\"}`; the name `retired id list full` survives only in the body prose, which is why tests/retired_ids_and_curation.rs:496-507 accepts \"line or body\". That contradicts the branch's own candidate-face discipline (serve.rs:304-313: the reason phrase carries the name because the status line is all the CLI shows) and the design's \"named refusal\" wording (proposal 440-446).",
    "fix": "Add a 503 arm (reason + error token) or carry `Named` through `AdminRefusal` so the admin face renders the same names; tighten the test to assert the status line."
  },
  {
    "severity": "minor",
    "location": "apps/registry/src/persist.rs:178-203",
    "problem": "`owner_only_dir` replaced an unconditional `set_permissions(0o700)` with a conditional (`if mode & 0o077 != 0`), changing a shipped invariant A2 records as \"parent `0700`\" for reasons of test harness behavior (the injected persist failure), not anything the proposal asked for. The owner-only ceiling is still enforced and a stricter directory now fails the write closed, so the concern is silent drift of a shipped discipline rather than a live hole.",
    "fix": "Either express the intent as an explicit never-widen check separate from the shipped discipline, or inject the read-only directory in the test without changing the shipped rule; record the choice under §Deviations."
  },
  {
    "severity": "minor",
    "location": "apps/registry/src/steam.rs:58-117",
    "problem": "§Budgets specifies a \"new `constants.rs` family ... kept in the one file\" (proposal 504-526), but the snapshot/page/retry/timeout family lives in steam.rs while constants.rs carries only the issued/retired budgets plus a pointer note. The justification at steam.rs:61-66 (\"They live here until the increment that wires this resolver into the serving face ... the binary declares its own copy of `constants` and has no reader for them yet\") is stale inside this diff: main.rs:39 already consumes `bindforge_registry::constants` and the resolver is wired.",
    "fix": "Move the store family into constants.rs as §Budgets states and delete the stale comment."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:901-920",
    "problem": "`normalize_store_name`/`clamp_to_name_bound` re-implement the whitespace-collapse-and-clamp rule instead of calling `discovery::game::friendly_name` \"through that existing `discovery` dependency\" (A7; proposal 306-312). I compared both and they are behaviorally identical, so nothing is wrong today — but a future divergence in crates/discovery/src/game.rs becomes invisible here.",
    "fix": "Call `discovery::game::friendly_name` after the `char::is_control()` strip."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:883-887",
    "problem": "A store record with a missing or non-string `name` silently becomes an empty name instead of the named malformed-envelope page refusal; A17's envelope carries `name` on every record and §The store pull names the malformed-body refusal, so this is a third unnamed posture.",
    "fix": "Refuse the page with the malformed-shape name, or document that a missing name takes the empty-name posture."
  },
  {
    "severity": "nit",
    "location": "apps/registry/src/steam.rs:439-503",
    "problem": "The design names the seam `StoreTransport`; the code calls it `Transport` with a `Loopback` variant, and the only production construction path is `SteamStore::for_test(address, key).with_production_transport(name)` (main.rs:354), so the name conceals which transport a hosted instance actually runs.",
    "fix": "Rename to `StoreTransport` and add a `SteamStore::production(...)` constructor for the bin path."
  }
]
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the correctness checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/correctness-r1c1.md, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only. The entire response must be a single valid JSON object with no prose, explanation, markdown, or additional text before or after it.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"apps/registry/src/steam.rs:437-471, 620-637, 336-343; apps/registry/src/state.rs:379-383","problem":"The r1 fixes' behavior is correct but unpinned. `startup_snapshot` has one call site outside its own definition (main.rs:362); grep over apps/registry/tests finds no reference to it, so none of `027-C3`'s three load-or-pull arms is exercised — deleting the `!snapshot.is_past_age_bound(now)` guard or reverting the arm order keeps the suite green, which is precisely the failure mode my round-1 blocker named. The same holds for the two new named refusals (`the store's worldlist answered with no records at all`, `the steam snapshot's `apps` is empty`) and for the canonical-appid-spelling gate: no leg answers `steam/007`/`steam/+7`, no fake returns `{\"apps\": [], \"have_more_results\": false}`, and no fixture plants an empty `apps` list (tests/steam.rs:730-806 covers only the page budget and the lost-later-page partial case). The design's Test strategy #8 names no stale-boot leg and #13's planted-stale half moved in-process, so no listed leg asks the question either. The property is establishable by inspection (35-line function, single reader of the guard), so this is evidence-thinness rather than a blocker.","fix":"Add three tests/steam.rs legs driving `startup_snapshot` against the existing loopback fake — fresh file returned as loaded with `fake.request_count() == 0`; past-bound file triggering a full pull that rewrites the file at the new stamp; unloadable file triggering the repair pull — plus one leg for each new refusal (empty-list answer, a continuation page carrying no records, an empty-`apps` file refused at load), and `steam/007`/`steam/+7` rows beside the `200 []` cells in tests/hosted_lookup_policy.rs."},{"severity":"minor","location":"apps/registry/src/steam.rs:620-637, 336-343; apps/registry/src/main.rs:456-461, 367-401; apps/registry/src/state.rs:400; apps/registry/src/lib.rs:25-35; crates/core/tests/gate_contract.rs:192-201","problem":"Six behaviors this revision added are not named in the proposal, and none can be shown declared (03-build-notes.md was outside my permitted inputs): the empty-worldlist pull refusal and the empty-`apps` load refusal, neither of which is in §The store pull's enumerated named-refusal list (proposal 300-305); the refresh thread's first sleep anchored on the snapshot's `fetched_at` rather than the proposal's \"sleep STEAM_SNAPSHOT_REFRESH_SECONDS\" (342-347); `steam-<appid>` spelled from the record's `appid` where mint step 5 spells it \"from the queried key's external id\" (399); `same_document`'s dev/inode plus case-insensitive identity for the `--corpus` exception, which also spares hard links; and the lib face taking `corpus`, `keys`, `mint`, `retired` and `state`. Separately, gate_contract.rs:193-201 re-pins design 008's Test-strategy-#6 \"two independent jobs per PR\" to three jobs so the new advisory screen can be checked.","fix":"One §Deviations line for each of the six, and one line recording the cross-design re-pin of design 008's two-job gate contract."},{"severity":"minor","location":"docs/ (no docs/registry-hosting.md); apps/registry/src/store_lock.rs:78-80","problem":"Rollout 6's runbook still does not exist (`ls docs/` returns adrs, designs, prompts, tasks, workflows.md), and this revision made it load-bearing. Because a present lock is now always the named refusal and no in-process liveness check remains, every unclean stop — including a SIGTERM, which runs no `Drop` — requires the runbook's \"delete a crashed run's leftover write.lock\" act before the instance can start again; three suites now perform that act as Rollout 6's (tests/common/mod.rs:432-455, hosted_startup.rs:376-403, data_layout.rs:193-198, issued_entries.rs:136-145). The proposal's other runbook-only controls for this branch's behavior — the restart policy bounding daily startup pulls (the corrected clause now pulls on every boot over an absent or past-age snapshot), forcing a full refresh by deleting the snapshot, and the retired-list size read — remain undocumented (proposal 736-756).","fix":"Land docs/registry-hosting.md with the enumerated Rollout 6 content, including the write.lock cleanup and the restart-policy arithmetic."},{"severity":"minor","location":"docs/adrs/026-registry-service.md:1; docs/adrs/024-game-slug-identity.md:1","problem":"Rollout 1's ledger annotations still have not landed (r1 finding #8, unchanged): the diff touches no file under docs/adrs/ except ADR-027, `grep 027` over 026-registry-service.md returns only the pre-existing 026-R2 reservation lines, and 024-game-slug-identity.md has none, while proposal 702-710 requires amended-by lines on 026-C1/026-C2, retirement lines on 026-C6, 026-C7 and 026-B1–B3, 026-L4's amended-by 027-L3, 026-L5's caching half, 026-R2's intake half, and the 024-C2 cross-reference — against a branch that deleted cache.rs, upstream.rs, endpoint.rs and the 026-B1–B3 constants.","fix":"Land the annotations per 021-L4."},{"severity":"nit","location":"apps/registry/tests/retired_ids_and_curation.rs:496-507","problem":"The rendering fix landed (AdminRefusal::Named plus Answer::named now emit `503 retired id list full` with the `retired_id_list_full` body token), but the assertion still accepts the name in \"line or body\", so it would pass a `503 Bad Request` with the name only in prose — the exact looseness that let the r1 finding through.","fix":"Assert the status line carries `retired id list full`."},{"severity":"nit","location":"apps/registry/src/steam.rs:790-813; apps/registry/src/main.rs:456-461","problem":"Test strategy #17's no-outbound rule now holds by convention only. Every keyed subprocess leg plants a `now_unix()` snapshot or starts `--no-steam`, and lazy resolution means a fresh snapshot never consults the resolver, but nothing structurally prevents a future leg that plants no snapshot — or one older than STEAM_SNAPSHOT_REFRESH_SECONDS — from dialing partner.steam-api.com for real; with the new `fetched_at` anchor such a snapshot makes the refresh thread dial within seconds of bind instead of a day later.","fix":"Give `Endpoint::Pinned` an injectable resolver seam, or add a suite-level guard that fails a run which dials the production store."},{"severity":"nit","location":".github/workflows/check.yml:83-126","problem":"The new advisory screen's current greenness is unevidenced: the runner's gate record contains npm run check, cargo fmt, cargo clippy and cargo test only — no `cargo deny check advisories` leg and no msvc leg — and cargo-deny is not installed in this environment, so I cannot establish that today's Cargo.lock clears the screen the job promises will fail the PR. A lock carrying an unwaived advisory would turn the new job red on landing while the scheduled scan (`|| true`, advisories.yml:79-83) stays silent.","fix":"Record a `cargo deny check advisories` run and a `cargo check --workspace --exclude bindforge-registry --all-targets --target x86_64-pc-windows-msvc` run against this lock in the rollout checks."},{"severity":"nit","location":"apps/registry/src/state.rs:227-237, 567, 622, 671, 709","problem":"`AdminRefusal::Named`'s doc says the variant carries capacity refusals \"or a mutation would push a document past its cap\", but all three document-cap refusals are still `AdminRefusal::Reject { status: 422 }`; `RetiredIdListFull` at state.rs:709 is its only producer, so the comment overstates the routing.","fix":"Drop the cap clause from the doc, or route those refusals through the named vocabulary too."},{"severity":"nit","location":"apps/registry/tests/health_route.rs:150-234; apps/registry/tests/hosted_lookup_policy.rs:486-520; apps/registry/tests/cli_end_to_end.rs:300-375","problem":"The relocated stale-supply pins leave no on-the-wire proof of the in-service case: the stale legs now read state.rs's faces in-process and the socket legs were re-spelled onto `--no-steam`, so no socket leg answers `503 steam snapshot stale` any more. The relocation is the honest reading of an instance that ages a snapshot only through a refresh thread that has no loopback seam, and §Compatibility matrix's bound-CLI claim is still pinned on a real named 503 whose reason survives the CLI hop.","fix":"Record the relocation in the build notes as the intended pairing with F1's `startup_snapshot` legs, rather than leaving the coverage shift implicit."}]}
```
