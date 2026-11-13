# compatibility review r3c0 (ratification of the r3 text)

verdict: approve

Scope: `git diff c1b351d 92f71e0` on `02-proposal.md`, then the full current
document, plus `docs/adrs/010-lightweight-plan-flow.md` and the Revision log.
Ratification only: does the r2 revision break what this seat approved at r3,
or make a section this seat cited false.

## Checklist walk (compatibility)

1. **Matrix rows all have a status** — all thirteen rows carry `yes`, `yes (no-op)`
   or `no`; no `unknown`, no empty cell. The r2 diff touches no matrix row, so the
   row set this seat approved at r3 is byte-identical. Pass.
2. **OS input APIs** — the design declares no input surface at all ("No new platform
   surface: a stage, a seat list, a config record", A20); the target is the
   orchestration CLI on linux. Justified by absence, as approved at r3. Pass.
3. **Node floor** — `Node ≥ 22.23.4 (package engines)`, A20; nothing in the changed
   text introduces an API above the floor (the additions are a `Flow` union, a zod
   `z.enum` key, spread arithmetic, an agent/skill/prompt file). Pass.
4. **Native modules** — pure TS, no prebuilds/node-gyp; the changed text adds no
   dependency, and the Windows/macOS row keeps the "the size meter is the JS line
   count … not a shell-out" reason that backs it. Pass.
5. **Keyboard reality** — no text-entry surface is touched; not applicable to this
   scope, unchanged by the diff.
6. **File paths** — A28 (git plumbing output, repo-root-relative, POSIX separators,
   no absolute / `.` / `..`) plus `moves.ts` Rule 1 and "case is **not** normalized
   — as today" are unchanged by the diff; the r2 additions add no path arithmetic.
   Pass.
7. **Platform test per supported row** — the only `yes` platform row is linux at the
   `engines` floor, and Test strategy #21 names exactly that (`npm run check` green,
   no new platform-specific code path). The `no` rows need no test and #21 says so.
   Pass.

Nothing in the r2 revision breaks a compatibility claim this seat approved. The
r2 closure (the planner's permission list copied verbatim from `designer.md:4-22`,
A12 carrying the shape) adds agent-config text only; it introduces no platform,
runtime, path or CI surface, and the new A12 basis cites agent config files, not
a new environment. Terminology sweep is clean: no `heavy`/`heavier` survives, so
the matrix, Risks and Rollout rows this seat cites read consistently.

## Findings

- [nit][re-raised] Compatibility matrix — "Existing ADRs and briefs (no `flow` frontmatter)" row, "`configFor('full', base)` is deep-equal to `base`" — unchanged text: the registry paragraph narrows the claim to "deep-equal to a **complete** `base`, #3" and #3 pins a complete literal by compiles-plus-five-seat resolution, not by deep equality on an arbitrary `base`; Rollout 1 ("this is a no-op for every run (#3)") cites #3 for the same unqualified reading. Nothing is false for the complete CLI-shaped config both sentences describe — the citation stays imprecise. Fix: matrix row reads "deep-equal to a complete `base`"; Rollout 1 reads "no-op for every run (a complete `base` is deep-equal, #3)".
- [nit][re-raised] Test strategy #21 — "no macOS or Windows job exists, so both compatibility rows read `no` and no test claims them" — unchanged text: the matrix carries one merged row, `| Windows / macOS | no | …`, so a reader cross-checking the anchor finds one row for the two #21 cites (the Deferred bullet, "move those compatibility rows from `no` to `yes`", inherits the same plurality). The status claim itself matches that row and the Deferred entry. Fix: "no macOS or Windows job exists, so the `Windows / macOS` row reads `no` and no test claims it", and the Deferred bullet reads "that compatibility row".

## FYI (no severity)

- The r2 A12 sentence asserts a universal over document-stage agents ("no document-stage primary agent reads, writes or shells") on two cited agents (`designer.md`, `researcher.md`); a reviewer checking completeness will want the agent set enumerated — correctness/security territory, not this seat's.
