# correctness review r2 (dissent pass c4)

verdict: approve

Scope: `git diff` of blob 11f23ce → 2168fac of 02-proposal.md, then the full
current document, against docs/adrs/005-runner-executed-file-moves.md and this
seat's r2 findings.

## Prior findings — closure check

- [major][closed] **Context/Options item-5 contradiction vs ADR-005.** The
  revision takes the second branch of my r2 fix: item 5 is out of this change.
  Context now reads "ADR-005 defers its earlier item 5, read-only git in stage
  shells, to a follow-up ADR, so this change ships items 1-4 only" — mirroring
  the ADR's Decision clause "Items 1-4 stand without it". Every dependent piece
  is gone: the Goals bullet, the "Land `moves` without item 5" Option, the whole
  Read-only git section (audit matcher, `git()` env pin, config allowlists,
  `subagent` deny), Risks row 1, A9, #13's ship gate, #4's `stagePathAllowed`
  `.git` assertion, #6's `git-plant-detected` clause, and the git-2.32 floor row.
  Grep confirms the only surviving "item 5" mentions are Context, Deferred, and
  the Revision log, each stating the deferral correctly. Rollout 1's
  `proposed → accepted` flip now flips an ADR whose Decision is exactly what the
  branch ships; the repo record is consistent. The Deferred bullet
  ("ADR-005 defers the earlier item 5 to a follow-up ADR") restates the ADR's
  own Decision clause — consistent with the source document, therefore not
  re-opened per checklist step 5.
- [minor][closed] **`spawnSync('git')` enumeration not exhaustive.** Closed by
  deletion: no paragraph or Risks row now claims a shared env helper behind
  every `spawnSync('git')` site, so the identity.ts/cli.ts sites no longer
  falsify anything. The only git-behavior claim the change now makes is the
  Compatibility row's "the only git call this change adds is the validator's
  `git cat-file -t` (rule 4)", which agrees with the Chosen design (validator
  rule 4 uses `git cat-file -t`; apply is `fs.renameSync`; commit/sweep calls are
  existing) and with the Node row's "plus the existing git helper".
- [nit][closed] **Stale ADR line cites.** Replaced with section-name anchors:
  "(ADR-005, Ordering)" and "(ADR-005, Consequences)". Verified against the
  ADR: Ordering/Dogfood acceptance is the paragraph at ADR:93-94, and the
  "Untouched: review protocol, gates, ownership semantics for non-move edits"
  sentence sits in Consequences. Both cites land.

## New findings from the changed text

None. Specifically checked:

- Validator rule 2's new wording ("no endpoint has a case-folded `.git`
  segment … a validator-local rule, not a `stagePathAllowed` change") is
  internally consistent with rule 7's resolved-path re-run of the test, the
  Compatibility worktree row ("covers the pointer, `.git/**` and any nested
  `.git/`"), the Risks case-insensitivity row's "`.git` endpoint predicate"
  phrasing, and #2's `.git`/`.GIT/hooks/pre-commit`/`src/.git/config`
  validator-rejection pins. Removing the `stagePathAllowed` `.git` assertion
  from #4 is required by the rule-2 change, not a lost pin — `stagePathAllowed`
  at gates.ts:260 is untouched by this change, and #4 still pins the
  sweep-interaction invariant rule 3 actually relies on.
- Options numbering after deleting the last option: the surviving "Options 1"
  (Non-goals) and "Options 5" (Sweep interaction) references still resolve.
- Deferred sandbox bullet: the removal of the review-seats' child-session `.git`
  route is consistent with the `subagent` deny leaving with item 5; the retained
  wording matches the ADR's deferral scope.
- Revision log carries a current entry per changed decision (deferral, rule-2
  locality, failure classes, git floor), including what deliberately did not
  change (rule 7 re-runs the resolved test; #12 macOS contrast stays).
- Assumption register: A9 removed and referenced nowhere; the A9 numbering gap
  preserves the surviving A10–A13 IDs, which run 004 cross-references use — not
  a finding. A6 remains load-bearing (cited by Options 1 and 2) and its URLs
  were verified in an earlier round; the changed text adds no new URL-backed
  claim (the changed rows cite in-repo anchors and already-verified A5/A8).

## FYI (no severity)

- The Deferred item-5 bullet extends "the same deferral" to review/research/
  doc-sync seats — a proposal-scope call the ADR neither grants nor forbids;
  whether the follow-up ADR should cover those seats too is the ADR author's
  question, not a correctness defect here.
