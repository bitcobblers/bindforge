# compatibility review r1c5 (ratification pass — changed text only)

verdict: approve

Scope as directed: `git diff 07e8b3f8 e1e63cb6` over `02-proposal.md`, then the
full current document. Seat: compatibility only. Checklist applied:
`references/compatibility.md`.

## Status of my r1 finding (the one carried into this pass)

**Closed by the changed text, and closed the way I asked.** §The serving face
step 1 now reads "none equal to `.` or `..`, and no control byte or
invalid-UTF-8 sequence" — `space` is gone, so the extra refusals are exactly the
named path-safety and line-integrity classes, and the sentence's claim that
"that shipped pair is the registry's whole key rule" is again true against the
shipped evidence (`Source::spells_lookup_key` + `025-B1`, with `lookup_key`
imposing nothing further). The dependent seams were carried with it: the
revision log names the r3 deletion and its reason; §Caching mode step 4 and
Test #7 pin the outbound hop, which is the one place the widened alphabet could
have leaked. I re-verified that fix rather than trusting it: the copied builder
exists as claimed (`apps/discovery-cli/src/catalog.rs:174-196`,
`percent_encode` escapes everything outside the RFC 3986 unreserved set, so a
space leaves as `%20` and a `%` as `%25`), and the outbound target is composed
from the *decoded* key (`catalog.rs:435-436`), which is what step 4 now
mandates. Nothing else in the document still assumes a space ban (grep for
`space`: only the new A23/#7 wording and the revision log).

## What I re-checked in the changed text (compatibility seat)

- **A23 and the one-answer-per-connection posture — verified, and it is the
  compatible choice.** `HttpCatalog::candidates` (`catalog.rs:408-446`) builds a
  fresh `TcpStream::connect_timeout` per call, writes exactly one
  `GET … Connection: close`, and reads one response; there is no connection
  reuse anywhere in the shipped client, and the CI fake answers one request per
  connection then shuts both halves (`tests/common/mod.rs:203-222`). So
  `http.rs` closing after one answer, `serve.rs` dropping "that connection's
  whole request loop", and Budgets' "which answers the one request §http.rs
  closes" are all true of, and compatible with, the shipped client `026-C2`
  pins the contract to. The over-cap-at-accept claim also still holds: zero
  bytes back is the client's named `Unreachable` (`catalog.rs:290-294`).
- **The sixth copied invariant** — `apps/registry`'s list, §Deferred's
  "six copied invariant blocks", and the revision-log dependency line agree, and
  #7 pins the new copy. The copied item is `pub`-adjacent pure code in a
  bin-only crate, so "copied, not shared" is the only available mechanism, as
  stated.
- **Matrix rows (checklist 1)** — unchanged by the diff; four rows, every one
  with a status, no `unknown`, no empty cell.
- **Platform tests (checklist 7)** — unchanged: #19 still names Linux through the
  cargo gates and Windows/macOS through the #13 env-closure seam plus case-unique
  fixture names. The §Catalog document reword ("#19's case-unique-fixture-name
  claim for a case-insensitive filesystem") narrows a pointer, not coverage: the
  fixture-name test is still explicitly kept un-guarded over catalog's `.json`
  fixtures, and the Windows `Yes` row makes no case-insensitive-filesystem claim
  that the new wording leaves unbacked.
- **Paths/config-dir conventions (checklist 6)** — the deleted `Defaults:` block
  loses nothing load-bearing: `--host`/`--port`/TTL defaults are in §CLI surface,
  the local-entries default `<data>/bindforge/registry/catalog-v1.json` is named
  at §`apps/registry`, the cache path is `026-C7`'s (quoted in §Options) and both
  caching documents are named on the Windows row; the admin-token posture is
  fully pinned by §The admin face and auth and #9.
- **The new `422`** — `026-C4` fixes "validates against `026-C3` and refuses by
  name" and fixes no status code for it, and `026-C2` reserves `404` for "any
  other route, and a malformed slug" and `503` for the cold case, so a named
  `422` for a well-formed-but-unusable upsert contradicts no ADR clause. It is
  flagged as an A21-dependent design choice per the register's own caveat, and
  the `MAX_REQUEST_BODY_OCTETS` = document-cap parenthetical stays true under it.
- Items 2 (OS input APIs), 3 (Node floor), 4 (native modules), 5 (keyboard
  reality): not applicable to this design's surface; the only OS-conditional
  dependency (`winreg`) is treated below.

## Findings

- [nit][new] §Deferred / out of scope, lift bullet ("Lifting the six copied
  invariant blocks … which also gives `Source`/`spells_lookup_key` that shared
  home") — the r4 rewrite of this bullet dropped the explicit retirement of
  `apps/registry`'s `discovery` edge "with its Windows-only `winreg` transitive"
  that r1 stated here, while §`apps/registry` still promises "retired by the
  deferred lift below" and the revision-log dependency line still asserts the
  retirement. The deferral now carries only half of the promise the body points
  at: the lift's named scope is the six client blocks plus a home for
  `Source`/`spells_lookup_key`, and the OS-specific-dependency consequence has to
  be inferred. Fix: restore the deleted clause into the same bullet — "…, which
  also gives `Source`/`spells_lookup_key` that shared home and so retires
  `apps/registry`'s `discovery` edge with its Windows-only `winreg` transitive" —
  no new scope, one clause.
- [nit][new] §The serving face step 1 ("Percent-decode `slug` **first**") read
  with the r3 space admission — the widened alphabet makes an unspecified part of
  the decode seam load-bearing. `percent_encode` (A19) never emits a bare `+` and
  escapes `+` as `%2B`, so a percent-decode that also folds `+` to a space (the
  form-urlencoding habit) turns a shipped-legitimate `local/foo+bar` key into
  `local/foo bar`. Before r3 that mistranslation died loudly on the space ban;
  now it yields a *legal* key, which the caching registry then forwards
  (`%20`-encoded) and caches under, answering a different question than it was
  asked. The design never authorizes `+`-folding, so this is underspecification
  at a seam the revisions moved, not a contradiction. Fix: in step 1, pin the
  decode to percent-escapes only — "percent-decode (a literal `+` stays `+`; the
  only space spelling is `%20`, as the copied `percent_encode` emits)" — one
  parenthetical, and #3's `404` list is unchanged by it.

## FYI (no severity)

- Test #10: "an upsert whose serialized corpus with its bumped `revision` *would
  pass* `MAX_CATALOG_DOCUMENT_OCTETS` is that named `422`" reads inverted against
  the body's "one over … is the named `422`"; language/clarity pass may want
  "would exceed".
- §http.rs says the writer answers once "then clos[es] the connection" but never
  says whether the response itself carries `Connection: close`. The shipped
  client is indifferent (it sends the token itself and reads exactly
  `Content-Length`), so this only matters to a hypothetical keep-alive client,
  which would learn of the close by EOF — correctness seat if it wants the header
  named.
- Performance seat: two changed costs — cache eviction re-serializes once per
  dropped entry (bounded by `026-B1`'s 5,000), and one-request-per-connection
  puts a fresh TCP setup on every upstream hop of a cold scan (which `026-R3`
  already books).
- Compatibility checklist items 2-5 remain inapplicable; item 1/6/7 verified
  above against the changed text, with no regression found.
