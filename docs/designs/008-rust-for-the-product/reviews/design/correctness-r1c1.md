# correctness review r1 (dissent pass c1)

verdict: approve

Scope: diff 04a4ae1→6e4ea21 of 02-proposal.md, then full current document. Inputs: 02-proposal.md,
docs/adrs/008-rust-for-the-product.md, the Revision log, my r1 findings. Dissent pass: my r1
findings checked first; new findings only where the changed text makes them true.

## Closure check of my r1 findings

1. [major] License row — CLOSED. `license = "Apache-2.0"    # matches the LICENSE file at the repo
   root`; the package.json-MIT mismatch is now recorded in A23's basis as a repo fix outside the
   design. Matches my r1-verified facts (LICENSE = Apache-2.0, package.json = MIT).
2. [major] Chain-semantics ownership — SUBSTANTIVELY CLOSED. The guard paragraph now says "This
   design decides that input budget, the two error variant names, and the depth bound — constraints
   ADR-009 must adopt" and Deferred is narrowed to "chain semantics other than the depth guard and
   its boundary fixtures". One wording residue remains (finding 2 below, nit).
3. [minor] `resolver` missing — CLOSED (`resolver = "3"` added; valid at the 1.85 floor).
4. [minor] serde entry without a source — CLOSED (`serde = { version = "1", features = ["derive"] }`).
5. [minor] `apps/*` glob vs `src-tauri` — CLOSED (members note now says the desktop era adds the
   explicit member path).
6. [minor] reach-classes fixture layout — layout CLOSED (`fixtures/valid/` + `fixtures/invalid/`),
   but the closure creates a new assertion-scope contradiction (finding 1 below).
7. [minor] two definitions of "supported" / absent version — CLOSED (majors↔directory equality in
   the rule + Test #2; missing-version rejection with a named invalid fixture).
8. [nit] A2 patch precision — CLOSED ("axum 0.8.x ships"; the axum README supports 0.8.x-as-shipped,
   MSRV 1.80, 0.9-in-dev, tower, per my r1 check).
9. [nit] A16 misattributed quote — CLOSED. Re-fetched https://v2.tauri.app/start/prerequisites/:
   "WebView 2 is already installed on Windows 10 (from version 1803 onward) and later versions of
   Windows" and `libwebkit2gtk-4.1-dev` on the Linux list — the narrowed A16 is supported.
10. [nit] sqlx rule paragraph overlap — CLOSED by removal; the mechanism now lives only in Options
    considered + Deferred (ADR-014), and every cross-reference (Gates, matrix, Rollout) points there.
11. [nit] macOS row ambiguity — CLOSED ("macOS | no (v1) | no macOS support committed for v1").

Spot-checks this pass (anti-hallucination gate on changed claims): Tauri prerequisites page (A16);
axum claim per r1-verified README; new body claim "tauri floor no lower than 2.11.1 (current fix for
the IPC-origin advisories)" — verified true: GHSA-7gmj-67g7-phm9 / CVE-2026-42184 (remote-origin IPC
command invocation) and the related `.localhost` origin-confusion fix were both patched in tauri
2.11.1 (release notes, May 6 2026). Register hygiene: A6/A17/A18 removals leave no dangling
references and no orphan rows (A5/A4 still cited by Options; A19 by members table + matrix; etc.).

## Findings

- [minor][new] Formats contract / Test strategy — harness paragraph "Reach-class table fixtures are
  asserted by load + distinctness (Test strategy #4), not re-serialization." vs Test #1 "for every
  directory under `schemas/*/v*/fixtures/valid/`, `bindforge-core` parses the fixture and
  re-serializes to a semantically equal value" — the r1 closure gave `reach-classes/v1` a
  `fixtures/valid/` directory, so Test #1's unqualified glob now sweeps reach-class valid fixtures
  into a re-serialization assertion the harness paragraph explicitly says does not apply to them.
  The two sentences directly contradict for that one directory; an implementer cannot tell whether
  the conformance harness must round-trip reach tables. Fix: scope Test #1 to the document formats
  ("for every `profile`/`patch`/`manifest`/`policy` directory under …") and add "reach-class
  fixtures per #4" to it, mirroring the harness sentence. (Arises from the r1-closure text; closes
  the residual of my r1 reach-classes finding.)
- [nit][re-raised] Goals / Non-goals — "The model's semantics (taxonomy, reach tables, chains,
  policies, manifest contents) — owned by ADR-009" — the guard paragraph and Deferred now carry the
  carve-out ("chain semantics other than the depth guard and its boundary fixtures"), but the
  Non-goals bullet still lists `chains` unqualified, restating the ownership the guard paragraph
  just partially reserved. The specific guard sentence resolves the ambiguity, so the r1 major is
  closed; this is the wording residue of my requested fix, which asked to narrow "Deferred/Non-goals"
  (only Deferred was narrowed). Fix: mirror the Deferred phrasing in the Non-goals bullet ("chains,
  except the depth guard owned by this design").
- [nit][new] Rollout 5 — "the cargo gate commands, the Node floor, the capability-file rule …" —
  the design never states a Node floor anywhere; the only Node statement is "Node version pinned
  explicitly in a `setup-node` step" (Gates). Rollout 5 instructs repo-practices to record a value
  this design has not decided. Fix: either state the floor (e.g. an LTS line matching
  `tools/orchestration`), or rewrite as "the Node version pinned in the `orchestration` CI job".

## FYI (no severity)

- The security-posture additions (`cargo deny check advisories` + root `deny.toml`, action-SHA
  pinning, job-level `permissions: contents: read`, disabled `fs`/`shell`/`dialog` plugins, tauri
  2.11.1 floor) read as the security seat's closures; correctness verified the one load-bearing
  external claim in them (2.11.1 fixes the IPC-origin advisories) is true. A citation
  (GHSA-7gmj-67g7-phm9 or the 2.11.1 release notes) would keep "current fix" durable — the word
  "current" risks staleness as later Tauri security releases land, though the floor is phrased as a
  minimum, so it is not false.
- The `Linux (glibc distros…)` label excludes musl (e.g. Alpine, which is on Tauri's prerequisites
  list); as a v1 scoping decision it is consistent, but the compatibility seat may want it explicit.
