# Code review — performance seat — round 1, pass c0

- Design: `docs/designs/013-game-manifests/02-proposal.md` (r1), ADR-013
- Diff: `fc71ff96f1143ecc21e01ad20faf22da45360ad2...HEAD -- ':(exclude)docs/designs'`
  (34 files: `crates/core/src/{manifest,document,placement,lib}.rs`, five test
  suites, `schemas/manifests/v1/`, two READMEs)
- Specialty: performance only (`references/performance.md`). Out-of-specialty
  observations are FYI lines with no severity.
- Verdict: **approve** — zero open blocker/major; two minor, two nit.

## Checklist results

| # | Item | Result |
|---|---|---|
| 1 | Event/keystroke path | **Pass.** Nothing in this diff is reachable from an event
or keystroke path: `load_manifest` has no caller in `crates/core/src` (only
tests and `zz_api_probe`), and `placement_is_legal` is untouched apart from the
`is_slug_name` rename. `load_manifest` (`manifest.rs:221-273`) does exactly one
file read plus one `BTreeMap::contains_key` per row — `ActionRegistry::contains`
is a map lookup (`action.rs:163-165`), not a linear scan, so the join is
O(rows log ids) ≈ 256 × log 40. |
| 2 | Complexity | **Pass with nits.** Every quadratic scan in `manifest.rs` is
bounded by a budget constant that is checked *before* the loop:
`check_row_count` at `284/436/463/505` and the group cap at `375` all precede
their element loops, so the worst case is ~65k predicate calls per entry
(FYI #1, finding 3). One bound is *not* applied before its materialization —
the group member split (finding 1). |
| 3 | Unbounded growth | **Minor.** No cache, registry, or long-lived array is
introduced: `MANIFEST_ENTRIES` is a static, `Manifest` is per-document and
dropped by the caller. The transient intermediates of a hostile document,
though, are sized by the 1 MiB document budget rather than by the manifest
budgets — findings 1 and 2. |
| 4 | Startup | **Pass.** The proposal's "size budget before the read" claim
holds: `load_manifest` → `parse_file` → `toml::read_file`, which refuses on
`metadata.len() > MAX_DOCUMENT_BYTES` before reading (`toml.rs:56-58`). Only
`manifest.toml` is opened; nothing is read at library load, and no static
initialization was added. Test-side cost is 23 fixture files, the largest
~4 KB. |
| 5 | Cache invalidation | **N/A, evaluated.** No cache is introduced, so there
is no invalidation logic to cover. The change's invalidation contract is the
data-level one — slug `id` plus monotone `revision`, joined by ADR-014's `for`
pin — and its CI matrix is Deferred by design (proposal, Deferred). |
| 6 | Benchmark-like test or timing note | **Nothing to run; checked by
arithmetic.** The diff contains no timing test or benchmark (grep for
`Instant`/`elapsed`/`bench` finds only the pre-existing `ScratchDir` timestamp
helper in `tests/common/mod.rs:188`), so there is no number to quote. The
performance *claims* in the new comments ("refused before any row is built",
"before any membership set is built", "the cap is what bounds a refusal") were
checked by reading plus worst-case arithmetic below; `cargo` could not be
executed in this review environment (the shell is restricted to `git`), so no
suite wall-clock was measured. |

## Findings

### 1. minor — `crates/core/src/manifest.rs:385` (cap at `394`)

The `MAX_GROUP_MEMBERS` bound is applied *after* the element is fully split:
`let fields: Vec<&str> = raw.split(' ').collect();` materializes one entry per
space before `if fields.len() > MAX_GROUP_MEMBERS` can refuse. A single
`exclusive-groups` element is bounded only by `MAX_DOCUMENT_BYTES` (the toml
reader has no per-value or per-element bound — `toml.rs:314-345`), so
`exclusive-groups = ["<1 MiB of spaces>"]` allocates ~1,048,576 `&str` fat
pointers ≈ 16.8 MB before the refusal fires — a ~16× transient amplification of
the 1 MiB input budget, on the untrusted-input path the design itself names
(A14, Risk row "A hostile manifest abuses the new shapes"). This is also the
one budget in the format that does not bound its own shape's materialization:
`parse_docked` (`470-475`) already avoids it with `split_once` +
`tail.contains(' ')`.

Fix: bound the split lazily before collecting, e.g.
`if raw.split(' ').take(MAX_GROUP_MEMBERS + 1).count() > MAX_GROUP_MEMBERS { … }`
(plus the `< 2` check from `raw.matches(' ').count() + 1`), and only then
`collect`. The existing 17-member case
(`tests/manifest.rs:750-760`) still covers the refusal; it would then cover it
without the allocation.

### 2. minor — `crates/core/src/manifest.rs:283-284`, `374-375`, `435-436`, `462-463`, `504-505`

For all five array-shaped entries the length cap is checked on the *cloned*
array: `string_array(…)` (`document.rs:840-857`) allocates a fresh `String` for
every element and returns it, and only then does `check_row_count` compare the
length against `MAX_MANIFEST_ROWS` (or the group cap against
`MAX_CONSTRAINT_GROUPS`). A hostile `actions = ["", "", …]` under the 1 MiB
budget materializes ~350k elements twice — the reader's `Value::Array` plus the
clone, ~8 MB of `String` headers on top of the reader's own ~8 MB — before the
budget refuses a document whose fault is knowable in O(1). The cap is therefore
a model-level bound, not the allocation bound the budget's doc comment
advertises (`manifest.rs:40-44`: "what stops a hostile manifest driving the
load-time join over an unbounded table").

Fix: check the borrowed array's length before cloning it — the length is
already available on `Value::Array(items)`. A small
`fn array_items<'a>(key: &'static str, value: &'a Value, cap: usize) -> Result<&'a [Value], CoreError>`
(or an inline `if let Value::Array(items) = value { if items.len() > cap { … } }`)
ahead of `string_array` keeps the refusal byte-identical and makes the budget
bind first. Note this is the house pattern (`chain.rs:27` caps `derives` after
`string_array` too), so the fix can stay local to `manifest.rs`.

### 3. nit — `crates/core/src/manifest.rs:171-180`, `288`, `296`, `406`, `413`, `441`, `480`, `509`

Every one-fact-per-entry rule is a linear scan over the values collected so far
(actions: two `iter().any(…)` per row; dead/docked disjointness:
`dead_keys.contains(…)` per docked row; group membership: a nested
`groups.iter().any(… inputs.contains(…))`). All are quadratic in the entry's
length, and all are bounded because the cap check precedes the loop:
a max-size `actions` array costs 2 × (255·256/2) ≈ 65k predicate calls, the
disjointness scan at most 256 × 73 (the closed vocabulary bounds `dead_keys`
before any duplicate is reachable), the group scan at most ~2.6k
`contains` calls. That is sub-millisecond, once per document load, and the
bounds are documented — so no change is required today. The exposure is the
stated growth path (Risk row: "the first real game → registry grows in
batches"): a raised `MAX_MANIFEST_ROWS` makes this the hot part of the load
silently.

Fix (optional, defensive): state the bound next to each scan
("`≤ MAX_MANIFEST_ROWS` elements, so the scan is bounded — see the budget") or
use a `BTreeSet<ActionId>` / `BTreeSet<String>` sidecar for the id and name
uniqueness checks, which is what the design's downstream posture already wants
(ADR-015's precondition checks are described as set-membership lookups).

### 4. nit — `crates/core/src/manifest.rs:344`, `562-578` (and `586-627`)

Two doc claims overstate what the budgets buy. (a) `MAX_NATIVE_NAME_OCTETS` is
documented as "what bounds a refusal that has to echo the name" (`54-57`): the
*displayed* echo is bounded by `one_line`'s 80-char cut at render
(`error.rs:132-134`, `155-169`), but the refusal String is built with the whole
value first — `format!("`{name}` is not a native name: {reason}")` allocates up
to the document budget for the oversized name it then truncates. The same
applies to the duplicate and slug refusals (`290`, `301`, `442`, `482`, `510`,
`539`, `546`). Bounded and once per parse, so not a defect — but the comment
should say the bound is on the rendered echo. (b) `check_fingerprint` decodes
the whole base64 body before checking the digest length, so a hostile 1 MiB
body drives `Vec::with_capacity(786432)` and ~786k pushes before the
"32 octets" refusal; the fingerprint is the only new shape with no length
budget.

Fix: reword the two comments to name the render-time bound, and in
`check_fingerprint` reject a body whose length is not the 44 chars a 32-octet
digest encodes to before calling `decode_base64` — or give the fingerprint its
own exported budget so it matches "named parse budgets for every new shape".

## FYI (no severity, out of specialty)

- `serialize`'s new `Document::Manifest` branch writes the five optional arrays
  unconditionally, so a document that omitted `dead-keys` round-trips as
  `dead-keys = []`. `PartialEq` still holds (empty array ≡ absent), but the
  byte form changes between an authored file and a re-serialized one; that is a
  format/round-trip question, not a performance one.
- `parse` calls `supported_majors()` on the happy path (`document.rs:258`),
  which allocates a `Vec<String>` per parse; adding `"manifests"` to
  `SUPPORTED_FORMATS` did not widen it (`document.rs:139-141` returns the two
  majors literally). Pre-existing, untouched here.
- The `Manifest` model stores `dead_keys` / `exclusive_groups` /
  `docked_actions` as `Vec`, so the ADR-015 lookups the proposal describes
  ("set-membership lookup over loaded data") will be linear per check; with the
  73-member vocabulary and the 256/64×16 caps that is fine, but a future
  per-transform-per-keystroke consumer would want a set built at load.

## What was verified, concretely

- Budget-before-scan ordering for all five array entries and for the group cap
  (`manifest.rs:284`, `375`, `394`, `436`, `463`, `505`) — holds; and the
  ordering inside `check_native_name` (`349` length check before the `360`
  control-char byte scan) — holds.
- `ActionId`/`InputId` parsing is length-checked before scanning
  (`action.rs:271-282`), and the registry join uses a map lookup
  (`action.rs:163-165`), so the two join loops are not quadratic.
- `load_manifest` reads one file under the pre-read size check
  (`toml.rs:51-60`) and does not re-read the registry — the registry is passed
  in by the caller, which is the right posture for repeated loads.
- No new public API returns a cloned collection per call; `Manifest` fields are
  owned once, and nothing in the diff clones a `Manifest` in `src`.