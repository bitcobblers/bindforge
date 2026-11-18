# compatibility review r3c1 (dissent pass on the post-r3 revision)

verdict: approve

Scope: `git diff b08c5589..61907ce3` on `02-proposal.md`, then the full current
document. Inputs honored as given: the proposal, the task document, the
Revision log, my r3 findings, plus read-only checks (`rust-toolchain.toml`
channel confirmed `1.85.0`; `env::home_dir` deprecation attribute per the Rust
changelogs — behavior fixed in 1.85.0, `#[deprecated]` attribute removed only
in 1.87.0, e.g. releases.rs 1.87.0 "Undeprecate `env::home_dir`").

## Status of my r3 findings

- **r3 major (home_dir deprecated on the pinned 1.85 floor vs §10's gate claim) — CLOSED.**
  §2 now reads "`home` comes from `std::env::var_os("HOME")` … `std::env::home_dir()`
  is deprecated on the pinned 1.85 floor and is not called, keeping the `-D warnings`
  gate clean (A9)". A9 is rewritten against the versioned
  `doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html` and its "(removed only in
  1.87.0)" parenthetical checks out against the 1.87.0 changelog — my r3 note
  attributed the undeprecation to 1.86.0; the attribute removal is indeed 1.87.0,
  so the register is right and my earlier version attribution was imprecise.
  §10 now states "no `std` feature flags and no `deprecated` lint to work around
  (A9)". The Revision log's q1 entry records the decision. `var_os` carries no
  lint exposure; the claim is now true as written.
- **r3 minor (empty/relative `$HOME` yielding relative candidates resolved against CWD) — CLOSED.**
  §2: "unset, empty, or relative means no home directory: empty candidates,
  `Ok(vec![])` (§7)", and Test strategy #5 adds "an empty or relative `home`
  yields the empty list". The Compatibility matrix row "Linux, Steam not
  installed / no home directory" is now consistent.
- **r3 minor (arbitrary-mounts `Yes` row: absolute-`path` branch and unmount-skip clause untested) — NOT CLOSED, re-raised below.**
- **r3 nit (non-ASCII display name never pinned) — NOT ADOPTED, re-raised below.**

## Findings

- [minor][re-raised] Compatibility matrix, row "Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes | Absolute `path` values are accepted as-is; an unmounted library is skipped, not fatal" — read against §3 ("a `path` is used as-is when absolute and otherwise joined to the directory holding the VDF") and the revised Test strategy #3 ("the root itself and `<root>/secondary` are both returned … the `"0" → ".."` entry adds no duplicate against the always-included root") — unchanged from r3 despite #3 and §3 both being rewritten in this diff. This remains the only `Yes` matrix row whose distinguishing mechanism no named test exercises (checklist item 7): the fixture reaches only the relative join branch (`"0" → ".."`, `"1" → "../secondary"`), a fabricated absolute value cannot pass the canonicalize + `steamapps` gate, and the `--steam-root` / `from_roots` seam supplies roots, bypassing the VDF branch entirely. The row's second clause ("an unmounted library is skipped, not fatal") is also still unclaimed: #7's two cases are a nonexistent root and an all-unparseable `steamapps`. Fix (as in r3): name the branch — `pub fn resolve_library_path(vdf_dir: &Path, value: &str) -> Option<PathBuf>` in §1 with `libraries_in_root` calling it — and extend #3 with two assertions on nameable values: an absolute value such as `/bindforge-abs/Media/SteamLibrary` resolves to itself, and `"1"` under the fixture VDF dir resolves inside the fixture. (An in-file `#[cfg(test)]` unit test over a private helper of the same name also closes it if adding public surface is unwelcome — but then §1's "every `pub` item … exercised by a test" invariant sentence stays untouched either way.)
- [nit][re-raised] Test strategy #8 — "returns exactly four games — `unknown/example-game`, `unknown/mixed-case-game`, `unknown/nameless-install`, `unknown/secondary-title`" (and #6's "`slug == normalize_slug(display_name)`") — unchanged by this diff, and the Revision log does not record a rejection. Every pinned display name is still ASCII, so nothing pins that a locale-named store title survives `normalize_slug`'s `char::is_alphanumeric` test rather than degrading to `unknown/`. Fix (as in r3): give one fixture manifest an accented `name` (e.g. `"Les Misérables"`), add its row to #8 and its `unknown/les-misérables` slug to #6. Slug format itself stays ADR-022's deferred concern.

## Checklist disposition

1. Matrix rows all carry a status; no `unknown`, no empty cell. ✓
2. OS input APIs — n/a: file-based discovery, no X11/Wayland/Raw Input/HID surface.
3. Node floor — n/a: no Node code touched; the `tools/orchestration` npm legs are unaffected.
4. Native modules — ✓ pure std Rust, `Cargo.lock` byte-identical posture restated in §10.
5. Keyboard reality — n/a; its non-ASCII sibling (non-ASCII display names) is the re-raised nit.
6. File paths — ✓ `PathBuf` join + `fs::canonicalize`, no string concatenation; XDG convention explicit, relative-`XDG_DATA_HOME` ignored, empty/relative `HOME` now handled.
7. Platform test per supported row — one gap remains, the arbitrary-mounts row (the re-raised minor); all other `Yes` rows are named (#5 candidates incl. Flatpak/Snap and empty/relative home, #3 root-without-VDF, #7 no-roots and all-unparseable, #10 CI job hermeticity).

## FYI (no severity)

- §4's new sentence "both caps restate the repo's shared parse budget (`schemas/reach-classes/v1/spec.md`) as plugin-local constants" cites a document outside this pass's allowed inputs, so the citation is unverified here; if that spec does not actually carry a 1 MiB / 16-level budget, the sentence should attribute the caps to this design alone (a correctness-seat check).
- §3 "`<root>` itself, always" sits slightly awkwardly against §2's canonical path rule "dropping entries … that lack the required `steamapps` directory"; Test #3's "a root with no `steamapps` directory yields an empty list" pins the intended precedence (gate wins over "always"), so the behavior is defined — the wording is for the correctness seat.
- §2's revised rule scopes library de-duping "as one set per `discover()` call" (cross-root), while §1's `libraries_in_root(root)` and tests #3/#4 operate per root; the cross-root union happens in the private scan loop, which is consistent, but the seam wording could say so.
