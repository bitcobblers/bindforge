# compatibility review r1

verdict: approve

Scope: whole document (full panel, c0). Inputs: `02-proposal.md`,
`docs/adrs/025-discovery-identity-and-catalog-matching.md`, the Revision log,
plus read-only inspection of the tree to check the design's claims about the
code. No prior findings from this seat.

## Checklist walk

1. **Matrix rows carry a status** — satisfied. All 12 rows carry a status
   (`Yes`, `Compile-verified, with a named gap`, `Yes, no CI leg`,
   `Declared, unpopulated`, `No (named refusal)`, `Not affected`); no `unknown`,
   no empty cell. Two rows' *basis* is the finding, not their status: see 1 and 3.
2. **OS input APIs (X11 / Wayland / Raw Input / HID)** — not applicable. The
   design touches no input capture: it reads `appmanifest_*.acf` text and the
   Windows registry (unchanged `winreg` seam) and opens a loopback `TcpStream`.
3. **Node version floor** — not applicable. No Node code changes;
   `tools/orchestration`'s check (`npm run lint && format:check && test`,
   `engines.node >= 22.23.5`) reads only pipeline sources and tmp-dir fixtures,
   and nothing there reads discovery output. The matrix row's claim holds.
4. **Native modules / silent native dependencies** — satisfied, and the
   strongest part of the design. The transport choice (Options, "Transport for
   the candidate query", (c)) keeps the dependency set closed; verified that
   `apps/discovery-cli/Cargo.toml` already carries `serde` (derive) and
   `serde_json`, and `crates/discovery` already carries `bindforge-core`, so
   Rollout 7's "No `Cargo.lock` change is expected" is reachable: the new
   modules need no new dependency edge. The rejected `ureq` branch, with A17's
   C-toolchain reading of the `x86_64-pc-windows-msvc` compile leg, is correctly
   identified as the compatibility risk and avoided.
5. **Keyboard reality (non-QWERTY, dead keys, IME, non-ASCII names)** —
   satisfied. `friendly_name` is free text clamped at a `char` boundary with a
   multi-byte case in Test #2; the only identifier that becomes a path
   component (`game_id`) is held to `is_slug_name`, which is ASCII-only, and a
   non-conforming candidate is dropped with a named line rather than carried.
6. **File paths / per-OS config-dir conventions** — partly satisfied. The
   env-lookup seam mirrors a real precedent (`steam_common::absolute_env`,
   crates/discovery/src/steam_common.rs:424) and keeps the rule unit-testable
   off-target. The macOS convention is unnamed: finding 1.
7. **A platform test per supported platform row** — Linux ✓ (Test #15, and
   check.yml:17-38 confirms the full suite runs only on `ubuntu-latest`, with
   Windows as a compile-check leg and no macOS leg — A11 verified). Windows
   names its substitute (the `%APPDATA%` branch through the env seam). macOS
   names none: finding 2.

Claims checked and found clean, recorded so the reviser need not re-open them:

- A3 — the shipped KeyValues reader is case-insensitive
  (crates/discovery/src/keyvalues.rs:51, `eq_ignore_ascii_case`), so the
  Steam row's `read case-insensitively (A3)` holds; no exact-case read is needed.
- A13 — the shipped amendment shape exists verbatim in the corpus
  (docs/adrs/013-game-manifests.md:117, `amended-by 024-B1 (ADR-024, per 021-L4).`),
  and `ledger_annotations.rs::ledger_unit` reads a unit entry by `**UNIT**`
  marker and continuation lines, so appending the annotation is gate-stable.
- A2/A4 — `parse_manifest` reads `installdir` only
  (crates/discovery/src/steam_common.rs:333-346), `discovered_game` derives from
  name/install_dir (238-257), `resolve_collisions` is the collision pass (278),
  and both platform plugins wrap the shared core.
- The store's tmp-file + `std::fs::rename` overwrite is **not** a Windows
  defect: `std::fs::rename` is documented as "replacing the original file if
  `to` already exists", and on Windows maps to `MoveFileExW` with the
  replace-existing flag (`FileRenameInfoEx` on Windows 10 1607+). No finding.
- `escape_and_truncate` is exported (crates/core/src/error.rs:190) and
  `MAX_NATIVE_NAME_OCTETS = 128` / `MAX_SLUG_OCTETS = 128` exist as the design
  states; the exported-constant family counts twelve today, so
  `schemas/README.md:143` ("exports all twelve constants") → thirteen is right.
- Deferred items were read against ADR-025's Decision and contracts; each names
  the clause it hands off (`025-R1`, `025-R2`, ADR-022 persistence, ADR-017).
  None contradicts the source document, so none is re-opened here.

## Findings

- [minor][new] Chosen design → "Match store — `apps/discovery-cli/src/matches.rs` (new)" + Compatibility matrix "macOS (APFS)" row — "`<data>` is `$XDG_DATA_HOME` when set and absolute (else `$HOME/.local/share`) on Linux/other, and `%APPDATA%` on Windows" and "`Yes, no CI leg` | `The XDG fallback path is the same rule; nothing in this design is macOS-specific`" — the design names exactly two data-dir conventions, and A16 states its own scope as "a Linux/Windows product", citing the freedesktop spec and Windows known folders. The macOS per-user data directory (`~/Library/Application Support`) is named nowhere, so on the platform the matrix supports as `Yes` the store lands at `~/.local/share/bindforge/matches.json` with neither a stated basis nor a recorded deviation — the checklist's per-OS config-dir convention is satisfied for two of the three named platforms (item 6). Fix: one clause that makes the macOS case a decision rather than a leftover — either the rule names it (`$XDG_DATA_HOME` if set and absolute, else `$HOME/Library/Application Support/bindforge/matches.json` on macOS, else `$HOME/.local/share`), or the row and the Match-store bullet state that macOS deliberately rides the Linux XDG fallback and a macOS-native location is Deferred — and A16's basis extended or its macOS half marked unverified to match.

- [minor][new] Test strategy #15 vs Compatibility matrix "macOS (APFS)" row — "`macOS has no leg — so no test asserts a platform-specific outcome; the Windows store-path rule is exercised on Linux through the env seam`" — the matrix claims macOS as supported (`Yes`) while the test strategy names no macOS-targeted verification at all, where the Windows row does name its substitute. Checklist item 7 wants a platform test named for every platform the matrix claims as supported, and the row's own answer ("runtime verification level is Test strategy #15's") is a statement that the level is *none*. Fix: name the pin the Windows row has — extend Test #5 with the non-Windows branch as the path macOS runs (`XDG_DATA_HOME` unset → `$HOME/.local/share/bindforge/matches.json`, asserted through the same env closure) and cite it from #15 — or change the cell to say what it is: expectation by code-path sharing with Linux, not verified support.

- [minor][new] Compatibility matrix (row set) — "`Third-party schema readers` | `Not affected`" and "`tools/orchestration` (Node) | `Not affected`" — the matrix enumerates the surfaces that survive and is silent about the two it breaks by construction: the `discovery` crate's public API (Options "Where the `game_id` lives" reasons about "the type a third-party plugin constructs"; `DiscoveredGame.namespace` and the `Namespace` enum are deleted, `friendly_name` added, `crates/discovery/src/slug.rs` and `resolve_collisions` deleted, `Source`/`lookup_key` added), and the CLI's published stdout contract that A7 cites T-003 for (fixed JSON key set with `namespace`; the `Namespace` column). Test #13/#14 re-pin both in-tree, but a reader of the matrix alone sees only `Not affected` rows. Fix: one row — e.g. "`discovery` public API and CLI stdout contract | Breaking, in-tree only" — stating the deletions, the re-pinned key set and columns, and the basis that bounds the blast radius (`crates/discovery` is an unpublished workspace member, and `discovery-cli` is the only consumer in the tree).

- [nit][new] Chosen design → "Bound — `crates/core` and `schemas/README.md`" — "`schemas/README.md` gains a sibling bullet in the exported budget family" — `crates/core/tests/ledger_annotations.rs::bound_bullet` picks the **first** line containing the literal `MAX_SLUG_OCTETS = 128` and asserts the seven-rider set on that whole bullet, and the family's house style for a sibling is to quote exactly that string (the existing bullet: "The value is the sibling of `MAX_NATIVE_NAME_OCTETS = 128`"). A new ADR-025 bullet written the same way, or ordered above the ADR-024 bullet, would make design 024's Test #13(c) pin fail on the new text; the design's own proposed wording ("set equal to the two 128 bounds") happens to dodge it, and neither the bullet description nor Test #16 pins that. Fix: state in the bullet's spec that it lands *after* the ADR-024 bullet and states its own `MAX_DISCOVERY_SLUG_OCTETS = 128` without the literal `MAX_SLUG_OCTETS = 128`, or name re-anchoring `bound_bullet` to the ADR-024 bullet in Test #16.

## FYI (no severity)

- Emit-boundary relaxation: `025-L3`'s "The product checks only `025-B1` at the emit boundary, not the grammar" means a third-party `DiscoveryPlugin` row now survives with control characters or newlines in its slug, and *surviving* cells print raw into the table and JSON — the `escape_and_truncate` posture the design states covers echoed values (refusals, candidate names, rejected ids), not rendered ones. Security seat's.
- The endpoint policy admits the host name `localhost`, so "loopback authority" is partly a resolver/`/etc/hosts` guarantee rather than an address guarantee. Security seat's.
- `025-L3`'s `(slug, install_path)` byte sort is lexicographic over `steam/<appid>`, so `steam/900001` precedes `steam/1000000`; the claim "makes the row order the byte order of `steam/<appid>`" is literally true, but a reader may expect numeric order. Correctness seat's, if it is a claim about output order at all.
- The matrix row "Flatpak / Snap Steam installs | Yes" reasons from `host_candidates` (Steam's sandbox, unchanged) and then from `XDG_DATA_HOME` (the CLI's own data dir) in one breath; the two are independent, and A16's sandbox half is marked unverified. Harmless today — no packaged CLI exists — but the row conflates two mechanisms.
