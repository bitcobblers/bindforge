# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: basalt/qwen-flash-next#high
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-24T04:07:56.173Z
- ended: 2026-11-24T04:14:50.490Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":57184,"output":13285,"reasoning":0,"cache.read":434671,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/017-keybinding-extraction-codecs/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 37bd2f3753a299cff3db8ca37d9427ca3ba5d404 a4dda6a8542694895102f4a9460835e189e4f08b` (both are blob ids of 02-proposal.md), then read the full current document.
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
verify what the design claims about the code, and the report at docs/designs/017-keybinding-extraction-codecs/reviews/design/security-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "§Config-path resolvers (closing paragraph) and §Read discipline — \"Candidate paths are never canonicalized: the anchors (`root`, `library`) are already canonical from the scan, and §Read discipline never descends a link, so no resolved path is needed to keep a walk inside a tree.\"",
    "problem": "The no-link-descent rule only governs the walk below a UserTree root; the root itself is built by join and is opened with the OS following links. SteamUserdata's <dir> component is a read_dir name filtered solely by the 1–20-digit rule (\"one per digit-named `<dir>`\"), so a digit-named symlink under userdata/ is accepted as a tree root, and for ProtonPrefix the whole pfx/drive_c/users/steamuser chain is filesystem content that A17 says is \"held together by symlinks\". A symlinked tree anchor relocates the locate walk outside the context's trees, breaking 017-C2's \"locates nothing outside the context's trees\" (re-quoted in §GameContext) — the very containment claim used to justify skipping canonicalization. Test strategy #6 pins links inside the walk only, so implementation and suite both miss the anchor.",
    "fix": "In §Config-path resolvers state that a UserTree is returned only when its final component is not a symlink (DirEntry::file_type() on enumerated userdata/<dir> entries; symlink_metadata on a template root), skipped and counted like a symlinked directory in the walk; replace the absolute claim \"no resolved path is needed to keep a walk inside a tree\" with the qualified one (no link is followed anywhere, anchor included) and extend the symlinked-prefix fixture and Test #6 with a symlinked tree root."
  },
  {
    "severity": "major",
    "location": "§`bindings/v1` — \"Bounds, stated once here and cited elsewhere: … `entries` ≤ `MAX_SNAPSHOT_ENTRIES`; `files` ≤ `MAX_CANDIDATE_FILES` per document.\"",
    "problem": "One entry's `inputs` array has no count bound (only each token's octets are bounded), so entries × inputs is unbounded, and no constant in the five-constant budget family or in §bindings/v1 caps the byte size of a document the snapshot loader parses. The contract tree's posture is the opposite: schemas/README.md's \"Input budget and chain bound (adversarial guard)\" binds a 1 MiB cap before the read for documents arriving from disk, another user, or the network, every existing spec.md restates it, and every array-shaped field of every format carries a named bound (MAX_MANIFEST_ROWS, MAX_PATCH_DELTA_ELEMENTS, MAX_POLICY_TRANSFORMS, MAX_RELOCATE_TARGETS). The precedent this design copies, schemas/catalog/v1/spec.md, states MAX_CATALOG_DOCUMENT_OCTETS with an fs::metadata-gated pre-read check. bindings/v1 is built to travel to manifest authoring, the apply tool and UI import, and ages through majors, so a refusal added later breaks already-valid documents and the hole becomes permanent at i… [elided by the checkpoint cap]",
    "fix": "Add one named per-entry input bound (MAX_INPUTS_PER_ENTRY) to the §Read discipline budget table, the schemas/README.md budget bullet and the aggregate sentence, and state in §bindings/v1 that the snapshot.rs loader applies a MAX_DOCUMENT_BYTES-style fs::metadata length check before parsing (the same A4 posture §Read discipline uses); add one invalid fixture per bound to Test #10."
  },
  {
    "severity": "minor",
    "location": "§The first family — \"collecting `*.ini` files whose path ends `Saved/Config/<one directory>/<name>.ini` and whose text carries a `[/Script/Engine.PlayerInput]` section with at least one `Bindings=` line\"",
    "problem": "Claiming requires opening and decoding game-owned files during detect, but only extract is routed through the reader (\"extract reads only located files, through `read.rs`\"), and the design never says what a path-grammar match that is over budget or undecodable does at locate time. The natural reading is a silent skip, which reports an over-budget or undecodable binding store as claim-refused/family-confirmed-files-absent — the distinction 017-L1 exists to keep (\"a claimed game with an unreadable binding file is a different fact from a game no codec claimed\") — and it is the one place a codec could allocate a file's whole body before MAX_BINDING_FILE_OCTETS is consulted.",
    "fix": "Add a clause stating that locate tests candidates through read::open_capped plus read::decode (which is also what makes the bom-utf16 fixture locatable), and that a path-grammar match that is over budget or undecodable claims and refuses by name (FileOverBudget / UndecodableText) rather than being skipped."
  },
  {
    "severity": "nit",
    "location": "§`bindings/v1` — \"`user` (the context label, absent when the family's storage is not user-scoped)\"",
    "problem": "The spec's stated content list (key set, bounds, ordering law, refusal names) never records that one key carries a persistent personal identifier (the local SteamID) in a document intended to travel between tools and machines, so a later sharing surface inherits the disclosure silently.",
    "fix": "One sentence in schemas/bindings/v1/spec.md naming `user` as launcher-local identity that any sharing surface must treat as personal data; behaviour unchanged."
  },
  {
    "severity": "nit",
    "location": "§Local paths (scoping) — \"`--probe` mode exists to name files, so it prints absolute paths in probe output and tree-relative ones in the contract document\"",
    "problem": "The scoping argument holds against the rule it cites (schemas/README.md's no-path clause is scoped to a message shown to whoever sent the document), but probe lines also disclose the OS username and the SteamID, and the Risks row frames the exposure only as a reviewer expecting the no-path discipline.",
    "fix": "Add that disclosure to the same paragraph (probe output is local-operator output, not safe to paste into a public issue) instead of changing the behaviour."
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
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/017-keybinding-extraction-codecs/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict":"approve","findings":[{"severity":"minor","location":"§Config-path resolvers (closing paragraph) — \"Candidate paths are never canonicalized: the anchors (`roots`, `library`) are already canonical from the scan and no link is followed at an anchor or below one, so a walk never leaves the tree it started in.\"","problem":"re-raised (narrowed; the r1 major is otherwise closed). The named checks — `DirEntry::file_type()` on the enumerated `<dir>`, `symlink_metadata` on a template root — establish only that the tree root's final component is not a link; the joined template path can still carry a link at an intermediate component below the canonical anchor (`userdata` for SteamUserdata; `pfx`/`drive_c`/`users` for ProtonPrefix), which the OS follows during the join, anchoring the locate walk in an outside tree while the new sentence claims that is impossible. A17 itself records prefix trees as \"held together by symlinks\", so an intermediate link is the documented real-world shape, and Test #6 pins only `.ini`, directory and tree-root links, so the case is untested. Exploitation needs write access to the launcher's own trees and yields a grammar-filtered read of `.ini` files outside the context's trees with a bounded echo through `UnparsableEntry`/snapshot tokens — hence minor, not major.","fix":"Extend the existing check up the fixed template chain in one clause: every path component between the canonical anchor and the tree root is `symlink_metadata`-checked, and a link at any of them skips-and-counts the tree exactly as a linked final component already does (template paths are ≤ 6 components, so no unbounded work); add one `#[cfg(unix)]` link at a template-intermediate component to the `symlinked-prefix` fixture and assert it in Test #6."}]}
```
