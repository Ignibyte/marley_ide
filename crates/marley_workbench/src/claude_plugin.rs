//! Marley's plugin for Claude Code (T7b), which makes Claude Code send #478's notifications.
//!
//! Claude Code's own notification channel does not recognize Marley's terminal, so the agent
//! bar offers to install a small plugin. Its `Notification` and `Stop` hooks answer with a
//! `terminalSequence`, an OSC 777 notify that Claude Code writes to its terminal, and stay
//! silent in terminals other than Marley's. [`install`] writes the plugin as a local
//! marketplace under Marley's data directory and installs it with `claude plugin`, and
//! [`update`] brings an older install up to the version Marley ships (#547).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use gpui::{App, AppContext as _, AsyncApp, Global, WeakEntity};
use util::ResultExt as _;
use workspace::Toast;
use workspace::Workspace;
use workspace::notifications::NotificationId;

/// The marketplace's name, and the plugin's.
pub const MARKETPLACE: &str = "marley";

/// The plugin's id as Claude Code lists it.
pub const PLUGIN: &str = "marley@marley";

/// Marley's MCP bridge (#491): Python 3, the standard library only. It reads Marley's endpoint
/// file and passes each JSON-RPC message of a stdio MCP client to Marley's server; the plugin
/// ships it, and Zed's own agents run it as the context server `marley` (#501).
pub(crate) const BRIDGE: &str = include_str!("../claude_plugin/marley/bin/marley-mcp-bridge");

/// The plugin's manifest, whose `version` is the one Marley ships.
const MANIFEST: &str = include_str!("../claude_plugin/marley/.claude-plugin/plugin.json");

/// The plugin's files, by their path in the marketplace, and whether each is a program.
const FILES: [(&str, &str, bool); 6] = [
    (
        ".claude-plugin/marketplace.json",
        include_str!("../claude_plugin/.claude-plugin/marketplace.json"),
        false,
    ),
    ("marley/.claude-plugin/plugin.json", MANIFEST, false),
    // Marley's MCP server, through the bridge (#491).
    (
        "marley/.mcp.json",
        include_str!("../claude_plugin/marley/.mcp.json"),
        false,
    ),
    ("marley/bin/marley-mcp-bridge", BRIDGE, true),
    (
        "marley/hooks/hooks.json",
        include_str!("../claude_plugin/marley/hooks/hooks.json"),
        false,
    ),
    // Each hook event, summarized for the rail (#519).
    (
        "marley/hooks/event.py",
        include_str!("../claude_plugin/marley/hooks/event.py"),
        true,
    ),
];

/// Where the plugin goes, where Claude Code keeps its state, and what Marley knows of it.
#[derive(Debug, Clone)]
pub struct ClaudePlugin {
    /// Where Marley writes the plugin and its marketplace.
    pub marketplace_dir: PathBuf,
    /// Claude Code's configuration directory.
    pub config_dir: PathBuf,
    /// The `claude` program: the one given, or the PATH's, found when Marley starts.
    pub claude: Option<PathBuf>,
    /// Whether the plugin is installed, `None` until Claude Code's list has been read.
    pub installed: Option<bool>,
    /// The installed plugin's version, as Claude Code's list gives it.
    pub installed_version: Option<String>,
    /// Whether an install is running.
    pub installing: bool,
    /// Whether an update is running (#547).
    pub updating: bool,
}

impl Global for ClaudePlugin {}

impl ClaudePlugin {
    /// Whether Claude Code has an older version of the plugin than Marley ships. Claude Code runs
    /// a plugin from a copy per version, so the new hooks reach it only through an update. A
    /// version that does not parse, or a newer one, needs none: two Marley builds of different
    /// ages must not keep offering each other's plugin.
    #[must_use]
    pub fn needs_update(&self) -> bool {
        let installed = self
            .installed_version
            .as_deref()
            .and_then(|version| semver::Version::parse(version).ok());
        self.installed == Some(true)
            && matches!(
                (installed, shipped_version()),
                (Some(installed), Some(shipped)) if installed < shipped
            )
    }
}

/// The plugin's version as Marley ships it, read from its manifest.
#[must_use]
pub fn shipped_version() -> Option<semver::Version> {
    let manifest: serde_json::Value = serde_json::from_str(MANIFEST).ok()?;
    semver::Version::parse(manifest.get("version")?.as_str()?).ok()
}

/// Sets up the plugin's global; [`crate::init`] calls it once.
///
/// It uses Marley's data directory and Claude Code's own, `CLAUDE_CONFIG_DIR` or `~/.claude`, and
/// runs the `claude` that `MARLEY_CLAUDE` names, else the PATH's (#547). Marley's PATH can come
/// from the login shell, so a scenario's stand-in names itself there, as `MARLEY_CHROMIUM` does
/// for Chromium.
pub fn init(cx: &mut App) {
    let config_dir = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map_or_else(|| util::paths::home_dir().join(".claude"), PathBuf::from);
    set_up(
        ClaudePlugin {
            marketplace_dir: paths::data_dir().join("claude-code"),
            config_dir,
            claude: std::env::var_os("MARLEY_CLAUDE").map(PathBuf::from),
            installed: None,
            installed_version: None,
            installing: false,
            updating: false,
        },
        cx,
    );
}

/// Uses `plugin`, then reads off the main thread whether the plugin is installed and at which
/// version, and finds `claude` on the PATH unless `plugin` names one.
pub fn set_up(plugin: ClaudePlugin, cx: &mut App) {
    let config_dir = plugin.config_dir.clone();
    let find_claude = plugin.claude.is_none();
    cx.set_global(plugin);
    cx.spawn(async move |cx| {
        let (installed, installed_version, claude) = cx
            .background_spawn(futures::future::lazy(move |_| {
                let claude = find_claude.then(|| which::which("claude").ok()).flatten();
                (
                    installed_in(&config_dir),
                    installed_version_in(&config_dir),
                    claude,
                )
            }))
            .await;
        cx.update(|cx| {
            let state = cx.global_mut::<ClaudePlugin>();
            state.installed = Some(installed);
            state.installed_version = installed_version;
            state.claude = state.claude.take().or(claude);
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Writes the plugin and its marketplace into `dir`, the programs executable.
///
/// # Errors
///
/// Creating a directory, or writing a file or its permissions.
pub fn write_plugin_in(dir: &Path) -> std::io::Result<()> {
    for (path, content, program) in FILES {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
        std::fs::write(&path, content)?;
        #[cfg(unix)]
        if program {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

/// Whether Claude Code's list in `config_dir` has Marley's plugin installed.
#[must_use]
pub fn installed_in(config_dir: &Path) -> bool {
    listed_in(&config_dir.join("plugins/installed_plugins.json"), |list| {
        list.get("plugins")?.get(PLUGIN)
    })
}

/// The version of Marley's plugin that Claude Code's list in `config_dir` has installed: its
/// user-scope entry's, else its first entry's.
#[must_use]
pub fn installed_version_in(config_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(config_dir.join("plugins/installed_plugins.json")).ok()?;
    let list: serde_json::Value = serde_json::from_str(&text).ok()?;
    let entries = list.get("plugins")?.get(PLUGIN)?.as_array()?;
    let entry = entries
        .iter()
        .find(|entry| entry.get("scope").and_then(serde_json::Value::as_str) == Some("user"))
        .or_else(|| entries.first())?;
    Some(entry.get("version")?.as_str()?.to_string())
}

/// Whether Claude Code in `config_dir` already knows Marley's marketplace.
#[must_use]
pub fn marketplace_known_in(config_dir: &Path) -> bool {
    listed_in(
        &config_dir.join("plugins/known_marketplaces.json"),
        |list| list.get(MARKETPLACE),
    )
}

/// Whether the JSON at `path` holds what `find` looks for; a missing or unreadable file holds
/// nothing.
fn listed_in(
    path: &Path,
    find: impl FnOnce(&serde_json::Value) -> Option<&serde_json::Value>,
) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .is_some_and(|list| find(&list).is_some())
}

/// Installs the plugin: writes it, adds the marketplace Claude Code does not know yet, and
/// installs it. The outcome shows in `workspace`.
pub fn install(plugin: ClaudePlugin, workspace: WeakEntity<Workspace>, cx: &mut App) {
    cx.global_mut::<ClaudePlugin>().installing = true;
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = run_install(plugin, cx).await;
        cx.update(|cx| {
            let state = cx.global_mut::<ClaudePlugin>();
            state.installing = false;
            if result.is_ok() {
                state.installed = Some(true);
            }
            cx.refresh_windows();
            workspace
                .update(cx, |workspace, cx| match result {
                    Ok(()) => workspace.show_toast(
                        Toast::new(
                            NotificationId::unique::<ClaudePlugin>(),
                            "Marley's plugin for Claude Code is installed: notifications, and \
                             Marley's tools for its terminals and Browser tabs. New Claude Code \
                             sessions use it; in a running one, run /reload-plugins.",
                        ),
                        cx,
                    ),
                    Err(error) => workspace.show_error(error, cx),
                })
                .log_err();
        });
    })
    .detach();
}

/// Writes the plugin and its marketplace off the main thread, and gives the `claude` to run, the
/// marketplace's directory, and whether Claude Code already knows the marketplace.
async fn write_marketplace(
    plugin: ClaudePlugin,
    cx: &AsyncApp,
) -> anyhow::Result<(PathBuf, PathBuf, bool)> {
    let ClaudePlugin {
        marketplace_dir,
        config_dir,
        claude,
        ..
    } = plugin;
    let dir = marketplace_dir.clone();
    let (claude, known) = cx
        .background_spawn(futures::future::lazy(move |_| {
            write_plugin_in(&dir).context("writing Marley's plugin for Claude Code")?;
            let claude = claude.context("`claude` is not on the PATH")?;
            anyhow::Ok((claude, marketplace_known_in(&config_dir)))
        }))
        .await?;
    Ok((claude, marketplace_dir, known))
}

async fn run_install(plugin: ClaudePlugin, cx: &AsyncApp) -> anyhow::Result<()> {
    let (claude, marketplace_dir, known) = write_marketplace(plugin, cx).await?;
    if !known {
        crate::run_program(
            &claude,
            &[
                OsStr::new("plugin"),
                OsStr::new("marketplace"),
                OsStr::new("add"),
                marketplace_dir.as_os_str(),
            ],
        )
        .await?;
    }
    crate::run_program(
        &claude,
        &[
            OsStr::new("plugin"),
            OsStr::new("install"),
            OsStr::new(PLUGIN),
        ],
    )
    .await
}

/// Updates the plugin to the version Marley ships, showing the outcome in `workspace`.
///
/// It writes the marketplace again, has Claude Code refresh it (or add it, when it no longer
/// knows it), and updates the plugin.
pub fn update(plugin: ClaudePlugin, workspace: WeakEntity<Workspace>, cx: &mut App) {
    cx.global_mut::<ClaudePlugin>().updating = true;
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = run_update(plugin, cx).await;
        cx.update(|cx| {
            let state = cx.global_mut::<ClaudePlugin>();
            state.updating = false;
            if result.is_ok() {
                state.installed_version = shipped_version().map(|version| version.to_string());
            }
            cx.refresh_windows();
            workspace
                .update(cx, |workspace, cx| match result {
                    Ok(()) => workspace.show_toast(
                        Toast::new(
                            NotificationId::unique::<ClaudePlugin>(),
                            "Marley's plugin for Claude Code is updated, and now tells the rail \
                             what Claude Code is doing. New Claude Code sessions use it; restart \
                             a running one to pick it up.",
                        ),
                        cx,
                    ),
                    Err(error) => workspace.show_error(error, cx),
                })
                .log_err();
        });
    })
    .detach();
}

async fn run_update(plugin: ClaudePlugin, cx: &AsyncApp) -> anyhow::Result<()> {
    let (claude, marketplace_dir, known) = write_marketplace(plugin, cx).await?;
    let marketplace = if known {
        [
            OsStr::new("plugin"),
            OsStr::new("marketplace"),
            OsStr::new("update"),
            OsStr::new(MARKETPLACE),
        ]
    } else {
        [
            OsStr::new("plugin"),
            OsStr::new("marketplace"),
            OsStr::new("add"),
            marketplace_dir.as_os_str(),
        ]
    };
    crate::run_program(&claude, &marketplace).await?;
    crate::run_program(
        &claude,
        &[
            OsStr::new("plugin"),
            OsStr::new("update"),
            OsStr::new(PLUGIN),
        ],
    )
    .await
}

#[cfg(test)]
#[path = "claude_plugin_tests.rs"]
mod tests;
