# security review r1

(pass c0, scope: whole document; inputs: 02-proposal.md, docs/adrs/013-game-manifests.md, Revision log)

verdict: approve

Checklist coverage: (1) trust boundaries — evaluated, A14 classifies the whole
`data/manifests/` tree as untrusted and Rollout 5 leaves the README's
untrusted-input paragraph intact; the `id`↔directory byte-equality location
rule is a real anti-spoofing control. (2) parsing untrusted data — evaluated;
closed entry set, no code evaluation, no compression, budget family extends the
existing 1 MiB/16-deep limits; one array is uncapped (F1). (3) path handling —
evaluated; the slug grammar makes `..`/absolute/separator-bearing ids
unrepresentable and the location rule refuses without echoing a path; one
file-type gap (F4). (4) secrets/PII — evaluated; the fingerprint is stated as
provenance not a guard (A16) and refusal echoing is escape-and-truncate; the
transport question is deferred to ADR-018 consistently with ADR-013. (5)
dependencies — nothing to fetch: the design names no dependency and states
"std-only, no new dependency, `Cargo.lock` byte-unchanged"; taken from the
proposal's own claim. (6) privileges — evaluated; one named file read, no
network, no config-path discovery (017). (7) fail closed — evaluated; the
`UnknownFormat` refusal for a pre-013 core and the parse-before-join refusal
ordering are correct; two soft spots (F2, F5).

## Findings

- [minor][new] Chosen design → The contract (entry table `presets` + **Budgets**) — "`MAX_MANIFEST_ROWS = 256` bounds each of `actions`, `dead-keys`, and `docked-actions`" — `presets` is the only array with no cardinality cap anywhere in the design: the entry table gives it none, the Budgets bullet omits it, and the Risk row's mitigation ("The four named budgets refuse before any join") therefore does not cover it. Under A14 it is untrusted input, and the "each preset id appears once" rule is a global uniqueness pass over it, so a hostile document within the 1 MiB budget carries on the order of 10^5 slug strings that every consumer copies, uniqueness-checks, and re-serializes. Fix: give it a cap in the same sentence as the others — "`MAX_PRESET_IDS = 64` bounds `presets`", mirror it in the entry table the way `exclusive-groups` already carries "at most 64 groups", and add `presets-past-cap.toml` to the invalid corpus plus the Test strategy #4 budget list.
- [minor][new] Chosen design → The contract → **Row encoding** — "a native name of 1–128 octets with no ASCII control character" — the ban is ASCII-only, so C1 controls (U+0080–U+009F) and Unicode format characters (U+202E RLO, U+200B/ZWJ, U+FEFF) are legal native names by the spec's own wording. Those names are the design's stated echo payload ("every echoed native name goes through the escape-and-truncate error path") and the content a reviewer and the CLI/hub display, so a hostile manifest can reorder or mask what a terminal shows in a refusal line and in the manifest diff that is this design's only review gate. Because `spec.md` + fixtures are the contract for third-party readers, an out-of-repo reader implements exactly the ASCII-only ban. Fix: change the one clause to "no Unicode control or format character (`Cc`/`Cf`)" and pin it with one more invalid fixture (e.g. `name-format-char.toml` carrying U+202E) beside `name-control-char.toml`.
- [minor][new] Chosen design → Load-time validation: the registry join — "the two cross-file rules (join, location) live at load, where the registry and the directory are in hand" — the design splits validation across two levels but never says what makes the split safe for consumers. The `document.rs` edit adds a public `Document::Manifest(Manifest)` arm to the existing generic `parse`, and `Manifest`'s fields are `pub`, so a consumer can obtain a `Manifest` whose `actions[].action` and `docked_actions[].action` were never `registry.contains`-checked and act on it; ADR-013's Decision ("ids are validated against the loaded registry, so a manifest cannot invent an action the taxonomy does not name") and ADR-015's precondition lookups then rest on a value that skipped the only guard. Fix: name the invariant once in this section — a `Manifest` that did not come from `load_manifest` is not contract-valid and is not a legal input to `seeded-from` or to 015's target refusals — and say where it is enforced; if the intent is enforcement rather than convention, keep the constructor crate-internal (`Manifest` fields `pub(crate)`, accessors `pub`) so the generic arm cannot hand out a consumable unjoined value.
- [minor][new] Chosen design → Load-time validation step 1 — "size budget before the read" — the pre-read budget is a stat-shaped check, and A14 makes the whole `data/manifests/` tree something that "arrived from elsewhere": a shared tree can carry `manifest.toml` as a symlink or a FIFO/character device, which reports a small (or zero) size and then yields an unbounded or never-ending read, so the budget that every other hostile-shape mitigation depends on is bypassed before any rule runs. Fix: in step 1, require the path to be a regular file (refuse a symlink or non-regular entry as `InvalidValue`, naming `manifest.toml`, never the resolved path), and pin it in Test strategy #4 beside the directory-segment case.
- [nit][new] Chosen design → The contract (entry table `revision`) — "the data revision, `>= 1`, monotone per game" — the field has a stated lower bound and no stated upper bound, and the enumerated integer edge cases are "`revision = 0`, a negative revision, and a string revision"; an out-of-range decimal (`revision = 99999999999999999999`) against an `i64` field is the one integer case with neither a rule nor a fixture, even though every other budget in the section is named precisely so a refusal is deterministic. Fix: bound it ("`revision` must fit `i64`" — or narrow the field to `u32`) and add one invalid fixture, so a third-party reader refuses the same documents this reader does.

## FYI (no severity)

- The example manifest's `docked-actions = ["ui.inventory Tab"]` names `Tab`, which the design itself says is not in the closed vocabulary (A8); the parenthetical already flags it as not shippable, so this is only a fixture-vs-example consistency question for the correctness seat.
- "the loader enforces monotonicity forward" reads as a loader guarantee, but the enforcement actually described is a review refusal plus A10 ("core cannot pin a file's history"); whether the loader can enforce anything is a correctness/wording question, not a trust-boundary one — the loader's refusal behavior is unaffected.
- `registry.toml` is implicitly trusted repo data while everything under `data/manifests/` is untrusted; the classification is unstated but consistent with A14, so no action is asked for here.
