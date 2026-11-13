# Security review r1 (dissent pass c1)

verdict: approve

Scope: `git diff 118d4d24 f23e3e46` (02-proposal.md blobs), then the full current
document. Inputs: proposal, `docs/adrs/011-action-taxonomy-and-core-skeleton.md`,
the Revision log, my r1 findings.

## Closure check of my r1 findings

- **[major] source authority for the registry — closed.** spec.md's "Extension and
  withdrawal" now says bindforge-core answers only from the `registry.toml`
  shipped in this versioned directory and that an `action-ids` document from
  elsewhere is data, "never loaded as the registry, never consulted for
  `axis_kind`", and `load_action_registry`'s doc repeats it ("`dir` is the
  directory shipped with the crate"). That is the clause the r1 fix asked for,
  not a new mechanism. ADR-015 no longer inherits the boundary by accident. One
  narrow residual, below as a nit.
- **[minor] declared version vs. loading directory — closed.** The entry table
  states the equality, `load_action_registry` performs "a check that the
  declared `version` equals `dir`'s version segment", the mismatch is named as an
  `InvalidValue` refusal, and Test strategy #4 carries the case. Fail-closed
  (checklist 7) satisfied; the wording creates a small placement ambiguity, below.
- **[nit] untrusted classification of the `data/` placeholders — closed.**
  "whatever is dropped there arrived from elsewhere and is untrusted input once
  013/014 read it".
- **[nit] array-length bound — closed** in the permitted form: "length bounded by
  the shared parse budget", which the model ties to "the size budget binds before
  the registry is read". Bounded byte size bounds id count, so uniqueness and
  document-order construction cost is bounded; no separate cap needed.
- **[nit] echoing a hostile id — closed:** "an offending id echoed escaped to
  printable characters". The new `non-ascii-id.toml` fixture plus Test strategy
  #1's non-ASCII case give that rule an exercised case, and Test strategy #5's
  "single-line bounded message" assertion covers it.

Checklist state: 1 (trust boundaries) now classified for every source the stage
touches — shipped `registry.toml` trusted; `data/profiles/`, `data/manifests/`
and documents arriving on the shared `parse` path untrusted. 2 (size limits,
closed entry set, TOML subset, no evaluation, no decompression) satisfied.
3 (path handling): `dir` is not caller-supplied per the new clause, `dir.join`
of a fixed name, refusals path-free ("refused without naming a local path"), no
shell interpolation. 4 (secrets/PII): no personal data in the format; the only
echoed content is an id, now escaped. 5 (dependencies): none named — std-only,
`Cargo.lock` byte-unchanged — so nothing to check for advisories. 6 (privileges):
reads a shipped repo file, writes two READMEs; least privilege. 7 (fail closed):
version gate plus directory equality, closed entry set, bounded messages.
Deferred items (installed-artifact data location, ADR-013 manifest-id
validation, ADR-015 use of axis kind) contradict no Decision or requirement in
ADR-011 — the ADR decides neither where `schemas/` data lives at install time nor
any consumer, so they stay deferred.

## Findings

- [minor][new] Chosen design — The contract, `spec.md` entry table — "`version`
  (string, required, equal to the version segment of the directory holding the
  file)" — the equality is a loader-side rule, but it is now stated in the entry
  table, the place whose constraints the shared `parse` path enforces ("Any other
  entry is rejected"), and `parse` has no directory: only `load_action_registry`
  holds `dir`. Read literally, the rule invalidates the corpus that exists to
  demonstrate it — `schemas/action-ids/v1/fixtures/valid/basic-registry.toml` is
  held by `fixtures/valid/`, which has no version segment — and a third-party
  reader of the document alone (the reader this contract is written for, per the
  section's own framing) cannot evaluate it at all. The r1 closure is right in
  substance; only its placement is wrong.
  Fix: in the entry table keep `version` (string, required) and put the equality
  where the loader is described — a spec.md line saying the loader refuses a
  document whose declared version differs from the version segment of the
  directory it loads it from — so Test strategy #4's case reads as a
  `load_action_registry` case and fixtures keep one fault per file.
- [nit][new] Chosen design — spec.md "Extension and withdrawal" × The model,
  `document.rs` — "never loaded as the registry, never consulted for `axis_kind`"
  beside "`Document::ActionIds(ActionRegistry)` variant" — the residual left by
  the r1 closure: the classification is stated, but the model still routes every
  `action-ids` document, from wherever it arrived, through the `action-ids` arm
  in `parse`, which returns the same public `ActionRegistry` with `axis_kind()`,
  re-exported from `lib.rs`. Nothing in the design distinguishes the value the
  corpus round-trip builds from untrusted bytes from the one
  `load_action_registry` returns, so the guarantee is consumer discipline rather
  than a boundary. A17 bounds today's damage (nothing acts on the attribute), and
  the hazard is for 013/015, which is exactly where the ADR makes `axis_kind`
  load-bearing. Fix: one clause in The model stating that the `parse` arm yields
  the registry only as parsed data for the corpus round-trip, and that the
  registry consumers read `axis_kind` from is the value `load_action_registry`
  returns — no new mechanism, it names which value the spec's rule attaches to.

## FYI (no severity)

- The version/directory equality cannot have a fixture: fixtures carry a document,
  not a loading path, so the rule lives only in Test strategy #4. Correctness and
  Compatibility seats may want the spec to say plainly that the fixture corpus
  does not exercise it.
- The Non-goals consolidation is loss-free for this checklist: the emphasis
  refusal survives in the entry table and Test strategy #3, and the 008-reset
  non-goal survives in Non-goals and Options considered.
- Rollout renumbering fixed the Risks cross-reference ("Retargeted in Rollout 4");
  Test strategy numbering is unchanged, so the `#2`/`#3`/`#6`/`#7` references in
  Chosen design and Risks still resolve.
