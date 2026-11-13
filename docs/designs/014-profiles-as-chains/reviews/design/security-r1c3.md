# security review r1 (c3 ratification pass)

verdict: approve

Scope as given: the diff `ec0ea47..2aae9c9` of `02-proposal.md`, then the full
current document. Inputs: the proposal, `docs/adrs/014-profiles-as-chains.md`,
the proposal's Revision log, and my r1 security findings as quoted by the
caller. No review reports or transcripts opened.

Checklist coverage (all evaluated): 1 trust boundaries — unchanged text still
classifies every opened source (`fixture, data/profiles/, hub bundle, or
personal chain directory`) under one source-independent gate, A12 keeps
`data/profiles/` untrusted; `Chain` still "comes only from `load_chain`" and
`links` is private, so the newly named `parse_file` single-document read path
cannot feed `resolve`. 2 parsing budgets — untouched, and the new
identity-cancelling refusal is fail-closed. 3 path handling — step 2's
slug-constructed direct-child rule untouched by this diff. 4 logging/sharing —
see findings B and C. 5 dependencies — the design names none ("std-only, no new
dependency, `Cargo.lock` byte-unchanged"), so no advisory check applies.
6 privileges — read-only chain directory; the only write named (Test #5's
"write it beside `a`'s head") is the consumer side, and finding A concerns its
name collision. 7 fail-closed — merge refuses the whole result, the `fork`
shell stays unloadable, `ChainLinkMissing` is named.

## Findings

- [minor][new] Chain operations → **`merge`**, and Test strategy #5 — "the
  output carries the caller's `id` (`fork`'s collision rule against `p1` and
  `p2`)" — naming the set is new in this revision, and the set named is
  narrower than the one applied everywhere else in the same paragraph pair:
  `fork` refuses "if it collides with any link id in the chain", `diff` now
  refuses "against every link id of `a`", and `merge` refuses only against
  `p1` and `p2` — not the base the two patches both name in `derives`, which
  `merge` has in hand (it already refuses a `derives` mismatch, so
  `p1.derives[0] == p2.derives[0]`). Under the same-major-directory rule a
  link's path is exactly `<major dir>/<slug>.toml`, so a merged patch whose
  `id` equals that shared base id has the base document's file name: the
  file-level append move this revision made the sanctioned round-trip ("write
  it beside `a`'s head, `load_chain` it as the new head") then overwrites the
  base link. A hostile shared chain picks the patch pair and lets the consumer
  supply the id, so the clobber happens before any guard runs; the load guards
  only fail closed *after* the file is gone (the merged link's own `derives`
  equals its id, which is `Cycle`, not a refusal to write). Checklist: path
  handling / hostile input must fail closed. Fix: one clause — "`fork`'s
  collision rule against `p1`, `p2`, and the `derives` base id they share" —
  plus the same words in Test #5's merge clause. No new mechanism; it is the
  rule `fork` and `diff` already state.

- [minor][new] Test strategy #7 + Provenance labels + the fixture tree — the
  tree gained "`generated-by-bidi-char.toml`", which creates a refusal message
  carrying a bidi formatting character, and no text says how it stays out of
  the log. Test #7 asserts "every invalid fixture is refused with a
  single-line, control-free, truncating message", and Provenance says the label
  "reaches logs only through `one_line`, the same escape-and-truncate path
  whenever a refusal echoes it (A13)". `one_line` is defined in A13 as escape
  plus truncate for control-bearing strings; a U+202E-class or tag-block
  character is not an ASCII control character, so the refusal for the new
  fixture can echo the exact codepoint the fixture exists to ban while still
  passing "single-line, control-free" — the renders-as-authority case in the
  log line, which is what the ban was added to close. Fix: pin it beside the
  `forged-log-line.toml` clause in Test #7 — the refusal for a `generated-by`
  value in the banned non-ASCII classes names the entry and the offending byte
  offset instead of echoing the value (or state that `one_line`'s escape set
  covers exactly the classes the grammar bans), and assert the message contains
  none of the fixture's banned codepoints.

- [minor][re-raised] Provenance labels / The contract entry table — "`generated-by`
  | string | no | 1–128 octets, no ASCII control, default-ignorable, or
  bidi-control characters" — my r1 finding is only partly closed: the fixture
  half landed (`generated-by-bidi-char.toml` is now in the invalid tree), the
  definition half did not — the row is byte-identical to the one I flagged, and
  neither class is pinned. The new fixture makes that concrete rather than
  theoretical: it must exist as a file containing *some* specific character,
  and `spec.md` says nothing about which, while the Compatibility matrix makes
  "`profile/v2/spec.md` + fixtures" the whole contract for a "Third-party
  reader, non-Rust". `default-ignorable` still has no fixture and no range at
  all, so a non-Rust implementer and the Rust parser may still disagree about
  whether a U+E0020 tag or U+F0000 private-use-adjacent default-ignorable in a
  label is refused. Fix (unchanged from r1): state the classes in the row
  without a Unicode table — `Default_Ignorable_Code_Point` plus the explicit
  ranges U+202A–U+202E and U+2066–U+2069 — and add `generated-by-default-
  ignorable.toml` beside the bidi fixture so both classes have a pinned case.

## FYI (no severity)

- Cross-file loading step 2 — "the constructed path is always a direct child
  of the head" and "which pins the lookup but not the value" — my r1 nit is
  untouched by this diff (it is not in the changed sections), so it is neither
  a regression nor in this pass's scope; it stays open as a wording nit only.
- Revision log: the r1 entry was compressed and no longer records the "depth
  pinned only in `chain_ops` tests, v2 re-issue of `chain-past-max-depth.toml`
  withdrawn" decision or "`generated-by` is a public label, ignorable and bidi
  characters refused". Nothing became false — the Goals bullet and Test #3's
  "the sole depth pin" still state the depth decision, and the entry row still
  bans bidi — but a reader who consults only the log loses those decisions, and
  the log is the run's record of deliberate non-changes. Documentation seat's
  call.
- The merge-conflict example flipped ("a `clear X` under a `swap X Y`" →
  "`swap X Y` with both bound in the base, upper `clear X`"). Both directions
  look non-representable only because `merge(p1, p2, id)` never sees the base
  and so cannot name the base bindings the composition needs; that reason is
  stated nowhere. Whether the example is representative is the correctness
  seat's read, not mine — the refusal itself is fail-closed, which is what my
  seat requires.
- Dependency checklist item is satisfied vacuously: "no new dependency,
  `Cargo.lock` byte-unchanged", so no advisory/CVE lookup applies.
