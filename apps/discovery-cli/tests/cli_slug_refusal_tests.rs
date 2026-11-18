//! ADR-024's CLI-side rendering and refusal pins for `discovery list`: rows reach
//! the table and the JSON array inside the shared slug bound, and a row that is
//! outside it never reaches either.
//!
//! proves: design 024's Test strategy #9, at the seam the repository can reach.
//! The design's line is that "a plugin emitting a row whose slug violates the
//! grammar or bound gets that row dropped by the CLI with one named stderr refusal
//! containing the escaped echo, while valid rows still render". The shipped
//! plugins refuse such a row one seam earlier — `steam_common`'s construction path
//! answers `None` and counts it into the plugin's `skipped` counter, and plugin-side
//! refusals are count-only, never echoed — so no plugin in this workspace can hand
//! the CLI an invalid slug, and `apps/discovery-cli` ships only `src/main.rs`: an
//! integration test can drive the built binary, and the binary builds its own two
//! plugins, so there is no seam at which to inject a bad row. This suite therefore
//! pins the observable half end-to-end: an out-of-bound row is absent from both
//! renderings while the in-bound rows of the same root render, and a root whose
//! every row is refused fails loudly with the game's own text kept out of the
//! diagnostics. The escaped-echo renderer the CLI's refusal line is built from is
//! pinned where it is defined, in `crates/core/tests/error.rs` (Test strategy #7).
//! The unit half of strategy #9 — the CLI's own `refuse_invalid_slugs` filter
//! dropping grammar-invalid and over-bound hand-built rows, valid rows surviving,
//! each refusal line the named prefix plus the escaped echo with no install path —
//! is pinned in-source, in the `#[cfg(test)]` tests of
//! `apps/discovery-cli/src/main.rs` (the r1 F1 resolution); this suite is the
//! end-to-end half.
//!
//! The tests build their own Steam roots under the system temp directory and remove
//! them again (repo practice: integration tests use temp directories and clean up
//! after themselves); no test probes the host or needs Steam installed.

use std::path::PathBuf;
use std::process::Command;

/// The shared slug bound, referenced from the exported constant rather than retyped
/// — `024-B1`'s one-stated-home rule; the crate carries `bindforge-core` in its
/// dependencies, so the tests read the number from the same source as the CLI.
const SLUG_BOUND: usize = bindforge_core::MAX_SLUG_OCTETS;

/// One Steam root in the system temp directory, removed when dropped.
struct SteamRoot {
    path: PathBuf,
}

impl SteamRoot {
    /// A root with one library: `steamapps/`, and one `appmanifest_<appid>.acf` per
    /// call to [`SteamRoot::acf`].
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("bindforge-cli-{}-{}", label, std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(path.join("steamapps")).expect("temp steamapps directory");
        Self { path }
    }

    /// Writes one manifest whose `name` and `installdir` are the given pair.
    fn acf(&self, appid: &str, name: &str, installdir: &str) {
        std::fs::write(
            self.path
                .join("steamapps")
                .join(format!("appmanifest_{appid}.acf")),
            format!(
                "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"{name}\"\n\t\
                 \"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{installdir}\"\n\t\
                 \"SizeOnDisk\"\t\t\"1000000\"\n}}\n"
            ),
        )
        .expect("temp appmanifest");
    }
}

impl Drop for SteamRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Runs the built `discovery-cli` with `args`.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_discovery-cli"))
        .args(args)
        .output()
        .expect("failed to execute discovery-cli")
}

fn stdout_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The cells of one rendered table line, trimmed of the padding the renderer wrote.
fn cells(line: &str) -> Vec<String> {
    let inner = line
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(line);
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// A name whose derived slug is `head + 1 + tail` octets long.
fn letters(head: usize, tail: usize) -> String {
    format!("{} {}", "k".repeat(head), "k".repeat(tail))
}

/// The slug a name of that shape derives to, spelled out for the assertions.
fn letters_slug(head: usize, tail: usize) -> String {
    format!("{}-{}", "k".repeat(head), "k".repeat(tail))
}

/// Whether `slug` is in the shared grammar: lower-case ASCII letters, digits and
/// single hyphens, beginning and ending with an alphanumeric, and inside the bound.
fn is_slug_shaped(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= SLUG_BOUND
        && slug.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && slug.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.contains("--")
}

/// The `Slug` column of every rendered table row.
fn rendered_slugs(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .skip(2)
        .map(|line| cells(line)[0].clone())
        .collect()
}

// proves: design 024's Test strategy #8, #9 (Chosen design: CLI boundary) — the
// bound is enforced at the render seam by length, not by "long names are dropped":
// a row whose slug is exactly `MAX_SLUG_OCTETS` renders, and the row one octet
// longer does not, while both sit in the same root.
#[test]
fn a_slug_at_the_bound_renders_and_one_octet_past_it_does_not() {
    let at_bound = letters(SLUG_BOUND - 64, 63);
    let past_bound = letters(SLUG_BOUND - 63, 63);
    let at_bound_slug = letters_slug(SLUG_BOUND - 64, 63);
    let past_bound_slug = letters_slug(SLUG_BOUND - 63, 63);
    assert_eq!(at_bound_slug.len(), SLUG_BOUND);
    assert_eq!(past_bound_slug.len(), SLUG_BOUND + 1);

    let root = SteamRoot::new("slug-bound");
    root.acf("5001", &at_bound, &at_bound);
    root.acf("5002", &past_bound, &past_bound);
    root.acf("5003", "Example Game", "Example Game");

    let output = run(&["list", "--steam-root", root.path.to_str().expect("utf-8")]);
    let stdout = stdout_of(&output);
    assert!(
        output.status.success(),
        "a root that yields rows inside the bound is a successful run, stderr: {}",
        stderr_of(&output)
    );

    assert!(
        stdout.contains(&at_bound_slug),
        "a slug of exactly {SLUG_BOUND} octets must render:\n{stdout}"
    );
    assert!(
        !stdout.contains(&past_bound_slug),
        "a slug of {} octets must never be rendered:\n{stdout}",
        past_bound_slug.len()
    );
    let rendered = rendered_slugs(&stdout);
    let rendered: Vec<&str> = rendered.iter().map(String::as_str).collect();
    assert_eq!(
        rendered,
        vec!["example-game", at_bound_slug.as_str()],
        "the in-bound rows of the root render, in slug order, and the over-bound row is gone"
    );
}

// proves: design 024's Test strategy #9 (Chosen design: CLI boundary) — the JSON
// rendering honours the same bound: every slug that reaches `--json` is a valid
// product id inside the bound, and the row the plugin refused is absent.
#[test]
fn every_slug_the_json_rendering_prints_is_a_valid_id_inside_the_bound() {
    let past_bound = letters(SLUG_BOUND - 63, 63);
    let past_bound_slug = letters_slug(SLUG_BOUND - 63, 63);
    let root = SteamRoot::new("slug-json-bound");
    root.acf("5101", &past_bound, &past_bound);
    root.acf("5102", "Example Game", "Example Game");
    root.acf("5103", "Hades", "Hades II");

    let output = run(&[
        "list",
        "--steam-root",
        root.path.to_str().expect("utf-8"),
        "--json",
    ]);
    let stdout = stdout_of(&output);
    assert!(
        output.status.success(),
        "one refused row must not fail a run that renders others, stderr: {}",
        stderr_of(&output)
    );

    let games: Vec<serde_json::Value> = serde_json::from_str(stdout.trim())
        .expect("`list --json` must print a JSON array of DiscoveredGame objects");
    assert_eq!(
        games.len(),
        2,
        "only the two in-bound rows are serialized:\n{stdout}"
    );
    for game in &games {
        let obj = game.as_object().expect("each row is a JSON object");
        let slug = obj["slug"].as_str().expect("a slug field");
        assert!(
            is_slug_shaped(slug),
            "`{slug}` reached the JSON rendering and is not a grammar-valid, in-bound id"
        );
        assert_eq!(
            obj["namespace"], "unknown",
            "every serialized row carries its namespace beside its slug"
        );
    }
    assert!(
        !stdout.contains(&past_bound_slug),
        "the refused row leaves no trace in the JSON array:\n{stdout}"
    );
}

// proves: design 024's Test strategy #9 + Chosen design (Emission, refusal and
// collisions) — a root whose only row cannot yield a slug is a loud failure: the
// CLI reports it, writes no table, and its diagnostic counts and names roots
// without echoing the game it could not name, so a refusal can never put a path or
// a foreign string on the operator's terminal.
#[test]
fn a_root_whose_only_row_has_no_slug_source_fails_without_echoing_that_row() {
    let un_derivable = "\u{5e83}\u{304f}\u{30ed}\u{30c8}\u{3068}\u{3072}\u{3068}\u{307f}";
    let root = SteamRoot::new("slug-no-source");
    root.acf("5201", un_derivable, un_derivable);

    let output = run(&["list", "--steam-root", root.path.to_str().expect("utf-8")]);
    let stderr = stderr_of(&output);

    assert!(
        !output.status.success(),
        "a root that yielded nothing but a refusal is a failure, stdout: {}",
        stdout_of(&output)
    );
    assert!(
        stdout_of(&output).is_empty(),
        "a failed run writes no table, got: {}",
        stdout_of(&output)
    );
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "exactly one stderr line reports the failure: {stderr}"
    );
    assert!(
        lines[0].starts_with("error: plugin steam-linux failed: "),
        "the CLI speaks in its own voice about the plugin that refused: {:?}",
        lines[0]
    );
    assert!(
        lines[0].contains("bindforge-cli-slug-no-source"),
        "the diagnostic names the selected root so the run can be traced: {:?}",
        lines[0]
    );
    assert!(
        !lines[0].contains(un_derivable),
        "a refusal names roots and counts, never the row it refused: {:?}",
        lines[0]
    );
    assert!(
        !stderr.contains("panicked at"),
        "the refusal is a clean error path, never a panic: {stderr}"
    );
}

// proves: design 024's Test strategy #8 + Goals (one namespace per row, kept out of
// the slug) — a row built from a scratch root, not the checked-in fixture, renders
// the same five columns with a prefix-free slug: the shape is the renderer's rule,
// not a property of the fixture.
#[test]
fn a_scratch_root_renders_the_same_five_columns_with_a_prefix_free_slug() {
    let root = SteamRoot::new("slug-columns");
    root.acf("5301", "Example Game", "Example Game");

    let output = run(&["list", "--steam-root", root.path.to_str().expect("utf-8")]);
    let stdout = stdout_of(&output);
    assert!(
        output.status.success(),
        "a plain root must succeed, stderr: {}",
        stderr_of(&output)
    );

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "header, separator, one row:\n{stdout}");
    assert_eq!(
        cells(lines[0]),
        vec![
            "Slug".to_string(),
            "Namespace".to_string(),
            "Display Name".to_string(),
            "Install Path".to_string(),
            "Platform".to_string(),
        ],
        "the five columns ADR-024 renders"
    );
    assert_eq!(
        cells(lines[2]),
        {
            let mut row = vec![
                "example-game".to_string(),
                "unknown".to_string(),
                "Example Game".to_string(),
            ];
            row.push(
                std::fs::canonicalize(&root.path)
                    .expect("the scratch root exists while the run is over")
                    .join("steamapps")
                    .join("common")
                    .join("Example Game")
                    .to_string_lossy()
                    .into_owned(),
            );
            // The `Platform` column keeps the shipped plugin-name spelling: ADR-024
            // touches only the slug and adds the namespace column. The JSON row's
            // variant-name spelling, `SteamLinux`, is pinned in `cli_tests.rs`.
            row.push("steam-linux".to_string());
            row
        },
        "the row's slug is prefix-free and its namespace sits beside it"
    );
}
