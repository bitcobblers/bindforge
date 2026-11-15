# Research: ADR-021 — the decision-unit ledger as the ADR system of record

## Summary

ADR-021 makes a product ADR's `Decision` section a typed ledger of numbered
units (`C`/`L`/`B`/`R`/`S`), each a bounded choice with a `→` pointer to the
normative text that carries it, and makes the ledger↔spec join checkable in
both directions by pattern before any inference runs and again at finalize
(source: `docs/adrs/021-decision-unit-ledger.md`). The branch already carries
one worked instance — the ADR-013 ledger plus the handles in
`schemas/manifests/v1/spec.md` — and that instance shows exactly where the
convention is underspecified: handle placement, unit classification, and the
ownership of deferred work are all ambiguous today (source: repo, Findings).
External practice supports the split (chronology in the record, current state
in the contract) and supports typed, machine-parsed convention lines, but the
mainstream ADR tradition amends by successor record rather than by
per-claim `amends` (source: https://github.com/npryce/adr-tools). The nearest
working relative of what 021 wants is Kubernetes KEPs: one living document
whose body states current state, whose `Implementation History` carries
chronology, and whose structured `kep.yaml` fields exist so automation can
verify the record (source:
https://github.com/kubernetes/enhancements/blob/master/keps/NNNN-kep-template/README.md).
The four things the designer must pin are the handle grammar, how the gate
decides which ADRs carry a ledger, where "reserved ADR number" lives, and how
landing the gate interacts with ADR-020's restart boundary.

## Findings

Repo-local, all read on this branch:

- **021's own ledger does not satisfy 021-C1.** The source ADR declares
  ten units (`021-C1`–`C3`, `021-L1`–`L5`, `021-R1`–`R2`) and not one of
  them carries the `→` pointer that `021-C1` requires of every unit; the only
  `→` in the file is the word inside `021-C1`'s own sentence. The gate this
  ADR commissions fails its own source document on day one (source:
  `docs/adrs/021-decision-unit-ledger.md`).
- **The 013 backfill has landed and is the only shipped instance of the
  convention.** `docs/adrs/013-game-manifests.md` carries 24 units across
  Contracts/Laws/Budgets/Reservations, and `schemas/manifests/v1/spec.md`
  carries the handles plus the `Unit handles` note naming the ledger file and
  stating what an unhandled line means — one shipped spelling of 021-C2's
  note to copy (source: `docs/adrs/013-game-manifests.md`,
  `schemas/manifests/v1/spec.md`).
- **The shipped pair is already non-conforming under 021-L2 and 021-L5.**
  Every 013 unit appears as a handle in the spec except `013-R5`, which
  carries neither a handle nor the `unassigned` marker — the exact failure
  mode 021 cites, still open. A gate that enforces "every unit carries at
  least one handle, except a reservation marked `unassigned`" lands red
  against 013 unless `013-R5` is re-marked first (source:
  `docs/adrs/013-game-manifests.md` `013-R5`;
  `schemas/manifests/v1/spec.md`; `docs/adrs/021-decision-unit-ledger.md`
  `021-L2`, `021-L5`).
- **"Backticked unit id at the end of a spec line" has counterexamples in
  the one spec that ships handles.** Fifteen table rows — the entry table
  (spec lines 49–59) and the budgets table (115–118) — end with the handle
  inside the cell, so the line itself ends ` |`, and at least three lines
  carry the handle mid-sentence (spec lines 27, 168, 183). The mechanical
  rule needs to say end-of-line, end-of-cell, or end-of-sentence, because
  today's corpus is not consistent with the literal wording (source:
  `schemas/manifests/v1/spec.md`).
- **Prose cross-references sit outside the join and already disagree with
  the ledger.** The spec states that a preset's contents belong to "the seed
  set's and the codec's (ADR-016, ADR-017)", while `013-L2` and `013-R4`
  assign the seed set to ADR-019 and ADR-016 is the harness-evidence ADR. A
  handle-resolves check cannot see this; only a cross-reference check can
  (source: `schemas/manifests/v1/spec.md` line 14 vs
  `docs/adrs/013-game-manifests.md` `013-L2`, `013-R4`).
- **Nothing marks an ADR as ledger-bearing.** The keys in use across
  `docs/adrs/` are `status`, `date`, `supersedes`, `series` and `depends-on`
  (`deciders` appears only in the template, and the `flow:` key ADR-010 adds
  is used by no ADR yet); only 011–015 carry `series: 011-015`, and 021 —
  which dogfoods the ledger while calling itself outside the product series —
  carries no `series` at all. 021-R1's scope ("the product series is
  backfilled … harness ADRs adopt forward-only") therefore has no
  machine-readable extension (source: frontmatter of every file in
  `docs/adrs/`; `docs/adrs/template.md`).
- **"Reserved ADR number" has no home.** `docs/adrs/` holds 001–016, 020 and
  021; 017, 018 and 019 are cited as future owners by 011, 013-R1–R4, 014,
  015 and by 021's own rollout step, and 021-L5 requires a reservation to
  name "an existing or reserved ADR number", but there is no index file, key,
  or registry that records a reservation (source: directory listing of
  `docs/adrs/`; `docs/adrs/021-decision-unit-ledger.md` `021-L5`).
- **021-L3 is green today and bites only going forward.** `depends-on` is
  `[]` for 011 and `[011]`, `[011]`, `[011,013]`, `[011,012,014]` for 012,
  013, 014 and 015 — every entry smaller-numbered; no harness ADR carries the
  key even where prose states ordering — 005 says "run after 004 completes"
  and 006 was carved out of 003 — so the gate is a rule for future ADRs plus a prompt to
  add the key to old ones (source: frontmatter of `docs/adrs/011`–`015`;
  `docs/adrs/005-runner-executed-file-moves.md`;
  `docs/adrs/006-self-update-checkpoint-compatibility.md`).
- **Both of 021-R2's hooks already exist.** `sourceAdmissible()` is the
  deterministic source gate that runs before any inference (frontmatter
  parse, `status: proposed`, non-empty `Context`/`Decision`), it is called on
  every entry path from `runPipeline()`, and the same module already exposes
  the ADR-status flip that closes a run — the ledger parse and the
  dangling-handle/unregistered-unit checks have an established home rather
  than a new one (source: `tools/orchestration/src/pipeline/gates.ts`
  `parseSource`/`sourceAdmissible`; `tools/orchestration/src/pipeline/runner.ts`).
- **Only a coder stage can touch a spec.** `stagePathAllowed()` confines
  research, proposal, review and doc-sync stages to `docs/**`; `implement`
  and `fix` own everything outside it. Adding handles to
  `schemas/*/spec.md` — the other half of every backfill, per the 013
  precedent — is therefore a coder-stage or operator action, while
  `021-R1`'s owner is `unassigned` (source:
  `tools/orchestration/src/pipeline/gates.ts` `stagePathAllowed`;
  `docs/adrs/021-decision-unit-ledger.md` `021-R1`).
- **Landing the gate is a harness self-change.** `EXECUTABLE_SURFACE_PREFIXES`
  covers `tools/orchestration/src/` and `tools/orchestration/prompts/`, and a
  stage commit that touches either opens an ADR-020 restart boundary, so a
  run that implements 021-R2 pauses and is resumed in a fresh process (source:
  `tools/orchestration/src/pipeline/gates.ts`
  `EXECUTABLE_SURFACE_PREFIXES`; `docs/adrs/020-restart-boundary-for-harness-self-changes.md`).
- **The ADR's rollout is one run's worth of four separate contracts.**
  Template adoption, four product backfills (011, 012, 014, 015), the gate,
  and adoption by 017–019, with the order and the gate's error taxonomy both
  explicitly deferred; the frontmatter carries no `flow:` key, so the run
  enters the full flow (source: `docs/adrs/021-decision-unit-ledger.md`
  Rollout and "Not decided here"; `docs/adrs/template.md`).

## Prior art

Existing implementations and formats, and what each contributes or refuses:

- **The ADR tradition amends by successor record, not by editing.** `adr
  new -s 9 <title>` creates a record flagged as superseding ADR 9 and flips
  ADR 9's status; MADR's status field is spelled `superseded by ADR-0123`,
  and MADR's optional `Confirmation` section is where "how compliance
  can/will be confirmed" belongs (source: https://github.com/npryce/adr-tools
  ; https://adr.github.io/madr/). 021-L4's per-claim `amends` inside a live
  ledger is a deliberate departure: no shipped ADR uses it, and no shipped
  frontmatter key carries it, so the run defines the notation from scratch.
- **MADR chose `NNNN-title.md`, one namespace, no categories, and lints ADRs
  with markdownlint in CI**; its own ADR on categories weighed category-scoped
  folders and rejected them because "two indices have to be maintained" and
  no unique number identifies a record, while global numbering was kept for
  uniqueness. That is the same trade 021-C1 makes when it puts the kind
  letter inside a globally unique id instead of in a section or folder
  (source: https://adr.github.io/madr/ ;
  https://adr.github.io/madr/decisions/0010-support-categories.html).
- **Kubernetes KEPs are the closest working hybrid of record and contract.**
  One KEP per feature for its whole life — "If new details emerge that belong
  in the KEP, edit the KEP. Once a feature has become 'implemented', major
  changes should get new KEPs" — an `Implementation History` section holding
  chronology while the body states current state, `<<[UNRESOLVED …]>>`
  markers for undecided passages, a `Release Signoff Checklist` whose items
  gate a release, and structured `kep.yaml` answers used "to enable automation
  to verify the presence of the review" (source:
  https://github.com/kubernetes/enhancements/blob/master/keps/NNNN-kep-template/README.md).
  The number itself is allocated by an external tracking issue ("`NNNN` is
  the issue number"), i.e. reservation lives in a registry, not in a
  directory listing — the model for 021-L5 if BindForge wants reservations
  to be checkable (same source).
- **Owner-per-scope registries enforce review, and their parse failures are
  silent.** GitHub CODEOWNERS maps gitignore-style patterns to owners with
  last-match-wins precedence, *skips* any line with invalid syntax, requires
  owners to hold write access, needs branch protection or rulesets as a
  separate switch to become blocking, and advises defining an owner for the
  CODEOWNERS file itself (source:
  https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners).
  021-L5 (a reservation names an owner) and 021-R2 (a reservation with
  neither owner nor `unassigned` is a gate failure) are stricter — they make
  the absence an error rather than a skipped line — and inherit the meta
  question: who owns the ledger convention, given `021-R1` is `unassigned`?
- **Typed, machine-parsed lines work, and their failure mode is invisibility,
  not an error.** Conventional Commits requires a `type(scope)!: description`
  prefix and recommends `docs:`, `test:`, `chore:` types beyond `feat`/`fix`
  — the same convention BindForge already mandates for commits — and its FAQ
  is explicit that a message not meeting the spec "will be missed by tools
  that are based on the spec" (source:
  https://www.conventionalcommits.org/en/v1.0.0/ ; `AGENTS.md`). A grep-only
  coverage check has that property: a misplaced handle is not flagged, it just
  stops existing to the tool.
- **Marking normative force by syntax, plus a declaration note, is a
  standing precedent for 021-C2.** RFC 8174 narrows RFC 2119 so the key words
  "have the meanings specified herein only when they are in all capitals",
  states that non-capitalised uses carry their ordinary English meaning, and
  asks every document that relies on the convention to say so in a boilerplate
  line near the beginning — precisely the backticked-handle rule plus the
  spec's `Unit handles` note (source:
  https://www.rfc-editor.org/rfc/rfc8174.txt).
- **Stable-identifier doctrine supports handles over heading anchors.** The
  case for a never-changing identifier is to leave out everything that can
  change — status, subject, mechanism, ownership — because that is the
  information that forces renames; BindForge's own ADR-011 id philosophy
  ("ids already in the wild stay stable") is the same rule applied to action
  ids (source: https://www.w3.org/Provider/Style/URI ;
  `docs/adrs/011-action-taxonomy-and-core-skeleton.md`).
- **Standards note:** the registry half of this lens (evdev codes, HID usage
  tables, SDL mappings, USB-IF pages) has no bearing on a documentation
  convention; the applicable standards here are the ones above — RFC 2119 /
  8174 for normative-by-syntax, CommonMark for what a markdown parser will
  and will not see as text (source: https://spec.commonmark.org/0.31/), and
  Conventional Commits for a regex-parsable convention.

## Platform notes

- **The grep form needs anchoring before it is a law.** `[0-9]{3}-[CLBRS][0-9]{1,2}`
  as written is unanchored and its quantifier is greedy, so it matches inside
  longer tokens (the tail of a four-digit run, the first two digits of a
  three-digit sequence); pinning it means a word boundary or a look-behind,
  both of which the harness's own engine offers as assertions (source:
  `docs/adrs/021-decision-unit-ledger.md` `021-C1`;
  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions).
  Whether any part of the check is ever re-implemented on the Rust side, and
  what that engine supports, is a separate question the design should answer
  by staying in one engine.
- **A raw-line scan is not a markdown parse.** Fenced code blocks and code
  spans are literal content to a CommonMark parser, so a spec example that
  quotes a handled line yields a handle the join must decide about; the
  shipped `Unit handles` note itself contains an example handle, `013-C4`
  (source: https://spec.commonmark.org/0.31/ ;
  `schemas/manifests/v1/spec.md`).
- **Line endings are already a live concern.** The source parser matches
  frontmatter with `/^---\r?\n…/` and section bodies with explicit `\r?\n`,
  so any ledger parser inherits CRLF tolerance rather than assuming LF
  (source: `tools/orchestration/src/pipeline/gates.ts` `parseSource`).
- **The parser belongs to the TypeScript harness, not the Rust product.**
  `tools/orchestration/src/pipeline/` is Node ESM (`node:fs`, `.js` import
  specifiers) and is where every deterministic gate lives, while `schemas/`
  stays language-neutral and is checked by the Rust conformance harness
  walking `schemas/*/v*/` (source: `tools/orchestration/src/pipeline/gates.ts`;
  `schemas/README.md`).
- **Spec growth is now a measured, capped quantity.** ADR-016 made document
  size a configured ceiling with a gate rather than a style preference, so a
  handle on every normative spec line is a cost the flow's own size meter
  will see (source: `tools/orchestration/src/pipeline/gates.ts`
  `documentOutgrewFlow`; `docs/adrs/016-harness-evidence-and-read-rights.md`
  Decision 4).
- **Research-stage read rights shaped this document.** ADR-016 replaced an
  open corpus with a named read set and a ceiling for the research stage,
  which is why the prior-art sweep here stayed outside `docs/designs/`
  (source: `docs/adrs/016-harness-evidence-and-read-rights.md` Decision 4).

## Risks & unknowns

- **Silent under-coverage.** A convention checked "by pattern, not by
  reading" inherits the skipped-line behavior documented for CODEOWNERS and
  Conventional Commits; unless the gate distinguishes a malformed handle from
  an absent one, coverage can be green while claims are unaccounted for —
  which is the problem 021 says it exists to close (source:
  https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners
  ; https://www.conventionalcommits.org/en/v1.0.0/).
- **Self-application is unsolved today.** 021-C3 dogfoods the convention, but
  021's units carry no pointers, its R1 has no owner, and 013-R5 has no
  `unassigned` marker, so any ordering of gate-vs-backfill either lands red
  or lands with no enforcement (source: `docs/adrs/021-decision-unit-ledger.md`;
  `docs/adrs/013-game-manifests.md`).
- **Pointer targets are names, not references.** `→ spec: Grammar, Version
  gate, Fixtures` names sections without a file, and the units of 011, 012,
  014 and 015 would span several specs (`action-ids`, `reach-classes`,
  `keyboard-geometry`, `profile`, `policy`) plus `schemas/README.md`; 021-L2
  checks handle resolution but says nothing about resolving a `→` target, so
  a pointer to a section that never exists is invisible (source:
  `docs/adrs/013-game-manifests.md`; `schemas/README.md`;
  `docs/adrs/021-decision-unit-ledger.md` `021-L2`).
- **The backfill is four editorial diffs against prose that is already
  drifted.** The 013 backfill found one contradiction, one wording
  tightening, seven units recovered only from the spec, and cross-attributed
  spec claims that stay uncovered until every series member is backfilled;
  expect the same class in 011, 012, 014 and 015, each needing a `Backfill
  findings` section (source: `docs/adrs/013-game-manifests.md` Backfill
  findings 1–5).
- **Two files must now agree, and a harness run cannot edit both.** The join
  spans `docs/adrs/**` (document stages) and `schemas/**` (coder stages), so
  a single run cannot make a ledger and its spec consistent on the document
  side alone (source: `tools/orchestration/src/pipeline/gates.ts`
  `stagePathAllowed`).
- **Amendment machinery is unexercised.** No shipped unit carries `amends` or
  `amended-by`, and the ADR tradition's tooling assumes the opposite posture,
  so 021-L4 is defined but has no fixture, no example, and no frontmatter key
  to hang it on (source: https://github.com/npryce/adr-tools ;
  `docs/adrs/021-decision-unit-ledger.md` `021-L4`).

## Open questions

For the designer:

1. Is 021 itself a conforming ledger at merge, and if so what are the ten
   `→` targets — this ADR's own section headings, the template, and which
   rollout artifact names? If not, does the gate exempt the ADR that creates
   it, and how is that exemption recorded so it is not a standing hole?
2. What exactly is a handle, mechanically: end of line, end of table cell, or
   end of sentence — and does the rule tolerate the placements already
   shipped in `schemas/manifests/v1/spec.md`, or does the run rewrite them?
3. Is the check a real markdown parse (skipping fenced blocks and the
   convention's own examples) or a line scan, and how are illustrative
   handles like the `Unit handles` note's `013-C4` excluded?
4. How does the gate decide an ADR is ledger-bearing — a new frontmatter key,
   the existing `series:` key, the presence of kind sections, or a number
   range — and what happens to the 017–019 ADRs that 021 says adopt at
   landing but that do not exist?
5. Where does a reservation get reserved? Without an index file, is
   "reserved ADR number" a key in the source ADR, a `docs/adrs/` file, or a
   rule like "a number is reserved once some unit names it as owner" — and
   does that rule let a typo reserve a number?
6. Does the gate validate `→` targets as well as handles, and if so are
   targets file-scoped (`schemas/manifests/v1/spec.md#budgets`) or prose
   (section names), given that one unit can govern several specs?
7. What is the join's scope during rollout: does an unbackfilled product ADR
   fail the gate, warn, or read as out-of-scope until its backfill lands, and
   is 013-R5 re-marked `unassigned` as part of this run or left as the gate's
   first red?
8. Is prose-level cross-referencing (`ADR-016` where the ledger says
   `ADR-019`) in scope for the gate, or explicitly out — and if out, is that
   gap recorded in the ADR, since it is the same class of drift the backfill
   found?
9. What is the storage choice for the machine half: markdown-parsed unit
   lines (current ADR shape), a frontmatter/YAML sidecar per ADR as KEPs do,
   or a derived index generated from the ADRs — and which one does the
   finalize assert read?
10. "Finalize asserts the run's own units all resolve in the merged tree" —
    at finalize there is no merge; does the assert run against the run's
    worktree, against a simulated merge, or does the ADR's wording need to
    change, and what does it do about the ADR-020 restart boundary the gate's
    own implementation crosses?
11. Who owns `021-R1` (and the template change), given that an unassigned
    owner is the failure mode 021-L5 exists to prevent?
