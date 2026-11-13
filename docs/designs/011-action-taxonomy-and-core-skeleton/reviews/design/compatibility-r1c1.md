# compatibility review r1 (dissent pass c1)

verdict: approve

Scope: blob diff `118d4d2..f23e3e4` of `02-proposal.md`, then the full current document. Inputs: the proposal, its Revision log, ADR-011, and my round-1 findings.

## Previous findings — closure check

1. **[major, Steam Deck / SteamOS row] — CLOSED.** The row now reads `no (v1) | Design 008's matrix; nothing here adds platform-specific work`, matching the macOS row's inheritance pattern. The matrix has no `unknown` cell and no empty cell; checklist item 1 is satisfied.
2. **[minor, non-Rust third-party reader row] — CLOSED.** The status is now `no (v1)` and the note states the deferral ("no non-Rust leg exists until the Deferred runner contract does") instead of claiming an unexercised `yes (fixtures)` leg. No test leg was added, which is the right call while the runner contract stays Deferred. The deferral does not contradict ADR-011: the ADR's Decision keeps `schemas/` the language-neutral contract with golden fixtures run by CI — that remains true; only an *external runner contract* is deferred, and the row no longer overstates support.
3. **[minor, Stability grammar-widening posture] — CLOSED and the replacement is sound.** Stability now says widening "rides that same directory: a third segment in `action-ids/v1/registry.toml` makes the file invalid for every v1 loader (Test strategy #6), so it goes to `action-ids/v2`, where the version gate is what refuses it for v1 readers." This matches `third-segment.toml` under fixtures/invalid, Test strategy #6's grammar-validity requirement, and the new version gate (entry table's `version`-equals-directory-segment rule + `load_action_registry`'s check + Test strategy #4's directory-mismatch case + `InvalidValue` reusing the existing error contract). The Risks row's mitigation was updated in lockstep — no orphaned "additive" claim survives anywhere in the document.
4. **[minor, non-ASCII fixture gap] — CLOSED.** `non-ascii-id.toml` is in the fixtures/invalid list and Test strategy #1 gains "a non-ASCII letter in a segment". The ASCII-only grammar ("ASCII lowercase letter first, then `[a-z0-9-]`") is now pinned for a third-party reader building from the corpora; checklist item 5's binding-name leg is satisfied for this contract (dead keys/IME live in ADR-012's input vocabulary, properly Deferred).

## New findings from the changed text

None. I checked each closure for collateral damage against the compatibility checklist:

- Matrix item 1: every row carries a status; "not applicable" (product artifacts) is justified in-row ("This stage ships no binary") and matches the ADR's deferral of codecs/CLI.
- Item 3 (version floor): the toolchain-floor row stands and the added checks (version-segment comparison, printable escaping) are std-only; no new dependency, `Cargo.lock` byte-unchanged.
- Item 4 (native modules): unchanged — pure std Rust.
- Item 6 (paths): `load_action_registry` still uses `dir.join("registry.toml")`, and config-dir conventions remain deferred to ADR-017 without contradicting ADR-011's "Not decided here" list.
- Item 7 (platform legs): the two `yes` rows (Linux CI, Windows contributor) both have named legs in Test strategy #7 and Rollout 5.
- The new `InvalidValue` sub-cases (version/directory mismatch) add no error-contract surface, preserving the single-line, bounded, path-free message rule the contract promises; the version-consistency chain (spec entry table ↔ loader ↔ Test #4 ↔ conformance walk declaring `version = "v1"`) is internally consistent, including for fixture files the harness walks.

## FYI (no severity)

- The model's "an offending id echoed escaped to printable characters" is error-message hygiene aimed at the non-ASCII id rejection — arguably a security-seat observation; from the compatibility side it is a net positive for third-party readers diffing error output.
- Editorial: Options considered (Withdrawal) now reads "the major-directory route whose rule Stability states" — awkward phrasing, and the Stability rule it names covers withdrawal (and now grammar widening) rather than a single "route". No contract content changes; wording only.
- The Revision log's r1 entry says a document from elsewhere "is parsed as data", while spec.md says it is "never loaded as the registry" — the spec is the narrower, correct statement; the log is only a log.
