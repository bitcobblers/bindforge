# security review r2

verdict: approve

Scope: the editorial round-2 diff
(`docs/designs/026-registry-service/reviews/design/editorial-r2.diff`), read
against `02-proposal.md` and `docs/adrs/026-registry-service.md`, plus the
proposal's Revision log. This is the single language pass of round 2, so the
only question for this seat is whether any touched sentence weakened, inverted
or contradicted a security property. Every hunk was checked against the
security checklist; no security-bearing claim was found broken.

## Checklist walk (this pass)

1. Trust boundaries — n/a to the diff: no hunk re-classifies a data source. The
   untrusted-input statements (`026-C7` cache at load and write seams, upstream
   ingest validation) are untouched by the diff.
2. Parsing untrusted data — touched only by hunk 4 (§Catalog document) and
   hunk 13 (#1). Both preserve the refusal set, including "a document over
   `MAX_CATALOG_DOCUMENT_OCTETS`". No cap, refusal or "by name" wording lost.
3. Path handling — the §Serving face step 1 rule was touched only for serial
   commas ("a corpus, cache document, or refusal line"; "candidate queries,
   admin upserts, and cache loads alike"). The decoded-slug rule, the `.`/`..`
   and separator refusals, the control-byte/invalid-UTF-8 refusal and the
   never-joined-onto-paths statement are verbatim intact.
4. Secrets/PII — #9 and the §Admin face token sentences changed only in
   indentation; "no echo of the configured token", `401`,
   `WWW-Authenticate: Bearer` and the constant-time compare survive.
   `026-L2`'s disclosure statement is untouched.
5. Dependencies — the dependency line (`bindforge-core` + `discovery` +
   `serde_json`) is not in this diff; the touched §`apps/registry` hunk is a
   line-wrap only. No new dependency is named, so no advisory lookup is
   triggered by this pass.
6. Privileges — `probe()` hunk 12 ("deletes" → "drops" the `is_loopback()`
   non-loopback-resolution refusal) preserves the fail-closed pre-plugin
   connect; verified against shipped code: `apps/discovery-cli/src/catalog.rs`
   line 248 (`is_loopback()` refusal), 256 (`connect_timeout`), 141
   (`LOOPBACK_HOSTS`) are the shipped blocks the proposal says are amended, and
   line 438 confirms the shipped request carries `Connection: close`, which is
   what hunk 11's reworded "answers exactly one request before `http.rs` closes
   the connection" rests on (A23).
7. Fail-closed — the one substantive wording change in the diff is #10: "an
   upsert whose serialized corpus with its bumped `revision` **would pass**
   `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422`" → "**would exceed**".
   That is a security improvement, not a regression: the old phrasing admitted
   a fail-open reading (a document *within* the cap gets refused). The new
   wording matches §The admin face ("one over `MAX_CATALOG_DOCUMENT_OCTETS` …
   is the named `422`", proposal line 285) and the Revision log's r4 entry
   ("an over-cap upsert the named `422` before any tmp"). Boundary direction is
   now consistent with #5's "≤ 1_048_576 octets" acceptance.

## Findings

None. No blocker, major, minor or security-severity nit in the diff. Nothing
this seat approved was broken, and no section cited earlier was made false. The
Revision log was read first: no finding here re-opens a rejected item, and the
`026-R1` (TLS/transport assurance) and `026-R2` (auth beyond the single bearer
token) deferrals stay consistent with the ADR's own reservations — they are not
re-opened.

## FYI (no severity)

- §Budgets (hunk 11): the reflowed `MAX_CONCURRENT_CONNECTIONS` sentence leaves
  a hard line break mid-sentence ("so ≤64 live / threads and ≤64 in-flight"),
  and several §Test strategy items (#9, #12, #13, #15, #16) had continuation
  lines de-indented by one column. Both are language/format hygiene for the
  editorial seat, not a security property.
