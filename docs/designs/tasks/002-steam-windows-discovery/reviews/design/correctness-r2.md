# correctness review r2

verdict: approve

Scope: the editorial-r2 diff only (single editorial confirmation pass, round 2; this seat's previous findings: none). All 11 hunks were checked against the current `02-proposal.md` — each is applied and each is language-only (spelling, articles, prepositions, a comma, `§"Options considered"` → `§*Options considered*`, `deduped` → `de-duped`).

- Checklist 1 (assumption register): not in the diff — no `verified` row or basis touched. N/A for this pass.
- Checklist 2 (URL spot-check): the diff adds or alters no citation; the anti-hallucination gate was exercised in earlier whole-document passes. N/A for this pass.
- Checklist 3 (internal contradictions): none introduced. `described below` (Options → §1) points the right way — the `steam_linux.rs` rewiring is explained in the paragraph following the code block. `which *Test strategy* #12 compiles` matches #12's stated content ("type-checking §3's `from_host()` and `registry_string_value`"). `is covered by the `%ProgramFiles%` default or by `--steam-root` (§8)` keeps §4's fallback consistent with §5's env candidates and the Compatibility matrix row.
- Checklist 4 (edge cases): the diff changes no rule text; the `\`-refusal, REG_SZ-only seam, candidate ordering and canonical-path rule are untouched.
- Checklist 5 (interfaces): no identifier renamed (`favor`, `de-duped` are prose; `host_candidates`, `RegistryKey`, `registry_string_value` unchanged). The `§*Options considered*` reference resolves to the existing `## Options considered` heading.
- Checklist 6 (decides what the task asked): unchanged by this pass.
- Checklist 7 (prior blocker/major closures): no prior findings for this seat; the Revision log's r0/r1 decisions are not re-opened by any hunk, and no hunk reverses a logged decision.
- Deferred items: untouched; nothing contradicts docs/tasks/T-002-steam-windows-discovery.md.

## Findings

None.

## FYI (no severity)

- *Risks*, "`winreg` goes unmaintained" row: "the migration to `windows-core` is one file" — the *Options considered* entry it cites names the `windows` crate as the actual migration target ("`windows-core` alone does not carry the registry feature set"); the row's shorthand predates this diff (the hunk only restyled the section reference), so it is not this pass's finding to raise.
