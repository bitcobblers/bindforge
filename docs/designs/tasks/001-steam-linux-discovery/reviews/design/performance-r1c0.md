# performance review r1

verdict: approve

Scope: whole document (full-panel pass c0). Inputs used: `02-proposal.md`,
`docs/tasks/T-001-steam-linux-discovery.md`, the proposal's Revision log, and read-only inspection of
`crates/discovery/src/{engine,game,plugin,slug,registry,lib}.rs`, `apps/discovery-cli/src/main.rs`, and a
package-count check of `Cargo.lock`. `01-research.md` was not in this pass's input list, so research
page-cites are assessed as quoted by the proposal, not re-read.

## Checklist walk-through

1. **Input-path latency budget / per-event cost** — not applicable. The plugin runs once per
   `discovery list` invocation (`apps/discovery-cli/src/main.rs:35-43`); nothing here sits on a keystroke,
   event, or per-frame path, and there is no allocation, table rebuild, or synchronous IO introduced on
   any hot path. The scan-level wall-clock budget is deferred (§4, *Deferred* "Scan-latency budgets and a
   benchmark harness"); T-001's Decision and Done-when impose no latency requirement, so the deferral is
   consistent with the source document and is not re-opened.
2. **Structure costs, O(n²) over unbounded n** — clean. Work is roots × libraries × manifests with each
   manifest read once: candidate probing is one `canonicalize` + one `is_dir` per fixed candidate (§2), the
   VDF is read at "that one fixed path per root and never searched for" and "nothing below `steamapps/` is
   walked recursively" (§3) — this is the design's strongest perf property, since it is what keeps the
   `compatdata/<appid>/pfx` prefix trees (thousands of files per prefix, A4) out of the syscall count
   entirely. De-duping is sort-then-dedupe (linear), and output ordering is one `sort` by
   `(slug, install_path)`. `normalize_slug` (crates/discovery/src/slug.rs:1-35) is three linear passes per
   game. No quadratic nesting found. One set-construction gap: see finding 1.
3. **Memory growth bounded?** — bounded per file by the 1 MiB / 16-level caps, and the `Section` tree is
   built and dropped per file, so peak memory is one file's tree plus the result vector. The cap's
   enforcement point is unstated: see finding 2.
4. **Startup cost** — fine. `from_host()` adds ~14 `canonicalize`/`is_dir` probes before any file read, and
   the plugin is only constructed inside the `List` arm; no schema or profile parsing is added anywhere.
5. **Caching invalidation** — not applicable. "Persistence, caching, or a performance-budget harness" is a
   stated Non-goal, and `discover_all` calls each plugin exactly once per run
   (crates/discovery/src/engine.rs:20-33).
6. **Research baseline quoted** — the proposal quotes it as absent: "no scan-latency baseline exists to
   budget against (01-research.md:319-324)" (§4), and Revision log "r0: q17 performance budget — no external
   baseline exists, so the budget is per file (1 MiB, 16 levels), not per scan". A16 ("far smaller than a
   1 MiB read cap") is self-declared `unverified`. **Missing evidence recorded**: I could not confirm
   01-research.md:319-324 within this pass's inputs. This is not blocking — with no baseline claimed there
   is no gap between expected numbers and a baseline to flag, and the per-file bound is the design's own
   stated budget, which Rollout step 5 (real-machine capture) will falsify loudly if wrong.

## Findings

- [minor][new] Chosen design §2 "Canonical path rule (canonical statement, referenced below)" — "every set
  of directories the plugin scans — roots from the candidate list, and library paths within a root — is
  built by canonicalizing each entry … then sorting, then de-duplicating … so no library is ever scanned
  twice" — the invariant is only established *within* one set, and the library sets are built per root, so
  nothing ever compares one root's library list against another's. Two distinct canonical roots (native
  `$XDG_DATA_HOME/Steam` plus the Flatpak `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` probed in
  §2) each read their own `libraryfolders.vdf`, and both can name the same library directory — an absolute
  `path` shared across installs, or a Flatpak-side symlink, which canonicalizes to the same target and so
  passes the `steamapps` gate in both sets. That library's `steamapps/` is then `read_dir`ed twice and every
  `appmanifest_*.acf` in it opened and parsed twice, and the result vector carries exact duplicate
  `(slug, install_path)` rows — §5's "Identical slugs from two libraries stay two rows" does not cover
  duplicates from *one* library reached by two roots. The duplicated IO is small (one library's file count),
  hence minor, but the stated "never scanned twice" property cannot be established from the mechanism as
  written. Fix: make the library set a single collection across all selected roots — append every root's
  VDF `path` entries and each root's own `steamapps` into one list and apply the canonical path rule once to
  that union, replacing the clause "and library paths within a root" (equivalently, carry a
  `HashSet<PathBuf>` of already-scanned canonical libraries through the root loop) — and extend Test
  strategy #3 to assert two roots sharing one library yield that library once.
- [minor][new] Chosen design §4 "KeyValues reader" — "Input is `&str`; the caller reads bytes and skips a
  file that is not valid UTF-8 rather than aborting a scan. Files larger than 1 MiB are skipped as malformed
  (A16): the reader's work is bounded per file" — the design states the size bound as a property of the
  reader over `&str`, which is satisfied just as well by reading the whole file and *then* checking its
  length; the text never says the bound is applied before the bytes are read. Same for file type: a FIFO or
  device node named `appmanifest_*.acf` sitting directly in a library's `steamapps/` (a symlink to such
  counts) would block the read indefinitely, and a multi-gigabyte regular file matching the glob would be
  allocated in full before the cap rejects it. So the per-file bound the §4 rationale rests on, and the
  "bounded work per file" claim in the Risks row ("Both caps are single named constants"), are not
  established by the specified sequence. Fix: state that the file-open path gates on `fs::metadata` first —
  skip unless `is_file()` and `len() <= 1 MiB` — so the 1 MiB cap replaces the post-read length check
  implied by "Files larger than 1 MiB are skipped as malformed", and count that skip in the §7 skipped-file
  tally.
- [nit][new] Chosen design §4 — "the caller reads bytes and skips a file that is not valid UTF-8" — read as
  `fs::read` followed by UTF-8 validation, that is one full-file `Vec<u8>` allocation plus a second full
  copy into a `String`, once per manifest in every library, where `fs::read_to_string` reads into the
  `String` and validates in place. Fix: name `fs::read_to_string` (mapping its `InvalidData` arm to the
  existing skip) as the read call, so the wording does not prescribe the extra copy.

## FYI (no severity)

- Correctness seat, §3 + §5: `libraries_in_root` returns "`<root>/steamapps` itself, always" as a *library*,
  but §5 composes `install_path = l.join("steamapps").join("common").join(m.install_dir)` — for the
  root-as-library case that yields `<root>/steamapps/steamapps/common/<installdir>`, while Test strategy #6
  ("`install_path` ends with `steamapps/common/<installdir>`") and #8 (four games from `minimal/`, whose
  manifests live in `minimal/steamapps/`) assume the single-`steamapps` form. The two statements cannot both
  hold; whichever way it is resolved, the root-as-library row's emitted path is the one at stake.
- Correctness/compatibility seat, Compatibility matrix "Linux, Flatpak Steam — Partial": the gate it cites
  drops paths that "exist only inside the sandbox", but Flatpak rewrites `$HOME` to
  `~/.var/app/com.valvesoftware.Steam`, so absolute `path` values written by a Flatpak client are typically
  host-visible under the user's home and pass the `steamapps` gate rather than being dropped — which is also
  the concrete route to finding 1's cross-root duplicate.
- Verified against the tree, no action: A15's "32 `[[package]]` entries" in `Cargo.lock` holds;
  `DiscoveryError::PluginError(String)` exists (crates/discovery/src/registry.rs:2-5); the engine does
  swallow a plugin `Err` into one `eprintln!` and return `Ok` (crates/discovery/src/engine.rs:23-30, A14);
  `lib.rs` exports the existing five modules; `DummyPlugin`'s only user is the `List` arm
  (apps/discovery-cli/src/main.rs:25-41, A17).
