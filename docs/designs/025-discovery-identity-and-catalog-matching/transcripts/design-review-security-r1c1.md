# Transcript: design-review:security r1c1

## Provenance
- stage: design-review:security (round r1c1)
- agent: review/design-security
- model: basalt/Qwen3.8-Flash-Next NVIDIA-NVFP4
- template: tools/orchestration/prompts/design-review.md @ fade37ef72a3
- started: 2026-11-19T01:59:14.657Z
- ended: 2026-11-19T02:03:20.363Z
- parameters: {"note":"not exposed by endpoint"}
- tokens: {"input":40972,"output":17852,"reasoning":0,"cache.read":314212,"cache.write":0}
- gate: (no gate)

## Prompt
```text
Review the design at docs/designs/025-discovery-identity-and-catalog-matching/02-proposal.md.
This is the dissent pass c1: the last revision was written to answer YOUR findings of round 1. This pass's scope: the sections changed since you last reviewed: run `git diff 2efd6adee0318436891ae32ccd6be5b3d7dab059 4461833195ec56a1f702f869cb3c83db3f95c017` (both are blob ids of 02-proposal.md), then read the full current document.
Review independently. Your inputs are exactly: 02-proposal.md, docs/adrs/025-discovery-identity-and-catalog-matching.md,
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
verify what the design claims about the code, and the report at docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/security-r1c1.md
you already wrote, which is yours to re-read through your `git diff*` shell
allow (your `read` tool stays fenced off `reviews/`). Opening any other run
record to check a claim is not open to you.
The Revision log records findings the author deliberately rejected, with
reasons — read it before treating a finding as new.
Your own previous findings (security seat):
[
  {
    "severity": "major",
    "location": "Chosen design > Catalog client and gate — `apps/discovery-cli/src/catalog.rs` (new) — \"It requires scheme `http` and a loopback host (`127.0.0.1`, `[::1]`, `localhost`)\"",
    "problem": "The loopback-only authority is the sole enforcement of the design's headline non-disclosure property (Options considered: \"it cannot disclose an installed-game inventory to a remote host\"; Risks row 3 mitigation: \"This build can only reach a loopback authority\" for a risk rated Medium impact / Certain-by-design), yet the document never states the string split that decides it: no rule for a userinfo (`http://127.0.0.1:8080@host.example/`), for where the authority ends (`/`, `?`, `#`), for an empty host, for the port's character set, or for non-dotted IPv4 spellings. No URL parser exists to fall back on (Cargo.lock's only external names are clap, serde, serde_json, winreg) and the design commits to no new dependency and no Cargo.lock change. Test strategy 9 pins only four spellings, none of which separates a correct authority split from one that merely contains `127.0.0.1`, so a build passing every pinned case can still send `GET /v1/candidates?slug=steam%2F900001` to a remote socket: th… [elided by the checkpoint cap]",
    "fix": "State a pure `fn parse_endpoint(base: &str) -> Result<Endpoint, CatalogError>` beside `candidates_url`/`selection` with the accepted grammar spelled out (literal `http://` prefix; authority runs to the first `/`, `?` or `#`; a `@` anywhere in the authority is refused; host must be exactly one of the three literals; a port, if present, is 1–5 ASCII digits), enforce it at the single `HttpCatalog` construction site, and add the userinfo, embedded-`?`, embedded-`#` and odd-IPv4 spellings to Test strategy 9's rejection list."
  },
  {
    "severity": "minor",
    "location": "Chosen design > Catalog client and gate — \"`probe()` asks the authority, not an endpoint: resolve the host/port and `TcpStream::connect_timeout`\"",
    "problem": "The loopback property is established by name, not by address. `localhost` is resolved by the system resolver, so a hosts/NSS entry pointing it elsewhere passes the stated check, and since resolution can return several addresses that `connect_timeout` tries in order, a name resolving to one loopback and one remote address passes the name test yet connects outward — the same disclosure as the endpoint-parse finding, from the resolver side.",
    "fix": "After resolution require every candidate `SocketAddr`'s `IpAddr::is_loopback()` and refuse otherwise (making the two IP literals redundant), or state the name-literal rule as an accepted residual risk beside the inventory-disclosure row in Risks."
  },
  {
    "severity": "minor",
    "location": "Chosen design > Scan flow and output — `apps/discovery-cli/src/main.rs` step 8 (\"`Game Id` prints the resolved id or `-`\") with the gate section's \"every echoed value — a slug, a candidate `name`, a rejected `game_id` — goes through `bindforge_core::escape_and_truncate`\"",
    "problem": "The escape invariant covers exactly three values while the two table columns this design keeps are raw third-party bytes: `display_name` and `install_path`, both read from the ACF (`parse_manifest` only trims `name`; `usable_install_dir` refuses `.`, `..`, `/`, NUL, absolute and — on Windows only — `\\`, admitting every other byte). `row_cells`/`render_table` (apps/discovery-cli/src/main.rs:157-165) pass cells through unchanged, so an ESC/OSC sequence in `name` or `installdir` reaches the terminal as a control sequence, bounded in length only by `keyvalues::MAX_DOCUMENT_BYTES` = 1 MiB. The product's own grammar refuses ASCII control characters in a native name (crates/core/src/manifest.rs:330-348), and this design clamps the hidden `friendly_name` to MAX_NATIVE_NAME_OCTETS while leaving the visible field uncapped. Inherited, but the design rewrites COLUMNS and the echo rule in the same section.",
    "fix": "State that the two ACF-sourced cells render through `escape_and_truncate` (replacing today's raw `row_cells` passthrough), or state ADR-013's control-character rule as the explicit acceptance, and pin a control-character `name` fixture so the table cannot emit a control sequence."
  },
  {
    "severity": "minor",
    "location": "Chosen design > Catalog client and gate — \"status other than `200`, a `Transfer-Encoding` header, a missing or over-`65_536`-byte `Content-Length`, a body that is not valid `Vec<Candidate>` JSON\"",
    "problem": "The refused shapes are named but the matching rules are not, and in a hand-rolled parse those details are the control: header field names are case-insensitive, so a `transfer-encoding: chunked` spelling must be matched case-insensitively or the named refusal is skipped and the JSON parse fails by luck rather than rule; two `Content-Length` lines, or one with leading zeros, a `+` or non-digits, is the confusion the cap exists to prevent, and the text says which value is too big but not which line wins or what spelling is legal; the 65 536 bound covers the body only, bounding nothing about the header block, which a peer that never sends the blank line streams into memory for the whole 5 s read timeout. The candidate array has no client-side count bound, so one 64 KiB `200` becomes a prompt of ~2000 numbered lines (the contract is deferred to 025-R1; a read cap is not).",
    "fix": "One sentence stating that header names are matched case-insensitively, that exactly one `Content-Length` is accepted and only as 1–5 ASCII digits, that the header block is refused past the same 65 536-octet budget, and that a body over a named candidate maximum is an `Unreachable` refusal; add those spellings to Test strategy 10's response-parse cases."
  },
  {
    "severity": "minor",
    "location": "Chosen design > Match store — `apps/discovery-cli/src/matches.rs` (new) — \"one file, `<data>/bindforge/matches.json`\" and \"through `matches.json.tmp` + `std::fs::rename`\"",
    "problem": "The section states path, shape and atomic-write mechanism but no access mode and no read bound, and the store is this design's trust root for cross-source identity: the record whose `game_id` is trusted forever (\"Records never expire and are re-trusted on every scan\", 025-L4) is protected only by directory ownership. A umask-default 0644 file in a 0755 `bindforge/` makes the confirmed installed-game inventory readable by every local user, and if any ancestor is group/world-writable (a shared `XDG_DATA_HOME`, which the seam accepts because it only checks absoluteness) another user can write a record and choose which `game_id` attaches to the user's installs — a wrong join that survives every later scan. The fixed `matches.json.tmp` name shares the assumption: a pre-created symlink is followed by a plain create. Load has no byte bound, so an arbitrarily large matches.json is read whole, where the ACF path caps size before allocation (crates/discovery/src/steam_common.rs:371-380).",
    "fix": "State that the `bindforge/` directory and both store files are created owner-only (0700/0600 on unix) and the tmp file exclusively, and that a store above a named byte budget (the `keyvalues::MAX_DOCUMENT_BYTES` shape) is the same named refusal; pin the mode and the budget in Test strategy 6 and 7."
  },
  {
    "severity": "nit",
    "location": "Chosen design > Match store — \"a `slug` past `025-B1`, or a `game_id` failing `is_slug_name` + `MAX_SLUG_OCTETS` is the named refusal\"",
    "problem": "`slug` is the only record field bounded by length alone, so a hand-edited or corrupted file persists a slug carrying control characters, spaces or path-ish spellings. Nothing renders or path-joins a store slug today (queries are issued only for discovered rows' slugs), so the gap is latent, but it sits against the design's own rule that an identifier is validated \"at all three places it is seen\".",
    "fix": "Also refuse at load a `slug` not spelled `<token>/<external-id>` with `<token>` in `Source`'s closed set, or state in the section that a store slug is only compared byte-for-byte with a discovered slug and never rendered or joined."
  },
  {
    "severity": "nit",
    "location": "Chosen design > Match store (\"the named refusal `error: match store unreadable: <path>`\") and Catalog client and gate (\"error: catalog unreachable: <reason>\")",
    "problem": "These two refusal lines interpolate values the escape clause does not enumerate: `<path>` is composed from `$XDG_DATA_HOME`/`$HOME`/`%APPDATA%` and `<reason>` from a connect error whose display text can carry the host spelling from `--catalog-url`/`BINDFORGE_CATALOG_URL`, so one embedded newline or escape puts a second, well-formed-looking line on stderr — against the invariant stated for this very renderer (crates/core/src/error.rs:176-188). Naming the store path is correct (the user's remedy is to remove the file), so the fix is not to drop it.",
    "fix": "Extend the escape clause to \"every interpolated value in every named refusal\", naming the store path and the reason, and add a newline-in-`$XDG_DATA_HOME` case to Test strategy 6."
  }
]
If this is a dissent pass: the last revision was written to answer the
findings above. Check those first; report anything else only if the changed
text makes it true.
If this is a ratification pass: read only the diff you were given and the
findings above. Report a finding only if the revisions broke something you
approved or made a section you cited false.
The proposal's Deferred section lists concerns it hands to another document;
re-open one only if it contradicts docs/adrs/025-discovery-identity-and-catalog-matching.md.
Load the `design-review` skill and apply the security checklist.
Write your report to docs/designs/025-discovery-identity-and-catalog-matching/reviews/design/security-r1c1.md. That file is the only path this session
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
{"verdict": "approve", "findings": [{"severity": "minor", "location": "Scan flow and output, step 8 — \"`Display Name`, `Install Path` — render through `escape_and_truncate` in place of today's raw `row_cells` pass-through\"", "problem": "new: the escaped set is exactly the two ACF cells, but the same revision retires the only thing that kept the `Slug` column clean — step 4 makes `refuse_invalid_slugs` \"the `025-B1` bound alone\", and the retired grammar check is what refused control bytes in a slug today (apps/discovery-cli/src/main.rs:115-129, whose in-module test pins a newline-in-slug reaching stderr as `\\u000a`, never as a second line). `DiscoveredGame.slug` is a `pub` field (crates/discovery/src/game.rs:44-51), so the Row identity claim that \"a plugin cannot spell the key differently without using another function\" does not bound a third-party row, and `Source::lookup_key` validates nothing about `external_id`: a row from any `dyn DiscoveryPlugin` — the population today's boundary doc comment says it exists for — can carry 128 arbitrary octets straight into the raw `Slug` cell.", "fix": "Add `Slug` (and `Game Id`, already grammar-validated, for symmetry) to the escaped-cell set in step 8 instead of enumerating only the ACF-sourced two, and extend Test strategy 12's no-raw-control-byte assertion to every column."}, {"severity": "minor", "location": "Scan flow and output, step 6 — \"one prompt on **stderr** — stdout carries only the table\"", "problem": "new: the prompt is the remaining render of a server-sourced string, and the revision's escape clause is scoped to \"every value a refusal interpolates\", so a loopback peer's candidate `name` (reachable by any local process that binds the port) reaches the terminal outside `escape_and_truncate` — the control-sequence path step 8 and Test 12 just closed for the table.", "fix": "Widen the escape clause from refusals to every out-of-band terminal write (the prompt's candidate names and slug), and pin a control byte in a fake's candidate `name` in Test strategy 12."}, {"severity": "minor", "location": "Catalog client and gate — \"`probe()` asks the authority ... after refusing unless every returned `SocketAddr` `IpAddr::is_loopback()` — the property is established by address, not by name, so a `hosts` entry for `localhost` cannot send the query outward\"", "problem": "new (residual of the r1-2 closure): the address check is stated inside `probe()` only, while the sentence claims it protects the query — which `candidates()` sends. `candidates()` is specified over a raw `base: &str` (`candidates_url(base: &str, slug: &str)`) plus a plain `TcpStream`, so as written the query path re-resolves the name and dials whatever the resolver returns; a differing answer between gate and query, or an implementation that puts the check only where the document does, puts the slug on a remote socket.", "fix": "State that the gate resolves once and `Endpoint` carries the validated `Vec<SocketAddr>` that `candidates()` dials, so the loopback requirement binds every connection the client makes rather than the probe alone; in the same sentence state that a non-numeric or out-of-range port is the endpoint policy's `TransportUnavailable`, not a later `Unreachable`."}, {"severity": "minor", "location": "Catalog client and gate — \"admits exactly: a literal `http://` prefix; an authority that is non-empty, carries no `@`, and runs to the first `/`, `?`, or `#`\"", "problem": "new: the grammar bounds the authority and is silent about the bytes after it, while `candidates_url` is still specified over the raw string (`<base>/v1/candidates?slug=<encoded>`) and the request is `GET <path> HTTP/1.1`. The section now classifies `--catalog-url`/`BINDFORGE_CATALOG_URL` as \"both untrusted config sources\", so a CR or LF in the path portion of an otherwise admitted base flows verbatim into the request line and lets that config value forge extra request lines or headers to whatever loopback service holds the port.", "fix": "One clause: everything after the authority in an admitted base is restricted to printable ASCII (no control bytes, no space), and the request target is built from the parsed `Endpoint` rather than the raw base; add a `http://127.0.0.1:8080/x\\r\\n` rejection to Test strategy 9."}, {"severity": "nit", "location": "Match store — \"the `bindforge/` directory and both files are created owner-only — 0700/0600 where unix modes apply — so the confirmed inventory is never a `umask` accident\"", "problem": "re-raised (half of r1-5): this closes the modes this process sets, but leaves the case where `<data>` or `bindforge/` already exists group/world-writable (a shared `XDG_DATA_HOME`, which the path rule accepts because it only checks absoluteness) unaddressed and unrecorded, while the store is the trust root for records that are \"re-trusted on every scan\".", "fix": "One sentence beside the inventory-disclosure row in Risks accepting a user-configured shared data directory as residual, or tightening the mode at load."}, {"severity": "nit", "location": "Match store, Save — \"`matches.json.tmp` (opened exclusively, never through a pre-existing entry)\"", "problem": "new: the exclusive open adds a failure mode with no named refusal — a `matches.json.tmp` left by an interrupted run makes every later save fail, and no named refusal covers a failed save, so the run reports a confirmation it never persisted.", "fix": "Name it: `error: match store unwritable: <path>` on stderr with a non-zero exit and no record claimed, and pin the pre-existing-tmp case in Test strategy 7."}]}
```
