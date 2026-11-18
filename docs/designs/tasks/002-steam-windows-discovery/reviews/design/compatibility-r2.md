# compatibility review r2 (editorial confirmation pass, scope: reviews/design/editorial-r2.diff)

verdict: approve

## Scope and inputs

Single editorial confirmation pass of round 2. Inputs: `02-proposal.md`,
`docs/tasks/T-002-steam-windows-discovery.md`, the Revision log, the named
editorial diff, and my previous findings (none — the editorial phase is a
single language pass per round, and I carry no open r1 compatibility
findings into this scope). The diff was confirmed against the applied
revision via `git show b3a00277` (allowed read-only git): the eleven changed
lines in the commit are exactly the eleven hunks in the diff, so the diff is
a complete and accurate account of the revision.

## Checklist walk (compatibility.md)

1. **Compatibility matrix — every row has a status.** Not touched by this
   diff; all 18 rows carry Yes / No / Partial / "Partial (local only)".
   Re-reading the matrix's cross-references (rows cite §2–§5, §7, §8, §9,
   A4–A12, A17–A19, *Rollout* #4/#5): all targets still resolve. Pass.
2. **OS input APIs (X11/Wayland/Raw Input/HID).** Not applicable — the
   design touches no input APIs; its platform-specific surface is the
   Win32 registry, named with view flags (§3) and target-gated. Not
   touched by the diff. N/A.
3. **Node version floor.** Not applicable — no `tools/orchestration`
   change in the proposal or the diff. N/A.
4. **Native modules / silent native dependencies.** Not touched: the only
   new dependency is the target-gated `winreg` (§9), the lock posture is
   unchanged (hunk 7 is a comma), and *Test strategy* #11's committed-
   `Cargo.lock`-diff scope (`winreg`, `cfg-if`) still matches §9's claim.
   Pass.
5. **Keyboard reality (layouts, dead keys, IME, non-ASCII names).** Not
   applicable — no binding-name or keyboard handling surface; the diff's
   hunks 2 and 10 ("that user" → "the user", #8 rewording) are prose-only
   in *Options considered* and *Test strategy* #8 and bind nothing. N/A.
6. **File paths (separators, case, config dirs).** Not touched: the path
   story (canonicalize, `\\?\` extended-length namespace, `absolute_env`,
   the `\` refusal in §6) is unchanged; hunks 6 and 7 add "that" and "with"
   only. The claim in §4 that a non-`REG_SZ` host "is covered by the
   `%ProgramFiles%` default or by `--steam-root`" (hunk 5, was "picked up
   by") states the same fallback as §5 and the Compatibility matrix row for
   non-`REG_SZ` values. Pass.
7. **Test strategy names a platform test per supported platform row.**
   Untouched: #11 (Linux gates), #12 (`x86_64-pc-windows-msvc` compile),
   #13 (manual Windows gate, named lone gate for §8's roots branch), and
   *Rollout* #4 (`windows-gnu` local check) still cover every row the
   matrix claims. Hunk 8 (`§"Options considered"` → `§*Options considered*`)
   restyles a cross-reference to the named, non-numbered section; the
   referenced migration target ("`windows` / `windows-core` ... recorded as
   the migration target if `winreg` goes unmaintained") is still there, so
   the Risk row's pointer still resolves. Pass.

## Diff-specific checks

- All eleven hunks are spelling ("favour"→"favor", "deduped"→"de-duped"),
  grammar ("starts" → "starts with"; "in favor of"), article/clarity ("that
  user" → "the user"; "without one" → "without a single"), and punctuation
  (comma in §9) edits. None alters a normative claim, a requirement, an
  anchor, or a cross-reference target.
- Hunk 11 (#8: "which *Test strategy* #12 is what compiles" → "which *Test
  strategy* #12 compiles") preserves the claim that #12 compiles the
  `#[cfg(windows)]` seam; the §8/Rollout #5 wording that still calls #12's
  role "type-checks" is consistent — `cargo check` type-checks, and #13
  remains the named execution gate for the roots branch.
- Hunk 3 ("described under them" → "described below") and hunk 9 (Risk-row
  reference restyle) keep every cited section present in the document.
- Revision log: no entry is contradicted by the diff; the log's own r1
  entries (view ordering, roots branch, A1b split, gating shape) describe
  text the editorial pass did not alter. No rejected finding is being
  re-raised here.
- Deferred / out of scope: untouched by the diff and consistent with
  T-002's Decision; nothing re-opened.

## Findings

None. Zero blocker, major, minor, or nit findings within the compatibility
specialty for this diff.

## FYI (no severity)

- Hunk 1 normalizes "favour" → "favor"; the document is otherwise
  consistently American ("behavior", "neighbor"-style spellings absent but
  "behavior" used throughout), so the editorial direction is coherent —
  language-consistency observation for the editorial seat, not a
  compatibility matter.
