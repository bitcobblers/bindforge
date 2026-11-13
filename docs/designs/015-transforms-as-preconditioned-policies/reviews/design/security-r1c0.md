# security review r1

verdict: approve

## Checklist coverage (security seat, r1c0, whole document)

1. Trust boundaries — covered. The policy document is explicitly the "fourth
   untrusted document class" (Risks row 1), and A8 pins that `error.rs` already
   counts policy among the untrusted classes; base/manifest/tables/geometry/
   registry all arrive through the shipped untrusted loaders with the shipped
   caps. No data source in the design is left unclassified.
2. Parsing untrusted data — covered. `MAX_DOCUMENT_BYTES` pre-read, closed
   entry set (`POLICY_V1_ENTRIES`), fixed per-kind field counts, closed transform
   kind set (no code evaluation — the free-form-predicate alternative is
   explicitly refused), `MAX_POLICY_TRANSFORMS`/`MAX_RELOCATE_TARGETS`,
   `MAX_NESTING_DEPTH` pinned by `nesting-past-limit.toml`, TOML-only (no
   compression, so no decompression-bomb surface).
3. Path handling — covered. `load_policy` slug-checks `id` before any
   filesystem question; a slug cannot carry `..`, a separator, or an absolute
   form; the file is `<id>.toml` directly in `dir`; no shell anywhere.
4. Secrets/PII and echo — covered. Refusals escape-and-truncate (A14);
   `forged-log-line.toml` pins single-line control-free messages; the produced
   base's only embedded string is `generated-by = "policy:<slug>"` (A18); no
   local path enters a message. What leaves the machine is chain + policy
   documents of grammar-checked ids; hub transport deferred to ADR-018 without
   contradicting ADR-015. See nit on the exact echo bound.
5. Dependencies — evaluated: the design names no third-party dependency
   (`crates/transforms` depends only on `bindforge-core`; std-only;
   `Cargo.lock`'s diff is exactly the new workspace package), so there are no
   external advisories/CVEs to webfetch. The new package itself ships no code
   beyond what the proposal itself describes.
6. Privileges — covered. Reads: one targeted file per load (`<id>.toml` in the
   caller's dir). No device access. The only write (the produced `<policy-id>.toml`)
   is consumer-side and ADR-017's storage question.
7. Fail-closed — covered: atomic rows, named verdicts, no partial application;
   version gate and `UnknownFormat` refuse old readers closed; pin-equality
   refuses stale links at load. One completeness gap in the id-collision
   refusal — see the minor finding; it is not partial application.

## Findings

- [minor][new] Chosen design → The output — a base in a chain ("A consumer
  regenerates by writing `<policy-id>.toml` beside the chain and bumping pins
  as a review") + The stack entry refusals ("`policy.id == base.id` — a produced
  base cannot name its own input (a chain directory holds one file per id)") —
  the collision refusal covers only the *source base's* id, yet the document
  itself states the governing fact — a chain directory holds one file per id —
  and a chain directory holds the personal patch too. A policy whose `id`
  equals any other link's id in that directory makes the prescribed
  regeneration write silently replace that file (a player's patch, or another
  base), which contradicts the ADR-015 Decision clause "a personal patch (014)
  sits above them in the chain and survives base regeneration untouched — the
  'produce bases, never overwrite players' rule". `policy.id` is document-
  controlled (the hostile-corpus threat model of Risks row 1), so this is not
  only an author-accident case. Fix: either widen the refusal in `apply` to any
  id present in the caller-supplied chain, or state beside this refusal that
  the write must collide-refuse against every id already in the chain and name
  that write-time rule as owned by ADR-017 — so the regeneration recipe in
  *this* document does not prescribe a write that can destroy the patch file
  the ADR promises survives.

- [nit][new] Chosen design → The report ("Every string a report carries is a
  grammar-checked id, so the bounded-echo posture holds without a `CoreError`
  in sight (A14)") vs. the tag bullet in The contract ("an inapplicable report
  naming the emptiness — never a silent no-op transform") — the claim is true
  for ids (≤ 63 octets, A15), but the tag *pattern* grammar has no length
  bound (the prefix is `[a-z][a-z0-9-]*`), and it is not stated whether the
  unmatched-tag report's `offender` is `None` or echoes the pattern. Not
  exploitable (grammar-checked, control-free, single-line is conformance-pinned),
  but the "bounded-echo posture" is only verifiable if the bound is named.
  Fix: one sentence in The report: `offender` is `None` for tag-level outcomes,
  or tag echoes share the 63-octet id cap — whichever is intended.

## FYI (no severity)

- The stack — "clause (a) checks the final set's distinctness" references a
  clause letter that appears nowhere in the section; correctness/compatibility
  seats may want that clause named where the match rule is "stated once".
- Per-kind checks — `relocate-cluster` fails NotApplicable on an unbound
  matched action (`unbound-action`) while `swap-polarity` deliberately passes
  unbound sides through ("unbound sides pass through, 014's `swap`
  semantics"); plausibly intentional, but correctness should confirm the
  asymmetry is the designed intent, not an oversight.
