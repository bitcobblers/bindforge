# security review r1c4 (ratification)

verdict: approve

Scope: `git diff 599636095daad2a723b1072ec8906ee3d57a3c84 1d1e68a6726648633a3a2139ed1616eedda39a07` on `02-proposal.md`, plus the full current document. Inputs: proposal, ADR-004, Revision log, my r1 security findings. Checklist: `references/security.md` (items 5–6 — dependency advisories, privileges — unchanged by this revision: no new dependency or privilege surface is introduced; the moved files remain trusted, repo-local).

## Findings

None. No open blocker or major findings from this pass.

## Status of my previously reported findings

- [resolved][minor] Test strategy #2 — the fail-closed check was the grep-vs-fixture contradiction. The revised text ("a file listing (`git ls-files tools/orchestration/tests/`) finds zero files named `pipeline-*.test.ts`; in-file fixture strings are not files (Deferred)") is exactly the name-level check the finding asked for: the deferred fixture strings (`pipeline-lane.test.ts` at `pipeline-independence.test.ts:145`, per unchanged A14) cannot satisfy it, it is satisfiable after a fully applied move, and it fails on any tracked leftover flat filename. Deferred now reads "they are not in Test strategy #2's listing" — consistent with the check and with ADR-004 (Decision bullet 2 forbids fixture changes; the ADR nowhere requires refreshing them), so the deferral is not re-opened. Risks row 1's mitigation ("fails closed on any leftover flat filename") matches the new mechanism.
- [resolved][nit] Test strategy #4 / A19 — A19 already carried (and this revision did not remove) the closure clause: "`TEST_PATH_RE` is the only test-path classifier in the gate and ownership code, so nesting cannot silently escape a pattern … consumed by `stagePathAllowed` … and the revert check (`runner.ts:1031`)`. That is the second accepted fix alternative (single shared regex ⇒ shared classification); the revert check's nested-path behavior is no longer carried by A7's regex argument alone.

## Regression check against security items approved in r1

- Chosen design ¶3 trust boundary: still present and still true after the ¶2 expansion — the 16 `new URL('<rel>', import.meta.url)` rewrites are static, repo-local specifiers resolved against the moved file's own location; no untrusted, network, or user-controlled input is parsed or interpolated ("Trust boundary: nothing here parses untrusted or network input"). Checklist item 3 unaffected: the `../` traversals are authoring-time constants, not runtime path input.
- Diff-bounding guard (my r1 minor's mechanism): ¶2's rewrite target is now A8's closed, grepped 87-site enumeration (62 imports + 16 `new URL` + 1 comment; the 4 inert `'../escape'` fixture strings and the 2 non-moved `cli.test.ts` sites are explicitly outside it). Test strategy #1's allowed set matches ¶2's set exactly, and Risks 3 still asserts it. The removal of the old "never a content-wide substitution" sentence is not a falsification: the positive enumeration plus #1's closed allowed set preserve the same bound, and the count identities (65+2+16+4=87) are self-consistent.
- Goals ("only at Chosen design ¶2's depth-sensitive sites and its one comment line"), A14, and A19 are unchanged or consistent with the changed text. Revision log carries one current entry per decision, including what was not changed.

## FYI (no severity)

- `git ls-files` sees only tracked files; an untracked leftover flat copy would escape the listing but is caught by the same item's "discovers and runs exactly 26 test files" count, so fail-closed holds — noting for correctness only, no change requested.
- The ADR bullet 2 amendment clause wording ("a relative path reference — code or comment — naming a location this move passes takes its new depth") is looser than A8's closed enumeration it sanctions; the mechanical guard (#1) is enumeration-keyed, so this reads as a governance/wording question for the correctness seat, not a security hole.
