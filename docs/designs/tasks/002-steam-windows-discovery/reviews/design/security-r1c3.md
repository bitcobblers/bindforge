# Security review r1 (c3 ratification)

verdict: approve

Scope: the c1→c2 diff of `02-proposal.md` (blobs d5bafff8 → bc928d9c), then the
full current document. Ratification rule applied: report only where the
revisions broke something this seat approved in r1 or made a section this seat
cited false.

## Checklist walk (changed text only)

1. **Trust boundaries** — unchanged data sources. The `RegistryKey` enum grows
   from two variants to three (`CurrentUserValveSteam`,
   `LocalMachineValveSteam32`, `LocalMachineValveSteam64`, §1/§4): same two
   keys, now one (key, view) pair per seam call, views ordered in the pure
   `host_candidates`. No new registry location becomes reachable; *Test
   strategy* #8's "exactly the three variants… a fourth pair cannot be added
   silently" is a containment **improvement**. Pass.
2. **Parsing untrusted data** — unchanged: one `get_raw_value`, `REG_SZ`-only
   (§3/§4), the 1 MiB / valid-UTF-8 read gate (§2), the manifest reject set.
   *Test strategy* #2 still explicitly pins `C:\games` and `\\server\share`
   refusing on the Windows side. The A1→A1/A1b split was checked against the
   code: `crates/discovery/src/keyvalues.rs:186-207` does unescape `\\` and
   `\"`, so A1b's "assumes no spelling — either form arrives usable" is true;
   dropping the backslash-spelling assumption removes an unverified input-form
   claim, which is security-neutral-to-good. Pass.
3. **Path handling** — the canonical-path gate is confirmed in
   `crates/discovery/src/steam_linux.rs:322-334` (canonicalize → require
   adjacent `steamapps/` → dedup) and §5 still routes every appended candidate
   (both HKLM view values, env defaults) through it; `absolute_env` (:338-342)
   still rejects relative env values. §8's new `#[cfg(windows)]`
   `--steam-root → SteamWindowsPlugin::from_roots` branch feeds
   caller-supplied paths from the user's own command line — same-user,
   intentionally supplied input, not a new trust-boundary crossing — and the
   same canonical-path rule gates them. Pass, with the pre-existing minor
   below.
4. **Secrets/PII** — no new emitted fields; rows and JSON shape untouched.
   Pass.
5. **Dependencies** — the diff adds none: still `winreg 0.56` + `cfg-if`,
   reusing locked `windows-sys 0.61.2` (§9, unchanged from the r1-approved
   text). Advisory check found no RustSec/GHSA entry against `winreg`,
   `cfg-if`, or `windows-sys` in the accepted windows. Pass.
6. **Privileges** — `KEY_READ` + fixed WOW64 view flags only (§3); still the
   crate's only `winreg` consumer; read-only scan. Pass.
7. **Fail-closed** — every seam failure maps to "no candidate"; roots that
   fail to canonicalize are dropped, never an error. Pass, with the known
   side-effect caveat below.

## Findings

None reportable under the ratification rule: no revision breaks what this
seat approved, and none makes a cited section false.

Carried, not re-reported as a finding (matches what r1 approved at minor
severity): the r1 observation that a UNC-form candidate root reaches
`std::fs::canonicalize` as first contact (outbound SMB/NTLM attempt and
possible stall before the gate rejects it) is **unchanged by these revisions**
— no revision adopted the drive-absolute syntax check, and no Revision-log
entry addresses or rejects it. The §5 sentence this seat cited was rewritten
("Registry values arrive in mixed case and either slash form…"), but the edit
is a factual re-phrasing that neither fixes nor worsens the gap, and the
`host_candidates`-purity claim (§5) and the `registry_string_value`
failure-mapping claim (§3) remain literally true of the seam itself. Severity
minor, exploitability bounded (a same-user registry writer can coerce SMB auth
directly); ratification stands.

## FYI (no severity)

- §5 now states "Registry values arrive in mixed case and either slash form
  (A3, A4)", but the register cites A3 as "stored lowercase with forward
  slashes" and A4 says nothing about spelling; the §5 generalization is
  broader than its cited basis. Assumption-consistency, not security.
