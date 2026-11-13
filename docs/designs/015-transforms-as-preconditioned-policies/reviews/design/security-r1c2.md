# security review r1c2

verdict: approve

Scope: the diff `fc0a1f9 → 0dacb96` of `02-proposal.md` (Risks row on
ANSI-US testing, new Compatibility matrix → Keyboard geometry row, amended
Product artifacts row, amended Deferred → ISO/JIS entry, Revision log),
then the full current document. Inputs per caller: the proposal, the ADR,
the Revision log, and my r1 findings.

## Checklist evaluation against the changed text

1. **Trust boundaries** — unchanged posture; the new Product artifacts row
   *strengthens* it: "What does ship — `core` + `crates/transforms` — touches
   no OS input API (X11, Wayland, Windows Raw Input, macOS HID): parsed
   documents in, parsed documents out" is consistent with A16 (transforms
   depends only on core; core depends on nothing) and with the `apply`
   signature taking already-parsed documents. Policy remains the fourth
   untrusted class (Risks row 1, untouched by the diff).
2. **Parsing untrusted data** — the new Keyboard geometry row's claim that
   "`apply` and `load_policy` see only action ids and closed-vocabulary
   physical input ids (A19), never a composed character" matches the model
   (tags are `TagPattern` over `ActionId`, targets are `InputId::parse`,
   ids are slug-checked per A15) and the parse refusals. A homoglyph or
   non-ASCII id fails the slug/tag grammar and lands as a clean parse
   refusal or `unmatched-tags` — fail-closed, no crash, no partial apply.
3. **Path handling** — untouched by the diff; `load_policy`'s
   slug-before-filesystem rule stands.
4. **Secrets/PII, 6. privileges** — the diff adds nothing that logs, stores,
   or transmits; the OS-input-API denial is a least-privilege improvement,
   not a change of privilege.
5. **Dependencies** — no new third-party dependency in the diff
   (`Cargo.lock` gains exactly one package, the workspace crate itself); no
   advisory exposure introduced.
6. **Fail-closed** — the amended Risks mitigation now points at the Keyboard
   geometry row, which does state the claim limit ("no policy may claim
   non-ANSI support until they land"), so the cross-reference the old text
   dangled is now real; a policy authored for a different geometry fails
   its checks against the loaded grid as a report/no-op, never corruption.

## Regression check against my r1 citations

- Finding 1 (minor, chain-wide id collision on the regeneration write): the
  entry refusal, The output regeneration recipe, and Test strategy #8 are
  byte-unchanged in this diff; the revision did not break anything I
  approved there and did not make my cited text false. Not re-reported per
  this pass's rules; it remains an open carryover (see FYI).
- Finding 2 (nit, tag-echo length bound / `offender` for tag-level
  outcomes): The report section is byte-unchanged; still as stated. Not
  re-reported per this pass's rules; open carryover (see FYI).
- Deferred → ISO/JIS entry: reworded to point at the Compatibility matrix
  row; does not contradict any ADR-015 Decision or requirement clause.

## Findings

None new in this diff.

## FYI (no severity)

- Both r1 security findings (chain-wide id collision on the regeneration
  write — minor; tag-echo bound / `offender` shape — nit) are untouched by
  this revision and unrecorded in the Revision log, so they are neither
  closed nor rebutted; they remain open carryovers that block nothing
  (neither is blocker/major).
