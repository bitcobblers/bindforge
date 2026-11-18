# action-ids format, v1

The action taxonomy: the id set every later contract speaks through
(ADR-011, decision two). An id names one bindable action across games —
`move.steer-left` is the same action in Remnant 2 and in the next game a
manifest describes. ADR-013 maps a game's own action names onto these ids,
ADR-014's profiles name them, and ADR-015 flips polarity only for the
`discrete-lateral` ones. What this format does *not* carry is emphasis:
where a game places an action is a game fact recorded in the manifest, and
an `emphasis` entry here is a refusal, not an ignored key.

## Registry data

`registry.toml`, sitting directly in this version directory, is the only id
list. `bindforge-core` loads it as the registry; the set is never repeated in
program code, so a reader and this file cannot drift into two different id
sets. Adding an id means a PR to this file. A `registry.toml` that arrives
from anywhere else is data: only the copy shipped in this versioned directory
is ever consulted for `axis_kind`.

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
outside this grammar, and are rejected — which is why every axis array here is
written on one line.

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | yes | must be `action-ids` |
| `version` | string | yes | the major this document is written against (`v1`) |
| `discrete-lateral` | array of strings | yes | ids that are one half of a separately-bound left/right pair; may be empty |
| `continuous-lateral` | array of strings | yes | ids that are one graded lateral control; may be empty |
| `none` | array of strings | yes | every other id; may be empty |

Any other entry is rejected, naming the entry: an entry naming a fourth axis
kind, or an `emphasis`, is refused rather than ignored, because the arrays
*are* the id set and an undefined entry carries meaning no implementer has
agreed to. A length bound on the arrays is the shared parse budget
(`schemas/README.md`), not a per-format constant.

## Id grammar

An id is exactly two `.`-separated segments. Each segment opens with an ASCII
lowercase letter, then holds only `a-z`, `0-9`, and `-`, closes with a letter or
a digit, and is at most `MAX_SLUG_OCTETS` octets — the shared slug bound
`schemas/README.md` states once (`024-B1`), whose bullet carries the superseded
DNS-label derivation so this format states no number or origin of its own. The
bound is shared rather than per-format: an id segment, a manifest `id`, a
profile `id` and a discovered game slug are the same kind of name, and a
discovered slug must stay adoptable by a manifest byte-for-byte. Comparison is
byte-exact: no case folding, no Unicode normalization, no canonicalization of
any kind, so
two spellings never name one action. `Move.Forward`, `move.strafe_left`,
`move..steer`, `steer-left`, `move.steer.left`, and `segment-past-cap.toml`'s
segment one octet past the shared bound are each a refusal, and each has a
fixture under `fixtures/invalid/` naming it.

A single-segment name is a malformed id, not an unregistered one: there is no
unprefixed id in this namespace, and an unregistered id is simply absent from
the registry. Namespaces are a naming convention, not a closed set: the v1
registry's are `move`, `aim`, `combat`, `interact`, `item`, `ui`, and `system`,
and a new namespace is just new rows in the registry.

## Uniqueness

An id appears once across the three arrays. A repeat — inside one array or
across two — is a rejection naming the id, because the three arrays are one id
set: membership and axis kind are one fact, and an id graded two ways would
quietly change what a later transform may act on.

## Axis kind

The classification rule, stated here because it is load-bearing:

- an action is **`discrete-lateral`** when it is one half of a left/right pair
  bound as two separately-bound endpoints, so mirroring swaps two bindings;
- it is **`continuous-lateral`** when it is one graded lateral control whose
  mirror is a sign flip on that control — camera look, whose inversion is a
  game option, never a key swap;
- otherwise it is **`none`**.

The discriminator is the shape of the binding, not the device: a game that
drives steering from an analog stick still binds `move.steer-left` and
`move.strafe-left` as two endpoints, so the `move.steer-*` and
`move.strafe-*` ids are `discrete-lateral`. Forward/back is not lateral, so it
is `none`. `registry.toml` pins the `discrete-lateral` set to exactly those
four ids, and nothing in v1 acts on the attribute: ADR-015 flips polarity
inside that set and nowhere else.

## Extension and withdrawal

The registry is a compatibility promise (ADR-011, Consequences):

- ids are added by PR to `registry.toml` and reviewed before merge; once
  ADR-013's manifests exist, an addition arrives through manifest review;
- within a major, no id is renamed, retargeted, or removed;
- withdrawal is a new major directory (`action-ids/v2`), which is the
  versioning rule `schemas/README.md` already states;
- widening the id *grammar* rides that same directory: a third segment in
  `action-ids/v1/registry.toml` makes the file invalid for every v1 loader, so
  it belongs in `action-ids/v2`, where the version gate is what refuses it for
  a v1 reader.

## Version gate and size budget

Same rules as every format (`schemas/README.md`): a document that declares no
`version`, or a major the reader does not support, is rejected with an error
naming the supported majors, never defaulted. A registry loaded from a version
directory must also declare the version of the directory holding it: a
`registry.toml` declaring `v1` inside `action-ids/v2/` is a refusal, not a
second reading of the file. Documents larger than 1 MiB or nested deeper than
16 levels are rejected before parsing, the cap binding before the file is read.
Every refusal is one bounded line naming what is wrong, with the offending id
echoed escaped to printable characters, and never a local filesystem path.

## Fixtures

- `fixtures/valid/` — documents this format accepts; an implementer must also
  be able to re-serialize them to a semantically equal document.
- `fixtures/invalid/` — documents every implementer must reject with a
  structured error naming what is wrong. Each carries one fault and is named
  after it.

The conformance harness's `schemas/*/v*/` walk picks this directory up with no
harness change; the corpus, not the code, is what makes the format supported.
