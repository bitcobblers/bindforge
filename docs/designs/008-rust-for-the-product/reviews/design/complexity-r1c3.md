# complexity review r1 (c3 ratification)

verdict: approve

Scope: `git diff` of `02-proposal.md` blobs `6e4ea21` → `796b2a6`, plus the full
current document. Inputs: the proposal, `docs/adrs/008-rust-for-the-product.md`,
the Revision log, and my r1 findings. Checked only whether the c3 revisions broke
what r1 approved or made a cited section false.

## Closure check (my r1 findings vs. the diff)

- **[closed] Finding 1 (cargo deny / deny.toml).** The diff removes
  `cargo deny check advisories` from the Gates block ("the three gate commands
  above, and nothing installed beyond the pinned toolchain (A11)"), removes
  `deny.toml` from Rollout 1, and records the exclusion in Deferred
  ("Dependency-advisory scanning (`cargo deny`, a root `deny.toml`) — no
  follow-up yet"). This is my prescribed drop. Cross-checks:
  - Deferred bullet ↔ Gates paragraph reference each other; both exist. No
    contradiction with ADR-008: clause 4 fixes the `check` gate as
    `cargo clippy` + `cargo fmt`; the ADR never requires advisory scanning, so
    "no follow-up yet" is a deferral consistent with the source document
    (skill step 5 — not re-opened).
  - Test #5 rewritten to "every command listed in the Gates block succeeds …
    with nothing installed beyond the pinned toolchain" — the one-gate promise
    now tracks the gate text and cannot drift; the previously-uncovered fourth
    command no longer exists. Rollout 6's prerequisites (rustup one-liner +
    Linux system-deps) are now complete, since no extra tool is installed.
  - Test #7 ↔ Rollout 7 now reference each other explicitly ("the Windows leg
    of Test strategy #7"); Windows gate verification is scheduled.
  - Revision log's Gates (r1) entry accurately records the removal and its
    stated reasons (unpinned tool outside the rustup toolchain; advisory DB is
    live network state). Reasons are sound.
- **[unchanged, not broken] Findings 2–5** (parse-budget fixture claim; Rollout 5
  "Node floor" + capability-file rule; Formats-contract tree vs. scaffold scope;
  Steam Deck row without a Deferred bullet). None of these passages appears in
  the diff — they are byte-identical to what I approved in r1 — and no revised
  passage makes them false. Per the ratification scope they are not re-reported;
  the Revision log (Formats contract r1, Adversarial guard r1, TypeScript
  boundary r1) records the authors' standing decisions on their themes.
- **[checked, no finding] Postgres matrix cell** — "server version pinned and
  tested by ADR-014's strategy, see Deferred" resolves to the real Deferred
  bullet assigning the hub DB schema and sqlx mechanics to ADR-014 (planned);
  consistent with ADR-008 clause 3 ("Postgres is used only by the hub"). No
  new knob, no dangling pointer.

## Checklist value

1. Reuse: the revision *removed* a tool and a config file — strictly thinner.
2. Goal trace: the remaining gate commands are the three ADR-008 clause 4 fixes.
3. Smaller design: the current slice is at/near that floor now.
4. New dependencies: none added; one CI dependency removed.
5. Deferral: advisory scanning deferred with reason; nothing else deferred that
   this slice needs.
6. Knobs with no consumer: the `deny.toml` knob is gone; no new knob introduced.
7. Value: complexity is proportionate and decreased this round.

## FYI (no severity)

- Supply-chain advisories are now handled in this slice only by the manual
  `tauri ≥ 2.11.1` floor in TypeScript boundary, with scanning deferred to no
  follow-up yet — worth a security seat's eye on when the Deferred item should
  gain an owner (ADR-012/ADR-014 introduce the real third-party graph).
