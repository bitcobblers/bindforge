# compatibility review r1c5 (ratification of the r2 revisions)

verdict: approve

Scope as given: `git diff fcb66a83 d105d78c` on `02-proposal.md`, then the full
current document, plus `docs/adrs/027-hosted-registry-with-store-resolvers.md`
and the proposal's Revision log. Only the changed text and my own r1 findings
were reviewed; nothing outside them was re-opened. Claims the revision makes
about shipped code were checked against the code with read-only inspection
(`crates/core/src/placement.rs`, `crates/core/src/manifest.rs`,
`crates/discovery/src/game.rs`, `crates/discovery/src/lib.rs`,
`apps/registry/Cargo.toml`, `apps/registry/src/keys.rs`,
`apps/registry/src/corpus.rs`, `apps/discovery-cli/src/catalog.rs`).

## What the revisions did to my r1 findings

- Finding 3 (the `incomplete` whole-instance certification cliff) is
  **resolved**, and resolved the ADR-conformant way: the field, its loader
  refusals, its health spelling and its `steam snapshot incomplete` policy
  branch are gone, and the surviving shapes —
  `{version: 1, fetched_at, apps: [{appid, name}]}` and health
  `{"games", "fetched_at"}` — are now literally `027-C3`'s (ADR-027, Contracts,
  `027-C3`), which the r1 text deviated from. The residual moved from
  *one* attacker-chosen rename failing every pull to a whole page exceeding
  the (now larger, 8 MiB → 12 MiB) read budget. No re-opening.
- Findings 1 and 2 are **still open**; the revision edited the exact sentences
  finding 1 cited and left the conflict in place (see below).
- Finding 4 (A16's `cargo check` parenthetical) is unchanged text and stays as
  filed; it is re-listed only for record consistency.

No revised section I approved was broken by this diff. The `200 []`
entitlement sentence, the health re-pin, the retired-ids cap/crash posture, the
deadline arithmetic (`12 × (3 × 32 + 2 × 30) = 1,872 ≤ 1,920`), the
`MAX_DISCOVERY_SLUG_OCTETS` arithmetic (the shipped rule caps the **whole**
joined key at 128, `apps/registry/src/keys.rs:73`, so `steam-` + ≤122 = ≤128,
and the bound is endpoint-inclusive), the `query`-as-stopped-instance
re-characterization, and the lock-order restatement all hold against the ADR
and against the tree. The compatibility matrix is untouched by the diff and
every row keeps a status; the platform-test row (#16) still names a leg for
each supported platform.

## Checklist

1. Compatibility matrix — pass (all five rows carry a status; no `unknown`, no
   empty cell).
2. OS input APIs — not applicable: this design adds no input surface (a
   server-side resolver and two JSON documents).
3. Node floor — not applicable: §Test strategy #17 states `tools/orchestration`
   is untouched, and the diff adds no orchestration surface.
4. Native modules — pass: `rustls`/`ring`/`webpki-roots` are named with floors,
   a RUSTSEC screen, the `check.yml` Windows-leg scope split and the `README.md`
   C-toolchain prerequisite, all inside Rollout 3. The new
   `discovery::game::friendly_name` use adds no dependency (`discovery` is
   already a dependency of `apps/registry`, `apps/registry/Cargo.toml:14`), but
   see finding 1 for the contract it crosses.
5. Keyboard/name reality — pass for non-ASCII store names: the clamp keeps a
   `char`-boundary prefix (`crates/discovery/src/game.rs:107-112`), and the
   `char::is_control()` strip covers the C1 range `split_whitespace` leaves
   (A7 verified). An empty normalized name is safe end to end: the shipped
   record validator refuses only over-bound names
   (`apps/registry/src/corpus.rs:424`), and the shipped client accepts any
   string `name` (`apps/discovery-cli/src/catalog.rs:391-405`), so
   "its label renders blank" is true and boots nothing.
6. File paths — pass: the three new files ride the shipped `<data>` seam
   (`$XDG_DATA_HOME` / `$HOME/.local/share` / `%APPDATA%`) and the
   `DataLayout`/fixed-tmp discipline, with the Windows row naming them.
7. Platform test per supported platform — pass (#16: Linux via the trio,
   Windows via the narrowed cross-check plus the `DataLayout` seams, macOS
   declared dev-only, clients via #12's byte-exact contract).

## Findings

- [minor][new] §The store pull — "`discovery::game::friendly_name` through
  that existing `discovery` dependency (A7: the shipped rule is whitespace and
  clamping only, which is why the filter is ours)" — this makes a shipped
  display-convention function load-bearing for identity, against its own
  stated contract. `025-C2` says the friendly name "is never end-user visible
  ..., participates in no join, and carries no derivation law", and the
  function's doc-comment says the clamp is admissible "here and only here"
  precisely because "nothing parses the result, it is never joined to anything,
  and it is never user-visible" (`crates/discovery/src/game.rs:96-100`). The
  design inverts all three: its output is persisted into a contract document,
  served to bound CLIs as the candidate `name`, and fed to the mint (steps 1-5)
  whose output is immortal under `027-L2`. Nothing pins it: `025-C2`
  deliberately gives `friendly_name` no derivation law, so a future display-side
  change in `crates/discovery` (a different whitespace set, a different clamp)
  silently changes every identity minted afterwards, with no gate and no
  ledger annotation. Fix: have `apps/registry` own the two-line rule (a
  `char::is_control()` strip, whitespace collapse, longest `char`-boundary
  prefix) as its own named function pinned by Test strategy #6, and delete the
  A7 `friendly_name` clause's load-bearing role — that replaces the cross-crate
  call with the copy it pins, rather than adding another mechanism. If the call
  is kept instead, the smallest fix is a registry suite that asserts the two
  rules byte-exactly plus a Rollout-1 line that a change to `friendly_name` is
  a mint-stability event.

- [minor][re-raised] §Issued entries, mint steps 4/6 × §Test strategy #1 —
  step 6: "Any spelling the rule lands on that is held or retired by a
  different record refuses the mint with the named `503` `mint identity held`"
  vs #1: "a suffixed id already held takes `steam-<appid>`, a held
  `steam-<appid>` refuses `mint identity held`". Unchanged in substance, and
  the revision *widened* it: r1's fixture said "already held by a curated
  record", the new text says "already held", so the unstated
  plain → suffixed → `steam-<appid>` cascade now covers every holder. Step 5's
  fallback triggers on length only, never on held-ness, so the rule text still
  refuses what the suite pins as a fallback, and implementations match suites.
  Fix: state the cascade in step 4's own sentence ("a suffixed id that is
  itself held or retired takes `steam-<appid>`; if *that* is held or retired,
  step 6 refuses"), and delete step 6's "any spelling" breadth down to the
  three named spellings.

- [minor][re-raised] §Test strategy #1 — "including the over-bound cases the
  arithmetic allows: a 128-octet ASCII name, a 128-octet digit-leading name,
  and a 128-octet *colliding* name all mint `steam-<appid>`" — the first
  fixture still contradicts §Issued entries step 5, which this revision
  restated: "The fold never exceeds the name's octets, but steps 2 and 4 add
  them". A 128-octet ASCII name with no leading digit and no collision folds to
  at most exactly 128 octets, and `MAX_SLUG_OCTETS` is endpoint-inclusive
  (`crates/core/src/placement.rs:83`), so it mints its own slug. The new
  sentence compounds it in the other direction: "an id still over
  `MAX_SLUG_OCTETS` after substitution refuses `mint identity over bound`" is
  unreachable through any admissible key, since `keys.rs` caps the whole joined
  key at 128 and `steam-` costs 6. So the suite pins a legal at-bound slug as a
  fallback and pins a production-impossible refusal. Fix: drop "a 128-octet
  ASCII name" from that fixture list (the digit-leading and colliding fixtures
  are the over-bound ones) and mark the `mint identity over bound` case a
  unit-only defensive branch or delete it.

- [nit][new] §Options and §Risks — "Nothing else a legal pull is handed makes
  the snapshot short of the games-only set (§The store pull normalizes, never
  drops)" and "The pull seam normalizes the `name` to the `013-B4`-legal form
  and keeps the record (§The store pull owns the rule)" — both cross-reference
  section titles that do not exist; the section is "The store pull (traces
  `027-C3`, `027-B4`, `027-L1`, `027-L4`...)". This document's tracing
  discipline reads those anchors as pointers. Fix: point both at §The store
  pull, as §Deferred already does.

- [nit][new] §Budgets — "`MAX_STEAM_PAGE_OCTETS = 12_582_912` — ... above the
  50k-records-at-~240-octets estimate ... the pinned `max_results` can return" —
  the revision introduces a legitimate-input estimate of ~240 octets per record
  for the page budget while leaving the snapshot's `027-B1` ceiling
  (`MAX_STEAM_SNAPSHOT_OCTETS = 33_554_432`) unstated against it: at 200k
  records that ceiling admits an average record of ~167 octets, so a catalog
  sitting at the per-record figure this section justifies for a page would
  breach the publish-time check, which after this revision has no fallback
  posture at all (the `incomplete` machinery is gone). Real catalogs sit far
  inside it; the finding is the one-line arithmetic, not a predicted outage.
  Fix: add the per-record figure the 32 MiB ceiling admits at 200k records next
  to the page estimate, and name the posture of a publish-time breach in one
  clause (the same `027-B1` refusal the loader applies), so the two budgets are
  sized from one table.

- [nit][new] §Risks — "the cost is a shorter label, never a different identity"
  — a clamp can change the identity, not just the label: a title whose first
  128 octets are all non-`[a-z0-9]` (an emoji or CJK run) with its Latin title
  past the bound folds empty and takes step 3's `steam-<appid>` where the
  unclamped name would have minted the slug. Harmless (the id is still unique
  and stable), but the sentence is the mitigation's own claim. Fix: qualify it —
  "a shorter label, and a different identity only where the surviving prefix
  folds empty, which is step 3's fallback".

- [nit][re-raised] §Assumption register A16 — "legitimate there only because
  `cargo check` generates no code" — unchanged from r1 and still in tension with
  §Options' own correction ("`ring` compiles C and assembly for the *target*
  through a build script"), which is exactly why Rollout 3 drops `apps/registry`
  from that leg. The cross-check is legitimate because no *other* dependency
  runs a target-native build script. Fix: reword the parenthetical that way.

## FYI (no severity)

- §The store pull's `STEAM_PULL_DEADLINE_SECONDS = 1_920` deletes the r1
  sentence that paced the pre-bind pull ("a seconds-scale retry, never an
  hour-long sleep"); a cold `serve` can now spend up to 32 minutes before it
  binds, and §Budgets notes startup pulls are "bounded only by process starts".
  That is a supervisor-interaction and quota question (performance seat), and
  the design hands the restart policy to the runbook.
- §Concurrency's "a corpus delete's write half is the one departure from the
  order as listed" is fine because the mutation mutex serializes every writer;
  whether the order's stated direction is now exactly what the implementation
  must do is the correctness seat's read.
