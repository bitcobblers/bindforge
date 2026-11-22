# complexity review r2

verdict: approve

Scope: the editorial round-2 diff (`reviews/design/editorial-r2.diff`, 15 hunks)
against `02-proposal.md`, with `docs/adrs/027-hosted-registry-with-store-resolvers.md`
and the proposal's Revision log as supporting inputs. No earlier review reports
or transcripts were opened. No prior complexity findings were carried into this
pass (the editorial phase is one language pass per round), so the pass tests one
thing: whether any editorial edit changed what a section commits the build to.

All fifteen hunks are language-only: hyphen placement and line re-wrapping
(`named-refusal family`, `retirement refusal`), comma insertion (the health
bullet's "…with the mode, and `steam_snapshot.fetched_at` is their successor",
"tmp-and-rename, then publish the new model", "and file untouched, and nothing
served", "Without it, a verifier relaxed…"), article insertion ("it is an
ordered insert with membership by binary search"), nominalization
("bookkeeping ensuring that deleted ids never return", "an overlap only
re-fetches an item"), verb restoration ("a delta merge holds the old snapshot,
the merged model, one serialize buffer and the current page body"), clause
re-ordering in Rollout 4 ("the issued-document rewrite measured at 100k and
200k records, recorded in §Budgets"), and one number correction ("New flag:
`--no-steam`"). None adds, removes, renames, or re-scopes a package, module,
constant, document, flag, refusal, budget, or lock.

## Checklist walk

1. Reuse (new package/module/interface; check `tools/` first): not applicable —
   the diff introduces no new part. The parts it brushes (`§Concurrency`'s one
   mutation mutex, the `StoreTransport` seam, `retired-ids-v1.json`) are
   untouched in their substance, and the design reuses shipped machinery for
   them (the `026-C7` tolerance pair, `corpus.rs::Catalog`,
   `discovery::game::friendly_name`, the A2 tmp-and-rename discipline).
2. Trace to a stated Goal: unchanged — no hunk adds a moving part, so no new
   untraced part exists. The re-worded Goal bullet still traces to
   `027-C4`/`027-C5`/`027-L2`.
3. A 30–50% smaller design: unchanged by this diff. The candidates for such a
   cut (retired-id document, `--no-steam`, the page-retry grid) each trace to
   `027-L2`, `027-C3`, and `027-B4` respectively, and the Revision log records
   the deliberate rejections of the two bookkeeping subsystems an earlier round
   had already shed (`dropped_appids`, `024-L2` ordinals).
4. New dependencies: unchanged — the diff does not touch the `rustls` /
   `webpki-roots` rationale in §Options, which stays confined to `steam.rs`.
5. Defer the second half: unchanged — Rollout ordering is untouched apart from
   Rollout 4's clause re-order, which still puts the measured rewrite in
   §Budgets and keeps issuance (item 4) after the resolver (item 3).
6. Knobs with no consumer / single-implementation abstractions: unchanged. The
   diff's §The store pull and §Budgets edits actually preserve the anti-knob
   text verbatim ("`STEAM_RETRY_SECONDS` — the only consumer of that constant"
   and the deadline grid that justifies `STEAM_PULL_DEADLINE_SECONDS`).
7. Value check: proportionate. The design's heaviest new machinery — one
   snapshot document, one issued document, one retired-id document, one mint
   module, one pull seam — buys the ADR's stated outcome (a registry that
   answers real Steam keys on day one, `027-S1`) without a merge operation, a
   federation path, or a per-query store call; the retiring half removes more
   code (`cache.rs`, `upstream.rs`, `endpoint.rs`, five suites) than the new
   half adds. Nothing in this diff shifts that balance.

## Independent checks run (read-only, this pass)

- "`New flag: `--no-steam`" is now the correct number: the proposal's flag
  inventory adds exactly one flag (`--root --corpus` are shipped, and
  `--upstream` / `--refresh-interval` stay in the parser only to be refused).
- The diff-touched host text still matches the tree it describes:
  `apps/registry/src/main.rs` ships `serve_command` (line 255) and
  `query_command` (line 280) behind a `serve`/`query` dispatch, so §The store
  pull's "lives in `main.rs`'s `serve_command` only (never `query` …)" and
  §Hosted mode's "`registry query <slug>` survives" both hold.
- `MAX_SLUG_OCTETS` / `MAX_NATIVE_NAME_OCTETS` / `MAX_DISCOVERY_SLUG_OCTETS`
  exist as shipped constants in `bindforge_core` (pinned at 128 in
  `crates/core/tests/zz_api_probe.rs`), which is the bound the re-worded §Budgets
  memory accounting and the mint bullet lean on.

## Findings

None. No editorial edit broke a mechanism, made a cited section false, moved a
budget or refusal name, or altered the deferral boundary. The one
deferral-adjacent hunk (§Deferred, "the separate small ADR the source
document's Consequences **leaves** outside its ledger") now matches the ADR's
Consequences more exactly than the previous wording did — the ADR says a
separate small amendment for CLI self-resolution of `local/<dir>` keys is
"proposed (outside this ADR's ledger)" — so the re-open bar is not met.

## FYI (no severity)

- To the language seat: "New flag" (singular) and "retirement refusal" are both
  improvements, and the Rollout 4 re-order removes the stacked prenominal
  "measured … rewrite"; no technical meaning moved in either.
- To the correctness seat: the diff-touched §Budgets sentence now reads as two
  separate transient claims ("a mint holds one model plus one serialized buffer"
  / "a delta merge holds the old snapshot, the merged model, one serialize
  buffer and the current page body"); the second matches §The store pull's
  one-linear-merge-plus-one-index-build wording, so the two sections stay in
  agreement — no action, noting only that the grammar repair is what makes that
  pairing readable.
