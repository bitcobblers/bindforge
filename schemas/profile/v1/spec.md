# profile format, v1

A profile document names one binding profile and the chain of profiles it
derives from. Content beyond what is pinned here — the taxonomy, reach
tables referenced by profiles, and chain semantics other than the depth
bound — is owned by ADR-009.

## Grammar (a documented subset of TOML v1.1.0)

Every valid document in this format is a legal TOML v1.1.0 document; what this
spec fixes is a subset of TOML, so a reader built on a standard TOML parser
still has to reject everything the subset excludes. Line-oriented entries, one
per line: comments (`#`), and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written on one line. Basic
strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`,
`\uXXXX`, and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate
digits; there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers,
dotted keys, multi-line arrays, floats, and hex integers are legal TOML and
outside this grammar: each is rejected, and each has a fixture under
`fixtures/invalid/` pinning it.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `profile` |
| `version` | string | yes | the major this document is written against (`v1`) |
| `id` | string | yes | the profile's identifier; not empty |
| `derives` | array of strings | no | ancestor chain, nearest ancestor first; default empty |

Any other entry is rejected. The entry set is closed on purpose: an entry the
grammar does not define carries meaning no implementer has agreed to, and an
alias of a known entry (`derives_`, `Derives`) would otherwise read as that
entry being absent — quietly changing what the profile derives from.
`fixtures/invalid/unknown-entry.toml` is the corpus case.

## Version gate

A document that declares no `version`, or a major the reader does not
support, is rejected with an error naming the supported majors — never
defaulted to v1. `schemas/README.md` states the rule;
`fixtures/invalid/missing-version.toml` is the corpus case.

## Chain guard

Resolving a profile's chain is iterative and refuses adversarial shapes:

- a chain that derives from itself (its id appears in its own chain, or an
  ancestor repeats) is rejected as `Cycle`;
- a chain longer than `MAX_CHAIN_DEPTH` links (32 initially; ADR-009 may
  tune, not remove, the bound) is rejected as `ChainTooDeep`.

Both boundary cases ship as invalid fixtures so every implementer rejects
them the same way.

## Size budget and rejection messages

Documents larger than 1 MiB, or with arrays nested deeper than 16 levels, are
rejected before any value is built (`schemas/README.md`);
`fixtures/invalid/nesting-past-limit.toml` is the nesting case. The size cap
binds before a file or request body is read into memory, not after. A
rejection message that quotes part of the document — a declared `format`, a
declared `version`, an entry name — is a single bounded line: it may not carry
a decoded newline into a log, and it may not echo an unbounded value.
`fixtures/invalid/forged-log-line.toml` is the corpus case.

## Fixtures

- `fixtures/valid/` — documents this format accepts; an implementer must
  also be able to re-serialize them to a semantically equal document.
- `fixtures/invalid/` — documents every implementer must reject with a
  structured error naming what is wrong.
