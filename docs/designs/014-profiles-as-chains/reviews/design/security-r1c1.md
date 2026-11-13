# security review r1 (dissent pass c1)

verdict: approve

Scope: `git diff ac9cb8a..ec0ea47` of `02-proposal.md`, then the full current
document. Inputs: `02-proposal.md`, `docs/adrs/014-profiles-as-chains.md`, the
proposal's Revision log, my own r1 findings. Checklist:
`references/security.md`.

## Status of my r1 findings (all closed in this revision)

- [closed] major — `derives` array ambiguity. The entry row now reads "at v2 a
  `patch` carries exactly one element and a `base` none", the per-link rule
  names the refusal fixture (`derives-two-elements.toml`), step 2 walks "each
  link's single `derives` element — a v2 chain is one path, not a tree", and
  the composite sentence now counts documents opened ("one resolution opens at
  most `MAX_CHAIN_DEPTH` documents — one per link"). The Risks row and Test #1
  follow. Both readings are no longer available; the ≤ 32 × 1 MiB figure the
  Risks row leans on is now true by construction. Withdrawal of the v2 re-issue
  of `chain-past-max-depth.toml` breaks nothing shipped: the existing
  corpus-boundary assertion in `crates/core/tests/chain_guard.rs` is hardcoded
  to `version_dir("profile", "v1")`, and the v1 fixture stays.
- [closed] minor — path handling. Step 2 now states the property ("the
  constructed path is always a direct child of the head,
  `<major dir>/<parse-validated slug>.toml`, and no other entry names a
  location (the slug grammar refuses `..` and `/` as `InvalidValue`)"), Test #1
  adds the `..`/`a/b`-shaped `id`/`derives` case named as `InvalidValue`, and
  Test #3 keeps the containment case.
- [closed] minor — source classification. "Every document `load_chain` opens —
  fixture, `data/profiles/`, hub bundle, or personal chain directory — parses
  under one version gate, byte budget, and chain guards, source-independent;
  trust stays with adoption (Decision Four)." Matches ADR Decision Four; the
  transport itself stays deferred to ADR-018 consistently.
- [closed] minor — `diff` base identity. "requires the two chains' deepest
  links to be the *same document* — equal parsed `Profile`s, ids being unique
  only within one directory", with Test #5 refusing chains whose deepest link
  differs "by id or by contents".
- [closed] minor — `generated-by` as a loggable/shared field. The public-label
  sentence and `one_line` are stated, and `forged-log-line.toml` pins the
  single-line, control-free, truncating message. The grammar tightening that
  closes it opens the small gap reported below.

## Findings

- [minor][new] Provenance labels / The contract entry table — "`generated-by` |
  string | no | 1–128 octets, no ASCII control, default-ignorable, or
  bidi-control characters". The r1 closure adds two new character classes to
  the grammar, but nothing defines or pins them: the invalid-fixture list still
  names only `generated-by-past-cap.toml` and `generated-by-control-char.toml`,
  and Test strategy #1 still tests "`generated-by` past 128 octets or
  control-character-bearing" only. Since the compatibility matrix makes
  "`profile/v2/spec.md` + fixtures … the contract" for a "Third-party reader,
  non-Rust", an undefined Unicode class with no fixture leaves two
  implementers free to disagree about whether a bidi override or a
  default-ignorable codepoint in a label is refused — precisely the
  renders-as-authority case the ban was added to close.
  Fix: extend the existing pair with one fixture carrying a bidi formatting
  character (or add `generated-by-bidi-control.toml`), add it to Test #1's
  `generated-by` clause, and state the classes in the entry row in a form a
  non-Rust implementer can implement without a Unicode table: `Default_Ignorable_Code_Point`
  plus the explicit ranges U+202A–U+202E and U+2066–U+2069. This replaces no
  mechanism; it pins the rule the revision just added.
- [nit][new] Cross-file loading: `load_chain`, step 2 — "the constructed path is
  always a direct child of the head, `<major dir>/<parse-validated slug>.toml`"
  and "mirroring `load_manifest`'s id↔directory equality, which pins the
  lookup but not the value". Two wording slips in the sentence that carries the
  containment property: "a direct child of the head" names a document as if it
  were a directory, and "but not the value" sits immediately after a rule whose
  subject is the `id` *value*, so it can be read as waiving the id↔filename
  check that stops one file answering for another link's id.
  Fix: "a direct child of the head's directory", and replace "which pins the
  lookup but not the value" with "which pins the lookup, not the document:
  equal ids in different directories are not the same link" — the sentence's
  own closing clause ("a chain's identity is its documents' contents") already
  says this.

## FYI (no severity)

- `merge` inherits `derives`/`for` and composes two 256-element lists over
  disjoint actions, so it can emit a patch over `MAX_PATCH_DELTA_ELEMENTS` that
  only refuses at load — same category the revision already handles for
  `fork`'s shell; belongs to the correctness/test seat.
- `Chain { links: Vec<Profile> }` is stated as private with "only `load_chain`
  builds one", but no accessor is named for the "prefix or the documents
  singly" a consumer is allowed to take; API-surface question for another seat.
