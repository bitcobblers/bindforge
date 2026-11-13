# compatibility review r2

verdict: approve

Scope: the editorial diff `reviews/design/editorial-r2.diff` (11 hunks) applied to
`02-proposal.md`, read against `docs/adrs/014-profiles-as-chains.md` and the
proposal's Revision log. Prior compatibility findings this round: none, so this
is a confirmation pass — checked whether the editorial edits broke a
compatibility surface or made a compatibility claim false.

Diff-by-diff check against the compatibility checklist:

- **Matrix rows (item 1)** — untouched by the diff; every row still carries a
  status (`yes`, `yes (refuses cleanly)`, `yes (untouched)`, `yes (data only)`,
  `no (v1)`, `not applicable`). No `unknown`, no empty cell.
- **OS input APIs (item 2)** — untouched; the matrix's `Product artifacts …
  not applicable — This stage ships no binary` row remains the justification for
  not naming X11/Wayland/Raw Input/HID targets, which is consistent with the
  ADR (Decision One ships a format and its resolution, not a capture layer).
- **Version floor (item 3)** — the toolchain-floor row (`1.85.0`, edition 2024,
  std-only, `Cargo.lock` byte-unchanged) is unchanged; none of the edits touch
  the dependency or floor story.
- **Native modules (item 4)** — unchanged; still no native dependency, no CI
  change, both workflows listed under Unchanged.
- **Keyboard reality (item 5)** — unchanged by the diff; the closed-vocabulary
  gap (Tab/Enter/arrows/F-keys, A9) is deferred to the `reach-classes` major and
  does not contradict the ADR's Decision or Consequences.
- **File paths (item 6)** — the touched hunk is only "Absent is" → "An absent
  file is"; the location rule itself is unchanged: direct child of the head's
  major directory, slug-validated component, `check_dir_version` pin, and the
  Windows case-uniqueness row plus the case-unique fixture-name rule still back
  id↔file-name equality.
- **Platform test per supported platform (item 7)** — Test strategy #9 is
  untouched: Linux CI leg named, Windows contributor leg named, macOS is `no`
  so needs no leg. Rollout 7 still points at that same #9 Windows leg.

Meaning-change audit of the edits themselves: `refused` → `refuse`, `chosen` →
`selected`, `Absent is` → `An absent file is`, `needs one` → `needed`, added
`to`/`that`/`applies`, and the re-wrapping in `diff`/`merge`/Test 5/Test 7 are
all language-only; each post-image matches the file as it stands, and no refusal,
pin, budget, or matrix row changed meaning.

## Findings

None. No blocker or major is open, so this seat approves.

## FYI (no severity)

- Chosen design → composite-cost paragraph — "one resolution opens at most
  `MAX_CHAIN_DEPTH` documents — one per link — totalling at most
  `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB)" — the added "totalling"
  sharpens the A21 statement (it was ambiguous between per-document and
  aggregate) and now agrees with the Risks row's "≤ 32 documents / 32 MiB". No
  action; noted because the sentence was touched.
- Test strategy #7 — the re-wrapped code span now reads `forged-log-line.toml`,
  matching the invalid-fixture name in the contract inventory; the previous
  mid-token break in the span was a rendering defect, and this pass fixed it.
- The CI re-validation matrix stays deferred exactly as design 013 left it, with
  the operand side (pin + equality rule) shipped here — consistent with the ADR,
  which lists no CI requirement among its Consequences. Not re-opened.
