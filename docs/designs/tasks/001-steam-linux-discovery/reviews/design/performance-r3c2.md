# performance review r3c2 (ratification)

verdict: approve

Scope: `git diff b08c55894c68e616e5536c73cc7a9921db842309 61907ce31bc8cb18f436caf4fcf96f29ea9c488d` (the
changed passages of `02-proposal.md`) plus the full current document, checked only against what those
revisions could break in what this seat approved. Allowed inputs honoured: `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the Revision log, this seat's quoted r3 findings. No review
report or transcript was opened. `01-research.md` was not in the allowed inputs, so the
"no scan-latency baseline exists" quote (01-research.md:319-324) is taken as stated, not re-verified.

## Disposition of this seat's r3 findings

- Cross-root library de-duping (r3 minor) — **closed.** §2 now reads "the library paths of all selected
  roots as one set per `discover()` call … so no library is ever scanned twice, including by two roots
  sharing one". The union-across-roots construction I asked for is the text now, so the duplicate
  read+parse and the duplicated `(slug, install_path)` row for one external library listed in two
  coexisting selected roots can no longer occur, and the duplicated case also cannot reach §5's
  "identical slugs from two libraries stay two rows" path as a byte-identical row.
- 1 MiB cap pinned at IO (r3 minor) — **not closed; re-raised below, still minor.** The revision rewrote
  the sentence to cite the repo's shared parse budget, which is a real improvement in grounding —
  I verified it: `schemas/reach-classes/v1/spec.md` ("documents larger than 1 MiB or nested deeper than 16
  levels are rejected before parsing") and `crates/discovery/Cargo.toml` carries only `serde`, so the
  "takes no `bindforge-core` edge" parenthetical is true. The enforcement position I asked for is still
  absent from the design's own words.

No revision in this diff made a section this seat cited false, and none reintroduced a per-scan cost that
this seat had closed.

## Findings

- [minor][re-raised] Chosen design §4 KeyValues reader — "Files larger than 1 MiB are skipped as
  malformed; both caps restate the repo's shared parse budget (`schemas/reach-classes/v1/spec.md`) as
  plugin-local constants, since `crates/discovery` takes no `bindforge-core` edge (A16)." — the citation
  now imports the *numbers* of the shared budget while leaving behind the one clause of that budget that
  produces the performance property this design offers instead of a wall-clock budget. The page cited says
  only "rejected before parsing", which an implementer can satisfy by `fs::read_to_string` then testing
  the length; the pre-read requirement is in `schemas/README.md` (*Input budget and chain bound*): "The
  cap binds *before* the read, not after it: a reader checks a file's length … instead of allocating the
  whole body and then refusing it", applied there by `bindforge-core::parse_file`/`parse_reader` — exactly
  the helpers §4 says `crates/discovery` will not use. Since §4 also says "the caller reads bytes", the
  read-then-check order this finding flagged in r3 remains the natural reading, so per-file boundedness is
  still not guaranteed by the text. Fix: one clause in §4 — "a file whose `fs::metadata` len exceeds the
  plugin-local 1 MiB constant is skipped without being opened" — and cite `schemas/README.md`'s pre-read
  clause for the position, replacing the unspecified read-then-check order rather than adding a mechanism.
- [nit][new] Chosen design §2 Canonical path rule / §3 Library selection within a root — §2: "the library
  paths of all selected roots as one set per `discover()` call"; §3: "`libraries_in_root(root)` returns,
  through the canonical path rule" — the canonical statement's subject is now a cross-root set assembled
  per scan, but the same phrase is still used for the public per-root function, which by signature cannot
  build that union; the text no longer says which step performs the cross-root de-dupe. Fix: in §3, say
  `libraries_in_root` returns its own root's paths canonicalized and de-duped, and name `discover()`'s
  scan loop as the site that takes the union of those sets through the same sort/de-dupe step — replacing
  §3's bare "through the canonical path rule" reference.

## Checklist disposition

1. Input-path latency: N/A — no keystroke/event path here; the plugin runs once per `discovery list`.
   Scan cost is one stat per fixed candidate, one VDF read per selected root, one directory read plus one
   capped read per manifest per selected library; no synchronous IO is added anywhere else.
2. Structure costs: no O(n²). Roots, libraries-per-root and manifests are each walked once; the only
   cross-set step is the de-dupe union introduced in this diff, which removes repeated scans instead of
   adding them. Output sort is `O(n log n)` on the emitted rows.
3. Memory growth: bounded. No cache or persistence (Non-goals); the plugin holds `Vec<PathBuf>` roots at
   construction, and the r3→c2 change to the per-scan library set does not retain anything across calls.
4. Startup cost: host probe plus capped per-file reads; the design states work is bounded per file and
   that no scan-latency baseline exists to budget against (01-research.md:319-324), and the task's Done-when
   clauses (docs/tasks/T-001-steam-linux-discovery.md:21-25) impose no latency requirement, so the absence
   of a wall-clock budget is not a gap against the source document.
5. Caching/invalidation: N/A — no caching in scope.
6. Baseline gap: no number in the design contradicts a research baseline; the only cost claims are the two
   caps, and the 1 MiB / 16-level pair is now anchored to the repo's shared budget (verified above). The
   open item is the enforcement position, reported as the minor.

## FYI (no severity)

- §3's change from "`<root>/steamapps` itself" to "`<root>` itself … a library is the *parent* of its
  `steamapps/`" also removed a latent double-join with §5's `l.join("steamapps").join("common")`, and
  fixtures/Test #3 were moved to `"0" → ".."` consistently. Correctness/compatibility territory, flagged
  only because it touches the same passage as my nit.
- The A9 rewrite (`std::env::var_os("HOME")` for the deprecated `home_dir()` on the pinned 1.85 floor) and
  the §8 import cleanup are lint/compatibility matters; neither changes scan cost — an unset or empty
  `$HOME` now short-circuits to `Ok(vec![])` with no probing, which is cost-neutral-to-cheaper.
