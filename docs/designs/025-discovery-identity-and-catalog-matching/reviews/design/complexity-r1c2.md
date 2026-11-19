# Complexity review r1 (ratification pass c2)

verdict: approve

Scope: the blob diff `2efd6ad → 4461833` of `02-proposal.md`, then the full
current document. This pass only checks that the r1 revisions did not break
what this seat approved in round 1, and did not make a cited section false.

## Disposition of this seat's r1 findings

- Finding 1 (minor, plugin-side dead-code bound + test #4) — **closed by the
  revision.** Test strategy #4 now reads: "the bound skip in `discovered_game`
  is pinned separately as an in-module `AppManifest` construction with an
  over-long appid — the only reachable route there, defence in depth." That is
  verified against the code: `AppManifest` (`crates/discovery/src/steam_common.rs:26-30`)
  has `pub` fields, so the in-module construction is the only route past
  `parse_manifest`'s 1–20-digit cap, and the text now says so out loud instead
  of hiding it behind a fixture-shaped case. The gate it defends also has a
  named future consumer (`epic` is declared this run, `025-R2`), so keeping
  the crate-side bound as defence in depth is proportionate. The Goals clause
  "is the only thing the emit boundary checks" still refers to
  `refuse_invalid_slugs` and is untouched and true. Rollout 3's parallel move
  (delete `cli_slug_refusal_tests`' "unproducible fixture route", pin the bound
  refusal in-module in `main.rs`) matches the existing in-module `past_cap`
  pins already in `main.rs` — coherent.
- Finding 2 (minor, Goals "one named refusal" vs "Two named refusals, one
  gate") — **not re-reported per the pass rule.** Neither cited passage
  changed; the mismatch stands exactly as approved in r1. Note that the r1
  revisions *expanded* the endpoint-policy machinery (`parse_endpoint`, the
  post-resolution loopback check, test #9's added cases) without adding the
  Goals bullet my fix asked for, so if the author touches that section again
  this wording should ride along.
- Finding 3 (minor, corpus sweep + `!is_slug_name(friendly_name)`) —
  **half applied, per my fix.** The friendly-name clause is gone; the bullet
  now rests the friendly-name half of `025-L5` on the Match store rule ("only
  `game_id` is ever an id or path component"), which is consistent with the
  store shape (the `friendly_name` field is file content, never a component)
  and with the JSON-row allowance. The corpus sweep survived unchanged, and
  with it my round-1 residual: the sweep's data/ half can fail on a future
  README documenting the store's own `$HOME/.local/share/bindforge` path.
  Since the sweep text and A20 are exactly what I approved, this is not a
  regression and is not re-reported.
- Finding 4 (nit, A17 premise in Options) — Options and A17 are unchanged by
  the diff; no regression.

## New-text complexity check (changed sections only)

- A10 rewrite: verified. `apps/discovery-cli/Cargo.toml:6-13` shows `clap` and
  `serde_json` per-package; workspace `Cargo.toml:20-24` carries only
  `bindforge-core`, `serde`, `winreg`. The claim "the new modules need no
  `Cargo.lock` change" holds for `matches.rs`/`catalog.rs` (std + existing
  serde/serde_json), and agrees with Rollout 7 and the Compatibility matrix.
- Rollout 1/3 suite names all exist: `plugin_tests`, `game_tests`,
  `steam_linux_tests`, `steam_windows_tests`, `emission_tests` in
  `crates/discovery/tests/`; `cli_tests`, `cli_output_polish_tests`,
  `cli_dual_plugin_tests`, `cli_slug_refusal_tests` in
  `apps/discovery-cli/tests/`.
- Step 8's "in place of today's raw `row_cells` pass-through" matches
  `apps/discovery-cli/src/main.rs:157`.
- Checklist: no new package, no new dependency, no new abstraction layer; the
  additions (`parse_endpoint`/`Endpoint`, store read budget and closed-set
  slug-shape check, exclusive tmp open with 0700/0600, response caps
  case-insensitive header match, 256-candidate cap) are hardenings of seams
  that already exist, each traced to a stated goal's mechanism or a Risks row,
  each with a pinned test case rather than a new knob. Every moving part still
  traces to a Goal or a risk mitigation. Is there a 30–50% smaller design?
  Not one that survives the panel's r1 findings — these revisions are what
  answering security/correctness costs. **Value check: the complexity remains
  proportionate to what ADR-025 ships.**

## Findings

None. Zero blockers, zero majors, zero new minors.

## FYI (no severity)

- Goals still says "a connectivity gate with one named refusal" while the
  gate names two — my r1 minor, untouched by this revision; the copy/language
  pass may want to reconcile the count while it is in there.
- Scan flow step 6 nests inline code inside inline code
  ("`(slug, game_id, friendly name of the first sharing row in `(slug,
  install_path)` order)`") — the inner backticks will render broken; a
  copyedit matter, not a design one.
