# complexity review r3 (c0 ratification)

verdict: approve

Scope: the blob diff `67a6a1a3..bbece027` of `02-proposal.md` plus the full current
document, checked only for regressions against what r3 approved. Inputs: the
proposal (including its Revision log), `docs/adrs/008-rust-for-the-product.md`,
and this seat's r3 findings. Per the caller, no review reports or transcripts
were opened, and no path beyond those named was inspected (A28's claim about
`tools/orchestration/package.json` `engines.node` is taken as given).

## Checklist pass over the changed text

1. Reuse: the new Node pin is taken verbatim from `tools/orchestration`'s
   existing `engines.node` entry (A28) — the r2 revision reuses an in-repo
   declaration instead of inventing a version, which is strictly better than
   the r3-approved "explicit Node version" placeholder.
2. Trace to Goals: the new sentence "this is the only Node floor this design
   sets" plus the Deferred line ("the desktop-era Node floor that toolchain
   may need — owned by the desktop design") closes the floor's ownership
   cleanly; the desktop-era deferral is owned on the Deferred list, so it is
   not the unowned-obligation shape r1c3 closed. Consistent with A15 and with
   the Non-goal "Any change to `tools/orchestration`" (the design references
   the engines value, it does not touch the tool).
3. Smaller design: the diff removes a constraint (unconditional Node
   prerequisite → conditional per A15) and fixes a wording ambiguity
   ("the same compiled types", matching ADR-008's Consequences verbatim).
   Nothing added that a 30–50%-smaller design couldn't carry.
4. New dependencies: none.
5. First merged slice: Rollout 4/5 wording now points at the single Gates
   value instead of restating it — the slice got thinner on duplication.
6. Knobs with no consumer: the `node-version` pin has a consumer (the
   orchestration job) and the value has exactly one home (Gates; Rollout 4/5
   and `repo-practices` reference it). No orphan knob introduced.
7. Value check: proportionate. Every r2 change is either a wording tightening
   or the pin-to-existing-value; no new mechanism was introduced by this
   revision.

## Diff-by-diff verification against r3-approved text

- Goals "with the same compiled types" — reword toward ADR-008's exact
  Consequences clause; the cited section is not made false.
- Workspace shape "bindforge-core depends on nothing" — same meaning as the
  approved "nothing → bindforge-core"; no effect on any cited passage.
- Gates — the only substantive change is the Node-floor sentence; the
  advisories.yml/deny.toml obligation and the "That same run is the trigger
  for the `rust-toolchain.toml` security-advisory bump" clause are byte-for-byte
  unchanged, so the revision neither broke them nor made my r3 critique of
  them newly true or newly false.
- TypeScript boundary — the tauri pin sentence (cited by r3 findings 3 and 4)
  is unchanged; the revised Node sentence is consistent with A15 and defers
  the desktop floor to an owned Deferred line.
- Adversarial guard, Risks, Deferred, Revision log — grammar/ownership edits
  only; no mechanism moved.

## Previously open, not re-reported (ratification rule)

The four r3 findings (toolchain-bump trigger clause; advisory-obligation trace
+ Rollout-4 timing; Tauri advisory-list check unowned at pin time; "no patch
number" vs "(never below 2.11.1)") were not addressed by this revision — the
passages they cite are unchanged — and the revision log records no rebuttal of
them. They remain open for the next content round but are not regressions, so
under the c0 rule they are not re-reported with severity.

## FYI (no severity)

- A28 marks `engines.node: ">=22.23.4"` as verified from
  `tools/orchestration/package.json`; that file is outside this pass's named
  inputs, so the correctness seat may want to confirm the file carries exactly
  that value.
