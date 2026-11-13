# BindForge formats contract

`schemas/` is the single home of BindForge's on-disk format contract: one
directory per format (`action-ids`, `keyboard-geometry`, `profile`,
`reach-classes`, and later the manifest and profile-chain formats of the
012-015 series), and under it one directory per major version (`v1`, `v2`, …).
Every product component — the CLI, the desktop app, the hub — validates
documents with these rules, and third-party readers can conform to them
without reading Rust.

The scaffold of this contract tree (ADR-008 design 008) ships `profile/v1` and
`reach-classes/v1`; ADR-011 adds `action-ids/v1`, the id set the rest of the
011-015 series speaks through. ADR-012 adds `reach-classes/v2` — the five-class
reach taxonomy with each hand's grade for every graded input, leaving `v1` and
its placeholder ids untouched — and `keyboard-geometry/v1`, the key grid the
placement predicate checks its shape rule against. The manifest and
profile-chain formats arrive with the 012-015 series, and the conformance
harness's `schemas/*/v*/` walk picks them up with no harness change.

## Versioning rule (canonical)

- One directory per major version: `v1`, `v2`, …
- Every document declares its `format` and its `version`. A document that
  declares no version, or a major `bindforge-core` does not support, is
  rejected with a structured error naming the supported majors. It is never
  defaulted to a version.
- The majors a document may declare are those of **its own format**: a format
  ships a major only when a `schemas/<format>/vN/` directory carries it, and a
  document declaring a major its format does not ship is refused naming that
  format's majors. `reach-classes` is the only format with two majors today, so
  its `v2` never widens what a `profile`, `action-ids`, or `keyboard-geometry`
  reader accepts. A reader that gates on one global list instead reads a
  future-major document with the grammar of the major it happens to have —
  the misreading this rule exists to prevent.
- Minor bumps are additive only.
- A file's presence in `vN/fixtures/valid/` is the definition of
  "supported", and the majors `bindforge-core` declares must equal the
  majors present under `schemas/*/v*/`.
- Fixture file names are lowercase ASCII words joined by `-`, and
  case-unique within a fixture directory.

## Formats

Formats are TOML (ADR-011-015) in a documented subset of TOML v1.1.0 (A20): every
valid contract document is a legal TOML v1.1.0 document, and each version
directory's `spec.md` states exactly which TOML constructs its grammar
admits — and which it refuses. A reader built on a standard TOML parser
therefore accepts more than the contract does, and must reject the rest:
table headers, dotted keys, multi-line arrays, floats, and hex integers are
legal TOML and outside the v1 grammar, and each has an invalid fixture. The
subset is stated here, in the contract, because a narrowing that only lives
in one implementation is not a contract.

The fixtures are the conformance currency — following the JSON Schema Test
Suite model, every implementer runs the same golden fixtures — and no
machine-checkable schema language is committed.

One vocabulary spans the formats: an **input id** is `<namespace>:<member>`
(`keyboard:*`, `mouse:*`, `gamepad:*`), and its grammar — including the closed
member sets that make an unknown member a refusal — is stated once, in
`schemas/reach-classes/v2/spec.md`, the first format that carries ids. Every
format that carries an id points there rather than restating it.

## Input budget and chain bound (adversarial guard)

Documents arriving from disk, from another user, or from the network are
untrusted at every entry point, and these limits apply before any document
can reach a writer:

- A document larger than 1 MiB (1048576 bytes) is rejected. The cap binds
  *before* the read, not after it: a reader checks a file's length or a
  request's `Content-Length` against the budget instead of allocating the
  whole body and then refusing it. `bindforge-core` exports
  `MAX_DOCUMENT_BYTES` and offers `parse_file`/`parse_reader`, which apply
  the check themselves; the hub's upload policy (ADR-014) carries the same
  requirement onto the network path.
- Tables or arrays nested deeper than 16 levels are rejected
  (`MAX_NESTING_DEPTH`).
- A chain deriving from itself is rejected (`Cycle`); a chain longer than
  `MAX_CHAIN_DEPTH = 32` links is rejected (`ChainTooDeep`) instead of
  recursed into. 32 is an initial value ADR-014 may tune but not remove.

The three constants are the reference implementation's public names for these
limits, so an implementer never has to guess which number a rejection names.

Rejections are structured errors that say what is wrong, in a single bounded
line: a message quoting part of the document — a declared `format`, a declared
`version`, an entry name — escapes control characters and truncates, so one
hostile document cannot forge extra log lines or inflate log volume, and a
local filesystem path never appears in a message shown to whoever sent the
document. The `fixtures/invalid/` corpora are the contract for which
condition produces which rejection.
