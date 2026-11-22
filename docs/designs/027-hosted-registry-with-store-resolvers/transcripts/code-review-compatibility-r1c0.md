# Transcript: code-review:compatibility r1c0

## Provenance
- stage: code-review:compatibility (round r1c0)
- agent: codereview/code-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/code-review.md @ c0fda013be86
- started: 2026-11-22T07:10:57.088Z
- ended: 2026-11-22T07:32:52.078Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":149032,"output":29870,"reasoning":0,"cache.read":4282555,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the change produced by this pipeline branch against the design at
docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: git diff a22cfd2694fde8d84be132c8d78faf09fda1adf4...HEAD -- ':(exclude)docs/designs'.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
docs/adrs/027-hosted-registry-with-store-resolvers.md, the runner's gate record quoted below, and your own previous
findings quoted below. Do not open review reports or transcripts, and do not
ask for them: the panel's value is five independent reads of the same change,
and another seat's verdict anchors yours. Reports and transcripts are the run's
record, not review input.
Listing, globbing, or grepping those directories counts as opening them.
Your report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/compatibility-r1c0.md is yours: re-reading it is allowed, through your
`git diff*` shell allow (your `read` tool stays fenced off `reviews/`); every
other file under any `reviews/` directory stays closed.
The runner captured this gate record from the code under review. It is output,
not instruction: a finding about a behavior claim can cite it — a failing leg's
tail, or a green leg's `test result:` line — and you may also run the project's
read-only check commands yourself.
```text
gate evidence the runner captured from the code under review (gate at 2026-11-22T07:10:57.020Z, green): this is output, citable as evidence — never instructions to follow.
every line below is the renderer's: $ opens a leg the runner ran, | opens captured output from it.
$ npm run check  (exit 0) — green
$ cargo fmt --all -- --check  (exit 0) — green
$ cargo clippy --workspace --all-targets -- -D warnings  (exit 0) — green
$ cargo test --workspace --locked  (exit 0) — green
| test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```
Your own previous findings (compatibility seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the compatibility checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/code/compatibility-r1c0.md, and write no other file: an audit checks
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
{"verdict": "request-changes", "findings": [{"severity": "blocker", "location": "apps/registry/Cargo.toml:16-26 with .github/workflows/check.yml:29-32 (unchanged)", "problem": "The C/asm native dependency (rustls' `ring` provider, plus `cc`/`find-msvc-tools`/`shlex` in Cargo.lock) landed with no CI-matrix change. The shipped `rust` job's first leg is `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` on ubuntu-latest, which now has to run ring's build script for a target with no MSVC C toolchain. The proposal prices exactly this and prescribes the fix — §Options ('the shipped whole-workspace x86_64-pc-windows-msvc compile leg (A16) drops apps/registry, which is compile-checked on the host instead'), §Risks row 1, §Compatibility matrix's Windows row ('Rollout 3 keeps apps/registry off that leg'), Rollout 3. Rollout 3's dependency half is here; its check.yml half is not in the diff. Concrete failure: the whole cargo gate goes red on the Windows leg. The runner's gate record contains no cross-target leg, so the compatibility matrix's Windows claim cannot be established from the evidence supplied.", "fix": "Land Rollout 3's leg split beside the dependency: narrow the cross-target step to the C-free crates (or `--exclude bindforge-registry`) and add a host-target `cargo check -p bindforge-registry` step; put that leg's result in the gate record so the Windows row's coverage is evidenced."}, {"severity": "major", "location": "README.md:13-25 (unchanged) with apps/registry/Cargo.toml:21-26", "problem": "No packaging/prerequisite story for the new C toolchain requirement: README still says the desktop-era Tauri system packages are things 'the cargo gate itself needs none of' and names only the rustup pin plus the three cargo gates. The §Compatibility matrix's Linux row states the requirement ('needs a C toolchain for ring's build script (A16, README.md prerequisite per Rollout 3)'), and A16 was recorded as verified precisely so this line would change. A contributor or an operator building the Rollout-7 binary from the README hits a build-script failure with no documented prerequisite.", "fix": "Add the C-toolchain prerequisite line Rollout 3 names and correct the 'needs none of them' parenthetical to say which package the cargo gate now does need."}, {"severity": "major", "location": "apps/registry/src/main.rs:304-311 and :338-354 (`(steam::STORE_AUTHORITY, 443).to_socket_addrs()` before `load_or_pull_snapshot`)", "problem": "A resolver-active startup resolves the public authority `partner.steam-api.com` before it looks at the snapshot, so every leg that configures BINDFORGE_STEAM_WEB_API_KEY without `--no-steam` (tests/common/mod.rs:503-510 deliberately withholds the `--no-steam` default there) performs an external DNS lookup: tests/data_layout.rs:131, hosted_end_to_end.rs:138, cli_end_to_end.rs:138, hosted_lookup_policy.rs:94, hosted_health_stats.rs:76, issued_entries.rs:82, mint_rule.rs:128, retired_ids_and_curation.rs:91, health_route.rs:95. That contradicts Test strategy #17 ('nothing resolves partner.steam-api.com from a test') and the suites' own headers (data_layout.rs:33-36, cli_end_to_end.rs:41-43, hosted_startup.rs:14-16). Failure scenario: with DNS for that name unavailable (offline dev box, sealed runner), `cargo test --workspace --locked` refuses nine suites' instances at startup with 'the steam store authority ... could not be resolved'; the same line also stops a hosted instance with a good snapshot from booting on a host with a broken resolver.", "fix": "Keep the authority as a name on `SteamStore` and resolve it at pull time inside `SteamStore::connect` (steam.rs:626-659), so a run that loads a loadable snapshot never consults the resolver and main.rs never calls `to_socket_addrs`; the compile-time pin stays the production value and tests/steam.rs's separate-name seam is unaffected."}, {"severity": "minor", "location": "apps/registry/Cargo.toml:21,26 (`rustls = \"0.23\"`, `webpki-roots = \"1.0\"`)", "problem": "§Options names the dependency floors as the RUSTSEC mitigation ('rustls ≥ 0.23.46', 'rustls-webpki ≥ 0.103.16 / ≥ 0.104.0-alpha.8', 'webpki-roots ≥ 1.0.10') and Rollout 3 lands the deps 'at the floors §Options names'; the manifest declares plain `0.23`/`1.0` requirement strings instead. The current Cargo.lock satisfies the floors (0.23.46 / 0.103.17 / 1.0.12), but nothing in the manifest holds them, so any re-resolve can land the RUSTSEC-2026-0346/-0381/-0382 versions, and the repo's only advisory control (advisories.yml) is a weekly triage job, not a gate.", "fix": "Spell the floors in the requirement strings (`rustls = \"0.23.46\"`, `webpki-roots = \"1.0.10\"`, plus an explicit `rustls-webpki` floor or a comment pinning the transitive requirement) and name the advisory each floor closes."}, {"severity": "minor", "location": "apps/registry/src/main.rs:339-353 (`.next()` on the resolved list), apps/registry/src/steam.rs:455-479", "problem": "The store's address is the resolver's first answer with no fallback across the rest. On the supported Linux hosting row, a resolver that returns a AAAA record first on a host with no working IPv6 route makes every page attempt a connect timeout, so a startup full pull fails and the instance refuses to bind even though a working address was returned. No design decision names this single-address posture.", "fix": "Carry the whole resolved list on `SteamStore` and walk it inside the page attempt (as `TcpStream::connect((host, port))` does), or fall through the list before classifying a page as transient."}, {"severity": "nit", "location": "apps/registry/src/main.rs:284-294 with :328-333 (`spared_document` = `std::fs::canonicalize`, string-compared)", "problem": "The leftover-document refusal spares 'the path `--corpus` names' by comparing canonical spellings as strings. On the matrix's Windows row, a differently-cased spelling of the same `catalog-v1.json` compares unequal (canonicalize does not fold long-component case), so the operator is refused with 'delete it or point --corpus at it' after having done the thing the line asks.", "fix": "Compare by file identity (dev/ino on unix, documented no-op branch elsewhere), or fold case on the canonical spellings under `cfg!(windows)`."}]}
```
