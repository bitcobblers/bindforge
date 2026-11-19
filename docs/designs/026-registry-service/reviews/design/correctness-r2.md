# correctness review r2

scope: the editorial diff `docs/designs/026-registry-service/reviews/design/editorial-r2.diff`
(the single editorial confirmation pass of round 2), read against `02-proposal.md`,
`docs/adrs/026-registry-service.md`, the proposal's Revision log, and the code
through the read-only inspection allowlist.

verdict: approve

## Checklist disposition

1. **Assumption register** — the diff re-wraps or re-words text anchored to A7,
   A11, A22, A23 only; no `verified` row's basis is changed or contradicted by
   the pass. Spot-checked the two code-fact claims the touched lines carry:
   core exposes exactly thirteen `pub const MAX_*` budgets
   (`crates/core/src/{toml,placement,manifest,document,chain}.rs`), so §Budgets'
   "core's thirteen-constant family … that count is unchanged" holds; and
   `apps/discovery-cli/src/catalog.rs:248` is indeed `if !addr.ip().is_loopback()`
   inside `probe()`, with the `LOOPBACK_HOSTS` host check inside `parse_endpoint`
   (catalog.rs:141) and the digits-only port below it — the §CLI amendment bullets
   the pass re-worded ("deletes" → "drops") describe shipped code accurately.
2. **Research URL spot-check** — not applicable to this scope: the diff adds,
   deletes and re-words no citation; the only external links (A20 RFC 6750,
   A21 RFC 9110) are untouched and already carried as `unverified` with the
   binding obligations pinned to the ADR's own unit texts.
3. **Internal contradictions** — every substantive re-wording in the diff moves
   *toward* the document's own definitions, not away: §Test strategy #10's
   "would pass `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422`" becomes
   "would exceed", which is what §The admin face and auth and §Budgets already
   state (the cap is checked against "the exact post-`revision`-bump buffer");
   §Budgets' "answers the one request §http.rs closes" becomes "answers exactly
   one request before `http.rs` closes the connection", which matches the
   `http.rs` module entry ("then closing the connection — one answer per
   connection") and A23, and drops a cross-reference to a section that does not
   exist; "`026-S1` is satisfied partially" → "is partially satisfied" keeps the
   §Rollout step 6 ledger annotation intact; "the service is tasks" → "lands as
   tasks" matches Rollout 1–7 and `026` Rollout 7. The added serial in §Caching
   mode ("reports the verdict as it stands; it never fires the probe") agrees
   with §The serving face's health paragraph and #12.
4. **Missing edge cases** — no behaviour statement is added, narrowed or
   removed; the refusals, caps (`5_000`, `1_048_576`, `3_600`, 64, 16,777,216),
   the `404`/`405`/`422`/`503` spellings and the `422`-before-any-tmp ordering are
   unchanged in value.
5. **Interfaces** — route names, field lists (`{games, source_keys, revision}`,
   the health body), constant names and module names appear once each after the
   pass; the two `§`-reference insertions ("§Budgets", "§Caching mode") point at
   sections that exist as spelled.
6. **Decides what the source asked** — unchanged by an editorial pass; the diff
   leaves the `026-C1`…`026-C7`, `026-L1`…`026-L6`, `026-B1`…`026-B3` and
   `026-S1` decisions where the ADR put them.
7. **Prior-round closures** — the Revision log's r4 entries on the write-time
   document cap (#10) and the copied request builder (#7) are still closed after
   this pass; if anything, #10's test line now says what the r4 closure meant.
   The "deletes" → "drops" change on `probe()` is the same instruction the ADR's
   Rollout item 5 uses ("`probe()` drops its loopback-resolution refusal").

## Findings

None. No blocker, major, minor or regression finding against this diff.

## Deferred-section check

Nothing in the diff touches §Deferred / out of scope, and the two touched-adjacent
deferrals (`026-R1` transport assurance; seed-ids adoption) remain consistent with
`026-R1` and `026-S1` as the ADR words them. No deferral re-opened.

## FYI (no severity)

- §Test strategy #9, #12, #13, #15, #16 and §Rollout steps 5–6: the pass reduced
  some continuation lines by one space, so within a single list item some
  continuations now sit at the marker's content indent and siblings one column
  deeper (e.g. #9's last line at 3 spaces against 4 above it; #12's last two at 4
  against 5 above). Rendering is identical and no item splits, and no gate covers
  markdown layout (`check` runs `prettier --check .` only inside
  `tools/orchestration`), so this is a language/style consistency note for the
  editorial seat, not a correctness one.
- §Budgets: the re-wrap of the `MAX_CONCURRENT_CONNECTIONS` parenthesis leaves two
  short lines mid-sentence; wording is correct, only the ragged wrap differs from
  the file's ~88-column style.
