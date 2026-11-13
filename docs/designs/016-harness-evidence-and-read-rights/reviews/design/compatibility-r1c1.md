# compatibility review r1c1

verdict: approve

Dissent pass on the r1 revision (blob 2160215 → 9b6b5ed). All six r1 findings
from this seat are closed by the changed text; one new minor finding is
introduced by the closure of §3's doc-family rules.

## r1 closure check

- **[major, r1] macOS/Windows `unknown` rows** — CLOSED. Both rows now read
  `not exercised`, name the two host-dependent surfaces (POSIX inspection set;
  spawned `cargo`/`rustfmt`/`npm` legs), and make no cross-host claim. Matches
  the checklist's demand for a declared posture; Test #14 names a platform test
  for the one platform (`yes` rows: Linux only among platforms) the matrix
  claims supported.
- **[minor, r1] separator/case spelling of the own-report identity** — CLOSED.
  A17 is now marked verified with the basis ("path matching normalizes `\` to
  `/` and is case-insensitive on Windows", permissions V2 page read
  2026-11-13); §1's canonical spelling invokes exactly that normalization and
  adds design-directory scoping; Test #3 pins separator- and case-variant
  spellings and the cross-design-directory same-spelled case. The scoping is
  strictly narrower than ADR-016 Decision 1's identity match (never exempts
  more), so no ADR contradiction; the widened wildcard reach over other runs'
  same-named reports is now recorded in Risks and Deferred.
- **[minor, r1] grammar pin for the deny-then-reallow stack** — CLOSED within
  what the repo can pin: A16 names the grammar as the one A17's page documents
  and states the fixture stays pinned to it; the matrix row names the pinned
  `@opencode/client ^2.1.4` as the version anchor; Test #4 states the fixture
  pin. The repo pins no server version (A20), so the client range plus the
  dated-page citation plus the fixture pin is the full nameable anchor.
- **[minor, r1] in-flight-run matrix row understating `docMaxLines`** —
  CLOSED: the row now carries the document-stage failure and the repair, and
  the cap was raised 600 → 900 on A24's measured shipped sizes (largest 733),
  removing the latent case where the largest shipped document exceeded the cap
  outright.
- **[minor, r1] byte-bounded tail over non-UTF-8 / split code points** —
  CLOSED: capture is "decoded once as UTF-8 with replacement ... bounded to the
  last `MAX_GATE_TAIL_CHARS = 2_000` characters"; the constant is renamed
  consistently in §2, Risks, Test #1 and Rollout — no stale `..._BYTES`
  reference remains in the document.
- **[nit, r1] unanchored Node floor / "agent-config YAML text"** — CLOSED: the
  Linux row cites `engines.node` (package.json:30–31), Rollout #2 owns the
  `>=22.23.5` bump as "the only runtime prerequisite added", and the wording
  now says "agent rule text in `.opencode/agents/`". No API above the floor is
  used (spawn pipes, decode, char bounds — all in-floor Node 22).

## Findings

- [minor][new] Chosen design §3 Read-only reach — "then the inspection set
  byte-for-byte as `code/coder.md` ships it — star form throughout" — the
  inherited read-only set was enumerated in this document in r0 (matching ADR
  Decision 3's named commands); the revision defines it only by reference to
  `code/coder.md`. ADR Decision 3 names four read commands (`git status`,
  `git diff`, `git log`, `git show`) besides the file readers, and Test
  strategy #4 simultaneously narrowed its pin from r0's "doc families allow
  `cat`/`wc`/`git status`" to "doc families allow `wc <file>` and `cat <path>`"
  — so neither this document nor its tests pin that the referenced coder.md
  set contains the ADR's git read commands, and drift in coder.md would
  silently shrink the doc families' ADR-mandated reach (the matrix's own
  macOS/Windows rows cite "§3's inspection set" as the host-dependent surface,
  which is now un-enumerable from the document). Fix: one clause (or one
  assumption row with coder.md line refs as A8 does) stating coder.md's
  inspection set is exactly ADR-016 Decision 3's read-only set in star form,
  and extend Test #4's pin to the `git status *`/`git diff *`/`git log *`/`git
  show *` allows — replacing the dropped r0 enumeration, not adding a new
  mechanism.

## FYI (no severity)

- §2 — "decoded once as UTF-8 with replacement" plus a "rolling tail buffer"
  per leg: an incremental decode must handle a multi-byte sequence split across
  chunk boundaries (e.g. `TextDecoder` `{stream: true}` semantics) for Test
  #1's "never a split code point" to hold; implementation detail, correctness
  seat's call.
- §2's provenance header and "cited with its leg command and exit, not
  verbatim" are this design's mitigation for hostile tail content reaching
  prompts and the build note; whether that mitigation is sufficient is the
  security seat's judgement, noted here only because the same passage was
  revised for this seat's finding.
