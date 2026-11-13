# security review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` (5 hunks: Context, Options considered,
`input.rs` paragraph, `document.rs` paragraph, Worked examples), read against
`02-proposal.md` as shipped and `docs/adrs/012-input-vocabulary-and-reach-classes.md`.
Inputs: the proposal, the ADR, the Revision log, the diff. Previous security
findings: none, so nothing here is re-raised or a regression of a closed item.

## Checklist walk

1. **Trust boundaries** — unchanged by the diff. The v2/v1 reader split the
   Context hunk restates ("the existing version gate refusing a v2 document
   for a v1-only reader (A2)") still classifies a foreign-major document as
   something to refuse, and the third-party/`reach-classes/v1` row of the
   Compatibility matrix is untouched. No data source is newly relabelled
   trusted.
2. **Parsing untrusted data** — the hunks touch no parsing rule. The closed
   entry set, the pair encoding, the closed escape set (A3) and the size
   budget referenced by `parse_file ... under the same budget` all survive
   the diff verbatim. No code evaluation or decompression is introduced.
3. **Path handling** — the r2 path rule (`load_geometry`: `name` matched
   against `[a-z][a-z0-9-]*` *before* the join, refusal "naming no path",
   Test strategy #4 refusing `..` and a separator before any file is read) is
   outside every hunk and still reads intact. Nothing in the diff reintroduces
   a path into a message or a shell string.
4. **Secrets/PII** — nothing in the diff concerns stored or exported data; the
   echo paths it does brush against (Hunks 3 and 4) keep their printable-escape
   guarantee. Hunk 4's "as an echoed id does (A11)" is a grammar fix that makes
   the claim *more* explicit, not weaker: the echoed declared version takes the
   same printable-escape-and-truncate path as an echoed id.
5. **Dependencies** — the diff names none. The design's std-only, "no new
   dependency, `Cargo.lock` byte-unchanged" position (Compatibility matrix,
   Rollout #3) is untouched, so there is no advisory/CVE surface to check.
6. **Privileges** — unchanged; the diff grants nothing new.
7. **Fail closed** — every refusal verb the diff touches still asserts a
   refusal ("refusing a v2 document", "refuses cleanly", "a structured
   refusal, not a verdict"). No hunk converts a refusal into a permissive
   default or a partial apply.

## Findings

- [nit][new] Chosen design — `input.rs`/Context terminology created by Hunk 1
  ("physical keys from an **ANSI-US** reference geometry") — the hunk
  normalises the Context to the ADR's spelling but leaves A4 carrying the old
  one ("names physical keys from a **US-ANSI** reference geometry"), so the
  document now names its one reference geometry two ways: `ANSI-US`
  everywhere else (Goals, A10, the id grammar, `ansi-us.toml`, the Risks row)
  and `US-ANSI` in A4. This matters only because the design's own security
  posture is that the geometry must be *stated*, not inferred ("the reason the
  spec states the geometry rather than inferring it (A4)") — a label that
  drifts inside the citing assumption is the kind of wobble that invites a
  second name in `spec.md`. Fix: in A4, change "US-ANSI reference geometry" to
  "ANSI-US reference geometry"; no other text changes.

## FYI (no severity)

- Hunk 2's rewrap leaves one overlong line: "reasons the ADR's Alternatives
  section states; this design only adds that the id shape" runs past the
  ~80-column wrap the rest of the document keeps. Formatting/editorial seat.
- Hunk 2's "the reasons the ADR's Alternatives **section** states" now matches
  the ADR's actual heading `## Alternatives considered` more loosely than the
  heading name; wording seat, not behaviour.
- The Revision log's r2 entry ("grammar-checked `name`, bounded signed deltas")
  is the closure that carries the path-traversal protection; the editorial diff
  leaves both the closure text and its test (#4) intact, so the security
  property from r2 is unaffected by this pass.
