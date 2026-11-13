# correctness review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`), pass c0, round 1. Inputs used: the
proposal (incl. its Revision log, which records "r1: none yet" — so every
finding here is `new`), `docs/adrs/014-profiles-as-chains.md`, and the sources
the register cites (needed for checklist items 1–2: `crates/core/src/{chain,
toml,error,input,placement,manifest,document}.rs`, `crates/core/tests/
conformance.rs`, `schemas/README.md`, `schemas/profile/v1/spec.md`,
`data/profiles/README.md`, design 013 §Identity), plus two live standards
spot-checks for item 2.

## Checklist notes

- **1 Assumption register** — spot-checked A1, A2, A3, A4, A6, A7, A8, A9, A11,
  A12, A13, A14, A15, A16, A17, A18, A19, A21 against the cited sources; each
  basis supports the row (`MAX_CHAIN_DEPTH = 32` and the two refusals at
  `chain.rs:18-41`; `supported_majors()` already `["v1","v2"]` while
  `format_majors("profile")` is `[v1]` at `document.rs:139-152`; `one_line`
  escape + `MESSAGE_VALUE_CHARS = 80`; no chain-completeness variant in
  `CoreError`; `MAX_MANIFEST_ROWS = 256`; 48-member closed `keyboard:` set;
  pin spelling `"<id>@<revision>"` at 013 §Identity; `data/profiles/` holds
  only a README, so "v1 has no shipped data" holds). No verified row was found
  unsupported; A20 is honestly marked `unverified`.
- **2 Standards spot-check** — RFC 6902 fetched: "Operations are applied
  sequentially in the order they appear in the array", op set is
  add/remove/replace/move/copy/test (no `swap`), `move` = remove-then-add
  (intermediate state) — all three A20 claims hold. RFC 7396 fetched: October
  2014, "Obsoletes: 7386" (7386 is Sept 2014) — the month claim holds. The
  proposal itself carries no URLs; its research-derived claims sit in the one
  `unverified` row.
- **3, 4, 5** — findings below.
- **6 Source document** — ADR Decision One (entry list), Two (deterministic
  resolution), Four (provenance never vouches) are all decided; Decision Three
  is decided for all three operations, with the two defects in findings 1–2.
- **7 Revision rounds** — nothing to re-check: the Revision log has no prior
  entries.

## Findings

- [major][new] **Chain operations: `fork`, `diff`, `merge`** — "empty delta
  lists for the caller to fill" (fork bullet) vs the contract's "A `patch` has
  non-empty `derives` and at least one element across `set`/`clear`/`swap`"
  and the invalid fixture `patch-empty-deltas.toml` — `fork`'s output is
  always a patch with zero delta elements, which the v2 grammar refuses at
  parse, so the one operation the ADR names never returns a document that
  parses, re-parses, or loads. Test strategy #5 compounds it:
  "`resolve(fork(a) + diff, manifest)` equals `resolve(b, manifest)`" composes
  that illegal document. Fix: either state that `fork` returns a
  not-yet-loadable shell and pin that (Test #5: "fork's output refuses to load
  until a delta is added", and the round-trip property appends `diff` to `a`
  alone, not `fork(a) + diff`), or give `fork` the caller's first delta
  elements so its output is a valid patch.
- [major][new] **Chain operations: `merge`** — "When no such patch exists —
  one side clears an action the other sets, an order the fixed list order
  cannot express — `merge` refuses the whole result" — clear-vs-set on one
  action *is* expressible as one patch under the design's own rules (fixed
  order `set` → `clear` → `swap`, last write wins, determinism rule 3): upper
  patch clears → emit `clear X`; upper patch sets → emit `set X i`. So the
  stated predicate refuses pairs the same paragraph promises are fine
  ("Compatible overlaps are not conflicts") and contradicts the Options
  criterion "a conflict is representable iff the composed effect cannot be
  written as one patch". Test strategy #5 pins the wrong case: "conflicting
  patches (clear vs set on one action) refuse the whole merge". Fix: state the
  predicate as per-action non-representability under the fixed order, and give
  an example that genuinely has no one-patch form — lower patch `clear X`,
  upper patch `swap X Y` with `Y` bound: the net "`X` takes `Y`'s binding, `Y`
  unbound" needs a swap *before* a clear, which the order forbids — and
  re-word Test #5 to that pair.
- [minor][new] **Chain operations: `diff` / `merge`** — "Its output is a v2
  patch document (the diff *is* the grammar) that `resolve`s to exactly what it
  must" — when the two chains resolve to the same layout (`b` is `a`, or `a`
  plus a self-cancelling patch) every action matches, so `diff` emits zero
  elements, which `patch-empty-deltas.toml` refuses; `merge` of two mutually
  inverse `swap` patches composes to the identity, same shape. Two of three
  operations are undefined on legal input. Fix: one clause each reusing the
  existing refusal — `diff` refuses when the two resolved layouts are equal,
  naming both heads; `merge` refuses an identity composition.
- [minor][new] **Resolution — "Mount at any rung"** — "a consumer takes a
  prefix, a suffix, or the documents singly. The chain is only meaningful
  whole, so a suffix without its base is refused by `load_chain` at load" —
  the sentence offers a suffix and the next sentence refuses it; as written the
  paragraph both grants and withholds the case ADR-014 Decision Three asks for.
  Fix: delete "a suffix" (prefix and single documents are what "take its base
  and patch independently" needs) and let the refusal sentence stand.
- [minor][new] **Cross-file loading step 3 / Goals / Test strategy #3** —
  "more than `MAX_CHAIN_DEPTH` links is `ChainTooDeep` — both written against
  the constant" vs the Goals' "the same guard rules the shipped single-document
  guard states" — the shipped predicate is `derives.len() > MAX_CHAIN_DEPTH`
  (`chain.rs:27`), so today a head with 32 ancestors (33 resolved links)
  passes, while the new test pins "a chain of exactly `MAX_CHAIN_DEPTH` links
  resolves and one link more refuses". The two layers disagree at exactly one
  link, and the re-issued `chain-past-max-depth.toml` and the `chain_ops`
  boundary test can then pin different documents. Fix: name the counted unit
  once — either "more than `MAX_CHAIN_DEPTH` ancestor links (the shipped
  `derives.len() > MAX_CHAIN_DEPTH` predicate)", or keep the resolved-links
  unit and say the cross-file guard refuses one link earlier than the
  single-document guard, and write the boundary test and the fixture to the
  unit chosen.
- [minor][new] **Chain operations signatures** — "`pub fn merge(p1: &Profile,
  p2: &Profile) -> Result<Profile, CoreError>`" (and the `diff` bullet) — the
  documents `diff` and `merge` emit have unspecified `id`, `derives`, and `for`
  even though those entries are the chain lookup key and the file name, and
  must be slug-grammar and collision-free; only `fork` states the collision
  rule. Fix: one clause each — `diff` emits `derives = [<a's head id>]` and the
  joined manifest's pin; `merge` inherits `id`, `derives`, `for` from `p1` and
  refuses when `p1.id == p2.id` is not a caller-supplied fresh id (or takes the
  id as a parameter exactly as `fork` does).
- [minor][new] **The contract: `schemas/profile/v2/` fixture list** —
  "`fixtures/invalid/  unknown-entry.toml … chain-past-max-depth.toml
  self-deriving-chain.toml`" — the v2 corpus re-issues every v1 grammar
  fixture except `forged-log-line.toml`, while v2 refusals echo far more
  untrusted strings than v1's did (`id`, `derives` elements, both pin halves,
  delta elements, `seeded-from`, `generated-by`, `ChainLinkMissing`'s id) and
  the design leans on that path three times ("pass `one_line`", "echoing the id
  through the escape-and-truncate path", "ride the same escape-and-truncate
  path"). `generated-by-control-char.toml` pins the entry's own grammar, not
  the echo. Test strategy #7 asserts "every invalid fixture is refused with a
  single-line, control-free, truncating message" but no v2 fixture carries a
  `\n`-bearing value to exercise it. Fix: re-issue `forged-log-line.toml` at
  v2 with a `\n`-escaped `generated-by` (or `id`) — the existing mechanism, no
  new rule.
- [minor][new] **Resolution — `Chain` and `resolve`** — "`pub struct Chain {
  pub links: Vec<Profile> }   // deepest base first … head last`" — a public
  field makes `Chain` consumer-buildable: an empty chain (which no rule in the
  document defines an output for), a suffix, and a chain whose `kind` positions
  were never checked all reach `resolve`, `fork`, and `diff`, because step 4's
  position rules and the completeness rules live only inside `load_chain`.
  Fix: say `Chain` is only ever produced by `load_chain` (keep the field
  private, or give it a non-public constructor), and state that `resolve`
  refuses an empty chain.
- [minor][new] **The model (serialize) and Test strategy #6** — "`serialize`
  writes the ten entries in the fixed order" and "a hand-built document
  carrying all ten entries, delta array order included" — the entry table and
  the Goals define eleven (v1's four plus `for`, `kind`, `set`, `clear`,
  `swap`, `seeded-from`, `generated-by`), and the order the sentence itself
  lists names eleven. A wrong count in the section that fixes the closed set
  propagates to `PROFILE_V2_ENTRIES` and the `grammar_closure.rs` expectations.
  Fix: "eleven entries" in both places.
- [nit][new] **Cross-file loading / Resolution naming** — "`pub fn
  resolve(chain: &Chain, manifest: &Manifest) -> Result<Vec<ResolvedBinding>,
  CoreError>`" — the crate will export `resolve_chain` (validation only) and
  `resolve` (delta application), two public names one token apart doing
  different jobs, while ADR-014 Decision Two says "Resolve the chain with
  `resolve_chain`" and nothing maps that sentence onto the split. Fix: one
  sentence in the cross-file section — `resolve_chain` keeps the id-level
  validation, `load_chain` walks the files, `resolve` applies the deltas — and
  consider `resolve_layout` for the new name.

## FYI (no severity)

- Pin equality means A10's quoted ADR-015 sentence ("a personal patch …
  survives base regeneration untouched") is true only as data: a bumped base
  forces every patch's `for` to be rewritten in the same PR. That is ADR-014's
  priced cost, but whether it satisfies ADR-015's wording is a cross-ADR
  coherence question for another seat — ADR-015 was not an input here.
- Compatibility matrix "macOS contributor — no (v1)" leans on design 008's
  matrix; not checked here.
- The Deferred list matches ADR-014's "Not decided here" list (transforms,
  CLI, storage) plus 018's transport; nothing there contradicts the ADR, so
  nothing is re-opened.
