# security review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) against
`02-proposal.md`, checked with the security checklist. Inputs: the proposal,
`docs/adrs/013-game-manifests.md`, the Revision log (r1 entries only — no
security decisions recorded there), and the diff. No previous security
findings exist for this run, so there is nothing to re-check for regression
from a prior verdict.

## Findings

None. Every hunk in the diff is language-only; no security-bearing text
changed meaning:

1. **Goals bullet (row encoding)** — reordering of "unambiguous inside the
   one-line string-array grammar". The grammar restriction (no tables,
   no evaluation) that keeps manifest parsing data-only is untouched.
2. **Options → action rows** — comma→semicolon and `honored`→`honoured`.
   The one-fact-per-document posture (registry as single source for axis
   kind), which is what closes the drift channel, survives verbatim.
3. **Options → manifest identity** — added "use" for parallelism. The slug
   grammar (`[a-z][a-z0-9-]*`) that makes a hostile `id` incapable of
   carrying `..` or an absolute path is untouched.
4. **Example note** — comma removed before "and Deferred". No change.
5. **Load-time validation → Registry join** — "must be `registry.contains`"
   → "must pass `registry.contains`; one that does not is refused, naming
   the id". This *sharpens* the fail-closed rule: the join is now an
   explicit check that every taxonomy id must pass, with the refusal
   naming the id preserved. Semantics identical to the intended reading of
   the old (ungrammatical) text; nothing weakened.
6. **What the series reads from here** — "in any `exclusive_groups`
   member" → "as any `exclusive_groups` member". The precondition refusal
   set (dead key, group member, docked input side) is unchanged.
7. **Test strategy #4** — "files a" → "reports a" refusal. The load-time
   refusal test coverage is unchanged.
8. **Test strategy #7** — `behavior`→`behaviour`. No change.

Checklist evaluation over the text the diff touches (and the sections it
leaves intact):

1. Trust boundaries — A14 keeps every `data/manifests/` file untrusted; the
   ADR's untrusted-posture clause (Context, last paragraph) is reflected
   unchanged. Pass.
2. Parsing untrusted data — closed grammar, closed entry set, `MAX_DOCUMENT_BYTES`
   1 MiB / 16-deep budgets binding first, four new named caps, base64 body
   pinned to exactly 32 octets, no code evaluation anywhere in the design.
   Pass.
3. Path handling — `id`/`codec`/preset slugs are lowercase-ASCII grammar, so
   user-controlled strings cannot smuggle `..` or absolute paths; the
   location rule compares the directory segment to `id` and the refusal
   never echoes a local path (`source_path` channel). No shell
   interpolation anywhere. Pass.
4. Secrets/PII — no secrets in the format; the fingerprint is provenance
   data with grammar-only semantics; echoed identifiers go through the
   escape-and-truncate error path. Pass.
5. Dependencies — the design names no third-party dependency (std-only,
   `Cargo.lock` byte-unchanged); nothing to advisories-check. Pass.
6. Privileges — file read only, within the existing parse path; least
   privilege. Pass.
7. Fail closed — unknown entries, unknown format (`UnknownFormat`),
   unregistered ids, budget overruns, and grammar violations are all clean
   refusals (`InvalidValue`/`MissingEntry`/version gate); partial application
   is not a path. Pass.

Deferred items (fingerprint transport / manifest trust over a channel →
ADR-018; fingerprint semantics → ADR-017) are consistent with ADR-013's
"Not decided here" and Decision clauses; none is re-opened.

## FYI (no severity)

- The diff is purely orthographic/syntactic (US→UK spelling, verb choice,
  punctuation); the substantive fix — "must be `registry.contains`" →
  "must pass `registry.contains`" — belongs to the editorial seat's
  language scope and happens to improve clarity of a fail-closed rule.
  No security action needed.
