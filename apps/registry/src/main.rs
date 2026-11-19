//! BindForge registry (design 026): a catalog registry with the two modes `026-C1`
//! names — a *root* instance that answers only from its own curated corpus, and a
//! *caching* instance that answers from local entries, then its own cache, then one
//! configured upstream.
//!
//! This crate is bin-only, and every budget it enforces is its own
//! ([`constants`]). The two documents it owns are the `026-C3` catalog document and
//! the `026-C7` cache document; the `026-L1` amendment that lets it point at any
//! parseable upstream origin lives in [`endpoint`].
//!
//! A refusal at startup is exactly one line and a non-zero exit, and it happens
//! before the process binds: an operator reads the condition, not a usage dump, and
//! a refused instance has touched no file and dialled nothing.

mod admin;
mod cache;
mod constants;
mod corpus;
mod endpoint;
mod http;
mod keys;
mod persist;
mod serve;
mod state;
mod upstream;
#[cfg(test)]
mod wire;

use std::io::Write as _;
use std::net::TcpListener;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::admin::Admin;
use crate::endpoint::parse_endpoint;
use crate::keys::key_is_admissible;
use crate::persist::{DataLayout, data_dir, document_path};
use crate::state::{Answer, State};
use crate::upstream::Upstream;

/// The environment seam `026-C5` names for configuring the admin token.
const ADMIN_TOKEN_ENV: &str = "BINDFORGE_REGISTRY_ADMIN_TOKEN";

/// The wall clock in whole seconds since the Unix epoch: the unit both `026-B3`'s
/// TTL and `026-C7`'s `fetched_at` are stated in.
#[must_use]
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let command = argv.next();
    let args = argv.collect();
    match run(command.as_deref(), args) {
        Ok(()) => ExitCode::SUCCESS,
        // Every message reaching here is one already-formatted line, prefixed by
        // whoever found the condition.
        Err(line) => {
            eprintln!("{line}");
            ExitCode::FAILURE
        }
    }
}

/// Dispatches one invocation.
fn run(command: Option<&str>, args: Vec<String>) -> Result<(), String> {
    let Some(command) = command else {
        return Err("error: `registry` needs a command: `serve` or `query`".to_string());
    };
    let config = Config::parse(&args)?;
    match command {
        "serve" => serve_command(config),
        "query" => query_command(config),
        other => Err(format!(
            "error: `{}` is not a command this registry has",
            bindforge_core::escape_and_truncate(other)
        )),
    }
}

/// Everything the command line decides, after parsing.
#[derive(Debug)]
struct Config {
    /// `--root`: this is a root instance.
    root: bool,
    /// `--corpus <path>`: the `026-C3` document a root instance answers from.
    corpus: Option<PathBuf>,
    /// `--upstream <url>`: the origin a caching instance fetches from.
    upstream: Option<String>,
    /// `--host`, defaulting to the loopback spelling.
    host: String,
    /// `--port`, defaulting to `0` so the kernel assigns one.
    port: u16,
    /// `--admin-token`, or the environment's value of [`ADMIN_TOKEN_ENV`].
    admin_token: Option<String>,
    /// `--refresh-interval <seconds>`, overriding `026-B3`.
    refresh_interval: Option<u64>,
    /// The one slug `query` asks about.
    slug: Option<String>,
}

impl Config {
    /// Parses the flag set. An unknown flag, a flag without its value, or a value
    /// outside its stated form is a one-line refusal before any file or socket is
    /// touched.
    fn parse(args: &[String]) -> Result<Self, String> {
        let refused = |reason: String| Err(format!("error: {reason}"));
        let mut config = Self {
            root: false,
            corpus: None,
            upstream: None,
            host: "127.0.0.1".to_string(),
            port: 0,
            admin_token: std::env::var(ADMIN_TOKEN_ENV)
                .ok()
                .filter(|value| !value.is_empty()),
            refresh_interval: None,
            slug: None,
        };
        let mut index = 0;
        while index < args.len() {
            // A value is taken with its flag, so `--corpus` and `--upstream` never
            // have to guess whether the next token is theirs or another flag's.
            let flag = args[index].as_str();
            // `--flag=value` is the same flag as `--flag value`, and accepting that
            // spelling is a credential rule as much as a convenience: the unknown-flag
            // refusal quotes the flag it did not recognize, and `--admin-token=S3CRET`
            // reaching it would print the configured token into a terminal, a journal,
            // or a CI log. Only a `--` flag is ever split — a `query` slug may hold
            // `=` and is a positional, not a flag.
            let (name, inline) = match flag.split_once('=') {
                Some((name, value)) if name.starts_with("--") => (name, Some(value)),
                _ => (flag, None),
            };
            let mut value = |what: &str| -> Result<String, String> {
                if let Some(inline) = inline {
                    return Ok(inline.to_string());
                }
                let Some(next) = args.get(index + 1) else {
                    return Err(format!("error: {what} needs a value"));
                };
                index += 1;
                Ok(next.clone())
            };
            match name {
                "--root" => {
                    if inline.is_some() {
                        return refused("`--root` names the mode and takes no value".to_string());
                    }
                    config.root = true;
                }
                "--corpus" => config.corpus = Some(PathBuf::from(value("--corpus")?)),
                "--upstream" => config.upstream = Some(value("--upstream")?),
                "--host" => config.host = value("--host")?,
                "--port" => {
                    let spelled = value("--port")?;
                    config.port = spelled.parse::<u16>().map_err(|_| {
                        refused(format!(
                            "`--port {}` is not a port number",
                            bindforge_core::escape_and_truncate(&spelled)
                        ))
                        .unwrap_err()
                    })?;
                }
                "--admin-token" => config.admin_token = Some(value("--admin-token")?),
                "--refresh-interval" => {
                    let spelled = value("--refresh-interval")?;
                    let Ok(seconds) = spelled.parse::<u64>() else {
                        return refused(format!(
                            "`--refresh-interval {}` is not a bare unsigned count of seconds",
                            bindforge_core::escape_and_truncate(&spelled)
                        ));
                    };
                    config.refresh_interval = Some(seconds);
                }
                other if other.starts_with('-') => {
                    // The *name* is quoted, never the value: whatever followed an
                    // unrecognized `--flag=` is not repeated back.
                    return refused(format!(
                        "`{}` is not a flag this registry has",
                        bindforge_core::escape_and_truncate(other)
                    ));
                }
                positional => {
                    if config.slug.is_some() {
                        return refused("only one slug is asked about".to_string());
                    }
                    config.slug = Some(positional.to_string());
                }
            }
            index += 1;
        }
        Ok(config)
    }

    /// Builds the instance state the command runs: the mode, its documents, and its
    /// gated upstream. The endpoint gate runs before anything else, so a refused
    /// origin binds nothing and resolves nothing.
    fn build(&self) -> Result<State, String> {
        let lookup = |key: &str| std::env::var(key).ok();
        self.build_with(&|key| lookup(key))
    }

    /// The same build against an injected environment reader — the seam #13's
    /// data-directory rules are tested through, since a process may not rewrite
    /// its own environment from a test thread.
    fn build_with(&self, lookup: &dyn Fn(&str) -> Option<String>) -> Result<State, String> {
        let refused = |reason: String| Err(format!("error: {reason}"));
        if let Some(spelling) = &self.upstream {
            // Caching mode: `026-C7`'s two documents under `<data>/bindforge/registry/`,
            // each created empty on first write rather than at startup.
            let base =
                parse_endpoint(spelling).map_err(|error| format!("error: {}", error.line()))?;
            let layout = DataLayout::host();
            let data = data_dir(layout, lookup).ok_or_else(|| {
                format!(
                    "error: caching mode needs a data directory: set `XDG_DATA_HOME` to an \
                     absolute path (`{ADMIN_TOKEN_ENV}` is unrelated); no usable `<data>` \
                     was resolved"
                )
            })?;
            let ttl = self
                .refresh_interval
                .unwrap_or(constants::REGISTRY_UPSTREAM_TTL_SECONDS);
            return State::caching(
                document_path(&data, persist::CATALOG_DOCUMENT),
                document_path(&data, persist::CACHE_DOCUMENT),
                Upstream::new(base),
                ttl,
            )
            .map_err(|error| error.line());
        }
        if !self.root {
            return refused(
                "one of `--root` (a root instance) or `--upstream <url>` (a caching instance) \
                 chooses the mode"
                    .to_string(),
            );
        }
        let Some(corpus) = self.corpus.as_ref() else {
            return refused("`--root` needs `--corpus <path>` to answer from".to_string());
        };
        State::root(corpus.clone()).map_err(|error| error.line())
    }
}

/// Binds, prints the bound port, and serves until stopped.
fn serve_command(config: Config) -> Result<(), String> {
    let state = config.build()?;
    let listener = bind(&config.host, config.port)?;
    let bound = listener
        .local_addr()
        .map_err(|error| format!("error: the bound address is not readable: {error}"))?;
    // `--port 0` is the documented default, so the printed value is the contract.
    println!(
        "registry {} listening on http://{bound}",
        state.mode().as_str()
    );
    std::io::stdout()
        .flush()
        .map_err(|error| format!("error: printing the bound port failed: {error}"))?;
    // Enablement is judged from the address actually bound, not from the spelling
    // that was asked for (`026-C5`).
    let admin = Admin::new(bound.ip().is_loopback(), config.admin_token.clone());
    serve::serve(listener, Arc::new(state), Arc::new(admin))
        .map_err(|error| format!("error: the accept loop stopped: {error}"))
}

/// Runs the mode's policy once for one slug, prints the answer with its
/// provenance, and exits non-zero on the `026-L4` cold case so scripts see failure
/// (open question 12). In caching mode this is the real policy, so it writes the
/// cache.
fn query_command(config: Config) -> Result<(), String> {
    let state = config.build()?;
    let Some(slug) = config.slug.as_ref() else {
        return Err("error: `query` needs one slug to ask about".to_string());
    };
    if !key_is_admissible(slug) {
        return Err(format!(
            "error: `{}` is not a lookup key this registry answers",
            bindforge_core::escape_and_truncate(slug)
        ));
    }
    let answer = state.candidates(slug, now_unix());
    match answer {
        Answer::Candidates {
            candidates,
            provenance,
        } => {
            println!(
                "queried `{}` answered from {}",
                bindforge_core::escape_and_truncate(slug),
                provenance.as_str()
            );
            if candidates.is_empty() {
                println!("  (no candidates)");
            }
            for candidate in &candidates {
                println!("  {}\t{}", candidate.game_id, candidate.name);
            }
            std::io::stdout()
                .flush()
                .map_err(|error| format!("error: printing the answer failed: {error}"))?;
            Ok(())
        }
        Answer::NoAnswer => Err(format!(
            "error: no catalog answer for `{}`: no local entry, no cache entry, and no \
             reachable upstream",
            bindforge_core::escape_and_truncate(slug)
        )),
    }
}

/// Binds the requested host and port, trying every address the host name yields.
fn bind(host: &str, port: u16) -> Result<TcpListener, String> {
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| {
            format!(
                "error: `--host {}` cannot be resolved: {error}",
                bindforge_core::escape_and_truncate(host)
            )
        })?
        .collect::<Vec<_>>();
    let mut last: Option<std::io::Error> = None;
    for address in addresses {
        match TcpListener::bind(address) {
            Ok(listener) => return Ok(listener),
            Err(error) => last = Some(error),
        }
    }
    Err(format!(
        "error: nothing could be bound on `{}` port {port}: {}",
        bindforge_core::escape_and_truncate(host),
        last.map(|error| error.to_string())
            .unwrap_or_else(|| "no address".to_string())
    ))
}

#[cfg(test)]
mod tests {
    //! The command line: which flag sets decide the mode, and what a bad one costs.
    use super::Config;

    fn args(spelled: &[&str]) -> Vec<String> {
        spelled.iter().map(|arg| (*arg).to_string()).collect()
    }

    #[test]
    fn the_flags_decide_the_mode_and_the_budgets() {
        let config = Config::parse(&args(&[
            "--root",
            "--corpus",
            "/tmp/corpus.json",
            "--port",
            "8080",
            "--refresh-interval",
            "30",
        ]))
        .expect("a well-formed root invocation");
        assert!(config.root);
        assert_eq!(
            config.corpus.as_deref(),
            Some(std::path::Path::new("/tmp/corpus.json"))
        );
        assert_eq!(config.port, 8080);
        assert_eq!(config.refresh_interval, Some(30));
        assert_eq!(config.host, "127.0.0.1", "the default host is loopback");
    }

    #[test]
    fn the_port_defaults_to_zero_so_the_kernel_assigns_one() {
        let config = Config::parse(&args(&["--root", "--corpus", "/tmp/corpus.json"]))
            .expect("a well-formed root invocation");
        assert_eq!(config.port, 0, "`--port` defaults to `0`");
        assert!(
            config.admin_token.is_none(),
            "no token is configured by default"
        );
    }

    #[test]
    fn a_flag_without_its_value_or_an_unknown_flag_is_refused() {
        for spelled in [
            vec!["--root", "--corpus"],
            vec!["--nope"],
            vec!["--port", "not-a-port"],
            vec!["--refresh-interval", "-5"],
        ] {
            let error = Config::parse(&args(&spelled)).expect_err("a refusal is expected");
            assert!(error.starts_with("error: "), "{spelled:?} is a named line");
        }
    }

    #[test]
    fn an_equals_form_takes_its_value_and_a_refusal_never_quotes_one() {
        let config = Config::parse(&args(&[
            "--root",
            "--corpus=/tmp/corpus.json",
            "--admin-token=s3cret-token",
        ]))
        .expect("`--flag=value` is the same flag as `--flag value`");
        assert_eq!(
            config.corpus.as_deref(),
            Some(std::path::Path::new("/tmp/corpus.json"))
        );
        assert_eq!(config.admin_token.as_deref(), Some("s3cret-token"));

        let error = Config::parse(&args(&["--admin-token=s3cret-token", "--nope=also-secret"]))
            .expect_err("an unknown flag is still refused");
        assert!(error.contains("--nope"), "{error}");
        assert!(
            !error.contains("s3cret") && !error.contains("also-secret"),
            "the refusal quotes the flag name it did not recognize, never a value: {error}"
        );

        let error = Config::parse(&args(&["--root=oops"])).expect_err("the mode flag takes none");
        assert!(error.contains("--root"), "{error}");
    }

    #[test]
    fn a_slug_holding_an_equals_sign_is_a_positional_and_not_a_flag() {
        let config = Config::parse(&args(&[
            "--root",
            "--corpus",
            "/tmp/corpus.json",
            "local/a=b",
        ]))
        .expect("a slug may hold `=`");
        assert_eq!(config.slug.as_deref(), Some("local/a=b"));
    }

    #[test]
    fn a_caching_build_without_a_data_directory_is_refused() {
        // No `XDG_DATA_HOME`, no `HOME`: caching mode has nowhere to put its documents.
        // The environment is injected here rather than rewritten in this process, whose
        // other test threads read it too.
        let config = Config::parse(&args(&["--upstream", "http://127.0.0.1:8080"]))
            .expect("the flags parse");
        let nothing = |_key: &str| None;
        let error = config
            .build_with(&nothing)
            .expect_err("a data directory is required");
        assert!(error.contains("XDG_DATA_HOME"), "{error}");
    }

    #[test]
    fn an_upstream_outside_the_gate_is_named_and_binds_nothing() {
        let config = Config::parse(&args(&["--upstream", "https://127.0.0.1:8080"]))
            .expect("the flags parse");
        let error = config.build().expect_err("the gate refuses");
        assert!(error.to_lowercase().contains("upstream"), "{error}");
    }

    #[test]
    fn a_mode_is_always_chosen() {
        let config = Config::parse(&args(&[])).expect("no flags is a parseable command line");
        let error = config.build().expect_err("neither mode was chosen");
        assert!(
            error.contains("--root") && error.contains("--upstream"),
            "{error}"
        );
    }
}
