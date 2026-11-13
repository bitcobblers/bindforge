# compatibility review r1c0

verdict: request-changes

Inputs read: `02-proposal.md` (whole document), `docs/adrs/011-action-taxonomy-and-core-skeleton.md`, the Revision log (r0: no prior decisions). No prior compatibility findings to re-check.

## Findings

- [major][new] Compatibility matrix — "| Steam Deck / SteamOS | unknown | Inherits design 008's position; this stage adds no platform-specific work |" — the checklist requires every matrix row to carry a status and treats `unknown` as a finding: the row points at design 008's position without recording what that position is, so the matrix itself leaves the platform undecided. The macOS row right above shows the pattern that closes it: it inherits 008 by *restating the status* ("no (v1) | Design 008's matrix"). Fix: give the Steam Deck row an explicit status — e.g. `no (v1)` with "inherits design 008's matrix", or `yes` if 008's CI actually covers SteamOS — so the row states a decision, not a pointer.

- [minor][new] Compatibility matrix × Test strategy — "| Third-party reader, non-Rust | yes (fixtures) / no (harness) |" — the matrix claims a support leg ("yes (fixtures)": the corpora are consumable by non-Rust readers) that no Test strategy entry exercises; the external runner that would prove it is Deferred with "no follow-up yet". The checklist wants a named leg for every support claim. Fix: restate the status as the deferral rather than a claim — `no (v1) — corpora language-neutral, runner contract deferred` — so no claimed-but-untested leg remains (a non-Rust leg cannot exist until the deferred runner contract does; adding one now would invent the deferred mechanism).

- [minor][new] Stability — "Widening the *grammar* later (a third segment, say) is an additive change: a v1 reader refuses the id outright rather than misreading it, which is the same posture as an unregistered id." — the posture claim is wrong for this contract as specified: an id violating the v1 grammar makes the whole registry document invalid (`third-segment.toml` is listed under `fixtures/invalid/`; Test strategy #6 requires every id in the loaded registry to be grammar-valid), while an unregistered id leaves the document loading with `axis_kind` → `None`. So widening the grammar inside `action-ids/v1/` breaks every v1 loader of that file — a breaking change, not an additive one. The version mechanism already handles this correctly; only the sentence is wrong. Fix: replace the sentence — grammar widening for ids actually in the registry rides a major directory (`action-ids/v2`), where the existing version gate (Test strategy #4) refuses cleanly with the supported majors named; refuse-don't-misread is the posture only for consumers validating *other* formats' ids (ADR-013), not for loading a widened v1 registry.

- [minor][new] The contract: `schemas/action-ids/v1/` (fixtures/invalid list) × Test strategy #1 — the id grammar is byte-exact ASCII ("each segment is ASCII lowercase letter first"; "Comparison is byte-exact: no case folding, no canonicalization"), and the checklist's keyboard-reality item names non-ASCII binding names, yet the enumerated invalid fixtures cover only uppercase, digit, hyphen, underscore, segment-shape and arity faults — no non-ASCII id — and Test strategy #1's reject list likewise omits one. A third-party reader building from the golden corpora gets no fixture pinning the non-ASCII case. Fix: add `non-ascii-id.toml` (a segment containing a non-ASCII letter, e.g. `aim.cibl` with a diacritic) to `fixtures/invalid/` and to Test strategy #1's reject list.

## Checklist coverage (no finding)

- OS input APIs: abstraction justified as written — this stage ships no OS-facing code ("std-only, no new dependency", "This stage ships no binary"), and input vocabulary is Deferred to ADR-012; no X11/Wayland/Raw Input/HID target is owed here.
- Toolchain floor: named (1.85.0, edition 2024); the design claims std-only, no new dependency, `Cargo.lock` byte-unchanged, mirroring existing `reach.rs` shapes — no API above the floor is named.
- Native modules: none; no silent native dependency; the packaging question (installed artifact locating `schemas/` data) is Deferred to ADR-017, consistent with the ADR's "Not decided here: codecs and the CLI (ADR-017)".
- File paths: `parse_file(dir.join("registry.toml"))` uses `Path::join`, not concatenation; refusals name no local path (Test strategy #6); Windows case-collision is addressed in the matrix row and Test strategy #7's Windows leg; config-dir conventions belong to ADR-017's config-path discovery, deferred consistently.
- Deferred section: every deferral (emphasis → 013, polarity → 015, vocabulary/reach → 012, chains → 014, codecs/CLI → 017) matches ADR-011's "Not decided here" and Decision clauses; none contradicts the ADR.

## FYI (no severity)

- A3 / Options (last): reading ADR-011's "no behaviour beyond this ADR" as governing additions rather than the shipped ADR-008 spine is a correctness/intent-fit call, not a compatibility one.
- The X11 vs Wayland vs Raw Input vs HID targeting this seat checks at matrix time will land in ADR-012's run; worth carrying this checklist item there.
