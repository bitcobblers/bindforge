# Security review r2

verdict: approve

## Scope

Single editorial confirmation pass, round 2. Scope is the diff
`reviews/design/editorial-r2.diff` (five hunks) against `02-proposal.md`, with
`docs/adrs/017-keybinding-extraction-codecs.md`, the Revision log, and my
previous findings (none — the editorial phase is one language pass per round)
as the only supporting inputs. This is neither a dissent nor a ratification
pass on prior security findings; there were none to re-check.

## Diff-by-hunk security read

1. **§Codec family id** — "is `[a-z][a-z0-9-]*` at most" → "of at most
   `MAX_SLUG_OCTETS` octets": grammar only. The bound itself is untouched and
   verified real: `crates/core/src/placement.rs:83` defines
   `MAX_SLUG_OCTETS: usize = 128`, exported from `crates/core/src/lib.rs`, and
   `schemas/manifests/v1/spec.md:56` shows the manifest `codec` entry riding
   exactly that bound — which is what the sentence claims. The ADR's
   "`013-B4` bounds" (`docs/adrs/017-keybinding-extraction-codecs.md:76`) is
   recorded as a slip in the Revision log ("Initial: §Codec family id"), and
   the A6 basis stands. Trust-boundary item 1 and validation item 2: unchanged.
2. **§Read discipline (aggregate budget)** — "the product also ceilings the
   *opens*" → "caps the *opens*": a verb swap that makes the sentence agree
   with the "caps a tree's reads" clause it cites. The decompression-bomb bound
   (item 2) is untouched: `(16+1) × 32 × 1 MiB = 544 MiB` worst case per game
   still reads correctly and matches `MAX_CANDIDATE_FILES = 32`,
   `MAX_USER_DIRS_PER_GAME = 16`, `MAX_BINDING_FILE_OCTETS = 1_048_576` in the
   §Read discipline table.
3. **Test strategy #3 (`user_trees` cap)** — "holding 9 and 8 digit
   directories" → "holding 9- and 8-digit directories": disambiguates the
   compound adjective only. The fixture counts (9+8=17, keep first 16, 17th
   past `MAX_USER_DIRS_PER_GAME`) and the digit-name skip-and-count rule
   (item 3, path handling from untrusted directories) are unchanged.
4. **Test strategy #6 (locate walk)** — "other than the fixtures' others" →
   "other than those the fixtures use": repairs a garbled comparative without
   weakening the assertion — an unseen platform directory name is still found,
   "(no constant assumed)" kept, matching `017-S1`'s no-platform-constant rule
   in §The first family. Fail-closed item 7 unaffected.
5. **Test strategy #16 (`zz_api_probe`)** — "and that the only `Cargo.lock`
   delta is" → "and pins that the only `Cargo.lock` delta is": restores the
   missing verb so the sentence states a test *assertion*. This strengthens the
   supply-chain posture (checklist item 5): it pins the §`crates/codecs` claim
   of "no new external package and no resolved version moves" and agrees with
   Rollout item 9's "any change to an external package or to a resolved version
   is a finding". No new dependency is named anywhere in the diff, so there is
   no new package whose advisories/CVEs require fetching; the existing
   workspace deps (`bindforge-core`, `discovery`, `serde`, `serde_json`,
   `clap`) are unchanged and gated by `cargo deny check advisories` in the
   Rollout gate list.

## Checklist disposition

1. Trust boundaries — not touched by the diff; classification unchanged. N/A.
2. Parsing untrusted data — budget arithmetic re-verified (above); unchanged. OK.
3. Path handling — hunk 3/4 touch test prose around digit-directory and
   locate-walk rules; the canonicalization and link rules of §Config-path
   resolvers are outside the diff and unchanged. OK.
4. Secrets/PII — the hunk-3/4 context lines restate the probe-output scoping
   ("for whoever is sitting at the machine, not for a public report");
   unchanged. N/A.
5. Dependencies — no new dependency named in the diff or the proposal
   (A12); hunk 5 adds a lock-delta pin. N/A.
6. Privileges — not touched. N/A.
7. Fail-closed — not touched. OK.

## Findings

None. All five hunks are language-only; none changes a bound, a refusal, a
path rule, a privilege, or a dependency claim. No blocker or major findings;
no minor or nit findings worth recording.

## FYI (no severity)

- None.

## Verification record

- All five post-image lines located verbatim in `02-proposal.md` (§Codec
  family id; §Read discipline; Test strategy #3, #6, #16).
- `MAX_SLUG_OCTETS` / `MAX_NATIVE_NAME_OCTETS` presence checked in
  `crates/core/src` (grep).
- ADR-017 `017-C1` wording and `schemas/manifests/v1/spec.md` `codec` row
  checked against the changed sentences.
- Revision log read in full; no entry rejects a security finding relevant to
  these hunks; no deferred item is re-opened by the diff.
- Re-read of my own prior report via `git diff` was denied by the permission
  layer; the caller states this seat's previous findings are "none", so no
  prior finding needed re-checking.
