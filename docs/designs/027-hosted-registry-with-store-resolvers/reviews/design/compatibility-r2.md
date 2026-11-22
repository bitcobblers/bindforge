# compatibility review r2

verdict: approve

Scope as given: the editorial diff `reviews/design/editorial-r2.diff` (15 hunks),
against `02-proposal.md`, `docs/adrs/027-hosted-registry-with-store-resolvers.md`,
and the Revision log. Previous findings for this seat: none (single editorial
language pass per round), so nothing to re-check for regressions of my own
earlier claims.

## Findings

None. Every hunk is language-only; no compatibility-bearing claim in the
changed text changed meaning or became false.

Hunk-by-hunk disposition (all 15 landed verbatim in the current proposal):

- Goals, "bookkeeping **ensuring** that deleted ids never return" (l.31) —
  grammar; the `027-L2` claim it summarizes is unchanged.
- §Hosted mode, `named-refusal` re-wrap (l.161–162) and "the retirement
  refusal" de-hyphenation in Test strategy #8 (l.632–633) — same term both
  times; still matches the shipped gate shape verified in
  `apps/registry/src/endpoint.rs:34` (`"--upstream \`{}\` is not an endpoint
  this registry runs: …"`) and the real shipped flags
  `apps/registry/src/main.rs:160,173`.
- §Hosted mode, "New **flag**: `--no-steam`" (l.173) — the only correction with
  a factual referent, and the singular is the true one: enumerating every
  flag span in the proposal, `--no-steam` is the only new one; `--root`,
  `--corpus`, `--upstream`, `--refresh-interval` are all shipped (A7, A15,
  `main.rs`). The plural "resolver flags" left in Goals l.21 mirrors the ADR's
  own wording (`027-C1`: "the resolver flags of `027-C3`"), so the edit did not
  introduce an inconsistency.
- §Hosted mode, health comma (l.199) — the re-pinned health list and the
  retired-fact list are untouched.
- §The store pull, "an overlap only re-fetches an item" (l.339) and
  "tmp-and-rename**,** then publish" (l.344–345) — sequence and the
  `fetched_at`-before-first-page rule unchanged.
- §Issued entries, "an ordered insert" (l.446) — unchanged.
- §Budgets, "and a delta merge **holds** the old snapshot, the merged model,
  one serialize buffer and the current page body" (l.533–534) — supplies the
  verb the sentence was missing; the four-cap total it sits beside checks out
  (`MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216`, `constants.rs:27`, plus the
  64/32/32 MiB family the design names = 144 MiB), and the transient list
  matches §The store pull's one-merge-plus-one-index-build publish path.
- Test strategy #4 comma (l.591), #6 "Without it**,**" (l.627) — unchanged
  assertions, including the `027-L4` no-key-in-failure-lines pin and the #17
  loopback rule.
- Rollout 4, "the issued-document rewrite **measured** at 100k and 200k
  records**,** recorded in §Budgets" (l.730–731) — same deliverable, and still
  consistent with §Budgets' "Rollout 4 records the measured issued-document
  rewrite".
- §Deferred, "the source document's Consequences **leaves** outside its
  ledger" (l.771) — matches the ADR's Consequences ("a separate small
  amendment proposed (outside this ADR's ledger)",
  `docs/adrs/027-hosted-registry-with-store-resolvers.md:291–294`), so the
  `local/<dir>` deferral stays consistent with its source document and is not
  re-opened per the deferral rule.

## Checklist disposition

1. **Compatibility matrix rows** — not touched by this diff; re-read to confirm
   nothing in the changed text contradicted it. All five rows carry a status
   (`Yes`, `Dev only`, `Yes (compile-only)`, `No`, `Yes`); no `unknown`, no
   empty cell. Not applicable as a finding this pass.
2. **OS input APIs (X11/Wayland/Windows Raw Input/macOS HID)** — not
   applicable: the diff touches only registry-side document/pull text; the
   design claims no input surface.
3. **Node version floor** — not applicable: no orchestration surface changes
   (Test strategy #17), and no changed hunk names a Node API.
4. **Native modules / CI matrix** — not touched by this diff; the `ring`
   C/asm story, the `x86_64-pc-windows-msvc` leg split and the `README.md`
   C-toolchain prerequisite all sit outside the hunks. Re-checked the shipped
   leg exists as A16 describes (`.github/workflows/check.yml:29–32`,
   whole-workspace `cargo check --target x86_64-pc-windows-msvc`, no macOS
   leg), so nothing in the changed text became false about it.
5. **Keyboard reality (non-QWERTY, dead keys, IME, non-ASCII names)** — the
   two hunks that brush it (#4, #6) changed punctuation only; the mint's
   fold/`steam-<appid>` fallback and the pull's control-strip-then-clamp rule
   are as previously settled in the Revision log (r0→r4 entry).
6. **File paths / per-OS config dirs** — Windows `%APPDATA%\bindforge\registry\…`
   spelling and the XDG branch are outside the diff; the diff's changed lines
   add no path handling. Shipped `FETCHED_AT_*` tolerances the neighboring
   text re-uses check out (`constants.rs:68,72`).
7. **Platform test per supported row** — #16 was not edited; the changed
   #8/#6/#4 hunks name no platform coverage.
8. **Revision log** — read first; no hunk resurrects a rejected finding, and
   no closed theme is re-opened by the changed wording.

## FYI (no severity)

- Outside this pass's hunks, and recorded here rather than as a finding: the
  design's line "The cargo trio screens no advisories, so Rollout 3 lands a
  RUSTSEC screen beside them" (§Options) and Rollout 3's "the RUSTSEC advisory
  screen added to the gate" do not mention the repo's existing scheduled
  `.github/workflows/advisories.yml` (`cargo deny check`, `deny.toml`
  waivers), whose header states advisory scanning is "never a PR check"
  (`check.yml:8–10`) — a CI-practice question for whoever owns the gate, not a
  compatibility finding and not text this diff touched.
- Also from this diff and purely cosmetic: the Rollout 4 line
  (`02-proposal.md:730`) now runs well past the document's wrap width; no
  markdown formatter or lint runs in either gate leg, so it is a style
  inconsistency only.
