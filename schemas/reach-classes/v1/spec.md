# reach-classes format, v1

Reach-class tables classify how far an input can reach in a binding map. The
class *content* — which classes exist and what they mean — is owned by
ADR-012; this spec pins the file layout and the load rules, and the current
class ids in `left.toml` / `right.toml` are placeholders pending ADR-012.

## Table data

The tables live directly in this version directory, one file per hand:
`left.toml` and `right.toml`. A loader reads each file and files the table
it holds under the hand its `hand` entry names, which must match the file
name; the two hands must stay two distinct tables (never one table answering
under both addresses). Reading a table file is subject to the same size budget
as every other document: a file larger than the budget is refused before it is
read into memory.

## Grammar (a documented subset of TOML v1.1.0)

Every valid document in this format is a legal TOML v1.1.0 document; the
grammar is a subset of TOML, so a reader built on a standard TOML parser still
has to reject everything the subset excludes. Line-oriented entries, one per
line: comments (`#`), and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written on one line. Basic
strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`,
`\uXXXX`, and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate
digits; there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers,
dotted keys, multi-line arrays, floats, and hex integers are legal TOML and
outside this grammar, and are rejected.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `reach-classes` |
| `version` | string | yes | the major this document is written against (`v1`) |
| `hand` | string | yes | `left` or `right`; any other value is rejected |
| `classes` | array of strings | yes | reach-class ids, in table order |

Any other entry is rejected; `fixtures/invalid/unknown-entry.toml` is the
corpus case.

## Version gate and size budget

Same rules as every format (`schemas/README.md`): a missing or unsupported
`version` is rejected with an error naming the supported majors; documents
larger than 1 MiB or nested deeper than 16 levels are rejected before
parsing. A rejection message quoting part of the document is a single bounded
line.

## Fixtures

`fixtures/valid/` and `fixtures/invalid/` hold stand-alone table documents:
every implementer must accept the valid ones and reject the invalid ones with
a structured error naming what is wrong, which is what the conformance
harness's corpus walk checks — parse, grammar, and the closed entry set. For
this format the walk also loads every valid corpus table: inside a stand-in
directory, with this directory's own table for the other hand, its classes
must come back filed under the hand its document declares, in table order.

The load rules above are also asserted against this directory's own table
data, `left.toml` and `right.toml`: they load as two tables, each filed under
the hand its file name and `hand` entry agree on. A stand-alone corpus fixture
carries no file name of its own, so the file-name/hand agreement rule is not
part of the fixture assertion; it is part of the loader contract.
