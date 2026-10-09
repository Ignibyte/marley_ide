//! A terminal's identity, as its programs see it (#520).
//!
//! Each local interactive terminal starts with its own [`TERMINAL_ID_VARIABLE`], a UUID Zed's
//! terminal builder mints, and [`PROJECT_VARIABLE`], its project's folder. The Claude Code
//! plugin's MCP bridge sends both on each request, so Marley's tools know which terminal called
//! them. The id scopes what a tool does by default; it is no authority, since the MCP bearer
//! gates every call. A terminal Zed restores at a launch keeps the id it had (#575): the restore
//! hands the saved id to the builder under [`RESTORED_ID_VARIABLE`]. Beside the id, a local
//! interactive terminal names Marley's `marley-agent` program as [`BIN_VARIABLE`], through which
//! an agent reports its state to Marley (#652), and, while it is wanted, the Claude Code plugin
//! Marley shares with rustal-harness first in [`PLUGIN_DIRS_VARIABLE`] (#709).

/// The variable holding a terminal's id.
pub const TERMINAL_ID_VARIABLE: &str = "MARLEY_TERMINAL_ID";

/// The variable holding the folder of the terminal's project.
pub const PROJECT_VARIABLE: &str = "MARLEY_PROJECT";

/// The key under which a restore hands the builder a terminal's saved id (#575).
///
/// The builder takes the value and leaves the key empty before anything else reads the
/// environment, so no program sees an id under it.
pub const RESTORED_ID_VARIABLE: &str = "MARLEY_RESTORED_TERMINAL_ID";

/// A new terminal id: a version 4 UUID.
#[must_use]
pub fn new_terminal_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Whether `id` has a terminal id's shape, so a malformed header names no terminal.
#[must_use]
pub const fn is_terminal_id(id: &str) -> bool {
    uuid::Uuid::try_parse(id).is_ok()
}

/// The variable naming the program an agent in the terminal reports its state through (#652).
pub const BIN_VARIABLE: &str = "MARLEY_BIN";

/// The program terminals started from now on name as [`BIN_VARIABLE`], once Marley wrote it.
static AGENT_PROGRAM: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

/// Sets the program each local interactive terminal started from now on names as
/// [`BIN_VARIABLE`].
pub fn set_agent_program(program: Option<std::path::PathBuf>) {
    *AGENT_PROGRAM
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = program;
}

/// The program a new local interactive terminal names, if Marley wrote one.
#[must_use]
pub fn agent_program() -> Option<std::path::PathBuf> {
    AGENT_PROGRAM
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// The variables a terminal gives its agents (#652).
///
/// [`BIN_VARIABLE`] names `program` in a terminal that `named` itself (a local interactive one,
/// as [`TERMINAL_ID_VARIABLE`] is set), and is empty in any other. Emptied, not removed: the
/// program inherits Marley's own environment besides the map, and a Marley started from a Marley
/// terminal carries its parent's.
pub fn agent_environment(
    env: &mut impl Extend<(String, String)>,
    program: Option<&std::path::Path>,
    named: bool,
) {
    let value = program
        .filter(|_| named)
        .map(|program| program.to_string_lossy().into_owned())
        .unwrap_or_default();
    env.extend([(BIN_VARIABLE.to_string(), value)]);
    if named {
        let inherited = std::env::var(PLUGIN_DIRS_VARIABLE).unwrap_or_default();
        let plugin_dirs = plugin_dirs(&inherited, shared_plugin().as_deref());
        if plugin_dirs != inherited {
            env.extend([(PLUGIN_DIRS_VARIABLE.to_string(), plugin_dirs)]);
        }
    }
}

/// The variable Claude Code reads plugin folders from, `:`-separated absolute paths (#709).
pub const PLUGIN_DIRS_VARIABLE: &str = "CLAUDE_CODE_PLUGIN_DIRS";

/// The shared plugin's folder that terminals started from now on load, while Marley wants it.
static SHARED_PLUGIN: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

/// Sets the shared plugin's folder local interactive terminals started from now on load first,
/// or `None` for none (#709).
pub fn set_shared_plugin(folder: Option<std::path::PathBuf>) {
    *SHARED_PLUGIN
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = folder;
}

/// The shared plugin's folder new local interactive terminals load, if Marley wants one.
#[must_use]
pub fn shared_plugin() -> Option<std::path::PathBuf> {
    SHARED_PLUGIN
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// `inherited`'s folders with `plugin` first and any other copy of it (a folder beside it, under
/// the same parent) left out, so a Marley started from a Marley terminal doesn't load two.
fn plugin_dirs(inherited: &str, plugin: Option<&std::path::Path>) -> String {
    let parent = plugin.and_then(std::path::Path::parent);
    let kept = inherited.split(':').filter(|folder| {
        !folder.is_empty()
            && parent.is_none_or(|parent| !std::path::Path::new(folder).starts_with(parent))
    });
    plugin
        .map(|plugin| plugin.to_string_lossy().into_owned())
        .into_iter()
        .chain(kept.map(str::to_string))
        .collect::<Vec<_>>()
        .join(":")
}
