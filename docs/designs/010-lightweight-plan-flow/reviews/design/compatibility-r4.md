# compatibility review r4

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r4.diff` (7 hunks: Context, Goals,
Budget resolution, Ownership, Rollout 6, two appended Editor notes), read against
`02-proposal.md`, `docs/adrs/010-lightweight-plan-flow.md` and the Revision log. No
prior compatibility findings were open for this seat, so the bar is: does the changed
text break a compatibility claim the platform story rests on? It does not. Every
changed line is language-only in the compatibility sense; no row of the
Compatibility matrix, no `Test strategy` platform item, no `engines`-floor claim, and
no path-separator/case claim is altered by the diff.

Checklist, evaluated:
1. **Matrix completeness** — all 12 rows carry a status; no empty cell, no `unknown`.
   Rows 1–2 pin the `engines` floor (`>=22.23.4`, A20), rows 3–7 pin checkpoint,
   document, brief and ADR compatibility, rows 8–11 pin CRLF and the three
   `--max-parallel` shapes, row 12 reads `no` for Windows/macOS. Untouched by the diff.
2. **OS APIs** — the design adds no input, display, or OS API surface; A20 plus the
   Windows/macOS row's "this design adds no platform-specific path, claims no support
   there" is the justification the checklist asks for. Applicable-by-abstraction, satisfied.
3. **Node floor** — the named floor is stated once (A20, matrix row 1) and no API
   above it is used anywhere in the diff's changed text; the only computation the
   changed text names is the JS line count at `runner.ts:179-180`.
4. **Native modules** — pure TS in `tools/orchestration`; the matrix row 12 note
   states the meter is a JS count "not a shell-out, so nothing here adds a Unix
   dependency", which closes the silent-native-dependency case. No prebuild/node-gyp
   story needed.
5. **Keyboard reality** — not applicable: no binding, key, layout, dead-key or IME
   surface is in scope. Not a finding.
6. **File paths** — Ownership: paths arrive only from git plumbing (A28),
   repo-root-relative with POSIX separators, absolute/backslash/`.`/`..` shapes
   rejected as usage errors by `moves.ts` Rule 1, case deliberately left sensitive and
   pinned by Test strategy #7/#18. The diff's only change here inserts one word
   ("no absolute" → "no absolute path"); the claim and its enforcement site are
   unchanged.
7. **Platform test per supported platform** — the matrix claims one supported
   platform (linux at the `engines` floor) and Test strategy #21 names exactly that
   platform test, while stating that no macOS or Windows job exists, so those rows
   are not claimed by any test. Consistent.

Deferred section: the "Windows or macOS CI job" deferral was checked against the ADR
as the rule requires. ADR-010's Decision, Test plan and Consequences name no
Windows/macOS support and no CI matrix; the proposal's `no` rows do not contradict any
clause. Not re-opened.

## Findings

None. No blocker, major, minor, or nit.

## FYI (no severity)
- Revision log — the two trailing lines "Editor notes: Budget resolution — …
  author to clarify." are unresolved editorial requests, not `rN:` decision entries,
  sitting in the shipped document. The editorial/proposal-editing seat should decide
  whether they are answered and removed before the run closes.
- Same note: the first Editor note's own claim checks out — Risks row 4 is the
  resume budget-drift row, and the editorial-round exposure of `DESIGN_SPECIALTIES`
  is row 1 — so the clarification the editor asks for is a cross-reference fix, not a
  design change. Not a compatibility matter.
