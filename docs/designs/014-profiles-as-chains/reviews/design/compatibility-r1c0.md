# compatibility review r1

verdict: approve

## Findings

- [minor][new] Chosen design → Delta inventory (Edited) — "`PROFILE_ENTRIES` renamed `PROFILE_V1_ENTRIES`, new `PROFILE_V2_ENTRIES`" — the rename is presented as an ordinary edit, but the proposal never states whether `PROFILE_ENTRIES` is private or part of `core`'s public surface; the `zz_api_probe.rs` edit is listed for "the new public names" only, so nothing in the document says whether existing downstream users of the old name break, and the Compatibility matrix has no core-API row. Fix: state in the Edited list (or a matrix row) that the rename is internal because `PROFILE_ENTRIES` is private — as the register already does for `placement::is_slug_name` (`pub(crate)`) — or, if it is public, record the rename as a public-surface break in the matrix; do not add a compat alias.
- [minor][new] The contract: `schemas/profile/v2/` — "The `spec.md` also states the load-level rules below (the directory rule, pin equality, the joins)" — the keyboard-reality check is only half covered: non-ASCII binding names are refused (slug grammar, uppercase fixtures) and named keys are Deferred consistently with the ADR, but chain documents bake input ids like `keyboard:d` into artifacts shared across players (the example carries `set = ["move.steer-left keyboard:d"]`) and nowhere — contract, matrix, or Deferred — is it stated that the physical-key semantics of the closed `keyboard:` vocabulary (character vs position on non-QWERTY layouts) are inherited from the shipped input/reach-classes contract unchanged and are not this design's to define. Fix: one sentence appended to the spec.md load-rule list pinned in Rollout 1 — input-id semantics are inherited unchanged from the shipped contract and a chain is layout-neutral data — replacing nothing; it clarifies the boundary the determinism rules already assume.
- [nit][new] Compatibility matrix — "macOS contributor | no (v1)" — the status cell "no (v1)" is cryptic: a first-time reader cannot tell which posture "(v1)" names or whether a 014-era contribution on macOS is in or out. Fix: spell the cell out, e.g. "no — outside design 008's supported matrix", and drop the bare "(v1)" marker.

## FYI (no severity)

- Chain operations → `merge` — the conflict example "one side clears an action the other sets, an order the fixed list order cannot express" is order-dependent in a way the sentence hides: with the fixed `set` → `clear` order, p1-set + p2-clear *is* expressible (as `set-then-clear-same-action.toml` already pins); only the reverse composition is in question. The exactness criterion belongs to the correctness seat; wording should say which order conflicts.
- Options ("Where cross-file resolution lives") reinterprets ADR Decision Two's "Resolve the chain with `resolve_chain`" as `resolve_chain` untouched + a new `load_chain` applying the same two refusal rules; the ADR's intent (same guards, deterministic walk) is preserved, but an ADR-fidelity reader may want the ADR's naming acknowledged once. Not a platform-compatibility issue.

## Checklist coverage

1. Matrix: every row has a status; no `unknown`, no empty cells. Pass (see nit for one opaque cell).
2. OS input APIs: justified abstraction — "Product artifacts … not applicable | This stage ships no binary"; core is pure data. Pass.
3. Version floor: floor named (1.85.0, edition 2024), std-only, no new API above it claimed. Pass.
4. Native modules: none — std-only, `Cargo.lock` byte-unchanged. Pass.
5. Keyboard reality: non-ASCII ids refused (slug grammar + fixtures); named keys deferred consistently with the ADR; dead keys/IME not touched at this layer; non-QWERTY boundary unstated → minor finding.
6. File paths: directory rule + `check_dir_version` + id↔file-name equality + lowercase-ASCII slugs cover case-insensitive filesystems; Rust `&Path` interface; per-OS config dirs (XDG/AppData) deferred to ADR-017 consistently with the ADR's "not decided". Pass.
7. Test strategy platform legs: Linux gate commands and the Windows contributor leg both named for the two platforms the matrix supports. Pass.
