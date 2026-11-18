# Transcript: design-review:security r1c0

## Provenance
- stage: design-review:security (round r1c0)
- agent: review/design-security
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-18T02:59:20.247Z
- ended: 2026-11-18T03:00:57.343Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":23660,"output":15804,"reasoning":0,"cache.read":170484,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/tasks/001-steam-linux-discovery/02-proposal.md.
This is the full-panel pass c0 of round 1. This pass's scope: the whole document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/tasks/T-001-steam-linux-discovery.md,
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
verify what the design claims about the code, and the report at docs/designs/tasks/001-steam-linux-discovery/reviews/design/security-r1c0.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
(none — this is your first pass, or your last verdict held no findings)
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/tasks/T-001-steam-linux-discovery.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/tasks/001-steam-linux-discovery/reviews/design/security-r1c0.md. That file is the only path this session
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
{"verdict":"request-changes","findings":[{"severity":"major","location":"§4 KeyValues reader + §8 CLI wiring — \"`keyvalues::parse` accepts, for both file kinds: whitespace-separated tokens; quoted and bare keys and values\" / \"The printing code is untouched, so the table header and JSON shape are unchanged.\"","problem":"The accepted subset constrains structure (braces, depth, conditionals, file size) but never the character class inside a key or value, and no per-value length cap exists beyond the 1 MiB file cap. §5 copies name/installdir verbatim, and the sole consumer prints them raw: apps/discovery-cli/src/main.rs:48-57 interpolates display_name and install_path.display() into a pipe-delimited stdout table. A manifest name carrying 0x1b gives terminal-escape injection into the user's terminal and any captured log; a value carrying a newline forges table rows; a ~1 MiB single-token name is printed whole. Valid-UTF-8 control characters pass both the &str gate and the grammar; the --json path is safe only incidentally (serde escapes control chars). These are not merely \"the user's own files\": name is written by the Steam client from server-supplied store metadata, and §3 accepts arbitrary absolute library paths, so a manifest on a world-writable auto-mounted library gets read and printed. This plugin is the first data source ever to feed that printer (DummyPlugin returned vec![]).","fix":"In §4 add the missing clause to the explicit subset: a key or value containing any control character other than tab, or exceeding a named per-value length cap, is a positioned KeyValueError; add the matching case to Test strategy #1. Leaves §8's printer untouched and keeps the public surface unchanged."},{"severity":"major","location":"Rollout step 5 — \"together with one real `appmanifest_*.acf` and the machine's `libraryfolders.vdf` if they can be shared\"","problem":"Instructs a contributor to paste raw host files into 03-build-notes.md, a checked-in artifact in a public repo (unretractable once committed), with \"if they can be shared\" as the only guard and no redaction requirement. Both file kinds carry personal data: absolute paths encoding the account name and machine layout, and libraryfolders.vdf's apps map, which is the owner's complete installed-app inventory. The task's Done-when asks only that discovery list correctly identify games on a real machine (docs/tasks/T-001-steam-linux-discovery.md:24); the pasting is the design's own addition, so the exposure is discretionary and owned here.","fix":"Make the step capture a redacted capture: replace the home prefix and username with a placeholder, drop the apps map and any label values, keep only the structural shape and the keys the reader must handle — stated inside the step so the contributor follows the scrub rather than deciding per-file."},{"severity":"minor","location":"Context / Assumption register — \"adds the first implementation — `SteamLinuxPlugin`, which reads library folders from `libraryfolders.vdf`, enumerates `appmanifest_*.acf`\"","problem":"No line classifies the two data sources, though every later control (§2 canonicalization, §4 caps, §5 installdir rejects) implicitly depends on that classification. The Risks table calls them only \"user-editable files\", which understates the surface and leaves the caps and rejects reading as robustness choices, so a later \"convenience\" relaxation looks harmless.","fix":"Add one assumption-register row (or one Context sentence) classifying libraryfolders.vdf and appmanifest_*.acf as untrusted external input — user-writable on disk and, for name/installdir, written by the client from store metadata — whose strings reach the CLI's unescaped printer."},{"severity":"minor","location":"§4 KeyValues reader — \"Files larger than 1 MiB are skipped as malformed (A16): the reader's work is bounded per file\"","problem":"The enforcement point is unspecified, and the neighbouring text describes the opposite order (\"the caller reads bytes and skips a file that is not valid UTF-8\", i.e. read-everything-then-decide). A read-then-check implementation defeats the cap it claims: peak memory equals the file's real size, and a non-regular entry named appmanifest_*.acf (FIFO or device node in a library directory) is read to completion or blocks the scan, with §7's skip-and-continue turning the block into a hang. Neither §4 nor Test strategy says only regular files are opened.","fix":"State in §4 that the caller checks symlink_metadata's file type and st_size, opens regular files only, and bounds the read itself (Read::take(1MiB + 1)), so the cap holds regardless of whether the size field is truthful."},{"severity":"nit","location":"§3 Library selection within a root — \"a `path` is used as-is when absolute and otherwise joined to the directory holding the VDF\"","problem":"Accepting an absolute path from an untrusted VDF is required by the Compatibility matrix row for libraries on arbitrary mounts, so this is not a defect; what is missing is the recorded blast radius, so the absence of a confinement rule reads as an oversight.","fix":"One sentence: a hostile path can only make the plugin open files named appmanifest_*.acf directly inside a directory that already has a steamapps directory, read-only, surfacing contents only as display_name/install_path — the reason no confinement rule is imposed."},{"severity":"nit","location":"§8 CLI wiring / Rollout step 4 — \"`install_path = l.join(\"steamapps\").join(\"common\").join(m.install_dir)`\"","problem":"The checklist question \"what leaves the machine when bindings are shared\" has no answer anywhere in the document, including Deferred: install_path embeds the host's account name and library layout, and this design is what puts it on the machine-readable --json handoff downstream tooling consumes.","fix":"Add one Deferred line assigning the decision — whether install_path may be embedded unredacted in a shared binding file — to the sharing/codec design."}]}
```
