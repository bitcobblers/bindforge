# Complexity review r2

verdict: approve

**Scope as given:** the single editorial confirmation pass of round 2, over
`docs/designs/026-registry-service/reviews/design/editorial-r2.diff` (20 hunks),
read against `02-proposal.md`, `docs/adrs/026-registry-service.md`, and the
Revision log. No earlier review reports or transcripts opened. No prior
complexity findings carried into this pass (the editorial phase is one language
pass per round), so nothing here is `re-raised` or `regression`.

## Findings

None. No `blocker`, `major`, `minor` or `nit` finding from the complexity
checklist against this diff.

## What was checked

Every hunk in the diff is a language or line-wrap change; none adds, removes,
renames or re-scopes a moving part. Walked hunk-group by hunk-group:

- **Context / `apps/registry` / modules** — `**root** mode serving`; the
  `std`-only reflow; `corpus.rs` "load + validation + … index"; `(see §Budgets)`.
  Word-class and cross-reference edits. The module list stays 8 files, the
  dependency line (`bindforge-core` + `discovery` + `serde_json`) and the
  six-copied-invariants statement are untouched.
- **Catalog document** — "whose write side … applies to every upsert" → "— on the
  write side, §The admin face and auth applies that cap to every upsert". Same
  single cap (`MAX_CATALOG_DOCUMENT_OCTETS`), same two enforcement sites; no second
  mechanism is implied. Serial-comma edits in §The serving face step 1 do not
  change the path-safety refusal set.
- **Concurrency** — "the state the ADR left unstated" → "what the ADR left
  unstated". The refused two-writer posture keeps its single mechanism (the
  fixed-name `create_new(true)` tmp failure); no startup check or instance marker
  crept in, which is what the Revision log entry (line "r3: persist runs inside
  the one model write lock…") closed.
- **Caching mode** — semicolon fix in the reachability paragraph; the
  "no timer thread" / "a health read never fires the probe" economy statement
  survives intact, so the design keeps its single-thread lazy probe rather than
  regaining a background timer.
- **Budgets** — `MAX_CONCURRENT_CONNECTIONS` glossary re-wrapped from "answers the
  one request §http.rs closes" to "answers exactly one request before `http.rs`
  closes the connection". Same one-thread-per-connection, one-answer-per-connection
  bound, same ≤64 live threads and ≤64 in-flight hops; no new knob, no new
  accounting.
- **CLI amendment** — "deletes" → "drops" the `is_loopback()` refusal: same single
  removal, the fail-closed pre-plugin connect preserved (checked: `catalog.rs:248`
  is the `is_loopback()` refusal the text cites, `LOOPBACK_HOSTS` at `catalog.rs:34`).
- **Test strategy** — #1 "each of the following"; #10 "would **pass**" → "would
  **exceed** `MAX_CATALOG_DOCUMENT_OCTETS`". The old word made the case state the
  opposite of §The admin face and auth ("one over `MAX_CATALOG_DOCUMENT_OCTETS` …
  is the named `422`") and of the Budgets Revision-log entry; the editorial word
  restores agreement instead of breaking it, so no new `422` path or extra
  validation stage is introduced. Remaining hunks (#9, #12, #13, #15, #16) are
  continuation-line indentation only.
- **Rollout** — "the service is tasks" → "lands as tasks"; step 5/6 re-wraps. The
  five-task split (`T-004`…`T-008`) and the divergence-window admission are
  unchanged, and still match the ADR's Rollout items 1–7 and item 7's "no product
  code".

Claims the changed sentences rest on, verified against the code rather than
assumed: core's budget family is **13** `pub const MAX_*` items across
`crates/core/src` (the "thirteen-constant family" phrasing, and the proposal's
decision to keep the registry budgets private to `constants.rs`, both hold);
`serde_json` is a direct dependency of `apps/discovery-cli` (its `Cargo.toml`),
supporting "already a shipped-client dependency"; `Connection: close` is the
shipped client's own framing (`tests/cli_catalog_gate_tests.rs`), supporting the
one-answer-per-connection bound behind the re-wrapped thread-count sentence.

## Checklist

1. *Reuse / existing package or module* — **not applicable**: the diff adds no
   package, module or interface. (Design-level reuse unchanged: the six copied
   invariant blocks with the lift deferred, and the `constants.rs` precedent
   copied from `matches.rs`.)
2. *Every moving part traces to a stated Goal* — **unchanged by the diff**; no
   passage in it introduces a part with no `026-*` or Goal trace.
3. *30–50 % smaller design* — **unchanged**; the diff removes no mechanism and
   adds none, so the smaller-design question is exactly where the Revision log
   already settled it (heavy deferral to `T-004`…`T-008`, `026-R1`/`R2`/`R3`,
   and the shared-crate lift).
4. *New dependencies* — **not applicable**: none named in the diff.
5. *Defer the second half* — **unchanged**; root face, admin face, caching and
   the CLI amendment remain separately taskable in `T-005`/`T-006`/`T-007`/`T-008`.
6. *Knobs with no consumer; abstraction with one implementation* — **unchanged**;
   the only knob touched in prose (`--refresh-interval`) keeps its consumer
   (`026-B3` TTL override, pinned by test #6) and its bare-seconds spelling.
7. *Value check* — the complexity remains **proportionate** to what ADR-026 buys
   (a per-slug serving face plus a user-hosted cache, with the bulk-transfer and
   TLS and prefetch surfaces all refused or reserved), and this pass moves that
   balance by zero.

## Deferred items

Re-checked only against the ADR's Decision: the deferrals this diff touches in
prose (the `T-004`…`T-008` split, `026-R1` transport assurance, the seed-corpus
real-id adoption) are consistent with `026` Rollout items 1–7 and item 7's
"ships the contract … and no product code". No deferral contradicts the source
document, so none is re-opened here.

## FYI (no severity)

- Test-strategy #9: the diff re-indents the closing continuation line "token;
  `405` carries `Allow`." from 4 to 3 spaces while the item's other continuation
  lines stay at 4; the item still renders as one list item (a lazy paragraph
  continuation), so it is a source-alignment inconsistency, not a contract
  change — the language seat's call whether to normalize.
- §Budgets: the `MAX_CONCURRENT_CONNECTIONS` sentence now wraps raggedly across
  "so ≤64 live / threads and ≤64 in-flight / upstream hops"; harmless in
  rendering, and the only residue of an otherwise clean re-flow.
