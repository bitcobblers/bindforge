# correctness review r2

scope: `reviews/design/editorial-r2.diff` (the single editorial language pass of round 2), read against
`02-proposal.md`, `docs/adrs/009-canonical-profile-and-manifest-model.md`, and the proposal's Revision log.
Previous correctness findings: none, so ratification rules apply — a finding is reported only where the
changed text breaks something previously approved.

verdict: request-changes

## Checklist pass

- Item 1/2 (assumption basis + anti-hallucination gate) — spot-checked the live status of every cited URL
  sitting in the diff's touched text: `toml.io/en/v1.1.0` (A12 bare-key charset, dotted keys define tables;
  A13 nested arrays, e.g. `nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]`) — supports the rows;
  `rfc-editor.org/rfc/rfc6902` (A14, "Operations are applied sequentially in the order they appear in the
  array") — supports the row; `pubmed.ncbi.nlm.nih.gov/25029899/` (A15) — live record, *The effect of
  keyboard key spacing on typing speed, error, usability, and biomechanics, Part 2: Vertical spacing*,
  Hum Factors 56(4), consistent with "no external ground truth for a per-key reach grading";
  `w3.org/TR/gamepad/` (A20, "Standard Gamepad layout", mapping `"standard"`) — supports the row.
  **One dead source: A16's Wikipedia link.** Reported below.
- Item 3/5 (contradictions, one-name-one-shape) — the diff's reference normalization is sound: every
  `§N` → `contract N` swap keeps its referent. `contract 3` = Chosen design §3 (Profiles are chains),
  which is where the stem rule and the two index constants live; `contract 4` = §4 (Transforms), where
  `apply_policy`'s `Err` channel lives; `contract 1` = §1 (Action taxonomy), where the canonical emphasis
  statement lives. The `contract N` numbering matches Context's five-contract order and the pre-existing
  `(contract 5)` usage, so no second naming scheme is introduced. `Test #N` → `Test strategy #N` points at
  real numbered items (#1, #3, #4, #6, #8, #9, #10). No interface renamed.
- Item 4 (edge cases) — no mechanism is added, removed, or re-scoped by the diff; every hunk is punctuation,
  an article, an Oxford comma, a verb insertion ("variants **are** pinned", "condition **is** pinned"),
  a dedup, `valid+invalid` → `valid/invalid` (which matches the harness's `fixtures/{valid,invalid}`), and
  `Platform:` → `Platform —` (matches the other items' shape).
- Item 6 (decides what the ADR asked) — unchanged; the one touched substantive claim, `docked` deferral,
  reads "apply-time work owned by ADR-012 (Deferred)", which matches the Deferred bullet naming
  "honoring a manifest's `docked` actions when a binding is applied — ADR-012" and does not contradict
  ADR-009 Decision 5 (a manifest records "hard-docked UI constraints"; it never requires v1 to honor them).
- Item 7 (previous blockers closed for real) — no prior correctness findings; Revision log r1–r3 closures
  are untouched in substance by this diff.

## Findings

- [major][new] Assumption register → A16 — "https://en.wikipedia.org/wiki/Cursor_home_key" — the cited
  basis is a dead link: the URL returns 404, and the MediaWiki query API reports `"title":"Cursor home key",
  "missing"` (the live article is `Home key`), so the row's `verified` status rests on nothing
  checkable. The row is not idle: the Revision-log line this diff rewrote cites it
  ("the "OKL; keeps the home-row anchor" claim demoted to a `validate` verdict (A16)"), and the body leans on
  the same row for the placement verdict paragraph. Severity is major, not blocker, because the design's
  substance survives: the claim is already demoted to "whatever `validate` and the graded tables report",
  so the fix is evidentiary only. Fix: replace the dead URL in the A16 basis with a live one —
  `https://en.wikipedia.org/wiki/Home_key` for the home-row anchor — plus the repo-internal basis that
  already states the practice split, ADR-009 Context observation 1 ("Community practice disagrees on
  placement even for one cluster: IJKL and Returnal's built-in southpaw OKL; are both common, and IJKL
  breaks the home-row anchor while OKL; does not"). Citation-only edit, no design text changes, so it is
  squarely inside this editorial lane.

## FYI (no severity)

- The diff's appended "Editor notes: Chosen design → Profiles are chains — "`for` is required at parse" is
  ambiguous; author to clarify." is a sound flag, and the ambiguity is real: §3 lists `for` among entries
  that are "**optional** entries only" with "absent = a profile not bound to a game", Migrations repeats
  "`for` absent = not bound to a game", while the Resolution bullet's "`for` is required at parse" reads as
  an entry-set requirement that would make every unbound profile unparsable. The intended rule is the one
  Test strategy #6 already pins — a `for` that is present is grammar-checked at parse (`BadPin`). The
  phrase is outside this diff's changed text, and the fix is technical, not linguistic, so it is the
  author's next revise round, not this pass: reword to "a present `for` is grammar-checked at parse".
- Consume that note in the next revise round and delete the "Editor notes:" line; it sits inside
  `## Revision log`, whose entries are author decisions, so a standing author-directed note in the merged
  proposal reads as an open item in a closed section.
- `contract N` now carries work that `§N` did: a reader must infer the mapping from Context's
  five-contract list order to the numbered Chosen design sections. One clause in Context ("the five
  contracts, numbered as the Chosen design sections below") makes every normalized reference self-resolving.
- The manifest paragraph's "(contract 3)" sits after "the game-id grammar and the stem rule", but only the
  stem rule lives in §3 — the game-id grammar is canonical in Encoding rules. Pre-existing attachment
  ambiguity inherited from `§3`; moving the parenthetical after "the stem rule" removes it.
