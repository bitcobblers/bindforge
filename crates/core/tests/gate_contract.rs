//! The CI gate contract: what the repo's two workflows promise, pinned at the
//! level a test can reach.
//!
//! Test strategy #5 ("from a fresh clone with no `DATABASE_URL` set and no
//! Postgres reachable, every command listed in the Gates block succeeds on the
//! Linux CI runner, with nothing installed beyond the pinned toolchain") and #6
//! ("a PR that breaks only TS fails `orchestration` while `rust` passes; a PR
//! that breaks only Rust fails `rust` while `orchestration` passes") are rollout
//! and CI observations: no test file can run a GitHub Actions job, and
//! Test strategy #7's platform legs (the gate green on the Linux runner, and the
//! same commands once on a Windows contributor machine) are dated rollout checks
//! recorded in the build notes (Rollout 7). What this suite pins is the
//! precondition those legs are run against — that `.github/workflows/check.yml`
//! runs exactly the three gate commands, in a job that needs no database, in two
//! jobs that cannot break each other, with the pins the design fixes. If these
//! assertions hold, the rollout checks have a defined thing to verify; if they
//! fail, the one-gate promise is broken before anyone clones the repo.

mod common;

use std::fs;

use common::repo_root;

/// Reads a file relative to the workspace root.
fn workspace_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
}

/// The top-level entries of a workflow's `jobs:` mapping, each with its block.
/// Hand-rolled because a suite that pulls in a YAML parser would be asserting
/// against a dependency rather than against the workflow text the design fixes.
fn job_blocks(workflow: &str) -> Vec<(String, String)> {
    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut inside_jobs = false;
    for line in workflow.lines() {
        if line.trim_end() == "jobs:" {
            inside_jobs = true;
            continue;
        }
        if !inside_jobs {
            continue;
        }
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if line.starts_with("  ") && !line.starts_with("   ") {
            blocks.push((line.trim().trim_end_matches(':').to_string(), String::new()));
            continue;
        }
        if let Some((_, body)) = blocks.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    blocks
}

fn job_block(jobs: &[(String, String)], name: &str) -> String {
    let found = jobs.iter().find(|(job, _)| job == name);
    let block = found.unwrap_or_else(|| {
        panic!(
            "the workflow declares no `{name}` job; it has {:?}",
            jobs.iter().map(|(job, _)| job.as_str()).collect::<Vec<_>>()
        )
    });
    block.1.clone()
}

/// The executable lines of a job block — `run:`, `with:`, `env:`, `image:`,
/// `services:` — comments and step names excluded, since prose may name the
/// very thing a rule forbids.
fn executable_lines(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#') && !line.starts_with("- name:"))
        .filter(|line| {
            line.contains("run:")
                || line.contains("with:")
                || line.contains("env:")
                || line.contains("image:")
                || line.contains("services:")
        })
        .map(str::to_string)
        .collect()
}

/// Every `uses:` reference in a workflow, with its trailing comment stripped.
fn action_references(workflow: &str) -> Vec<String> {
    workflow
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("- uses:"))
        .map(|reference| {
            reference
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}

/// The text after `marker` up to the next `"` — the shape these YAML, TOML and
/// JSON files all use for a quoted scalar.
fn value_after(text: &str, marker: &str) -> Option<String> {
    let rest = text.split_once(marker)?.1;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// The workflow's non-comment lines, trimmed: a header or step comment may name
/// the very spelling a rule forbids, in order to warn against it.
fn code_lines(workflow: &str) -> Vec<&str> {
    workflow
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .collect()
}

/// The scheduled scan's own `cargo deny` command lines.
fn scan_command_lines(workflow: &str) -> Vec<String> {
    code_lines(workflow)
        .into_iter()
        .filter(|line| line.contains("cargo deny"))
        .map(str::to_string)
        .collect()
}

/// Which cargo-deny format a scan line captures its report in — or `None` when
/// the line captures nothing. Which stream carries the report is a fact about
/// the pinned 0.20.2, not a choice the workflow makes: with `--format json`
/// every record the tool produces (diagnostics, log records, the run summary)
/// goes to stderr, and only SARIF is written to stdout. So the two sanctioned
/// spellings are `--format sarif` with a stdout redirect and `--format json`
/// with a `2>` redirect; a stdout redirect of the JSON spelling captures an
/// empty file while the scan runs and its findings disappear.
fn report_capture(line: &str) -> Option<&'static str> {
    let to_stderr = line.contains("2>");
    let to_stdout = line.replace("2>", "").contains('>');
    if line.contains("--format sarif") && to_stdout {
        return Some("sarif");
    }
    if line.contains("--format json") && to_stderr {
        return Some("json");
    }
    None
}

/// The file a `>` or `2>` redirect on a command line writes to.
fn redirect_target(line: &str) -> Option<String> {
    let target = line.split_once('>')?.1.split_whitespace().next()?;
    if target.is_empty() {
        return None;
    }
    Some(target.to_string())
}

/// The quoted entries of a bracketed list, e.g. `members = ["crates/*"]`.
fn quoted_list(text: &str, marker: &str) -> Vec<String> {
    let at = match text.find(marker) {
        Some(at) => at + marker.len(),
        None => return Vec::new(),
    };
    let rest = &text[at..];
    let end = match rest.find(']') {
        Some(end) => end,
        None => return Vec::new(),
    };
    rest[..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// proves: Test strategy #5 — the PR gate's `rust` job runs the three cargo gate
/// commands, verbatim as the design's Gates block states them, and needs no
/// database. The commands themselves are what #7's platform legs run — once on
/// the Linux CI runner, once on a Windows contributor machine — and those dated
/// runs are recorded in the build notes (Rollout 7).
#[test]
fn the_rust_gate_runs_exactly_the_three_gate_commands_with_no_database() {
    let check = workspace_file(".github/workflows/check.yml");
    let jobs = job_blocks(&check);
    let names: Vec<&str> = jobs.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        ["rust", "orchestration"],
        "the design fixes two independent jobs per PR (Test strategy #6); the workflow declares \
         {names:?}"
    );

    let rust = job_block(&jobs, "rust");
    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace --locked",
    ] {
        assert!(
            rust.contains(command),
            "the `rust` job must run `{command}`: the Gates block is the whole gate, and Test \
             strategy #5 cites those commands rather than restating them"
        );
    }

    assert!(
        !rust.contains("services:"),
        "the cargo gate must declare no service — Test strategy #5 runs it with no Postgres \
         reachable and no `DATABASE_URL` set"
    );
    for line in executable_lines(&rust) {
        assert!(
            !line.contains("DATABASE_URL") && !line.to_lowercase().contains("postgres"),
            "the `rust` job must need no database; it runs `{line}`"
        );
    }
}

/// proves: Test strategy #6 — the two jobs are independent at the level the
/// workflow structure can show: the `rust` job never touches npm, the
/// `orchestration` job never touches cargo. A PR that breaks only TypeScript can
/// therefore fail one job while the other passes, and vice versa.
#[test]
fn the_two_gate_jobs_cannot_break_each_other() {
    let check = workspace_file(".github/workflows/check.yml");
    let jobs = job_blocks(&check);

    let rust_lines = executable_lines(&job_block(&jobs, "rust"));
    let orchestration_lines = executable_lines(&job_block(&jobs, "orchestration"));

    assert!(
        rust_lines.iter().any(|line| line.contains("cargo "))
            && !rust_lines.iter().any(|line| line.contains("npm ")),
        "the `rust` job must be cargo-only, or a broken TypeScript build fails it: {rust_lines:?}"
    );
    assert!(
        orchestration_lines
            .iter()
            .any(|line| line.contains("npm run check"))
            && !orchestration_lines
                .iter()
                .any(|line| line.contains("cargo ")),
        "the `orchestration` job must be npm-only and run the package's own `npm run check`: \
         {orchestration_lines:?}"
    );
}

/// proves: Chosen design → Gates — every third-party action in both workflows is
/// pinned to a full commit SHA, and every job declares read-only contents
/// permission and a timeout.
#[test]
fn every_action_is_pinned_and_every_job_declares_read_only_permissions_and_a_timeout() {
    for workflow_path in [
        ".github/workflows/check.yml",
        ".github/workflows/advisories.yml",
    ] {
        let workflow = workspace_file(workflow_path);

        for reference in action_references(&workflow) {
            let (_action, revision) = reference.split_once('@').unwrap_or_else(|| {
                panic!(
                    "{workflow_path} uses `{reference}`; every third-party action is pinned to a \
                     commit SHA, not a floating tag"
                )
            });
            assert_eq!(
                revision.len(),
                40,
                "{workflow_path} pins `{reference}` to something shorter than a full commit SHA"
            );
            assert!(
                revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{workflow_path} pins `{reference}` to something that is not a commit SHA"
            );
        }

        let jobs = job_blocks(&workflow);
        assert!(!jobs.is_empty(), "{workflow_path} declares no jobs");
        for (name, block) in &jobs {
            assert!(
                block.contains("permissions:") && block.contains("contents: read"),
                "job `{name}` of {workflow_path} must set `permissions: contents: read` at the \
                 job level"
            );
            assert!(
                block.contains("timeout-minutes"),
                "job `{name}` of {workflow_path} must time out: a hung job holds a runner, and on \
                 the scheduled scan it means no report that week"
            );
        }
    }
}

/// proves: Chosen design → Gates + A28 — the Node version CI installs is
/// `tools/orchestration`'s own `engines.node` floor, read from the package
/// rather than restated in the workflow, so the one Node floor this design sets
/// cannot drift.
#[test]
fn the_orchestration_job_pins_the_node_floor_the_package_declares() {
    let check = workspace_file(".github/workflows/check.yml");
    let manifest = workspace_file("tools/orchestration/package.json");

    let declared = value_after(&manifest, "\"node\": \"")
        .expect("tools/orchestration/package.json declares its engines.node floor")
        .trim_start_matches(">=")
        .to_string();
    let installed = value_after(&check, "node-version: \"")
        .expect("the orchestration job pins setup-node's node-version")
        .trim_start_matches(">=")
        .to_string();

    assert_eq!(
        installed, declared,
        "setup-node must install the floor tools/orchestration declares (`{declared}`), not a \
         second number that can drift from it"
    );
}

/// proves: Chosen design → Gates (the scheduled scan, Rollout 4) — the lockfile
/// scan is never a PR check, runs the whole `cargo deny check` so every
/// `deny.toml` section has a consumer, spells the command in the form the pinned
/// cargo-deny parses (its shared options sit at the root, before the
/// subcommand), captures the report from the stream that pinned version actually
/// writes it to, proves the captured file is a report by content rather than by
/// size, builds the scanner on its own pinned toolchain rather than the product
/// pin, and fails loudly when no report was produced — so a dead control cannot
/// idle unnoticed. The capture and the marker are pinned in either sanctioned
/// spelling (`--format sarif` on stdout, or `--format json` with `2>`), so a
/// correct fix cannot break this suite.
#[test]
fn the_lockfile_scan_is_scheduled_not_a_pr_check_and_builds_on_its_own_toolchain() {
    let scan = workspace_file(".github/workflows/advisories.yml");
    let toolchain = workspace_file("rust-toolchain.toml");
    let deny = workspace_file("deny.toml");

    assert!(
        !scan.contains("pull_request"),
        "the advisory scan must not run on a PR: its advisory database is live network state, so \
         as a PR check it would turn an unchanged PR red on any day the database moves"
    );
    assert!(
        scan.contains("schedule:") && scan.contains("workflow_dispatch"),
        "the scan runs weekly and on manual dispatch"
    );
    // Only non-comment lines: a header comment may name the form this rule
    // forbids in order to warn against it.
    let scan_commands = scan_command_lines(&scan);
    let report_format = scan_commands
        .iter()
        .map(String::as_str)
        .find_map(report_capture)
        .unwrap_or_else(|| {
            panic!(
                "the scheduled job must capture the report from the stream the pinned cargo-deny \
                 writes it to: 0.20 sends every `--format json` record — diagnostics, log records, \
                 the run summary — to stderr and writes only SARIF to stdout, so the scan line \
                 needs `--format sarif` with a stdout redirect or `--format json` with `2>` — saw \
                 {scan_commands:?}"
            )
        });
    for line in &scan_commands {
        let after_subcommand = line.split_once("check").map_or("", |(_, rest)| rest);
        assert!(
            !after_subcommand.contains("--format"),
            "a shared option after `check` is a parse error in the pinned cargo-deny: it exits \
             before scanning, the report file stays empty, and the weekly job goes red with no \
             scan ever having run: `{line}`"
        );
        // Written on the parsed command rather than on one spelling, so it holds
        // whatever format and redirect the command uses: a scan narrowed to one
        // section leaves the other `deny.toml` policies as knobs without a
        // consumer.
        let restricted: Vec<&str> = ["advisories", "bans", "licenses", "sources"]
            .into_iter()
            .filter(|section| {
                after_subcommand
                    .split_whitespace()
                    .any(|word| word == *section)
            })
            .collect();
        assert!(
            restricted.is_empty(),
            "the scheduled job runs the whole `cargo deny check`; a scan restricted to one section \
             leaves the sections `deny.toml` configures unrun: `{line}`"
        );
    }

    // And the guard step has to prove the captured file *is* a report, by the
    // marker that format always emits. A size test is not that proof: a run that
    // dies loading `deny.toml` or the advisory database still writes an error
    // line, and a scan whose findings went to the uncaptured stream leaves an
    // empty file — `[ ! -s … ]` was green for the first and red for the second,
    // which is how a dead control survived two rounds of this suite.
    let markers: &[&str] = if report_format == "sarif" {
        &["\"runs\""]
    } else {
        &["\"summary\""]
    };
    assert!(
        markers.iter().copied().any(|token| scan.contains(token)),
        "the guard step must test the captured file for a content marker of the format the scan \
         line emits — one of {markers:?} for `--format {report_format}` — not for mere \
         non-emptiness"
    );
    for line in code_lines(&scan) {
        assert!(
            !line.contains("[ ! -s"),
            "`[ ! -s … ]` is a size test, not a report: it stays green for an error line and goes \
             red for a scan that ran and reported elsewhere: `{line}`"
        );
    }

    let configured: Vec<&str> = ["advisories", "bans", "licenses", "sources"]
        .into_iter()
        .filter(|section| deny.contains(&format!("[{section}]")))
        .collect();
    assert!(
        !configured.is_empty(),
        "deny.toml configures none of the sections the design says it does"
    );
    assert!(
        !scan.contains("cargo deny check advisories"),
        "deny.toml configures {}; a job running only `check advisories` leaves those policies as \
         knobs without a consumer",
        configured.join(", ")
    );

    assert!(
        scan.contains("timeout-minutes"),
        "a hung cargo-deny install must not silently cost the week's scan"
    );
    assert!(
        scan.contains("produced no report"),
        "a permanently failing scheduled job is a dead control: it removes the repo's only \
         recurring look at the lockfile while looking green to nobody, so it has to be loud"
    );
    assert!(
        scan.contains("CARGO_DENY_VERSION: \""),
        "cargo-deny is a tool rather than a workspace dependency, so the workflow pins its version"
    );

    let product_pin = value_after(&toolchain, "channel = \"")
        .expect("rust-toolchain.toml pins the product toolchain for the checkout");
    let scan_toolchain = value_after(&scan, "SCAN_TOOLCHAIN: \"")
        .expect("the scan names the toolchain it builds cargo-deny on");
    assert_ne!(
        scan_toolchain, product_pin,
        "cargo-deny's MSRV sits above the product pin ({product_pin}); building it under \
         rust-toolchain.toml's channel fails, and the scan silently stops running"
    );
    assert!(
        scan.contains("RUSTUP_TOOLCHAIN=\"$SCAN_TOOLCHAIN\""),
        "both installing the scanner and running it must happen on the scanner's toolchain"
    );
}

/// proves: Chosen design → Gates (the scheduled scan, Rollout 4) — the week's
/// triage record outlives the runner. The report the scan step captures is
/// uploaded by a SHA-pinned `actions/upload-artifact` step, and what it uploads
/// is that captured file. The guard step's `head -c` preview truncates any real
/// finding set and the file itself dies with the job, so "the maintainer triages
/// what it reports within the week" has nothing to triage later without it.
#[test]
fn the_scan_uploads_the_report_it_captured_as_the_weeks_triage_record() {
    let scan = workspace_file(".github/workflows/advisories.yml");

    let scan_commands = scan_command_lines(&scan);
    let captured: Vec<String> = scan_commands
        .iter()
        .map(String::as_str)
        .filter_map(redirect_target)
        .collect();
    assert!(
        !captured.is_empty(),
        "the scan step must redirect its report to a file the job can upload — a scan that only \
         prints findings into the run log has no triage record to upload: {scan_commands:?}"
    );

    let Some((_, after_upload)) = scan.split_once("actions/upload-artifact@") else {
        panic!(
            "the scheduled job uploads its report with a SHA-pinned `actions/upload-artifact` step: \
             the report file exists only on the runner, and the log preview truncates it, so the \
             week's findings would be gone before anyone triaged them"
        )
    };
    let revision = after_upload.split_whitespace().next().unwrap_or_default();
    assert_eq!(
        revision.len(),
        40,
        "`actions/upload-artifact` must be pinned to a full commit SHA, not a floating tag — the \
         upload is the only durable copy of the scan"
    );
    assert!(
        revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "`actions/upload-artifact` is pinned to `{revision}`, which is not a commit SHA"
    );
    for path in &captured {
        assert!(
            after_upload.contains(path.as_str()),
            "the artifact must be the report the scan step captured: `{path}` has to be one of the \
             upload step's inputs, or the job uploads something else and the scan's findings still \
             die with the runner"
        );
    }
}

/// proves: Test strategy #5 — a fresh clone can load the workspace at all: every
/// `members` entry in the root `Cargo.toml` names an existing member crate. A
/// glob matching nothing aborts every gate command at manifest load, and git
/// tracks no empty directory, so a target-state glob is a gate that never runs.
#[test]
fn every_workspace_members_entry_names_an_existing_member() {
    let manifest = workspace_file("Cargo.toml");
    let entries = quoted_list(&manifest, "members = [");
    assert!(
        !entries.is_empty(),
        "the root `Cargo.toml` declares workspace members"
    );

    for entry in &entries {
        if let Some(parent) = entry.strip_suffix("/*") {
            let dir = repo_root().join(parent);
            let Ok(reads) = fs::read_dir(&dir) else {
                panic!(
                    "`members = [\"{entry}\"]` names no directory: `{parent}` is absent from a \
                     fresh clone, so every gate command aborts at manifest load"
                );
            };
            let matched = reads
                .filter_map(|child| child.ok())
                .any(|child| child.path().join("Cargo.toml").is_file());
            assert!(
                matched,
                "`members = [\"{entry}\"]` matches no member crate; a glob matching nothing \
                 aborts every gate command at manifest load, so only globs that match an existing \
                 member belong in `members`"
            );
        } else {
            let member = repo_root().join(entry);
            assert!(
                member.join("Cargo.toml").is_file(),
                "`members = [\"{entry}\"]` must name a crate: {} has no `Cargo.toml`",
                member.display()
            );
        }
    }
}

/// proves: Chosen design → Workspace shape + Test strategy #5 — the toolchain
/// floor is declared once: `rust-toolchain.toml` pins a release of the version
/// the workspace declares as `rust-version`, so a fresh clone builds the
/// toolchain the gate assumes, "with nothing installed beyond the pinned
/// toolchain".
#[test]
fn the_pinned_toolchain_sits_on_the_workspace_rust_version_floor() {
    let toolchain = workspace_file("rust-toolchain.toml");
    let manifest = workspace_file("Cargo.toml");

    let channel = value_after(&toolchain, "channel = \"")
        .expect("rust-toolchain.toml pins the rustup toolchain for the checkout");
    let floor = value_after(&manifest, "rust-version = \"")
        .expect("the workspace declares its toolchain floor exactly once");

    let parts: Vec<&str> = channel.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "the channel must name a specific release, not a rolling one; it is `{channel}`"
    );
    assert_eq!(
        format!("{}.{}", parts[0], parts[1]),
        floor,
        "`rust-version = \"{floor}\"` and `channel = \"{channel}\"` must declare the same floor, \
         and be bumped together"
    );
}
