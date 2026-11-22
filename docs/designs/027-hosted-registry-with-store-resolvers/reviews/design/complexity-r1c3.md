# complexity review r1 (dissent pass c3)

verdict: request-changes

Scope: `git diff 19dcdcb1 1d288993` of `02-proposal.md`, then the full current
document. Inputs: the proposal, ADR-027, its Revision log, my r1 findings.
Code claims checked directly against the tree (`crates/discovery/src/game.rs`,
`crates/discovery/src/lib.rs`, `apps/registry/Cargo.toml`,
`apps/registry/src/corpus.rs`, `apps/registry/src/upstream.rs`,
`crates/core/src/manifest.rs`).

## Findings

- [major][new] §The store pull — "The rule is the shipped `discovery::friendly_name`
  one, through the existing `discovery` dependency (A7) — whitespace collapse then
  the longest `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`, **with control
  bytes dropped in the same pass**" (also A7's new clause, §Options "Partial snapshot
  serving" — "an over-bound name is normalized, not dropped", §The hosted lookup
  policy step 5 — "every record the pull was handed is in it", §Test strategy #6,
  Rollout 3). The clause is false against the code A7 cites as `verified`:
  `crates/discovery/src/game.rs:102-113` is `display_name.split_whitespace()
  .join(" ")` then an `is_char_boundary` clamp — those two operations and nothing
  else. `char::is_whitespace` covers only `\t \n \u{b} \u{c} \r` and U+001C–U+001F /
  U+0085, so NUL, `\u{1}`–`\u{8}`, `\u{e}`–`\u{1b}` (ESC), DEL and the C1 range all
  survive into the snapshot name. That matters because the design's own loader refuses
  exactly that content — §The steam snapshot resolver: "a `name` over
  `MAX_NATIVE_NAME_OCTETS` **or carrying control bytes** (the `013-B4` bound `027-C3`
  names — the pull normalizes one before writing it, §The store pull)" — so with the
  shipped rule as literally cited, the instance publishes a document its own loader
  will not read: every restart takes the "else pull in full" branch (a full 12-page
  keyed pull where a usable file sits on disk), and a restart during a store/key
  outage — §Risks' own A6 row — is the named startup refusal instead of serving the
  last snapshot. That is the remote-chosen denial of supply r3 exists to delete,
  arriving back through the loader. Second, concrete: §Test strategy #6 pins "a
  bound-breaking or control-byte `name` is clamped to the `013-B4`-legal form and its
  record stays in the snapshot" — behavior `friendly_name` does not have, so the suite
  fails as written or the implementer adds a strip step the document says is already
  shipped, which is the mechanism r3 recorded as deleted ("a bookkeeping subsystem the
  shipped clamp makes redundant"). A7's *own* description of the shipped rule —
  "whitespace collapse plus the longest `char`-boundary prefix inside
  `MAX_NATIVE_NAME_OCTETS`" — is accurate and contradicts the body clause; the
  document contains both. Fix: delete "with control bytes dropped in the same pass"
  and name the one line the registry owns instead — the pull seam drops
  `char::is_control()` characters before the `friendly_name` clamp (a std filter: no
  constant, no document, no policy branch, no ADR amendment) — and mirror it in A7 and
  Rollout 3's "`friendly_name` name normalization". Keep the loader's control-byte
  refusal as the contract-document discipline it is. One clause plus one named filter
  replaces the false reuse; nothing else the r3 deletion removed needs restoring.
- [nit][new] §Budgets — "`MAX_STEAM_PAGE_OCTETS = 12_582_912` — the response *read*
  budget for one page, 50k records at ~240 octets". The relabel fixes my r1 #3
  ("worst-legal") and the residual is now correctly in §Risks; only the arithmetic
  misleads: 50,000 × ~240 = 12,000,000, while 12,582,912 is 12 MiB (≈252 octets per
  record), so the sentence reads as a product that does not produce the constant. Fix:
  "— 12 MiB, which the 50k-records-at-~240-octets estimate sits inside —". Re-deriving
  the budget then lands on the shipped number.
- [nit][new] §The store pull (and §Test strategy #6) — the normalization has no
  stated disposition for a name that becomes empty: a whitespace-only title (and, once
  the strip above lands, an all-control title) clamps to `""`, which r2's pass-through
  could not produce. The identity is safe (mint step 3 mints `steam-<appid>`), but the
  issued record and the served candidate carry `name: ""`, where the shipped ingest
  rule treats empty as a fault ("`name` is outside the 1-128-octet bound",
  `apps/registry/src/upstream.rs:247`) and the record gate the issued document reuses
  checks only the upper bound (`apps/registry/src/corpus.rs:424`), so nothing refuses
  it. Fix: one clause — "a name that normalizes to empty is stored empty: its label
  renders blank and its identity is step 3's `steam-<appid>`" — or substitute one
  space. No mechanism either way.
- [nit][new] A7 / §The store pull / Rollout 3 — "`discovery::friendly_name`" is not
  the shipped path. `crates/discovery/src/lib.rs` publishes `pub mod game;` with no
  crate-root re-export, and every shipped caller writes
  `use discovery::game::{… friendly_name}`. Fix: spell it `discovery::game::friendly_name`
  in A7's basis and Rollout 3, so the import the code review pins is not misnamed.

## Closures (my r1 findings — all four answered by this revision)

- r1 #1 (major, `dropped_appids`): closed as ordered — the field, its loader refusals,
  the delta set-merge/self-clear/reset rules, the policy step-5 branch with its
  `steam snapshot incomplete`, `MAX_DROPPED_APPIDS`, the ADR `amended-by` and the two
  Test-strategy rows are all gone; `027-C3`'s snapshot and `steam_snapshot` shapes ship
  verbatim (confirmed against the ADR's `027-C3` text, which lists
  `{version, fetched_at, apps: [{appid, name}]}`), and no long-named title is
  permanently unanswerable. What the closure leaves open is only the control-byte half
  of the premise, above.
- r1 #2 (major, the cap's number and breach consequence): closed — the constant is
  gone with the list, so neither the 600k legal drop count nor the remote-chosen named
  pull failure has a home left.
- r1 #3 (minor, "worst-legal" label): closed — now "the response *read* budget … not a
  worst case, since over-bound store names make a page's true size unbounded", carried
  as the §Risks residual with a stated posture. Only the arithmetic nit remains.
- r1 #4 (nit, "would breach"): closed — "A delete whose write breaches that cap, on the
  rename-buffer measure above", and Test #5 re-pinned so a no-op re-delete against an
  at-cap list completes the removal.

## FYI (no severity)

- Pre-existing, not this diff: `apps/registry/src/corpus.rs::record_from_value` — the
  loader the issued document reuses verbatim — validates `name` only against
  `name.len() > MAX_NATIVE_NAME_OCTETS`, so a hand-edited corpus or issued record can
  carry a control byte in `name` onto the candidate face that this design's new
  snapshot loader refuses by name. The manifest-side rule
  (`crates/core/src/manifest.rs::check_native_name`: 1–128 octets, no ASCII control
  character, no leading/trailing space) is stricter than the registry's record gate.
  Security/correctness seats' call.
- §Risks row 7's residual is now explicit: an over-`MAX_STEAM_PAGE_OCTETS` page is a
  named page refusal that keeps the previous snapshot until `027-B2`'s age bound, and
  §Compatibility matrix makes an aged snapshot fail a bound CLI scan closed — so
  bulk long-named titles can age steam answers out without touching the key. A
  performance/security seat reads whether that reads as a denial path.

## Value check

With the drop/bookkeeping subsystem gone, the design is materially smaller than the
version I reviewed in r1 and every remaining moving part traces to a stated Goal
(`027-C1`–`027-C6`, `027-L1`–`027-L5`, `027-B1`–`027-B4`); no new dependency beyond
the named `rustls`/`webpki-roots` pair, and the one reuse this round adds is through an
existing crate edge. Complexity looks proportionate. The single open item is that the
smaller design must own the control-byte strip it currently credits to the shipped
clamp.
