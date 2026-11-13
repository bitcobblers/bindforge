# security review r1

verdict: request-changes

Scope: whole document, `docs/designs/011-action-taxonomy-and-core-skeleton/02-proposal.md`
(r1, pass c0). Inputs per the caller: the proposal, its Revision log, and
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`. `01-research.md` was not in
this pass's input set, so claims the proposal sources to it (A15, A17) were read
only through the ADR. Previous security findings: none — this is the first
security seat. Revision log carries only `r0: no reviews yet`, so every finding
below is `new` and none collides with a rejected decision.

## Checklist walk

1. **Trust boundaries** — not satisfied. No data source in the document is
   classified, and the one load-bearing case (who owns the id set the code
   answers from) is left implicit. → finding 1.
2. **Parsing untrusted data** — largely satisfied: the loader claims "the size
   budget binds before the registry is read", the format is TOML-only (no code
   evaluation, no compressed layer), the entry table is closed, and each fault
   has its own invalid fixture. The array-length bound is implicit only. → nit 4.
3. **Path handling** — not applicable as a user-input surface in this stage: no
   user- or network-supplied path is consumed; `dir` is caller-chosen in-repo.
   The open question of how an installed artifact locates `schemas/` is a
   deferral, and it contradicts nothing in ADR-011's Decision One ("`schemas/` —
   the language-neutral, versioned contract directory"), so it is not re-opened
   here. Finding 1 touches the *authority* of that directory, not its lookup.
4. **Secrets and PII** — satisfied: the registry holds id strings only; the
   design keeps refusals "path-free"; nothing in this stage leaves the machine.
   One output-hygiene gap. → nit 5.
5. **Dependencies** — nothing to grade: "std-only, no new dependency" and
   "`Cargo.lock` byte-unchanged" (Goals, Compatibility matrix, Rollout 2). No new
   advisory surface was introduced, so no CVE check was required. The RFC/
   toml-test/semver/evdev/SDL links are citations, not dependencies.
6. **Privileges** — satisfied: least privilege is read-only filesystem access to
   repo-shipped contract data; no device access, no network, no binary shipped.
7. **Fail closed** — strong overall (closed entry set, `InvalidValue`/
   `MissingEntry` reuse, per-fault fixtures, `axis_kind` → `None` for anything
   unregistered, loader returns `Result` so no partial registry escapes). One
   version-agreement hole. → finding 2.

## Findings

- [major][new] **The model: `crates/core/src/action.rs`** (with **Stability**) —
  "`load_action_registry(dir: &Path) -> Result<ActionRegistry, CoreError>`" is
  the only registry entry point, and its `dir` is caller-chosen, while this same
  section adds "an `action-ids` arm in `parse`" and an entry in
  `SUPPORTED_FORMATS`. Nowhere does the document say which `action-ids` document
  is *authoritative*, and the proposal itself states that the shared parse path
  is the one carrying "the guard ADR-008's design decided for the hub's upload
  path". Consequence: after this stage, a document arriving over the untrusted
  path can be accepted as a `Document::ActionIds`, and nothing distinguishes it
  from the shipped registry, so `contains()` and `axis_kind()` — the attribute
  the ADR makes load-bearing because "Only `discrete-lateral` actions may have
  polarity flipped by a transform (015)" — could be answered from ids the
  counterparty chose, re-grading arbitrary actions into the flip-eligible set and
  breaking the "ids in the wild cannot churn" promise the Stability section is
  making. "Nothing in this stage acts on the attribute" (A17) bounds the damage
  today but does not record the boundary, so 015 inherits it as an accident.
  Fix: one clause, in `spec.md`'s Extension and withdrawal rule and repeated on
  `load_action_registry`'s doc: the registry bindforge-core answers from is the
  versioned directory shipped with the crate (the app-pinned path), and an
  `action-ids` document that arrives from elsewhere is parsed as data only — it
  is never passed to `load_action_registry` and never consulted for `axis_kind`.
  This classifies the source rather than adding a mechanism; the "never passed
  to `load_action_registry`" clause is the behavior it replaces.
- [minor][new] **Test strategy 4** (with **Chosen design**, `load_action_registry`)
  — "an `action-ids` document declaring no version, or `v99`, is refused with the
  error naming the supported majors" covers unsupported majors, but the loader is
  specified as "`parse_file(dir.join("registry.toml"))` and the variant check",
  which never compares the document's declared `version` against the `v1`
  segment of the directory the harness discovers it in (`conformance.rs`'s
  `schemas/*/v*/` walk takes the format and major from the path). A document
  declaring a *later supported* major sitting in `action-ids/v1/` therefore
  parses as a v1 document under v1 rules, so the contract and the corpus drift
  without a refusal. Fix: add to the spec's entry table (and as a case in Test
  strategy 4) that a declared `version` not equal to the version segment of the
  directory being loaded is an `InvalidValue` refusal — one rule, no new
  mechanism, replacing the current path-vs-document silence.
- [nit][new] **Placeholders and lineage** — "one paragraph saying what lands
  there and which ADR populates it (013 and 014), that nothing reads them yet"
  describes a README for the directory that will hold exactly the artifact the
  security posture treats as untrusted: a shared binding file from elsewhere. The
  spec for that paragraph never says so. Fix: add one sentence to the README
  content spec — anything dropped into `data/profiles/` or `data/manifests/` is a
  file from elsewhere and is untrusted input once 013/014 read it — which is the
  classification, not a restatement of the stability rule the README already
  declines to repeat.
- [nit][new] **Chosen design, `spec.md` entry table** — "`discrete-lateral` /
  `continuous-lateral` / `none` (arrays of strings, all required, may be empty)"
  bounds element *type* but not array length, so for an `action-ids` document
  that reaches the parser from outside the repo the only limit on id count is the
  parse budget; uniqueness ("an id appears once across the three arrays") and
  document-order construction then cost whatever that budget allows. Fix: state
  the bound in the entry table (an explicit max id count refused with
  `InvalidValue`), or state plainly that the existing parse budget is the bound
  for this format as well — either replaces the implicit-only limit.
- [nit][new] **The model: `crates/core/src/action.rs`** — "every rejection keeps
  the single-line, truncating, path-free message the contract already promises"
  while the spec requires refusals that name the offender: "a repeat is a
  rejection naming the id and the array". For a *malformed* id the offending
  bytes are by definition outside the grammar charset and may contain terminal
  control sequences; single-line bounds newlines, not escapes. Fix: one clause
  on the error rule saying an offending id is echoed escaped to printable
  characters, so the naming requirement cannot inject into a log or terminal
  line.

## FYI (no severity)

- The 40-id enumeration and the per-namespace counts (`move` 8, `aim` 3,
  `combat` 8, `interact` 3, `item` 8, `ui` 6, `system` 4) reconcile exactly with
  `registry.toml`, and the pinned `discrete-lateral` foursome matches Test
  strategy 2 — worth a correctness seat confirming, not a security concern.
- `document.rs` gaining a `serialize` arm so "the corpus round-trips like
  `profile` does" (whereas `reach-classes` is load-asserted) means an untrusted
  `action-ids` document can be re-emitted by this crate; relevant to
  compatibility/correctness readers of finding 1.
