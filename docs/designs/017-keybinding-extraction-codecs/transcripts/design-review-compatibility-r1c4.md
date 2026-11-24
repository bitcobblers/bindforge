# Transcript: design-review:compatibility r1c4

## Provenance
- stage: design-review:compatibility (round r1c4)
- agent: review/design-compatibility
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:55:56.278Z
- ended: 2026-11-24T05:08:15.419Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":54227,"output":27860,"reasoning":0,"cache.read":902289,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the ratification pass c4: revisions landed after you approved; you are checking only that they did not break what you approved of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff a4dda6a8542694895102f4a9460835e189e4f08b 316ef46c5517497b633f76f56fa0ffa689babebf` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/017-keybinding-extraction-codecs.md,
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/compatibility-r1c4.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (compatibility seat):
[
  {
    "severity": "minor",
    "location": "§Discovery plumbing + §GameContext — \"`roots()` (the selected root candidates, in candidate order)\" / \"`pub roots: Vec<PathBuf>,  // every root the scan resolved, candidate order`\"",
    "problem": "[new] The order is false of the field the inherent wrapper would expose: `SteamCore` holds its roots through `canonical_paths` (crates/discovery/src/steam_common.rs:369-381), which canonicalizes (resolving symlink aliases such as `~/.steam/root`), drops entries that fail to canonicalize or lack a `steamapps/` directory, and returns a byte-sorted, de-duplicated set — not the candidate list. Test strategy #3 already says \"the byte-order union over `ctx.roots`\", so the design contradicts itself about the order an implementer must produce.",
    "fix": "In both places state \"the canonicalized, de-duplicated roots that actually hold a `steamapps/`, in byte order (the set `SteamCore` retains, steam_common.rs:369-381)\" and drop \"candidate order\"; keep the existence-filter half as the reason \"a tree is used only if it exists\" is already true."
  },
  {
    "severity": "minor",
    "location": "Compatibility matrix — \"Flatpak / Snap Steam | Yes, for the root-relative trees\" and \"Steam Deck | Yes, by the Linux row\"",
    "problem": "[new] Both rows were upgraded to Yes this round and both rest on the new `roots()` seam, which no test reaches: #1 pins `ScanRecord.root`/`library`, #2 pins only that `from_record` carries the slice it was handed, #3 pins the union over `ctx.roots`. No test or fixture is shaped like a Flatpak/Snap or Deck root, so the upgraded commitments are covered only by a structural test of the generic mechanism (checklist item 7).",
    "fix": "One clause in #1 or #2 — `roots()` returns exactly what `SteamCore::with_roots` kept — and one `ctx.roots` entry in #3 shaped `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` that yields a `SteamUserdata` tree. No discovery-side work is implied: the candidate list is already pinned at crates/discovery/tests/steam_linux_tests.rs:294-345."
  },
  {
    "severity": "minor",
    "location": "§Read discipline and budgets — \"`MAX_USER_DIRS_PER_GAME = 16` | `userdata/` directories considered per game\" against §Config-path resolvers' \"`<root>/userdata/<dir>/<ctx.appid>` for every `root` in `ctx.roots`, one per digit-named `<dir>`, byte order, at most `MAX_USER_DIRS_PER_GAME`\"",
    "problem": "[new] Making the template a union over roots leaves the cap's unit undecided — sixteen per game in total or per root — and the aggregate sentence counts only one set of sixteen (\"`(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` … (544 MiB)\"). Under the per-root reading, A3's nine Linux candidates plus any number of repeated `--steam-root` values put the worst case at `(|roots| × 16 + 1) × 32` files, several multiples of the figure `schemas/README.md`'s budget bullet is told to state.",
    "fix": "One clause deciding it: the cap counts `userdata/` directories across all `ctx.roots` for one game (so the arithmetic stands), and say which directories are dropped when the union exceeds it. (Shared with the performance seat.)"
  },
  {
    "severity": "minor",
    "location": "§Determinism + §The CLI face step 3 — \"across documents, deduplicated by (`user`, `provenance.files`) — two records that resolve the same tree state it once\"",
    "problem": "[new] The key is sound for one game in two libraries, not for the roots union introduced in the same round: two distinct resolved roots can each carry `userdata/<same SteamID>/<appid>/` (native install plus a Flatpak or Snap Steam on one account), and since `files` are deliberately tree-relative (\"the portability rule: the same layout states the same document on any machine\"), those two trees share the key and one document is silently dropped — no refusal, no status, and no printed path that says which install answered.",
    "fix": "Deduplicate at the `UserTree` level on the tree's absolute root path (which is exactly what the two-records case repeats) and let §The CLI face emit one document per distinct tree; same one-fact-stated-once property, without building document identity out of values the contract strips machine context from."
  },
  {
    "severity": "minor",
    "location": "§Rollout item 9 — \"`cargo deny check advisories`, and `npm ci && npm run check` in `tools/orchestration`, per `.github/workflows/check.yml`\"",
    "problem": "[new] That leg is not one of the run's cargo gates: the workflow runs it as the separate ADR-027 `advisory-gate` job, which builds its own `cargo-deny` 0.20.2 from source on toolchain 1.88.0 inside a 30-minute budget precisely because it is \"too heavy for the 15-minute `rust` job\" (check.yml:83-126). A22's own enumeration of CI (\"the cargo trio plus a Windows *compile* check … and the Node check\") does not count it, and this run adds no external package for it to screen; listing it inside the run's gate set imports a tool install no other part of the design owns.",
    "fix": "Either cite it as what it is — \"the `advisory-gate` job, its own pinned scanner and toolchain; nothing here adds a package for it\" — or keep A22's list; the same line could also name the `cargo check -p bindforge-registry --all-targets` step the `rust` job does run."
  },
  {
    "severity": "minor",
    "location": "§Rollout item 8 (Ledger) — \"the units whose shipped reading differs from their wording (`017-C1`'s bound and `extract` return shape, `017-C3`'s `inputs`, `017-C4`'s 'same scan', `017-S1`'s launcher contexts)\"",
    "problem": "[new] `017-C2` is missing, although the Revision log already records a divergence from it (\"§GameContext — `roots` is `017-C2`'s whole resolved set … not a 'main' one\") and the ADR clause reads \"every steam root the scan resolved — main root and library roots separately, because `userdata/` lives only under the main root\". What ships is one undifferentiated `roots` vector with none labelled main, i.e. a shipped reading that differs from the wording, and this hand-off is the only mechanism that keeps the operator-owned `docs/adrs/` accurate.",
    "fix": "Add `017-C2`'s main/library separation to the hand-off list and to the \"Five clauses\" count in §Context and the matching Risks row."
  },
  {
    "severity": "nit",
    "location": "§Goals — \"`unreal-ini` ships with fixture trees for the two launcher contexts whose roots a record can justify\"",
    "problem": "[new] ADR-017's Terminology fixes a launcher context as \"the native host layout, or the Proton prefix the game was launched under\", so after this round's removal of the host template the two shipped trees (`steam-userdata`, `proton-prefix`) are not the ADR's two contexts, and the phrase hides that the native context is exactly what moved to §Deferred.",
    "fix": "Reword to \"the two root-relative trees a record can justify\", with the context named where §Deferred already names it."
  },
  {
    "severity": "nit",
    "location": "Compatibility matrix, Steam Deck row — \"A Linux Steam install with the `$HOME` and Proton prefix layout the two rows above claim\"",
    "problem": "[new] Nothing hangs off `$HOME` any more (§Config-path resolvers: \"each built from a `GameContext` anchor and nothing else — no resolver reads the process environment\"); the resolved Steam roots merely live under it, so the row describes a layout the design no longer claims.",
    "fix": "\"the root-relative `userdata/` and Proton prefix layout the Linux and Proton rows claim; unexercised here — no Deck leg (A22) and no Deck fixture\"."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/017-keybinding-extraction-codecs.md.
Load the `design-review` skill and apply the compatibility checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/compatibility-r1c4.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Compatibility matrix — \"Flatpak / Snap Steam | Yes, for the root-relative trees\" and \"Steam Deck | Yes, by the Linux row\", against Test strategy #3 \"two fixture roots holding 9 and 8 digit directories keep the first 16 in that enumeration order\"", "problem": "[re-raised] This round re-wrote #3 without adding either piece round 1 asked for: no test states that `roots()` returns the set `SteamCore::with_roots` kept (the seam both \"Yes\" rows now rest on), and no `ctx.roots` entry in #3 is shaped like a sandboxed install, so checklist item 7 (a platform test per supported row) is met only by a structural test of the generic union. The two-root fixture does strengthen the union mechanism, and no discovery-side work is implied — the Flatpak and Snap candidates are already pinned at crates/discovery/tests/steam_linux_tests.rs:288-347 — which keeps this minor.", "fix": "One clause in #1 or #2 pinning `roots()` to the set the core retained, plus one `ctx.roots` entry in #3 of the shape `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` that yields a `SteamUserdata` tree."}, {"severity": "minor", "location": "§`bindings/v1` — \"`files` are tree-relative paths, sorted byte order (the portability rule: the same layout states the same document on any machine)\" plus this round's \"`files` ≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` tree-relative components\" and Test #10's new \"a `files` entry over `MAX_LOCATE_DEPTH` components\"", "problem": "[new] The spelling of a `files` entry is never fixed — the separator a Windows-produced entry carries (`\\` from a joined `PathBuf`) versus `/`, and the case of components taken from real directory entries. Now that the bound is counted in components, the loader must split the JSON string by a rule it is not given, and the \"byte order\" §Determinism publishes and that the 013-R2 fingerprint recipe depends on (\"the located files' octets concatenated in the snapshot's sorted `provenance.files` order\") is separator- and case-sensitive, so one layout could state two different documents — and hash two fingerprints — on two machines, contrary to the portability parenthetical. Nothing breaks this run (the Windows row claims nothing; `017-R2` owns it), hence minor, but the contract is published now. No house precedent exists to inherit: no tree under `schemas/` carries a path in a document (checked manifests, profile, policy, catalog, action-ids, keyboard-geometry). Checklist item 6.", "fix": "One clause: a `files` entry is the located file's components below the tree root joined by `/` (from `Path::components()`, never a displayed path), the loader counts and splits on `/` and refuses an entry containing `\\`, and byte order is over that joined string — replacing today's platform-native \"tree-relative paths\" wording."}, {"severity": "nit", "location": "§Config-path resolvers — \"in `TreeKind` declaration order then byte order of the tree's `root` — a total order over the trees, which §Determinism relies on\" against \"for every `root` in `ctx.roots` in `roots()` order, one per digit-named `<dir>` in byte order, keeping at most `MAX_USER_DIRS_PER_GAME` such trees in that enumeration order across all roots\"", "problem": "[new] The revision states two orders for one list. Root-major-then-`<dir>` equals byte order of the whole tree path only while no canonical root is a proper prefix of another; repeated `--steam-root` values can nest with `steamapps/` present under each, since `canonical_paths` keeps both (crates/discovery/src/steam_common.rs:369-381), and then the set the cap keeps and the order §Determinism publishes are decided by different rules. Test #3 pins the second order while §Determinism relies on the first. Nit: it needs nested roots and more than 16 userdata directories.", "fix": "Make one order do both — enumerate `SteamUserdata` trees by byte order of the tree root they build, which is what §Determinism and the ordering law in `schemas/bindings/v1/spec.md` publish — so Test #3's cap case is the first 16 of that same sequence."}]}
```
