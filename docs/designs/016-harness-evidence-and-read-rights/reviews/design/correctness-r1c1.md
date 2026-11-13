# correctness review r1 — dissent pass c1

verdict: approve

Scope: the diff 2160215..9b6b5ed plus the full current document, checked
against `docs/adrs/016-harness-evidence-and-read-rights.md` and the Revision
log. Inputs kept as instructed: proposal, ADR, revision log, my round-1
findings; no review reports or transcripts opened.

## Round-1 findings — closure check

1. [closed] §3 doc-family double rule set — the first bullet's narrower
   glob/grep clause is gone; "Then the run-record block below: it is the single
   authority for these families' run-record denies" makes one rule set per
   agent, and Test strategy #4 now pins doc families "carry the same run-record
   `glob`/`grep` denies as the seats". Real fix, not rewording.
2. [closed] §3 bare-name inspection set — now "byte-for-byte as
   `code/coder.md` ships it — star form throughout". Verified against
   `.opencode/agents/code/coder.md`: every inspection entry is star-form
   (`cd *`, `ls *`, `cat *`, `head *`, `tail *`, `wc *`, `grep *`, `rg *`,
   `find *`, `git status|diff|log|show *`). Test #4 pins `wc <file>`/`cat
   <path>` with arguments. Real fix.
3. [closed] §1 prompt route to own report — "re-reading it is allowed through
   your `git diff*` allow, since your `read` tool stays fenced off `reviews/`
   (A21)". Real fix.
4. [closed] unregistered artifact counts — A24 registered as `verified`; I
   independently re-measured every cited count: 014 proposal 733, 015 728,
   012 599, 013 557, 011 479, this run's research 440 — all exact. The
   "largest shipped full-flow document in this repo" claim also holds beyond
   A24's measured set: the other in-repo proposals (003, 004, 005, 006, 007,
   008, 010) are 360/334/397/379/403/286/660 lines, all under 733. Cap 600→900
   correctly sits above 733. Real fix.
5. [closed] §3 ordering contradiction — "appended after every other rule
   except the per-seat re-open below" vs "per seat agent after the deny" are
   now consistent orderings.

## Verification performed

- A16/A17 (rewritten rows) against https://opencode.ai/docs/permissions/:
  `*` spans `/`; a rule ending ` *` also matches the bare command; last match
  wins; `grep` resource = requested regex not search path; `glob` resource =
  requested pattern; `read` resource = Location-relative or canonical absolute
  path; backslashes normalized to `/`, case-insensitive matching on Windows;
  shell resource = scanner-produced command string with compound commands
  scanned leg by leg. All confirmed.
- A18 against https://doc.rust-lang.org/book/ch11-01-writing-tests.html:
  cargo test ends each target with a `test result:` line. Confirmed.
- Node claim (new in Compatibility/Rollout): `engines.node >=22.23.5` "for the
  2026-11-11 Node security release" — Node v22.23.5 is a real security release
  published 2026-11-11 (nodejs.org November 2026 security release notice);
  current package.json:30–31 is `>=22.23.4`, matching the cited basis and the
  rollout's "rises to" framing. Confirmed.

## Findings

- [minor][new] Chosen design §1, shell branch — "collect every `REVIEWS_KEY`
  match in the command; if the set is non-empty and every match canonicalizes,
  under the rule above, to `own`, exempt" combined with the identity rule's
  "resolved against the repo root — lies under the attempt's own design
  directory" — the input to canonicalization is ambiguous, and both readings
  have a defect. If the *match* (a span starting at `reviews/`) is what
  canonicalizes, resolving it against the repo root never puts it under the
  design directory, so under a literal reading no shell access is ever exempt
  and Test #3's "shell `git diff <own>` is clean" contradicts the normative
  rule. Under the sensible reading (canonicalize the full path token), a
  legitimate relative spelling the per-seat permission wildcard itself allows
  — `cd docs/designs/<dir> && git diff reviews/design/<specialty>-r1c1.md`
  (`*` matches zero characters, and the inspection set allows `cd *`) —
  resolves to `<repo>/reviews/...`, outside the design directory, and fails
  the new directory test. That spelling was exempt under the pre-revision
  `reportKey(value) === own` rule, so the r1 fix introduces a regression in
  exactly the friction class Decision 1 removes. Fix: canonicalize the full
  path token the match sits in, and apply the design-directory test only to
  values that spell a design directory (bare `reviews/...` spans pass on
  reportKey equality alone); add the cd-relative `git diff` spelling to Test
  #3's clean set.
- [nit][new] Chosen design §1 identity rule — "normalized the way the
  permission layer normalizes paths (A17: `\` to `/`, case folded)" — the
  gloss over-attributes: A17 and the cited page make case-insensitive matching
  Windows-only, so "the way the permission layer normalizes paths" on the only
  exercised host (Linux) does *not* case-fold, while Test #3 expects
  "case-variant spellings" of own report to be clean. The audit therefore must
  fold case on every platform by its own rule — a deliberate superset of the
  permission layer's normalization (and of ADR Decision 1's "identity match on
  the report key"; unreachable through the permission layer on Linux, so no
  exploitable over-exemption, but the citation claims support it does not
  give). Fix: state the audit's canonicalization folds case unconditionally as
  its own rule citing A17 only for `\`→`/`, or drop "case folded" and the
  case-variant spellings from Test #3.

## FYI (no severity)

- The new tail-risk row ("can carry absolute paths or registry text … no
  redaction pass ships") and the provenance header ("never instructions to the
  seat") are prompt-injection/data-leak postures — security seat's call.
- The macOS/Windows "not exercised" reclassification and the `engines.node`
  bump interact with other hosts' claims — compatibility seat's ground.
