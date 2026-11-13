# data/manifests

Game manifests land here: one file per supported game, mapping that game's own
action names onto the taxonomy ids in `schemas/action-ids/v1/registry.toml`
and recording the game facts — where an action sits, what a preset means —
that the taxonomy deliberately refuses to claim. ADR-013 owns the manifest
format and populates this directory; nothing in the tree reads it yet.

Whatever ends up here arrived from elsewhere — a contributor's PR, a hub
upload, a forum paste — so from the moment ADR-013 reads it, every file in
this directory is untrusted input and goes through the same guards every
contract document gets (`schemas/README.md`: the size budget, the closed entry
set, the bounded single-line rejection). The ids a manifest names are the
registry's ids; the stability rule that keeps them from churning is stated in
`schemas/action-ids/v1/spec.md`, not restated here.
