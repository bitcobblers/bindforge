//! The first family — `unreal-ini` (`017-S1`, §The first family).
//!
//! `id()` is `"unreal-ini"`; `appids()` is empty this run (A21: no appid
//! knowledge is surveyed into the tree — the pass that reads it is pinned by
//! the synthetic family in `tests/common`). `detect` walks each located
//! `UserTree` examining at most `MAX_LOCATE_ENTRIES_PER_TREE` entries and
//! pruning on nothing but depth and entry count, locating every `*.ini`
//! whose tree-relative components end in `Saved/Config/<one directory>/<name>.ini`
//! within `MAX_LOCATE_DEPTH` and whose contents hold a
//! `[/Script/Engine.PlayerInput]` section with a `Bindings=` line — no
//! platform-directory constant is assumed (A15), no project name is guessed.
//! `extract` reads only located files and folds UE's four documented merge
//! operators (`+` adds, `.` adds and may duplicate, `-` removes on an exact
//! match, `!` removes by name; a bare `!` clears the key), keying entries by
//! the `Command` string verbatim and carrying the distinct `Name` tokens
//! verbatim (§Input spelling (canonical)). The install tree is never a
//! candidate tree, so a `DefaultInput.ini` there can contribute no entry:
//! "defaults are not absorbed" holds structurally.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use bindforge_core::MAX_NATIVE_NAME_OCTETS;
use discovery::game::Source;

use crate::codec::{CodecFamily, Detection, Evidence, ExtractionRefusal, UserCandidates};
use crate::context::GameContext;
use crate::paths::{UserTree, user_trees};
use crate::snapshot::{
    BindingSnapshot, Provenance, SNAPSHOT_FORMAT, SNAPSHOT_VERSION, SnapshotEntry,
};
use crate::{MAX_CANDIDATE_FILES, MAX_LOCATE_DEPTH, MAX_LOCATE_ENTRIES_PER_TREE, read};

/// The one section whose `Bindings=` lines this grammar reads.
const SECTION: &str = "/Script/Engine.PlayerInput";

/// The family id, slug grammar, 10 octets (§Codec family id).
const FAMILY_ID: &str = "unreal-ini";

/// The shipped `unreal-ini` family.
#[derive(Debug, Clone, Copy)]
pub struct UnrealIni;

impl UnrealIni {
    /// The trait's `detect`, callable on the concrete family — the seam a
    /// caller that names this grammar (a suite, or a face that asks one family
    /// first) uses without importing [`CodecFamily`](crate::CodecFamily).
    #[must_use]
    pub fn detect(&self, ctx: &GameContext) -> Option<Detection> {
        <Self as CodecFamily>::detect(self, ctx)
    }

    /// The trait's `extract`, callable on the concrete family.
    /// # Errors
    /// Names the [`ExtractionRefusal`](crate::ExtractionRefusal) variant that
    /// refused the run.
    pub fn extract(&self, claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        <Self as CodecFamily>::extract(self, claim)
    }
}

impl CodecFamily for UnrealIni {
    fn id(&self) -> &'static str {
        FAMILY_ID
    }

    fn appids(&self) -> &'static [&'static str] {
        // Empty this run (A21): inventing two AppIDs would be fabricated
        // content; the appid pass is pinned by the synthetic test family.
        &[]
    }

    fn detect(&self, ctx: &GameContext) -> Option<Detection> {
        let mut per_user: Vec<UserCandidates> = Vec::new();
        for tree in user_trees(ctx) {
            let files = locate(&tree);
            if files.is_empty() {
                continue; // a probe claims only when it located a file
            }
            per_user.push(UserCandidates {
                user: tree.user,
                files,
                tree_root: tree.root,
            });
        }
        if per_user.is_empty() {
            return None;
        }
        Some(Detection {
            family: FAMILY_ID,
            // The registry stamps the claiming pass's evidence; a family's
            // own `detect` is the probe.
            evidence: Evidence::InstallTreeProbe,
            per_user,
        })
    }

    fn extract(&self, claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        // §Determinism: the order `per_user` arrives in never reaches a
        // document — groups run in tree-identity byte order, and a tree two
        // records resolve states itself once.
        let mut groups: Vec<&UserCandidates> = claim.per_user.iter().collect();
        groups.sort_by(|a, b| a.tree_root.cmp(&b.tree_root));
        groups.dedup_by(|a, b| a.tree_root == b.tree_root);

        let mut documents: Vec<BindingSnapshot> = Vec::new();
        for group in groups {
            let game =
                game_key(&group.tree_root).ok_or_else(|| ExtractionRefusal::UnreadableFile {
                    path: group.tree_root.display().to_string(),
                })?;
            // "each file's entries folded in byte order of its relative path"
            let mut files: Vec<&PathBuf> = group.files.iter().collect();
            files.sort();

            let mut fold = Fold::default();
            for file in &files {
                let relative = tree_relative(file, &group.tree_root);
                let text = read_and_decode(file, &relative)?;
                fold_file(&text, &relative, &mut fold)?;
            }

            documents.push(BindingSnapshot {
                codec: claim.family.to_string(),
                entries: fold.entries(),
                format: SNAPSHOT_FORMAT.to_string(),
                game,
                provenance: Provenance {
                    evidence: claim.evidence.token().to_string(),
                    files: files
                        .iter()
                        .map(|file| tree_relative(file, &group.tree_root))
                        .collect(),
                },
                user: Some(group.user.clone()),
                version: SNAPSHOT_VERSION,
            });
        }
        Ok(documents)
    }
}

/// The appid embedded in one of §Config-path resolvers' two templates — the
/// `bindings/v1` `game` key is the row's lookup key, and the templates are
/// the only trees this family ever folds, so the appid is readable from the
/// tree's own components.
fn game_key(tree_root: &Path) -> Option<String> {
    let components: Vec<String> = tree_root
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let is_digits = |value: &str| {
        !value.is_empty() && value.len() <= 20 && value.bytes().all(|byte| byte.is_ascii_digit())
    };
    if let Some(index) = components.iter().position(|name| name == "compatdata") {
        // `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser`
        let appid = components.get(index + 1)?;
        return is_digits(appid).then(|| Source::Steam.lookup_key(appid));
    }
    if let Some(index) = components.iter().position(|name| name == "userdata") {
        // `<root>/userdata/<steamid>/<appid>` — the tree ends at the appid.
        if components.len() == index + 3 {
            let appid = components.last()?;
            return is_digits(appid).then(|| Source::Steam.lookup_key(appid));
        }
    }
    None
}

/// The located candidate files of one tree: path-grammar matches in the
/// walk's byte order, at most `MAX_CANDIDATE_FILES` of them **opened** —
/// the open cap is counted on the opens themselves, not on the located set,
/// so the README's "the product caps the opens too" holds whatever the
/// matches contain; a further match is skipped and counted the way an entry
/// past `MAX_LOCATE_ENTRIES_PER_TREE` is. A match a *reader* refusal
/// (`FileOverBudget`, `UndecodableText`) falls to cannot pass the content
/// test either, so it is located on path grammar alone and `extract` refuses
/// it by name; an `UnreadableFile` — an entry the OS will not stat or open —
/// is skipped and counted, never a candidate. A decodable file without the
/// section is not located. Claims iff located.
fn locate(tree: &UserTree) -> Vec<PathBuf> {
    let mut candidates = UserCandidates {
        user: tree.user.clone(),
        files: Vec::new(),
        tree_root: tree.root.clone(),
    };
    let mut examined = 0usize;
    // §The first family's open cap: counted on every grammar match the walk
    // attempts to test, independent of what the read and content test find.
    let mut opened = 0usize;
    // Byte-ordered level walk: the entry budget is what stops it, so shallow
    // noise consumes the budget before deeper grammar is examined (§The
    // first family's stop-and-count law).
    let mut queue: VecDeque<(PathBuf, Vec<String>)> = VecDeque::new();
    queue.push_back((tree.root.clone(), Vec::new()));
    while let Some((dir, relative)) = queue.pop_front() {
        // The budget is tested before another directory is entered: an
        // over-budget walk examines nothing further at all.
        if examined >= MAX_LOCATE_ENTRIES_PER_TREE {
            break; // stop and count the rest
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue; // an unreadable directory is a skip, not a refusal
        };
        // The walk's byte order: `read_dir` return order never reaches it.
        // The lossy name is the sort key only — the walked path is always
        // `DirEntry::path()`, whose bytes the OS actually named, so a
        // non-UTF-8 name is never re-materialized into a candidate that
        // never existed.
        let mut names: Vec<(String, std::fs::DirEntry)> = entries
            .flatten()
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                (name, entry)
            })
            .collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, entry) in names {
            if examined >= MAX_LOCATE_ENTRIES_PER_TREE {
                return candidates.files; // stop and count the rest
            }
            examined += 1;
            // `DirEntry::file_type()` does not follow symlinks: a link is
            // never descended, never read as a file, never an anchor.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let mut child_relative = relative.clone();
            child_relative.push(name);
            let path = entry.path();
            if file_type.is_dir() {
                // Prune a directory whose tree-relative path is at the
                // depth bound: its files could only sit past it.
                if child_relative.len() < MAX_LOCATE_DEPTH {
                    queue.push_back((path, child_relative));
                }
                continue;
            }
            if !grammar_match(&child_relative) {
                continue;
            }
            if opened >= MAX_CANDIDATE_FILES {
                continue; // a further match is skipped and counted
            }
            opened += 1;
            match read::open_capped(&path).and_then(read::decode_owned) {
                Ok(text) => {
                    if states_bindings(&text) {
                        let _ = candidates.push(path); // within the cap
                    }
                }
                // A match the reader refuses is located on path grammar
                // alone; `extract` refuses it by name, never unseen.
                Err(
                    ExtractionRefusal::FileOverBudget { .. }
                    | ExtractionRefusal::UndecodableText { .. },
                ) => {
                    let _ = candidates.push(path);
                }
                // An entry the OS will not stat or open is a skip and a
                // count — a path that answers no read is never a candidate.
                Err(_) => {}
            }
        }
    }
    candidates.files
}

/// Whether one tree-relative component list spells
/// `…/Saved/Config/<one directory>/<name>.ini` inside `MAX_LOCATE_DEPTH`.
/// The match is **byte-exact (case-exact) by design**: `Saved`, `Config` and
/// the `.ini` suffix compare octet-for-octet, which is UE's own spelling on
/// the case-sensitive homes this family ships for. On a case-insensitive
/// filesystem a case-folded `saved/config` or `X.INI` tree is simply outside
/// the grammar — a silent non-claim, not a misread; widening the match would
/// be a grammar change, not a bug fix (§The first family).
fn grammar_match(relative: &[String]) -> bool {
    let depth = relative.len();
    (4..=MAX_LOCATE_DEPTH).contains(&depth)
        && relative[depth - 4] == "Saved"
        && relative[depth - 3] == "Config"
        && relative[depth - 1].ends_with(".ini")
        && relative[depth - 1].len() > 4
}

/// Whether the decoded text carries the section with at least one
/// `Bindings=` line in it.
fn states_bindings(text: &str) -> bool {
    let mut in_section = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[')
            && let Some(section) = rest.strip_suffix(']')
        {
            in_section = section == SECTION;
            continue;
        }
        if in_section && bindings_line(line).is_some() {
            return true;
        }
    }
    false
}

/// The merge operator a section line carries and its value text, or `None`
/// for a line out of the `Bindings=` grammar.
fn bindings_line(line: &str) -> Option<(Operator, &str)> {
    let (operator, rest) = match line.as_bytes().first()? {
        b'+' => (Operator::Add, &line[1..]),
        b'.' => (Operator::Add, &line[1..]),
        b'-' => (Operator::Remove, &line[1..]),
        b'!' => (Operator::Remove, &line[1..]),
        _ => (Operator::Add, line),
    };
    let value = rest.strip_prefix("Bindings=")?;
    Some((operator, value.trim()))
}

/// What a located `Bindings=` line does to the working set.
#[derive(Debug, Clone, Copy)]
enum Operator {
    Add,
    Remove,
}

/// The working set of one tree's fold: distinct (`Command`, `Name`) pairs as
/// admitted, with the `017-B2` budget counting *as admitted* — a removal
/// frees no room.
#[derive(Default)]
struct Fold {
    pairs: Vec<(String, String)>,
    seen: HashSet<(String, String)>,
    admitted: usize,
}

impl Fold {
    fn add(&mut self, action: &str, input: &str) -> Result<(), ExtractionRefusal> {
        let pair = (action.to_string(), input.to_string());
        if !self.seen.insert(pair.clone()) {
            return Ok(()); // byte-identical pairs collapse; nothing new admitted
        }
        self.pairs.push(pair);
        self.admitted += 1;
        if self.admitted > crate::snapshot::MAX_SNAPSHOT_ENTRIES {
            return Err(ExtractionRefusal::SnapshotEntryBudget {
                limit: crate::snapshot::MAX_SNAPSHOT_ENTRIES,
            });
        }
        Ok(())
    }

    fn remove(&mut self, action: &str, input: &str) {
        // `seen` and `pairs` are maintained in lockstep by `add`/`remove`/
        // `clear`, so set membership is exact: a removal naming a pair the
        // working set never held is a no-op, and one that names a held pair
        // pays a single `retain` — removal lines over a hostile file cannot
        // scan a set they never entered.
        let key = (action.to_string(), input.to_string());
        if !self.seen.remove(&key) {
            return;
        }
        self.pairs.retain(|pair| pair != &key);
    }

    fn clear(&mut self) {
        self.seen.clear();
        self.pairs.clear();
        // `admitted` is kept: removals free no room in the budget.
    }

    /// Entries keyed by action byte order, inputs deduplicated and byte-sorted
    /// (§Determinism); a `Command` whose pairs were all removed leaves no entry.
    fn entries(self) -> Vec<SnapshotEntry> {
        let mut grouped: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (action, input) in &self.pairs {
            grouped
                .entry(action.as_str())
                .or_default()
                .insert(input.as_str());
        }
        grouped
            .into_iter()
            .map(|(action, inputs)| SnapshotEntry {
                action: action.to_string(),
                inputs: inputs.into_iter().map(str::to_string).collect(),
            })
            .collect()
    }
}

/// Folds one located file's `[/Script/Engine.PlayerInput]` section into
/// `fold`. Blank lines and out-of-grammar lines in the section are skipped;
/// a bare `!Bindings=` clears every pair accumulated for the key, a
/// `!Bindings=(…)` removes only the pairs it names, and a bare `+`, `-` or
/// `.` operand is an out-of-grammar skip; a `Bindings=(…)` tuple that lacks
/// a quoted `Name=` or `Command=`, carries an unexpected key, or is
/// unbalanced is `UnparsableEntry` naming the relative path and 1-based line.
fn fold_file(text: &str, relative: &str, fold: &mut Fold) -> Result<(), ExtractionRefusal> {
    let mut in_section = false;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[')
            && let Some(section) = rest.strip_suffix(']')
        {
            in_section = section == SECTION;
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((operator, value)) = bindings_line(line) else {
            continue;
        };
        let bare = line.as_bytes().first().is_some_and(|byte| *byte == b'!');
        if value.is_empty() {
            if bare {
                // A bare `!Bindings=` (no value) clears every pair the fold
                // has accumulated for that key.
                fold.clear();
            }
            // `Bindings=`, `+Bindings=`, `-Bindings=`, `.Bindings=` with no
            // operand are out-of-grammar skips, not drift.
            continue;
        }
        let (name, command) =
            parse_tuple(value).map_err(|detail| ExtractionRefusal::UnparsableEntry {
                path: relative.to_string(),
                line: index + 1,
                detail,
            })?;
        match operator {
            Operator::Remove => fold.remove(&command, &name),
            Operator::Add => fold.add(&command, &name)?,
        }
    }
    Ok(())
}

/// Parses one `(Name="…",Command="…")` tuple: balanced parentheses, exactly
/// the two documented keys, both values quoted, and every token inside the
/// native-name grammar — the bound's name is carried when a verbatim string
/// is over `MAX_NATIVE_NAME_OCTETS`.
fn parse_tuple(value: &str) -> Result<(String, String), String> {
    let inner = value
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .ok_or_else(|| "the Bindings tuple is unbalanced".to_string())?;
    let parts = split_top_level(inner);
    if parts.len() != 2 {
        return Err(format!(
            "expected exactly a Name= and a Command= pair, found {} parts",
            parts.len()
        ));
    }
    let mut name: Option<String> = None;
    let mut command: Option<String> = None;
    for part in parts {
        let part = part.trim();
        let Some((key, raw)) = part.split_once('=') else {
            return Err(format!("`{}` is not a Key=Value entry", short(part)));
        };
        let key = key.trim();
        if key != "Name" && key != "Command" {
            return Err(format!(
                "unexpected key `{}` in the Bindings tuple",
                short(key)
            ));
        }
        let Some(token) = raw
            .trim()
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            return Err(format!("{key}= must carry a quoted value"));
        };
        let slot = if key == "Name" {
            &mut name
        } else {
            &mut command
        };
        if slot.is_some() {
            return Err(format!("{key}= appears twice in one tuple"));
        }
        *slot = Some(token.to_string());
    }
    let name = name.ok_or_else(|| "the tuple lacks a quoted Name=".to_string())?;
    let command = command.ok_or_else(|| "the tuple lacks a quoted Command=".to_string())?;
    for (key, token) in [("Name", &name), ("Command", &command)] {
        if token.is_empty() {
            return Err(format!("{key}= is empty"));
        }
        if token.len() > MAX_NATIVE_NAME_OCTETS {
            return Err(format!(
                "{key}= string is {} octets, past MAX_NATIVE_NAME_OCTETS ({MAX_NATIVE_NAME_OCTETS})",
                token.len()
            ));
        }
        if token.chars().any(char::is_control) || token.starts_with(' ') || token.ends_with(' ') {
            return Err(format!(
                "{key}= carries a control character or an edge space"
            ));
        }
    }
    Ok((name, command))
}

/// Splits on `,` outside double quotes.
fn split_top_level(text: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    for (offset, ch) in text.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                parts.push(&text[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

/// A value echo bounded for a refusal detail.
fn short(text: &str) -> String {
    bindforge_core::escape_and_truncate(text)
}

/// The located file's path relative to its tree — the spelling
/// `provenance.files` and refusal messages carry. The contract spelling is
/// built once, `/`-joined component by component (§`bindings/v1`: "the same
/// layout states the same document on any machine"): a host whose native
/// separator is not `/` still states the same tree-relative path, the
/// byte-order sort stays `/`-based, and the loader's component count reads
/// the same depth everywhere. Absolute probe paths stay host-native
/// (§The CLI face: Local paths).
fn tree_relative(file: &Path, tree_root: &Path) -> String {
    file.strip_prefix(tree_root)
        .unwrap_or(file)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join("/")
}

/// Reads one located file through the capped reader and decodes it, refusing
/// by name with the tree-relative path (never an absolute path the fold
/// happened to hold).
fn read_and_decode(file: &Path, relative: &str) -> Result<String, ExtractionRefusal> {
    let bytes = read::open_capped(file).map_err(|refusal| match refusal {
        ExtractionRefusal::FileOverBudget { octets, limit, .. } => {
            ExtractionRefusal::FileOverBudget {
                path: relative.to_string(),
                octets,
                limit,
            }
        }
        ExtractionRefusal::UnreadableFile { .. } => ExtractionRefusal::UnreadableFile {
            path: relative.to_string(),
        },
        other => other,
    })?;
    read::decode_owned(bytes).map_err(|_| ExtractionRefusal::UndecodableText {
        path: relative.to_string(),
    })
}
