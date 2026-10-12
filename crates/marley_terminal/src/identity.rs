//! A terminal's identity, as its programs see it (#520).
//!
//! Each local interactive terminal starts with its own [`TERMINAL_ID_VARIABLE`], a UUID Zed's
//! terminal builder mints, and [`PROJECT_VARIABLE`], its project's folder. The Claude Code plugin's
//! MCP bridge sends both on each request, so Marley's tools know which terminal called them. The id
//! scopes what a tool does by default, and since #704 keys the editors area's approvals too, so it
//! is proven: beside it each terminal holds [`TOKEN_VARIABLE`], a token minted for its id and kept
//! for the run, which the bridge sends with the id; a call whose token does not match is taken as
//! one from no terminal (#755). A terminal Zed restores at a launch keeps the id it had (#575): the
//! restore hands the saved id to the builder under [`RESTORED_ID_VARIABLE`]. Beside the id, a local
//! interactive terminal names Marley's `marley-agent` program as [`BIN_VARIABLE`], through which an
//! agent reports its state to Marley (#652), and, while it is wanted, the Claude Code plugin Marley
//! shares with rustal-harness first in [`PLUGIN_DIRS_VARIABLE`] (#709), then the `rustal-ste`
//! skill's plugin folder (#725), and, for a Marley on a data directory of its own, that directory
//! in [`DATA_DIR_VARIABLE`], so `marley <command>` typed there reaches that Marley (#749).

/// The variable holding a terminal's id.
pub const TERMINAL_ID_VARIABLE: &str = "MARLEY_TERMINAL_ID";

/// The variable holding the folder of the terminal's project.
pub const PROJECT_VARIABLE: &str = "MARLEY_PROJECT";

/// The key under which a restore hands the builder a terminal's saved id (#575).
///
/// The builder takes the value and leaves the key empty before anything else reads the
/// environment, so no program sees an id under it.
pub const RESTORED_ID_VARIABLE: &str = "MARLEY_RESTORED_TERMINAL_ID";

/// The variable holding a terminal's token, which proves its id to Marley's MCP server (#755).
pub const TOKEN_VARIABLE: &str = "MARLEY_TERMINAL_TOKEN";

/// The token of each terminal id this run, minted on first ask.
static TOKENS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(Default::default);

/// The token of the terminal `id`, minted the first time it is asked for: a version 4 UUID's 122
/// random bits. A restored terminal keeps its id and gets a new token each run.
#[must_use]
pub fn terminal_token(id: &str) -> String {
    let mut tokens = TOKENS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    tokens
        .entry(id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().simple().to_string())
        .clone()
}

/// Whether `token` is the terminal `id`'s, compared in constant time; false for an id this run
/// minted no token for.
#[must_use]
pub fn token_matches(id: &str, token: &str) -> bool {
    let expected = TOKENS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(id)
        .cloned();
    let Some(expected) = expected else {
        return false;
    };
    let (expected, token) = (expected.as_bytes(), token.as_bytes());
    if expected.len() != token.len() {
        return false;
    }
    expected
        .iter()
        .zip(token)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

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

/// The variable naming the data directory of the Marley a terminal belongs to, while that Marley
/// runs on one of its own (`--user-data-dir`), so `marley <command>` typed there reaches it (#749).
pub const DATA_DIR_VARIABLE: &str = "MARLEY_USER_DATA_DIR";

/// The data directory terminals name as [`DATA_DIR_VARIABLE`]: Marley's own, when it was given one.
static DATA_DIR: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

/// Notes the data directory Marley runs on, when its launch named one (#749).
pub fn set_data_dir(directory: Option<std::path::PathBuf>) {
    *DATA_DIR
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = directory;
}

/// The variable naming the tools whose permission call an agent's plugin hands to Marley through
/// [`BIN_VARIABLE`]'s `approve` (#726), comma-separated: `ExitPlanMode` while plan review is on.
pub const APPROVE_TOOLS_VARIABLE: &str = "MARLEY_APPROVE_TOOLS";

/// The tools terminals started from now on name as [`APPROVE_TOOLS_VARIABLE`].
static APPROVE_TOOLS: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Sets the tools each local interactive terminal started from now on names as
/// [`APPROVE_TOOLS_VARIABLE`]; none empties it (#726).
pub fn set_approve_tools(tools: Option<String>) {
    *APPROVE_TOOLS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = tools;
}

/// The variables a terminal gives its agents (#652).
///
/// [`BIN_VARIABLE`] names `program` in a terminal that `named` itself (a local interactive one,
/// as [`TERMINAL_ID_VARIABLE`] is set), and is empty in any other; [`DATA_DIR_VARIABLE`] the same
/// for Marley's own data directory, and [`APPROVE_TOOLS_VARIABLE`] for the tools handed to Marley
/// (#726). Emptied, not removed: the program inherits Marley's own
/// environment besides the map, and a Marley started from a Marley terminal carries its parent's.
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
    let data_dir = DATA_DIR
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|_| named)
        .map(|directory| directory.to_string_lossy().into_owned())
        .unwrap_or_default();
    env.extend([(DATA_DIR_VARIABLE.to_string(), data_dir)]);
    let approve_tools = APPROVE_TOOLS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|_| named && program.is_some())
        .cloned()
        .unwrap_or_default();
    env.extend([(APPROVE_TOOLS_VARIABLE.to_string(), approve_tools)]);
    if named {
        let inherited = std::env::var(PLUGIN_DIRS_VARIABLE).unwrap_or_default();
        let plugin_dirs = plugin_dirs(&inherited, &marley_plugins());
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

/// The `rustal-ste` skill's plugin folder that terminals started from now on load (#725).
static SKILL_PLUGIN: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

/// Sets the `rustal-ste` skill's plugin folder that local interactive terminals started from now on
/// load, after the shared plugin, or `None` for none (#725).
pub fn set_skill_plugin(folder: Option<std::path::PathBuf>) {
    *SKILL_PLUGIN
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = folder;
}

/// The `rustal-ste` skill's plugin folder new local interactive terminals load, if Marley wants one.
#[must_use]
pub fn skill_plugin() -> Option<std::path::PathBuf> {
    SKILL_PLUGIN
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// The plugin folders Marley wants its terminals to load: the shared plugin, which carries the
/// `rustal-ste` skill itself (#724), else the skill's own folder, so a Claude Code lists the skill
/// once.
fn marley_plugins() -> Vec<std::path::PathBuf> {
    shared_plugin().or_else(skill_plugin).into_iter().collect()
}

/// `inherited`'s folders with `plugins` first and any other copy of one (a folder beside it, under
/// the same parent) left out, so a Marley started from a Marley terminal doesn't load two.
fn plugin_dirs(inherited: &str, plugins: &[std::path::PathBuf]) -> String {
    let parents: Vec<&std::path::Path> = plugins
        .iter()
        .filter_map(|plugin| plugin.parent())
        .collect();
    let kept = inherited.split(':').filter(|folder| {
        !folder.is_empty()
            && !parents
                .iter()
                .any(|parent| std::path::Path::new(folder).starts_with(parent))
    });
    plugins
        .iter()
        .map(|plugin| plugin.to_string_lossy().into_owned())
        .chain(kept.map(str::to_string))
        .collect::<Vec<_>>()
        .join(":")
}

/// `inherited`'s folders with Marley's plugin folders first, for a process Marley starts outside a
/// terminal, such as the Agent Panel's Marley entry on Claude Code (#725).
#[must_use]
pub fn plugin_dirs_for(inherited: &str) -> String {
    plugin_dirs(inherited, &marley_plugins())
}
