# compatibility review r1

verdict: approve

Scope: whole document, full-panel pass c0, round 1. Inputs: 02-proposal.md,
docs/adrs/013-game-manifests.md, the Revision log (r0: no prior findings).
No prior compatibility findings to re-check.

## Findings

- [minor][new] Chosen design → "Row encoding" (with the `schemas/manifests/v1/`
  fixture list and Test strategy #1) — "a native name of 1–128 octets with no
  ASCII control character" — the contract never pins whether *raw non-ASCII*
  native names (accented Latin, CJK) are legal, even though A13's own basis is
  localized display text ("Steam IGA 'Move'/'Camera' with localized text").
  The octet-based cap and the ASCII-only control-character exclusion imply
  tolerance, but the corpus — which per A3 "is what makes the format
  supported" — contains no non-ASCII name fixture, and Test strategy #1
  exercises only spaces, `#`, `"`, and `\`. Two readers can therefore diverge
  on raw-UTF-8 passthrough vs escape-only, and pinning the answer after v1
  freezes would ride a `manifests/v2`. Fix: extend the Row encoding rule with
  one clause (names are any UTF-8 text except the refused characters), add
  `fixtures/valid/localized-native-name.toml`, and extend Test strategy #1 to
  parse-and-round-trip it; if non-ASCII control/format characters are also
  refused, pin that with an invalid fixture. Smallest fix is spec sentence +
  one fixture + one test clause — no new mechanism.

## Checklist coverage (compatibility)

1. Matrix: every row carries a status (yes / no (v1) / not applicable); no
   `unknown`, no empty cell. Pass.
2. OS input APIs: the stage ships no input capture (std-only parse path; the
   matrix's "Product artifacts … not applicable — this stage ships no
   binary"), so no per-platform input API is targeted; the abstraction is
   justified by the Toolchain-floor row's "std-only, no new dependency". Pass.
3. Toolchain floor (1.85.0, edition 2024): std-only, `Cargo.lock`
   byte-unchanged, no API named that sits above the floor. Pass.
4. Native modules: none; explicit "no new dependency, no `Cargo.lock` change".
   Pass.
5. Keyboard reality: dead keys are a first-class entry (`dead-keys`); IME and
   layout naming ride the already-closed `InputId` vocabulary (012's, widened
   only by a stated vocabulary major — deferral consistent with the ADR).
   Non-ASCII binding names: partially handled — see the minor finding.
6. File paths: `game_dir.join("manifest.toml")` (library join, not string
   concatenation); the id==directory byte-equality rule plus lowercase-only
   grammar is an explicit case-insensitive-filesystem posture, pinned by Test
   strategy #4. Per-OS config-dir conventions (XDG/AppData) do not apply —
   manifests are repo data under `data/manifests/`. Pass.
7. Test strategy names a platform leg for each supported row: Linux CI (Test
   #8, the merge PR's job), Windows contributor (Test #8's once-run leg),
   toolchain floor (same gate commands, pinned toolchain per A10). macOS and
   Steam Deck are claimed *not* supported, so need no leg. Pass.

## FYI (no severity)

- The matrix never names X11/Wayland/Raw Input/HID anywhere; the std-only
  data-layer posture covers it, but an explicit "no OS input interaction in
  this stage" half-sentence in the matrix notes would make the abstraction
  argument self-contained.
- The closed `keyboard:` vocabulary's char-based member naming (`keyboard:s`
  in the example) is a real non-QWERTY concern, but it is 012's vocabulary,
  untouched by this diff — flagging only for the record.
- The location rule compares the caller-passed path's final segment to `id`
  byte-for-byte; on a case-insensitive filesystem the *on-disk* case only
  surfaces when the tree is enumerated (e.g. `read_dir`), not when a caller
  passes a lowercase path. The stated posture holds under CI enumeration; how
  a future loader (017/018) discovers the directory may deserve the
  correctness seat's eye.
