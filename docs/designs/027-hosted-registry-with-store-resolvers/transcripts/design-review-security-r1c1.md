# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: basalt/qwen-flash-next
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-22T03:22:42.564Z
- ended: 2026-11-22T03:31:30.763Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":64744,"output":18019,"reasoning":0,"cache.read":871763,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/027-hosted-registry-with-store-resolvers/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 7b5cec46a35f016bf0e05e5cb90abfc015a6cc32 fcb66a83bd31cc82493e01e93f55b745bd37e25c` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/027-hosted-registry-with-store-resolvers.md,
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
verify what the design claims about the code, and the report at docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/security-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "The store pull (\"in tests, the same request/response codec run against a loopback fake (plain TCP)\") and Test strategy #6/#17",
    "problem": "The design's core security control on the store hop — that the production rustls client authenticates the pinned partner.steam-api.com authority — is never exercised: every pull test runs the codec over plain TCP, so a hand-built ClientConfig that accepts self-signed certs or a wrong server name (a natural shortcut to keep the fake transport seam shareable) passes the whole suite green. The confidentiality of the personal, rotation-foreclosed BINDFORGE_STEAM_WEB_API_KEY and the integrity of the snapshot feeding every mint rest on a property no planned test can establish.",
    "fix": "Add one named test that drives the production transport against a loopback-only TLS listener with a self-signed certificate and asserts handshake refusal (and a wrong-server-name refusal); 127.0.0.1 dialing preserves the no-test-dials-a-non-loopback-name rule."
  },
  {
    "severity": "major",
    "location": "Risks (\"013-B4's 128-octet name bound has no margin (A10): one over-long store name fails every pull\") and the r0 revision entry \"over-bound store names — fail the pull by name\"",
    "problem": "Rated Likelihood Low, but store display names are attacker-chosen content — the design's own A11 records games renaming existing appids to impersonate hits. One planted >128-octet name fails every delta (the record recurs in the if_modified_since window) and every forced full refresh, with the only remedy (re-sizing) deferred with no follow-up: a permanent, remote, unauthenticated denial that converts every hosted steam answer to 503 after the 7-day age bound. The rejected-finding rebuttal is unsound because it treats the choice as binary between loud total failure and a silent record hole under 027-L3; a third posture preserves honesty while the pull survives.",
    "fix": "Drop the offending record, persist an `incomplete` fact in the snapshot document and health.steam_snapshot, and while it stands deny `200 []` for all steam non-hits (named 503; corpus/issued/appid-hits keep answering) — no partial list is ever certified, so 027-L3 and the partial-snapshot rule both hold."
  },
  {
    "severity": "major",
    "location": "Options considered (\"The store hop gets `rustls` (no default features, `ring` provider) plus `webpki-roots`\") and the Risks mitigation \"versions pinned\"",
    "problem": "The design introduces webpki-roots/rustls one month after CVE-2026-111943 produced advisories in exactly these crates: RUSTSEC-2026-0381 (webpki-roots, patched >= 1.0.10, with an explicit trap where the 0.26.x compatibility package stays affected unless its 1.x parent resolves >= 1.0.10) and RUSTSEC-2026-0382 (rustls-webpki, patched >= 0.103.16 / >= 0.104.0-alpha.8) — certificate-path acceptance of distrusted emSign intermediates with public keys, i.e. precisely the impersonation tooling for the hop carrying the non-rotatable key; RUSTSEC-2026-0346 (rustls Debug disclosing TLS secrets, patched >= 0.23.46) also collides with the no-secrets-in-diagnostics posture. \"Versions pinned\" names no floor and \"cargo trio covers it\" covers no advisory screening (fmt/clippy/test do not screen RUSTSEC).",
    "fix": "State the minimum floors in the dependency decision (rustls >= 0.23.46, rustls-webpki >= 0.103.16 or >= 0.104.0-alpha.8, webpki-roots >= 1.0.10 with the 1.x-parent resolution note) and add a RUSTSEC advisory screen (cargo audit or cargo deny) to Rollout item 3."
  },
  {
    "severity": "minor",
    "location": "Issued entries, retired ids (\"`MAX_RETIRED_IDS_OCTETS = 33_554_432`\")",
    "problem": "The budget is named but the posture at exhaustion is unpinned, unlike the issued document's spelled-out enforcement. Left open, an implementer can either block all future deletes (killing correction and promotion) or fail open by dropping the append — and a dropped append silently re-opens a retired game_id, breaking 027-L2 ('deleting a record never frees its id') through budget arithmetic.",
    "fix": "Pin fail-closed in one sentence: a delete whose retired-list write would exceed the cap is a named refusal that mutates nothing; the load-side metadata gate mirrors the issued document's."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/027-hosted-registry-with-store-resolvers.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/027-hosted-registry-with-store-resolvers/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "§The steam snapshot resolver (loader refusals) / §The store pull (response rules) — \"a `name` over `MAX_NATIVE_NAME_OCTETS` or carrying control bytes (the `013-B4` bound `027-C3` names — the pull never writes one, §The store pull)\"", "problem": "The r1 parenthetical \"the pull never writes one\" is unenforced for the control-byte half of the loader's name rule: `013-B4` is only the 128-octet budget (docs/adrs/013-game-manifests.md), and the pull's sole name disposition is the drop for a record \"whose `name` breaks the `013-B4` bound\". The shipped native-name grammar (crates/core/src/manifest.rs::check_native_name) additionally bans ASCII control characters, empty, and leading/trailing-space names. A store record with, say, a control character in its attacker-chosen display name (A11) passes every pull rule and is published, and the snapshot's own loader then refuses the file: every later boot takes the forced full-pull path, and once the non-rotatable key ends (Risks row on key termination) the unloadable snapshot converts to a startup refusal. The same class reaches minted issued records: a name the `026-C3` loader would refuse contaminates the issued document at mint time.", "fix": "Extend the pull's single drop-and-bump-`incomplete` clause from \"breaks the `013-B4` bound\" to \"breaks the loader's name rule\" (length, control bytes, empty, leading/trailing space — the `check_native_name` family), keep it the only dropped case, and pin one fixture per clause in Test strategy #6's drop case."}, {"severity": "minor", "location": "§The store pull (merge sentence) vs §The steam snapshot resolver (definition) — \"`incomplete` is how many records the pull dropped for the `013-B4` name bound; `0` marks a worldlist\" and \"a delta merges by `appid` … and re-persists; a drop for the `013-B4` bound bumps `incomplete` rather than failing the page\"", "problem": "The definition is worded per-pull while the merge rule only says drops \"bump\" the field, leaving delta carry-forward unpinned. Under the per-pull reading, a delta with no fresh drops republishes `incomplete: 0` over a merged `apps` list that still lacks the record an earlier pull dropped — a silently incomplete worldlist regains the right to certify `200 []` for exactly the appid it is missing, reintroducing the dishonest miss certification the r1 posture exists to prevent; an attacker needs only one planted over-long name plus the passage of one delta.", "fix": "One sentence in §The store pull: a delta's published `incomplete` is the loaded snapshot's value plus its own drops (dropped records never enter the merged `apps` list, so nothing decrements it); only a full pull can publish `0` — and pin it in Test strategy #6's delta case."}]}
```
