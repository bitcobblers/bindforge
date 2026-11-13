# complexity review r1 c3

Scope: the diff between the r1-approved blob (`ac9cb8a`) and the current blob
(`2aae9c9`) of `02-proposal.md`, then the full current document. Ratification
pass: checking only whether the post-approval revisions broke what this seat
approved in round 1.

verdict: approve

Value check (checklist 7): the r1 revisions are net complexity-*reducing*.
`Chain`'s `links` field became private ("only `load_chain` builds one"), the
multi-file fixture `chain-past-max-depth.toml` was dropped from the corpus in
favour of a `chain_ops` test pin, "mount at any rung" became a file-level move
rather than an operation, and the composite cost statement keeps the Options
decision "no new aggregate cap". No new package, module, dependency, or
configuration knob appears in the changed text; `MAX_CHAIN_DEPTH`,
`MAX_DOCUMENT_BYTES`, `MAX_PATCH_DELTA_ELEMENTS` and the shipped slug predicate
are the only bounders, each with a stated consumer and named fixtures. Nothing
the revisions added is a second mechanism for a job one already ships, except
the one clause noted below.

## Findings

- [minor][re-raised] Options considered → "Profile ids" / The contract → `id`
  entry row and **Id grammars** — "`id` and each `derives` element are
  slug-grammar checked and capped at 63 octets" — the 63-octet cap still has
  no provenance and no owning name. It is absent from A14's basis (the shipped
  slug grammar, no length cap), absent from Rollout 2's named constants
  (`MAX_PATCH_DELTA_ELEMENTS` / `MAX_GENERATED_BY_OCTETS`), and absent from
  the Compatibility matrix's shipped limits — yet the entry table, the Id
  grammars rule, `id-past-cap.toml` and Test strategy #1's "64-octet" case all
  enforce it. The r1 edit sharpened this contrast rather than closing it: the
  same Options bullet now cites a shipped constant for the sibling cap
  ("mirroring the shipped `MAX_MANIFEST_ROWS = 256`"), so the document shows it
  knows how to source a bound and left this one unowned. Not YAGNI — the cap
  is wanted, the directory rule wants a bounded path segment — so it stays a
  traceability gap. Fix: one line. If manifest/codec/preset ids already carry
  a length cap, cite it in A14's basis; if not, name the constant beside the
  two in Rollout 2 (it is a parse-time bound like them) and state the reason
  in the Id grammars rule: the profile-id↔file-name directory rule needs a
  bounded path segment.

- [minor][new] The contract → `generated-by` entry row — "1–128 octets, no
  ASCII control, default-ignorable, or bidi-control characters" — the two new
  character classes are a validation surface with no Goal trace and no home in
  the text. Goals state only "Provenance entries validated referentially (they
  must name real things) and consumed by nothing"; no Options bullet, Risks
  row, or Revision log r1 entry explains why ignorable/bidi classes are refused
  (the r1 log covers `derives` cardinality, chain operations, and the model —
  this contract change is unlogged). The implementation cost is disproportionate
  to the benefit: the matrix commits to "std-only, no new dependency,
  `Cargo.lock` byte-unchanged", and std exposes no predicate for
  default-ignorable or bidi-control codepoints, so the check is a hand-rolled
  codepoint-range table — a fourth closed table inside a parser whose shipped
  posture (A19) is a *closed escape table* — plus a new fixture
  (`generated-by-bidi-char.toml`). Meanwhile the protection those classes are
  reaching for is already shipped once: A13's `one_line` escape-and-truncate
  runs on every document string echoed in a refusal, and the Provenance labels
  section now says exactly that ("reaches logs only through `one_line`").
  Fix: restore the row to "1–128 octets, no ASCII control characters" and lean
  on `one_line` for echo safety (drop `generated-by-bidi-char.toml` and the
  matching Test strategy #1 scope); if bidi filtering is genuinely wanted, keep
  it but state the requirement it serves in a Goal or an Options line and say
  in the same row which codepoint ranges count, so the table is a cited list
  rather than invented at implementation time.

## FYI (no severity)

- `merge(p1, p2, id)` pins the collision check to "`p1` and `p2`" while
  `fork`/`diff` check "every link id in the chain": a merged id that collides
  with a deeper link survives until `load_chain` refuses it as `Cycle`. merge
  only holds two documents, so it may not be able to check more — the shape of
  that set is a correctness question, not a complexity one.
- The Revision log's r1 entry omits the contract-level edits it also made
  (`derives` cardinality fixtures `derives-two-elements.toml`, the `id`
  `..`/`a/b` refusals, the `generated-by` character classes) though it logs the
  model and operations edits — the run's editor/other seats may want the log to
  cover them.
- `diff` requiring the deepest links to be the *same document* ("equal parsed
  `Profile`s") reads a whole-document equality per call; harmless at 32 × 1 MiB
  worst case, but the equality notion (structural, ids unique only per
  directory) is load-bearing in a way the earlier "same base id" rule was not.

## Checklist coverage

1. Reuse: no new package/module beyond the already-planned `chain_ops.rs`; the
   revisions reuse `check_dir_version`, `parse_file`, `ActionId::parse`,
   `one_line`, `MAX_MANIFEST_ROWS` — no duplicate helper appears in the changed
   text. (Repo sources beyond the named inputs were not opened, so shipped-
   constant existence was read off A14/A6/A17/A13 as stated.)
2. Goal trace: all moving parts trace except the `generated-by` character
   classes above.
3. Smaller design: the r1 text already is the smaller design (private `Chain`,
   fewer fixtures, no mount operation).
4. New dependencies: none — matrix row confirms std-only, `Cargo.lock`
   unchanged; that is what makes the bidi/ignorable check costly.
5. Deferrable half: chain operations remain a separable slice; nothing in the
   revisions pulls the deferrable half into the first one.
6. Knobs/abstractions: no config knobs; `ProfileKind`, `Chain`,
   `ResolvedBinding` each have one consumer; the `id: &str` parameters on
   `diff`/`merge` avoid inventing an id-generation policy — a net simplification.
