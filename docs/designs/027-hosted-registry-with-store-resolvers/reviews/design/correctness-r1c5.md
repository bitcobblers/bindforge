# correctness review r1c5

verdict: approve

Ratification pass. Scope: the blob diff `19dcdcb1…d105d78c` of `02-proposal.md`
(the revision that deletes the `dropped_appids` subsystem and moves name
normalization to the pull seam, relabels `MAX_STEAM_PAGE_OCTETS` as a page
*read* budget, measures the retired-id cap against the rename buffer, and
re-pins Test #2/#3/#5/#6/#13 and Rollout 1/3), then the full current document.
I report only what the revisions broke in what I approved, or what makes a
section I cited false; round-1 findings I simply re-verified are in the
disposition section, not re-filed.

## Checklist walk

1. **Assumption register.** A7's new clause is the only register row the diff
   touches, and it is the load-bearing one for the new design. Verified against
   the code: `crates/discovery/src/game.rs:102-113` is exactly
   `split_whitespace().collect::<Vec<_>>().join(" ")` then
   `end = normalized.len().min(MAX_NATIVE_NAME_OCTETS)` walked down to
   `is_char_boundary` — whitespace collapse plus clamp, nothing else; no
   `char::is_control` anywhere in it. `crates/discovery/src/lib.rs:1-8` has
   `pub mod game;` and no `pub use`, so `discovery::game::friendly_name`
   (no crate-root re-export) is the correct path, and `apps/registry/Cargo.toml:14`
   already carries the `discovery` dependency, so it is an import, not a
   dependency. The row's "leaves non-whitespace control bytes in place" is
   right: `char::is_whitespace` covers `\t`–`\r` and the space/Zs/Zl/Zp set, so
   NUL, ESC, DEL and C1 survive `split_whitespace`. Status `verified` is earned.
   No other row's basis changed.
2. **Anti-hallucination spot-check.** No research URL entered or left the
   register in this diff (A7 only gained two *code* bases, both re-read above);
   nothing new to spot-check on this scope. Not applicable further.
3. **Internal contradictions.** The deletion is clean: searching the document
   for `dropped_appids`, `MAX_DROPPED_APPIDS` and `steam snapshot incomplete`
   leaves only the Revision-log entry that records deleting them. Cross-
   references re-resolved in the renumbered/rewritten text: "(policy step 2)" →
   §The hosted lookup policy item 2 ("a key that already has a record never
   pays the mint path again") ✓; "mint step 3's `steam-<appid>`" → step 3 is the
   empty-result fallback ✓; "equal-prefix clamps resolve at mint steps 4–5" →
   step 4 is the collision suffix, step 5 the over-bound fallback ✓; "on the
   rename-buffer measure above" → §Issued entries' "gated at load (metadata-gated)
   and against the buffer a write renames" ✓; §Options' "the pull normalizes,
   never drops" agrees with §The store pull's "Nothing is *dropped*" ✓. Rollout
   1's replacement line ("`027-C3`'s snapshot and `steam_snapshot` health shapes
   ship verbatim") now matches the ADR's own `027-C3` text
   (`{version, fetched_at, apps: [{appid, name}]}`; health fact
   `{games, fetched_at}` or `none`) — the r2/r3 shape amendment and its
   `amended-by` are gone, which is a strict improvement in fidelity to the
   source document, and the loader still refuses names outside `013-B4` as
   `027-C3` demands. Arithmetic: `12_582_912` = 12 MiB > 50,000 × ~240
   (A17's four fields with `name` at 128 serializes near 209–240 octets) ✓; the
   144 MiB document sum is 16 (`MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216`,
   `constants.rs:27`) + 64 + 32 + 32 ✓.
4. **Missing edge cases.** The core claim of the new text holds: strip
   `char::is_control()` (Cc — NUL, ESC, DEL, C1) and then clamp, and the
   snapshot name the pull writes carries no control byte, no
   leading/trailing or non-ASCII whitespace (the `split_whitespace`/`join` pass
   removes those separators), and at most 128 octets — so the pull can never
   publish a document its own loader refuses, which was exactly my round-1
   finding. A name whose record survives with an empty `name` is admitted
   downstream: the issued document is `026-C3`-validated by
   `corpus.rs::record_from_value`, which bounds `name` only above
   (`corpus.rs:424`), and the CLI's candidate parser requires a *string*, not a
   non-empty one (`apps/discovery-cli/src/catalog.rs:393-401`) — so no
   present-but-unloadable-file cliff and no refused scan, only a blank label
   (finding 3). Empty-name records mint step 3's appid id, so no two records
   share an identity.
5. **Interfaces.** One shape per thing across sections; the removed subsystem
   left no second name behind.
6. **Decides what the source document asked.** Yes, and more tightly than
   round 1: the snapshot ships `027-C3`'s shape verbatim instead of amending it,
   `027-L3`'s certification is now unconditional inside the age bound, and no
   appid's miss is answerable only by `503`.
7. **Revision rounds — closed for real, not reworded.** My round-1 finding 1
   (the control-byte gap between the drop rule and the loader's refusals) is
   closed structurally, by deleting the case rather than widening the drop
   rule's wording as I suggested. The Revision log's `r0→r4` entry states the
   reversal and its counters honestly; I do not re-open a rejected alternative
   (`dropped_appids`) — the rebuttal (cap breach returns remote-chosen denial of
   supply; a listed appid can never answer against `027-S1`) is sound.

## Findings

- [nit][re-raised] §The store pull / §Risks / §Deferred — "The publish line
  counts the names it clamped (§Risks, §Deferred)" beside "the pull's clamp
  count is the operator's signal" and "the pull's clamp count is the operator's
  signal (§The store pull)". Same class as my round-1 nit 4, which the revision
  answered by substituting a new signal for the deleted one: the signal named
  three times has no surface the design defines and no runbook duty. "The
  publish line" appears nowhere else — §The store pull names only the startup
  `registry hosted listening` line and the refresh thread's *failure* warning
  line — and the count is not in `health` (excluded, correctly, since `027-C3`
  fixes that fact to `{games, fetched_at}`) nor in `stats` (counts only, and
  Deferred's own item). Rollout 6's reading duty enumerates "reading the retired
  list's size against its cap" and never the clamp count, so the one stdout line
  at publish time is the only place the §Deferred trigger ("if a store name
  breaches it in earnest") can ever be observed: it does not survive a restart,
  and nothing tells the operator to look. Fix: one clause in Rollout 6's reading
  list — the clamp count on the pull's publish line, named where it is printed —
  beside the retired-list size it already carries; no new surface.

- [nit][re-raised] §Budgets / §Risks — "`MAX_STEAM_PAGE_OCTETS = 12_582_912` —
  the response *read* budget for one page … not a worst case, since over-bound
  store names make a page's true size unbounded and the over-cap page is the
  residual §Risks carries". The relabel closes the framing half of my round-1
  nit (the cap no longer claims to be worst-legal) but the residual is attributed
  to one cause only. Over-bound names are not the only way a *legal* page
  exceeds 12 MiB: the body carries JSON string escapes, so a name at its 128-octet
  decoded bound serializes at up to 2× when quote- or backslash-heavy (~340 octets
  per record, ~17 MiB for the 50k page) and up to 6× per character for a name made
  of characters the encoder escapes as `\u00XX`. §Risks also states the residual's
  cost as "a page refusal that keeps the previous snapshot until `027-B2`'s age
  bound", which is the steady-state half: on a cold instance with no snapshot the
  same named page refusal is the startup refusal, and the operator's only way to
  bind is `--no-steam`, i.e. every Steam key answering `503` and every bound CLI
  scan failing closed (§Compatibility matrix). Fix: the one clause §Deferred's
  item already asks for, extended — name JSON-escaped serialization, not only
  over-bound names, as the second way a legal page breaches the read budget, and
  say the cold-start branch is the named startup refusal; keep the value, which is
  arithmetically right against the ~240-octet estimate.

- [nit][new] §The store pull / Test strategy #6 — "A name that normalizes to
  empty stays empty: its label renders blank and its identity is mint step 3's
  `steam-<appid>`" and "#6 … an all-whitespace or all-control title stores
  empty". The blank is not registry-visible only: that `name` is what the mint
  persists into the issued record and what the candidate contract serves, and
  `apps/discovery-cli/src/catalog.rs:46-47` documents the candidate `name` as
  "the name the prompt shows" — accepted there as any string (`catalog.rs:393-401`),
  so a store title made only of control or whitespace characters reaches a user's
  confirmation prompt as an invisible option, chosen by remote content. Under the
  round-1 posture this record would have been dropped; the record is better kept,
  but the blank label is the one piece of attacker-chosen content the revision
  lets through to a user's decision point. Fix: one clause giving the empty case
  a label the registry owns instead of nothing — store `steam-<appid>` (already
  mint step 3's spelling, no new constant) as the `name` when normalization
  yields empty, and let #6 assert that instead of "stores empty".

## Round-1 findings — disposition

1. Control-byte gap between the drop rule and the loader's refusals — **closed,
   verified against code** (see checklist 1 and 4). The revision's fix is stronger
   than the one I proposed.
2. Mint step 5 spells the over-bound fallback from the queried key's external id
   (`steam/42` vs `steam/0042`) with the probe's canonicalization unnamed —
   **untouched by this diff**; nothing in the changed text makes it more or less
   true. Not re-filed (ratification scope); it stays an open minor for whoever
   tracks closure.
3. `MAX_STEAM_PAGE_OCTETS` derivation — **partially closed**; what remains is the
   second nit above.
4. Operator signal with no runbook line or restart-surviving surface — **not
   closed**, re-raised in its new form as the first nit.
5. Deadline clause scoped to deltas vs the pre-bind full pull — **untouched by
   this diff**; not made false by the changed text, so not re-filed.

## FYI (no severity)

- `friendly_name`'s doc comment says its clamp is "admissible here and only
  here — a truncated display string costs a shorter label, never a different
  identity (`025-C2`)", precisely because nothing in `crates/discovery` parses
  the result. This design now uses it from `apps/registry` where the mint *does*
  derive identity from the result, so the clamp does choose the derived slug
  (`steam-<appid>` becomes the plain slug for a >128-octet title). The Revision
  log owns that counter and the reasoning holds (`027-L2` + policy step 2 + mint
  steps 4–5, and equal-prefix collisions resolve to distinct appid-suffixed ids),
  so it is not a finding — but a code-review seat may want the comment's
  "only here" scope noted at the new call site.
- A7's Cargo.toml comment (`apps/registry/Cargo.toml:12-13`) documents the
  `discovery` dependency as `Source::spells_lookup_key` alone; the design adds a
  second use. Dependency-free either way — a comment touch-up, another seat's
  call.
