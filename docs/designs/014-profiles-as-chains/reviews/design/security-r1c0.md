# security review r1

verdict: request-changes

Scope: whole document (`docs/designs/014-profiles-as-chains/02-proposal.md`), round 1, pass c0.
Inputs used: the proposal (incl. its Revision log — "r1: none yet"), `docs/adrs/014-profiles-as-chains.md`.
No prior security report for this run was consulted or opened.

## Findings

- [major][new] Cross-file loading: `load_chain` in `chain_ops.rs` / "Resolution: deterministic" —
  "Walk `derives` nearest-first: the ancestor `<id>.toml` is looked up *directly in that same directory*"
  vs. "a resolution reads at most `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB) across at most 32
  documents" — the walk text is singular, but the entry table keeps v1's plural array ("`derives` |
  array of strings | no | ancestor chain, nearest first") and nothing says what `derives[1..]` means
  at v2, while `merge` speaks of `derives[0]`, so multi-element lists are expected to be legal. Under
  an untrusted-input format that is the one place the design's own mitigation collapses: if the walk
  follows every element, the chain is a tree, `MAX_CHAIN_DEPTH` no longer bounds documents read
  (the ≤32 × 1 MiB figure that the Risks row "Cross-file machinery is new attack surface" leans on is
  false), and the meaning of "more than `MAX_CHAIN_DEPTH` links is `ChainTooDeep`" — links in a walk,
  or links in a document — is left to the implementer, i.e. two implementers disagree about how much
  hostile input they will read. If instead only `derives[0]` is followed, every later element is an
  ancestor claim that is never resolved, so `ChainLinkMissing`'s completeness posture ("a chain is a
  contract object") silently stops covering part of the document. Fix: one rule at v2 — a `patch` has
  exactly one `derives` element and a `base` has none (v1's array is untouched, A15/A1, and the ADR's
  "nearest ancestor first" reading survives) — pin it with a new `fixtures/invalid/
  derives-two-elements.toml`, and rewrite the composite-cost sentence to state it counts documents
  opened per resolution under that rule. If the author keeps the array, the alternative is to state
  that all elements are followed, give the walk a visited-set and an explicit documents-opened
  counter, and replace the ≤32 × 1 MiB sentence with the resulting bound; do not leave both readings
  available.

- [minor][new] Cross-file loading: `load_chain` in `chain_ops.rs` — "the ancestor `<id>.toml` is looked
  up *directly in that same directory*" — this is the only place in the design where untrusted document
  content becomes a filesystem path, and the security property is never stated. It holds only because
  `derives` elements are slug-grammar-checked (`[a-z][a-z0-9-]*`, ≤ 63 octets) at parse, which excludes
  `/`, `.` and `..`; the proposal never says the constructed path's parent is pinned to the head's
  directory, never says no other entry can name a location, and Test strategy #3 has no traversal-shaped
  case — the invalid fixtures test `derives-element-uppercase.toml`, which is a case rule, not a path
  rule. Fix: add one sentence to the load rule — the filesystem path is `<major dir>/<parse-validated
  slug>.toml` and no other entry names a location; the constructed path is a direct child of the head's
  directory — and add the parse case that a `derives` element of `..` or `a/b` refuses as
  `InvalidValue` (the slug refusal already produces it; name it so a non-Rust implementer conforms).

- [minor][new] Cross-file loading / Deferred — "A hub upload bundle is exactly such a directory (ADR-018
  ships the transport; this is the shape it transports" — the only data source the proposal classifies as
  untrusted is `data/profiles/` (A12, and the Risks row says "`data/profiles/` posture unchanged"). The
  design defines a second, network-sourced input — a directory of chain documents arriving from a hub —
  and a third that its own `load_chain(head: &Path)` is written for, a player's personal chain directory,
  and neither is classified. Deferring *trust over a channel* to ADR-018 is consistent with the ADR
  (Decision Four, and Consequences' "hold against hostile uploads"), so this is not re-opening a
  deferral; it is the missing one-line classification of the sources whose guards are decided here.
  Fix: one sentence where the bundle shape is recorded (or in A12's row): every document `load_chain`
  opens — fixtures, `data/profiles/`, a hub bundle directory, a personal chain directory — is parsed
  under the same version gate, byte budget and chain guards, source-independent; trust stays with
  validation and adoption (Decision Four).

- [minor][new] Chain operations: `fork`, `diff`, `merge` — "`diff` requires the two chains to share their
  deepest link id — the same base" — identity here is the id string, not the document. Two chains loaded
  from two different directories can name the same base id with different bytes (the id↔filename rule
  gives uniqueness only *within* one directory), and `diff` then states a property — "the chain `a`
  extended by the diff resolves to `b`'s resolved layout" — that is false for the consumer who applies
  the emitted patch over the base it actually has. Pin equality at load will not catch it, because the
  ids match. That is a misattribution path for a shared patch, and `diff`'s output is exactly the
  artifact that gets shared. Fix: make the shared-base test content-bearing — require the deepest link's
  parsed `Profile` (or its `serialize` bytes) to be equal for the two chains, refusing with a named
  error when the ids match and the documents differ — and state it in the `diff` bullet rather than
  leaving it to "share their deepest link id".

- [minor][new] Provenance labels — "`generated-by` | string | no | 1–128 octets, no ASCII control
  characters: a label naming what produced the document" — this is the only free-text field in the
  format, and the checklist's "what may be logged, what leaves the machine when bindings are shared" is
  nowhere answered for it. It is exported with the document in the bundle shape defined here, it is
  echoed into logs through the `one_line` path on every refusal (A13), and the grammar permits arbitrary
  non-ASCII, so the same label can carry a personal name or email, or a bidi-override/homoglyph string
  that renders as an authority name in a UI — Decision Four says nothing consumes it as trust, which
  bounds the damage but does not make the label non-public or non-spoofable. Fix: one sentence in
  Provenance labels stating the label is a public, shared artifact that must not be treated as secret or
  displayed as authority, and that it reaches logs only through `one_line`; tighten the grammar to
  exclude default-ignorable and bidi control characters (the fixture list already carries
  `generated-by-control-char.toml`, so this reuses an existing shape).

## FYI (no severity)

- `merge`'s conflict condition ("one side clears an action the other sets, an order the fixed list order
  cannot express") looks narrower than reality given `set` → `clear` → `swap` ordering: composition of a
  `set` under a `clear` *is* expressible as a lone `clear`, while a `swap` under a later `set` may not be.
  Whether the stated conflict set is exactly the inexpressible set is the correctness seat's question.
- Rollout 5 keeps `data/profiles/README.md`'s "untrusted-input posture paragraph stands untouched" while
  the same step announces that the loader now exists; whether that README still describes reality is a
  documentation-accuracy question, not a security one.

## Checklist coverage

Trust boundaries (one source unclassified — finding 3); untrusted parsing (budgets, closed entry set,
closed vocabulary, caps present; array bound collapses with finding 1); path handling (implicit, unstated
— finding 2); secrets/PII (finding 5); dependencies — nothing to evaluate: "std-only, no new dependency,
`Cargo.lock` byte-unchanged", so no advisory/CVE check was due and none was skipped; privileges (pass —
read-only filesystem access, unchanged from `parse_file`/`load_manifest`, and `fork` explicitly does not
touch the filesystem); fail-closed (pass — `resolve`/`merge` return `Result`, `merge` refuses the whole
result, `ChainLinkMissing` is a typed refusal not an I/O leak, walk is iterative, and per `one_line` no
local path enters a message).
