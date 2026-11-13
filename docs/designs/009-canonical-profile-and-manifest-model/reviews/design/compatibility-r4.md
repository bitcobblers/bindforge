# compatibility review r4

Scope: the editorial diff `reviews/design/editorial-r4.diff` (round 4 confirmation pass),
read against `02-proposal.md` and `docs/adrs/009-canonical-profile-and-manifest-model.md`.
Inputs used: the diff, the proposal, the ADR, the Revision log. No prior compatibility
findings existed for this seat, so the pass asks one question: did the r4 language edits
break a compatibility claim that was sound before them?

verdict: approve

## Findings

None. Every change in the diff is word-order, punctuation, quote-style, or voice; no
compatibility-bearing statement changes meaning. Checklist walked against the post-diff text:

1. **Compatibility matrix — every row has a status.** All eleven rows carry a status; no
   cell is empty and no cell says `unknown`. `macOS | no (v1)`, `ISO / JIS geometry | no
   (v1)`, `Non-US symbol layouts | no (v1)`, `Gamepad bindings | no (v1, reserved)`,
   `Mouse bindings | names only`, `A game's own config files (any OS, Proton prefix) | n/a`,
   `Hub upload path … | no (v1) — out of scope`. The hub row's non-`unknown` wording is the
   r1 decision recorded in the Revision log, and it matches ADR-014's ownership. Pass.
2. **OS input APIs (X11 / Wayland / Windows Raw Input / macOS HID).** The design targets
   none, and justifies the abstraction in the text the diff touched only cosmetically:
   Non-goals — "Any input-event API (injection, capture)" — plus the matrix row "A game's
   own config files (any OS, Proton prefix) | n/a — this design reads and writes only
   `schemas/` and `data/` documents; no game config I/O here". A dependency-free Rust data
   model with no event source needs no per-OS API row. Pass.
3. **Toolchain floor.** The floor is named (`Rust toolchain 1.85.0 (pinned) | yes`, A9:
   floor Rust 1.85 / edition 2024, pinned 1.85.0). Everything the design adds — `Path`,
   `DirEntry::file_type()`, `BTreeMap`, a bounded directory walk — is long-stable, far under
   the floor. No API above the floor. Pass.
4. **Native modules / dependencies.** No native dependency and no packaging story needed:
   "nothing in this design adds a dependency: `bindforge-core` stays depends-on-nothing,
   `bindforge-transforms` depends on the path crate only (A9, A10)". The diff's only edit
   here is `Core's reader` → `core's reader` (line 170) and the `(serde-based)` →
   `'(serde-based)'` quote-style change (Options, Revision log) — the deliberate deviation
   from ADR-009's "(serde-based)" stays stated in the same place with the same A9/A10 basis.
   Pass.
5. **Keyboard reality.** The diff's two substantive compatibility edits *strengthen* this
   leg: "ANSI-grid coordinates" → "US-ANSI grid coordinates" (Options → Placement legality)
   and "ANSI-US grid" → "US-ANSI grid" (Reach classes). Both now agree with the matrix rows
   ("US-ANSI keyboard geometry | yes (v1) — the only graded geometry", "ISO / JIS geometry |
   no (v1)"), with Deferred — "ISO/JIS reach tables and the layout-independent `pos:` key
   naming (A17) — … needs a layout axis in the reach-table file shape plus a v2 of every
   format carrying input ids" — and with the ADR's Consequences, "The reach tables are
   ANSI-US only at first". Non-QWERTY has two rows; non-ASCII names are refused by the id
   grammars (taxonomy/game ids `[a-z0-9-]+`, input ids lowercase words joined by `-`), while
   game-native action names stay free strings in the `action` rows. Pass.
6. **File paths.** Unchanged in substance: "**Id → file, never id → path**", indexing by the
   declared `id` / `game` + `revision`, "no path is ever built from a document string",
   "`DirEntry::file_type().is_file()`, symlinks not followed", size cap on bytes read, ids
   "only ever string-compared, so `..`, absolute paths, and encoding tricks cannot reach the
   filesystem". Separators/case are Rust `Path` + the lowercase-ASCII `-`-joined stem rule,
   and the diff's edit there is only "…, with the extension stated separately". Per-OS config
   dir convention (XDG / AppData) is not claimed and is deferred: "Where a player's personal
   policy document lives on their machine — ADR-009 leaves it open … and this design sets no
   location", which matches the ADR's "Not decided here: … where a user's personal policy
   document lives". No contradiction with the ADR, so the deferral is not a finding. Pass.
7. **Platform test per supported platform.** Matrix rows claiming support are Linux x86-64
   and Windows 10/11, and Test strategy #14 names both: "all of the above is green on the
   Linux CI runner and on a Windows contributor machine (case-insensitive filesystem,
   `BTreeMap`-ordered resolution, no cwd, no network, no database…)". Rollout 7 makes the
   Windows leg a gate before announcing the model. macOS claims no support, so it needs no
   leg. Diff touched #12's punctuation only. Pass.

## FYI (no severity)

- Proposal, last line — "Editor notes: Chosen design → Encoding rules → Nested arrays for
  structured lists — '… a leading id (action, group) followed by its values' is ambiguous;
  author to clarify." — an unresolved editorial artifact is left inside `02-proposal.md`
  after the r4 pass; that is the editorial/completeness seat's business, not mine. No
  compatibility statement depends on that sentence's reading: the per-format row shapes
  (`stack`, `swap`, `action`/`default`/`exclusive`) are pinned again in each format section
  and by Test strategy #1/#10, so the platform verdicts do not turn on it.
- Compatibility matrix — the keyboard rows cover non-QWERTY geometry and non-US symbol
  layouts, and `dead-keys` names inputs a game ignores, but no row names IME / composing
  input. Nothing in v1 can observe composition (no capture, no injection, glyph-named
  US-ANSI ids), so it is not a gap today; a half-clause beside the "Non-US symbol layouts"
  row saying so would close the checklist leg for a reader who looks for it. Not a finding:
  pre-existing, not introduced by this diff, and the abstraction is justified by Non-goals.
