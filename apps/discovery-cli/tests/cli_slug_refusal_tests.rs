//! Design 025's CLI-side refusal and rendering pins for `discovery list`.
//!
//! The ADR-024 shape of this suite is half-retired: its two bound-boundary
//! fixture routes are deleted, per the Rollout 3 migration ("the unproducible
//! fixture route of `cli_slug_refusal_tests` is deleted, its bound refusal
//! pinned in-module in `main.rs` instead") — under ADR-025 a plugin row's slug
//! is `steam/<appid>` and the appid read caps it far inside the discovery
//! bound, so no fixture ACF can hand the CLI an over-bound row; the bound-only
//! `refuse_invalid_slugs` is pinned at its definition in-module.
//!
//! What remains is the observable half the repository can still produce: a root
//! whose every manifest is refused fails loudly in the CLI's own voice, naming
//! roots and counts without echoing the row it could not name (the plugin-side
//! refusals stay count-only, and `escape_and_truncate` is pinned where it lives,
//! in `crates/core/tests/error.rs`); and the grammar reversal shows at the
//! render seam — a `steam/7` lookup key, grammar-invalid by design and therefore
//! unrenderable under the retired check, now rides the bound-only boundary into
//! the five-column table, from a scratch root so the shape is the renderer's
//! rule, not a property of the fixture.
//!
//! All runs sit behind the loopback catalog fake and a temp
//! `XDG_DATA_HOME`/`HOME` sandbox (Rollout 3); the tests build their own Steam
//! roots under the system temp directory and remove them again (repo practice:
//! integration tests use temp directories and clean up after themselves); no
//! test probes the host or needs Steam installed.

mod common;

use common::{FakeCatalog, SteamRoot, TempDir, cells, ok, run_cli};

/// A fake that serves no candidates to any query.
fn fake_empty() -> FakeCatalog {
    FakeCatalog::start("refusal-fake-empty", |_| ok("[]"))
}

/// The sandbox pair that keeps every store byte inside the test.
fn sandbox(label: &str) -> (TempDir, TempDir) {
    (
        TempDir::new(&format!("refusal-{label}-data")),
        TempDir::new(&format!("refusal-{label}-home")),
    )
}

fn env<'a>(data: &'a TempDir, home: &'a TempDir) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ]
}

/// proves: design 025 keeps the loud all-refused-root failure (Chosen design:
/// plugin-side refusals are count-only and the engine reports an empty scan) — a
/// root whose only manifest carries an appid the plugin refuses (`900/1`, an
/// embedded `/`, skipped per the row-identity rules) is a failure, not an empty
/// table: the CLI reports it in its own voice, writes no table, names the root
/// without echoing the game it could not name, and never panics.
#[test]
fn a_root_whose_only_row_has_no_valid_appid_fails_without_echoing_that_row() {
    let un_derivable = "\u{5e83}\u{304f}\u{30ed}\u{30c8}\u{3068}\u{3072}\u{3068}\u{307f}";
    let root = SteamRoot::new("slug-no-source");
    // The appid is what a row's identity is built from now; an embedded `/` is
    // refused at the plugin seam and counted into `skipped`.
    let mut acf: Vec<u8> = Vec::new();
    acf.extend_from_slice(b"\"AppState\"\n{\n\t\"appid\"\t\t\"900/1\"\n");
    acf.extend_from_slice(b"\t\"name\"\t\t\"");
    acf.extend_from_slice(un_derivable.as_bytes());
    acf.extend_from_slice(b"\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Some Dir\"\n");
    acf.extend_from_slice(b"\t\"SizeOnDisk\"\t\t\"1000000\"\n}\n");
    root.acf_raw("900-slash-1", &acf);
    let (data, home) = sandbox("no-source");
    let fake = fake_empty();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );

    assert!(
        !run.success(),
        "a root that yielded nothing but a refusal is a failure, stdout: {}",
        run.stdout
    );
    assert!(
        run.stdout.is_empty(),
        "a failed run writes no table, got: {}",
        run.stdout
    );
    let lines = run.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "exactly one stderr line reports the failure: {}",
        run.stderr
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
        !run.stderr.contains("panicked at"),
        "the refusal is a clean error path, never a panic: {}",
        run.stderr
    );
}

/// proves: design 025's Test strategy #4 at the render seam (Chosen design: the
/// emit boundary "stays `refuse_invalid_slugs`, its body becoming the `025-B1`
/// bound alone") — a row whose lookup key is `steam/7`, grammar-invalid and
/// therefore impossible under the retired shared-grammar check, renders here
/// from a scratch root in the same five columns the fixture tree gets: the shape
/// is the renderer's rule beside the bound-only boundary, not a property of the
/// fixture.
#[test]
fn a_scratch_root_renders_the_five_columns_with_a_grammar_exempt_lookup_key() {
    let root = SteamRoot::new("slug-columns");
    root.acf("7", Some("Lucky Number"), "Lucky Number");
    let (data, home) = sandbox("columns");
    let fake = fake_empty();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );
    assert!(
        run.success(),
        "a plain root must succeed, stderr: {}",
        run.stderr
    );

    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "header, separator, one row:\n{}",
        run.stdout
    );
    assert_eq!(
        cells(lines[0]),
        vec![
            "Slug".to_string(),
            "Game Id".to_string(),
            "Display Name".to_string(),
            "Install Path".to_string(),
            "Platform".to_string(),
        ],
        "the five columns ADR-025 renders"
    );
    assert_eq!(
        {
            let mut row = vec![
                "steam/7".to_string(),
                "-".to_string(),
                "Lucky Number".to_string(),
            ];
            row.push(
                std::fs::canonicalize(root.path())
                    .expect("the scratch root exists while the run is over")
                    .join("steamapps")
                    .join("common")
                    .join("Lucky Number")
                    .to_string_lossy()
                    .into_owned(),
            );
            // The `Platform` column keeps the shipped plugin-name spelling; the
            // JSON row's variant-name spelling, `SteamLinux`, is pinned in
            // `cli_tests.rs`.
            row.push("steam-linux".to_string());
            row
        },
        cells(lines[2]),
        "the lookup key `steam/7` survives the bound-only boundary and renders — \
         the reversal `025-B1` states, from a grammar the slug is exempt from"
    );
}
