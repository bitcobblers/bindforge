//! The `<data>` write lock (ADR-027, §Concurrency).
//!
//! Two persist-before-serve writers on one `<data>` rename models they each read at
//! their own startup over each other's `027-L2` records, so the process that wants
//! to write takes `<data>/bindforge/registry/write.lock` under `create_new` before
//! anything binds and removes it when it exits. `serve` and `query` both take it —
//! a one-shot `query` can mint, so it is a writer like any other.
//!
//! Finding the file there **is** the named startup refusal `registry data directory
//! is locked` — full stop. §Concurrency's sentence is "finding it present is the
//! named startup refusal", and the proposal's answer to a crashed run's leftover is
//! a runbook step ("a crashed run leaves the file; the runbook names deleting it"),
//! not an act a run performs while it starts. An earlier draft added stale-owner
//! adoption — read the pid, ask `/proc`, take over a corpse's lock — and that
//! mechanism is not the design's: it decides *between* processes who holds the
//! catalog, on evidence the lock file cannot carry. `create_new`'s exclusivity is
//! the mutual exclusion §Concurrency needs; a liveness heuristic bolted beside it is
//! one more way for two writers to reach the same documents, and the review r1c0
//! (S3) named it the crate's own double-write hazard. So a present lock is refused,
//! and the pid in it is information for the operator's line, not input to a decision.
//!
//! There is no `flock` here: this crate has no syscall dependency, and the guard
//! this lock has to provide is the one §Concurrency names — that a second *process*
//! is refused at startup — which an exclusive-creation file gives. The `create_new`
//! flag is what makes two simultaneous startups one winner rather than a window.

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::persist;

/// The lock this process holds, and the file that says so. Removing it is `Drop`'s
/// job, so a `serve` that stops and a `query` that prints its answer both clean up;
/// a run that is killed from outside leaves the file, and the runbook's cleanup step
/// is what clears it for the next run.
#[derive(Debug)]
pub struct WriteLock {
    path: PathBuf,
}

/// Why the lock was not available. One line, in the words §Concurrency names.
#[derive(Debug)]
pub struct Locked {
    pub path: PathBuf,
    /// What was found there, for the operator's line: `stamped owner <pid>`, or
    /// `no owner recorded`. Diagnostic only — the refusal stands either way.
    pub found: String,
}

impl fmt::Display for Locked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "registry data directory is locked: {} is held ({}); a run that was killed \
             leaves this file, and deleting it is the runbook's recovery step",
            persist::echo_path(&self.path),
            self.found
        )
    }
}

impl WriteLock {
    /// Takes the lock in `registry` (which is created owner-only if it is missing,
    /// since the lock is the first thing an instance puts there).
    ///
    /// # Errors
    ///
    /// Returns the named [`Locked`] refusal when the file is already there — whoever
    /// its stamp names, whatever that stamp says — and a plain message when the
    /// directory or the file could not be created for another reason.
    pub fn take(registry: &Path) -> Result<Self, String> {
        let path = registry.join(persist::WRITE_LOCK);
        persist::owner_only_dir(registry).map_err(|error| {
            format!(
                "error: the registry data directory `{}` could not be prepared: {error}",
                persist::echo_path(registry)
            )
        })?;
        if path.exists() {
            return Err(Self::locked_line(&path));
        }
        match persist::owner_only_new_file(&path) {
            Ok(mut file) => {
                // The stamp is the whole content: a decimal pid, nothing that could
                // be read as a credential or a path.
                if let Err(error) = writeln!(file, "{}", std::process::id()) {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(format!(
                        "error: the `<data>` write lock `{}` could not be stamped: {error}",
                        persist::echo_path(&path)
                    ));
                }
                if let Err(error) = file.flush() {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(format!(
                        "error: the `<data>` write lock `{}` could not be stamped: {error}",
                        persist::echo_path(&path)
                    ));
                }
                Ok(Self { path })
            }
            // Between the check above and this creation another process won the
            // race, which is what `create_new` is for.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(Self::locked_line(&path))
            }
            Err(error) => Err(format!(
                "error: the `<data>` write lock `{}` could not be taken: {error}",
                persist::echo_path(&path)
            )),
        }
    }

    /// The named refusal §Concurrency pins. What the file's stamp says goes into the
    /// parenthetical for the operator, and changes nothing about the answer.
    fn locked_line(path: &Path) -> String {
        let reason = Locked {
            path: path.to_path_buf(),
            found: match owner_pid(&Self::owner_of(path)) {
                Some(pid) => format!("stamped owner {pid}"),
                None => "no owner recorded".to_string(),
            },
        };
        format!("error: {reason}")
    }

    fn owner_of(path: &Path) -> String {
        std::fs::read_to_string(path)
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    }
}

impl Drop for WriteLock {
    fn drop(&mut self) {
        // The owner removes its own lock. A lock that is not there any more is not
        // an error worth a line: nothing else was asked to refuse it.
        let _ = std::fs::remove_file(&self.path);
    }
}

/// The pid a lock's content names, if it names one. Read for the operator's line
/// only: §Concurrency's refusal does not depend on it, which is also what makes the
/// answer the same on every platform.
fn owner_pid(content: &str) -> Option<u32> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<u32>().ok()
}
