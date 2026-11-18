# Performance review r1 (c0, full panel)

verdict: approve

Scope: whole document (`docs/designs/024-game-slug-identity/02-proposal.md`), plus
`docs/adrs/024-game-slug-identity.md` and the Revision log. Checklist:
`references/performance.md` (items 1–6). `01-research.md` was not in this pass's
allowed inputs — see Missing evidence.

## Checklist walk

1. **Input-path latency budget / per-event cost** — not applicable, with
   evidence. The repo ships one binary, `apps/discovery-cli/src/main.rs`, and no
   event/keystroke reader exists anywhere in `crates/` (only a doc mention of
   evdev in `crates/core/src/input.rs:6`). Everything this design touches is
   either load-time validation in `crates/core` (`policy.rs:55`,
   `document.rs:1238/1259/1506`, `manifest.rs:538`, `action.rs:273`) or the
   once-per-invocation discovery scan (`steam_common.rs:82-118`). The bound check
   itself stays a `len()` compare, so 63 → 128 adds no per-call cost. Nothing
   here puts allocation, a table rebuild, or synchronous IO on an event path.
   No latency budget is stated and none is required by ADR-024.
2. **Structure costs** — the per-plugin collision pass is the only new
   superlinear structure; finding 1 (minor). No bindings × devices × platforms
   nesting is introduced.
3. **Memory growth** — no cache or translation table is introduced, so there is
   nothing to bound beyond the collision pass's local maps, which die with the
   pass. The bound doubling does raise worst-case id bytes inside the capped
   arrays the README budget family describes (`MAX_MANIFEST_ROWS`,
   `MAX_PATCH_DELTA_ELEMENTS`, `MAX_POLICY_TRANSFORMS × MAX_RELOCATE_TARGETS`),
   and the design does not state why that composes safely; finding 2 (minor).
4. **Startup cost** — pass. No new file reads or schema parsing; the one new
   `discovery → bindforge-core` edge is a build/link-time cost only, and the
   fold table is a compiled `match`, not a loaded table (A5, A15).
5. **Caching / invalidation** — not applicable: the design ships no cache. The
   assigned-slug map is per-run, not held across runs, so there is no
   invalidation surface. Persistence stays ADR-022 (Deferred) — consistent with
   the ADR's "Not decided here".
6. **Research baseline** — the design quotes its own baseline honestly
   (A6: surveyed max 64 octets; A16: the "one octet above the longest surveyed
   title" rationale is *not* what the survey found) and explicitly says
   "01-research.md Risks: no baseline exists; none claimed". The gap between the
   ADR's stated rationale and the verified survey is therefore flagged by the
   design itself, not papered over; re-reporting it adds nothing. See Missing
   evidence for what this seat could not check.

## Findings

- [minor][new] Chosen design → "Emission, refusal and collisions — `crates/discovery`", read with the Risks row "Collision-suffix probe loop repeats bound checks many times" — "probe count is bounded by distinct-name count, no measurable cost today" — the stated bound is per name, not per run. As specified, every colliding name restarts its probe at `base-2` and walks the claimed set, so n distinct display names sharing one base cost ≈ n²/2 candidate constructions, each one a fresh `String` plus an `is_slug_name` and bound check. Nothing in the design caps how many rows a plugin may return — contrast the sibling budgets the same run adds its constant beside (`MAX_MANIFEST_ROWS = 256` and friends) — so the multiplier is whatever the host's `appmanifest_*.acf` count is. Minor, not major: this is a bulk CLI scan run by hand, never an event path, and realistic n (hundreds) keeps the quadratic term invisible. Fix: replace the row's mitigation clause with the honest shape — "aggregate probes are O(n²) in the distinct-name count of one plugin run, n bounded by the host's manifest-file count; no product cap applies" — or, better, store a next-free-suffix counter on each base entry of the assigned-slug map so a collision consumes one candidate instead of rescanning the claimed ones, making the pass linear; either way the pass stays one mechanism and no new constant is needed.

- [minor][new] Chosen design → "Specs, fixtures, ledger" — "`schemas/README.md` gains the bound's one stated home, a bullet in the exported budget family: `MAX_SLUG_OCTETS = 128`" — every existing entry in that family (`schemas/README.md:99-125`) carries an explicit aggregate-cost clause: ADR-013's ends "the shared 1 MiB / 16-deep budgets still bind first", ADR-014's states "adds no aggregate cap … `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` (32 MiB)", ADR-015's gives `64 × 16` predicate moves. The design specifies the bullet's rationale, supersession sentence and eleven→twelve fix, but no composition clause, even though this is the one entry that *doubles* an existing bound on values that appear inside every capped array that section governs, in a section headed as an adversarial guard. Fix: add one clause to the bullet, in ADR-013's words — the doubling adds no aggregate byte ceiling because `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` still bind first — so the family keeps its property of stating, per constant, what bounds the total.

- [nit][new] Chosen design → "Derivation — `crates/discovery/src/slug.rs`" — "ASCII case-fold (`A-Z`→`a-z`); fold precomposed Latin via a private `fold_latin_base` …; map every remaining character outside `[a-z0-9]` to `-`; collapse `-` runs; trim `-` at both ends; prefix a leading digit with `g-`" — read as steps, this is five to six full-string passes over an input whose length no field-level cap constrains: `read_text_file` caps an ACF at 1 MiB (`steam_common.rs:282`) but nothing caps the `name` value inside it, and the bound is deliberately checked only *after* the whole slug exists ("Length is **not** truncated anywhere"). Today's `normalize_slug` already allocates four intermediate `String`s per name (`crates/discovery/src/slug.rs:1-35`), so this is not a regression — but the canonical rewrite is the moment to say the shape. Fix: add one clause that `derive_slug` folds case, folds Latin, maps, and collapses runs in a single char loop into one `String` builder, with trim and the `g-` prefix done as slices/prepend rather than rewrites, so a name of any length costs one buffer.

## Missing evidence (explicit)

- This seat could not read `01-research.md`: it is outside this pass's allowed inputs. The two claims it would confirm — "no baseline exists; none claimed" (Risks row) and the 143-title survey behind A6/A16 — are therefore unverified here, not disputed. Nothing in the design's performance story depends on a numeric property that cannot be established: the design claims no numbers, and A16 already concedes the one place a number matters. Not blocking.
- The shell tool was permission-denied in this session (one attempt, `ls`), so no `cargo`/`clippy`/`fmt` leg could be run to re-confirm the green tree at the current constants. Every code claim this report relies on was read directly: `placement.rs:74`, the four private 63-bound constants (`policy.rs:37`, `manifest.rs:63`, `document.rs:220`, `action.rs:38` — four, as A2 says, against the ADR's "three"), the eleven exported constants (`lib.rs:97-117`, confirming A9), `one_line` (`error.rs:172-186`), `steam_common.rs:82-234`, `game.rs:20-27`, `slug.rs:1-35`, `apps/discovery-cli/src/main.rs`.

## FYI (no severity)

- "Where a refusal message embeds the number (e.g. `action.rs` `SEGMENT_SHAPE_RULE` '1-63 octets'), the string is built with `format!` from the constant" — no cost concern (`action.rs:123` builds it only on a refusal path, and `document.rs:1267` already interpolates the constant the same way), but the rendered text is what `schemas/action-ids/v1/spec.md` and the `fixtures/invalid/` corpora pin: message-vs-spec conformance is the compatibility and correctness seats' ground.
- 024-L1 covers "a plugin or the discovery CLI", and the design's CLI check sits in `apps/discovery-cli`. `discovery::engine::DiscoveryEngine::discover_all` is a second public exit for plugin rows — `main.rs:39-42` says the CLI deliberately does not go through it — so whether that exit is inside L1's scope belongs to the correctness seat.
- Adding `MAX_SLUG_OCTETS` to the README family places a bound over a *derived, local-filesystem* value inside a section scoped to untrusted documents; the section's opening sentence ("Documents arriving from disk, from another user, or from the network") may need a word of scope. Vocabulary, not performance.
- Choosing no headroom reserve (rejected option (b)) means two distinct names sharing a ≥126-octet base yield one accepted slug and one refused game, since candidates are never truncated. The design owns this openly and A6 makes it hypothetical; the availability shape of it is correctness's call, not mine.
