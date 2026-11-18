# performance review r2

verdict: approve

Scope: the editorial-r2 diff (`reviews/design/editorial-r2.diff`) — the single
language-confirmation pass of round 2. Inputs: the diff, `02-proposal.md`,
`docs/tasks/T-002-steam-windows-discovery.md`, the Revision log. The diff's
eleven hunks are word-level substitutions only (`favour`→`favor`, `that user`→
`the user`, `under them`→`below`, `one`→`a single`, `picked up`→`covered by`,
`way`→`way that`, `starts`→`starts with`, one added comma, `§"Options
considered"`→`§*Options considered*`, `deduped`→`de-duped`, `#12 is what
compiles`→`#12 compiles`). No changed sentence carries a performance claim that
the substitution falsifies.

Checklist dispositions:

1. Input-path latency budget — n/a: this is a batch discovery scan, no
   keystroke/event path; the diff changes no cost-bearing text.
2. O(n²) structure costs — unchanged: candidate list is fixed-size (three
   registry reads + two env defaults), roots de-duped by the canonical-path
   rule (§5 "a double-listed root costs nothing"); the `de-duped` hunk is
   spelling and stays consistent with §2's `sort → de-dup` rule and #3's
   "not de-duped by `host_candidates`" wording.
3. Memory growth / caches — n/a: caching stays Deferred; no new text claims a
   cache.
4. Startup cost — unchanged: the 1 MiB / valid-UTF-8 read gate is untouched by
   the diff.
5. Cache invalidation — n/a: no caching introduced.
6. Research baseline gap — none introduced; no numeric claim was edited.

Spot-checks on changed references: `§*Options considered*` matches the existing
`## Options considered` heading; "which *Test strategy* #12 compiles" is true —
#12 is the `cargo check --workspace --all-targets
--target x86_64-pc-windows-msvc` step that compiles `registry_string_value` and
the `#[cfg(windows)]` code; "described below" in §1 correctly points to the
rewiring paragraph that follows the code block in the same section.

## Findings

None. Zero blocker, zero major, zero minor, zero nit from this seat on this
diff.

## FYI (no severity)

- §4's edited sentence says the fallback is "the `%ProgramFiles%` default"
  while §5 lists two defaults (`%ProgramFiles(x86)%\Steam` first, then
  `%ProgramFiles%\Steam`); the singular spelling predates this edit (the hunk
  only changed the verb) and is a wording nit for another seat, not a
  performance issue.
