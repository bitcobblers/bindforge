# compatibility review r1c3 (ratification of r1 revisions)

verdict: approve

Scope: the r1 diff (`ac9cb8a..2aae9c9` of `02-proposal.md`) plus the full
current document, read against `docs/adrs/014-profiles-as-chains.md` and the
Revision log. Checklist applied: `references/compatibility.md`.

Ratification result: none of the r1 revisions breaks the compatibility
surface approved in round 1. Verified against the shipped tree:
`MAX_MANIFEST_ROWS = 256` is a real exported constant
(`crates/core/src/manifest.rs:45`, re-exported at `lib.rs:71`), so the newly
named mirror in Options and the composite-budget sentence in Resolution are
accurate. Dropping `chain-past-max-depth.toml` from the v2 corpus breaks no
shipped test: `chain_guard.rs::the_invalid_corpus_ships_both_chain_boundary_cases`
is pinned to `version_dir("profile", "v1")` only, and v1 keeps both boundary
fixtures (A15). The depth unit change ("documents opened") keeps the shipped
boundary semantics — exactly `MAX_CHAIN_DEPTH` accepted, one more refused —
which is also what `chain_guard.rs` pins for v1, so v1 and v2 agree at the
boundary. The `ten` → `eleven` entry-count fix is correct: the entry table and
the `serialize` order both hold eleven entries, matching Goals (4 + 7).

## Findings

- [minor][new] Chosen design → The contract: `schemas/profile/v2/`, `generated-by` row — "1–128 octets, no ASCII control, default-ignorable, or bidi-control characters" — the revision adds two exclusion classes named as Unicode properties that the std-only story in the Compatibility matrix ("Toolchain floor (1.85.0, edition 2024) | yes | std-only, no new dependency, `Cargo.lock` byte-unchanged") cannot answer: `char::is_control()` covers Cc only, and Rust std exposes no `Default_Ignorable` or bidi-category accessor (shipped code checks exactly that — `document.rs:875`, `error.rs:164`). Two consequences: the reference implementation needs a hand-maintained range table this document never states, and `profile/v2/spec.md` is named as the language-neutral contract (matrix row "Third-party reader, non-Rust"), so a third-party reader cannot decide whether `generated-by = "tool 🛠️"` (U+FE0F is Default_Ignorable) or a soft-hyphenated label is a valid shipped document — a validity disagreement over a shared, public artifact, which is the surface this design pins. Only bidi and ASCII-control are pinned by fixtures (`generated-by-bidi-char.toml`, `generated-by-control-char.toml`); "default-ignorable" has none. Fix: in the entry row (and `spec.md`), replace the property names with the closed code-point list — bidi-control `U+061C`, `U+200E`–`U+200F`, `U+202A`–`U+202E`, `U+2066`–`U+2069` — and either drop "default-ignorable" or pin it to an explicit range list; either way add the matching invalid fixture so the third boundary is corpus-pinned like the other two.

- [nit][new] Chosen design → Cross-file loading: `load_chain` in `chain_ops.rs`, step 2 — "the constructed path is always a direct child of the head, `<major dir>/<parse-validated slug>.toml`" — the link path is stated as a `/`-joined template, and the checklist asks that separators be handled by a library rather than string formatting; the Windows contributor row is `yes`, where `\` is the separator. The traversal safety is already closed (slug grammar refuses `..` and `/`, named in Test strategy #1), so only the separator wording is missing. Fix: one clause naming the std construction — "the link path is `Path::join`ed from the head's parent directory" — and drop the `/`-template spelling.

- [nit][new] Chosen design → Chain operations, `fork` — "empty delta lists — a shell `load_chain` refuses as `patch-empty-deltas` until the caller fills one element" — `patch-empty-deltas` is a fixture name, not a refusal; every other promise to a caller here names the typed variant (`InvalidValue`, `ChainLinkMissing`, `MissingEntry`), and the refusal a caller must match is part of the public error surface. Fix: name the variant (`InvalidValue` naming the link/element, the shape Per-link rules already implies) and cite `patch-empty-deltas.toml` as the fixture that pins it.

- [nit][new] Chosen design → Resolution, "Mount at any rung" — "`pub struct Chain { links: Vec<Profile> }  // only load_chain builds one`" plus "a consumer mounts a prefix by `load_chain`ing a lower link as head, or reads the documents singly with `parse_file`" — the revision made the only read path into `Chain` private and added no accessor, so a consumer holding a `Chain` cannot see its links, and the promised mount-at-any-rung (ADR Decision Three: "take its base and patch independently") is reachable only by re-parsing files and walking `Profile.derives` by hand. Fix: one clause stating the intent — either `Chain::links()` as an iteration accessor, or "`Chain` is opaque by design; the rung ids are read from the head document's `derives` with `parse_file`" — so `zz_api_probe.rs`'s "new public names" is decidable before implementation.

## FYI (no severity)

- Carried from r1, unchanged by this diff, so not re-reported as findings per the ratification rule: (a) the `PROFILE_ENTRIES` → `PROFILE_V1_ENTRIES` rename is still not stated as internal — it *is* internal, `crates/core/src/document.rs:69` declares it with no `pub`, so one clause closes it; (b) the sentence inheriting `keyboard:` physical-key semantics from the shipped input/reach-classes contract is still absent from the `spec.md` load-rule list pinned in Rollout 1; (c) the matrix cell "macOS contributor | no (v1)" is still cryptic. None of these sections appears in the diff.
- The Revision log's r1 entries record what landed but not the disposition of the r1 review findings that did not land (the three above). The editor may want one line per deliberately-held position; that is the run's process, not a compatibility defect.
- `Chain` field privacy and the `diff` "same document — equal parsed `Profile`s" rule look like API-shape and semantics questions for the correctness seat; I read them only for their public-surface and cross-implementation consequences, noted above.
