# Security review r2 (editorial confirmation pass — scope: reviews/design/editorial-r2.diff)

verdict: approve

## Scope and method

Single editorial confirmation pass of round 2. Inputs: the r2 editorial diff,
`02-proposal.md` (post-edit), `docs/adrs/025-discovery-identity-and-catalog-matching.md`,
and the Revision log. I verified the code-side claims the changed passages rest
on with read-only greps: `escape_and_truncate` is exported from `bindforge-core`
(`crates/core/src/error.rs:190`, re-exported `crates/core/src/lib.rs:110`), and
`is_slug_name` / `MAX_SLUG_OCTETS` / `MAX_NATIVE_NAME_OCTETS` are exported as
the design states (`crates/core/src/lib.rs:83,114,116`). No new dependency is
introduced by the diff (the client is std-only; `ureq` stays rejected), so
checklist item 5 (dependency advisories) is not applicable to this scope.

## Hunk-by-hunk security read

- Context hunk ("exempt from the shared slug grammar and from `024-B1`, and
  rides its own bound"): grammar only; the 025-C1 exemption statement is
  unchanged in meaning.
- Friendly-name clamp hunk ("both within `MAX_NATIVE_NAME_OCTETS` and on a
  `char` boundary"): grammar only; the clamp bound and its joins-nothing
  justification are intact.
- `schemas/README.md` hunk ("and is explicitly *not* a slug-grammar bound"):
  copula added; the not-a-grammar-bound clause stands.
- Match-store load hunk: pure line re-wrap. The `65_536`-octet read budget,
  slug/`game_id` load validation, refusal-before-enumeration, and the
  explicit statement that both the store file and the server response are
  untrusted inputs are byte-identical in content.
- `probe()` hunk ("unless every returned `SocketAddr`'s
  `IpAddr::is_loopback()`"): the possessive makes the post-resolution
  loopback-address check clearer, not weaker; the hosts-file spoof defense is
  intact.
- Refusal-echo hunk: pure re-wrap; the full list of interpolated values routed
  through `bindforge_core::escape_and_truncate` is unchanged.
- Scan-flow step 6 hunk: repairs a malformed nested code span around the
  stored record tuple; content identical.
- Step 8 and boundary-tests hunks: a comma and a lowercased "match store";
  no semantic change.
- Test strategy 4 hunk: "defence" → "defense" (spelling).
- Test strategy 10 hunk: "a body over 65 536 bytes" → "a body over 65 536
  octets", which aligns the test wording with the spec's "`65_536` body
  octets" cap — terminology tightened, semantics unchanged.

No security-relevant sentence was weakened, and nothing the revisions touched
contradicts ADR-025. Deferred items (TLS/auth/ADR-026 transport, server-side
`?slug=` reading) are untouched by this diff and remain consistent with the
ADR's `025-R1`.

## Findings

None. All eleven hunks are language-only; every fail-closed, size-cap,
untrusted-input, path-handling, and escape property I check is preserved
verbatim or clarified.

## FYI (no severity)

- The r2 editorial diff renamed the unit in Test strategy 10 from "bytes" to
  "octets" to match the spec; a correctness seat may note the reverse drift
  nowhere — I checked the response-parse bullet, and octets is now consistent.
