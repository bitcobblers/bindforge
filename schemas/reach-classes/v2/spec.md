# reach-classes format, v2

The reach taxonomy and the key→class mapping ADR-012 owes. A v2 table says,
for one hand, which of five classes grades each graded input — so "can this
hand reach this key" becomes data a reader can check, and ADR-015's transforms
can precondition on it.

Everything `v1` pins stays pinned: the file layout, the file-name/hand
agreement rule, the grammar subset, the size budget, and the per-hand rule that
the two hands are two distinct tables. What `v2` adds is `keys`. This is the
new-major route ADR-012 chose precisely because `v1`'s entry set is closed:
`v1`'s class ids stay placeholders forever,
`reach-classes/v1/fixtures/invalid/keys-entry.toml` pins that a `v1` document
carrying `keys` is refused naming the entry, and the version gate refuses a v2
document for a v1-only reader by naming the supported majors.

## Input ids

This spec is the canonical home of the input-id grammar: v2 is the first
format that carries input ids, so the grammar is stated where its first corpus
validates it, and a third-party reader that never loads a reach table still
reads it here (`schemas/README.md` points at this section).

An id is `<namespace>:<member>`: lowercase ASCII, no spaces, exactly one colon.
The member sets are **closed** — an id outside a set is a refusal, not an
unknown member — because accepting new members widens what every validator
accepts and therefore rides a new vocabulary major. The id *shape* is what
reserves the deferred physical-position naming. The set is ASCII, not
Unicode-lowercase: a reader that case-folds or normalizes turns two different
physical inputs into one name, so every member is spelled with the ASCII bytes
below and nothing else.

- `keyboard:` — exactly one lowercase ASCII glyph of the ANSI-US set `{a-z}`,
  `{0-9}`,
  `` ` `` `-` `=` `[` `]` `\` `;` `'` `,` `.`, or the reserved word `space`
  (48 members). A glyph means the **US legend at that position under ANSI
  geometry**: every mature key registry (USB HID, evdev, SDL, W3C `code`,
  macOS kVK) names physical keys from that reference geometry, and stating the
  geometry here is what keeps the assumption from being silently inferred. A
  spelled word is not a member: `keyboard:semicolon` is refused, `keyboard:;`
  is the id, and a letter outside ASCII is not a member either: `keyboard:é` is
  refused naming the member, never folded onto `keyboard:e`
  (`fixtures/invalid/non-ascii-member.toml` is the corpus case).
- `mouse:` — `left`, `middle`, `right`, `side-1`, `side-2`, `wheel-up`,
  `wheel-down`. The wheel press is `middle` because Pointer Events button 1
  *is* that press. The wheel is an event, not a button, so its two directions
  are the members.
- `gamepad:` — `a`, `b`, `x`, `y`, `lb`, `rb`, `lt`, `rt`, `back`, `start`,
  `l3`, `r3`, `dpad-up`, `dpad-down`, `dpad-left`, `dpad-right`, `left-stick`,
  `right-stick`: the 16 W3C standard-layout button names plus the two stick
  presses.

No member carries a space, which is what makes the pair encoding below split
unambiguously. Reach grades attach to `keyboard:*` only: a pair naming a
`mouse:*` or `gamepad:*` id is a refusal (ADR-012, Decision Three names them
without grading them).

## Table data

The tables live directly in this version directory, one file per hand:
`left.toml` and `right.toml`. A loader reads each file and files the table it
holds under the hand its `hand` entry names, which must match the file name;
the two hands must stay two distinct tables (never one table answering under
both addresses). A table's declared `version` must also name the major of the
directory holding it: a `v2` document sitting in a `v1` directory is refused,
not read with `v1` semantics. Reading a table file is subject to the same size
budget as every other document: a file larger than the budget is refused
before it is read into memory.

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
outside this grammar, and are rejected — which is why the `keys` array is
written on one line, and why the backslash glyph is written `\\`.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `reach-classes` |
| `version` | string | yes | the major this document is written against (`v2`) |
| `hand` | string | yes | `left` or `right`; any other value is rejected |
| `classes` | array of strings | yes | the five taxonomy ids, once each, in **severity order** (worst last) |
| `keys` | array of strings | yes | each element exactly `"<input-id> <class-id>"` |

Any other entry is rejected, naming the entry and the version it was refused
against.

## The taxonomy

The five classes, graded by hand **posture from the home-row anchor** rather
than by distance (ADR-012, Decision Two). The order of the `classes` array is
the severity order, worst last, and it is *shared*: every v2 document lists
exactly these five ids, once each, in this order, so two hands cannot define
two scales. A placement bound is then integer rank arithmetic over that order —
which is why the order lives in the table rather than in a reader.

| Rank | Class | Meaning, for the hand that owns the table |
|---|---|---|
| 0 | `home` | the key is a home-position anchor for a finger of that hand |
| 1 | `thumb` | the thumb's designated key, reached with the hand at rest |
| 2 | `stretch` | a designated finger reaches it without moving the hand off its anchor |
| 3 | `far` | reached by shifting the hand slightly off the anchor |
| 4 | `off-reach` | not reachable from that hand's anchor posture |

Each hand grades each key exactly once, by the posture its designated home-row
finger (or thumb) uses from that hand's anchor; that is why `thumb` is a class
of its own rather than a home-key qualifier, and why the two hands' tables
carry different grades for the same key by definition — the right pinky's home
position is `;` and its `keyboard:'` is a stretch, while the left hand grades
both `off-reach`.

## `keys` rules

Every rule here is a rule about one document, so a loader never compares one
table file against another:

- each element is exactly **two** space-separated fields; three fields, or one,
  is refused naming the element;
- the input id is grammatically valid and `keyboard:*`;
- the class id is one this document's own `classes` entry lists;
- each input id appears once — a key graded two ways by one hand would make the
  table's own severity order ambiguous.

The shipped `left.toml` and `right.toml` grade all 48 members of the closed
`keyboard:` set: 26 letters, 10 digits, 11 ANSI-US punctuation keys, and
`space`. Modifiers and named keys (`shift`, `ctrl`, `tab`, `enter`) are outside
the closed member sets, so they are not graded and cannot appear until a
vocabulary major adds them. The grades themselves are authored BindForge data:
no external benchmark validates them, so the tables are the audit surface and a
retune is a reviewed PR to this directory, with no reader change.

## Version gate and size budget

Same rules as every format (`schemas/README.md`): a missing or unsupported
`version` is rejected with an error naming the supported majors; documents
larger than 1 MiB or nested deeper than 16 levels are rejected before parsing.
A `v1`-only reader refuses a v2 document through the version gate, naming the
majors it supports, and refuses `keys` in a v1 document naming the entry; those
v1 refusals stay byte-exact. The gate is **per-format**: `reach-classes` is the
only format with two majors, so a document declaring a major its own format
does not ship is refused naming that format's majors, and this directory's
`v2` widens nothing for a `profile`, `action-ids`, or `keyboard-geometry`
reader. A rejection message quoting part of the document is a single bounded
line, and never a local filesystem path.

## Fixtures

`fixtures/valid/` and `fixtures/invalid/` hold stand-alone table documents:
every implementer must accept the valid ones and reject the invalid ones with a
structured error naming what is wrong, which is what the conformance harness's
corpus walk checks — parse, grammar, and the closed entry set. For this format
the walk also loads every valid corpus table: inside a stand-in directory named
for the fixture's own major, with a table for the other hand taken from this
same major directory, its classes and grades must come back filed under the
hand its document declares, in table order. (The directory/version rule above
therefore holds for the stand-in too: it carries the fixture's own major as its
segment, and the pair completing it comes from that same major, so a fixture
never answers beside another major's table.)

The load rules above are also asserted against this directory's own table data,
`left.toml` and `right.toml`: they load as two tables, each filed under the
hand its file name and `hand` entry agree on, and both hands grade exactly the
same 48 input ids. A stand-alone corpus fixture carries no file name of its
own, so the file-name/hand agreement rule is part of the loader contract rather
than the fixture assertion.
