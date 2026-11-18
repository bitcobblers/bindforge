# Complexity review r3c1 (dissent pass, c1)

verdict: approve

Scope: `git diff b08c5589 61907ce3` on `02-proposal.md`, then the full current
document. Inputs: the proposal, `docs/tasks/T-001-steam-linux-discovery.md`,
the Revision log, my r3 findings. Repo claims verified with read-only checks
(`rust-toolchain.toml`, `schemas/*/spec.md` and `schemas/README.md` parse
budget text, `apps/discovery-cli/src/main.rs`, `crates/discovery/src/engine.rs`,
the pinned 1.85.0 `std::env::home_dir` doc page).

## Findings

No open blocker or major findings.

## Closures confirmed (all four r3 findings)

- [major][closed] A9 + §2 (`home_dir` vs `-D warnings`) — §2 now reads
  "`home` comes from `std::env::var_os("HOME")`" and "`std::env::home_dir()` …
  is not called, keeping the `-D warnings` gate clean (A9)`"; §10 now claims
  "no `deprecated` lint to work around (A9)"; the Revision log q1 entry records
  the r3 reason. Verified against the pinned floor: `rust-toolchain.toml`
  channel = "1.85.0", and the 1.85.0 doc page carries both the
  "Deprecated since 1.29.0" banner and the Unix clause "Returns the value of
  the 'HOME' environment variable if it set (including to an empty string)" —
  so A9's two claims are accurate as cited, and the `var_os` choice (which
  makes the empty-string case explicit rather than inheriting it) needs no new
  mechanism: `host_candidates` already takes `home` as a parameter and Test #5
  now pins "an empty or relative `home` yields the empty list". Cargo.lock
  posture intact.
- [minor][closed] §4 parse-budget cross-reference — now reads "both caps
  restate the repo's shared parse budget (`schemas/reach-classes/v1/spec.md`)
  as plugin-local constants, since `crates/discovery` takes no
  `bindforge-core` edge (A16)". Citation verified: that spec states "larger
  than 1 MiB or nested deeper than 16 levels are rejected before" parsing
  (schemas/reach-classes/v1/spec.md:46), matching the pair. The reuse decision
  is recorded without adding the edge.
- [minor][closed] §1 pub-surface coverage claim — now reads "Every `pub` item
  above except `from_host()` is exercised by a test in *Test strategy* — #5
  pins its candidate list through the pure `host_candidates` and *Rollout* #5
  checks its host probe", i.e. the exception is named instead of the claim
  papering over it.
- [nit][closed] §1 + Compatibility Windows row — the matrix row no longer
  promises reuse ("`keyvalues.rs` is written platform-neutral to be reusable"
  is gone) and the Revision log q18 entry re-bases the decision on the
  code-as-property rationale, consistent with §1's comment line. Speculative
  cross-plugin justification for surface is gone.

## Changed-text scan for new complexity cost (none found)

- §2/§3/§9/#3 root-as-library restatement (`<root>` included, fixture
  `"0" → ".."`): removes the `steamapps`-as-library entry and relies on the
  §5 composition `l/steamapps/common/<installdir>`, so it is surface-neutral
  (same public functions, one set) and the cross-root "one set per
  `discover()` call" de-dupe is one sort/dedup over an existing set, tracing to
  the deterministic-order Goal and the Flatpak/Snap aliasing risk (A8, A10).
- §7 `Err` condition ("zero games emitted, and at least one file skipped")
  costs two counters at one call site; strictly narrower than the replaced
  wording and still one error path.
- No new package, module, dependency, or knob appears in the diff; the net
  change removes one std call and one reuse promise. Complexity remains
  proportionate to T-001's scope.

## FYI (no severity)

- §8 — "its `DiscoveryPlugin` import goes with it rather than being
  re-imported" is true (the `Box::new(SteamLinuxPlugin::…)` →
  `Box<dyn DiscoveryPlugin>` coercion in `register_plugin` needs no trait in
  scope; apps/discovery-cli/src/main.rs:16), but by the same
  unchanged-printing-code logic the `discovery::game::DiscoveredGame` and
  `discovery::registry::DiscoveryError` imports (main.rs:3,5 — today used only
  inside `DummyPlugin`'s impl) also become unused and die with it under
  `-D warnings`; correctness seat may want the import accounting stated whole.
