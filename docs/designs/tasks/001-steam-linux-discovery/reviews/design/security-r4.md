# security review r4

Scope: the round-4 editorial diff
`docs/designs/tasks/001-steam-linux-discovery/reviews/design/editorial-r4.diff`,
read against `02-proposal.md`, `docs/tasks/T-001-steam-linux-discovery.md`, and the
proposal's Revision log. This is the single editorial confirmation pass; the security
seat had no open findings entering the round, so the test applied is whether any changed
passage weakened a security property the proposal states.

verdict: approve

## Checklist walk

1. **Trust boundaries** — not applicable to this diff. The diff touches no passage that
   classifies a data source; `libraryfolders.vdf` / `appmanifest_*.acf` remain
   user-editable, user-owned files (§3, §5 untouched by the diff).
2. **Parsing untrusted data** — not applicable to this diff. The only §4 change is
   "behaviour" → "behavior"; the accepted/rejected grammar subset, the 1 MiB and
   16-level caps, the non-UTF-8 skip, and the no-`#base`/no-conditional rejection are
   byte-identical.
3. **Path handling** — reviewed most closely, since §2's canonical path rule is the
   security spine and one hunk sits inside that paragraph. The change splits
   "It is the seam tests and `--steam-root` use; `from_host()` is a thin wrapper over
   it." into "It is the seam that tests and `--steam-root` use; / `from_host()` is a
   thin wrapper over `from_roots()`." Naming `from_roots()` — the function the rule is
   stated against — instead of the ambiguous "it" *tightens* the reading: a caller-supplied
   `--steam-root` demonstrably passes through canonicalization, the `steamapps`-directory
   gate, sorting and de-duplication, and `from_host()` inherits the same treatment rather
   than bypassing it. No path-handling property was weakened; the §2 sentence "nothing the
   plugin emits carries an unresolved `..`" and the §5 `installdir` rejection
   (`.`/`..`/absolute/`/`) are outside the diff and still hold together with it.
4. **Secrets / PII** — not applicable; the diff touches no logging or emission text, so
   §7's "names the selected roots and the count of skipped files and nothing else — no
   file contents" is unchanged.
5. **Dependencies** — nothing to check. The design names no dependency it adopts (the
   hand-rolled reader is chosen precisely to add none), so there is no advisory to look up.
   Verified against the repo: `Cargo.lock` still holds exactly 32 `[[package]]` entries, and
   neither `crates/discovery` nor `apps/discovery-cli` is touched by the diff.
6. **Privileges** — least privilege unaffected: the diff adds no capability, no network
   use, no write path; `discovery list` stays read-only over the user's own Steam tree.
7. **Fail closed** — the §7 error surface is untouched; the only nearby change is the
   serial comma in *Test strategy* #10, whose gate commands are unchanged. `cargo fmt --all
   -- --check` runs clean on this tree, so the fmt leg of the changed #10 statement is
   currently true as written.

Verbatim-checked changed passages (Context "…trait, and the data model…"; Context "…settles
each such question that traces to T-001's Decision clauses…"; A9 "…so, `home` is read from
`std::env::var_os("HOME")`…"; §1 "both constructors say more than `new()` would"; §4
"behavior"; #10 gate list) carry no change of technical meaning. The A9 change is a comma
only; the HOME unset/empty/relative → "empty candidates, `Ok(vec![])`" behavior in §2 is
untouched.

## Findings

None. No blocker, major, minor or nit within the security specialty.

## FYI (no severity)

- A9 row, "…for a set-but-empty `$HOME`; so, `home` is read from…": the inserted comma
  after "so" reads as a spliced pause; the pre-diff "so `home` is read" was cleaner.
  Language-only, and the editorial seat's call, not a security concern.
- Revision log: every entry is labelled `r0` (one carries an inline "(r3)"), so rounds
  1–3 are not visible as their own decisions. If the run record is meant to show one
  current entry per decision, the log looks stale; that is a process/records observation
  for the runner, and the diff does not touch it.
