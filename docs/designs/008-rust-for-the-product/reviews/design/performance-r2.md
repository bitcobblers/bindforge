# performance review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` — six language hunks plus two Revision-log
editor notes. Ratification read: I report only where the revisions broke something the
performance seat approved or made a cited section false. Prior findings from this seat:
none (the editorial phase is one language pass per round). Inputs used: `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`, the Revision log inside the proposal.

## Findings

None — no open blocker or major finding from this seat.

## Checklist coverage against the diff

1. **Input-path latency / per-event cost** — no hunk touches an event path. The scope
   claim that keeps the keystroke path out of this design is untouched by the diff:
   "it never reads or injects input events, so no X11/Wayland/Raw Input/HID API is
   targeted" (Context). Nothing in the diff adds allocation, table rebuild, or
   synchronous IO to a per-event path.
2. **Structure costs, O(n) growth over unbounded n** — the only changed hunk adjacent to
   chain work is the added comma in Formats contract → Adversarial guard: "nested deeper
   than 16 table/array levels, with typed errors". The cost-bearing wording survives
   verbatim: `resolve_chain` "returns typed errors `Cycle` … and `ChainTooDeep` … instead
   of recursing" and "chain longer than `MAX_CHAIN_DEPTH = 32`". Before the comma, "with
   typed errors" could be read as modifying "levels"; the change removes that ambiguity
   and weakens no bound. Test strategy #3's boundary tests against the constant are
   unchanged.
3. **Memory growth (tables, caches bounded?)** — both parse bounds are intact after the
   rewrite: "documents larger than 1 MiB" and "nested deeper than 16 table/array levels".
   The diff introduces no new table, cache, or retention claim, and the constants remain
   published in `schemas/README.md` as the third-party conformance contract.
4. **Startup / gate install cost** — the Gates hunk is wording only: "the three gate
   commands above, with nothing installed beyond the pinned toolchain". The `rust` PR job
   still installs nothing beyond the pinned toolchain, and the `cargo-deny` obligation
   stays off the PR path — "runs weekly and on manual dispatch, never as a PR check" —
   so neither PR latency nor the one-gate promise (Test strategy #5) moves.
5. **Caching and invalidation** — the sqlx offline-cache invalidation mechanism remains
   deferred to ADR-014 ("the sqlx offline-cache mechanism decided in Options considered
   arrives with `apps/hub` under ADR-014"). The Deferred hunk changes only grammar
   ("no interpolation of paths into shell strings") and leaves the deferral list intact;
   it contradicts nothing in the ADR's Decision or Consequences.
6. **Research baseline gap** — no hunk states or alters an expected number, so there is
   no design-versus-baseline gap for this seat to flag. Evaluated against the diff, which
   is this pass's named scope and contains no numeric performance claims.

## FYI (no severity)

- The two lines the diff appends to the Revision log — "Editor notes: … is ambiguous;
  author to clarify" for "nothing → `bindforge-core`" (Workspace shape) and "the Node
  floor" (Rollout 5) — leave open editor questions carried in the artifact. Other seats'
  and the reviser's business, not this seat's.
- The Goals reword ("the hub validating uploads with the same compiled types") now tracks
  ADR-008's Consequences clause "the hub validates uploaded profiles with the same
  compiled types the client uses". That alignment is performance-positive — one compiled
  validator, no second reader to keep in agreement — and adds no new requirement this
  design has to fund.
