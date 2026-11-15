# Security review r1

verdict: approve

## Checklist coverage

1. **Trust boundaries** — satisfied. Every data source the gate reads is
   classified in the text: the corpus is "every markdown file under
   `docs/adrs/` and under `schemas/`" (¶5), read from the run's own worktree
   (A22), "a bounded, repo-local set" (A23). No network source, no user
   input, and no shared binding file is parsed anywhere in the design, so
   the UNTRUSTED-binding-file class does not arise. The trees read are
   version-controlled repo content; the design treats even that as
   validate-before-trust, which is the gate's purpose.
2. **Parsing untrusted data** — satisfied. No code evaluation, no
   decompression; file count is bounded (A23, verified by glob: 19 + 9
   files); the id regex `(?<![\dA-Za-z-])\d{3}-[CLBRS]\d{1,2}(?![\d])` and
   the frontmatter regex are anchored and linear; the only corpus reaching
   the parser is repo-local, so no ReDoS or bomb exposure from an attacker
   position the design does not already trust more broadly (stages have
   commit rights). Per-file byte caps are unstated but bounded in practice
   by the same repo-local trust model — not made a finding.
3. **Path handling** — satisfied. Paths come only from listing two fixed
   prefixes; "`ledger.ts` reads files with `node:fs` … and parses no path it
   did not list itself" (¶1); owner existence builds `docs/adrs/NNN-*.md`
   from a filename-derived number; no shell interpolation anywhere.
4. **Secrets and PII** — not applicable in substance; posture is sound.
   No secret or PII flows through the gate; gate-log output is bounded ("the
   first twelve findings … plus `+N more`", ¶7, mirroring the verified
   `boundedLegs` precedent at gates.ts:457); nothing leaves the machine.
5. **Dependencies** — clean. The design adds no dependency ("no new
   dependency (A14)"); A14 verified against `tools/orchestration/package.json`
   (`@opencode/client ^2.1.4`, `yargs ^17.7.3`, `zod ^4.6.7`). Advisory
   check (GitHub Advisory DB): `@opencode/client` — none; `yargs` — none;
   `zod` — GHSA-mrhx-c7xc-m3w5 (vulnerable `< 4.6.6`, patched 4.6.6: outside
   the declared `^4.6.7` range) and GHSA-m95q-7qp3-xv42 (`<= 3.22.2`): no
   known unfixed advisory at the declared versions.
6. **Privileges** — least privilege. Read-only `node:fs` over two repo
   trees, "never writes" (¶1); no device, network, or write privilege added
   at either hook.
7. **Fail closed** — two gaps, both minor: read-failure behavior is
   unspecified, and a malformed fence fails the join open. See findings.

## Verification run (read-only)

- A9 confirmed: `sourceAdmissible` has one call site, `runner.ts:2438` in
  `runPipeline`; a deny there routes to `finalizeEscalate` (2440–2442).
- A10 confirmed: `finalizeDone` (runner.ts:3570–3603) reads the ADR, flips
  status, then completion-commit paths.
- A13 confirmed: `parseSource` frontmatter regex uses `\r?\n`
  (gates.ts:54–75); A14/A15/A24 confirmed against package.json:23, 30–31,
  45–49. `boundedLegs` exists (gates.ts:457) as the bounded-record precedent.
- Revision log: r0 is empty — nothing dispositioned, so no rejected finding
  is re-raised here.
- Deferred list checked against ADR-021: supplying the taxonomy here matches
  the ADR's "the gate's exact error taxonomy (021-R2's run)"; the backfill
  order deferral to ADR-022 matches 021-R1 and the ADR's "Not decided here".
  No deferral contradicts the ADR — none re-opened.

## Findings

- [minor][new] Chosen design ¶1 (components) / ¶7 (hooks) — "`ledger.ts`
  reads files with `node:fs`, never writes, and parses no path it did not
  list itself" — the design specifies what the gate does when a file
  *parses* wrong (sixteen codes → deny → escalate) but says nothing about a
  file that cannot be read or decoded at all (broken symlink, permission
  error, non-UTF-8 bytes named `.md`). An exception escaping `checkLedgers`
  at the entry hook crashes the run at `runPipeline` with no gate-log record
  and no `finalizeEscalate`, and at `finalizeDone` crashes the completion
  path — the exact crash-vs-escalate-with-record class the harness already
  legislated for itself (runner.ts:2430–2431 comment, runner.ts:3590–3593
  "Security r1, finding 3"), and checklist item 7's "reject cleanly, never
  crash". Fix: one sentence in ¶7 — a corpus read or decode failure makes
  `ledgerGate` deny with `ledger corpus not readable: <file>` (no new code;
  the way its sibling `sourceAdmissible` already converts its own read
  failure into `deny("source document not readable: …")` at
  gates.ts:103–107), so both hooks take the same escalate path a finding
  takes.
- [minor][new] Chosen design ¶4 (handle grammar) — "Handles in a fenced
  block (lines whose trimmed start is ``` or `~~~`, toggled) … are not
  handles", the "fence toggle, no parser" middle path (Options) — the toggle
  flips on any fence-looking line and no check validates the state at
  end-of-file: a single stray fence marker — malformed input any document
  stage can emit — silently suppresses handle detection for the rest of the
  file, so a dangling handle after the stray marker joins green instead of
  red. The check then applies only to part of the file (checklist item 7's
  "never … partially apply"), failing open — and the design's own Risks
  table names silent under-coverage as the defect to close, without
  mechanically detecting its cheapest instance. Fix: ¶7 adds `fence-unbalanced`
  to the code list — a file whose fence state is still open at EOF is a
  finding, turning the stray marker red instead of mute — with one fixture
  added to Test strategy #2.

## FYI (no severity)

- ¶4's fence toggle also ignores CommonMark's pairing rules (a closing
  fence must use the same character and be at least as long as its opener);
  the fail-open half is covered by the finding above, the exact-coverage
  half (e.g. a `~~~` line closing a ``` fence) is a correctness-seat
  judgement call the design makes deliberately by choosing the simple
  toggle.
- ¶2's owner rule calls any `ADR-\d{3}` naming an absent file "reserved", so
  a typo'd number reads green until `nextNnn` reaches it; the design argues
  the bound itself (A18) — correctness seat's call whether the argument
  holds, not a security defect.
