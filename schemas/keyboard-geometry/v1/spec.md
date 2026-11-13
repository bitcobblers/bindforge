# keyboard-geometry format, v1

The key grid the placement predicate checks its shape rule against (ADR-012,
Decision Four). A clause-(a) placement "preserves the cluster's internal
shape", and shape is a fact about where keys sit, not about how far a hand
travels — so the grid is contract data: `schemas/reach-classes/v2/` says how
comfortable a key is for a hand, this format says where that key is.

Keeping the grid in a table is what makes the ADR's stated consequence true —
*adding a keyboard geometry is new table files, not new logic*. Another
geometry is another file in this directory, under the same entry set and the
same rules: no new major, no reader change. v1 ships `ansi-us.toml`, the
reference geometry every surveyed key registry (USB HID, evdev, SDL, W3C
`code`, macOS kVK) names its physical keys from.

## Table data

One file per geometry, sitting directly in this version directory: the file is
`<geometry>.toml` and its `geometry` entry must name the same value — the
file-name agreement rule `reach-classes` already states for a hand, applied to
a geometry name. A document's declared `version` must name the major of the
directory holding it. Reading a geometry file is subject to the same size
budget as every other document: a file larger than the budget is refused
before it is read into memory.

## Grammar (a documented subset of TOML v1.1.0)

Every valid document in this format is a legal TOML v1.1.0 document; the
grammar is the same documented subset `schemas/README.md` states and
`reach-classes/v1/spec.md` spells out. Line-oriented entries, one per line:
comments (`#`), and `key = value` pairs whose key is a TOML bare key
(`[A-Za-z0-9_-]+`) or a quoted basic string, and whose value is a basic string,
a decimal integer, a boolean, or an array of those written on one line. Basic
strings may use the escapes `\b`, `\t`, `\n`, `\f`, `\r`, `\"`, `\\`, `\uXXXX`,
and `\UXXXXXXXX`, and no other. In an integer, `_` may only separate digits;
there is no leading zero and no `0x`/`0o`/`0b` prefix. Table headers, dotted
keys, multi-line arrays, floats, and hex integers are legal TOML and outside
this grammar, and are rejected — which is why the `keys` array is written on one
line, and why the backslash glyph is written `\\`.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `keyboard-geometry` |
| `version` | string | yes | the major this document is written against (`v1`) |
| `geometry` | string | yes | the geometry's name: `[a-z][a-z0-9-]*`, and the stem of the file holding it |
| `keys` | array of strings | yes | each element exactly `"<keyboard-input-id> <col> <row>"` |

Any other entry is rejected, naming the entry: `fixtures/invalid/unknown-entry.toml`
is the corpus case.

Input ids are spelled by the grammar `schemas/reach-classes/v2/spec.md` states
— that section is the canonical home of the id grammar for every format that
carries ids.

## `keys` rules

- each element is exactly **three** space-separated fields: the id, the column,
  and the row. Two fields or four is refused naming the element;
- the id is a grammatically valid `keyboard:*` id. A `mouse:*` or `gamepad:*`
  id has no position on a keyboard, so it is a refusal rather than a key no
  placement can use;
- each id appears once: one key placed at two coordinates would make the shape
  rule answer two different deltas for one action;
- coordinates are `0..=999`, read through checked integer conversion. The
  grammar's integers are 64-bit, so a negative or oversized literal is a
  grammar refusal rather than a coordinate that wraps into a legal one. A
  length bound on the array is the shared parse budget (`schemas/README.md`),
  not a per-format constant.

Column is the position from a row's left; row counts from the digit row, so on
ANSI-US `q` is `0 1`, `a` is `0 2`, `z` is `0 3`, and `space` is `3 4`. The
grid is deliberately sparse: only the keys a format places appear, and a
geometry may place a subset — which is what a stand-in fixture exercises.

## Agreement with the reach tables

The shipped data agrees by construction: the ids `ansi-us.toml` places and the
ids `reach-classes/v2/` grades are both the 48 members of the closed
`keyboard:` set — 26 letters, 10 digits, 11 ANSI-US punctuation keys, and
`space`. That agreement is what keeps the placement predicate's definedness
rule (every id has a coordinate *and* a grade) from ever firing on shipped
data; a move naming an id outside both sets is a structured refusal, not a
verdict. A geometry that places a key the reach tables do not grade is a
vocabulary addition, and it rides the same closed member sets.

## Version gate and size budget

Same rules as every format (`schemas/README.md`): a missing or unsupported
`version` is rejected with an error naming the supported majors, never
defaulted. A rejection message quoting part of the document — an id, a geometry
name — is a single bounded line with the part echoed escaped to printable
characters, and never a local filesystem path. A name handed to a loader is
checked against the `geometry` grammar *before* it is joined onto a directory,
so `..` and a separator are refusals that never ask the filesystem anything.

## Fixtures

`fixtures/valid/` — documents this format accepts; the reference reader also
re-serializes each to a semantically equal document.
`fixtures/invalid/` — documents every implementer must reject with a structured
error naming what is wrong; each carries one fault and is named after it.

The file-name agreement rule is asserted against this directory's own table
data, `ansi-us.toml`, rather than against the fixtures: a stand-alone fixture
carries no file name of its own. The conformance harness's `schemas/*/v*/` walk
picks this directory up with no harness change; the corpus, not the code, is
what makes the format supported.
