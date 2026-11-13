# data/profiles

Shared binding profiles land here: a profile is what a player shares and what
the hub syncs, written in the profile format and speaking the taxonomy ids in
`schemas/action-ids/v1/registry.toml`. ADR-014 owns the profile format —
identity, chains, `seeded-from` — and populates this directory.

Nothing lands here yet: the first real chains are their own PRs, and this run
ships the contract and the reader, not the data. The reader exists — ADR-014's
`load_chain` opens a chain of `profile/v2` link documents and `resolve` joins it
to a manifest — so a chain placed in this directory is already read through the
chain rules `schemas/profile/v2/spec.md` states: one directory per chain, the
head's `vN/` directory fixing the chain's major, every link looked up as
`<id>.toml` beside it, and the chain guards applied to the resolved chain.

Whatever ends up here arrived from elsewhere — another player, another
machine — so from the moment ADR-014 reads it, every file in this directory is
untrusted input: the version gate, the parse budget, and the chain guards
(`schemas/README.md`) apply to it exactly as they do to a hub upload, and no
local path ever appears in a rejection shown to whoever sent it. The action
ids a profile names are the registry's ids, and the stability rule that keeps
them from churning is stated in `schemas/action-ids/v1/spec.md`, not restated
here.
