# security review r2

verdict: approve

Scope: the editorial-r2 diff (`reviews/design/editorial-r2.diff`) as confirmed
against `02-proposal.md`. Inputs: the proposal, the task text, the Revision
log, the diff. This is the single editorial confirmation pass; the security
seat carried no open findings from round 1.

The diff contains ten hunks, all language-only: spelling normalization
("favour"→"favor", "deduped"→"de-duped"), grammar fixes ("that user"→"the
user", "starts"→"starts with", "is what compiles"→"compiles", "picked up
by"→"covered by", added comma and "that"), and one cross-reference style
change (`§"Options considered"`→`§*Options considered*`, the section exists
under that name). None alters a security-relevant claim:

- §6 untrusted-manifest sentence ("A manifest is untrusted input in the same
  way that the Linux design already treats it") — classification intact.
- §4 REG_SZ-only refusal — only "covered by" wording changed; fail-closed
  behavior (other value types → `None`, never an error) unchanged.
- §5/§2 canonical-path de-dup ("not de-duped by `host_candidates`") —
  meaning identical; the canonical-path rule still owns collapsing.
- §7 extended-length prefix sentence — "starts with `\\?\`" restates the
  same emitted-path claim.
- Test strategy #8 — "#12 compiles" still points at #12's Windows-target
  `cargo check` of the `#[cfg(windows)]` `registry_string_value`.
- Risks winreg row — cross-reference restyle only; the unmaintained posture
  and its mitigation (target-gated, one private consumer, one-file migration)
  stand.

Checklist walk:
1. Trust boundaries — no change in the diff; registry and manifest sources
   still classified (manifest untrusted, §6). n/a-new.
2. Parsing untrusted data — caps/UTF-8/`REG_SZ`-only gates untouched by the
   diff. n/a-new.
3. Path handling — §5/§6/§7 reworded only; canonicalization and the `\`
   refusal unchanged. n/a-new.
4. Secrets/PII — not touched. n/a.
5. Dependencies — the diff's only dependency mention is the winreg risk row.
   Checked RustSec advisory-db: no advisory entry exists for `winreg`
   (github.com/rustsec/advisory-db has no `crates/winreg` path); no known
   unfixed advisory found for winreg 0.56, `windows-sys`, or `cfg-if`. Pass.
6. Privileges — unchanged (read-only `KEY_READ` registry reads, read-only
   scan). n/a-new.
7. Fail-closed — unchanged (§3/§4 "failure is 'no candidate', never an
   error"). n/a-new.

Revision log: all ten r0/r1 decisions logged; the diff's edits are editorial
pass output, none re-opens a rejected finding, none contradicts
docs/tasks/T-002-steam-windows-discovery.md. Deferred items untouched by the
diff; none contradicts the task text.

## Findings

None. No security finding is introduced, reopened, or made false by this
diff; nothing the seat previously approved was broken (there were no prior
security findings).

## FYI (no severity)

- The diff's US-spelling normalization ("favor", "de-duped") is consistent
  with the document's existing US spellings ("behavior", "normalization") —
  a language-consistency observation for the editorial seat, not a security
  matter.
