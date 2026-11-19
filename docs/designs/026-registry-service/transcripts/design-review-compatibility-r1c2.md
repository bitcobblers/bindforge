# Transcript: design-review:compatibility r1c2

## Provenance
- stage: design-review:compatibility (round r1c2)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T19:18:28.033Z
- ended: 2026-11-19T19:25:07.038Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":78855,"output":25102,"reasoning":0,"cache.read":2495587,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/026-registry-service/02-proposal.md.
This is the dissent pass c2: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 9ff2ec7839f444005fc7df06fbad447c26f078c6 07e8b3f8b64d6421c1f257e00aeeb876f54423ca` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/026-registry-service.md,
the Revision log inside the proposal, and your own previous findings quoted
below. Do not open review reports or transcripts, and do not ask for them:
the panel's value is five independent reads of the same text, and another
seat's verdict anchors yours. Reports and transcripts are the run's record,
not review input.
Listing, globbing, or grepping those directories counts as opening them.
Every path you need is named in this prompt — never browse to verify one, in
the sense that sentence prohibits: walking a directory to locate a path, or to
check that a named path exists. That bar is on finding paths; it is not a bar on
running checks. What is open to you instead: the project's read-only check
commands — the cargo, rustfmt and npm legs, with every write leg denied — to
verify what the design claims about the code, and the report at docs/designs/026-registry-service/reviews/design/compatibility-r1c2.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "major",
    "location": "§The serving face step 1 (\"after the first `/`, exactly one non-empty `.`-free component, no embedded `/`\") + §Catalog document and its spec (\"a `source_key` failing `spells_lookup_key` (A16) or the tightened key grammar of §The serving face step 1\") + §Budgets (\"any `slug` failing §The serving face step 1's validator by name\")",
    "problem": "The new validator's dot ban makes the registry's admitted lookup-key vocabulary narrower than every shipped spelling rule for the same keys: Source::spells_lookup_key (crates/discovery/src/game.rs:84-89) admits any non-empty external id after a closed-set token, Source::lookup_key (game.rs:66-68) — the single composer, Source::ALL including Source::Local — imposes nothing, and the shipped store validates a slug as exactly \"len ≤ MAX_DISCOVERY_SLUG_OCTETS + spells_lookup_key\" (apps/discovery-cli/src/matches.rs:222-234). So `local/Steam.SE` is a legal slug the shipped client stores and sends as `slug=`, and this revision refuses it at four seams at once: candidate query (404), admin upsert (refused by name — closing ADR-026's own local/<dir> curation path: \"a `local/<dir>` install has a lookup key\", \"giving `local/`-source installs a curation path no public root can own\"), corpus load (a curated corpus can never carry such a key), and cache load (one dotted slug refuses the whole cache d… [elided by the checkpoint cap]",
    "fix": "In §The serving face step 1 replace \"exactly one non-empty `.`-free component, no embedded `/`\" with \"exactly one non-empty component with no embedded `/`\" and move the dot ban to where its precedent lives — one clause naming is_slug_name/A4 as the game_id-only rule; carry the same substitution into the §Catalog document refusal list and the §Budgets cache-load clause, and re-word #3 and #4 so a dotted `local/…` key is an admitted spelling rather than a 404/refusal. If the ban is wanted, keep it deliberately: say so there, name it as a narrowing of 026's `local/<dir>` use case, and name which task refuses dotted local keys loudly — replacing the path-safety rationale the paragraph already disclaims."
  },
  {
    "severity": "minor",
    "location": "§Catalog document and its spec — \"`crates/core/tests/conformance.rs`: guard the fixture-tree loops — `every_valid_fixture_…`, both `every_invalid_fixture_…` message tests, and `fixture_file_names_…` (which keeps governing the `.json` fixtures: lowercase dash-separated names, case-unique) — behind the core-TOML check\"",
    "problem": "The sentence says both that fixture_file_names_… is moved behind the core-TOML check (so it does not run for catalog) and that it \"keeps governing the `.json` fixtures\"; the instruction is undecidable as written. The property at stake is a platform one: fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique (crates/core/tests/conformance.rs:336-376) walks format_dirs() — the same walk that picks catalog/v1 up automatically — and its doc-comment names what it buys: names \"case-unique within a fixture directory, which is what keeps the same corpus green on a contributor's case-insensitive Windows filesystem as on the Linux CI runner\". That is the only coverage behind the Windows `Yes` row's corpus claim and Test #19's \"case-unique fixture names\". Mechanically related: is_fixture_name takes a bare file name (crates/core/tests/common/mod.rs:421-435), so \"accepts `.json` for that directory only\" already requires the predicate to learn the directory, and whether this test is g… [elided by the checkpoint cap]",
    "fix": "Drop `fixture_file_names_…` from the guarded list instead of guarding it with a parenthetical that reverses the guard, and state the reason in the common/mod.rs bullet: \"`is_fixture_name` takes the format name (or a per-directory suffix set) so the unguarded `fixture_file_names_…` walk keeps its lowercase/`-`-separated and case-uniqueness rule over `catalog/v1`'s `.json` corpus — the Windows case-insensitive-filesystem claim's only coverage.\" Text replacement only; no new mechanism."
  },
  {
    "severity": "minor",
    "location": "§Compatibility matrix — \"| Linux & other Unix | Yes |\" plus the new footer \"the three `Yes` rows (Linux & other-Unix CI, macOS, Windows) are the rows the tasks owe behavior to\", against \"| Other / exotic Unix (BSD et al.) | No | Not a v1 target … nothing exercises or promises it.\"",
    "problem": "The r1 fix decided the fourth row but landed on a label that overlaps the first: \"other Unix\" is simultaneously a Yes the tasks owe behavior to and \"not a v1 target … nothing promises it\". A T-005…T-008 author cannot tell whether a BSD build is owed behavior or explicitly refused — the same failure mode as an undecided row under two names.",
    "fix": "Rename the first row's label to the platform it actually promises — \"Linux\" (its Notes already say \"Primary CI target\") — and the footer's parenthetical to \"(Linux CI, macOS, Windows)\"; if the design means to promise the generic-Unix data-dir branch beyond BSD, give it its own row with its own status instead of hiding it inside row 1's \"other Unix\". Labels only; #19 already names the three supported platforms and needs no change."
  },
  {
    "severity": "minor",
    "location": "§Catalog document and its spec — \"`crates/discovery/tests/boundary_tests.rs`: … so re-scope the sweep to `data/manifests` + `data/profiles`, with its doc-comment and `checked > 0` guard re-pinned\"",
    "problem": "The amendment removes coverage nobody asked it to remove. Shipped shape (crates/discovery/tests/boundary_tests.rs:110-156): the sweep reads every file under `data/` and every `schemas/<format>/<major>/fixtures/valid/`, and its doc-comment pins \"no source key or friendly name reaches a manifest, a profile, or a path\" against exactly those trees. 026-S1 requires only that `data/catalog/` and `schemas/catalog/v1/` be exempt; re-scoping to the two data/ directories also drops `schemas/manifests/v1/` and `schemas/profile/v{1,2}/` valid fixtures — the trees the pinned rule names — from the gate, permanently.",
    "fix": "Replace the re-scope with a carve-out that moves coverage only where 026-S1 forces it: \"keep both current roots; skip files under `data/catalog/` and `schemas/catalog/`; amend the doc-comment to name `catalog` as the one tree where a lookup key is legal content\". The `checked > 0` guard then needs no re-pinning (data/manifests/ and data/profiles/ are counted either way)."
  },
  {
    "severity": "nit",
    "location": "§`apps/registry` CLI surface (\"relocated in tests through the same env seam as #13\") + Test #13 (\"this seam is how caching tests relocate the local-entries document (no dedicated CLI knob)\")",
    "problem": "Dropping `--local-entries <path>` leaves the copied `<data>` rule's env inputs as the only relocation lever, and the design names the seam (the in-process StoreLayout/data_dir env closure, apps/discovery-cli/src/matches.rs:46-103) without naming the variables, while #17's caching end-to-end runs `registry serve --upstream` as a subprocess that writes both catalog-v1.json and cache-v1.json.",
    "fix": "One clause in #13: \"for the subprocess cases (#17), set `XDG_DATA_HOME` (Unix) / `APPDATA` (Windows) in the child environment; there is no path knob\" — so the task author doesn't reintroduce a flag to compensate."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/026-registry-service.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/026-registry-service/reviews/design/compatibility-r1c2.md. That file is the only path this session
may write: another seat's report is the run's record, not your surface, and the
runner inspects what your session actually did and fails the attempt on any
other write.
Reply with the JSON verdict object only.

Finish ALL your work with tool calls before replying. Do not describe what
you are going to do; just do it. Never end a turn except by replying with
the JSON object below: if work remains, end the turn only with further
tool calls, never with prose. Your final message must contain exactly one
JSON object and nothing else: no markdown fences, no prose, no preamble.
JSON Schema: {"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"verdict":{"type":"string","enum":["approve","request-changes"]},"findings":{"type":"array","items":{"type":"object","properties":{"severity":{"type":"string","enum":["blocker","major","minor","nit"]},"location":{"type":"string","minLength":1},"problem":{"type":"string","minLength":1},"fix":{"type":"string"}},"required":["severity","location","problem"],"additionalProperties":false}}},"required":["verdict","findings"],"additionalProperties":false}
```

## Raw reply
```text
{
  "verdict": "approve",
  "findings": [
    {
      "severity": "minor",
      "location": "§The serving face step 1 — \"Path safety is the only extra refusal, at its own scope: after the first `/`, one non-empty component with no further `/` or `\\`, none equal to `.` or `..`, and no control byte, space, or invalid-UTF-8 sequence\" (carried into §Catalog document and its spec, §Budgets' cache-load clause, and #1/#3/#4)",
      "problem": "re-raised (narrowed from r1's major; the dot case is closed). The space ban is not a path-safety refusal, and the same paragraph's parenthetical says lookup keys are \"opaque in-memory strings, never joined onto paths\" — yet the sentence above it claims \"that shipped pair is the registry's whole key rule\". Shipped evidence: Source::spells_lookup_key (crates/discovery/src/game.rs:84-89) admits any non-empty external id after a closed-set token; Source::lookup_key (game.rs:66-68), the single composer, imposes nothing further; the shipped store's only slug checks are the 025-B1 cap plus spells_lookup_key (apps/discovery-cli/src/matches.rs:222-234); and the shipped client puts such a key on the wire, escaping a space as %20 (catalog.rs:174-185, A19). So a space-bearing 025-C1 key — the local/<dir> external id, whose only stated derivation law is \"the kebab'd install directory name\" with no grammar — is refused at all four seams this rule governs: candidate query (404), admin upsert (refused by name), corpus load, and cache load (one such slug refuses the whole cache document). The document is also internally split: #3 admits the non-kebab'd `local/Steam.SE` (lenient reading of the derivation) while this clause refuses a space (strict reading), so T-005 cannot tell which vocabulary it is pinning. Blast radius is much smaller than r1's dot case — no shipped emitter produces a space today — hence minor rather than major.",
      "fix": "Delete `space` from the banned set, leaving \"no control byte or invalid-UTF-8 sequence\", so the extra refusals are exactly the named path-safety and line-integrity classes; nothing else changes because every dependent seam already references this one clause. If a space ban is wanted on some other ground, keep it but state that ground here and name it as a narrowing of the 025-C1/026-C3 key vocabulary that no shipped spelling rule shares."
    }
  ]
}
```
