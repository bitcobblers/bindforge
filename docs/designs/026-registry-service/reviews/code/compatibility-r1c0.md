# Code review — compatibility seat, round 1 pass c0

- Design: `docs/designs/026-registry-service/02-proposal.md` (r4), ADR-026
- Scope: `git diff 217c46fda9eb95230c8df62233eca8ef7b6ffb71...HEAD -- ':(exclude)docs/designs'`
- Inputs used: the diff, the proposal, `docs/adrs/026-registry-service.md`, the runner's
  gate record (green at 2026-11-19T21:53:05Z). No review reports or transcripts opened.
- Gate record note: the quoted `test result: ok. 0 passed` line is the doc-test summary
  tail of `cargo test --workspace` (the crates ship no doc-tests); it is not evidence
  that a suite is empty. The suites themselves carry no `#[ignore]` anywhere under
  `apps/registry`, and the whole `cargo test` leg is green, so the Linux legs are
  established; the Windows/macOS legs are, per §Test strategy #19 and the matrix,
  declared not-run-in-CI by design.

## Checklist walk

1. **Platform code vs the Compatibility matrix — pass, one finding.**
   The `<data>` rule is a runtime `DataLayout::{Unix, Windows}` enum with an injected
   env reader (`persist.rs:19-71`), so both branches compile and unit-run on one host
   (`persist.rs:317-330` runs the Windows/`%APPDATA%` branch, drive-prefix included, on
   Linux) — exactly the #13 seam. Chmod machinery is `#[cfg(unix)]` with a
   `#[cfg(not(unix))]` no-op mirroring the shipped `matches.rs` posture (A8). Path
   pieces are `PathBuf::join` (`registry_dir`, `document_path`), so the Windows
   backslash spelling in the matrix row falls out of the platform join. Keys are
   opaque strings never joined onto paths (`keys.rs:6-10`), and the separator/dot-dot
   refusals enforce that at the door (`keys.rs:88-93`). See finding 1 for the one place
   where the Windows branch is contradicted by a hard-coded Unix message.
2. **Node floor — not applicable.** No `tools/orchestration` or JS change in the diff;
   the npm leg of the gate is green and untouched.
3. **Paths — pass, one finding at the URL seam.** File paths are `join`-built; the only
   string-format join is the `Host:` authority in `upstream.rs` (finding 2), which
   breaks for the bracketed IPv6 literals the gate itself admits (`endpoint.rs:44-53`,
   unit-pinned at `endpoint.rs:146`). Case-sensitivity assumptions are flagged and held:
   fixture names stay under the ungoverned case-unique rule
   (`conformance.rs:353-370` + directory-aware `is_fixture_name`,
   `crates/core/tests/common/mod.rs:433-448`), ids/slugs are byte-compared
   (`BTreeMap<String, _>` is byte order).
4. **Guarded platform imports — pass.** All `std::os::unix::fs::*` imports sit inside
   `#[cfg(unix)]` blocks (`persist.rs:133-178`); the `cfg(not(unix))` arms keep the same
   signatures. The only crate-wide platform divergence is that mode-setting pair, which
   the proposal named (A8). Caveat recorded: "Windows no-op path compiled" (#19) cannot
   be established from a Linux-only CI record for either this crate or the shipped
   precedent; the design names that as its evidence posture, so it is a noted gap, not
   a defect.
5. **Locale/layout reality — pass, nit.** `percent_encode` escapes per UTF-8 byte
   (`keys.rs:52-63`), `percent_decode` demands valid UTF-8 after expansion
   (`keys.rs:45`), the `025-B1`/`013-B4` caps ride octets (`keys.rs:73`,
   `corpus.rs:311`), and JSON documents are UTF-8 — non-ASCII keys and display names
   survive structurally, and `echo_safe` keeps non-ASCII out of the stderr face
   (`upstream.rs:283-296`). But no fixture or suite row carries a non-ASCII key/name
   (finding 4 — nit).
6. **Native dependencies — pass.** `Cargo.lock` gains only the `bindforge-registry`
   workspace-member entry (`bindforge-core`, `discovery`, `serde_json`); no new external
   or native dependency. The Windows-only `winreg` edge is taken transitively through the
   already-shipped `discovery` crate exactly as the proposal states (§`apps/registry`),
   not silently.
7. **Tests run where they claim — pass, one finding.** Unix mode asserts are
   `#[cfg(unix)]` (`data_layout.rs:22,63,120,163,231`; `common/mod.rs:1097`), the
   `%APPDATA%` subprocess case is `#[cfg(windows)]`-guarded rather than skipped
   everywhere (`data_layout.rs:199-224`), and everything else runs green on the Linux CI
   legs. Finding 3: the shared caching-subprocess helper relocates the documents through
   `XDG_DATA_HOME` only, while Test strategy #13 (and the harness's own header,
   `common/mod.rs:13-15`) name `APPDATA` as part of the same child-env seam — the
   caching-mode subprocess suites cannot come up on the platform the matrix calls
   supported.

CLI amendment legs (`026-L1`): remote admission is pinned at `parse_endpoint` in the
unit suite only (`catalog.rs` tests), the subprocess gate keeps refused spellings that
all die before resolution, `probe()`'s admission is pinned against the loopback fake,
and the refusal line drops the loopback wording (A12) — no test dials or resolves a
non-loopback host (A15 holds). No compatibility finding there.

## Findings

- **[minor] `apps/registry/src/main.rs:199-208` — the missing-`<data>` startup refusal
  names `XDG_DATA_HOME` on the Windows branch too.**
  `build_with` resolves the layout with `DataLayout::host()` but hard-codes
  "set `XDG_DATA_HOME` to an absolute path" in the refusal; a Windows instance (layout
  `Windows`, `<data>` = absolute `%APPDATA%` per A6 and the matrix's Windows row) that
  finds `APPDATA` absent or relative is told to set a variable the platform rule never
  reads. The unit test at `main.rs:383-393` pins this Unix-only wording unconditionally.
  Fix: branch the message on the resolved layout — name `%APPDATA%` on the Windows
  branch (or name both) — and let the test assert the wording of `DataLayout::host()`.

- **[minor] `apps/registry/src/upstream.rs:72-73,102-105` (with `endpoint.rs:122-131`) —
  the outbound `Host:` header is malformed for an IPv6 upstream literal.**
  `authority_of` strips the brackets from a bracketed host (`endpoint.rs:126`) and
  `fetch` rebuilds it as `format!("{host}:{port}")`, so the explicitly admitted
  `http://[::1]:8080` (normalized and unit-pinned at `endpoint.rs:146`) sends
  `Host: ::1:8080` — not the bracketed form RFC 9110 requires, which a standards-
  conforming front (proxy/vhost — the edge story `026-R1` sketches) can reject or
  misroute; it only works against this registry's own Host-blind reader. Fix: in
  `fetch`, bracket the host when it contains `:` (`if host.contains(':') {
  format!("[{host}]:{port}") } else { format!("{host}:{port}") }`). The shipped client
  (`catalog.rs:442`) carries the same shape — it is out of this diff's scope, but the
  copied seventh path should not re-import the flaw silently.

- **[minor] `apps/registry/tests/common/mod.rs:462-476` — the caching subprocess seam
  sets only `XDG_DATA_HOME`.**
  `Registry::start` scrubs `XDG_DATA_HOME`/`HOME`/`APPDATA` from every child
  (`common/mod.rs:671-677`), and `Registry::caching` — used by `cache_document`,
  `caching_policy`, `upstream_client`, `upstream_endpoint_gate`, `data_layout`, and the
  `cli_end_to_end` caching legs — relocates the instance's documents via `XDG_DATA_HOME`
  alone. On a Windows host the child's `DataLayout::host()` is `Windows`, so it reads
  the scrubbed `APPDATA`, finds no `<data>`, and refuses to start: every caching-mode
  subprocess suite fails on the platform the Compatibility matrix marks `Yes`. The plan
  (#13, quoted verbatim in `data_layout.rs:8-12`) and the harness header
  (`common/mod.rs:13-15`) both name `APPDATA` as part of this same child-env seam; CI
  never sees it because the matrix defers Windows to non-CI. Fix: set
  `XDG_DATA_HOME`, `HOME`, and `APPDATA` all to the sandbox in `Registry::caching`
  (each layout ignores the others' variables).

- **[nit] `schemas/catalog/v1/fixtures/valid/` + `apps/registry/tests/slug_decoding.rs` —
  no locale row pins a non-ASCII key or name.**
  The dotted/percent/space spellings are pinned (`slug_decoding.rs:124-147`,
  `dotted-external-id.json`), but every fixture and suite key/name is pure ASCII;
  survival of a `local/Jörn`-style key (byte-wise encode, octet cap, UTF-8-validated
  decode) is only structurally true, never asserted. Fix: one valid fixture with a
  non-ASCII external id (and a candidate-query row using it) would close it.

## Verdict

Approve. No blocker or major: every compatibility matrix claim that the design says CI
must establish is established by the green legs, the platform guards are explicit and
mirror the shipped precedent, and the four findings are wording-, wire-edge-, and
harness-portability issues on paths the design itself gates behind loopback or defers
to non-CI platforms.
