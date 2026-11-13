# compatibility review r1

verdict: request-changes

Scope: whole document (`02-proposal.md`, c0, round 1). Inputs: the proposal,
its Revision log (r0 — no decisions recorded yet, so every finding below is
`new`), and `docs/adrs/016-harness-evidence-and-read-rights.md`.

## Findings

- [major][new] Compatibility matrix — "| macOS | unknown | Nothing
  platform-specific is added (pipe capture and `spawn` are portable Node), but
  no environment here exercises it |" and "| Windows / WSL2 | unknown | Same
  posture as macOS |" — the checklist makes an `unknown` row a finding, and the
  justification in those notes is contradicted by §3: the doc families gain a
  POSIX-only inspection set ("append the inspection set (`cd`, `ls`, `cat`,
  `head`, `tail`, `wc`, `grep`, `rg`, `find`, `git status *`, …)") and the
  design panel gains spawned `cargo`/`rustfmt`/`npm` legs. Both are
  platform-dependent tool surfaces (no `cat`/`wc`/`find` under a native Windows
  shell; cargo/rustfmt availability per host), so "nothing platform-specific is
  added" overstates the abstraction rather than justifying it. Fix: replace
  `unknown` in both rows with a declared posture — e.g. "not exercised —
  harness change developed and tested on Linux only" — and add one sentence
  naming the two platform-dependent additions (POSIX inspection set, cargo/npm
  check legs) as why no claim is made for those hosts. Document-only; no new
  mechanism.

- [minor][new] §1 Own-report identity — "a `reviews/` access value is the
  seat's own report iff `reportKey(value)` (everything from the `reviews`
  segment on) equals `reportKey(opts.ownReportPath)`" — the identity rule, §3's
  `glob` denies over `docs/designs/**/reviews/**`, and the per-seat shell allow
  all key on a `/`-separated, case-exact spelling of the path, and no
  assumption row (A2, A16, A17) states that access values and shell command
  text reach the audit separator-normalized. Where paths are spelled with `\`
  or differ in case, the deny side fails safe but the own-report exemption
  silently never fires, so Decision 1's promise is unmet on that host while the
  design claims nothing platform-specific changes. Fix: one clause in §1's
  identity rule — `reportKey` compares after normalizing separators to `/` and
  case — or, if the SDK already normalizes, an assumption row saying so with
  where it was verified.

- [minor][new] §3 Own-report re-open — "`shell git diff
  *reviews/<phase>/<specialty>-*` → allow, naming the agent's own specialty in
  the pattern (A11)" — the deny-then-reallow order works only under OpenCode's
  exact matcher grammar (`*` spans `/`, a rule ending ` *` matches the bare
  command, last match wins), and A16 says the shipped pin is "a repo-local
  re-implementation of OpenCode's wildcard matching". The matrix's claim
  ("Pre-ADR-016 OpenCode server config | yes | Only standard rule types … are
  used") vouches for rule *types*, not for this grammar, so the whole §3 deny
  stack silently inverts if the engine's matcher differs from the local
  re-implementation. Fix: name the OpenCode version whose documented grammar
  §3's rule order relies on (A17's permissions page) in one assumption row and
  in that matrix row's note, and say in Test strategy #4 that the local matcher
  must stay pinned to that version.

- [minor][new] Compatibility matrix — "| A run in flight across the merge |
  yes | Pre-016 checkpoint ⇒ `gateEvidence` absent …" — that row claims a flat
  `yes` but covers only the checkpoint and transcript effects; the same merge
  also applies `docMaxLines: 600` and `research` in `docStages` to a resumed
  run, and the Risks table's own row says "A resumed or in-flight run meets the
  new `docMaxLines`: a document already over 600 fails its next document
  stage". The matrix understates a behavior change across the merge. Fix:
  extend that row with the condition — "…; a document already over
  `docMaxLines` fails its next document stage — raise the cap or force the
  stage (Risks)" — keeping the status.

- [minor][new] §2 The gate record — "`tail` the last
  `MAX_GATE_TAIL_BYTES = 2_000` bytes of the leg's merged stdout+stderr, passed
  through `printable` (independence.ts:311) before storage" — the bound is in
  bytes on raw child output, so the slice can cut a multi-byte code point, and
  cargo/npm output under a non-UTF-8 console codepage (the Windows row) is not
  UTF-8 at all; that same string is persisted in the checkpoint and rendered
  into four prompts and the build note. The Risks row on the rolling tail
  addresses only truncation of the `test result:` line, not decoding, so
  "tails are evidence, not parsed contracts" doesn't cover mojibake reaching
  five seats. Fix: one clause — capture bytes, decode once as UTF-8 with
  replacement, then bound the tail in characters (same `2_000` budget) — or
  state explicitly that the head of a tail may drop a partial code point and
  who strips it.

- [nit][new] Compatibility matrix, Linux row — "all changes are Node ≥
  22.23.4 in `tools/orchestration`, agent-config YAML text, and prompt text" —
  the floor is asserted with no anchor (no engines reference, no assumption
  row), so the checklist's Node-floor check can't be verified from the
  document; and §3 edits `.md` agent rule files ("Config edits in
  `.opencode/agents/`, using the rule text already shipped in
  `code/coder.md`"), not "YAML". Fix: cite the declared floor's source (the
  package's engines field) and say "agent rule text in `.opencode/agents/`".

## Checklist coverage

1. Matrix statuses: every row filled, no empty cells; two `unknown` rows →
   major above.
2. OS input APIs (X11/Wayland/Raw Input/HID): not applicable — "No product
   (`crates/`, `schemas/`) change" (Non-goals) and ADR-016's "It does not touch
   the product". Evaluated, no finding.
3. Node floor: no API above the stated floor is named (child pipes, buffer
   bounds, zod `optional()`); floor citation → nit.
4. Native modules: none added; the toolchain prerequisites (`cargo`, `npm`,
   `git`) are the ones `testsGreen` already spawns (A5). Evaluated, no finding.
5. Keyboard reality (layouts, dead keys, IME, non-ASCII names): not applicable,
   same justification as item 2.
6. File paths: separator/case handling of `reportKey` and the `glob`/`grep`
   deny patterns → minor above.
7. Platform test per supported platform: only Linux is claimed supported and
   Test strategy #14 names it. Evaluated, no finding.
8. Deferred section vs the ADR: every deferral matches ADR-016's "Not decided
   here" list and Decision 4's refusal (subagent deny, middle flow tier, served
   slots/KV, read-ceiling enforcement, non-text fence). No deferral
   re-opened.

## FYI (no severity)

- §3 lets review seats spawn `cargo`/`npm`, which writes build artifacts
  (`target/`, dependency caches) into the run worktree; the proposal asserts
  the ownership matrix and `namesReservedSegment` don't change, but nothing
  states how a seat-run build's artifacts stay out of the diff/ownership sweep.
  Correctness/complexity seat's call.
- Merging "stdout+stderr" from two separate pipes gives flush-order interleaving,
  so a "tail from the end" can interleave legs of different streams.
  Correctness seat's call; it does not change the capture budget.
- Design seats running cargo legs on the single-slot host described in A20
  competes with the inference process for the RAM the ADR says is already
  committed. Performance seat's call.
