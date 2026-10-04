//! The workbench's one module that starts programs (CONSTITUTION §14, gate:22).
//!
//! `claude`, `voxtype`, `git` and `gh` run through `output` and `follow`, and nothing else in
//! the crate spawns a process. A program the workbench runs goes through one of them, so the
//! gate's count of spawn calls stays where it is.

use std::ffi::OsStr;
use std::path::Path;
use std::process::Output;

use anyhow::Context as _;
use util::command::{Child, Stdio};

/// Runs `program` with `args` in `dir` (Marley's own folder when `None`), with `env` added, and
/// gives what it printed and how it ended, unjudged. Dropped before it ends, as a call raced
/// against a timeout is, the program is killed rather than left running (#648).
///
/// # Errors
///
/// When the program cannot start.
pub(crate) async fn output<I, S>(
    program: impl AsRef<OsStr>,
    args: I,
    dir: Option<&Path>,
    env: &[(&str, &str)],
) -> std::io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = util::command::new_command(program);
    command.args(args).kill_on_drop(true);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    for (key, value) in env {
        command.env(key, value);
    }
    command.output().await
}

/// Runs `program` with `args`, an error carrying what it printed when it fails. The agent bar's
/// adapters run `claude` and `voxtype` with it.
///
/// # Errors
///
/// When the program cannot start, or exits with a failure.
pub(crate) async fn run_program(program: &Path, args: &[&OsStr]) -> anyhow::Result<()> {
    let name = program
        .file_name()
        .unwrap_or(program.as_os_str())
        .to_string_lossy();
    let output = output(program, args, None, &[])
        .await
        .context(format!("running `{name}`"))?;
    anyhow::ensure!(
        output.status.success(),
        "`{name} {}` failed: {}",
        args.iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

/// Starts `program` with `args` to read its output and its errors as they come: stdin closed,
/// stdout and stderr piped, and the process killed when the child is dropped (#632).
///
/// # Errors
///
/// When the program cannot start.
pub(crate) fn follow_with_errors(program: &Path, args: &[&OsStr]) -> std::io::Result<Child> {
    following(program, args, Stdio::piped())
}

/// Starts `program` with `args` to read its output as it comes: stdin and stderr closed, stdout
/// piped, and the process killed when the child is dropped.
///
/// # Errors
///
/// When the program cannot start.
pub(crate) fn follow(program: &Path, args: &[&str]) -> std::io::Result<Child> {
    following(program, args, Stdio::null())
}

/// Starts `program` with `args`, stdin closed, stdout piped, stderr as `stderr` says, killed when
/// the child is dropped.
fn following(program: &Path, args: &[impl AsRef<OsStr>], stderr: Stdio) -> std::io::Result<Child> {
    util::command::new_command(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .kill_on_drop(true)
        .spawn()
}
