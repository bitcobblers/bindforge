# Complexity review r2

verdict: approve

## Scope and inputs

Editorial confirmation pass of round 2. Inputs opened, exactly as permitted by
the caller: `02-proposal.md` (post-editorial text), `docs/tasks/T-002-steam-windows-discovery.md`,
and the proposal's Revision log, plus read-only inspection of the current code the
proposal's reuse claims rest on (`crates/discovery/src/lib.rs`,
`crates/discovery/src/steam_linux.rs`, `crates/discovery/Cargo.toml`,
`apps/discovery-cli/src/main.rs`, `crates/discovery/tests/steam_linux_tests.rs`,
grep of `Cargo.lock`).

Missing evidence, recorded explicitly: `reviews/design/editorial-r2.diff` could
not be opened — every access path to it (`read`, and `git diff*` shell forms)
is fenced off `reviews/**` for this seat, and no earlier complexity report exists
(this seat had no findings in the editorial phase). I therefore confirmed against
the proposal's current text and the source task, not line-by-line against the
editorial diff. This does not block: nothing in the current text fails the
complexity checklist, so no residual could hide in the diff for this specialty.

## Checklist walk-through

1. **Reuse before new code** — verified, not assumed: every item §1 moves to
   `steam_common` (`library_paths`, `libraries_in_root_with_skips`, `scan_library`,
   `read_text_file`, `canonical_paths`, `discovered_game`, `absolute_env`,
   `parse_manifest`/`usable_install_dir`) exists in `steam_linux.rs` and the move
   list matches the file's actual structure; the "existing seven modules" claim
   matches `lib.rs:1-7`; `keyvalues`/`normalize_slug` are reused as-is; the CLI
   selection point is exactly `main.rs:33-38`. `tools/` (Node orchestration) has
   nothing reusable here.
2. **Every moving part traces to a Goal** — `SteamCore<T>`/`SteamTarget` traces
   to the task's hard reuse requirement ("refactor into a shared module or a
   platform-parameterized core", task :32-33); `RegistryKey` + `&dyn Fn` seam to
   the injectable-seam clause (task :45-51); the cross-`cargo check` step to the
   Done-when :69-71. Nothing else was added: no new flags, no config, no schema
   change, no `apps/` member.
3. **Smaller design** — the Options section already rejected the smaller shapes
   (forked file, `bool` flag) for reasons anchored in task clauses; an enum-in-
   place-of-trait variant is the same size. The trait carries exactly the three
   per-platform facts that §2 lists as the only parameterized points.
4. **Dependencies** — one new crate, target-gated, two calls through one private
   function. Verified the reuse claim: `Cargo.lock` already contains
   `windows-sys 0.61.2` and `windows-link`; `cfg-if` is absent, matching A10's
   predicted lock delta, and Test strategy #11 pins the lock diff to exactly
   `winreg` + `cfg-if`.
5. **Deferrable second half** — Rollout already ships it that way: refactor alone
   (#1), plugin + dependency (#2), CLI wiring (#3), CI step (#4), manual gate (#5).
6. **Knobs / one-implementation abstractions** — none live: `SteamTarget` gains
   two implementations the same day; the registry seam's fake is a test closure,
   not a new public type; `RegistryKey`'s three variants are exactly the pairs §5
   reads, and #8 pins the set against silent growth.
7. **Value** — the complexity is proportionate: one trait, one generic struct, one
   three-variant enum and one gated dependency are the price of the task's
   no-duplication clause plus Linux-unchanged clause; the alternative readings of
   those clauses were considered and rejected with reasons in *Options considered*.

## Findings

No new, re-raised, or regression findings. The Revision log shows every r0/r1
decision (seam shape, view ordering, decomposition, VDF location, path
namespace, `installdir` hardening, `REG_EXPAND_SZ`, roots branch, gating) still
matches the current text; the editorial pass left the complexity-relevant
structure intact, and none of the *Deferred* items contradicts T-002's Decision.

## FYI (no severity)

- §1's `SteamTarget` is described as "sealed: `Sealed` supertrait, no downcast",
  but `steam_common` is a private module and the blanket `DiscoveryPlugin` impl is
  crate-internal anyway, so the sealed marker guards against an out-of-crate
  implementor that cannot exist; it is ceremony, not risk. A correctness or
  complexity-strengthening pass could drop it; not worth a round here.
