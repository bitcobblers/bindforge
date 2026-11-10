# security review r1c0

verdict: request-changes

## Findings

- [major][new] Chosen design → Pass-level fan-out in `drive` (`src/pipeline/runner.ts`) — "No pass finalize runs on this path — a failed stage skips ownership enforcement today too, and `finalizeEscalate`'s own `commitStage` lands the siblings' partial reports in the escalation commit. Accept this widened commit" — the escalation path runs `commitStage` (`git add -A`) with **no `enforceOwnership` sweep**, and the fan-out silently widens what that unenforced commit can land. The "today too" justification compares only the failing seat's files: today, every *succeeded* seat in a pass was `enforceOwnership`-checked and committed at its own finalize before anything could reach history, so the escalation commit never carried another seat's output; under pass-level finalize, whenever any seat fails, the complete output of up to four succeeded siblings reaches history only through this unchecked commit. A seat induced — e.g. via prompt injection in the reviewed document — to write a file outside the review ownership rule (`docs/**`, gates.ts:260–273) and then fail its reply validation twice gets that out-of-policy file committed with the permission check the design otherwise preserves, and the run stops with it in history. Fix: on the escalation path, run the same pass-level `enforceOwnership` sweep `finalizeReviewPass` uses, after `writer.drain()` and before `finalizeEscalate` — siblings are settled at the barrier, so the dirty-file scan no longer sees in-flight sibling edits, which was the only reason per-seat ownership was unsound under fan-out; state the posture when the sweep trips (e.g. revert-and-still-escalate). This replaces the sentence "No pass finalize runs on this path — a failed stage skips ownership enforcement today too…", updates the flow diagram's escalation note, and the Risks row "Escalation mid-pass loses sibling work or races its writes" should name the ownership sweep in its mitigation.

## Checklist coverage

1. **Trust boundaries** — no new data source introduced: operator flag/env (validated), inference-server replies (schema-validated, existing path), checkpoint files (trust deferred to ADR-004, consistent with ADR-003 §Self-update), reviewer-written files (model-controlled, governed by `enforceOwnership` — the gap above). Reviewed; one finding.
2. **Parsing untrusted data** — no new untrusted parsing; `resolveMaxParallel` gates the env string with a size-capped `^\d{1,5}$` regex plus range check before any side effect. Pass.
3. **Path handling** — no new user-controlled paths; report/transcript paths derive from fixed specialty enums; git stays `spawnSync` with no shell interpolation (A6). Pass.
4. **Secrets / PII** — nothing new leaves the machine (prompts to the inference server and committed reports/transcripts are unchanged in kind); the new wall-clock fields add no sensitive data. Pass.
5. **Dependencies** — design adds none (`p-limit`/`p-queue` named but rejected, not introduced); Node engine floor unchanged (A10). No advisory check applicable. Pass.
6. **Privileges** — no new filesystem/device privileges; writer is scoped to the design dir; `.pipeline.lock` posture unchanged. Pass.
7. **Fail closed** — config validation rejects bad env even when overridden; drained writer prevents stray `.pipeline.json.*.tmp` sweeps; escalation stops driving. The one gap is the unenforced escalation commit (finding above).

## FYI (no severity)

- "finalizeEscalate's own `commitStage` lands the siblings' **partial** reports in the escalation commit" contradicts the same bullet's "siblings already in flight **complete** before escalation is visible" — the reports are complete, not partial; the commit is widened by content, not by truncation. (correctness/wording)
- `--max-parallel` has no explicit upper bound beyond the 5-digit format gate (99999 accepted); harmless only because a pass holds at most 5 seats — fine as written. (correctness)
