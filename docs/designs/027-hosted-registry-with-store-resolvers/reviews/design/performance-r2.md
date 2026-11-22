# Performance review r2

verdict: approve

Scope: the editorial r2 diff (`reviews/design/editorial-r2.diff`) — the single
editorial confirmation pass of round 2, performance seat. Prior findings from
this seat: none (the editorial phase is a language-only pass). Ratification
standard applied: report only what the changed text breaks or renders false.

## Diff-by-diff check against the performance checklist

- `bookkeeping ensuring that deleted ids never return` (Goals) — grammar
  only; the `027-L2` bookkeeping mechanism is untouched.
- `named-refusal family` reflow, `New flag:` singular, two comma insertions
  (query startup, health facts) — punctuation/hyphenation only. The startup
  cost claims (`same startup as serve, including load-or-pull, no refresh
  thread`; key required regardless of snapshot freshness) survive verbatim.
- `an overlap only re-fetches an item the appid merge absorbs` — restates
  the delta-overlap cost claim with a noun instead of a gerund; the
  freshness/`if_modified_since` arithmetic (`fetched_at` read before the
  first page) is unchanged. Checklist items 5 (cache invalidation) and the
  r2 wall-clock revision entry stay true.
- `tmp-and-rename, then publish` — comma only; refresh-thread rhythm
  (86,400 s pull, `STEAM_RETRY_SECONDS` gap, keep-old-on-failure) unchanged.
- `In memory it is an ordered insert with membership by binary search` —
  article insertion only; the retired-list structure cost (ordered insert +
  binary search, two bounded rewrites per delete) is exactly what this seat
  accepted at r2.
- `a delta merge holds the old snapshot, the merged model, one serialize
  buffer and the current page body` — supplies the missing verb the r1
  sentence lacked; the transient-memory accounting this seat's r2 revision
  entry demanded (largest transient counted, one linear two-way merge plus
  one index build) is stated identically. Meaning preserved, grammar fixed.
- Mint-write-path and TLS-test comma insertions, `retirement refusal`
  de-hyphenation — grammar only.
- Rollout 4: `the issued-document rewrite measured at 100k and 200k records,
  recorded in §Budgets` — re-attached participle; the measurement obligation
  §Budgets already carries ("Rollout 4 records the measured issued-document
  rewrite at 100k and 200k records beside that figure") is unchanged, so the
  batched-writes deferral still defers against a number.
- Deferred: `leaves outside its ledger` — verb fix matching the ADR's
  Consequences ("a separate small amendment ... outside this ADR's ledger").

No changed hunk alters a latency budget, a structure cost, a memory bound, a
startup cost, an invalidation rule, or the estimate-vs-baseline posture
(A5's 164k–200k band against the 600k-slot page budget). The diff is
language-only; nothing the performance seat approved was broken and no cited
section was made false.

## Findings

None.

## FYI (no severity)

- The `New flag: --no-steam` singular is locally correct (it is the only new
  flag; `--upstream`/`--refresh-interval` survive only as refusals), though
  the Goals bullet and `027-C1` say "resolver flags" — an editorial
  number-agreement observation, not a performance one.

Checklist items 1–6: all checked against the diff; none implicated by the
changed text.
