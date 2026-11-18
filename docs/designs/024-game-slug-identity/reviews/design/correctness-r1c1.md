# correctness review r1c1 (dissent pass)

verdict: approve

Scope: `git diff 3012370e c326ff4a` of `02-proposal.md` plus the full current
document. Inputs per the caller: the proposal, ADR-024, its Revision log, and
this seat's r1 findings. Code claims introduced by the revision were verified
against the repository (read-only): `schemas/action-ids/v1/spec.md:52-59`,
`schemas/action-ids/v1/fixtures/invalid/segment-past-63-octets.toml`,
`crates/core/tests/action_ids.rs:113-128`,
`crates/discovery/src/steam_common.rs:82-118`,
`apps/discovery-cli/tests/cli_tests.rs:181-190`,
`tools/orchestration/src/pipeline/types.ts:286-287`,
`.github/workflows/check.yml`, and the fixture-enumeration pattern in
`crates/core/tests/` (conformance.rs scans `fixtures/invalid/` directories, so
the new manifest `id-past-cap.toml` needs no enumeration re-pin; action_ids.rs's
explicit list is the sole exception and is now named). No new external-source
claims were introduced in the changed text, so the r1 URL spot-check stands.

## Round-1 finding closure

1. [major] Latin fold vs ASCII case-fold order — **closed for real**. The
   table now "each entry emitting its lower-case ASCII base (`Å`→`a`,
   `É`→`e`)" and runs before the ASCII case-fold (Derivation; Options (c)
   reworded to match); Test strategy #1 adds upper-case `Å`→`a`, `É`→`e`.
   The ADR's literal C2 step order carried the same accented-caps loss; the
   Goals section declares the Chosen design the canonical statement, and the
   Revision log records the reorder explicitly. Survey cases (`nier-automata`
   from `NieR:Automata™`, `g-7-days-to-die`) still derive identically under
   the new order; `ø`/`æ`/`ß`/`ḗ` → `-` unchanged.
2. [major] action-ids `spec.md:58` stale example — **closed for real**. A3
   now cites `spec.md:54,58`; verified line 58 carries "a 64-octet segment";
   the amendment text replaces it with "a segment past the shared bound" and
   names the renamed `segment-past-cap.toml`; Rollout item 2 says "the
   action-ids refusal example included". The "each has a fixture ... naming
   it" sentence stays true post-amendment.
3. [minor] `action_ids.rs` re-pin site — **closed for real**. Verified the
   literal at `crates/core/tests/action_ids.rs:124`; Test strategy #6 now
   names `action_ids.rs` re-pinning to `segment-past-cap.toml`, Rollout
   item 1 lists it, and the rename lives in item 2 of the same commit
   (items 1+2 commit-atomic), so the enumeration list can never point at a
   renamed-away fixture at any commit boundary.
4. [minor] plugin-side refusal visibility — **closed via the second option
   this seat offered**: count-only is now stated ("Plugin-side refusals are
   count-only ... 024-L1's named per-game refusal is the CLI-boundary line
   of the next section, alone"), consistent with the verified guard at
   `steam_common.rs:101`. One residual created by the new wording is
   reported below as a new minor.
5. [minor] `Platform`'s nonexistent serde precedent — **closed for real**.
   "mirroring `Platform`'s pattern" is gone; verified
   `cli_tests.rs:186-189` carries the reverted-rename comment and
   `"SteamLinux"` pin; the bullet now states the mixed-spelling divergence
   as deliberate and pins the value to build as `"unknown"`.
6. [nit] two rationale texts — **closed**. One rationale text, at the
   `schemas/README.md` bullet, copied verbatim by the `placement.rs` doc
   comment; the surveyed-title clause is stated as "replaced rather than
   repeated (it is unverified, A16)". The ADR-024-B1 rationale sentence is
   literally dropped; the rebuttal (A16: the verified survey found a
   64-octet slug, not a 127-octet title) is sound, so this is not
   re-raised — see FYI for the ledger/compatibility seat.
7. [nit] commit atomicity — **closed**. Rollout states items 1+2 commit-
   atomic, 4+5 commit-atomic, the rest branch-level with item 7's gates at
   branch scope; the `cli_tests.rs:183` five-field pin rides the 4+5 commit.
   Test strategy #11's CI claims verified against `.github/workflows/check.yml`:
   `cargo test --workspace --locked` runs on ubuntu-latest only; the Windows
   leg is the existing `cargo check --workspace --all-targets --target
   x86_64-pc-windows-msvc` (runs no test binary); no macOS leg.

## Findings

- [minor][new] Chosen design → Emission, refusal and collisions — "they
  increment the existing `skipped` counter (whose message counts files, so a
  refused row counts there as a skipped row)" — count-only is the accepted
  closure, but the new parenthetical bakes in a conflation that misreports in
  the one case the counter is ever user-visible: its only surface is the
  `steam_common.rs:101` error, whose message reads "`{}` file(s) skipped as
  unreadable or malformed". When every game a plugin finds is refused for an
  un-derivable name — exactly the case ADR-024's Consequences foresees ("a
  game whose display name and install directory are both non-Latin gets
  refused at discovery") — the diagnostic fires and attributes readable,
  well-formed files to "unreadable or malformed", naming neither a game nor
  the actual cause. Fix: in this bullet, split the fired message's counts (e.g.
  "M file(s) skipped as unreadable or malformed; K game(s) refused") or drop
  the "unreadable or malformed" attribution from that count.

## FYI (no severity)

- The README bullet (024-L3's one stated home) drops ADR-024-B1's literal
  "one octet above the longest surveyed store display title" rationale,
  stating it is "replaced rather than repeated (it is unverified, A16)".
  Correctness-wise the rebuttal is sound and closure is accepted; the
  ledger/compatibility seats may want to confirm the accepted ADR's prose is
  allowed to differ from the amended contract's published rationale without
  an ADR-side note.
