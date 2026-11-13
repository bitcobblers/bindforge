# security review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`), read against
`02-proposal.md` and `docs/adrs/016-harness-evidence-and-read-rights.md`. Inputs
were only those named by the caller plus my own prior findings (none — the
editorial phase is a single language pass per round).

## Checklist coverage (security seat, against the diff)

1. **Trust boundaries** — every hunk is language-only; none re-classifies a data
   source. §1's fail-closed clause survives the edit verbatim in meaning:
   "Anything else — a directory, a glob, a listing, a command naming a second
   report, another design directory's same-named report — is not an identity
   match and fails as it does today." ("as today" → "as it does today" is
   grammar.) §3's run-record denies and the "the deed audit stays the backstop"
   sentence are unchanged in substance; the "deeds audit" → "deed audit" fix
   brings the proposal into agreement with ADR-016 Decision 3's own wording.
2. **Parsing untrusted data** — §2's decode/bound chain is untouched
   semantically: "decoded once as UTF-8 with replacement so the stored tail never
   cuts a code point", the `MAX_GATE_TAIL_CHARS` bound, and `printable` at write
   and render all remain. Test #1's "bytes that are not valid UTF-8 become
   replacement characters, never a split code point" is the same assertion with
   a corrected verb.
3. **Path handling** — the normalization/resolution sentence ("normalized the way
   the permission layer normalizes paths (A17: `\` to `/`, case folded), resolved
   against the repo root") and "a prefix-less token resolves outside the attempt's
   design directory and fails closed" are outside the diff and unchanged.
4. **Secrets and PII** — the committed-carrier rule from the r2 revision log
   still reads as written: the build note is "the one committed carrier", with
   citation "line-selectively, dropping any credential-bearing or absolute-path
   line". No hunk softens it.
5. **Dependencies** — checked against advisory sources: `@opencode/client ^2.1.4`
   carries no entry in Snyk's vulnerability DB (0 direct findings across 2.1.x and
   the 2.2.x line); the `engines.node: >=22.23.5` claim matches the real
   2026-11-11 Node.js security release (22.23.5 'Jod', highest severity HIGH), so
   the floor is the patched build rather than a vulnerable pin. No unfixed
   advisory against anything the design names.
6. **Least privilege** — the Context/Goals rewording ("families get a read-only
   shell allowlist and both review panels get the project's check commands")
   states the same grant as ADR-016 Decision 3; the deny-then-reallow order and
   the per-seat `git diff *reviews/<phase>/<specialty>-*` allow are untouched by
   the diff.
7. **Fail closed** — the diff introduces no new failure path and preserves the
   existing ones ("a sibling report … fails", "`ls .../reviews` fails",
   "transcripts access is never exempted").

Every hunk in the diff is grammar/clarity only; none changed a permission,
identity, bounding, or redaction statement. Nothing the security seat relies on
was broken, and no section cited in my earlier (empty) findings was made false.

## Findings

None at blocker, major, minor, or nit severity within this specialty.

## FYI (no severity)

- Options considered — "raw logs blow past the 8 MiB checkpoint ceiling and the
  seat-board clone problem A7 exists for": the editor's own note asks the author
  to clarify. The bounding facts themselves (8 MiB loader cap, A6/A7 caps, Test
  #6) are stated elsewhere, so nothing resource-bounding is lost either way —
  clarity, not security.
- Test strategy #3 — "a sibling names the child session" stays ambiguous after
  the semicolon fix, and the editor note already records it. The audit
  expectation it encodes (a child reading a *sibling* report fails, naming that
  sibling) is the counterpart to the intentional child-walk exemption, so whoever
  clarifies it should keep both halves testable.
