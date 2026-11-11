# security review r1

verdict: approve

## Findings

- [minor][new] §1 Fence runner-reserved paths in the ownership matrix —
  "return `false` when the path's last segment is `STATE_FILE`
  (`.pipeline.json`), starts with `${STATE_FILE}.`" — the last-segment rule is
  an exact-string match with no case normalization. On case-insensitive
  filesystems — macOS is listed `yes` in the Compatibility matrix — a stage
  writing `.Pipeline.json` / `.PIPELINE.JSON` passes the string rule while
  hitting the *same* on-disk file the fence exists to protect, so "the
  worktree copy and the committed copy can only carry runner-authored bytes"
  (Goals) is not guaranteed on a supported platform. Fix: in the same §1
  sentence, make the reserved-name comparison case-insensitive when the
  underlying filesystem is case-insensitive (practically: `process.platform`
  `darwin`/`win32`), and add one note to the macOS row of the Compatibility
  matrix that the fence rule is case-folded there. No new mechanism — a
  qualifier on the existing rule.

- [nit][new] Node advisories (checklist item 5) — §Compatibility matrix:
  "Node `>= 22.23.4` (engine floor)" — dependency check performed: 22.23.4 is
  itself the fixed release for CVE-2026-111497 (the "Node2Shell" Brotli RCE,
  High), and the Node project has pre-alerted another 22.x security release
  for 2026-11-11 (the day after this ADR's date), highest severity High, with
  details not yet public. Because the design states a *floor* (`>=`), not a
  pin, newer patched releases satisfy it — no unfixed advisory is forced on
  the design, so this is not a blocker. Fix: one sentence in the matrix row
  making the posture explicit — the floor is a minimum and operators run the
  newest patched 22.x (the 2026-11-11 release once it lands).

- [nit][new] §3 An escalation that survives its own failing write — "prints
  the repair commands — `git checkout HEAD -- <designDir>/.pipeline.json`,
  ... the `rm -r <designDir>/.pipeline.json && git checkout HEAD -- …`
  variant" — these are path-interpolated command strings an operator
  copy-pastes; an unquoted path would mis-tokenize in a shell (run slugs are
  conventionally simple, hence nit, not a finding against checklist item 3 —
  the commands are printed, never executed by the runner). Fix: specify the
  printed commands with the paths single-quoted.

## Checklist coverage

1. Trust boundaries: classified — stage writes under `docs/**` treated as
   hostile (fence), runner sole writer of reserved names, committed copy as
   recovery anchor, pre-fence checkpoints' residual explicitly in Risks,
   consistent with ADR-006:55–57 interim posture. Pass.
2. Parsing untrusted data: schema validation on cross-version load
   (`version: z.literal(1)`, shape refusal); no decompression or code
   evaluation anywhere in the design. Pass.
3. Path handling: last-segment matching is prefix- and traversal-insensitive
   ("at any depth", Test 1); no runner-side shell interpolation; the one gap
   is case-folding (finding above). Pass with minor.
4. Secrets/PII: nothing new stored or logged; gate-log entries and failure
   messages carry errnos and local paths only. Pass.
5. Dependencies: webchecked Node 22.x advisories — see nit; no dependency
   pinned to a version with an unresolved advisory. Pass.
6. Privileges: the design only *narrows* writable surface (fence) and adds
   an exclusive-create lock; no new privileges. Pass.
7. Fail closed: refusal "never writes or deletes anything"; unwritable
   checkpoint escalates with a terminal record (exit 2) instead of crashing;
   lock race admits one winner. Pass.

Deferred items (lock-ID nonce, digest reconciliation, v2 ladder, Windows CI)
were checked against ADR-006's Decision and interim posture: none contradicts
the source document; not re-opened.

## FYI (no severity)

- §2 New fixtures: copying the live run-003 checkpoint into
  `tests/pipeline/fixtures/checkpoints/` duplicates run state in version
  control — hygiene/process seat's call, no security exposure since the file
  is already committed.
- §4's acknowledged narrow stale-clear race is a compatibility-window concern
  for the compatibility seat; the deferral itself is ADR-consistent.
