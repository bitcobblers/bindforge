# Security review r1 (dissent pass c1)

verdict: approve

Scope: the diff `7b5cec46..fcb66a83` on `02-proposal.md`, then the full current
document. Inputs: the proposal, ADR-027, the Revision log, and my round-1
findings. Checklist: `references/security.md` (trust boundaries, untrusted
parsing, path handling, secrets/PII, dependency advisories, least privilege,
fail-closed).

## Previous findings — closure check

1. **Store-hop authentication never exercised (major)** — CLOSED. §The store
   pull now names "one TLS listener in the same family for the
   transport-assurance case", and Test strategy #6's **Transport assurance**
   paragraph drives the *production* `StoreTransport` (real rustls config,
   webpki-roots anchors, server-name verification) against a loopback listener
   with a self-signed cert and one for another name, refusing both; #17 keeps
   the no-outbound rule by dialing `127.0.0.1`. This is the prescribed test.
2. **Over-bound store name = permanent remote denial (major)** — CLOSED. The
   r1 posture is the third option prescribed: the record is dropped, the
   snapshot publishes `incomplete` (§The steam snapshot resolver), while it
   stands every steam non-hit is the named `503 steam snapshot incomplete`
   (§The hosted lookup policy step 5), appid hits/corpus/issued keep
   answering, health carries the fact, and "no partial list is ever certified
   either way" (§Options, Partial snapshot serving). Re-sizing stays deferred
   with the posture named in the deferral. Two residual seams in the new text
   are reported below as minors; the theme itself is closed.
3. **Advisory floors for rustls/webpki-roots (major)** — CLOSED and verified
   against the RustSec advisories themselves: RUSTSEC-2026-0346 (rustls,
   patched `>=0.23.46`), RUSTSEC-2026-0382 (rustls-webpki, patched
   `>=0.103.16` / `>=0.104.0-alpha.8`), RUSTSEC-2026-0381 (webpki-roots,
   patched `>=1.0.10`, incl. the 0.26.x-compat-re-exports-a-1.x-parent trap
   the design now states verbatim). §Options carries the floors, Rollout 3
   adds the RUSTSEC screen to the gate, and #17 records the cargo-job change.
   `ring`'s advisories (RUSTSEC-2025-0009/-0010) are old-version entries the
   screen and ≥0.17 resolution cover.
4. **Retired-id exhaustion posture (minor)** — CLOSED. "A delete whose retired
   append would breach the cap is the named refusal `retired id list full` and
   mutates nothing; failing open there re-opens a retired id, which is
   `027-L2` broken by arithmetic" (§Issued entries), pinned by Test #5; the
   load side is metadata-gated.

## Findings

- [minor][new] §The steam snapshot resolver (loader refusals) / §The store
  pull (response rules) — the loader rejects "a `name` over
  `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound
  `027-C3` names — **the pull never writes one**, §The store pull)", but the
  pull's only name disposition is the drop for a record "whose `name` breaks
  the `013-B4` bound", and `013-B4` is *only* the 128-octet budget
  (`docs/adrs/013-game-manifests.md`: "013-B4 `MAX_NATIVE_NAME_OCTETS = 128`")
  — the no-control-character rule is a separate clause of the shipped native
  name grammar (`crates/core/src/manifest.rs::check_native_name`, which also
  bans empty and leading/trailing-space names). A store record with a control
  character (or a leading space) in its attacker-chosen display name (A11)
  therefore passes every pull rule and gets published, and the snapshot's own
  loader refuses the result: every later boot takes the forced full-pull path,
  and once the non-rotatable key ends (Risks row 8) the unloadable snapshot
  converts to a startup refusal. The claim "the pull never writes one" is
  asserted by the r1 revision without a rule that makes it true. Fix: extend
  the pull's single drop-and-bump-`incomplete` clause from "breaks the
  `013-B4` bound" to "breaks the loader's name rule" (length, control bytes,
  empties, leading/trailing space — the `check_native_name` family), keep it
  the only dropped case, and let Test #6's drop case cover one fixture per
  clause. Same class covers the names copied into minted issued records: the
  snapshot admitting a name the `026-C3` loader would refuse contaminates the
  issued document at mint time.

- [minor][new] §The store pull (merge sentence) vs §The steam snapshot
  resolver (definition) — "`incomplete` is how many records **the pull**
  dropped for the `013-B4` name bound; `0` marks a worldlist, and the only
  kind allowed to certify a miss" defines the mark per-pull, while the merge
  rule says only "a delta merges by `appid` … and re-persists; a drop for the
  `013-B4` bound bumps `incomplete` rather than failing the page". Under the
  per-pull reading a delta with no fresh drops republishes `incomplete: 0`
  over a merged `apps` list that still lacks the record an earlier pull
  dropped — a silently incomplete worldlist regains the right to certify
  `200 []` for exactly the appid it is missing, reintroducing the dishonest
  certification the r1 posture exists to prevent, and the attacker needs only
  one planted over-long name plus the passage of one delta. Fix: one sentence
  in §The store pull — a delta's published `incomplete` is the loaded
  snapshot's value plus its own drops (dropped records never enter the merged
  `apps` list, so nothing ever decrements it); only a full pull can publish
  `0` — and pin it in Test strategy #6's delta case.

## FYI (no severity)

- §Hosted mode re-pins health `steam_snapshot` to
  `{games, fetched_at, incomplete}`, but `027-C3` as written carries
  `{games, fetched_at}` or `none` — the added field is a silent amendment to
  an ADR-pinned contract the design's Rollout 1 ledger annotations don't list;
  compatibility seat's call.
- The `--corpus` exception to the leftover-document refusal is shape-safe as
  far as I checked (caching mode already loads `catalog-v1.json` as a
  `026-C3` Catalog, `apps/registry/src/state.rs:210`), but it hands the
  curated-corerank of `027-L5` to a file the operator chose; correctness seat
  may want the runbook to say what a cache-era local-entries document
  shadowing a minted identity implies.

## Checklist notes

Trust boundaries, path handling, secrets/PII, and least privilege: no change
in the revised text reopens them (key stays env-only, no-URI-in-logs,
`026-C4`/`026-C5` auth posture extended verbatim to the new admin route;
`write.lock` is a same-user-directory guard, not a privilege change). The
newly named dependency surfaces were screened against RustSec (finding 3
above). Malformed/hostile input: the two minors above are the residual
fail-closed gaps; everything else in the revised pull/loader rules fails
closed by name. No prior major or blocker remains open.
