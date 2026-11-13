# compatibility review r1 c2

verdict: approve

Ratification pass. Scope: the r2 revisions (`git diff 15ba04f..0b13556` of
`02-proposal.md`) plus the full current document. Inputs: the proposal, ADR
020, the Revision log, and this seat's r1 findings.

## Both r1 findings are closed, and the closing text is true against the code

- A7 / matrix row / Test #2 — verified. `loadState` runs
  `StateSchema.safeParse(parsed)` and then `return parsed as PipelineState`
  (state-store.ts:239–256), so the raw document, unknown key included, is
  what the caller holds; `saveState` writes
  `JSON.stringify(state, null, 2)` of exactly the state handed in
  (state-store.ts:365) and the write path snapshots with
  `structuredClone(state)` (state-store.ts:411), so "the key round-trips
  through its later saves" and "loads it with the key inert" are both
  accurate. `StateSchema` is a plain `z.object` (state-store.ts:80), so an
  unknown sibling key validates. Test #2's new clause — "parses and loads
  with it inert (A7)" — now asserts something the current loader passes.
- Chosen design §1 — verified. The named form
  `git diff -z --no-renames --name-only <headBefore>..HEAD` runs (checked
  read-only against this repo: NUL-separated, repo-root-relative paths), the
  parser it is piped through exists (`nulPaths`, runner.ts:262 — the NUL
  splitter for status-prefix-free listings, already used with
  `diff-tree -r --no-commit-id -z --name-only` at runner.ts:308), and
  `--no-renames` matches the sweep's own posture
  (`status --porcelain=v1 -z ... --no-renames`, runner.ts:268). The predicate
  contract it feeds (gates.ts:554–557: "repo-root-relative, POSIX
  separators, the real spelling") is now actually satisfied on non-ASCII
  paths, and the Node-floor row's "no new primitive" claim still holds —
  `-z`/`--no-renames` are git flags, not a new runtime dependency.
- The other changed lines ("tolerating one unknown key", "the record inert to
  it", §1's pointer to Test #7, Options' "§1's net-diff listing", the two r2
  Revision-log entries) move no mechanism and contradict nothing; they are
  consistent with ADR 020 Decisions 1–3 and its "Not decided here" list.

## Findings

- [nit][new] Chosen design 2 — "the record keeps the `boundedGateEvidence`
  bounding (first N paths plus the count)" vs the field the same section
  declares, "`restartRequired?: { stage: string; paths: string[]; at: number
  }`", and vs Test #3's "naming the stage and the hit paths". The cited
  mechanism bounds a different shape: `boundedGateEvidence` (runner.ts:3057–
  3075) slices `evidence.legs` at `MAX_GATE_LEGS` and puts the count in an
  extra pseudo-leg (`'(gate legs elided)'`), and `boundedFindings` uses the
  same sentinel-in-array shape. `paths: string[]` has no member to hold a
  count, so as written the sentence either re-types the field §2 just
  declared (which is what the grammar guards against, A6) or leaves the count
  nowhere; and once the record is bounded, Test #3's "the hit paths" is true
  only up to the cap, while §2's earlier "commit whose body names the changed
  paths" and §3's "how many harness files it changed" read the record as
  complete. Fix: say where the bound lands — smallest form: bound only the
  printed copies (gate-log entry, commit body, status line, summary) and keep
  the record's `paths` the full hit list the predicate produced; otherwise
  add the count as a named member of the §2 field (or spell the elision entry
  exactly as `boundedFindings` does) and make Test #3 assert that bounded
  shape rather than "the hit paths".

## FYI (no severity)

- The r2 "every copy of a path takes `printable`" posture
  (independence.ts:443 — control/escape bytes → space, non-ASCII preserved)
  means the record, gate-log, commit-body and summary carry sanitized path
  strings, not the plumbing spelling §1's predicate consumes. Nothing in the
  design reads record paths back as paths (they are only printed), so
  compatibility sees no break; whether the copy should stay byte-exact for
  later tooling is the security seat's call.

## Checklist coverage

Matrix rows all carry a status (no `unknown`, no empty cell); the platform
rows are unchanged from r1 and Test strategy still names only the Linux suite,
which is honest against the partial/no macOS and Windows rows; Node floor
unchanged; no native modules; no OS input APIs in scope for this change; path
spelling handled by git plumbing plus `nulPaths`, not string concatenation.
All evaluable.
