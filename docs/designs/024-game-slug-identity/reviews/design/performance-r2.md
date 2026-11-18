# Performance review r2

verdict: approve

Scope: `reviews/design/editorial-r2.diff` — the single editorial confirmation
pass of round 2. This seat's previous findings: none (editorial phase is one
language pass per round). Ratification rule applied: report only what the
revisions broke or made false.

## Findings

None. Every hunk in the diff is language-only: sentence resegmentation in
Context and Goals, `install-dir` → `install-directory`, "nothing is
transliterated multi-character" → "nothing maps to more than one character",
"as an un-derivable name" → "used for an un-derivable name", "naming the
renamed" → "and names the renamed", "the survey cases reproduce" → "are
reproduced", backticks around `024-L1`/`024-L3`, "Resolver" → "A resolver",
and the private-constant bullet's resegmentation in Shared grammar and bound.
No hunk alters a performance-bearing passage or its meaning. The passages that
carry the design's performance shape are either untouched by the diff or
reworded with identical semantics:

- Emission, refusal and collisions: "run once per plugin over all rows that
  plugin's `discover()` collected", "keeping an assigned-slug map", and
  "skipping candidates already claimed" are unchanged; the count-only
  plugin-side refusal wording is resegmented only.
- Options, "Collision suffix vs the length bound": the chosen rule "accept
  the first candidate that is unclaimed and still within the bound" is intact;
  only the refusal-path phrasing moved.
- Risks row "Collision-suffix probe loop repeats bound checks many times |
  Low | Low" and its baseline citation are outside every hunk.
- Non-goals "No persistence or performance-budget work beyond what today's
  code already counts" is outside every hunk.

Nothing this seat approved in round 1 is broken; no cited section is now
false.

## Checklist disposition

1. Input-path latency budget — not applicable at this scope: derivation and
   the collision pass run per plugin discovery run, not on a keystroke/event
   path, and no diff hunk adds allocation, table rebuild, or synchronous IO
   on such a path.
2. O(n²) over unbounded n — not changed by the diff: the per-plugin
   assigned-slug map and already-claimed-candidate skip are byte-identical
   semantics after the edit; probe count stays bounded by the per-run
   distinct-name count (Risks row, untouched).
3. Memory growth — no new structure in the diff; the assigned-slug map
   remains bounded by one plugin run's rows.
4. Startup cost — not affected; no file-read or schema-parse passage in the
   diff.
5. Caching — not applicable; the design ships no cache, and the diff adds
   none.
6. Research baseline gap — no number changed in the diff; the design still
   states the baseline gap explicitly where it claims cost ("01-research.md
   Risks: no baseline exists; none claimed"), which is the explanation the
   checklist requires, not a gap.

## FYI (no severity)

- None for other seats; the diff's most substantive-looking edit ("nothing
  maps to more than one character") restates the Latin-fold rule in the same
  direction the Options section already pinned.
