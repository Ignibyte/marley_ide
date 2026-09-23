//! Marley's shell integration: the scripts that make a shell report each prompt and command to
//! the terminal as Marley's hook frames, and what a terminal adds to a shell's start to load
//! them.
//!
//! The scripts are embedded here, and [`install_in`] writes them to a directory the caller
//! names, from which the shell reads them at startup. [`for_program`] says how to start a
//! program with them. bash takes `--rcfile`, and its script sources the user's own `~/.bashrc`
//! first. zsh and fish follow (#465, #466).

use std::io;
use std::path::Path;

/// Marley's bash integration script.
pub const BASH_INTEGRATION: &str = include_str!("../shell_integration/marley.bash");

/// The file [`install_in`] writes [`BASH_INTEGRATION`] to.
pub const BASH_FILE: &str = "marley.bash";

/// The variable a shell started with Marley's integration finds set.
pub const MARKER_VARIABLE: &str = "MARLEY_SHELL_INTEGRATION";

/// What to add to a shell's start so it loads Marley's integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellIntegration {
    /// The arguments to start the shell with.
    pub args: Vec<String>,
    /// The environment variables to add.
    pub env: Vec<(String, String)>,
}

/// Writes Marley's integration scripts into `dir`, creating it.
///
/// A script already there with the same content is left alone; one that is missing, unreadable
/// or different is written.
///
/// # Errors
///
/// Creating the directory or writing a script.
pub fn install_in(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(BASH_FILE);
    let current = std::fs::read_to_string(&path).ok();
    if current.as_deref() != Some(BASH_INTEGRATION) {
        std::fs::write(&path, BASH_INTEGRATION)?;
    }
    Ok(())
}

/// How to start `program` with Marley's integration, reading the scripts from `dir`, where
/// [`install_in`] writes them. `None` for a program Marley has no integration for.
#[must_use]
pub fn for_program(program: &str, dir: &Path) -> Option<ShellIntegration> {
    let name = Path::new(program).file_stem()?.to_str()?;
    match name {
        "bash" => Some(ShellIntegration {
            args: vec![
                "--rcfile".to_string(),
                dir.join(BASH_FILE).display().to_string(),
            ],
            env: vec![(MARKER_VARIABLE.to_string(), "1".to_string())],
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_starts_with_marleys_rcfile_and_other_programs_are_left_alone() {
        let dir = Path::new("/data/marley/shell_integration");
        let bash = Some(ShellIntegration {
            args: vec![
                "--rcfile".to_string(),
                "/data/marley/shell_integration/marley.bash".to_string(),
            ],
            env: vec![("MARLEY_SHELL_INTEGRATION".to_string(), "1".to_string())],
        });
        assert_eq!(for_program("bash", dir), bash);
        assert_eq!(for_program("/usr/bin/bash", dir), bash);
        for other in ["zsh", "/bin/sh", "fish", "", "/"] {
            assert_eq!(for_program(other, dir), None, "{other}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn install_writes_the_script_leaves_an_identical_one_and_rewrites_a_changed_one() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt as _;

        let root = tempfile::tempdir().expect("a scratch directory");
        let dir = root.path().join("shell_integration");
        install_in(&dir).expect("installed");
        let path = dir.join(BASH_FILE);
        assert_eq!(
            std::fs::read_to_string(&path).expect("the script"),
            BASH_INTEGRATION
        );

        // An identical script is not written again: read-only, it still installs.
        std::fs::set_permissions(&path, Permissions::from_mode(0o444)).expect("read-only");
        install_in(&dir).expect("left alone");

        // A changed one is rewritten.
        std::fs::set_permissions(&path, Permissions::from_mode(0o644)).expect("writable");
        std::fs::write(&path, "stale").expect("changed");
        install_in(&dir).expect("rewritten");
        assert_eq!(
            std::fs::read_to_string(&path).expect("the script"),
            BASH_INTEGRATION
        );
    }
}
