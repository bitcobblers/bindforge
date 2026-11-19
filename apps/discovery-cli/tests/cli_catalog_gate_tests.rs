//! The connectivity gate, the endpoint policy, and the response-parse refusals of
//! design 025 / ADR-025, driven end-to-end against the built CLI with a loopback
//! `TcpListener` fake living in the test process.
//!
//! proves: design 025's Test strategy #11 (the gate refuses before enumerating —
//! no listener behind `--catalog-url` gives exit non-zero, one named stderr line,
//! and stdout carries neither the table header nor any fixture row, which is what
//! proves "before enumerating"), #9 (`candidates_url` and endpoint policy — the
//! accepted loopback spelling and the named `TransportUnavailable` refusals of the
//! malformed authorities; the same refusal with the flag and env both unset), and
//! #10 (the response parse — every hostile response raises `Unreachable` as the
//! named refusal, and a hostile `game_id` in a `200` body is dropped with one
//! echoed, escaped line while the survivors keep order).
//!
//! And design 026's Test strategy #15 (the `026-L1` amendment to this same gate):
//! `endpoint_policy_refuses_everything_beyond_a_configured_http_endpoint` keeps only
//! the spellings the amended gate still refuses — `https`/`ftp`/`@`/empty/
//! non-numeric port — with the refusal line re-pinned off the loopback wording (A12).
//! Remote-host admission itself is pinned at `parse_endpoint` in the unit suite
//! (design 026 §CLI amendment), never here.
//!
//! Integration (CLI, subprocess) per repo practice: every run gets a temp
//! `XDG_DATA_HOME` and `HOME` (the store sandbox is owned by
//! `cli_match_store_tests.rs`) and stdin piped; no test performs a non-loopback
//! request (`025-L1` and Test strategy #15).

mod common;

use common::{
    SteamRoot, TempDir, candidates_body, cells, closed_loopback_port, ok, respond_to, run_cli,
    slug_needle, store_path,
};
use std::sync::{Arc, Mutex};

/// The scratch one-game root shared by the gate runs: the identity seam itself is
/// pinned in `crates/discovery`; here the CLI is what refuses or renders.
fn one_game_root(label: &str) -> SteamRoot {
    let root = SteamRoot::new(label);
    root.acf("5001", Some("Example Game"), "Example Game");
    root
}

fn sandbox(label: &str) -> TempDir {
    TempDir::new(label)
}

fn run(
    root: &SteamRoot,
    extra_args: &[&str],
    data: &TempDir,
    url: &str,
    stdin: &str,
) -> common::Run {
    let xdg = data.path().to_str().expect("utf-8 temp path").to_string();
    let mut args = vec!["list", "--steam-root", root.path().to_str().expect("utf-8")];
    args.extend_from_slice(extra_args);
    if !url.is_empty() {
        args.push("--catalog-url");
        args.push(url);
    }
    run_cli(
        &args,
        &[("XDG_DATA_HOME", xdg.as_str()), ("HOME", xdg.as_str())],
        stdin,
    )
}

/// proves: design 025's Test strategy #11 (Chosen design: Scan flow step 1 —
/// "`probe()` failure is the named refusal and `ExitCode::FAILURE`, before any plugin
/// is constructed (`025-L1`)") — with no listener behind `--catalog-url`, the run
/// exits non-zero with exactly one `error: catalog unreachable: …` line, and stdout
/// carries neither the table header nor any fixture row.
#[test]
fn the_gate_refuses_before_enumerating() {
    let root = one_game_root("gate-closed");
    let data = sandbox("gate-closed");
    let url = format!("http://127.0.0.1:{}", closed_loopback_port());

    let run = run(&root, &[], &data, &url, "");
    assert!(!run.success(), "an unreachable catalog is a failed run");
    assert_eq!(
        run.code(),
        Some(1),
        "the refusal exits with `ExitCode::FAILURE`"
    );
    let lines = run.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "exactly one named stderr line reports the gate: {:?}",
        run.stderr
    );
    assert!(
        lines[0].starts_with("error: catalog unreachable:"),
        "the gate speaks the named refusal `error: catalog unreachable: <reason>`, \
         got: {:?}",
        lines[0]
    );
    assert!(
        run.stdout.is_empty() && !run.stdout.contains("Slug") && !run.stdout.contains("steam/5001"),
        "stdout carries neither the table header nor any fixture row — nothing was \
         enumerated before the gate: {:?}",
        run.stdout
    );
    assert!(
        !run.stderr.contains("panicked at"),
        "the refusal is a clean error path: {}",
        run.stderr
    );
}

/// proves: design 026's Test strategy #15 — "the subprocess suite re-pins only the
/// refused spellings (`https`/`ftp`/`@`/empty/non-numeric port) and the new refusal
/// line" — with §CLI amendment's rule behind it: `parse_endpoint` "keeps the
/// literal-`http://` requirement and the digits-only port but drops the
/// `LOOPBACK_HOSTS` host check and the constant with it".
///
/// So this list is exactly what survives the amendment, and nothing else: the two
/// spellings that only the host allowlist refused (`http://catalog.example`, and the
/// non-dotted `http://0177.0.0.1/`) are **gone**, because after `026-L1` they are
/// admitted endpoints whose admission design 026 pins at `parse_endpoint` in the unit
/// suite and nowhere near a socket (A11, A15: "no test dials or resolves a
/// non-loopback host"). The line each refusal prints is re-pinned too (A12): the
/// `loopback` wording is dropped and the admitted spelling — a configured `http://`
/// endpoint, any host — takes its place, naming `--catalog-url` and
/// `BINDFORGE_CATALOG_URL`.
#[test]
fn endpoint_policy_refuses_everything_beyond_a_configured_http_endpoint() {
    let root = one_game_root("gate-policy");
    let data = sandbox("gate-policy");
    let bad_endpoints = [
        ("https-scheme", "https://127.0.0.1:8080/"),
        (
            "userinfo-in-the-authority",
            "http://127.0.0.1:8080@host.example/",
        ),
        ("empty-host", "http://:8080/"),
        ("non-numeric-port", "http://127.0.0.1:99x/"),
        ("ftp-scheme", "ftp://127.0.0.1/"),
        ("no-scheme-at-all", "127.0.0.1:8080"),
    ];
    for (label, endpoint) in bad_endpoints {
        let run = run(&root, &[], &data, endpoint, "");
        assert!(
            !run.success(),
            "`{label}` (`{endpoint}`) is outside the endpoint policy and must fail the run, \
             stderr: {}",
            run.stderr
        );
        let lines = run.stderr_lines();
        assert_eq!(
            lines.len(),
            1,
            "the endpoint refusal is exactly one named line for `{label}`: {:?}",
            run.stderr
        );
        assert!(
            lines[0].starts_with("error: catalog transport unavailable:"),
            "the endpoint policy speaks `error: catalog transport unavailable: <reason>` \
             for `{label}`, got: {:?}",
            lines[0]
        );
        assert!(
            lines[0].contains("--catalog-url") && lines[0].contains("BINDFORGE_CATALOG_URL"),
            "the refusal names both config sources for `{label}`: {:?}",
            lines[0]
        );
        assert!(
            lines[0].contains("http://"),
            "the refusal now points at the spelling it actually admits — a configured \
             `http://` endpoint, any host — for `{label}`: {:?}",
            lines[0]
        );
        assert!(
            !lines[0].contains("loopback"),
            "A12: the `loopback` wording is gone from the refusal line, and the host \
             allowlist with it (`026-L1`), but `{label}` still said: {:?}",
            lines[0]
        );
        assert!(
            run.stdout.is_empty(),
            "`{label}` must fail before enumeration: {:?}",
            run.stdout
        );
    }

    // The fourth refused spelling needs the flag present with no value at all, which
    // the `run` helper cannot express (it omits an empty URL rather than passing it).
    let xdg = data.path().to_str().expect("utf-8 temp path").to_string();
    let empty_flag = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8"),
            "--catalog-url",
            "",
        ],
        &[("XDG_DATA_HOME", xdg.as_str()), ("HOME", xdg.as_str())],
        "",
    );
    assert!(
        !empty_flag.success(),
        "an empty `--catalog-url` is a refused endpoint, not an unconfigured one: {}",
        empty_flag.stderr
    );
    let lines = empty_flag.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "the empty endpoint is one named line: {:?}",
        empty_flag.stderr
    );
    assert!(
        lines[0].starts_with("error: catalog transport unavailable:")
            && lines[0].contains("http://")
            && !lines[0].contains("loopback"),
        "the empty endpoint speaks the re-pinned refusal line, got: {:?}",
        lines[0]
    );
    assert!(
        empty_flag.stdout.is_empty(),
        "and nothing is enumerated: {:?}",
        empty_flag.stdout
    );
}

/// proves: design 025's Test strategy #9 (Chosen design: "`HttpCatalog` is built
/// from `--catalog-url` … or `BINDFORGE_CATALOG_URL`") — the env var is an accepted
/// spelling of the same untrusted config source: with only the env var set to a
/// loopback endpoint the scan runs; with neither flag nor env var, the same named
/// refusal as any rejected endpoint.
#[test]
fn the_catalog_endpoint_comes_from_the_flag_or_the_env_var_and_from_nothing_else() {
    let root = one_game_root("gate-env");
    let data = sandbox("gate-env");

    // No fake listener answers, so the env-var run uses a listener that does:
    let fake = common::FakeCatalog::start("env-route", |_| common::ok("[]"));
    let xdg = data.path().to_str().expect("utf-8").to_string();
    let run_env = common::run_cli(
        &["list", "--steam-root", root.path().to_str().expect("utf-8")],
        &[
            ("XDG_DATA_HOME", xdg.as_str()),
            ("HOME", xdg.as_str()),
            ("BINDFORGE_CATALOG_URL", fake.url().as_str()),
        ],
        "",
    );
    assert!(
        run_env.success(),
        "`BINDFORGE_CATALOG_URL` is an accepted endpoint source, stderr: {}",
        run_env.stderr
    );
    assert!(
        run_env.stdout.contains("steam/5001"),
        "the env-var route enumerates and renders: {:?}",
        run_env.stdout
    );

    // Neither flag nor env var: the same refusal name.
    let run_unset = run(&root, &[], &data, "", "");
    assert!(!run_unset.success(), "no endpoint is a refused run");
    let lines = run_unset.stderr_lines();
    assert_eq!(lines.len(), 1, "one named line: {:?}", run_unset.stderr);
    assert!(
        lines[0].starts_with("error: catalog transport unavailable:")
            && lines[0].contains("--catalog-url")
            && lines[0].contains("BINDFORGE_CATALOG_URL")
            && lines[0].contains("http://")
            && !lines[0].contains("loopback"),
        "an unset flag and env raise the same name, and after `026-L1` it points at a \
         configured `http://` endpoint of any host rather than at a loopback spelling: \
         {:?}",
        lines[0]
    );
}

/// proves: design 025's Test strategy #9 (Chosen design: "`fn candidates_url(base,
/// slug)` is pure: `<base>/v1/candidates?slug=<encoded>` … `steam/900001` travels as
/// `steam%2F900001`") — as driven end-to-end over the real socket, per Test strategy
/// #12: the subprocess's request line is exactly the `GET` of the encoded singular
/// query, once per distinct unresolved slug.
#[test]
fn one_get_per_unresolved_slug_travels_percent_encoded() {
    let root_extra = one_game_root("gate-url");
    root_extra.acf("5002", Some("Second Game"), "Second Game");
    let data = sandbox("gate-url");

    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = seen.clone();
    let fake = common::FakeCatalog::start("url-pin", move |request| {
        sink.lock().expect("sink").push(request.to_string());
        common::ok("[]")
    });

    let run = run(&root_extra, &[], &data, &fake.url(), "");
    assert!(
        run.success(),
        "scan with two unresolved slugs runs: {}",
        run.stderr
    );

    let mut requests = fake.requests();
    requests.sort();
    assert_eq!(
        requests,
        vec![
            "GET /v1/candidates?slug=steam%2F5001 HTTP/1.1",
            "GET /v1/candidates?slug=steam%2F5002 HTTP/1.1",
        ],
        "one GET per distinct unresolved slug, each at `/v1/candidates` with the slug \
         percent-encoded (`steam%2F…`, never a literal `/`), and nothing else"
    );
    let full = seen.lock().expect("sink").clone().join("\n---\n");
    assert!(
        full.contains("Host:")
            && full.contains("Accept: application/json")
            && full.contains("Connection: close"),
        "the std-only request carries `Host`, `Accept: application/json`, and \
         `Connection: close`: {full}"
    );
}

/// proves: design 025's Test strategy #10 (Chosen design: "the response parse
/// refuses anything it has not been told to understand") — a status other than
/// `200`, a `Transfer-Encoding` header (in either casing), a missing, duplicated,
/// or non-numeric `Content-Length`, a header block over the `65_536`-octet budget,
/// a body over it, a 257-candidate body, and a non-JSON body each raise the named
/// `error: catalog unreachable: …` refusal, exit non-zero, and print no partial
/// table (the run fails closed).
#[test]
fn every_ununderstood_response_raises_the_named_unreachable_refusal() {
    let header_pad = format!(
        "HTTP/1.1 200 OK\r\nX-Pad: {}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]",
        "p".repeat(65_536)
    );
    let wide_body = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        65_537,
        "x".repeat(65_537)
    );
    let candidates_257: Vec<String> = (1..=257)
        .map(|index| format!(r#"{{"game_id":"cand-{index}","name":"Candidate {index}"}}"#))
        .collect();
    let too_many = ok(&format!("[{}]", candidates_257.join(",")));

    let hostile: [(&str, Vec<u8>); 11] = [
        (
            "404",
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "500",
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "transfer-encoding",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"
                .to_vec(),
        ),
        (
            "lowercase transfer-encoding",
            b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"
                .to_vec(),
        ),
        (
            "missing content-length",
            b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
        (
            "duplicated content-length",
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "non-numeric content-length",
            b"HTTP/1.1 200 OK\r\nContent-Length: ten\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
        (
            "over-long header block",
            header_pad.into_bytes(),
        ),
        ("over-long body", wide_body.into_bytes()),
        ("257 candidates", too_many),
        ("non-JSON body", ok("this is not JSON")),
    ];

    for (label, response) in hostile {
        let root = one_game_root(&format!("gate-parse-{label}"));
        let data = sandbox(&format!("gate-parse-{label}"));
        let fake = common::FakeCatalog::start("hostile", {
            let response = response.clone();
            move |_request| response.clone()
        });
        let run = run(&root, &[], &data, &fake.url(), "");
        assert!(
            !run.success(),
            "the `{label}` response must fail the run, stdout: {}",
            run.stdout
        );
        let lines = run.stderr_lines();
        assert_eq!(
            lines.len(),
            1,
            "the `{label}` refusal is exactly one named line: {:?}",
            run.stderr
        );
        assert!(
            lines[0].starts_with("error: catalog unreachable:"),
            "the `{label}` response raises `error: catalog unreachable: <reason>` \
             (the named refusal of a transport/parse failure), got: {:?}",
            lines[0]
        );
        assert!(
            run.stdout.is_empty(),
            "a mid-scan query failure fails closed with no partial table (`{label}`): \
             {:?}",
            run.stdout
        );
    }
}

/// proves: design 025's Test strategy #10, last clause (Chosen design: "a candidate
/// whose `game_id` fails `is_slug_name` + `MAX_SLUG_OCTETS` is dropped with one
/// named stderr refusal line; the surviving candidates keep the response's order") —
/// a `game_id` of `../x` in a `200` body never reaches the prompt's list or the
/// store, is echoed exactly once (escaped, one line, no raw path forging a second
/// line), and the surviving candidate is confirmable as number one.
#[test]
fn a_hostile_candidate_game_id_is_dropped_with_one_named_echo() {
    let root = one_game_root("gate-poison");
    let data = sandbox("gate-poison");
    let fake = common::FakeCatalog::start(
        "poisoned",
        respond_to(
            vec![(
                slug_needle("5001"),
                ok(&candidates_body(&[
                    ("../x", "Evil Twin"),
                    ("good-game", "Good"),
                ])),
            )],
            ok("[]"),
        ),
    );

    let run = run(&root, &[], &data, &fake.url(), "1\n");
    assert!(
        run.success(),
        "one dropped candidate does not fail the run, stderr: {}",
        run.stderr
    );

    let stderr_lines = run.stderr_lines();
    let poison_lines: Vec<&&str> = stderr_lines
        .iter()
        .filter(|line| line.contains("../x") || line.contains("..%2Fx"))
        .collect();
    assert_eq!(
        poison_lines.len(),
        1,
        "the hostile `game_id` is dropped with exactly one named stderr refusal line \
         (the escaped echo — never a line that forges a path of its own): {:?}",
        run.stderr
    );

    // The survivor keeps the response's order and is the first prompt choice.
    assert!(
        run.stderr.contains("good-game"),
        "the surviving candidate is offered: {:?}",
        run.stderr
    );

    let store = std::fs::read_to_string(store_path(data.path())).expect("the store lands");
    let document: serde_json::Value = store.parse().expect("the store is JSON");
    let records = document["matches"].as_array().expect("a matches array");
    assert_eq!(
        records
            .iter()
            .map(|record| record["game_id"].as_str().unwrap_or_default())
            .collect::<Vec<&str>>(),
        vec!["good-game"],
        "the confirmed record carries the surviving `game_id`, never the traversal \
         spelling (`025-L5`: only a grammar-checked id is ever kept)"
    );
    assert!(
        !run.stdout.contains("../x"),
        "stdout — the table — never carries the dropped value: {:?}",
        run.stdout
    );
}

/// proves: design 025's Test strategy #11, table half — with the gate satisfied and
/// zero candidates everywhere, the plain table renders the new `Game Id` column
/// between `Slug` and `Display Name`, unmatched rows show `-`, and the retired
/// namespace column is gone (the re-pinned output contract, Rollout 3).
#[test]
fn the_satisfied_gate_renders_the_game_id_column() {
    let root = one_game_root("gate-renders");
    let data = sandbox("gate-renders");
    let fake = common::FakeCatalog::start("empty-candidates", |_| ok("[]"));
    let run = run(&root, &[], &data, &fake.url(), "");
    assert!(run.success(), "stderr: {}", run.stderr);

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
        "the five columns ADR-025 renders: the retired namespace column is gone, \
         `Game Id` joins"
    );
    let row = cells(lines[2]);
    assert_eq!(row[0], "steam/5001", "the slug cell carries the lookup key");
    assert_eq!(row[1], "-", "an unmatched row prints `-` in `Game Id`");
    assert_eq!(row[2], "Example Game");
    assert_eq!(row[4], "steam-linux");
}
