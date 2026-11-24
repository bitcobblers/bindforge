//! `discovery bindings <slug>` — the ADR-017 extraction face (`017-C4`).
//!
//! The face is local by construction: it builds the Steam plugins exactly as
//! `list` does (same host probe, same `--steam-root` override and its
//! `cfg(windows)` selection) and then stops — no catalog client, no gate probe,
//! no match store, no prompt, no candidate query. That is Options' A13
//! divergence from `017-C4`'s "the same scan", and it is what makes the ADR's
//! privacy consequence true of this face (`017-L1`: the run reads and prints,
//! and writes nothing anywhere).
//!
//! The order is the design's §The CLI face: select the records whose lookup key
//! equals the slug (no match is the ordinary failure voice, exit 1), resolve
//! each through `GameContext::from_record` → `registry::claim` → `extract`
//! (`017-C5`'s two passes live in the registry, and `--probe` stops at the
//! claim), render through the shared width algorithm, and exit the
//! highest-ranked status the run reported on that status's own code.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use bindforge_codecs::codec::{Detection, Evidence, UserCandidates};
use bindforge_codecs::context::GameContext;
use bindforge_codecs::registry;
use bindforge_codecs::snapshot::BindingSnapshot;
use discovery::game::ScanRecord;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;

use crate::{cell, refused, render_grid};

/// The `bindings` table's columns, left to right (`017-C4`).
const COLUMNS: [&str; 4] = ["Codec", "User", "Action", "Inputs"];

/// The `--probe` table's columns. Probe output is a local operator tool, so it
/// names the files it located by absolute path (§The CLI face: Local paths).
const PROBE_COLUMNS: [&str; 4] = ["Family", "Evidence", "User", "File"];

/// The four named statuses, declared in the order §The CLI face step 5 ranks
/// them — `extraction-refused` > `family-confirmed-files-absent` >
/// `claim-refused` > `ok` — so the `Ord` derive *is* the ranking and
/// "highest-ranked status reported" is one `max` call. All four codes sit
/// above the 2 clap's parser returns for a usage error before `main` runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    /// Everything the run located, it extracted (exit 0, and the one status
    /// that prints no `status: ` line).
    Ok,
    /// No shipped grammar located the game (`017-L2`, exit 3).
    ClaimRefused,
    /// A family claimed the game and the located store states nothing readable
    /// (`017-L2`, exit 4).
    FamilyConfirmedFilesAbsent,
    /// A located file the claiming codec refused to parse, by name (exit 5).
    ExtractionRefused,
}

impl Status {
    /// The name the `status: ` line and the status document spell.
    fn token(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::ClaimRefused => "claim-refused",
            Status::FamilyConfirmedFilesAbsent => "family-confirmed-files-absent",
            Status::ExtractionRefused => "extraction-refused",
        }
    }

    /// The status's own exit code.
    fn code(self) -> ExitCode {
        match self {
            Status::Ok => ExitCode::SUCCESS,
            Status::ClaimRefused => ExitCode::from(3),
            Status::FamilyConfirmedFilesAbsent => ExitCode::from(4),
            Status::ExtractionRefused => ExitCode::from(5),
        }
    }
}

/// The record seam both Steam plugins expose as inherent methods, named once so
/// the plugin selection below is one typed list rather than two copies of the
/// loop. `scan_records()` and `roots()` are deliberately not on the
/// `DiscoveryPlugin` trait, which is what keeps `017-L5` structural.
trait Records {
    fn name(&self) -> &'static str;
    fn scan_records(&self) -> Result<Vec<ScanRecord>, discovery::registry::DiscoveryError>;
    fn roots(&self) -> &[PathBuf];
}

impl Records for SteamLinuxPlugin {
    fn name(&self) -> &'static str {
        discovery::plugin::DiscoveryPlugin::name(self)
    }

    fn scan_records(&self) -> Result<Vec<ScanRecord>, discovery::registry::DiscoveryError> {
        SteamLinuxPlugin::scan_records(self)
    }

    fn roots(&self) -> &[PathBuf] {
        SteamLinuxPlugin::roots(self)
    }
}

impl Records for SteamWindowsPlugin {
    fn name(&self) -> &'static str {
        discovery::plugin::DiscoveryPlugin::name(self)
    }

    fn scan_records(&self) -> Result<Vec<ScanRecord>, discovery::registry::DiscoveryError> {
        SteamWindowsPlugin::scan_records(self)
    }

    fn roots(&self) -> &[PathBuf] {
        SteamWindowsPlugin::roots(self)
    }
}

/// One family's claims across every matching record that resolved it, merged so
/// a tree two records both located states itself once (§Determinism's
/// tree-identity dedup — the fact that makes "more than one record matching is
/// not an error" safe).
struct MergedClaim {
    family: &'static str,
    evidence: Evidence,
    groups: Vec<UserCandidates>,
}

impl MergedClaim {
    fn open(claim: &Detection) -> Self {
        Self {
            family: claim.family,
            evidence: claim.evidence,
            groups: Vec::new(),
        }
    }

    fn absorb(&mut self, claim: &Detection) {
        for group in &claim.per_user {
            if self
                .groups
                .iter()
                .any(|seen| seen.tree_root == group.tree_root)
            {
                continue;
            }
            self.groups.push(group.clone());
        }
    }

    /// The `Detection` the claiming family's own `extract` runs on: its groups
    /// in tree-identity byte order.
    fn detection(&self) -> Detection {
        let mut groups = self.groups.clone();
        groups.sort_by(|a, b| a.tree_root.cmp(&b.tree_root));
        Detection {
            family: self.family,
            evidence: self.evidence,
            per_user: groups,
        }
    }
}

/// Runs `discovery bindings <slug>` end to end.
pub(crate) fn run(json: bool, slug: &str, steam_root: &[PathBuf], probe: bool) -> ExitCode {
    // Step 1 — the scan, with `list`'s plugin selection and nothing else. Each
    // record keeps its plugin's whole root set: A18 puts `userdata/` under a
    // root no scan row labels, so the resolvers search every root (`017-C2`).
    let mut plugins: Vec<Box<dyn Records>> = Vec::new();
    if steam_root.is_empty() {
        plugins.push(Box::new(SteamLinuxPlugin::from_host()));
        plugins.push(Box::new(SteamWindowsPlugin::from_host()));
    } else {
        #[cfg(not(windows))]
        plugins.push(Box::new(SteamLinuxPlugin::from_roots(
            steam_root.iter().cloned(),
        )));
        #[cfg(windows)]
        plugins.push(Box::new(SteamWindowsPlugin::from_roots(
            steam_root.iter().cloned(),
        )));
    }

    let mut matched: Vec<(ScanRecord, Vec<PathBuf>)> = Vec::new();
    for plugin in &plugins {
        match plugin.scan_records() {
            Ok(records) => {
                let roots = plugin.roots().to_vec();
                matched.extend(
                    records
                        .into_iter()
                        .filter(|record| record.game.slug == slug)
                        .map(|record| (record, roots.clone())),
                );
            }
            // A scan failure is today's one voice, exit 1.
            Err(error) => {
                return refused(format!("plugin {} failed: {error:?}", plugin.name()));
            }
        }
    }
    // Step 3's order is the scan's own `(slug, install_path)` order, kept
    // across the plugins so a run over several records is deterministic.
    matched.sort_by(|a, b| {
        (&a.0.game.slug, &a.0.game.install_path, &a.0.library).cmp(&(
            &b.0.game.slug,
            &b.0.game.install_path,
            &b.0.library,
        ))
    });
    if matched.is_empty() {
        // Step 2 — no match is the ordinary failure voice: the escaped slug,
        // and no local path.
        return refused(format!(
            "no discovered game named {}",
            bindforge_core::escape_and_truncate(slug)
        ));
    }

    // Step 3 — one claim per matching record, merged per family. A row that is
    // not a Steam install gets no `GameContext` at all, which is `claim-refused`
    // (`017-C2`); so does a Steam row no shipped grammar locates (`017-L2`).
    // Two records of one game in two libraries hand the resolvers the same
    // `(appid, library, roots)` anchors, so the claim is memoized per anchor
    // triple: the tree walk behind a claim is paid once per tree set, and
    // §Determinism's tree-identity dedup still merges the groups.
    let mut claims: BTreeMap<&'static str, MergedClaim> = BTreeMap::new();
    let mut claim_route: BTreeMap<(String, PathBuf, Vec<PathBuf>), Option<Detection>> =
        BTreeMap::new();
    let mut status = Status::Ok;
    for (record, roots) in &matched {
        let claim = match GameContext::from_record(record, roots) {
            None => None,
            Some(ctx) => claim_route
                .entry((ctx.appid.clone(), ctx.library.clone(), ctx.roots.clone()))
                .or_insert_with(|| registry::claim(&ctx))
                .clone(),
        };
        match claim {
            Some(claim) => {
                claims
                    .entry(claim.family)
                    .or_insert_with(|| MergedClaim::open(&claim))
                    .absorb(&claim);
            }
            None => status = std::cmp::max(status, Status::ClaimRefused),
        }
    }

    if probe {
        // `--probe` prints the claims and extracts nothing.
        if status == Status::Ok && claims.is_empty() {
            status = Status::ClaimRefused;
        }
        return report(json, status, &claims, &[], None, probe);
    }

    // Step 3 (continued) — each claiming family extracts its merged claim, in
    // family-id byte order, one document per located tree.
    let mut documents: Vec<BindingSnapshot> = Vec::new();
    let mut refusal: Option<String> = None;
    for (family, merged) in &claims {
        let Some(codec) = registry::family(family) else {
            // A claim whose family is not in the registry cannot be read.
            status = std::cmp::max(status, Status::ClaimRefused);
            continue;
        };
        match codec.extract(&merged.detection()) {
            Ok(docs) => {
                if !docs.iter().any(|doc| !doc.entries.is_empty()) {
                    status = std::cmp::max(status, Status::FamilyConfirmedFilesAbsent);
                }
                documents.extend(docs);
            }
            // Step 5 — `extraction-refused`, and the `error: ` line names the
            // `ExtractionRefusal` variant.
            Err(error) => {
                refusal = Some(format!("{error}"));
                status = std::cmp::max(status, Status::ExtractionRefused);
            }
        }
    }
    if status == Status::Ok && documents.is_empty() {
        status = Status::ClaimRefused;
    }

    report(json, status, &claims, &documents, refusal.as_deref(), probe)
}

/// Writes the run out (§The CLI face steps 4 and 5): exactly one `status: `
/// line for a non-ok status — `ok` prints none — plus the refusal's `error: `
/// line for `extraction-refused`, all on stderr. stdout stays the table or the
/// compact JSON document, and a status that produced nothing prints nothing
/// there: with `--json` a non-ok status prints its one status document.
fn report(
    json: bool,
    status: Status,
    claims: &BTreeMap<&'static str, MergedClaim>,
    documents: &[BindingSnapshot],
    refusal: Option<&str>,
    probe: bool,
) -> ExitCode {
    if status != Status::Ok {
        eprintln!("status: {}", status.token());
        if let Some(message) = refusal {
            eprintln!("error: {message}");
        }
    }
    if status != Status::Ok {
        // Nothing on stdout is the whole story for a refused run: the status
        // line (and, in `--json`, the one status document) names it.
        if json {
            let mut document = serde_json::json!({ "status": status.token() });
            if probe {
                document["claims"] = serde_json::Value::Array(Vec::new());
            }
            let text = match serde_json::to_string(&document) {
                Ok(text) => text,
                Err(error) => return refused(format!("failed to serialize status: {error}")),
            };
            println!("{text}");
        }
        return status.code();
    }

    let rendered: String = if json && probe {
        let entries: Vec<serde_json::Value> = claims
            .iter()
            .flat_map(|(family, merged)| {
                let detection = merged.detection();
                detection.per_user.into_iter().map(move |group| {
                    serde_json::json!({
                        "family": family,
                        "evidence": merged.evidence.token(),
                        "user": group.user,
                        "files": group
                            .files
                            .iter()
                            .map(|file| file.display().to_string())
                            .collect::<Vec<String>>(),
                    })
                })
            })
            .collect();
        let document = serde_json::json!({ "status": status.token(), "claims": entries });
        match serde_json::to_string(&document) {
            Ok(text) => text,
            Err(error) => return refused(format!("failed to serialize claims: {error}")),
        }
    } else if json {
        match serde_json::to_string(documents) {
            Ok(text) => text,
            Err(error) => return refused(format!("failed to serialize snapshots: {error}")),
        }
    } else if probe {
        let rows: Vec<Vec<String>> = claims
            .values()
            .flat_map(|merged| {
                merged.groups.iter().flat_map(move |group| {
                    group.files.iter().map(move |file| {
                        vec![
                            cell(merged.family.to_string()),
                            cell(merged.evidence.token().to_string()),
                            cell(group.user.clone()),
                            cell(file.display().to_string()),
                        ]
                    })
                })
            })
            .collect();
        render_grid(&PROBE_COLUMNS.map(str::to_string), &rows)
    } else {
        let rows: Vec<Vec<String>> = documents
            .iter()
            .flat_map(|doc| {
                doc.entries.iter().map(|entry| {
                    vec![
                        cell(doc.codec.clone()),
                        cell(doc.user.clone().unwrap_or_else(|| "-".to_string())),
                        cell(entry.action.clone()),
                        cell(entry.inputs.join(", ")),
                    ]
                })
            })
            .collect();
        render_grid(&COLUMNS.map(str::to_string), &rows)
    };

    println!("{rendered}");
    status.code()
}
