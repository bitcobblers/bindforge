# compatibility review r1

verdict: approve

Scope: whole document (02-proposal.md), against docs/adrs/004-mirror-src-hierarchy-in-tests.md.
Revision log read first: r1 records no prior findings, so every finding below is `new`
and none re-opens a rejected decision.

Checklist coverage (all items evaluated):

1. **Matrix rows** — all four rows carry a status; no empty cell, no literal `unknown`.
   I did *not* score the two `Expected — assumption` rows as `unknown` under item 1: the
   change introduces no platform-specific code, and the hedge is explicit and tied to a
   numbered unverified assumption (A16) rather than to a gap in reasoning. Item 1 passes.
2. **OS input APIs (X11 / Wayland / Raw Input / HID)** — not applicable: the design opens
   no input surface and adds no runtime code. Nothing to target or abstract.
3. **Node floor** — stated (`Node >= 22.12.0`, matrix row, package.json:31) and no API is
   introduced at or above it; the move is file layout plus import specifiers. Pass, with a
   nit about the word "pinned" (F3).
4. **Native modules** — none added; the only native-capable dependency already in the tree
   (`@trivago/prettier-plugin-sort-imports`, A10) is pre-existing and untouched. Pass.
5. **Keyboard reality (layouts, dead keys, IME, non-ASCII names)** — not applicable; all 23
   moved names are ASCII lowercase (ADR:33-38), no binding names involved.
6. **File paths — separators and case** — case is handled (A16). Separators are *not*
   addressed anywhere for the nested layout; this is the dimension the move actually
   changes. See F1, F2. Per-OS config dir conventions (XDG / AppData): not applicable.
7. **Platform test per supported platform** — Linux is claimed `Yes` and Test strategy #7
   names Linux execution of items 1–5; macOS/Windows are not claimed supported, so their
   absence from #7 is consistent rather than a gap. Pass.

## Findings

- [minor][new] Compatibility matrix + Assumption A7 — A16 "no case-only or case-colliding
  renames among the 23 names", and A7 "`TEST_PATH_RE = /(^|\/)tests\/|\.test\.ts$/` already
  matches nested `tests/…/x.test.ts` paths, so path gates are unchanged" — the only
  platform dimension the matrix justifies for macOS/Windows is filename **case**. The
  dimension the move changes is **separators**: nesting adds a second separator between
  `tests/` and the filename, and the quoted gate regex anchors the `tests/` tier with a
  literal forward slash (`(^|\/)tests\/`). A7 is "verified" by reading the regex and a
  POSIX-style path form, not by establishing which separator form the pipeline actually
  feeds that regex; the Windows row's Notes cite only A16 and A15 (git's rename), which say
  nothing about runtime path matching. Fix: add one clause to A7's Basis naming the
  separator form the pipeline hands to `TEST_PATH_RE` (e.g. "paths reach the gate in
  POSIX form — git and vitest report `/`-separated paths"), and carry that clause into the
  Windows row's Notes so the row answers separators, not only case. If the pipeline builds
  OS-native paths instead, downgrade the Windows row and name the normalization step; the
  `.test\.ts$` alternation keeps file *classification* separator-safe, so the exposure is
  limited to gates that key off the `tests/` prefix specifically.

- [minor][new] Test strategy #3 — "`vitest run tests/pipeline` selects exactly the 23 moved
  files and none of the 3 root-tier tests (A6)" — the only acceptance check in the strategy
  whose meaning depends on the new nesting is a command with a forward slash in its filter
  string, and #7 removes it from every platform except Linux ("macOS/Windows rely on the
  A16 inference"). So on the two matrix rows that describe case-insensitive checkouts, the
  behavior of the nesting-specific filter is never established: the file-count assertion
  turns a mismatch into a failure rather than a silent pass, but it would fail as a
  platform artifact of the check rather than of the move. The same separator coupling shows
  up in #6's grep wording. Fix: add a one-line separator note to #3 — "filter string is
  matched against `/`-separated paths; verified on Linux" — so the platform boundary of the
  acceptance command is stated rather than implied by #7.

- [nit][new] Compatibility matrix, Node row — "discovery/compile behavior verified at these
  pinned majors (A4, A5; package.json:31,41,43)" — the cited ranges are caret ranges
  (`vitest ^5.0.5`, `typescript ^6.0.3`), not pins, and A5 records that no `vitest.config.*`
  exists, so nothing holds discovery to the version that was checked. Fix: reword to
  "verified against the vitest 5.x / TypeScript 6.x versions in the lockfile; discovery
  relies on vitest's default recursive `include`, which the design does not pin".

## FYI (no severity)

- Chosen design ¶2's carve-out ("a comment path-reference to a file moved by this same
  commit is updated in the same commit") widens the ADR's Decision bullet 2, which says the
  only content change permitted is the import depth. The reasoning is sound — the move
  itself creates the dead reference (A14) — but the ADR text is stricter than the design.
  Fidelity-to-source question, not a platform one.
- A3 (13 modules vs ADR Context's "14") is correctly parked in Deferred; the ADR's Context
  is a record, not a requirement.
- A11's evidence is the agent permission glob `**/tests/**` — whether that dialect's
  trailing `**` covers the added nesting depth is a tooling-semantics question for another
  seat; the proposal asserts it as verified without quoting the matcher.
