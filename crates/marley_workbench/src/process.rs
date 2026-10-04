//! The workbench's one module that starts programs (CONSTITUTION §14, gate:22).
//!
//! `claude`, `voxtype`, `git`, `gh`, `rh` and `codex` run through `output`, `follow` and `serve`,
//! and nothing else in the crate spawns a process. A program the workbench runs goes through one of them, so the
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
    following(program, args, (Stdio::piped(), Stdio::piped()), None)
}

/// Starts `program` with `args` in `dir`, with `env` added to Marley's own, as a server Marley
/// owns: stdin and stdout closed, stderr piped for its errors, and the process killed when the
/// child is dropped (#650).
///
/// # Errors
///
/// When the program cannot start.
pub(crate) fn serve(
    program: &Path,
    args: &[&OsStr],
    dir: &Path,
    env: &[(String, String)],
) -> std::io::Result<Child> {
    following(
        program,
        args,
        (Stdio::null(), Stdio::piped()),
        Some((dir, env)),
    )
}

/// Starts `program` with `args` to read its output as it comes: stdin and stderr closed, stdout
/// piped, and the process killed when the child is dropped.
///
/// # Errors
///
/// When the program cannot start.
pub(crate) fn follow(program: &Path, args: &[&str]) -> std::io::Result<Child> {
    following(program, args, (Stdio::piped(), Stdio::null()), None)
}

/// Starts `program` with `args`, stdin closed, stdout and stderr as given, in `place`'s
/// folder with its variables added when given, killed when the child is dropped.
fn following(
    program: &Path,
    args: &[impl AsRef<OsStr>],
    (stdout, stderr): (Stdio, Stdio),
    place: Option<(&Path, &[(String, String)])>,
) -> std::io::Result<Child> {
    let mut command = util::command::new_command(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .kill_on_drop(true);
    if let Some((dir, env)) = place {
        command.current_dir(dir);
        command.envs(env.iter().map(|(key, value)| (key, value)));
    }
    command.spawn()
}
