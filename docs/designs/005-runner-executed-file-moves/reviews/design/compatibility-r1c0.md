# compatibility review r1

verdict: request-changes

Checklist applied: matrix statuses, OS input APIs, Node floor, native modules,
keyboard reality, file-path handling, test-strategy platform coverage. Node
floor checked against `tools/orchestration/package.json` `engines: >=22.12.0` —
the proposal's "Node ≥ 22.12" row is accurate and every API it introduces
(`fs.renameSync`, `existsSync`, `realpath`) predates the floor. No new native
module; the design reuses the existing `spawnSync('git', …)` helper. OS input
APIs and keyboard reality: no input surface in this design, so nothing to
target or justify (see FYI).

## Findings

- [major][new] Compatibility matrix — "| macOS (APFS, case-insensitive default) | unknown | suite expected to pass; case-only moves rejected by the disk check — unverified (A10) |" — the row carries no decision. `unknown` is not a status a test strategy or an operator can act on, and the note hedges twice ("expected to pass", "unverified"). Fix: replace `unknown` with a committed status and back it with a named check — either "yes (unverified)" with Test strategy #12 made concrete for macOS (run one move round trip and one case-only move, and record the observed rejection as the A10 evidence), or "not exercised" with the note stating the runner is not run on macOS. Same wording budget as the existing "yes"/"partial" rows.

- [major][new] Compatibility matrix — "| Windows (NTFS, `core.protectNTFS`) | unknown | never exercised in this repo's pipeline; name-legality rules untested |" — same missing decision, and this row names a gap ("name-legality rules untested") that no validator rule and no Test strategy case addresses, so the row cannot resolve itself. Fix: pick the status as above and make #12 name the two Windows observations that matter to this design — a move round trip, and what happens to a `\`-separated or Windows-illegal `to` (see the two rule-1 findings below). If the answer is "no one will run this pipeline on Windows", say that in the Notes cell and drop the hedge.

- [minor][new] Chosen design / Validator rule 1 — "repo-relative POSIX paths, normalized, no absolute path, no `.` or `..` segments" — "normalized" is undefined, and every downstream check assumes `/`: rule 4 builds the revspec `<headBefore>:<from>`, the new `.git` rule is the string test "path === '.git' || path.startsWith('.git/'", and `stagePathAllowed` itself is regex-on-`/` (gates.ts:253,260). A reply that emits `src\old.ts` has no `/`-separated `..` segments and no absolute-path marker, so it clears rule 1, then fails rule 4 (or, for a `.git\config` endpoint, escapes the `.git` string test and only fails rule 4 by accident), reaching the model as a confusing "not tracked at headBefore". The repo already normalizes in the other direction (`relative(repoDir, doc).split(/[\\/]/).join('/')`, runner.ts:134), so the inverse rule is the missing half. Fix: define rule 1's normalization in one clause — reject any `\` in either endpoint, and compare with `path.posix` segment semantics — and pin it with one case in Test strategy #2's rejection list.

- [minor][new] Chosen design / Validator rule 1 — the shape list covers `.`/`..`/absolute but no per-OS name legality, while the Windows row itself says "name-legality rules untested". A pair with a Windows-reserved component (`CON`, `NUL`, a trailing dot or space) or a `:` in a path validates and commits cleanly on the reference Linux platform; `:` additionally corrupts the `<headBefore>:<from>` revspec in rule 4, so it can fail in a way that looks like a git error rather than a validator rejection. Fix: extend the same rule-1 sentence with the reserved/illegal-name check (one regex on each segment) and one Test strategy #2 case, or state in the Windows Notes cell that name legality is operator-checked — currently the text says it is untested without deciding which of those is true.

- [minor][new] Chosen design / Validator rule 6 — "all `from`s distinct, all `to`s distinct" and "so the list is a matching on disjoint paths and apply order is irrelevant" — distinctness is string-distinct, which is not disjointness on a case-insensitive checkout, the exact platform class the macOS/Windows rows name. Two `to`s differing only in case both pass rules 5 and 6, then the second pair's per-pair "`to` absent" re-check trips in `applyMoves` and the whole list rolls back as `moves-apply-failed` — safe, but the model pays an environmental-class failure with a fresh session instead of the cheap in-attempt nudge, and the "apply order is irrelevant" claim is false as written. Fix: in rule 6, compare endpoint sets case-folded when the checkout is case-insensitive (the same `existsSync` probe rule 5 already uses), or state the unconditional rule: reject case-insensitively colliding endpoints everywhere.

- [nit][new] Compatibility matrix / Test strategy — "| Repository spanning mount points | partial | `rename(2)` fails `EXDEV` (A8): the pair fails at apply, the list rolls back…" — the row describes a specific path through `applyMoves` that no test names; #5 and #11 exercise a generic mid-list failure, not the cross-mount case. Fix: name it in #5 (a pair whose `to` parent sits on a different mount, e.g. a tmpfs under the worktree) so the "partial" status has an observation behind it.

- [nit][new] Compatibility matrix — "Node ≥ 22.12 (tools/orchestration/package.json:30-32) | yes | no new runtime dependency" — the matrix pins the runtime floor but not the git binary floor, which this design newly leans on for `git status --no-optional-locks*` (git ≥ 2.19), `git cat-file -t`, and default-on rename detection. Fix: add one matrix row, "git ≥ <floor>", or one clause in the Node row naming the floor the pipeline assumes.

## FYI (no severity)

- Checklist items 2 (X11/Wayland/Raw Input/HID) and 5 (layouts, dead keys, IME, non-ASCII binding names) are evaluated as not applicable: the design introduces no input surface, so no per-platform input API is targeted and no abstraction needs justifying.
- Non-move compatibility is handled well: `moves` defaults to `[]` with `version` unchanged (Rollout 2), so old and in-flight checkpoints re-parse, and the feature is inert until a stage emits a pair — that matches ADR-005's additive-field consequence.
- Deferred items I checked against ADR-005 and left closed: the runner's git-env hardening (`GIT_CONFIG_NOSYSTEM`/`GIT_CONFIG_GLOBAL`) and case-only rename support do not contradict the ADR's Decision or item 5.
- For the security seat, from a platform angle only: the new `.git` deny matches the repo-root `.git` alone; in a checkout containing submodules or a nested repo, `<subdir>/.git/` is inside the writable root and unmatched. No `.gitmodules` exists in this repo today, so the linked-worktree row (A11) is the only shape the matrix covers.
- For the correctness seat: ADR-005 item 5 says stage shells allow `git status`; the pinned entry "git status --no-optional-locks*" is a prefix pattern that does not match bare `git status`, which is the exact invocation the ADR's friction story names.
