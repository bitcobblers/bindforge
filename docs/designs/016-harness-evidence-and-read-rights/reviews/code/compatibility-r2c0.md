# Code review — compatibility — round 2, pass c0

Seat: compatibility (full panel, round 2, pass c0).
Diff under review: `git diff 22ce12bf09a600d3983f374294d9d5844b07af54...HEAD -- ':(exclude)docs/designs'`
(47 files, +3957/-78). Inputs: that diff, `02-proposal.md`, `docs/adrs/016-harness-evidence-and-read-rights.md`,
the runner's gate record (none captured for this run), this seat's previous findings (none — first pass).
Checklist: `.opencode/skills/code-review/references/compatibility.md` (7 items) — only that file applied.

## Verdict

**approve** — no blocker or major finding. Four findings, all minor/nit: the Node floor bump left two
floor authorities in the repo stating the old version, one platform leg the plan names is not delivered
by any shipped test, the green-at-the-new-floor claim has no executed evidence in this run, and one new
test leg is POSIX-only and unguarded.

## Checklist, item by item

**1. Platform code vs the Compatibility matrix — pass.**
The diff adds no OS branch anywhere in `tools/orchestration/src/`: the capture (`gates.ts` `runLeg`,
`MAX_TAIL_BYTES` rolling window, `boundedTail`) and the record (`types.ts` `GateLeg`/`GateEvidence`,
`state-store.ts` `GateLegSchema`) are plain portable Node. The matrix's two `not exercised` rows
(macOS, Windows/WSL2) name exactly the platform-dependent surfaces the diff adds — §3's POSIX inspection
set in the agent configs and the design panel's spawned `cargo`/`rustfmt`/`npm` legs — and the diff ships
no OS guard for them, which is what "not exercised" licenses; no claim is made for those hosts, so no code
path is owed there. Every `yes` row has a code path: pre-change checkpoints
(`state-store.ts:103` `gateEvidence` `.optional()`, pinned by `gate-evidence-checkpoint.test.ts` against
the golden fixtures), in-flight run (`gateEvidenceText` stand-in + `NO_GATE_EVIDENCE` default in
`prompts.ts` `TEMPLATE_DEFAULTS`), full flow (`FLOWS.full.config` `docMaxLines: 900`, `docStages` + `research`),
light flow (absent cap → `documentOutgrewFlow` allows; `docFileFor('plan')` keeps `02-proposal.md`;
`planner.md` widened), pre-ADR-016 server config (only `shell`/`glob`/`grep` rule types appear in the diff;
no `policies` overlay). Windows spawn posture is unchanged in kind: `spawn('npm', …)` resolved through
`PATH` failed before (`stdio:'ignore'` → `resolve(false)`) and fails now (`exit: null` + "spawn failed" in
the tail), so the bump introduces no new Windows break.

**2. Node floor — finding 1 (minor).** `tools/orchestration/package.json:31` is now `>=22.23.5` and
`.github/workflows/check.yml:50` moved to `"22.23.5"` in lockstep, and `crates/core/tests/gate_contract.rs:304-322`
cross-checks those two files, so the two cannot drift — the CI comment's claim ("the workspace gate contract
pins this string to that floor") is true. Nothing in the diff uses an API above the floor: newest surfaces
are `Array.from(string)` code-point bounding, `Buffer.concat`/`toString('utf8')`, `Object.entries`,
`Number.isFinite`, `String.prototype.trimEnd`, zod `.int().nullable()` — all far below 22.x. The finding is
the two floor statements the diff left behind (below).

**3. Paths — pass.** No new path is built by concatenation: `documentSize` uses `join(designDir, docName)`
with `docFileFor` naming the doc; test fixtures build every path with `join`; `runLeg`'s `command` is a
joined command line, not a path. Case sensitivity is flagged deliberately, not silently: `canonicalPath`
(independence.ts, new) folds `\`→`/` and case, which is the normalization the proposal's A17 registers from
the permission layer, and `isOwnReport` scopes the exemption by design-directory prefix; the two directions
that cannot be proven equal (`own` absolute + access repo-relative) fail closed — the safe direction, and
the runner hands `reportPath` repo-relative anyway.

**4. Guarded platform imports — pass.** No platform-only module is imported anywhere in the diff
(`node:child_process` was already imported in `gates.ts`; the change adds only the `ChildProcess` type).

**5. Locale/layout reality — pass.** The tail is bounded in code points (`Array.from(text)`), never UTF-16
units, and `gateEvidenceText`'s final clamp is `Array.from(block).slice(...)`, so no stored or rendered
string can hold a lone surrogate; the decode is `Buffer…toString('utf8')` (replacement characters for
invalid bytes, pinned by the `badutf` fixture). `printable` (`independence.ts:418`, `\p{Cc}` → space) keeps
every printable non-ASCII character, so a failing test naming a non-ASCII binding or id survives into the
checkpoint, the prompt block and the build note. Worst case for the new checkpoint field is
33 legs × 2 000 chars ≈ 66 k chars (≤ ~264 KB encoded), orders of magnitude under `MAX_CHECKPOINT_BYTES`.

**6. Native dependencies — pass.** The diff introduces no dependency of any kind; the only runtime
prerequisite is the `engines.node` bump, which is stated in Rollout 2, mirrored in the CI job, and pinned
from both sides (`flows.test.ts:361-367`, `gate_contract.rs`).

**7. Tests named in the plan run where they claim — findings 2, 3, 4.** Test strategy #1–#13 are each
delivered by a named file (capture, checkpoint grammar, audit identity, agent-shell matrix, design-seat
invariants, `documentOutgrewFlow`/`validateConfig`/`documentSize`, prompt render, transcript tokens, red-gate
round trip, green-gate seat render, own-report seat, oversized document, legacy resume) and none carries a
skip; #14's platform half is the Linux CI posture the matrix claims. Its second clause is not delivered.

## Findings

### 1. minor — README and the `repo-practices` skill still state the floor the diff raised away from

Location: `README.md:24`; `.opencode/skills/repo-practices/SKILL.md:79-80`
(against `tools/orchestration/package.json:31` and `.github/workflows/check.yml:50` in this diff)

Problem: new — checklist item 2 names README as a floor authority. The diff raises the floor to
`>=22.23.5` and moves the CI pin with it, but two other statements of the same floor were left at
`22.23.4`: README's gate paragraph ("`npm ci && npm run check` in `tools/orchestration` (Node 22.23.4)")
and the `repo-practices` skill's "Node floor for the orchestration gate: 22.23.4
(`tools/orchestration` `engines.node`)". Neither copy is checked by anything: the two machine pins that
exist (`gate_contract.rs:304-322` compares `package.json` with `check.yml`; `flows.test.ts:361-367` pins the
`engines` literal) never read README or the skill. The skill copy is the live
trap — every `coder`/`test-engineer` stage loads `repo-practices`, so a later stage that follows it and
restores `22.23.4` in the workflow turns the cargo gate red on a test outside this design's surface.
(Out of specialty FYI: `docs/workflows.md` gained three ADR-016 lines in this diff but names no Node floor,
so it did not need the same touch.)

Fix: in this branch, change `README.md:24` and `.opencode/skills/repo-practices/SKILL.md:80` to `22.23.5`,
so the floor reads the same in all four places; optionally name all four in the same sentence in
`docs/workflows.md`'s gates paragraph so the next bump has one checklist to update.

### 2. minor — Test strategy #14's "cargo legs run against the real workspace" leg is not delivered

Location: `tools/orchestration/tests/pipeline/gate-evidence-integration.test.ts:60-95`
(`tmpGateRepo`: no `Cargo.toml`), `own-report-seat.test.ts:63-84`, `document-cap-integration.test.ts:52-81`,
with `gate-evidence-capture.test.ts:205-247`; claim at `02-proposal.md` Test strategy #14

Problem: new — #14 says "the cargo legs run against the real workspace in the integration tests that need a
green gate". No shipped test does that: all three fixture repos are `mkdtempSync` trees with a `package.json`
check script and no `Cargo.toml`, so `hasCargoWorkspace` is false and `checkGate` never spawns a cargo leg
there — the persisted record in the red/green round trips contains only `npm run check` legs. The only cargo
capture in the suite is `gate-evidence-capture.test.ts`, which stubs `cargo` on `PATH`. Consequence for this
checklist item: A18's premise — that a `test result:` line sitting at the *end* of real `cargo test` output
survives `MAX_TAIL_BYTES` dropping and the 2 000-char tail bound, and that the green-leg render finds it — is
pinned only against a two-line synthetic stub, never against the workspace's own output.

Fix: one integration leg aimed at a fixture workspace (a `Cargo.toml` with `[workspace] members = []` plus a
trivial member crate, or the repo's own workspace behind a `checkGate(realRepoDir)` case the CI job already
pays for) asserting a `cargo test --workspace --locked` leg with a real `test result:` line in its tail;
or amend #14 to state the shipped posture — cargo legs stubbed at unit level, real-workspace proof delegated
to the CI job — and register it in the Deferred list.

### 3. minor — green at the raised floor is unevaluated in this run

Location: Test strategy #14 + `.github/workflows/check.yml:50` (`node-version: "22.23.5"`);
`02-proposal.md` Compatibility matrix, Linux row ("`tools/orchestration` Node at the `engines.node` floor")

Problem: new — the skill counts an unevaluated checklist item as a finding, not a pass. This run carries no
gate record ("(no gate record for this run yet)"), and this seat's shell is limited to `git diff`, so no
`test result:` line exists anywhere in this review's inputs: whether the nine new and four extended vitest
files and the existing suite are green under the *new* floor the diff sets is asserted by the matrix and by Rollout 2 but
not evidenced. (This is precisely the "unevaluable" class Decision 2 exists to end, so the change is the one
place it should not survive.) The static half holds: nothing in the diff is version-sensitive, and the CI
orchestration job runs `npm run check` at exactly that pin.

Fix: run `npm run check` in `tools/orchestration` at Node 22.23.5 and put the leg's command, exit and
`test result:` line into the new `03-build-notes.md` "Gate posture" section this diff's own prompts create —
or state in that section that CI's orchestration job is the standing proof, quoting the pinned `node-version`.

### 4. nit — the POSIX-only cargo stub leg carries no platform guard

Location: `tools/orchestration/tests/pipeline/gate-evidence-capture.test.ts:218-247`
(`#!/bin/sh` script written with `mode: 0o755`, `PATH` prefixed), with `:166-178` (`PATH` swapped to a
non-existent bin dir)

Problem: new — the suite's established pattern for a host-specific leg is an explicit guard
(`moves.test.ts:704`, `:753`, `:765` use `describe.runIf(process.platform …)`); this one has none. The matrix
lists macOS and Windows/WSL2 as `not exercised`, so no support claim breaks, but a contributor on an
unclaimed host gets a red `npm run check` from a shebang script that cannot execute there rather than the
suite's usual skipped-and-labelled leg.

Fix: wrap the stub-cargo case (and the `PATH`-shadowing spawn-failure case) in
`describe.runIf(process.platform !== 'win32')` with a one-line note pointing at the matrix's `not exercised`
rows, matching `moves.test.ts`'s spelling.

## Out-of-specialty FYI (no severity, not verdict-bearing)

- `gates.ts` `boundedTail`: the elision note's count is `chars.length - (MAX_GATE_TAIL_CHARS - 64)` while the
  span actually kept is `MAX_GATE_TAIL_CHARS - note.length`, so the note overstates how much was cut (e.g.
  "65 characters elided" where 43 were).
- `printable` at write turns tail newlines into spaces, so the tail reaches `gateEvidenceText` as one line and
  the green-leg extraction `/test result:[^\n]*/` runs to the end of the tail rather than one line — and
  `docs/workflows.md`'s new table row ("each leg, exit, bounded tail") describes more than the build note's
  line-selective citation actually carries.

## Re-read note

This seat's own report was not re-read beyond this pass's writing; no other seat's report or transcript was
listed, read, globbed or grepped.
