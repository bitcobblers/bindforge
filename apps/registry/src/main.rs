//! BindForge registry (design 026, grown by ADR-027): the hosted catalog registry.
//!
//! One mode. A hosted instance answers from three documents it owns — the curated
//! `026-C3` corpus the operator pointed `--corpus` at, the issued document it mints
//! into, and the retired-id set that keeps a deleted id deleted — plus the steam
//! snapshot it pulls from the store on a schedule and consults, never the other way
//! round, on a miss (`027-C1`'s one-way content supply). Caching mode, its upstream
//! origin, its gauge, and its two documents retired with ADR-027; their flags stay in
//! the parser only to be refused, so a stale operator invocation explains itself.
//!
//! The crate has two targets. This is the binary; [`bindforge_registry`] is the
//! library face the suites drive directly (the resolver, the documents, the policy —
//! `src/lib.rs` says why each of them is on the face). Either way, every budget the
//! instance enforces is its own ([`constants`]).
//!
//! A refusal at startup is exactly one line and a non-zero exit, and it happens
//! before the process binds: an operator reads the condition, not a usage dump, and a
//! refused instance writes no document of the catalog's and dials nothing beyond the
//! pinned store. The two files it may have touched are the `<data>` write lock
//! §Concurrency takes first — `Drop` takes the refusal's run out of it on the way out
//! — and the snapshot a *successful* `027-C3` startup pull published before a later
//! refusal stopped the run, which is a valid fresh snapshot, not a half-written one.

mod admin;
mod corpus;
mod http;
mod keys;
mod mint;
mod persist;
mod retired;
mod serve;
mod state;
mod store_lock;
#[cfg(test)]
mod wire;

// The store resolver is the library face's (`src/lib.rs`): the snapshot loader, the
// pull codec, and the refresh cadence are one set of items, compiled once, that
// `tests/steam.rs` pins directly. The bin uses them rather than keeping a second
// copy of its own — [`constants`] among them, because the `026-C7` plausibility pair
// §The steam snapshot's load admits names for the snapshot's `fetched_at` is read by
// the library's loader and by nothing in this target, so a second copy of that file
// compiled into the binary would carry two budgets no binary code reads.
use bindforge_registry::constants;
use bindforge_registry::steam::{self, SteamStore};

use std::io::Write as _;
use std::net::TcpListener;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::admin::Admin;
use crate::corpus::Catalog;
use crate::keys::key_is_admissible;
use crate::persist::{DataLayout, data_dir, document_path, registry_dir};
use crate::retired::RetiredIds;
use crate::state::{Answer, State, SteamPosture};
use crate::store_lock::WriteLock;

/// The environment seam `026-C5` names for configuring the admin token.
const ADMIN_TOKEN_ENV: &str = "BINDFORGE_REGISTRY_ADMIN_TOKEN";

/// `027-C3`: the steam Web API key arrives through the environment and nowhere else
/// — not a flag, not a file, not a diagnostic.
const STEAM_KEY_ENV: &str = "BINDFORGE_STEAM_WEB_API_KEY";

/// The wall clock in whole seconds since the Unix epoch: the unit both `027-B2`'s
/// age bound and every document's `fetched_at` are stated in.
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
    /// `--root`: the mode ADR-027 leaves, spelled as `026-C1` spelled it.
    root: bool,
    /// `--corpus <path>`: the `026-C3` document this instance answers from.
    corpus: Option<PathBuf>,
    /// `--no-steam`: serve with the steam resolver inactive.
    no_steam: bool,
    /// `--host`, defaulting to the loopback spelling.
    host: String,
    /// `--port`, defaulting to `0` so the kernel assigns one.
    port: u16,
    /// `--admin-token`, or the environment's value of [`ADMIN_TOKEN_ENV`].
    admin_token: Option<String>,
    /// The one key `query` asks about.
    slug: Option<String>,
}

impl Config {
    /// Parses the flag set. An unknown flag, a flag without its value, or a value
    /// outside its stated form is a one-line refusal before any file or socket is
    /// touched — and so is either retired caching flag, which is refused as a
    /// *retirement* rather than as an unknown name (§Hosted mode).
    fn parse(args: &[String]) -> Result<Self, String> {
        let refused = |reason: String| Err(format!("error: {reason}"));
        let mut config = Self {
            root: false,
            corpus: None,
            no_steam: false,
            host: "127.0.0.1".to_string(),
            port: 0,
            admin_token: std::env::var(ADMIN_TOKEN_ENV)
                .ok()
                .filter(|value| !value.is_empty()),
            slug: None,
        };
        let mut index = 0;
        while index < args.len() {
            // A value is taken with its flag, so `--corpus` never has to guess
            // whether the next token is its own or another flag's.
            let flag = args[index].as_str();
            // `--flag=value` is the same flag as `--flag value`, and accepting that
            // spelling is a credential rule as much as a convenience: the unknown-flag
            // refusal quotes the flag it did not recognize, and `--admin-token=S3CRET`
            // reaching it would print the configured token into a terminal, a journal,
            // or a CI log. Only a `--` flag is ever split — a `query` key may hold
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
                "--no-steam" => {
                    if inline.is_some() {
                        return refused(
                            "`--no-steam` names the resolver's posture and takes no value"
                                .to_string(),
                        );
                    }
                    config.no_steam = true;
                }
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
                // ADR-027's retirement pair. They are refused here, in the parser,
                // quoting the flag and naming the retirement, rather than reaching the
                // unknown-flag arm: an operator whose deployment still spells them has
                // to learn what happened from this line, and a `--upstream` value must
                // never be repeated back the way an unknown flag's never is.
                "--upstream" | "--refresh-interval" => {
                    let _ = value(name)?;
                    return refused(retirement_refusal(name));
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
                        return refused("only one lookup key is asked about".to_string());
                    }
                    config.slug = Some(positional.to_string());
                }
            }
            index += 1;
        }
        Ok(config)
    }

    /// Builds the instance the command runs: `<data>` and its write lock, the three
    /// documents, and the snapshot. Shared by `serve` and `query` because a one-shot
    /// runs the same policy and may mint.
    fn build(&self) -> Result<Instance, String> {
        let lookup = |key: &str| std::env::var(key).ok();
        self.build_with(&|key| lookup(key))
    }

    /// The same build against an injected environment reader — the seam the
    /// data-directory rules are tested through, since a process may not rewrite its
    /// own environment from a test thread.
    fn build_with(&self, lookup: &dyn Fn(&str) -> Option<String>) -> Result<Instance, String> {
        let refused = |reason: String| Err(format!("error: {reason}"));
        if !self.root {
            return refused(
                "`--root` names the one mode this registry runs: `serve --root --corpus <path>` \
                 (ADR-027 retired the mode `--upstream` chose)"
                    .to_string(),
            );
        }
        let Some(corpus) = self.corpus.as_ref() else {
            return refused("`--root` needs `--corpus <path>` to answer from".to_string());
        };
        // `027-C4`: `<data>` is now a startup requirement, a posture change from root
        // mode, which needed none.
        let data = data_dir(DataLayout::host(), lookup).ok_or_else(|| {
            "error: a hosted registry needs a data directory: set `XDG_DATA_HOME` to an \
             absolute path (or `HOME`, whose `.local/share` it falls back to); no usable \
             `<data>` was resolved"
                .to_string()
        })?;
        let registry = registry_dir(&data);
        // §Concurrency, and §Hosted mode's order: `<data>` resolved, "whose write lock
        // the run then takes". One writer per `<data>`, taken before any document is
        // written and before anything else is refused, and held for the whole run —
        // including the one-shot `query`, which can mint. A run that is refused below
        // drops the lock on the way out, so a refusal leaves no lock behind.
        let lock = WriteLock::take(&registry)?;
        // The key is required *before* anything is loaded, regardless of snapshot
        // freshness: the refresh thread is part of the run (`027-C3`).
        let steam_key = if self.no_steam {
            None
        } else {
            let key = lookup(STEAM_KEY_ENV).filter(|value| !value.trim().is_empty());
            match key {
                Some(key) => Some(key),
                None => {
                    return refused(format!(
                        "the steam Web API key is not configured: set `{STEAM_KEY_ENV}`, or \
                         start `--no-steam` to serve with the steam resolver inactive"
                    ));
                }
            }
        };
        // The retired document pair is a refusal by *presence*, except at the path
        // `--corpus` names: an upgraded host cleans its data directory rather than
        // carrying inert state, and the exception is how an operator keeps reading
        // their old entries across the upgrade.
        let corpus_path = self.spared_document(corpus);
        for retired_document in [persist::CACHE_DOCUMENT, persist::CATALOG_DOCUMENT] {
            let path = document_path(&data, retired_document);
            if path.exists() && !same_document(&self.spared_document(&path), &corpus_path) {
                return refused(format!(
                    "`{}` is the caching mode's document, retired with ADR-027: delete it or \
                     point `--corpus` at it",
                    persist::echo_path(&path)
                ));
            }
        }
        // `026-C1` survives: the corpus has to exist.
        let corpus = Catalog::load_required(corpus, constants::MAX_CATALOG_DOCUMENT_OCTETS)
            .map_err(|error| error.line())?;
        let steam = match &steam_key {
            None => SteamPosture::Inactive,
            Some(_) => SteamPosture::Active,
        };
        let snapshot_path = document_path(&data, persist::STEAM_SNAPSHOT_DOCUMENT);
        let (mut store, mut snapshot) = (None, None);
        if let Some(key) = steam_key.as_ref() {
            let built = self.store(key)?;
            // `--no-steam` never reads the file at all, so it must never write one
            // either (§The steam snapshot resolver); this branch is the resolver's.
            let loaded = self.load_or_pull_snapshot(&built, &snapshot_path)?;
            store = Some(built);
            snapshot = loaded;
        }
        let issued = Catalog::load(
            &document_path(&data, persist::ISSUED_DOCUMENT),
            constants::MAX_ISSUED_DOCUMENT_OCTETS,
        )
        .map_err(|error| error.line())?;
        let retired = RetiredIds::load(&document_path(&data, persist::RETIRED_IDS_DOCUMENT))
            .map_err(|error| error.line())?;
        let state = State::hosted(corpus, issued, retired, snapshot, steam);
        Ok(Instance {
            state,
            snapshot_path,
            store,
            _lock: lock,
        })
    }

    /// The canonical spelling of one file, for the `--corpus` exception's comparison:
    /// two spellings of one path spare one document. A file that is not there is
    /// compared as given.
    fn spared_document(&self, path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
    /// The store client: the authority pinned at compile time (`027-C3`) and the
    /// verified transport, held as a *name*. Building one resolves nothing and dials
    /// nothing — `SteamStore` looks the name up when a page is about to be dialed — so
    /// an instance whose snapshot is on disk and fresh never consults the resolver, and
    /// nothing outside a pull can reach the network. No flag, environment value, or
    /// document can point this instance at another store.
    fn store(&self, key: &str) -> Result<SteamStore, String> {
        Ok(SteamStore::production(key))
    }

    /// Startup's load-or-pull decision, delegated to [`steam::startup_snapshot`] where
    /// the library face can drive it with a loopback store; this seam only adds the
    /// `error: ` the binary's refusal lines carry.
    ///
    /// The clause is `027-C3`'s: load only a present, loadable **and** fresh snapshot,
    /// so a past-bound one is the full pull's case. An operator who wants the network
    /// kept out of startup starts the instance `--no-steam`, the posture that neither
    /// reads nor pulls the file.
    fn load_or_pull_snapshot(
        &self,
        store: &SteamStore,
        path: &Path,
    ) -> Result<Option<steam::SteamSnapshot>, String> {
        steam::startup_snapshot(store, path, crate::now_unix())
            .map(Some)
            .map_err(|line| format!("error: {line}"))
    }
}

/// Whether two canonical spellings name one file.
///
/// The `--corpus` exception to the retired-document-pair refusal spares "the path
/// `--corpus` names", and a *string* comparison of canonical paths is a spelling
/// comparison: on the §Compatibility matrix's Windows row,
/// `%APPDATA%\BINDFORGE\REGISTRY\Catalog-V1.JSON` and
/// `<data>/bindforge/registry/catalog-v1.json` are one file, and a case-folding split
/// tells the operator to delete or re-point a document they have already spared. So
/// identity is asked for where the platform can answer it — device and inode on unix —
/// and everything else falls back to the case-insensitive spelling, which is the
/// direction that spares a file rather than refusing a start over nothing.
fn same_document(a: &Path, b: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        match (std::fs::metadata(a), std::fs::metadata(b)) {
            (Ok(first), Ok(second)) => (first.dev(), first.ino()) == (second.dev(), second.ino()),
            // A file that is not there has no identity to compare, which leaves the
            // spelling — and a missing file is not the spared one either way.
            _ => a == b,
        }
    }
    #[cfg(not(unix))]
    {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    }
}

/// A built instance: the state, the `<data>` it owns, and the write lock that keeps
/// the next one out until this one exits.
struct Instance {
    state: State,
    /// The snapshot file's path, and the store client the startup seam built. Both
    /// are `serve`'s refresh thread's; `query` never uses them, and `--no-steam`
    /// leaves the client unbuilt.
    snapshot_path: PathBuf,
    store: Option<SteamStore>,
    _lock: WriteLock,
}

/// Binds, prints the bound port, and serves until stopped.
fn serve_command(config: Config) -> Result<(), String> {
    let instance = config.build()?;
    let listener = bind(&config.host, config.port)?;
    let bound = listener
        .local_addr()
        .map_err(|error| format!("error: the bound address is not readable: {error}"))?;
    // `--port 0` is the documented default, so the printed value is the contract.
    println!(
        "registry {} listening on http://{bound}",
        instance.state.mode().as_str()
    );
    std::io::stdout()
        .flush()
        .map_err(|error| format!("error: printing the bound port failed: {error}"))?;
    // Enablement is judged from the address actually bound, not from the spelling
    // that was asked for (`026-C5`).
    let admin = Admin::new(bound.ip().is_loopback(), config.admin_token.clone());
    let state = Arc::new(instance.state);
    // §The store pull: the refresh thread lives here and nowhere else, takes only the
    // snapshot's lock, and is the snapshot's only writer. `--no-steam` built no store,
    // so it starts no thread.
    if let Some(store) = instance.store {
        spawn_refresh(Arc::clone(&state), store, instance.snapshot_path);
    }
    serve::serve(listener, state, Arc::new(admin))
        .map_err(|error| format!("error: the accept loop stopped: {error}"))
}

/// Runs the refresh schedule §The store pull states: sleep the full interval, pull,
/// publish through the tmp-and-rename and the model swap, and on failure keep the old
/// snapshot, print one line, and retry on the shorter clock. Never the mutation lock,
/// never the corpus or issued documents.
fn spawn_refresh(state: Arc<State>, store: SteamStore, snapshot_path: PathBuf) {
    if let Err(error) = std::thread::Builder::new()
        .name("steam-refresh".to_string())
        .spawn(move || {
            // The cadence is the pure function [`steam::next_refresh_seconds`], and the
            // thread sleeps on it and nothing else. The anchor is the snapshot's own
            // `fetched_at` — the moment the pull that produced it ran — because that is
            // when the last pull actually happened: a startup that *pulled* seeds this
            // with the present clock and so sleeps the full interval, and a startup that
            // *loaded* a nearly-age-bound one sleeps only what is left before `027-B2`'s
            // bound, instead of leaving steam misses on the named `503` for a day after
            // a boot the design made legal.
            let mut outcome = steam::PullOutcome::Published;
            let mut last_pull_at = state
                .snapshot_slot()
                .read()
                .ok()
                .and_then(|slot| slot.as_ref().map(|slot| slot.fetched_at()))
                .unwrap_or_else(crate::now_unix);
            loop {
                let now = crate::now_unix();
                let wait = steam::next_refresh_seconds(now, last_pull_at, outcome);
                std::thread::sleep(Duration::from_secs(wait));
                // The clock read *before* the pull's first page is the stamp the pull
                // publishes with (§The store pull), so it is also the anchor the next
                // gap is measured from.
                let started_at = crate::now_unix();
                let loaded = state
                    .snapshot_slot()
                    .read()
                    .ok()
                    .and_then(|slot| slot.as_ref().map(Arc::clone));
                let pulled = match loaded {
                    Some(loaded) => store.delta(started_at, &loaded),
                    None => store.full(started_at),
                };
                last_pull_at = started_at;
                outcome = match pulled {
                    // A pull that produced a snapshot the file refused is this
                    // thread's failure posture too: the old snapshot keeps serving,
                    // one line is printed, and the retry is on the short clock.
                    // Scoring it `Published` would sleep the full interval over a
                    // supply that is already running down `027-B2`'s age bound.
                    Ok(snapshot) => match snapshot.publish(&snapshot_path) {
                        Ok(()) => {
                            state.publish_snapshot(snapshot);
                            steam::PullOutcome::Published
                        }
                        Err(error) => {
                            eprintln!(
                                "warning: the steam snapshot could not be published: {}",
                                persist::echo_safe(&error.to_string())
                            );
                            steam::PullOutcome::Failed
                        }
                    },
                    Err(error) => {
                        // One line, naming the condition and never the URL or the key
                        // (`027-L4`); the old snapshot keeps serving.
                        eprintln!(
                            "warning: the steam pull: {}",
                            persist::echo_safe(&error.to_string())
                        );
                        steam::PullOutcome::Failed
                    }
                };
            }
        })
    {
        eprintln!(
            "warning: the steam refresh thread could not start: {error}; the snapshot will \
             not refresh on its own"
        );
    }
}

/// Runs the hosted policy once for one key, prints the answer with its provenance,
/// and exits non-zero on every named-`503` posture. A `query` can mint, so it took
/// the same `<data>` write lock `serve` takes: it is a stopped-instance tool.
fn query_command(config: Config) -> Result<(), String> {
    let instance = config.build()?;
    let Some(key) = config.slug.as_ref() else {
        return Err("error: `query` needs one lookup key to ask about".to_string());
    };
    if !key_is_admissible(key) {
        return Err(format!(
            "error: `{}` is not a lookup key this registry answers",
            bindforge_core::escape_and_truncate(key)
        ));
    }
    match instance.state.candidates(key, crate::now_unix()) {
        Answer::Candidates {
            candidates,
            provenance,
        } => {
            println!(
                "queried `{}` answered from {}",
                bindforge_core::escape_and_truncate(key),
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
        // Every named `503` is a non-zero exit, with the same name the wire answers:
        // the status line is all the shipped CLI shows a user, and this is what a
        // script sees instead.
        Answer::Refusal(named) => Err(format!("error: {}/{}", named.code(), named.detail())),
    }
}

/// The named retirement `027-C1`'s flag posture owes an operator: it quotes the flag,
/// names ADR-027 and what retired, and is not the unknown-flag line.
#[must_use]
fn retirement_refusal(flag: &str) -> String {
    match flag {
        "--upstream" => "--upstream is not a flag this registry runs: caching mode retired \
             with ADR-027, so a hosted registry fetches from the steam store and its own \
             corpus alone"
            .to_string(),
        _ => "--refresh-interval is not a flag this registry runs: the caching mode's \
             upstream TTL retired with ADR-027; the steam snapshot refreshes on its own \
             schedule"
            .to_string(),
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
