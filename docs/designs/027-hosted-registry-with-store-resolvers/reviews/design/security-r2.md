# Security review r2

verdict: approve

Scope: the editorial round-2 diff
(`reviews/design/editorial-r2.diff`) against `02-proposal.md`, with
`docs/adrs/027-hosted-registry-with-store-resolvers.md` and the proposal's
Revision log as inputs. This is the single editorial confirmation pass: I read
only the changed hunks and the sections they land in, and reported only where a
revision changed meaning. The injected previous-findings list is empty, so
there is nothing of mine for the revisions to have broken.

All 14 hunks were checked against the current `02-proposal.md` and are applied
verbatim; every one is orthography, comma placement, hyphenation, or a
verb/article insertion (`bookkeeping that` → `bookkeeping ensuring that`,
`New flags:` → `New flag:`, `overlapping only re-fetches` → `an overlap only
re-fetches`, `a delta merge the old` → `a delta merge holds the old`,
`retirement-refusal` → `retirement refusal`, `stands outside` → `leaves
outside`). None moves a security posture.

Checklist walked over the diff's security-bearing passages:

1. Trust boundaries — unchanged. Store display names stay "attacker-chosen
   content" (§The store pull); the corpus/issued/retired/snapshot documents keep
   their loader refusals. `§Deferred`'s reworded `local/<dir>` clause now reads
   "the source document's Consequences *leaves* outside its ledger", which
   matches the ADR's Consequences ("a separate small amendment proposed
   (outside this ADR's ledger)") — a correction, not a widening.
2. Parsing untrusted data — unchanged. `MAX_STEAM_PAGE_OCTETS` as a page *read*
   budget, the metadata-gated load caps, the envelope/`apps`-list refusal, and
   the over-cap page as the §Risks residual are untouched text. The
   `144 MiB (16 + 64 + 32 + 32)` boot figure stands (arithmetic re-checked), and
   the "a delta merge *holds* the old snapshot, the merged model, one serialize
   buffer and the current page body" fix states the same transient as before, now
   grammatically — the memory claim is unchanged, not raised or lowered.
3. Path handling — untouched by the diff; `<data>` resolution, owner-only
   `0700`/`0600`, fixed-tmp `create_new` and the `write.lock` refusal are as
   approved.
4. Secrets and PII — unchanged. Key stays environment-only
   (`BINDFORGE_REGISTRY_ADMIN_TOKEN` precedent verified at
   `apps/registry/src/main.rs:45,121`); "every pull-failure line names the
   condition and status, never the URL or the key" and the `027-L4`
   no-queried-key-to-the-store assertion (Test strategy #6) are byte-identical
   apart from one comma after "Without it". The file-custody migration for both
   secrets stays `§Deferred` — consistent with `027-C3`, so not re-opened.
5. Dependencies — the `rustls`/`rustls-webpki`/`webpki-roots` floors and their
   RUSTSEC references are in §Options, which the diff does not touch; no new
   dependency appears in any hunk, so no new advisory surface. No unfixed
   advisory introduced by this pass.
6. Privileges — unchanged: the store hop is one compile-time-pinned HTTPS
   authority with no operator-configurable URL; `endpoint.rs`'s gate retiring
   does not open an outbound path (verified: that gate only parses `--upstream`,
   `apps/registry/src/endpoint.rs:17,70`).
7. Fail-closed postures — unchanged: named `503`s, per-page attempt/deadline
   refusals, `retired id list full` and `mint cap reached` all mutate nothing,
   and Test strategy #4's "and file untouched, and nothing served" gained only a
   comma. The Test #6 transport-assurance pair (self-signed cert *for* the
   verified name, so the case fails on the chain rather than the hostname) is
   unaltered apart from punctuation.

Evidence note: re-reading my own earlier r2 report was attempted through the
`git diff` allow and the sandbox denied it (`Permission denied: shell`), so this
pass proceeds from the caller's injected previous-findings list (empty) rather
than from that file. That gap blocks nothing: the pass's scope is the diff, and
the diff introduces no open blocker or major from this seat.

## Findings

None.

## FYI (no severity)

- The separate `MAX_ISSUED_DOCUMENT_OCTETS` for the issued document is
  expressible without forking the loader: the shipped
  `Catalog::load_required`/`Catalog::load` take the octet cap as a parameter
  (`apps/registry/src/state.rs:180,210`), so §Issued entries' "same loader
  refusals, private separate budget" claim holds. Ownership: correctness.
- The `026-C3` record loader validates `name` for presence and the upper bound
  only (`apps/registry/src/corpus.rs:420-429`), so a name that normalizes to
  empty at the pull seam does produce a loadable issued record — §The store
  pull's "a name that normalizes to empty stays empty" does not publish a
  document its own loader refuses. Ownership: correctness.
