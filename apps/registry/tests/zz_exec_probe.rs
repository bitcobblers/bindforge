//! The harness precondition the Test strategy #16/#17 CLI end-to-end suites stand
//! on: `cargo test --workspace` has built the shipped `discovery` CLI beside the
//! registry, and the harness resolves it to the *executable file* — never a
//! same-named directory in the target tree, which is what made every CLI leg die
//! at spawn (`Permission denied`) before it ever reached the socket.
//!
//! proves: design 026's Test strategy #16/#17 precondition — the harness pin that
//! both binaries exist for the pair's suites, so their failures stay about the
//! registry's answers rather than about a missing executable.

mod common;

use std::process::{Command, Stdio};

#[test]
fn the_shipped_cli_is_built_beside_the_registry_for_the_e2e_suites() {
    let cli = common::discovery_cli_bin();
    assert!(
        cli.is_file(),
        "the harness must resolve the shipped CLI to an executable file, not `{}` ({})",
        cli.display(),
        if cli.is_dir() {
            "a directory"
        } else {
            "missing"
        }
    );
    // The spawn itself is the fact under test: a path that is not the binary, or a
    // file that is not executable, dies here — the exact failure mode the #16/#17
    // suites would otherwise report as a transport mystery.
    let spawned = Command::new(&cli)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(mut child) => {
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(err) => panic!("`{}` must spawn as the shipped CLI: {err}", cli.display()),
    }
}
