# Complexity review r1 (dissent pass c4)

verdict: approve

Scope as directed: `git diff 1d288993 d105d78c` over `02-proposal.md` (A7, §Options
partial-snapshot sentence, §The steam snapshot resolver opening, §The store pull
normalization paragraph, §Budgets `MAX_STEAM_PAGE_OCTETS`, §Risks `013-B4` row,
§Test strategy #6, Rollout 3, two Revision-log lines), then the full current
document. Checklist items 1–7 applied to the changed text.

## My r1 findings — all four closed, and closed against the code

- **major (control-byte drop credited to the shipped rule) — closed.** §The store
  pull now says the normalization is "one std filter plus the shipped clamp, in
  that order: drop the `char::is_control()` characters, then
  `discovery::game::friendly_name`", A7 states the shipped rule as "whitespace
  collapse plus the longest `char`-boundary prefix … **and nothing else**", and
  Rollout 3 names the strip beside the clamp. Verified against the code A7 now
  cites: `crates/discovery/src/game.rs:102-113` is `split_whitespace().join(" ")`
  then the `is_char_boundary` clamp — nothing else, exactly as A7 now reads. The
  loop closes: `char::is_control()` is C0 + DEL + C1, a strict superset of the
  ASCII-control reading the shipped gates use (`crates/core/src/manifest.rs:355`,
  `apps/registry/src/keys.rs:78`), so the pull can never publish a `name` the
  §The steam snapshot resolver loader refuses, and the restart-into-outage branch
  my r1 finding named is gone. No new mechanism: std filter + existing call.
- **nit (`MAX_STEAM_PAGE_OCTETS` derivation) — closed.** "12 MiB, above the
  50k-records-at-~240-octets estimate" reads as the sum it is: 12 × 1024² =
  12,582,912 > 50,000 × 240 = 12,000,000, and ~240 octets is right for A17's four
  fields with `name` at 128. The r3 Revision-log line matches.
- **nit (empty-name disposition) — closed** with the first option my r1 fix
  offered: "A name that normalizes to empty stays empty: its label renders blank
  and its identity is mint step 3's `steam-<appid>`", pinned by §Test strategy #6
  ("an all-whitespace or all-control title stores empty"). Checked for breaks:
  `friendly_name` clamps a non-empty string to non-empty (budget 128 > 0), so
  "normalizes to empty" is exactly all-control-or-whitespace, as #6 spells it;
  `corpus.rs:424` (the `026-C3` gate the issued document reuses) checks the upper
  bound only, so the issued record loads; the serving face applies no name gate on
  the way out (no `MAX_NATIVE_NAME_OCTETS` use outside `cache.rs`/`corpus.rs`/
  `upstream.rs`); `apps/discovery-cli/src/catalog.rs:378-406` gates only
  `game_id`, so a blank label is display, not a refusal; mint step 1 on `""`
  falls to step 3's appid-unique `steam-<appid>`. No break found.
- **nit (`discovery::friendly_name` misnamed) — closed.** `crates/discovery/src/lib.rs`
  is `pub mod game;` with no crate-root re-export, so A7's
  `discovery::game::friendly_name` (no crate-root re-export), §The store pull, and
  Rollout 3 all name the importable path. A7's dependency claim also checks out
  against `apps/registry/Cargo.toml` (`bindforge-core`, `discovery`, `serde_json`),
  so the reuse costs no new dependency, and grep finds no existing in-repo control
  stripper to reuse (`char::is_control` appears only as a check, e.g.
  `apps/discovery-cli/src/main.rs:329`), so checklist #1 is satisfied rather than
  violated.

## Findings

- [nit][new] §The store pull — "The publish line counts the names it clamped (§Risks, §Deferred)" — the revision made the registry own a *second* normalization (the `char::is_control()` strip) but the only count in the design is still the clamp count, and §Risks plus §Deferred both name "the pull's clamp count" as the sole operator signal feeding the deferred `013-B4`/`027-C3` bound amendment. So a store that answers with control-byte-laden titles is normalized invisibly, and a name that becomes in-budget *because* it was stripped never reaches the count that decides whether the bound needs re-sizing. Fix: one word — "counts the names it normalized (clamped or control-stripped)" — or, if the split is deliberate, one clause here saying control-strip events are not counted as bound pressure. No new counter, constant, or log line either way.
- [nit][new] §Options / §Risks — "of the games-only set (§The store pull normalizes, never drops)" and "keeps the record (§The store pull owns the rule)" — the revision introduced two parentheticals where the `§` sigil binds to a clause rather than a heading; there is no section named "The store pull normalizes, never drops" or "The store pull owns the rule", so the cross-reference resolves only by guesswork, unlike the document's other `§`-refs to real headings. Fix: put the assertion outside the sigil — "(§The store pull: it normalizes, never drops)" / "(the rule is §The store pull's)". Preference only; the meaning is recoverable.

## Complexity verdict (checklist #7)

Proportionate. The changed text is a net *subtraction* of complexity: one false
reuse claim replaced by a two-line std filter, no new package, module, interface,
dependency, constant, configuration knob, or abstraction anywhere in the diff, and
the deleted sentence in §The steam snapshot resolver removes a duplicated copy of
a claim policy step 5 still makes once. The design's heaviest parts (issued
document, retired-id list, mutation lock, page budget family) all trace to
`027-C4`/`027-C5`/`027-L2`/`027-B3`/`027-B4`, and the second half (curation face,
runbook) is already sequenced behind the first in Rollout 4–6.

## FYI (no severity)

- For the correctness seat, offered as an observation and not a demand for churn: my r1 fix's chosen disposition stores `name: ""`, and `027-C3` in the ADR says the snapshot is "refused by name outside its shape, bounds, and the name bound `013-B4`". Under the registry's record gate that value is admissible (`corpus.rs:424`, upper bound only) and nothing here breaks; under the shipped ingest wording for the same bound (`upstream.rs:247`, "outside the 1-{N}-octet bound") an empty name sits at the edge of what `013-B4` spells. The design's loader refusal as written (`over MAX_NATIVE_NAME_OCTETS or carrying control bytes`) is narrower than the ADR phrase, and that narrowing predates this revision — the empty-name clause is simply the first case that leans on it.
- The `char::is_control()` strip is a superset of both control readings in the repo, so whichever way the loader's "carrying control bytes" is implemented, the pull cannot publish a value it rejects.
