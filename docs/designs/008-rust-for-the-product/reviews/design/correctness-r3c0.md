# correctness review r3c0 (ratification pass)

verdict: approve

## Scope

Diff `67a6a1a..bbece02` of `02-proposal.md` (the r2 revision), plus the full
current document. Inputs per the caller: the proposal, `docs/adrs/008-rust-for-the-product.md`,
the Revision log, and my r3 findings. This pass reports only regressions:
whether the r2 edits broke what was approved in r3 or made a section cited by
an r3 finding false.

## Findings

None. No regression found in the changed text.

Verification per changed hunk:

- **Context** — "client types" → "the same compiled types": now quotes the
  ADR's Consequences clause ("validates uploaded profiles with the same
  compiled types") verbatim, matching the Adversarial guard's parenthetical.
  Consistent, no break.
- **Assumption register — new A28 (Node floor `>=22.23.4`)** — does not
  contradict A23 (the only existing gate is `npm run check` in
  `tools/orchestration`). Its `verified` status is repo-local; see FYI.
- **Gates** — `node-version` pinned to `22.23.4`, "this is the only Node
  floor this design sets": agrees with TypeScript boundary ("any Node floor
  they bring are the desktop design's choice"), the Deferred bullet ("the
  desktop-era Node floor that toolchain may need — owned by the desktop
  design"), Rollout 4 ("the Node floor from Gates") and Rollout 5 (the
  "`engines.node` value cited in Gates"). One value, mirrored, no second
  floor introduced. The `rust`-job parenthetical change ("and nothing
  installed" → "with nothing installed") is meaning-preserving.
- **Workspace shape** — "`bindforge-core` depends on nothing" rewords
  "nothing → `bindforge-core`"; the dependency graph is unchanged.
- **Formats contract / Risks / Deferred** — comma insertion, "silently" →
  "silent", "never interpolating" → "no interpolation of": cosmetic; no
  cited section affected.
- **Revision log** — r2 annotations accurately describe the body changes.

Sections cited by my r3 findings are untouched or semantically unchanged:

- r3 minor 1 (pinned `cargo-deny` install vs the 1.85.0 toolchain-file
  override), minor 2 ("That same run is the trigger for the
  `rust-toolchain.toml` security-advisory bump"), minor 3 (`deny.toml`
  `skip` as advisory-waiver key): all three sentences stand verbatim; the
  findings remain open exactly as stated, neither fixed nor aggravated.
- r3 nit (Test strategy #5 referent vs the Gates section): #5 is unchanged;
  the r2 addition of the `npm ci && npm run check` Node-pin detail sits in
  the same section but does not widen or narrow the ambiguity.
- r3 nit (Revision log Gates entry "unpinned tool outside the rustup
  toolchain"): the parenthetical is unchanged and still contradicts the
  body's "installing `cargo-deny` at a pinned version" — unchanged, not
  re-reported per the ratification scope.
- r3 nit (Tauri advisory-list check has no recurring owner): the TypeScript
  boundary sentence is unchanged; the extended Deferred bullet adds the
  Node floor but not that check — still open, not aggravated.

## FYI (no severity)

- Assumption register — A28 — the basis (`tools/orchestration/package.json`
  `engines.node`, repo listing) supports the *declaration* of a `>=22.23.4`
  floor but not the second clause, "the repo builds and checks under it
  today"; a listing evidences declaration, not build behavior. Not in scope
  for this ratification pass (new text, breaks no approved section), and the
  cited path is not openable from this seat; flagged for the author to make
  sure the basis wording (or an added CI-run basis) carries the full claim.
