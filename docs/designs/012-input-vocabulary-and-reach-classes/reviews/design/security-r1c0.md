# security review r1

verdict: request-changes

Scope: whole document (full-panel pass c0, round 1). Inputs: `02-proposal.md`,
`docs/adrs/012-input-vocabulary-and-reach-classes.md`, the Revision log (empty —
no prior round; every finding below is labeled `new`).

## Findings

- [major][new] "Placement legality: `schemas/keyboard-geometry/v1/` and `placement.rs`" —
  "`load_geometry(dir: &Path, name: &str)` — `parse_file` on `dir.join(format!("{name}.toml"))`" —
  the caller-supplied `name` is interpolated into the path before any grammar check:
  the `[a-z][a-z0-9-]*` geometry grammar is enforced only on the *file's* `geometry`
  entry, after the load. A `name` containing `..` or a path separator traverses out of
  `dir`, and `Path::join` with an absolute argument silently *replaces* `dir` entirely —
  so a caller that forwards user-selected geometry names (the CLI, 015's policy layer)
  reads arbitrary `*.toml` files. The `geometry_matches_file_name` rule is a post-load
  check and cannot prevent the read. Security checklist 3 (path handling) is not met by
  the text as written. Fix: state that `load_geometry` validates `name` against the same
  `[a-z][a-z0-9-]*` grammar *before* the join and refuses a non-matching `name` with the
  existing `InvalidValue` shape (A11, no path in the message) — one sentence replacing
  the unspecified pre-join behavior; `geometry_matches_file_name` stays as the post-load
  rule. This is the minimal in-contract fix: the grammar already exists, it just has to
  be applied before the path is built.
- [minor][new] "The model: `crates/core/src/input.rs` and the reach edits" —
  "`reject_unknown_entry`'s message names that version instead of hard-coding `v1`" —
  the named version is now document-controlled text, and TOML strings can carry newlines
  and unbounded length; the design cites the A11 "single-line, truncating, path-free"
  message shape but nowhere says the echoed version goes through that escape/truncation,
  unlike the explicit escaping required for echoed ids ("echoing the id escaped to
  printable characters"). A hostile document can inject a multi-line or megabyte-long
  refusal into whatever logs refusals (checklist 4: what may be logged). Fix: one clause
  requiring the echoed version to pass the same printable-escape-and-truncate path as
  echoed ids.
- [minor][new] "Placement legality" predicate steps 2–3 —
  "`coordinate(target) − coordinate(source)` is the same `(dx, dy)`" and
  "the action's degradation is `rank(class(to-hand, target)) − rank(class(from-hand, source))`" —
  subtraction over parsed values without a stated arithmetic regime. The design itself
  requires negatives ("A negative degradation (an improvement) never blocks"), and
  "non-negative integer coordinates" puts no upper bound on a coordinate literal, so an
  adversarial-but-grammatical geometry (e.g. a 300-digit coordinate) or a plain improving
  move can overflow/underflow and panic rather than "reject cleanly, never crash"
  (checklist 7). Fix: one sentence fixing the arithmetic regime — signed or checked
  comparison (e.g. legality is `rank(target) ≤ rank(source) + bound` with checked add,
  deltas computed in signed integers) and a stated coordinate ceiling (or "parsed with
  checked integer conversion") so malformed data fails closed.

## Checklist coverage

1. Trust boundaries — evaluated: sources are the repo-owned contract tree and
   caller-supplied directories/documents; the design treats every parsed document under
   the closed entry sets, closed member grammars, and the invalid-fixture corpus, i.e. an
   untrusted-input posture. The one unclassified flow is the caller-supplied `name`
   reaching a path — finding 1.
2. Parsing untrusted data — size budget carried ("under the same budget"), closed schema,
   closed escape table, no code evaluation, no decompression: satisfied. Coordinate
   magnitude unbounded — folded into finding 3.
3. Path handling — not satisfied as written: finding 1 (pre-join validation). Case-unique
   name rules cover the Windows/macOS collision surface.
4. Secrets/PII — no secrets or PII surface; refusal messages are path-free and the geometry
   mismatch is refused "without naming a local path": satisfied except the echoed-version
   gap (finding 2).
5. Dependencies — the design names none: "no new dependency, no `Cargo.lock` change",
   std-only. Toolchain floor 1.85.0 checked against public advisories: no unfixed advisory
   touching this design's surface (read-only parsing; no shell, no `remove_dir_all`).
6. Privileges — filesystem reads only, no device or network access; least privilege holds,
   contingent on finding 1's containment of the geometry read.
7. Fail closed — refusal fixtures cover the grammar surface, and the predicate returns
   "a structured refusal, not a verdict"; the arithmetic-crash gap is finding 3.

## FYI (no severity)

- The `classes`-array-order-equals-severity rule makes the two hand tables interdependent
  data (both must carry one canonical list); the parse-level pinning is sound, but the
  grade data itself (e.g. left hand: `keyboard:-` = `far`, right hand: `stretch`) is
  authored-data territory for the correctness seat, not a security concern.
