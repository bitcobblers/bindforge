# Research: Profiles as chains of base and patch (ADR-014)

Scope: the ADR adds `schemas/profile/v2/` — v1's four entries plus `for`,
`kind`, the `set`/`clear`/`swap` delta lists and provenance — decides that
chain resolution is deterministic and that `diff`/`fork`/`merge` operate on
chains, and keeps provenance as a label rather than a trust mechanism. It does
not decide the transforms that produce bases (015), the CLI surface (ADR-017),
or where a player's personal chain is stored. Lenses applied: **Prior art**,
**Standards**, **Security landscape**, under Findings, Prior art, and Risks &
unknowns.

## Summary

The chain's skeleton is already decided and shipped — `derives` nearest-first,
`resolve_chain` walking iteratively, `MAX_CHAIN_DEPTH = 32`, `Cycle` and
`ChainTooDeep` refusals — so the open design work is the v2 grammar and what
each link *contributes*, not the chain idea (source: schemas/profile/v1/spec.md;
source: crates/core/src/chain.rs). Independent systems that solved the same
problem — git rebase, quilt, Nix overlays, systemd drop-ins, Steam Input action
set layers — converge on the ADR's shape: an ordered delta list over a named
base, re-appliable to a regenerated base, with a deterministic, documented
merge order (source: https://git-scm.com/docs/git-rebase;
source: https://savannah.nongnu.org/projects/quilt;
source: https://wiki.nixos.org/wiki/Overlays;
source: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html;
source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
The delta vocabulary has two IETF standards as common denominators — JSON Patch
(RFC 6902) and JSON Merge Patch (RFC 7396, which obsoletes 7386 within the same
month it was published) — and both show that minimal patch semantics still get
published wrong and must be corrected, which is the case for pinning every v2
rule in fixtures from day one (source: https://datatracker.ietf.org/doc/html/rfc6902;
source: https://datatracker.ietf.org/doc/html/rfc7396). The security posture
the repo already enforces (1 MiB before read, 16-deep nesting, 32-link chains,
cycles refused) matches the published failure classes: recursive-expansion DoS
in XML/YAML/TOML parsers and config-execution loops that hang game clients
(source: https://en.wikipedia.org/wiki/Billion_laughs_attack;
source: https://github.com/toml-rs/toml/issues/206;
source: https://wiki.teamfortress.com/wiki/Scripting). The largest unknowns
are outside the ADR's explicit "not decided" list: cross-file chain resolution
(what today's guard validates is one document's `derives` list), what a v1
profile means inside a v2 chain, and how the line-oriented grammar encodes a
`set` mapping and a `swap` pair.

## Findings

### The chain as it is decided and shipped on `main`

- `profile/v1` fixes exactly four entries — `format`, `version`, `id`,
  `derives` — with `derives` an array of strings, ancestor chain nearest-first,
  default empty; the entry set is closed on purpose, and the refusal it exists
  to refuse is an alias like `derives_` reading as *no ancestors at all*
  (source: schemas/profile/v1/spec.md; source: crates/core/src/document.rs).
- `resolve_chain` walks the chain iteratively and refuses a repeated id with
  `CoreError::Cycle` and more than `MAX_CHAIN_DEPTH` (32) links with
  `ChainTooDeep`, before any deeper traversal; its header says chain
  *validation* is decided and "all other chain semantics are ADR-014's"
  (source: crates/core/src/chain.rs).
- But the guard validates **one document's** `derives` array: the cycle check
  is a `HashSet` over the strings in that single list, and the depth check is
  `profile.derives.len() > MAX_CHAIN_DEPTH`. No code resolves an ancestor id
  into another document, so a two-file cycle (A derives B, B derives A) is
  invisible to it today — the shipped `Cycle` fixture is single-document
  (`id = "self-deriving"` deriving from itself)
  (source: crates/core/src/chain.rs;
  source: schemas/profile/v1/fixtures/invalid/self-deriving-chain.toml).
- The version gate is per-format: `format_majors` gives `profile` only `v1`
  today, while `supported_majors()` already declares `["v1", "v2"]` for
  `reach-classes`'s sake. Adding `profile/v2` changes one `format_majors` line
  and adds a closed `PROFILE_V2_ENTRIES` set picked by declared version — the
  exact mechanism `reach_classes_entries()` demonstrates — without widening the
  crate-wide major union (source: crates/core/src/document.rs).
- `check_dir_version` refuses a document whose declared major differs from the
  `vN/` directory it loads from, which is what stops a v2 profile ever being
  read with v1 grammar once both exist (source: crates/core/src/document.rs).
- The conformance harness walks `schemas/*/v*/fixtures/{valid,invalid}` with no
  per-format hook, so a `profile/v2` directory is picked up automatically; for
  profiles it asserts the full round-trip — parse, `serialize`, re-parse,
  semantic equality — with `reach-classes` the only exempt format — so every
  v2 entry, including the delta lists, must be writable back through
  `serialize` (source: crates/core/tests/conformance.rs).
- The zero-fixture rule (a version directory with an empty valid or invalid
  corpus fails the run) and the case-unique lowercase-`-`-joined fixture-name
  rule bind `profile/v2` the moment its directory exists
  (source: crates/core/tests/conformance.rs; source: schemas/README.md).
- Shared parse budgets apply to v2 unchanged: `MAX_DOCUMENT_BYTES = 1 MiB`
  checked against file metadata *before* the read and by taking at most one
  byte more than the budget from a stream, and `MAX_NESTING_DEPTH = 16`
  (source: crates/core/src/toml.rs; source: schemas/README.md).
- The right-hand side of the `for` pin already exists and is stricter than
  ADR-009's sketch: `manifests/v1` carries `id` plus an integer `revision` of
  at least 1 — `fixtures/invalid/revision-zero.toml` pins that "a pin of
  `\"<id>@0\"` names a revision no bump rule can reach" — so the v2 pin is
  `<manifest-id>@<revision-int>` against a shipped grammar, not the invented
  `remnant2@0.1` form (source: crates/core/src/document.rs;
  source: schemas/manifests/v1/fixtures/invalid/revision-zero.toml;
  source: docs/adrs/009-canonical-profile-and-manifest-model.md).
- The grammar's precedent for a mapping inside a one-line string array is the
  space-joined pair, and it works because no member carries a space: manifest
  rows are `"<action-id> <native name>"`, docked rows `"<action-id> <input-id>"`,
  reach pairs `"<input-id> <class-id>"`, geometry triples `"<id> <col> <row>"`,
  each with an invalid fixture pinning the wrong field count
  (source: crates/core/src/document.rs; source: schemas/reach-classes/v2/spec.md;
  source: schemas/keyboard-geometry/v1/fixtures/invalid/pair-two-fields.toml).
  A `set` entry ("action id → input id") and a `swap` pair ("pairs to
  exchange") fit that encoding only if taxonomy ids and input ids carry no
  space — which they do not by their grammars — but `swap` between two *actions*
  versus two *inputs* is a different pair type the ADR does not disambiguate.
- The inputs a patch may `set` are the closed vocabulary of `reach-classes/v2`:
  `keyboard:` is exactly 48 members (letters, digits, fixed punctuation,
  `space`), `mouse:` and `gamepad:` fixed sets, and "an id outside a set is a
  refusal, not an unknown member" — so a v2 patch still cannot name Tab, Enter,
  Esc, arrows or F-keys, the same 012 gap ADR-013 collided with
  (source: schemas/reach-classes/v2/spec.md).
- The consumers of 014's semantics are already written: ADR-015's rule that
  "a personal patch (014) sits above them in the chain and survives base
  regeneration untouched" makes *deterministic re-resolution over a changed
  base* a load-bearing property, and 015 explicitly rejects "acting on a
  resolved layout" because it destroys the base/patch separation
  (source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- `data/profiles/` states the posture v2 inherits: every file there "arrived
  from elsewhere" and is untrusted input under the version gate, parse budget
  and chain guards, exactly as a hub upload
  (source: data/profiles/README.md).

### Deterministic application — what standards pin (Standards lens)

- RFC 6902 (JSON Patch) is the closest standard to `set`/`clear`/`swap`: "a
  sequence of operations", and "Operations are applied sequentially in the
  order they appear in the array. Each operation in the sequence is applied to
  the target document; the resulting document becomes the target of the next
  operation" — the ADR's "apply the deepest base, then each patch in chain
  order" is the same rule one level up
  (source: https://datatracker.ietf.org/doc/html/rfc6902).
- RFC 6902 also shows what `swap` costs: it has no `swap`; `move` is defined
  as "functionally identical to a `remove` operation on the `from` location,
  followed immediately by an `add`", so exchanging two bindings is a
  sequence whose intermediate state every implementer must agree on — a
  first-class `swap` avoids that, but only if its semantics against a `set` of
  the same pair in the same patch are pinned
  (source: https://datatracker.ietf.org/doc/html/rfc6902).
- RFC 6902's `test` operation is the standardized shape of a precondition —
  "The target location MUST be equal to the `value` value" — which is 015's
  preconditioning expressed in a patch language; 014 needs no equivalent, but
  the existence of the op shows a patch format can carry preconditions without
  becoming a program (source: https://datatracker.ietf.org/doc/html/rfc6902;
  source: docs/adrs/015-transforms-as-preconditioned-policies.md).
- RFC 7386 (JSON Merge Patch) is the minimal alternative to a delta list — the
  patch mimics the target, and "Null values in the merge patch are given
  special meaning to indicate the removal of existing values in the target",
  i.e. a built-in `clear`; it also states its limits — "not appropriate for
  all JSON syntaxes", arrays replaced wholesale — which is why an action→input
  map suits a delta list better than a merge document
  (source: https://datatracker.ietf.org/doc/html/rfc7396).
- Both merge-patch RFCs carry the same content one month apart because RFC
  7386 is marked "Obsoleted by RFC 7396" — a published, Standards-Track patch
  format was replaced immediately to fix itself. The lesson for v2 is the one
  the repo already lives by: every semantic edge needs a golden fixture at
  publication time (source: https://datatracker.ietf.org/doc/html/rfc7386;
  source: https://datatracker.ietf.org/doc/html/rfc7396;
  source: schemas/README.md).
- Order determinism is treatable as contract, not implementation: systemd
  specifies it outright — drop-ins "merged in the alphanumeric order",
  "Multiple drop-in files with different names are applied in lexicographic
  order, regardless of which of the directories they reside in", with
  `/etc` > `/run` > `/usr/lib` precedence — so "the result is a function of
  the chain" has a working example of pinning *why* the order is what it is
  (source: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html).

### Chain operations — what comparable tools built (Prior art lens)

- git rebase is the ADR's rebase, named: "Reapply commits on top of another
  base tip", "Transplant a series of commits onto a different starting point";
  and when a patch no longer fits its moved base, git does not guess — rebase
  "will stop at the first problematic commit and leave conflict markers",
  with `--continue`, `--skip`, `--abort` as the operator's options. That is
  the model for surfacing "a re-pinned base silently changed what the same
  patch means" — which git treats as a hard stop
  (source: https://git-scm.com/docs/git-rebase).
- quilt is the patch-series analogue of a chain: it "allows you to easily
  manage large numbers of patches by keeping track of the changes each patch
  makes. Patches can be applied, un-applied, refreshed" — push/pop over a
  base is decades-old practice, and quilt's `series` file is an explicit,
  ordered, editable list — the same thing `derives` encodes in the opposite
  direction (source: https://savannah.nongnu.org/projects/quilt).
- Nix overlays are the ADR's "re-appliable when the base is regenerated":
  overlays "provide a method to extend and change nixpkgs", applied
  reproducibly (fetchpatch with content hashes) every time the package set is
  evaluated — modifications as data over a generator, never edits to the
  generated artifact, exactly the 015 rule. The wiki also documents the
  self-reference failure mode: an overlay that overrides an attribute with a
  function of *itself* "would cause infinite recursion" — the same shape
  `CoreError::Cycle` refuses
  (source: https://wiki.nixos.org/wiki/Overlays;
  source: crates/core/src/chain.rs).
- systemd drop-ins are the base/patch adoption mechanic in a shipped system:
  a main unit file plus override files that "alter or add configuration
  settings for a unit, without having to modify unit files", and prefix-hierarchied
  drop-ins where "equally named drop-in files further down the prefix
  hierarchy override those further up" — prior art for the ADR's "mount at
  any rung"
  (source: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html).
- Steam Input already ships a layered binding model for games: Action Set
  Layers "overlay incremental changes to the active Action Set", declared with
  a `parent_set_name` — a patch naming the base it modifies, in the industry's
  own input vocabulary — though as runtime overlays for modal UI, not as
  shareable files (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Steam's adoption semantics are the player-side behavior 014 assumes: "the
  official configuration will be loaded by players, but if the user makes
  edits and manually selects another configuration they will no longer be
  pointed at that official configuration" — the personal chain breaking the
  pointer to the base is a deliberate design choice Valve made too
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Steam's revision flags show the cost side of re-pinning a base: breaking
  changes are carried as `major_revision` plus population-scoping flags
  (`major_revision_affects_xinput`, `..._mousekb`), and Valve recommends
  "prompt the user rather than force players who've constructed custom
  configurations to rebuild them" — a base bump that silently re-resolves
  personal patches is player-visible damage, priced by asking
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).
- Source-engine config chains are the closest everyday analog to a chain of
  text files driving bindings: `config_default.cfg` executes when there is no
  `config.cfg`, `autoexec.cfg` runs on startup, per-class `scout.cfg`-style
  files execute on class switch, and `exec` pulls in another file — a base
  plus ordered personal layers living in the game's own format
  (source: https://wiki.teamfortress.com/wiki/Scripting).

### Adversarial chain semantics — published failure classes (Security lens)

- Recursive-reference expansion is a named, old attack class: the billion
  laughs attack turns a sub-kilobyte document into gigabytes through nested
  definitions, the quadratic-blowup variant evades nesting detectors, and the
  pattern generalizes to "any file format that can contain macro
  expansions" — YAML anchors included; the Go YAML processor was "modified to
  fail parsing if the result object becomes too large"
  (source: https://en.wikipedia.org/wiki/Billion_laughs_attack).
- The concrete CVE class for a parser fed hostile network input is live:
  CVE-2022-28948 / GHSA-hp87-p4gw-j4gq — `gopkg.in/yaml.v3`'s `Unmarshal`
  crashes on crafted input, scored 7.5 HIGH, CWE-502 "Deserialization of
  Untrusted Data" — the same trust boundary `data/profiles/README.md` and the
  hub upload path declare
  (source: https://github.com/advisories/GHSA-hp87-p4gw-j4gq;
  source: data/profiles/README.md).
- Rust TOML parsers shipped exactly the DoS the repo's budgets pre-empt:
  toml-rs issue #206 is "Stack overflow when parsing a deeply nested
  `toml_edit::Document`" — a `x=[[…]]` string crashing the process — and the
  fix history shows how hard a safe depth number is: a recursion limit was
  added in toml_edit 0.16.1 (2022-12-27) with an `unbounded` opt-out feature,
  and 0.22.8 later had to "drop recursion limit from 128 to 100 to work on
  `opt-level = 0`" because even the *limit* overflowed an unoptimized stack.
  BindForge's own subset parser (`toml.rs`) rejects at 16 levels before
  building any value, and its grammar admits no nesting-heavy syntax, so the
  class is refused at the grammar level — v2's delta lists must not change
  that (source: https://github.com/toml-rs/toml/issues/206;
  source: https://github.com/toml-rs/toml/blob/main/crates/toml_edit/CHANGELOG.md;
  source: crates/core/src/toml.rs; source: schemas/profile/v1/spec.md).
- Game config-chain semantics are proven powerful enough to exhaust a client:
  TF2's own wiki warns "Loops which run without a wait command will cause the
  TF2 client to hang", `wait` "disabled on some servers" as a countermeasure,
  and — most relevant to a chain budget — "Each config file is limited to one
  mebibyte … though this restriction can be circumvented by exec-ing another
  config file at the end of the first": a per-document size cap does not bound
  a chain of documents. BindForge's per-document 1 MiB cap plus the 32-link
  chain bound is the closed version of that loophole, but the *composite*
  budget (up to 32 × 1 MiB per chain resolution) is a stated number nowhere
  (source: https://wiki.teamfortress.com/wiki/Scripting;
  source: schemas/README.md; source: crates/core/src/chain.rs).
- The chain's own guard already anticipates hostile uploads — ADR-014's
  Consequences line says "the 32-deep bound and cycle refusal already hold
  against hostile uploads" — and that claim holds only for single-document
  validation; see Risks
  (source: docs/adrs/014-profiles-as-chains.md; source: crates/core/src/chain.rs).

## Prior art

How comparable systems answered the same five questions:

| System | Chain/layer model | Delta language | Order rule | Re-apply to new base | Cycle/bound guard |
|---|---|---|---|---|---|
| git (`rebase`) | commit series on a base tip | whole commits (diffs) | replay commits one by one, in order | first-class (`rebase`, `--onto`); stops at first conflict | n/a (DAG by construction) (source: https://git-scm.com/docs/git-rebase) |
| quilt | patch series over one base | unified diffs, one file each | `series` file order | push/pop/refresh, then re-series on a new base | n/a (source: https://savannah.nongnu.org/projects/quilt) |
| Nix overlays | ordered overlay stack over a generated package set | function returning changed attributes | fold over the overlay list | automatic — overlays re-apply whenever the base is evaluated | infinite recursion refused | (source: https://wiki.nixos.org/wiki/Overlays) |
| systemd units | unit file + `.d/` drop-in stack | ini sections merged key-wise | alphanumeric merge, lexicographic across directories, fixed path precedence | drop-ins re-apply under any regenerated unit | 255-char name cap, `/dev/null` masking (source: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html) |
| RFC 6902 JSON Patch | operation list over a target | `add remove replace move copy test` | "applied sequentially in the order they appear"; abort on error | not defined | none in spec (source: https://datatracker.ietf.org/doc/html/rfc6902) |
| RFC 7396 JSON Merge Patch | patch document mimicking the target | members = set, `null` = remove | single merge function, no order question | not defined | none (source: https://datatracker.ietf.org/doc/html/rfc7396) |
| Steam Input | action set + `action_layers` overlays | per-action rebinds, layers per parent set | parent/child set relation | game regenerates; layers declared by name | revision counters + affects flags (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file) |
| Source engine cfg | `config_default` → `config.cfg` → per-class cfg → `exec` | `bind`/`alias` console lines | execution order, last write wins | none — a game patch overwrites | 1 MiB per file, bypassed by `exec` (source: https://wiki.teamfortress.com/wiki/Scripting) |
| BindForge `profile/v1` + `chain.rs` | `derives` list, nearest first | none yet (014 adds it) | nearest-first walk, iterative | guard only — semantics are 014's | `Cycle`, `MAX_CHAIN_DEPTH = 32` (source: schemas/profile/v1/spec.md; source: crates/core/src/chain.rs) |

What won by convention, across all of them:

- **Deltas as an ordered list applied base-first**, not a merged blob
  (git replay, quilt series, RFC 6902, systemd drop-in merge)
  (source: https://git-scm.com/docs/git-rebase;
  source: https://savannah.nongnu.org/projects/quilt;
  source: https://datatracker.ietf.org/doc/html/rfc6902;
  source: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html).
- **A patch names the base it modifies** — `parent_set_name`, overlay `prev`,
  quilt's base dir — matching `derives` carrying the relation in the document
  rather than in the tool
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file;
  source: https://wiki.nixos.org/wiki/Overlays).
- **When moved to a new base, stop visibly rather than guess** — git's conflict
  stop, overlays' recursion refusal — support the ADR treating a re-pinned
  base's drift as a surfaced failure
  (source: https://git-scm.com/docs/git-rebase;
  source: https://wiki.nixos.org/wiki/Overlays).

What they later regret, in their own records or visible workarounds:

- RFC 7386 obsoleted by RFC 7396 the same month it became a standard — patch
  semantics published without every edge pinned had to be re-issued
  (source: https://datatracker.ietf.org/doc/html/rfc7396).
- toml-rs shipping stack-overflow DoS and then having to tune the *limit
  itself* twice (source: https://github.com/toml-rs/toml/issues/206;
  source: https://github.com/toml-rs/toml/blob/main/crates/toml_edit/CHANGELOG.md).
- TF2's size cap defeated by exec chains and loops that hang clients
  (source: https://wiki.teamfortress.com/wiki/Scripting).
- Valve needing per-population flags to price a forced base update on players
  who built personal configurations
  (source: https://partner.steamgames.com/doc/features/steam_controller/iga_file).

## Platform notes

- The reference reader is a hand-rolled subset parser in a zero-dependency
  crate, so no third-party TOML parser's CVEs apply to `bindforge-core`
  itself; the ecosystem history above is why its budgets stay, not why they're
  redundant (source: ADR-008, docs/adrs/008-rust-for-the-product.md;
  source: crates/core/Cargo.toml; source: crates/core/src/toml.rs).
- The contract cites a documented subset of TOML v1.1.0, which is the current
  full spec (published 12/18/2025); v1.1.0 admits dotted keys, inline tables,
  multi-line arrays, hex integers, and `\e`/`\xHH` escapes that the contract
  refuses — a third-party reader built on a standard 1.1 parser accepts more
  than the contract, and each refusal family is pinned per format by an invalid
  fixture, so `profile/v2` starts with its own corpus of these
  (source: https://toml.io/en/v1.1.0; source: schemas/README.md;
  source: schemas/profile/v1/spec.md).
- Two `profile` majors live at once by decision; the shipped mechanism for
  that is the per-format entry set chosen by declared version plus
  `check_dir_version`, so `profile/v2` documents never fall through to v1
  parsing (source: crates/core/src/document.rs).
- Fixture-name case-uniqueness (for case-insensitive contributor filesystems)
  is a harness rule today; if a personal chain is later stored as one file per
  profile id, the same concern applies to profile ids as path components —
  relevant because the ADR leaves storage location open
  (source: crates/core/tests/conformance.rs; source: docs/adrs/014-profiles-as-chains.md).
- Nothing in the pipeline/TS side is implicated: everything user-facing that
  reads profiles is Rust, so v2 has one reference implementation to keep the
  fixtures honest (source: docs/adrs/008-rust-for-the-product.md).

## Risks & unknowns

- **"The 32-deep bound and cycle refusal already hold against hostile
  uploads" is true only per document.** `resolve_chain` validates one
  document's `derives` array; resolving a chain *across files* — loading each
  ancestor by id — is new v2 machinery with no shipped cross-file guard, and
  a cross-document cycle is not the same object the `Cycle` fixture pins. The
  loader needs the same "walk iteratively, refuse before descending" rule at
  the join, and the fixture corpus a location rule for ancestors
  (source: crates/core/src/chain.rs;
  source: schemas/profile/v1/fixtures/invalid/self-deriving-chain.toml;
  source: docs/adrs/014-profiles-as-chains.md).
- **Mixed-major chains are undefined.** `derives` carries ids only, with no
  version; a v2 patch may name a v1 profile as its base (v1 documents have no
  `kind`, no deltas — is a v1 profile a base by default, or are v2 chains
  v2-only?). The per-format version gate guarantees each *document* is read
  with its own grammar, but says nothing about the chain spanning majors
  (source: crates/core/src/document.rs; source: schemas/profile/v1/spec.md).
- **The delta lists must fit a grammar that forbids tables.** `set` is a
  mapping action→input; the only shipped encoding for one is the space-joined
  pair, and `swap` adds a pair type the ADR does not disambiguate (inputs ↔
  inputs? actions ↔ actions? one of each?). Every encoding choice inherits the
  closed escape table and needs wrong-field-count fixtures
  (source: crates/core/src/document.rs; source: docs/adrs/014-profiles-as-chains.md).
- **`kind` vs `derives` redundancy is unaddressed.** A `patch` without
  `derives`, or a `base` that is second in a chain, are representable and
  contradict each other; the contract's pattern is a refusal naming the
  contradiction, but the ADR lists no such rule
  (source: docs/adrs/014-profiles-as-chains.md).
- **Determinism extends to array order, and order is already data elsewhere.**
  Manifest serialize "keeps document order: row, group, and member order is
  data"; if v2 delta arrays are ordered data (they must be, given sequential
  application), round-trip equality depends on preserving order through
  `serialize`, and two documents differing only in `set` order are *not*
  semantically equal — a fixture class with no precedent yet for ordered
  mappings (source: crates/core/src/document.rs).
- **Composite resource budget is unstated.** 32 links × 1 MiB, plus resolved
  layout size per link: the per-document caps exist, the chain total does not;
  TF2 shows per-file caps getting worked around the moment files can reference
  files (source: schemas/README.md; source: crates/core/src/chain.rs;
  source: https://wiki.teamfortress.com/wiki/Scripting).
- **The re-pinned-base drift has no named failure mode.** The ADR prices it as
  a cost ("a re-pinned base can silently change what 'the same' patch means")
  but names no detection; CI re-validates dependents on manifest bumps (013),
  which covers manifest drift but not base-profile drift inside a chain, and
  git's precedent says a moved patch either applies cleanly or stops visibly —
  BindForge's semantic deltas need to pick which
  (source: docs/adrs/014-profiles-as-chains.md; source: docs/adrs/013-game-manifests.md;
  source: https://git-scm.com/docs/git-rebase).
- **Provenance strings are document-controlled text.** `seeded-from` and
  `generated-by` flow the same escape-and-truncate path as every other echoed
  value whenever they appear in a rejection; and `seeded-from` names a preset
  id whose stability across game patches is a 013 open question 014 inherits
  (source: crates/core/src/error.rs; source: docs/adrs/013-game-manifests.md).
- **`diff`/`fork`/`merge` correctness is unbounded work parked outside the
  ADR.** The merge of "two patches hanging off one base" is a three-way merge
  over an ordered delta list, with no conflict rule defined; git's is a hard
  stop on first conflict, RFC 6902's is abort-the-whole-patch — neither maps
  1:1 onto a semantic action→input delta that collides
  (source: docs/adrs/014-profiles-as-chains.md;
  source: https://git-scm.com/docs/git-rebase;
  source: https://datatracker.ietf.org/doc/html/rfc6902).
- **The v2 invalid corpus owes at least:** unknown-entry under v2 grammar,
  `kind` missing or outside `{base,patch}`, `for` malformed or `@0`,
  a `patch` with empty `derives`, a wrong-field-count `set`/`swap` element,
  an input id outside the closed vocabulary, and chain cases at the boundary —
  on top of re-issuing v1's whole refusal family against the v2 grammar
  (source: schemas/profile/v1/spec.md; source: schemas/manifests/v1/fixtures/invalid/revision-zero.toml;
  source: schemas/reach-classes/v2/spec.md).

## Open questions

- What exactly does `MAX_CHAIN_DEPTH = 32` bound once ancestors resolve across
  files — one document's `derives` length (as today), or the resolved chain's
  total links — and where does the cross-file `Cycle`/`ChainTooDeep` refusal
  live: in `resolve_chain`, or a new `load_chain` the way the manifest's
  cross-file rules live in `load_manifest`?
- Where do ancestor documents come from when a chain is resolved: bundled in
  one upload, looked up in `data/profiles/`, fetched from the hub? The fixture
  harness parses single documents; a chain corpus needs a location rule, and
  the ADR defers personal-chain storage entirely.
- May a v2 profile's chain include a v1 profile, and how is a v1 document's
  `kind` decided (absent → `base`)? If not, does `derives` need to carry the
  major, or does resolution just read each file's own `version`?
- What is the exact encoding of `set`, `clear`, `swap` elements in the
  one-line grammar — `"<action-id> <input-id>"` for both `set` and `swap`
  pairs? For `swap`: exchange two inputs' actions, or two actions' inputs, or
  is each pair type spelled? What collision rules apply within one patch
  (`set` twice, `set` + `swap`, `clear` + `set`)?
- Are delta arrays ordered data (order-insensitive equality vs order-
  significant resolution)? Which order does `serialize` write, and does a
  `fork` copy renumber or preserve it?
- What does `for` pin exactly — `"<manifest-id>@<revision>"` with the shipped
  manifest grammar — and what happens when resolution finds the profile's `for`
  disagrees with the manifest the chain is being applied to: refusal, warning,
  or a resolver option?
- Is `kind` orthogonal to position — is a profile with `kind = "base"` legal
  below another link (015 says transforms sit below a personal patch, which
  reads as a transform output *becoming* a base mid-chain — does that produce
  a new document with a new id)?
- What is a patch's failure behavior when an ancestor id in `derives` cannot
  be found at all — a missing link is not a cycle or a depth breach, and
  neither `Cycle` nor `ChainTooDeep` names it.
- What conflict rule does `merge` define for two patches over one base: last
  patch in order wins, a refusal naming the colliding action, or a produced
  conflict marker a human resolves in the file (git's stop model vs RFC
  6902's abort model)?
- Does `diff` output share the v2 grammar (a patch document is exactly a
  diff), and must `diff` + `apply` round-trip under the harness's equality
  rules — i.e. is "apply(apply(base, d), d') = apply(base, merge(d, d'))"
  part of the compatibility surface the ADR opens?
