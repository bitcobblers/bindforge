# Code review — performance seat, ratification pass r1c5

Range: `git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/008-rust-for-the-product.md`.
Scope of this pass: confirm the revisions that landed after the round-1 approval did
not break the performance posture that seat approved. Findings below are the r1
findings re-anchored to their current line numbers, plus the deltas the later fixes
introduced.

## Checklist pass

1. **Event/keystroke path** — none exists (ADR-008: files are the product; the crate
   targets no input API). The hot-for-this-design path is document parse on untrusted
   input (hub upload, CLI file read). On that path the later fixes are net *improvements*:
   `read_file` refuses over-budget files from `fs::metadata` *before* allocating, and
   `read_reader` takes at most `MAX_DOCUMENT_BYTES + 1` from a stream, so an oversized
   upload is never held whole. Avoidable per-line/per-entry allocations remain (findings
   1–4), unchanged from r1, plus one small copy in the new `read_reader` (finding 5).
2. **Complexity** — no O(n²) over unbounded n. `resolve_chain` is O(n) with a
   `HashSet<&str>` and `Vec::with_capacity` (chain.rs:27–40); `reject_unknown_entry`
   scans a closed constant-size set; parse is one pass over lines with the 1 MiB and
   16-level budgets bounding every container.
3. **Unbounded growth** — no caches, registries, or long-lived arrays in the diff;
   every collection is bounded by `MAX_DOCUMENT_BYTES` or a closed constant.
4. **Startup** — the cargo gate (`check.yml`) runs the three design commands with no
   database, no services, `timeout-minutes` on both jobs, and the advisory scan is
   scheduled, never a PR check (Gates section) — CI hot path matches the design.
   `bindforge-core` declares no dependencies (Cargo.toml comment + empty deps), so gate
   builds stay light. Harness reads only the fixture tree it asserts on (A26).
5. **Cache invalidation** — no cache exists in the change; item is vacuous.
6. **Benchmark-like tests / timing notes** — the diff contains none to run.
   `parse_budget.rs` and the oversized-file case in `reach_tables.rs` are allocation-
   *boundary* tests (they assert refusal, not duration); the 1 MiB padding lines they
   build are comment lines, so they skip the per-char buffer before it is allocated.
   Nothing timing-shaped exists, so there is no number to quote. Note for the record:
   the gate commands themselves could not be executed in this review environment
   (shell unavailable); this pass is a static re-read, which its scope permits.

## Verdict of this pass

The post-approval revisions did not break what the performance seat approved. The
memory-cap fixes (`read_file`, `read_reader`) strengthen the budget posture. The only
performance *delta* introduced by the new code is one added small allocation in
`parse_integer` (finding 4) and one full-buffer copy in `read_reader` (finding 5) —
both bounded, nit-level, neither changing what was approved.

## Findings

1. **minor** — `crates/core/src/toml.rs:119` (r1: `toml.rs:60`) — unchanged.
   `let chars: Vec<char> = trimmed.chars().collect();` still allocates a fresh
   4-bytes-per-char buffer per parsed line on the untrusted-document path; a 1 MiB
   single-line document forces one ~4 MiB allocation on top of the document. Bounded
   by `MAX_DOCUMENT_BYTES`, so still not blocking. Fix (unchanged): scan
   `trimmed.as_bytes()` with byte-indexed positions; all structural grammar characters
   are ASCII and no UTF-8 continuation byte is ASCII, while string contents are still
   collected as chars.

2. **nit** — `crates/core/src/toml.rs:134` (r1: `toml.rs:75`) — unchanged.
   `entries.insert(key.clone(), value)` clones every key only to keep `key` available
   for the duplicate-key error on line 137; the accept path pays one allocation per
   entry. Fix (unchanged): `match entries.entry(key) { Occupied(e) => Err(… e.key() …),
   Vacant(v) => v.insert(value) }`.

3. **nit** — `crates/core/src/document.rs:108` (r1: `document.rs:91`) — unchanged.
   `!supported_majors().contains(&version)` builds a fresh `Vec<String>` on every parse
   call, including the accept path; a version-rejecting document builds it again
   (lines 103, 111). Fix (unchanged): compare `version != MAJOR_V1` directly and build
   the `supported` vector only inside the error branches.

4. **nit** — `crates/core/src/toml.rs:283-316` (r1: `toml.rs:201-224`) — slightly
   worsened by the later grammar-closure fix. `parse_integer` accumulates digits into
   a `String` (288), the new leading-zero rule adds `digits.replace('_')` (303, a
   second small allocation per integer), and the sign path still `format!("-{plain}")`
   (307-311) before `parse::<i64>()`. Every integer in a document now pays 2–3 small
   allocations where it paid 1–2 at r1. Still bounded and nit; the r1 fix stands and
   now removes more: accumulate the value arithmetically with checked overflow and
   apply the sign at the end, eliminating `digits`, `plain`, and the format.

5. **nit** — `crates/core/src/toml.rs:93-94` — new in this range. `read_reader`
   validates with `std::str::from_utf8(&bytes)` and then copies the whole buffer with
   `text.to_owned()`, so every stream upload (the hub path the design names) pays one
   full extra copy of up to 1 MiB. Fix: `String::from_utf8(bytes).map_err(|_|
   CoreError::NotUtf8)` validates in place and reclaims the buffer without the copy.

## Out-of-specialty FYI (no severity)

`parse_integer`'s `digits.starts_with('_')` check (toml.rs:300) is unreachable — the
loop only accepts `_` when `digits` is already non-empty — so the condition can never
be true. Behavior-neutral; correctness seat's call whether to mention it.
