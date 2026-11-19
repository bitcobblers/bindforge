# catalog format, v1

A catalog document is the corpus one registry answers from: the curated list of
games, each with the store keys its installation is recognised by. It is the
document `026-C3` fixes, and it is the join's whole input — a client's
`GET /v1/candidates?slug=<lookup key>` question (`025-C4`) is answered by the
records whose `source_keys` contain that key, projected to `{game_id, name}` and
nothing else (`026-C2`).

A catalog document is **JSON**, not TOML: `schemas/README.md`'s "Formats are
TOML" paragraph records this format as the one documented exception, and its
conformance is exercised by the loader in `apps/registry` (`corpus.rs`), not by
core's TOML `parse`/`rejection` sweep. It joins the other JSON contract
documents — the match store, the candidate response body, and the `026-C7` cache
document — outside the TOML subset.

The document is untrusted input at both of its seams: at load, and again at the
admin write that renames a new one into place (`026-C4`). Nothing a refusal
touches is rewritten.

## Shape

```json
{
  "version": 1,
  "revision": 0,
  "games": [
    { "game_id": "example-game", "name": "Example Game", "source_keys": ["steam/900001"] }
  ]
}
```

| Entry | Type | Required | Meaning |
|---|---|---|---|
| `version` | integer | yes | the bare integer `1`. It is a bare integer here and not the `"v1"` string the TOML formats declare: the version refusal lives in this format's own loader, named, and never in core's `UnsupportedVersion` path |
| `revision` | integer | yes | a non-negative integer, bumped by every accepted mutation; it is a count, not a timestamp, and it is one of `GET /v1/admin/stats`' three integers |
| `games` | array | yes | the records, `0..` of them; a document with no games is valid and answers `200` with `[]` |

A record is an object with exactly these three entries:

| Entry | Type | Meaning |
|---|---|---|
| `game_id` | string | the curated id: the slug grammar `[a-z][a-z0-9-]*` within `MAX_SLUG_OCTETS` (128) — `024-B1` as `026-C3` borrows it (`013-L5`) |
| `name` | string | the game's display name, at most `MAX_NATIVE_NAME_OCTETS` (128, `013-B4`) octets |
| `source_keys` | array of strings | the `<token>/<external-id>` lookup keys this game is installed as, each one admitted by the shipped pair below |

A lookup key is admitted by exactly the rule the shipped client and the shipped
plugins already agree on: `spells_lookup_key` over the closed token set `steam` ·
`epic` · `local` (ADR-025), within `MAX_DISCOVERY_SLUG_OCTETS` (128, `025-B1`).
The key rides **no** slug grammar — its `/` is the point, and a dotted external
id (`local/Steam.SE`, the `local/<dir>` spelling) is admitted. The only extra
refusals are path safety, at their own scope: no further `/` or `\` after the
first one, no `.` or `..` component, and no control byte. Keys are opaque
in-memory strings and are never joined onto a path.

The entry set is closed at **both** levels: any other entry — at the top of the
document or inside a record — a missing entry, or a wrong-typed entry is a named
refusal. An entry the format does not spell is never ignored, and because the same
gate serves the admin upsert, a `POST /v1/admin/games` body carrying a fourth
record entry is refused as the record it is not.

## Named refusals

One refusal is one line, escaped and truncated (`schemas/README.md`'s logging
rule), and names the condition and the file. The `fixtures/invalid/` corpora are
the contract for which condition produces which refusal:

| Condition | Fixture |
|---|---|
| `version` absent or not the bare integer `1` | `version-string-v1.json`, `version-two.json`, `version-missing.json` |
| `revision` negative, absent, or not an integer | `revision-negative.json`, `revision-missing.json` |
| `games` absent or not an array | `games-missing.json` |
| a record that is not an object, or missing/miss-typed one of its three entries | `record-not-an-object.json`, `record-missing-name.json`, `source-keys-not-an-array.json` |
| `game_id` out of the grammar or over 128 octets | `game-id-uppercase.json`, `game-id-over-bound.json` |
| `game_id` repeated — records are keyed by it, so the document loads ambiguously | `game-id-repeated.json` |
| `name` over 128 octets | `name-over-bound.json` |
| a `source_key` the lookup-key rule refuses | `source-key-unknown-token.json`, `source-key-over-cap.json` |
| a `source_key` repeated within one record — the index would post the game twice and crowd the candidate window | `source-key-repeated-in-record.json` |
| an entry outside the closed set, at document or at record level | `record-entry-outside-closed-set.json`; the document-level twin (a fourth top-level entry) is refused by the same gate |
| a document that is not JSON at all | `not-a-document.json` |
| a document over `MAX_CATALOG_DOCUMENT_OCTETS` | no fixture: the cap is 16,777,216 octets and a golden file of that size is not worth committing. The refusal — `fs::metadata`-gated at load, and measured on the post-`revision`-bump buffer at write — is pinned by the loader and mutation suites at `apps/registry/tests/catalog_document.rs` and `apps/registry/tests/admin_mutations.rs` |

Every bound is a cap, not a target: a record sitting exactly on 128-octet
`game_id`, 128-octet `name`, and a 128-octet lookup key loads
(`fixtures/valid/on-every-length-bound.json`).

## This format's budget

`MAX_CATALOG_DOCUMENT_OCTETS = 16_777_216` (16 MiB) caps one catalog document at
**both** ends of its life, and it is private to `apps/registry`: it is not one of
core's exported thirteen constants, and this file is where the contract states
it (`026-B1`–`026-B3`'s siblings are stated the same way — the precedent is the
shipped client's private `STORE_READ_BUDGET_OCTETS`).

- **At load**, the check rides `fs::metadata` *before* the read and is refused by
  name, so an oversized file is never buffered.
- **At write**, the check runs on the exact buffer — with its bumped `revision`
  already in it — that the admin face renames into place, so no accepted mutation
  can persist a corpus its own loader would refuse. An upsert whose document comes
  out over the cap is a `422` and leaves the model, the `revision`, and the file
  untouched, with no temporary file written.

The number is the corpus scale ADR-026 states — tens of thousands of games, each
with several store keys — at the ~140 octets one
`{game_id, name, source_keys[]}` record costs: 16 MiB ÷ ~140 ≈ 120,000 records.
The arithmetic is per record, not per candidate list: a catalog document carries
no candidate lists, so the only unbounded growth is record count, and the record
count is what both caps bound.

## The answer's budget

What one document may be *asked for* is capped separately, and the cap is A1's:
`MAX_RESPONSE_OCTETS = 65_536` and `MAX_CANDIDATES = 256`, applied in **both**
directions.

- **Inbound**, the upstream client refuses a header block or body over the cap, a
  `Content-Length` promising more than it, and an answer carrying more than
  `MAX_CANDIDATES` candidates.
- **Outbound**, `GET /v1/candidates` answers with the largest prefix of the join
  that fits both caps. The postings are already ordered by `game_id`, so the window
  the octet cap trims is the same stable window the count cap uses: two requests
  for one key get the same bytes.
- **In the `026-C7` cache**, the per-entry candidate list is bounded at the write
  seam and an entry over the cap is a load refusal — a stored entry that could only
  be answered by exceeding the caps is not a document this instance keeps.

The arithmetic the count cap alone misses: each field of the `{game_id, name}`
projection rides its own 128-octet bound (`024-B1`, `013-B4`) — inclusive, as
`fixtures/valid/on-every-length-bound.json` shows — so one candidate costs ≈283
serialized octets and `MAX_CANDIDATES` of them ≈72,000, over the cap at a crossover
near 231 candidates. A body like that is not merely large: it is one the shipped
reader refuses, so a root that emitted it would answer `catalog unreachable` to the
CLI and an upstream error to a caching instance, and would thereby stop serving the
face `026-C1` fixes. Since a candidate never costs more than that, a non-empty join
always keeps at least one: the trim is a shorter answer, never an empty one, and
never the `026-L4` `503`, which stays the caching-mode cold case.

## Placement and the two documents

The same shape appears in two places, with two owners:

- `data/catalog/v1/catalog.json` is a root instance's corpus (`026-S1`), named by
  `--corpus`. `--corpus` names a file that has to **exist**: absent-is-empty is the
  rule for the caching instance's documents below, and a root whose one content
  argument is missing is the named startup refusal, not a silently empty public
  catalog.
- `<data>/bindforge/registry/catalog-v1.json` is a caching instance's
  local-entries document (`026-C7`), created empty when absent and mutated only by
  that instance's admin face. `<data>` is the shipped data-directory rule:
  absolute `$XDG_DATA_HOME`, else absolute `$HOME/.local/share`, and absolute
  `%APPDATA%` on Windows; a relative value counts as unset.

Both are saved through the shipped atomic-persist discipline: a fixed temporary
name opened `create_new`, mode `0600` under `0700` parents, `sync_all`ed before the
rename — so a mutation the admin face answered for is on the medium before it can
become the document — and then renamed into place. (The parent directory is not
synced: std offers no portable directory handle, and a rename lost that way leaves
the *previous* document whole, which is the safe direction.) It follows that
**running two `registry serve` instances against one corpus is a refused,
unsupported state**: the second writer's temporary file already exists and the write
fails loudly. There is no startup check and no instance marker — the loud write-time
failure is the whole mechanism.
