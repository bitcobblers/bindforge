# correctness review r3 (dissent pass c3)

verdict: approve

Scope: `git diff e068cf6132a6fc1e4d93c99cfa7a7f303e66939f 316ef46c5517497b633f76f56fa0ffa689babebf`
(the seven revised passages), then the full current `02-proposal.md`,
`docs/adrs/017-keybinding-extraction-codecs.md`, and the Revision log. Inputs
were those named by the caller; no run record other than this report was opened.

## Disposition of this seat's round-1 findings

1. **[major] dedup key carried by no type — CLOSED.** `UserCandidates` now
   carries `pub tree_root: PathBuf` with the stated role ("that tree's
   `UserTree.root`: §Determinism's dedup and ordering key, never rendered into a
   document"), §The codec face states the pairing that makes it reachable from a
   `BindingSnapshot` ("`per_user` is in `user_trees` order and `extract`'s `Vec`
   is positionally parallel to it"), §Determinism names the key
   ("deduplicated by each document's `UserCandidates.tree_root`"), and #12's
   shuffle now moves `per_user` "(its `tree_root`s with it)", so byte-identity
   under shuffling is now achievable by re-sorting on a field that exists. The
   key is also a *total* one: a `SteamUserdata` tree path ends in a digit appid
   component and a `ProtonPrefix` tree path ends in the literal `steamuser`, so
   no two trees of different `TreeKind` can share a `tree_root` string, which is
   what the Revision log's claim ("the `UserCandidates.tree_root` alone, so
   §Determinism's order over documents is total") rests on and which holds.
2. **[minor] per-tree document identity absent from the divergence record —
   CLOSED.** §Context reads "Nine clauses" and names "`017-C3`'s `inputs`
   spelling and document identity"; §Rollout item 8's hand-off list carries
   "`017-C3`'s `inputs` and document identity"; the Revision-log entry's basis is
   now "`017-C3`'s per-local-SteamID wording reads as per-tree" instead of the
   former "per-user rule wins"; and §`bindings/v1` gained the consumer clause
   ("two trees may share a `user` label, and then nothing but `entries`
   separates their documents, so a consumer unions same-`user` documents'
   entries"). All nine listed clauses have a Revision-log entry behind them.
3. **[minor] emptied-`Command` ambiguity, budget counting, unnamed bare operands
   — CLOSED.** §The first family now decides it ("a `Command` whose pairs the
   fold has all removed leaves no entry") and names the other bare operands ("a
   bare `+`, `-` or `.` operand is out-of-grammar and skipped"); §`bindings/v1`'s
   empty-`inputs` allowance and its "never for any other reason" tail are gone, so
   the §`bindings/v1`/Options(b) contradiction is resolved in Options(b)'s
   direction; the budget is fixed to admissions ("checked against the distinct
   pairs a tree's fold has ever admitted — a removal frees no room", table row
   "pairs as admitted"), which kills the churn-file over-admission path; #4 and #7
   assert both changes. One residual of the deletion is reported as nit A.
4. **[minor] `MAX_LOCATE_DEPTH` counted two units — CLOSED.** The constant is now
   defined in the candidate's own terms ("components in a located file's
   tree-relative path — its last four are `Saved/Config/<dir>/<name>.ini`"),
   §`bindings/v1` cites that unit ("at most `MAX_LOCATE_DEPTH` tree-relative
   components (§Read discipline's table)"), and #6 says "located at
   ≤ `MAX_LOCATE_DEPTH` tree-relative components and not beyond". The new `detect`
   wording ("pruning a directory whose tree-relative path already has
   `MAX_LOCATE_DEPTH` components") is consistent with the document bound: a pruned
   directory's files would exceed it, so #10 and #11 can both hold. The shipped
   templates put a candidate at 4 components, inside the ceiling.
5. **[nit] silent cap's cut order — CLOSED.** The bullet separates the two orders
   ("for every `root` in `ctx.roots` in `roots()` order, one per digit-named
   `<dir>` in byte order, keeping at most `MAX_USER_DIRS_PER_GAME` such trees in
   that enumeration order") from the return order in the lead-in, the table row
   now says "trees kept", and #3 pins the cut with "two fixture roots holding 9
   and 8 digit directories keep the first 16 in that enumeration order". Verified
   against the code the order is well-defined and deterministic:
   `crates/discovery/src/steam_common.rs:369-381` builds the core's root set
   through a `BTreeSet` (canonicalized, de-duplicated, ordered), so `roots()`
   order — and therefore which 16 trees survive — is a function of the context
   alone, and de-duplication is also what makes §Determinism's cross-record
   `tree_root` dedup reach the same tree spelling from two records. The fixture
   root/directory *names* #3 uses are still unspelled; the semantics are pinned,
   which is what the finding asked for.

## Checks run

- `git diff` of the two named blobs (scope), then the whole document.
- Read-only inspection of `crates/discovery/src/steam_common.rs`
  (`canonical_paths`, `SteamCore::discover`'s root/library handling) and
  `crates/discovery/src/steam_linux.rs` (`host_candidates`) to confirm the
  revised enumeration order over `ctx.roots` is a total, de-duplicated order.
- No cargo/rustfmt/npm leg: nothing in the changed text makes a new claim about
  shipped code that a toolchain leg could confirm or refute (no code exists yet
  for `crates/codecs`).

## Findings

- [nit][new] §`bindings/v1` — the snapshot contract, **Input spelling
  (canonical)** — "so both halves of a row are reported in the game's words and
  nothing in this format validates against the closed member sets (A5, A20;
  Options)" (the paragraph from which "`inputs` may be empty when the file's own
  merge operators removed every input for an action, and never for any other
  reason" was deleted) — new, residual of the round-1 closure of this seat's
  emptied-`Command` finding: deleting that sentence settled what the *family*
  emits, but the contract never gained the array bound that replaces it.
  `entries` is `[{action, inputs}]`, entry keys are closed to `{action, inputs}`,
  and the Bounds sentence bounds only `action` and "every input token" lengths —
  no clause anywhere makes `inputs` non-empty, `spec.md` is said to carry "the
  key set, the bounds, … the refusal names", and Test #10's invalid-corpus list
  does not name it. Failure scenario: a hand-authored or future-family
  `bindings/v1` document with `{"action":"MoveForward","inputs":[]}` loads clean,
  prints an empty `Inputs` column, and states a binding no located file states
  (`017-L3`) — the same fact the deleted sentence was carrying, now unstated.
  Fix: three words in the Bounds sentence — "`inputs` is a non-empty array" —
  plus one item in #10's invalid list ("an empty `inputs` array"). This replaces
  the deleted allowance as the contract's statement of that fact; it adds no new
  mechanism and no new refusal name beyond the loader's existing
  "value outside its bound" clause.
- [nit][new] Test strategy #4 — "`UserCandidates::push` refuses a 33rd file
  (`017-B3`'s type invariant, behind the walk's skip-and-count)" — new: this
  revision names two owners for one bound on one type. §The codec face publishes
  the field as `pub files: Vec<PathBuf>` and never lists a `push`, so a caller
  can build or extend a `UserCandidates` past 32 without calling it and the
  bound is not in fact a "type invariant"; §The first family meanwhile puts
  enforcement in the walk ("At most `MAX_CANDIDATE_FILES` path-grammar matches
  per tree, in the walk's byte order … a further match is skipped and counted"),
  and §Read discipline's row reads "one user tree's located candidate set". #4
  therefore asserts the enforcement point the published surface cannot guarantee.
  Fix: name one owner — either make the walk the bound and have #4 assert "a 33rd
  path-grammar match in one tree is skipped and counted", or keep `push` and make
  the listing's field private-with-constructor (`files` built only by
  `UserCandidates::new`/`push`), in which case §The first family's
  "skipped and counted" clause is what stops claiming to enforce it.

## FYI (no severity)

- §Read discipline's aggregate ("one extraction opens at most
  `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES` files … the product also
  ceilings the *opens* … `17 × 32 × MAX_BINDING_FILE_OCTETS`") counts each
  located file once, but `detect` opens the located matches for its content test
  and `extract` "reads only located files, through `read.rs`" with no stated
  caching, so the octet figure is a per-pass bound, not a per-extraction one.
  Unchanged text, outside this pass's scope; the figure is a risk-table
  magnitude, not a contract bound, so it is at most a wording item for the
  complexity seat.
