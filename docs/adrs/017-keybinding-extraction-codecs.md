---
doc_type: adr
status: proposed
date: 2026-11-23
depends-on: [012, 013]
ledger: yes
---

# ADR 017: Extract installed games' key bindings through codec families

This ADR takes the number `017`, reserved since ADR-009 for "codec
implementations and config path discovery", and builds the read half of that
reservation: codec families, config-path discovery, a raw binding-snapshot
contract, and an extraction face on `apps/discovery-cli`. The author-and-apply
CLI half the old reservation also named is re-reserved here (`017-R1`) — the
apply face is a hard downstream of the extraction contract this ADR first
defines, and ADR-023's intent/implementation decoupling postdates the
reservation's wording, so that half is not claimed by taking this number.
The `manifests/v1` `codec` id space (`013-C3`) keeps its closed-set posture
and receives its first named family; `DiscoveredGame.codec_id` leaves `None`
for the first time, but only on the face this ADR adds.

**Terminology.** A **codec family** is one engine or launcher lineage's binding
storage: where a game of that lineage keeps its binding files, in what format,
and what one binding entry means in that format. A **claim** (and the
**detection** record it returns) is a codec's declaration that it recognizes
one discovered game, carrying the candidate files it located and the evidence
it matched on. A **binding snapshot** is this ADR's artifact: the bindings a
game's own files state, in the game's own action words. A **native action
name** is the game's own spelling for what is bound; mapping it to a taxonomy
action id is the manifest's join (`013-C4`), never the extraction's. The
**launcher context** of a discovered game is the tree its user-owned config
actually lives in: the native host layout, or the Proton prefix the game was
launched under.

## Context

The framework has spent its history on contracts and none on content:
`data/manifests/` and `data/profiles/` are README-only trees, and
`discovery list` answers `200 []` because the catalog knows only hand-typed
seed entries. Everything downstream — manifests, profiles, transforms, the
apply tool, the UI — consumes the same missing fact: what a game's own
binding surface is. That fact is not something a curator can type at catalog
scale, but installed games state it on disk already: a game's bindings live
in files the game owns, in a format its engine family dictates. Extraction
is therefore the only content path that scales, and ADR-023's split says
exactly where its output lands — raw native bindings at the implementation
end, taxonomy ids only through a manifest's join.

Where those files live is the hard Linux fact this ADR must pin. The same
game stores user bindings in different trees depending on launcher context:
a native install keeps them under the host's user directories, a Proton
launch keeps them under `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/...`
at the Windows paths the game believes, and a steam-published game may keep
them under `<main root>/userdata/<steamid>/<appid>/` — the *main* steam
root, not the library folder the game itself sits in. A discovered-game row
carrying one install path cannot name any of those trees by itself, so this
ADR pins the context a codec is handed, not only the calls it may make.
Steam remains the only discovery source (`022`'s surface), and non-Steam
launcher lineages stay behind `022`'s and `025`'s unclaimed keys.

## Decision

We will add a `crates/codecs` crate owning the codec-family trait, the codec
registry, and the config-path resolvers; a `bindings/v1` snapshot contract
under `schemas/`; and a read-only `bindings` face on `apps/discovery-cli`
that resolves a discovered game, lets the registry claim it, extracts, and
prints the snapshot. Codecs are game-lineage knowledge and stay out of
`bindforge-core` (ADR-008 clause 3, the `transforms` precedent) and out of
`crates/discovery`, which keeps scanning installs and not reading a game's
private files. The unit ledger follows.

### Contracts

- **017-C1** The codec trait: `id()` returns the family id — slug grammar,
  `013-B4` bounds, the value a snapshot's `codec` records and the registry
  iterates by — `detect(&GameContext) -> Option<Detection>` claims or does
  not, and `extract(&Detection) -> Result<BindingSnapshot,
  ExtractionRefusal>` parses only what its own claim located. A claim is
  never a bare boolean: a `Detection` carries the family id, the located
  candidate files grouped per local SteamID where the family is
  user-scoped, and the match evidence (appid-table hit or install-tree
  probe). No claim, a claim, and a claim whose parse refused are three
  distinct outcomes. → spec: The codec face.
- **017-C2** The `GameContext` a codec is handed: the discovered-game row
  (`025`'s lookup key among them, its source token and store key parsed),
  and every steam root the scan resolved — main root and library roots
  separately, because `userdata/` lives only under the main root while
  `steamapps/compatdata/` lives under each library. A codec reads nothing
  outside the located candidate set, and locates nothing outside the
  context's trees. → spec: The codec face.
- **017-C3** `bindings/v1` is the snapshot contract — one document per game
  per local SteamID where user-scoped: `version`, `codec` (family id), the
  game's lookup key, `entries` as `{action, inputs}` pairs with `action`
  the native action or command name verbatim and `inputs` the ADR-012
  vocabulary spellings, and `provenance` naming the located files behind
  the entries. The entry set is closed at both levels like every other
  contract in `schemas/`, every string sits inside the shared bounds, and
  the loader refuses by name. → spec: `schemas/bindings/v1/spec.md`.
- **017-C4** The CLI face: `discovery bindings <slug>` runs the same scan
  `list` runs, resolves the slug to one discovered game, hands the context
  to the registry, and prints the snapshot as the aligned table or as
  `--json` (the snapshot document verbatim); `discovery bindings <slug>
  --probe` prints the located candidate files and the claim outcome without
  extracting — a found-but-unclaimed file is the registry of tomorrow's
  codec families, so it is visible. The face names the claim-refused and
  family-confirmed-files-absent outcomes (`017-L1`, `017-L2`) as their own
  statuses and exits non-zero on either. → spec: The CLI face.
- **017-C5** The codec registry is a closed static set iterated in family-id
  order; an appid-table claim pass runs before any install-tree probing,
  the first claim wins, and the winner's evidence is carried in the
  snapshot's provenance so an artifact says which codec claimed it and on
  what basis. → spec: The codec registry.

### Laws

- **017-L1** A codec never writes: not to a game, not to a launcher, not to
  the framework tree. Every file a codec opens is untrusted input inside
  `017-B1`'s cap read through the size-capped reader posture every loader
  already shares, and a parse that fails after a claim is a named
  `ExtractionRefusal` — never an empty snapshot, because a claimed game
  with an unreadable binding file is a different fact from a game no codec
  claimed. → spec: The read discipline.
- **017-L2** Family-confirmed-but-files-not-yet-found is a named status: a
  codec may recognize the game and locate nothing (never launched, cloud
  storage, deleted prefix); that status is printed as itself and is never
  collapsed into an empty entry list or a non-claim. → spec: The CLI face.
- **017-L3** A snapshot states only what a located file states: no native
  name is translated to a taxonomy id (`013-C4` owns that join), no binding
  is inferred, defaults are not absorbed, and no file outside the located
  candidate set contributes an entry. → spec: What extraction is.
- **017-L4** Detection and extraction are pure over their inputs: the same
  `GameContext` against the same file contents yields a byte-identical
  snapshot; nothing about wall-clock time, environment beyond the context,
  or registry iteration order can reach an artifact. → spec: Determinism.
- **017-L5** `discovery list` stays claim-free: no codec runs during a
  `list`, its `codec_id` column keeps spelling `None`, and only the
  bindings face pays for detection, so `list` keeps its scan-only runtime
  and its pinned output shapes (`025-C5`). → spec: The CLI face.

### Budgets

- **017-B1** `MAX_BINDING_FILE_OCTETS = 1_048_576` — the cap on any single
  file a codec opens; a binding store larger than a megabyte is the wrong
  file, and the reader refuses before reading it whole. → spec: Budgets.
- **017-B2** `MAX_SNAPSHOT_ENTRIES = 4096` — the cap on one snapshot's
  entries; a game with more bindings than this does not exist today, and a
  codec that claims otherwise is over budget by name. → spec: Budgets.
- **017-B3** `MAX_CANDIDATE_FILES = 32` — the cap on one game's located
  candidate set per local SteamID. → spec: Budgets.

### Reservations

- **017-R1** The apply-and-authoring half of this number's old reservation:
  writing bindings back into game-owned files with its backup-and-restore
  discipline, and the authoring CLI. Owner: unassigned. → unassigned
- **017-R2** Windows launcher-context path resolution (install-registry
  roots, `Documents\My Games`, `%LOCALAPPDATA%`). Owner: unassigned. → unassigned
- **017-R3** Further codec families, starting with the Source-family
  `config.cfg`/keybinding lineage. Owner: ADR-017 (follow-on runs). → 017
- **017-R4** Machine-readable codec routing — appid→family as `data/`
  content editable without a rebuild. Owner: unassigned. → unassigned

### Seeds

- **017-S1** The first family: `unreal-ini` — the `Saved/Config` INI layout
  (`Input`/binding-property spellings) resolved for native and Proton
  launcher contexts, with fixture-tree tests in the `codecs` crate and a
  read-only verification pass against two locally installed Unreal-lineage
  titles. → spec: The first family.

### Rollout

**Unit handles.** The handles in this ADR sit at the end of the Rollout
list items: one trailing code span of unit ids governs the line's claim, an
id anywhere else governs nothing.

1. The `codecs` crate lands with the codec trait, the `Detection` claim
   payload, and the closed registry iterated in family-id order. `017-C1 017-C5`
2. The scan hands the codec face its context — parsed lookup key, main
   root, and library roots — and nothing else. `017-C2`
3. `schemas/bindings/v1/spec.md` lands with its loader and the three
   budgets enforced at read and locate time. `017-C3 017-B1 017-B2 017-B3`
4. The `unreal-ini` family ships with fixture trees and the read-only
   verification pass, under the never-write discipline. `017-S1 017-L1`
5. The `bindings` face lands with probe mode, the two named statuses, and
   `list` untouched and claim-free. `017-C4 017-L2 017-L5`
6. Determinism and extraction-honesty pins run against the fixtures from
   the first family onward. `017-L3 017-L4`
7. The apply face, Windows paths, further families, and routing data ride
   their reservations; nothing in this run pre-empts their owners. `017-R1 017-R2 017-R3 017-R4`

## Alternatives considered

**A bare-boolean `is_match` in front of the extractor.** The shape reads
naturally — ask, then take — but it throws away the located files and the
match basis, forcing the extractor to re-probe the tree a second time and
leaving the CLI with nothing to print in probe mode; worse, it collapses
"my file exists but did not parse" into "this is not my game", which is
exactly the distinction an operator needs. The `Detection` payload keeps
the two-step shape while paying for neither flaw.

**Codecs inside `crates/discovery`.** Detection begins where discovery
ends, and one crate would mix install scanning with reading a game's
private config. `transforms` is the precedent for the split rule: core owns
formats, one crate owns one responsibility. Discovery feeds the
`GameContext`; codecs read files.

**The Source family first.** Widest reach across Steam's catalog and the
friendliest grammar (`bind <key> <command>`), but its per-user layout under
`userdata/` exercises none of the Proton-prefix resolution that is this
platform's actual gap, and the games on hand to verify against are
Unreal-lineage. Source is reserved in (`017-R3`) as the generality proof
next.

**Keeping the snapshot an internal CLI output rather than a contract.**
Anything that only a developer's terminal ever sees never has to age. The
snapshot is the input to manifest authoring, to the apply tool, and to the
UI's import, so `schemas/` is where its rules must live for those consumers
to exist without reading Rust.

## Consequences

Content becomes possible: the first manifests can be curated against real
extracted artifacts instead of guesswork, the apply tool (`017-R1`) gets
the input contract it depends on, and the UI gets something worth managing
locally. The cost is a maintenance tail that is genuinely open-ended: every
family is a game-lineage format that games update underneath us, so the
provenance block exists to notice drift and `017-L1`'s named refusals exist
to make drift loud rather than wrong. Proton layout assumptions track
Valve's releases and will need touching when they move. `bindings/v1` must
now age through majors like every other contract. Privacy stays local by
construction: SteamIDs and bindings are read on the user's machine and this
ADR ships nothing that leaves them — the catalog and registry are untouched,
and the DB-backed registry stays the separate downstream decision it
already is.
