# Transcript: design-review:compatibility r1c1

## Provenance
- stage: design-review:compatibility (round r1c1)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:07:56.173Z
- ended: 2026-11-24T04:25:46.578Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":64824,"output":33233,"reasoning":0,"cache.read":1109429,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 37bd2f3753a299cff3db8ca37d9427ca3ba5d404 a4dda6a8542694895102f4a9460835e189e4f08b` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/017-keybinding-extraction-codecs.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/compatibility-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "Compatibility matrix — Flatpak / Snap Steam row: \"The face inherits `host_candidates` roots (A3) unchanged, and every candidate template is relative to the owning root, not `$HOME`\"",
    "problem": "[new] The premise is false of one of the three shipped resolvers: §Config-path resolvers defines `NativeHost` as `$XDG_CONFIG_HOME`, else `$HOME/.config`, read from the BindForge process's own environment, so it is root-independent; only `SteamUserdata` and `ProtonPrefix` are root-relative. A Flatpak/Snap Steam install puts a native-launched game's `$HOME` inside the package (`$HOME/.var/app/com.valvesoftware.Steam/config/…`, `~/snap/steam/common/.config/…`), which no shipped template reaches, so the row's unqualified `Yes` is overclaimed — such a title answers no-claim. Evidence: crates/discovery/src/steam_linux.rs:112-125 against the proposal's own `NativeHost` rule.",
    "fix": "Narrow the row to what the templates cover: root-relative trees Yes, sandboxed launcher's own config home not searched (native-context title answers no-claim), and either derive a per-root config-home candidate or name the gap in §Deferred beside `017-R2`."
  },
  {
    "severity": "major",
    "location": "Compatibility matrix — \"Steam Deck | Unknown\" and \"macOS | Unknown\"",
    "problem": "[new] The checklist admits no `unknown`: every row must commit, and both are decidable from evidence the design already cites. macOS is a definite No — crates/discovery/src/game.rs:21-28 publishes `Platform` as exactly SteamLinux/SteamWindows and apps/discovery-cli/src/main.rs:131-132 builds only those plugins, so no macOS row can ever reach `GameContext::from_record`; §Deferred already defers \"the macOS Steam root\" to `017-R2`, which contradicts calling the row unknown. The Deck is a Linux Steam install with the same `$HOME` layout and Proton prefix rule the Proton row claims Yes for; \"no Deck leg exists (A22)\" is a coverage fact, not a status.",
    "fix": "Mark macOS `No — no macOS discovery source exists, so no row resolves` (pointing at `017-R2`), and Steam Deck `Yes, by the Linux row`, moving the no-Deck-fixture note into that row's Notes."
  },
  {
    "severity": "major",
    "location": "Test strategy #16 — \"the workspace `Cargo.lock` is unchanged\" — and Rollout 9 — \"any `Cargo.lock` change is a finding\"",
    "problem": "[new] These contradict the run's own gate. `Cargo.lock` records every workspace member (stanzas for bindforge-core, bindforge-transforms, discovery, discovery-cli at Cargo.lock:56,72,141,151), so landing `crates/codecs` necessarily adds a `bindforge-codecs` stanza plus the `apps/discovery-cli` → `bindforge-codecs` edge, and Rollout 9's own `cargo test --workspace --locked` fails while the lock is stale. As written the design makes its gate unsatisfiable and forces the implementer to break a stated rule.",
    "fix": "Restate both places as the narrower claim the matrix row already makes: the only delta is the `bindforge-codecs` stanza and the `discovery-cli` dependency edge, with no new third-party package and no version moving; pin that instead."
  },
  {
    "severity": "major",
    "location": "The CLI face, step 5 — \"`EXIT_FAILURE` 1 (usage, undiscovered slug, scan failure — today's one voice) · `claim-refused` 2\"",
    "problem": "[new] Exit 1 is assigned to a class that never reaches this code: the binary parses through clap 4 (`Cli::parse()`, apps/discovery-cli/src/main.rs:73; `clap = { version = \"4\", features = [\"derive\"] }`, apps/discovery-cli/Cargo.toml:11), and clap exits 2 on a usage error before `main` runs. So `discovery bindings` with a missing `<slug>` or a misspelled flag exits 2 — the code reserved for `claim-refused` — breaking the section's own promise that \"the code is a function of what stdout showed\" and making Test strategy #13's (0/2/3/4) pins undecodable by consumers.",
    "fix": "State that the face parses with `Cli::try_parse()` and routes the parse error through the existing `error: ` + `ExitCode::FAILURE` voice (main.rs:80-83) so usage stays 1; or move the four status codes above clap's 2 and say so in the same sentence."
  },
  {
    "severity": "minor",
    "location": "The first family fixture list / Test strategy #6 — \"`symlinked-prefix`\" … \"a symlinked directory and a symlinked `.ini` in `symlinked-prefix` are neither descended nor read\"",
    "problem": "[new] The design never says how those links come to exist, and the likeliest route breaks the gate Rollout 9 promises: the Windows leg is `cargo check --workspace --exclude bindforge-registry --all-targets --target x86_64-pc-windows-msvc` (.github/workflows/check.yml:42-46), so `crates/codecs` test targets compile for that target, and a test-time `std::os::unix::fs::symlink` helper (this would be the repo's first symlink test — no existing file under crates/ calls `symlink`) does not compile there; a symlink checked into git is not a link in a Windows checkout.",
    "fix": "One sentence pinning the mechanism: links created at test time behind `#[cfg(unix)]` with a `#[cfg(not(unix))]` early return so the compile leg stays green, and the assertion runs on the Linux leg only."
  },
  {
    "severity": "minor",
    "location": "`bindings/v1` bounds — \"`user` one of the three spellings above\"",
    "problem": "[new] The contract never says whether the three labels are a closed set the loader refuses anything outside, or the set `v1` emits. Since `bindings/v1` ages through majors, this is a permanent commitment: `017-R2`'s Windows launcher context has a local user who is neither a SteamID nor `steamuser` nor `host`, so a closed enumeration forces either overloading `host` (losing the label) or a format major.",
    "fix": "One sentence deciding it: the loader validates `user` against a bounded token grammar (the shared slug-shaped bound plus the section's control-character and space rules) and spec.md lists the spellings `v1` emits; or keep the closed set and state that a new launcher-context label is a `bindings` major."
  },
  {
    "severity": "minor",
    "location": "Read discipline and budgets — the `ExtractionRefusal` variant list \"`FileOverBudget`, `UndecodableText`, `UnparsableEntry` … and `SnapshotEntryBudget`\" — against `bindings/v1` bounds' \"each input token the same bound\"",
    "problem": "[new] A located line whose verbatim `Command=` or `Name=` string exceeds `MAX_NATIVE_NAME_OCTETS` has no named outcome at the extraction seam: the format's over-bound refusal (\"a value outside its bound is a refusal naming the key\") governs a loaded document, while at extraction the family must drop the pair — which §Deferred forbids (\"`v1` reports the file's tokens verbatim and drops nothing\") — or refuse under a name the list does not carry, which `017-L1` forbids. UE's `Command` field is a free-form console command line, so this is real-file territory.",
    "fix": "Name the outcome: state that an over-bound verbatim token is an `UnparsableEntry` carrying the bound's name with the tree-relative path and 1-based line, or add one variant; the status stays `extraction-refused`."
  },
  {
    "severity": "minor",
    "location": "Read discipline and budgets — \"MAX_USER_DIRS_PER_GAME = 16 | `userdata/` directories considered per game\" with Test strategy #3's \"a 17th digit-named directory is past `MAX_USER_DIRS_PER_GAME` and unvisited\"",
    "problem": "[new] This is the only budget in the design that ends in silent omission instead of a named outcome: on a machine with more local Steam users the run emits fewer documents with no status, refusal or stderr line, so an operator cannot tell \"every user's bindings\" from \"the first sixteen\" — where every other budget here names itself (`FileOverBudget`, `SnapshotEntryBudget`, the candidate cap).",
    "fix": "One sentence deciding it: the cap is reported (a named line in the run's output, run still `ok`), or spec.md and the README budget bullet state it plainly as a silent bound on users per game."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix — Windows host row: \"no Windows tree template ships, so the face answers `claim-refused`\"",
    "problem": "[new] The stated reason misdescribes the behavior: `SteamUserdata` and `ProtonPrefix` are root-relative and OS-neutral, so on a Windows host with a discovered Steam row both templates are built and stat'd; `claim-refused` arrives because no `Saved/Config` grammar is located inside them, not because nothing resolves — the wrong failure model for whoever picks up `017-R2`.",
    "fix": "Restate as: the root-relative trees resolve; no template ships for the Windows host config home (`%LOCALAPPDATA%`, `Documents\\My Games`), so a real title answers `claim-refused`."
  },
  {
    "severity": "nit",
    "location": "The first family — \"collecting `*.ini` files whose path ends `Saved/Config/<one directory>/<name>.ini`\"",
    "problem": "[new] The design does not say the match is component-wise; a suffix test against `path.display()` is separator-dependent, which is exactly where `017-R2`'s Windows trees will bite.",
    "fix": "Add one clause: matched on `Path::components()` suffix components plus `file_name()` ending `.ini`, never on a displayed string."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/017-keybinding-extraction-codecs.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/compatibility-r1c1.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{"verdict":"approve","findings":[{"severity":"minor","location":"§Discovery plumbing + §GameContext — \"`roots()` (the selected root candidates, in candidate order)\" / \"`pub roots: Vec<PathBuf>,  // every root the scan resolved, candidate order`\"","problem":"[new] The order is false of the field the inherent wrapper would expose: `SteamCore` holds its roots through `canonical_paths` (crates/discovery/src/steam_common.rs:369-381), which canonicalizes (resolving symlink aliases such as `~/.steam/root`), drops entries that fail to canonicalize or lack a `steamapps/` directory, and returns a byte-sorted, de-duplicated set — not the candidate list. Test strategy #3 already says \"the byte-order union over `ctx.roots`\", so the design contradicts itself about the order an implementer must produce.","fix":"In both places state \"the canonicalized, de-duplicated roots that actually hold a `steamapps/`, in byte order (the set `SteamCore` retains, steam_common.rs:369-381)\" and drop \"candidate order\"; keep the existence-filter half as the reason \"a tree is used only if it exists\" is already true."},{"severity":"minor","location":"Compatibility matrix — \"Flatpak / Snap Steam | Yes, for the root-relative trees\" and \"Steam Deck | Yes, by the Linux row\"","problem":"[new] Both rows were upgraded to Yes this round and both rest on the new `roots()` seam, which no test reaches: #1 pins `ScanRecord.root`/`library`, #2 pins only that `from_record` carries the slice it was handed, #3 pins the union over `ctx.roots`. No test or fixture is shaped like a Flatpak/Snap or Deck root, so the upgraded commitments are covered only by a structural test of the generic mechanism (checklist item 7).","fix":"One clause in #1 or #2 — `roots()` returns exactly what `SteamCore::with_roots` kept — and one `ctx.roots` entry in #3 shaped `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` that yields a `SteamUserdata` tree. No discovery-side work is implied: the candidate list is already pinned at crates/discovery/tests/steam_linux_tests.rs:294-345."},{"severity":"minor","location":"§Read discipline and budgets — \"`MAX_USER_DIRS_PER_GAME = 16` | `userdata/` directories considered per game\" against §Config-path resolvers' \"`<root>/userdata/<dir>/<ctx.appid>` for every `root` in `ctx.roots`, one per digit-named `<dir>`, byte order, at most `MAX_USER_DIRS_PER_GAME`\"","problem":"[new] Making the template a union over roots leaves the cap's unit undecided — sixteen per game in total or per root — and the aggregate sentence counts only one set of sixteen (\"`(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` … (544 MiB)\"). Under the per-root reading, A3's nine Linux candidates plus any number of repeated `--steam-root` values put the worst case at `(|roots| × 16 + 1) × 32` files, several multiples of the figure `schemas/README.md`'s budget bullet is told to state.","fix":"One clause deciding it: the cap counts `userdata/` directories across all `ctx.roots` for one game (so the arithmetic stands), and say which directories are dropped when the union exceeds it. (Shared with the performance seat.)"},{"severity":"minor","location":"§Determinism + §The CLI face step 3 — \"across documents, deduplicated by (`user`, `provenance.files`) — two records that resolve the same tree state it once\"","problem":"[new] The key is sound for one game in two libraries, not for the roots union introduced in the same round: two distinct resolved roots can each carry `userdata/<same SteamID>/<appid>/` (native install plus a Flatpak or Snap Steam on one account), and since `files` are deliberately tree-relative (\"the portability rule: the same layout states the same document on any machine\"), those two trees share the key and one document is silently dropped — no refusal, no status, and no printed path that says which install answered.","fix":"Deduplicate at the `UserTree` level on the tree's absolute root path (which is exactly what the two-records case repeats) and let §The CLI face emit one document per distinct tree; same one-fact-stated-once property, without building document identity out of values the contract strips machine context from."},{"severity":"minor","location":"§Rollout item 9 — \"`cargo deny check advisories`, and `npm ci && npm run check` in `tools/orchestration`, per `.github/workflows/check.yml`\"","problem":"[new] That leg is not one of the run's cargo gates: the workflow runs it as the separate ADR-027 `advisory-gate` job, which builds its own `cargo-deny` 0.20.2 from source on toolchain 1.88.0 inside a 30-minute budget precisely because it is \"too heavy for the 15-minute `rust` job\" (check.yml:83-126). A22's own enumeration of CI (\"the cargo trio plus a Windows *compile* check … and the Node check\") does not count it, and this run adds no external package for it to screen; listing it inside the run's gate set imports a tool install no other part of the design owns.","fix":"Either cite it as what it is — \"the `advisory-gate` job, its own pinned scanner and toolchain; nothing here adds a package for it\" — or keep A22's list; the same line could also name the `cargo check -p bindforge-registry --all-targets` step the `rust` job does run."},{"severity":"minor","location":"§Rollout item 8 (Ledger) — \"the units whose shipped reading differs from their wording (`017-C1`'s bound and `extract` return shape, `017-C3`'s `inputs`, `017-C4`'s 'same scan', `017-S1`'s launcher contexts)\"","problem":"[new] `017-C2` is missing, although the Revision log already records a divergence from it (\"§GameContext — `roots` is `017-C2`'s whole resolved set … not a 'main' one\") and the ADR clause reads \"every steam root the scan resolved — main root and library roots separately, because `userdata/` lives only under the main root\". What ships is one undifferentiated `roots` vector with none labelled main, i.e. a shipped reading that differs from the wording, and this hand-off is the only mechanism that keeps the operator-owned `docs/adrs/` accurate.","fix":"Add `017-C2`'s main/library separation to the hand-off list and to the \"Five clauses\" count in §Context and the matching Risks row."},{"severity":"nit","location":"§Goals — \"`unreal-ini` ships with fixture trees for the two launcher contexts whose roots a record can justify\"","problem":"[new] ADR-017's Terminology fixes a launcher context as \"the native host layout, or the Proton prefix the game was launched under\", so after this round's removal of the host template the two shipped trees (`steam-userdata`, `proton-prefix`) are not the ADR's two contexts, and the phrase hides that the native context is exactly what moved to §Deferred.","fix":"Reword to \"the two root-relative trees a record can justify\", with the context named where §Deferred already names it."},{"severity":"nit","location":"Compatibility matrix, Steam Deck row — \"A Linux Steam install with the `$HOME` and Proton prefix layout the two rows above claim\"","problem":"[new] Nothing hangs off `$HOME` any more (§Config-path resolvers: \"each built from a `GameContext` anchor and nothing else — no resolver reads the process environment\"); the resolved Steam roots merely live under it, so the row describes a layout the design no longer claims.","fix":"\"the root-relative `userdata/` and Proton prefix layout the Linux and Proton rows claim; unexercised here — no Deck leg (A22) and no Deck fixture\"."}]}
```
