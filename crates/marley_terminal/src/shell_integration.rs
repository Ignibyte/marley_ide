//! Marley's shell integration: the scripts that make a shell report each prompt and command to
//! the terminal as Marley's hook frames, and what a terminal adds to a shell's start to load
//! them.
//!
//! The scripts are embedded here, and [`install_in`] writes them to a directory the caller
//! names, from which the shell reads them at startup. [`for_program`] says how to start a
//! program with them. bash takes `--rcfile`, and its script sources the user's own `~/.bashrc`
//! first. zsh takes `ZDOTDIR`, and its `.zshenv` puts the user's `ZDOTDIR` back, so zsh reads
//! the user's own files as it would have. fish follows (#466). [`shown_arguments`] is what the
//! user is shown of a process's arguments: all but the ones [`for_program`] adds.

use std::io;
use std::path::Path;

/// Marley's bash integration script.
pub const BASH_INTEGRATION: &str = include_str!("../shell_integration/marley.bash");

/// The file [`install_in`] writes [`BASH_INTEGRATION`] to.
pub const BASH_FILE: &str = "marley.bash";

/// Marley's zsh integration script.
pub const ZSH_INTEGRATION: &str = include_str!("../shell_integration/marley.zsh");

/// The directory, inside the one [`install_in`] is given, that zsh starts with as its
/// `ZDOTDIR`.
pub const ZSH_DIR: &str = "zsh";

/// The file in [`ZSH_DIR`] that [`install_in`] writes [`ZSH_INTEGRATION`] to: the first
/// startup file zsh reads.
pub const ZSH_FILE: &str = ".zshenv";

/// The variable that carries the user's own `ZDOTDIR` into a zsh started with Marley's, for
/// [`ZSH_INTEGRATION`] to put back.
pub const ZSH_ZDOTDIR_VARIABLE: &str = "MARLEY_ZSH_ZDOTDIR";

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
/// Creating a directory or writing a script.
pub fn install_in(dir: &Path) -> io::Result<()> {
    let zsh_dir = dir.join(ZSH_DIR);
    std::fs::create_dir_all(&zsh_dir)?;
    write_if_changed(&dir.join(BASH_FILE), BASH_INTEGRATION)?;
    write_if_changed(&zsh_dir.join(ZSH_FILE), ZSH_INTEGRATION)
}

fn write_if_changed(path: &Path, content: &str) -> io::Result<()> {
    let current = std::fs::read_to_string(path).ok();
    if current.as_deref() == Some(content) {
        return Ok(());
    }
    std::fs::write(path, content)
}

/// How to start `program` with Marley's integration, or `None` for a program it has none for.
///
/// The scripts are read from `dir`, where [`install_in`] writes them. `user_zdotdir` is the
/// `ZDOTDIR` the shell would otherwise have inherited.
#[must_use]
pub fn for_program(
    program: &str,
    dir: &Path,
    user_zdotdir: Option<&str>,
) -> Option<ShellIntegration> {
    let name = Path::new(program).file_stem()?.to_str()?;
    let marker = (MARKER_VARIABLE.to_string(), "1".to_string());
    match name {
        "bash" => Some(ShellIntegration {
            args: vec![
                "--rcfile".to_string(),
                dir.join(BASH_FILE).display().to_string(),
            ],
            env: vec![marker],
        }),
        "zsh" => Some(ShellIntegration {
            args: Vec::new(),
            env: [
                (
                    "ZDOTDIR".to_string(),
                    dir.join(ZSH_DIR).display().to_string(),
                ),
                marker,
            ]
            .into_iter()
            .chain(
                user_zdotdir.map(|zdotdir| (ZSH_ZDOTDIR_VARIABLE.to_string(), zdotdir.to_string())),
            )
            .collect(),
        }),
        _ => None,
    }
}

/// The arguments to show of the process `argv`, without the ones Marley's integration added.
///
/// Every argument after the program is kept, less the arguments [`for_program`] adds to that
/// program with the scripts in `dir` where they appear as one run, so a shell Marley started
/// with its integration reads as the same shell started without it.
#[must_use]
pub fn shown_arguments<'a>(argv: &'a [String], dir: &Path) -> Vec<&'a str> {
    let Some((program, arguments)) = argv.split_first() else {
        return Vec::new();
    };
    let added =
        for_program(program, dir, None).map_or_else(Vec::new, |integration| integration.args);
    // `windows` takes no empty run.
    let run = (!added.is_empty())
        .then(|| {
            arguments
                .windows(added.len())
                .position(|window| window == added)
        })
        .flatten()
        .map_or(0..0, |start| start..start + added.len());
    arguments
        .iter()
        .enumerate()
        .filter(|(index, _)| !run.contains(index))
        .map(|(_, argument)| argument.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn bash_starts_with_marleys_rcfile_and_other_programs_are_left_alone() {
        let dir = Path::new("/data/marley/shell_integration");
        let bash = Some(ShellIntegration {
            args: vec![
                "--rcfile".to_string(),
                "/data/marley/shell_integration/marley.bash".to_string(),
            ],
            env: pairs(&[("MARLEY_SHELL_INTEGRATION", "1")]),
        });
        assert_eq!(for_program("bash", dir, None), bash);
        assert_eq!(
            for_program("/usr/bin/bash", dir, Some("/home/me/zsh")),
            bash
        );
        for other in ["/bin/sh", "fish", "", "/"] {
            assert_eq!(for_program(other, dir, None), None, "{other}");
        }
    }

    #[test]
    fn zsh_starts_with_marleys_zdotdir_and_carries_the_users_own() {
        let dir = Path::new("/data/marley/shell_integration");
        let marleys = [
            ("ZDOTDIR", "/data/marley/shell_integration/zsh"),
            ("MARLEY_SHELL_INTEGRATION", "1"),
        ];
        let zsh = Some(ShellIntegration {
            args: Vec::new(),
            env: pairs(&marleys),
        });
        assert_eq!(for_program("zsh", dir, None), zsh);
        assert_eq!(for_program("/usr/bin/zsh", dir, None), zsh);
        let mut with_users = marleys.to_vec();
        with_users.push(("MARLEY_ZSH_ZDOTDIR", "/home/me/.config/zsh"));
        assert_eq!(
            for_program("zsh", dir, Some("/home/me/.config/zsh")),
            Some(ShellIntegration {
                args: Vec::new(),
                env: pairs(&with_users),
            })
        );
    }

    fn argv(arguments: &[&str]) -> Vec<String> {
        arguments.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_shell_started_with_marleys_integration_shows_only_its_own_arguments() {
        let dir = Path::new("/data/marley/shell_integration");
        let script = "/data/marley/shell_integration/marley.bash";
        assert!(shown_arguments(&argv(&["bash", "--rcfile", script]), dir).is_empty());
        assert!(shown_arguments(&argv(&["/usr/bin/bash", "--rcfile", script]), dir).is_empty());
        assert_eq!(
            shown_arguments(&argv(&["bash", "-l", "--rcfile", script, "-x"]), dir),
            ["-l", "-x"]
        );
    }

    #[test]
    fn any_other_process_shows_every_argument() {
        let dir = Path::new("/data/marley/shell_integration");
        let script = "/data/marley/shell_integration/marley.bash";
        for arguments in [
            &["zsh", "--rcfile", script][..],
            &["zsh", "-l"],
            &["bash", "--rcfile", "/home/me/.bashrc"],
            &["bash", "--rcfile"],
            &["bash", script, "--rcfile"],
            &["bash", "--rcfile", "-i", script],
            &["vim", "notes.md"],
            &["bash"],
        ] {
            assert_eq!(
                shown_arguments(&argv(arguments), dir),
                arguments[1..],
                "{arguments:?}"
            );
        }
        assert!(shown_arguments(&[], dir).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn install_writes_the_scripts_leaves_identical_ones_and_rewrites_changed_ones() {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt as _;

        let root = tempfile::tempdir().expect("a scratch directory");
        let dir = root.path().join("shell_integration");
        install_in(&dir).expect("installed");
        let scripts = [
            (dir.join(BASH_FILE), BASH_INTEGRATION),
            (dir.join(ZSH_DIR).join(ZSH_FILE), ZSH_INTEGRATION),
        ];
        for (path, content) in &scripts {
            assert_eq!(&std::fs::read_to_string(path).expect("a script"), content);
        }

        // Identical scripts are not written again: read-only, they still install.
        for (path, _) in &scripts {
            std::fs::set_permissions(path, Permissions::from_mode(0o444)).expect("read-only");
        }
        install_in(&dir).expect("left alone");

        // Changed ones are rewritten.
        for (path, _) in &scripts {
            std::fs::set_permissions(path, Permissions::from_mode(0o644)).expect("writable");
            std::fs::write(path, "stale").expect("changed");
        }
        install_in(&dir).expect("rewritten");
        for (path, content) in &scripts {
            assert_eq!(&std::fs::read_to_string(path).expect("a script"), content);
        }
    }
}
