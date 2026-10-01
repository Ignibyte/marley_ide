//! Marley's shell integration: the scripts that make a shell report each prompt and command to
//! the terminal as Marley's hook frames, and what a terminal adds to a shell's start to load
//! them.
//!
//! The scripts are embedded here, and [`install_in`] writes them to a directory the caller
//! names, from which the shell reads them at startup. [`for_program`] says how to start a
//! program with them. bash takes `--rcfile`, and its script sources the user's own `~/.bashrc`
//! first. zsh takes `ZDOTDIR`, and its `.zshenv` puts the user's `ZDOTDIR` back, so zsh reads
//! the user's own files as it would have. fish takes a vendor snippet from the first folder of
//! `XDG_DATA_DIRS`, which puts the user's `XDG_DATA_DIRS` back before their `config.fish` runs
//! (#466). [`shown_arguments`] is what the user is shown of a process's arguments: all but the
//! ones [`for_program`] adds.
//!
//! A terminal gives its program a [`new_nonce`] in [`NONCE_VARIABLE`]. The scripts take it out
//! of the environment before the user's files run and add it to each command's frame, which is
//! how the terminal tells the shell's own frames from output that prints one.
//!
//! A local terminal also gives its programs [`browser_opener`] as [`BROWSER_VARIABLE`] while one
//! is set (#561): Marley's opener, which puts a local URL a program opens in a Browser tab of its
//! project. The workbench sets it with [`set_browser_opener`].
//!
//! The scripts define an `ssh` function (#526): an interactive login runs [`ssh_remote_command`]
//! on the host, which starts the host's bash or zsh with the same scripts, carried in the command
//! and removed once read, and a nonce of the connection's own that the scripts announce first.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

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

/// Marley's fish integration script (#466).
pub const FISH_INTEGRATION: &str = include_str!("../shell_integration/marley.fish");

/// The directory, inside the one [`install_in`] is given, that fish reads vendor snippets from
/// when that one is on `XDG_DATA_DIRS`.
pub const FISH_DIR: &str = "fish/vendor_conf.d";

/// The file in [`FISH_DIR`] that [`install_in`] writes [`FISH_INTEGRATION`] to.
pub const FISH_FILE: &str = "marley.fish";

/// The variable that carries the user's own `XDG_DATA_DIRS` into a fish started with Marley's
/// folder first on it, for [`FISH_INTEGRATION`] to put back.
pub const FISH_DATA_DIRS_VARIABLE: &str = "MARLEY_FISH_DATA_DIRS";

/// The data directories the XDG specification takes when `XDG_DATA_DIRS` is unset.
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// The variable a shell started with Marley's integration finds set.
pub const MARKER_VARIABLE: &str = "MARLEY_SHELL_INTEGRATION";

/// The variable that gives a terminal's program the terminal's nonce.
pub const NONCE_VARIABLE: &str = "MARLEY_SHELL_NONCE";

/// The variable that, as `0`, has the scripts keep a line typed with a leading space out of the
/// shell's history: how an agent's `terminal_run` commands stay out of it (#553).
pub const AGENT_HISTORY_VARIABLE: &str = "MARLEY_AGENT_HISTORY";

/// The file, beside the scripts, holding the remote command Marley's `ssh` runs on a host (#526).
pub const SSH_COMMAND_FILE: &str = "ssh-remote-command";

/// How [`ssh_remote_command`] starts, which [`shown_arguments`] knows it by.
const SSH_COMMAND_START: &str = "sh -c 'b=$(printf %s ";

/// The variable naming [`SSH_COMMAND_FILE`] for the scripts, which read it and take it out of the
/// environment as they do the nonce.
pub const SSH_COMMAND_VARIABLE: &str = "MARLEY_SSH_COMMAND";

/// The POSIX sh that [`ssh_remote_command`] carries to a host (#526).
///
/// It writes the two scripts into a folder of `mktemp -d`, then starts the host's bash or zsh with
/// them and the connection's nonce, its first argument; the scripts remove the folder once read.
/// Any other shell, or a host that cannot make the folder, gets its login shell as plain ssh
/// starts it.
#[must_use]
pub fn ssh_bootstrap() -> String {
    format!(
        r#"dir=$(mktemp -d "${{TMPDIR:-/tmp}}/marley.XXXXXX" 2>/dev/null) || exec "${{SHELL:-/bin/sh}}" -l
case ${{SHELL##*/}} in
bash)
    cat > "$dir/{BASH_FILE}" <<'MARLEY_BASH_SCRIPT_END'
{BASH_INTEGRATION}MARLEY_BASH_SCRIPT_END
    MARLEY_SHELL_NONCE=$1 __MARLEY_LOGIN=1 __MARLEY_CLEANUP=$dir exec bash --rcfile "$dir/{BASH_FILE}" -i
    ;;
zsh)
    mkdir "$dir/{ZSH_DIR}"
    cat > "$dir/{ZSH_DIR}/{ZSH_FILE}" <<'MARLEY_ZSH_SCRIPT_END'
{ZSH_INTEGRATION}MARLEY_ZSH_SCRIPT_END
    if [ -n "${{ZDOTDIR+set}}" ]; then
        {ZSH_ZDOTDIR_VARIABLE}=$ZDOTDIR
        export {ZSH_ZDOTDIR_VARIABLE}
    fi
    ZDOTDIR=$dir/{ZSH_DIR} MARLEY_SHELL_NONCE=$1 __MARLEY_CLEANUP=$dir exec zsh -l
    ;;
esac
rm -rf "$dir"
exec "${{SHELL:-/bin/sh}}" -l
"#
    )
}

/// The command Marley's `ssh` gives a host, the connection's nonce appended (#526).
///
/// It is one line with no backslash, `!` or newline, so a login shell that is fish or tcsh reads
/// it as sh does: `sh -c` decodes [`ssh_bootstrap`] from base64 and runs it, and a host without
/// `base64` starts its login shell plain.
#[must_use]
pub fn ssh_remote_command() -> String {
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(ssh_bootstrap());
    format!(
        "{SSH_COMMAND_START}{encoded} | base64 -d 2>/dev/null) && eval \"$b\" || exec \"${{SHELL:-/bin/sh}}\" -l' marley"
    )
}

/// A new random nonce for a terminal, as 32 hex digits.
#[must_use]
pub fn new_nonce() -> String {
    format!("{:032x}", rand::random::<u128>())
}

/// The variable programs read for the program that opens a URL for them.
pub const BROWSER_VARIABLE: &str = "BROWSER";

/// The opener terminals started from now on give their programs, if any.
static BROWSER_OPENER: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Sets the opener each local terminal started from now on gives its programs as
/// [`BROWSER_VARIABLE`], or none, so they keep the one they inherit.
pub fn set_browser_opener(opener: Option<PathBuf>) {
    *BROWSER_OPENER
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = opener;
}

/// The opener a new local terminal gives its programs, if one is set.
#[must_use]
pub fn browser_opener() -> Option<PathBuf> {
    BROWSER_OPENER
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

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
    let fish_dir = dir.join(FISH_DIR);
    std::fs::create_dir_all(&fish_dir)?;
    write_if_changed(&dir.join(BASH_FILE), BASH_INTEGRATION)?;
    write_if_changed(&zsh_dir.join(ZSH_FILE), ZSH_INTEGRATION)?;
    write_if_changed(&fish_dir.join(FISH_FILE), FISH_INTEGRATION)?;
    write_if_changed(&dir.join(SSH_COMMAND_FILE), &ssh_remote_command())
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
/// The scripts are read from `dir`, where [`install_in`] writes them. `user_zdotdir` and
/// `user_data_dirs` are the `ZDOTDIR` and `XDG_DATA_DIRS` the shell would otherwise have
/// inherited.
#[must_use]
pub fn for_program(
    program: &str,
    dir: &Path,
    user_zdotdir: Option<&str>,
    user_data_dirs: Option<&str>,
) -> Option<ShellIntegration> {
    let name = Path::new(program).file_stem()?.to_str()?;
    let marker = (MARKER_VARIABLE.to_string(), "1".to_string());
    let ssh_command = (
        SSH_COMMAND_VARIABLE.to_string(),
        dir.join(SSH_COMMAND_FILE).display().to_string(),
    );
    match name {
        "bash" => Some(ShellIntegration {
            args: vec![
                "--rcfile".to_string(),
                dir.join(BASH_FILE).display().to_string(),
            ],
            env: vec![marker, ssh_command],
        }),
        "zsh" => Some(ShellIntegration {
            args: Vec::new(),
            env: [
                (
                    "ZDOTDIR".to_string(),
                    dir.join(ZSH_DIR).display().to_string(),
                ),
                marker,
                ssh_command,
            ]
            .into_iter()
            .chain(
                user_zdotdir.map(|zdotdir| (ZSH_ZDOTDIR_VARIABLE.to_string(), zdotdir.to_string())),
            )
            .collect(),
        }),
        "fish" => Some(ShellIntegration {
            args: Vec::new(),
            env: [
                (
                    "XDG_DATA_DIRS".to_string(),
                    format!(
                        "{}:{}",
                        dir.display(),
                        user_data_dirs.unwrap_or(DEFAULT_DATA_DIRS)
                    ),
                ),
                marker,
                ssh_command,
            ]
            .into_iter()
            .chain(
                user_data_dirs
                    .map(|data_dirs| (FISH_DATA_DIRS_VARIABLE.to_string(), data_dirs.to_string())),
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
    // Marley's ssh (#526): the `-t` and the remote command it added carry the bootstrap and the
    // connection's nonce, so the title shows the ssh the user typed.
    if Path::new(program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        == Some("ssh")
        && let Some((last, rest)) = arguments.split_last()
        && last.starts_with(SSH_COMMAND_START)
    {
        let rest = match rest.split_first() {
            Some((first, after)) if first == "-t" => after,
            _ => rest,
        };
        return rest.iter().map(String::as_str).collect();
    }
    let added =
        for_program(program, dir, None, None).map_or_else(Vec::new, |integration| integration.args);
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
            // Marley's `ssh` command rides along since #526.
            env: pairs(&[
                ("MARLEY_SHELL_INTEGRATION", "1"),
                (
                    "MARLEY_SSH_COMMAND",
                    "/data/marley/shell_integration/ssh-remote-command",
                ),
            ]),
        });
        assert_eq!(for_program("bash", dir, None, None), bash);
        assert_eq!(
            for_program("/usr/bin/bash", dir, Some("/home/me/zsh"), None),
            bash
        );
        for other in ["/bin/sh", "nu", "", "/"] {
            assert_eq!(for_program(other, dir, None, None), None, "{other}");
        }
    }

    #[test]
    fn zsh_starts_with_marleys_zdotdir_and_carries_the_users_own() {
        let dir = Path::new("/data/marley/shell_integration");
        // Marley's `ssh` command rides along since #526.
        let marleys = [
            ("ZDOTDIR", "/data/marley/shell_integration/zsh"),
            ("MARLEY_SHELL_INTEGRATION", "1"),
            (
                "MARLEY_SSH_COMMAND",
                "/data/marley/shell_integration/ssh-remote-command",
            ),
        ];
        let zsh = Some(ShellIntegration {
            args: Vec::new(),
            env: pairs(&marleys),
        });
        assert_eq!(for_program("zsh", dir, None, None), zsh);
        assert_eq!(for_program("/usr/bin/zsh", dir, None, None), zsh);
        let mut with_users = marleys.to_vec();
        with_users.push(("MARLEY_ZSH_ZDOTDIR", "/home/me/.config/zsh"));
        assert_eq!(
            for_program("zsh", dir, Some("/home/me/.config/zsh"), None),
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

    #[test]
    fn a_nonce_is_32_hex_digits_and_new_each_time() {
        let nonce = new_nonce();
        assert_eq!(nonce.len(), 32, "{nonce}");
        assert!(
            nonce.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "{nonce}"
        );
        assert_ne!(new_nonce(), nonce);
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
