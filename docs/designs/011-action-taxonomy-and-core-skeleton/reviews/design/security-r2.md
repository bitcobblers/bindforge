# security review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) against
`02-proposal.md`. This is the single editorial confirmation pass; inputs were
the proposal, the ADR, the Revision log, and the diff. Prior security
findings: none.

## Findings

None. Each diff hunk was checked against the security checklist for whether
the rewritten text changes its security meaning:

1. **Trust boundaries** — unchanged by the diff. The r1 closure quoted in the
   Revision log ("only the `registry.toml` shipped in this versioned directory
   is the registry, so a document from elsewhere is parsed as data and never
   consulted for `axis_kind`") survives intact in spec.md Extension and
   withdrawal and in `load_action_registry` ("`dir` is the directory shipped
   with the crate: a document from elsewhere is never loaded here"), and the
   Placeholders and lineage section still classifies data dropped into
   `data/manifests/` and `data/profiles/` as "untrusted input once 013/014
   read it".
2. **Parsing untrusted data** — unchanged. The grammar, closed entry set,
   rejection of unknown entries, and the shared parse budget bound
   ("length bounded by the shared parse budget") are untouched by the diff's
   comma/hyphenation edits.
3. **Path handling** — unchanged. `load_action_registry(dir)` joins a fixed
   `"registry.toml"`; errors stay "single-line, truncating, path-free" and
   Test strategy #6 still refuses "without naming a local path".
4. **Secrets / hostile echo in errors** — the @@ -221 hunk rewrites
   "an offending id echoed escaped to printable characters" into "with an
   offending id echoed, escaped to printable characters". This is grammar
   only: the id is still echoed *and* escaped to printable characters, so the
   injection-into-logs protection is preserved.
5. **Dependencies** — the diff touches no dependency text; the design remains
   std-only with `Cargo.lock` byte-unchanged, so there is no new advisory to
   check.
6. **Privileges** — unchanged; this stage ships no binary and needs no device
   or new filesystem privilege beyond reading the crate's own shipped
   directory.
7. **Fail closed** — unchanged. The rewordings ("adds" for "gains",
   em-dash parenthetical on the ADR-009 module-doc references, Oxford commas)
   do not weaken any rejection rule; every refusal still names a fault.

## FYI (no severity)
- The @@ -261 em-dash rewrite makes "the ones that describe future model
  ownership" an appositive over all listed ADR-009 references rather than a
  restrictive clause; whether every ADR-009 mention in those module docs
  describes future model ownership is a correctness/accuracy question, not a
  security one.
