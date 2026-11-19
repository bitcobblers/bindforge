# data/catalog

The catalog corpus the root registry (`apps/registry`, ADR-026) serves from:
`v1/catalog.json` is one `026-C3` catalog document (`schemas/catalog/v1/spec.md`)
— curated game ids, each carrying the store lookup keys its installation is
recognised by.

`v1/catalog.json` is the **seed** ADR-026's `026-S1` asks for. Its game ids are
minted, grammar-valid kebab strings, not the ids of shipped manifests: ADR-025's
research records that `data/manifests/` is empty today, and the registry never
joins a `game_id` against that tree — it validates the id against the slug
grammar alone (`024-B1`, `013-L5`). Adopting the real manifest ids is a data edit
for the first manifest PR to make.

The unit's own wording is ahead of this file: `026-S1` still reads as drawing ids
from the shipped manifests and carries **no** partially-satisfied annotation. That
annotation is design 026's Rollout step 6 (`amended-by`/partially-satisfied lines
on `026-S1`, and `amended-by 026-L1` on `025-C4` and `025-L1`, per `021-L4`); the
decision-unit ledger is not a file this data tree may edit, so the annotation lands
with the ledger pass and is recorded as owed in
`docs/designs/026-registry-service/03-build-notes.md`. Until it does, read the
paragraph above as this seed's description of itself, not as the unit's text.

The file is a contract document like everything else in `data/`: untrusted input,
loaded through the format's own refusals, size-capped before it is read, and
never rewritten by a refusal. A root instance reads it through
`registry serve --root --corpus`; the admin face (`026-C4`) is the only writer,
and it saves through the tmp-and-rename discipline — which is why running two
instances against this one file is a refused, unsupported configuration.
