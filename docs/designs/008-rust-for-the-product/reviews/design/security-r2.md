# security review r2

Scope: ratification pass over `reviews/design/editorial-r2.diff` only, per the
caller. Inputs: the diff, `02-proposal.md` (to confirm the hunks landed),
`docs/adrs/008-rust-for-the-product.md`, the Revision log. Previous security
findings: none (r1 editorial was a language pass).

## Checklist evaluation (diff-touched text)

1. **Trust boundaries** — Hunk at "Adversarial guard": unchanged classification;
   profile/patch/policy/manifest from disk, other user, or network remain
   untrusted at every entry point (CLI, desktop, hub). Intact.
2. **Parsing untrusted data** — 1 MiB size cap, 16-level nesting cap, `Cycle` /
   `ChainTooDeep` typed errors, reject-before-writer, invalid-fixture corpus as
   the contract: all still present after the diff (the hunk adds one comma
   before "with typed errors"). Intact.
3. **Path handling** — unchanged by the diff; the Deferred bullet's rewrite
   ("never interpolating" → "no interpolation of") preserves all three
   obligations (canonicalization + `..`/absolute rejection, no shell-string
   interpolation, redaction when fingerprinting) and stays consistent with the
   ADR (files remain the product; codecs are ADR-012's). Not a re-opened
   deferral: no contradiction with the ADR's Decision.
4. **Secrets/PII** — untouched by the diff; manifest/scanner user- and
   host-identifying fields remain deferred to ADR-009 as before.
5. **Named dependencies** — `cargo-deny` (pin-checked, weekly, non-PR) — no
   unfixed advisory found for cargo-deny itself. `tauri` — the design's floor
   rule was re-touched by the diff; see the one minor below. A27's two advisory
   facts (GHSA-w28w-mhc8-qvjv: affected `>= 2.0.0, <= 2.11.5`, patched 2.11.6;
   crates.io OSV entry GHSA-7gmj-67g7-phm9 fixing at 2.11.1 with no 2.11.6
   entry) verified against Tauri's advisory list and the OSV data — the Revision
   log's r1c3 closure still stands.
6. **Privileges / least privilege** — capability files enumerate only the app's
   own commands; `fs`, `shell`, `dialog` plugins stay disabled; CI jobs keep
   `permissions: contents: read` and SHA-pinned actions. The hunk's reword
   ("and nothing installed" → "with nothing installed") preserves the
   nothing-beyond-the-pinned-toolchain constraint. Intact.
7. **Fail closed** — invalid-fixture rejection contract, majors-equality gate,
   zero-fixture-directory failure: untouched by the diff. Intact.

The summary-line hunk ("client types" → "the same compiled types") tightens the
proposal's echo of the ADR's consequence ("the hub validates uploaded profiles
with the same compiled types the client uses") — an improvement, no regression.

## Findings

- [minor][new] Chosen design → TypeScript boundary — "the desktop design pins
  the newest `tauri` release available when it lands in `[workspace.dependencies]`
  (never below 2.11.1)" — the parenthetical floor contradicts the design's own
  verified A27: GHSA-w28w-mhc8-qvjv is patched only at 2.11.6, so a pin of
  2.11.1 satisfies "(never below 2.11.1)" while leaving the app vulnerable to
  the very advisory A27 exists to track. This wording sits in a diff-touched
  hunk; the reword itself did not introduce it, but it is security-relevant
  and the binding instruction ("newest available when it lands") already makes
  the floor harmless to raise. Fix: change the floor to "(never below 2.11.6)"
  in the same sentence; the surrounding patch-number policy ("no patch number
  carried here", advisory-list check) is unchanged.

## FYI (no severity)

- A newer Tauri v2 advisory, CVE-2026-95626 (high, CSP nonce bypass via
  `data:`/`blob:` in `script-src`), is published in NVD and referenced as
  GHSA-6vxm-x265-58qf but does **not** appear in tauri-apps/tauri's published
  security-advisories list, and has no upstream fix. This does not block this
  design — the design itself takes no tauri dependency (desktop design decides,
  deferred), and the sentence's controls (pin newest, check Tauri's advisory
  list) stand — but whoever lands the desktop design inherits the "capability
  isolation assumes a patched framework" assumption with one known unpatched
  framework advisory; worth carrying into that design's threat model.
- Rollout 5's "the Node floor" (editor note) — if it refers to the floor of
  supported Node versions, note the CI `setup-node` pin is the supply-chain
  control; the prose there is for the editor's ambiguity pass, not a security
  gap.

## Verdict

approve — the editorial diff broke no security control in any section it
touched; the single minor does not gate.
