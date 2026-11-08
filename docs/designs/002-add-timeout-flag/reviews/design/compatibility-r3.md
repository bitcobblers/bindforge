# compatibility review r3

verdict: approve

Scope: this change set only — `reviews/design/editorial-r3.diff` applied to `02-proposal.md`. The document body and the r1/r2 closure state are context, not the target; the previous compatibility rounds already walked the checklist against the whole document.

## Findings
- none

Every hunk in the diff is language-only and compatibility-neutral. Verified change by change:

1. Context (02-proposal.md:5) — "honoring" → "honouring". Spelling only; no compatibility claim touched.
2. Goals bullet 2 (02-proposal.md:11) — "honors" → "honours". Spelling only.
3. Options considered — Value semantics (02-proposal.md:55) — "its unit-suffix syntax we are not adopting" → "its unit-suffix syntax, which we are not adopting". Grammar; the curl/libpq/coreutils divergence discussion (the deliberate `0`-semantics incompatibility, A14) is byte-identical otherwise.
4. Chosen design — resolver bullet (02-proposal.md:85) — "avoid second hardcoded bounds" → "avoid a second set of hard-coded bounds". The single-source-of-truth rule for the Node-floor-adjacent bounds (`DEFAULT_REPLY_TIMEOUT_MS / 1000`, `MAX_REPLY_TIMEOUT_MS / 1000`) is unchanged.
5. Test strategy #6 (02-proposal.md:126) — inserted "that". The two-check sub-poll contract and the `within 0.1 s` formatting pin, which the Node `setTimeout`-clamp exemption (A9) depends on, are unchanged.
6. Test strategy #10 (02-proposal.md:130) — inserted comma after "without the stub result". Text of the boundary-proof route unchanged.
7. Revision log r3 block (02-proposal.md:160) — removes a verbatim duplication of the "Deliberately not changed this round" paragraph. Diffed against the r2 closure state: the retained copy preserves every compatibility statement verbatim ("no CI configuration … a `macos-latest`/`windows-latest` matrix … remains outside `tools/orchestration`, so those rows stay 'Yes — unverified'"), and the appended "register statuses … stay `unverified`" sentence matches the current document (02-proposal.md:160). Nothing compatibility-bearing was dropped.

The `+` side of every hunk was confirmed present verbatim in the applied `02-proposal.md` at lines 5, 11, 55, 85, 126, 130, 160 — the diff applies cleanly with no collateral edits. The hunks touch no line in the Compatibility matrix (02-proposal.md:108–117), the `package.json`/`engines` bullet (line 90), the Test #12 platform gate (line 132), or Rollout (lines 134–139).

## Checklist walk-through (against the change set)
- 1 Matrix statuses: unaffected — no hunk touches the matrix; all six rows still carry explicit statuses, no `unknown`, no empty cells.
- 2 OS input APIs: not applicable — the diff adds no input-API surface; the HTTP-polling OS-neutral justification (A12) stands as closed in r1/r2.
- 3 Node version floor: unaffected — no API usage, no `engines` text, and no toolchain-row wording changed; nothing in the diff can use an API above Node ≥ 22.12.0.
- 4 Native modules: unaffected — packaging/dependency wording untouched; still pure JS with the `dist/` story (Rollout #4).
- 5 Keyboard reality: not applicable — no keyboard or binding-name surface in the diff.
- 6 File paths: not applicable — no path or config-dir handling in the diff.
- 7 Platform test per claimed-supported platform: unaffected — Test #6/#10 edits are punctuation/grammar; Test #12 (the Linux-local `npm run check` gate plus the "Yes — unverified" macOS/Windows disclosure that closed r1's major) is not in any hunk.

## FYI (no severity)
- The compatibility-r2 nit (toolchain-floor `engines.node` possibly warning/publishing `EBADENGINE` at lower-Node runtime consumers, 02-proposal.md:90) is untouched by this editorial diff; it remains open as a non-blocking item and needs no action for this change set.
