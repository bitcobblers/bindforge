# security review r3 (c2 ratification pass)

verdict: approve

## Scope and method

Ratification only: reviewed `git diff b08c5589..61907ce3` of `02-proposal.md`, then read the
full current document, `docs/tasks/T-001-steam-linux-discovery.md`, and the Revision log.
Checked only whether the revisions broke what this seat approved in r3 or made a section this
seat cited false. One named claim added by the revision was verified against the file it
cites: `schemas/reach-classes/v1/spec.md` ("Version gate and size budget") does state the
shared budget — "documents larger than 1 MiB or nested deeper than 16 levels are rejected
before parsing", with the table-data section's "refused before it is read into memory" — so
§4's new "both caps restate the repo's shared parse budget" sentence is accurate, not invented.

## Findings

None. No blocker, major, minor, or nit arising from the changed text.

## Disposition of this seat's r3 findings

- [minor, from r3] §4 KeyValues reader — 1 MiB cap enforcement point. Not a regression and
  effectively narrowed by the revision rather than broken: the changed §4 sentence now ties
  the caps to `schemas/reach-classes/v1/spec.md`, whose budget is defined as rejection
  *before the file is read into memory* / *before parsing*. An implementer following the
  cited budget cannot satisfy it by reading-then-measuring, which was the failure mode the
  r3 finding named. The design still does not name the mechanism (`fs::metadata(path)?.len()`
  pre-check or a capped read) inline; that residual wording preference is not a regression
  and is recorded below as an observation, not a finding.

## Regression check, changed text hunk by hunk (security lens)

- A9 / §2 / Revision log q1 — `home` switched from `std::env::home_dir()` to
  `std::env::var_os("HOME")`; unset, empty, or relative means no home → empty candidates →
  `Ok(vec![])`. Fails closed, drops the passwd-entry fallback (a strictly narrower scan
  surface), and Test strategy #5 pins empty/relative `home` → empty list. No new trust
  boundary: HOME was already an input to every candidate. No regression.
- §1 — test-coverage note for `from_host()` (Rollout #5 manual probe). No security content.
- §2 canonical path rule — de-duping widened to one set across all selected roots per
  `discover()` call; strictly improves the symlink/double-scan posture this seat approved,
  and "nothing the plugin emits carries an unresolved `..`" is retained verbatim.
- §3 root-as-library (`<root>` replaces `<root>/steamapps`; fixture `"0" → ".."`) — the
  approved path-validation machinery is untouched: VDF `path` values still pass
  canonicalize + existing-`steamapps`-gate, absolute-vs-join rule unchanged, no recursion
  below `steamapps/` so A4 prefix look-alikes are never opened, and the Risks row on
  user-editable `install_path` retains every mitigation. The `..` inside a fixture VDF path
  is exactly the canonicalize-and-gate case already approved. The root/library semantics
  themselves are the correctness seat's.
- §4 — see disposition above; the UTF-8 skip, `#base`/`[`-conditional rejection, 16-level
  cap and positioned errors are unchanged.
- §7 — the single `Err` case retriggered to "zero games emitted and at least one file
  skipped". Message content unchanged (root names + skip count, no file contents, stderr
  only); per-file skip-and-continue still rejects malformed input cleanly with no partial
  apply. The narrower trigger is a diagnostics-surface change, consistent with the existing
  deferral ("Richer per-skip diagnostics … §7's error path is the v1 floor"); nothing unsafe
  is emitted or applied in the cases it now returns `Ok(vec![])` for. Not a security
  regression.
- §8 import tidy, §10 deprecated-lint note, Compatibility matrix Windows-row tidy, Test
  strategy #3/#5 updates tracking the root-as-library and HOME changes: no security surface.

## Checklist dispositions

1. Trust boundaries — unchanged classifications; VDF/ACF still treated as user-editable and
   validated. 2. Parsing untrusted data — size/depth caps intact, now grounded in the repo's
   shared budget; no code evaluation, no decompression. 3. Path handling — canonicalize +
   gate + unresolved-`..` guarantee intact and strengthened by cross-root de-dup; new
   relative-`$HOME`/`$XDG_DATA_HOME` guards fail closed. 4. Secrets/PII — unchanged; stderr
   message still names roots and a count only. 5. Dependencies — no new dependency (A15, §10
   unchanged in posture); nothing new to advisories-check. 6. Privileges — read-only user-
   owned filesystem access, unchanged. 7. Fail closed — per-file malformed rejection intact.

## FYI (no severity)

- Observation for the correctness seat, not a security finding: §7's new Err trigger keys on
  "at least one file was skipped", while an unreadable *library directory* is described in
  the same section as a skipped directory; whether that counts toward the trigger is
  unstated, and Test strategy #7 pins only the all-unparseable-manifests case.
- The two parse-budget constants now live in two places by stated design ("as plugin-local
  constants, since `crates/discovery` takes no `bindforge-core` edge"); drift risk is a
  complexity/maintainability observation.
