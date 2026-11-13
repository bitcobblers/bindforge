# data/profiles

Shared binding profiles land here: a profile is what a player shares and what
the hub syncs, written in the profile format and speaking the taxonomy ids in
`schemas/action-ids/v1/registry.toml`. ADR-014 owns the profile format —
identity, chains, `seeded-from` — and populates this directory; nothing in the
tree reads it yet.

Whatever ends up here arrived from elsewhere — another player, another
machine — so from the moment ADR-014 reads it, every file in this directory is
untrusted input: the version gate, the parse budget, and the chain guards
(`schemas/README.md`) apply to it exactly as they do to a hub upload, and no
local path ever appears in a rejection shown to whoever sent it. The action
ids a profile names are the registry's ids, and the stability rule that keeps
them from churning is stated in `schemas/action-ids/v1/spec.md`, not restated
here.
