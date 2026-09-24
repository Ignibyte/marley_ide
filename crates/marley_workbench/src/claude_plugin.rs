//! Marley's plugin for Claude Code (T7b), which makes Claude Code send #478's notifications.
//!
//! Claude Code's own notification channel does not recognize Marley's terminal, so the agent
//! bar offers to install a small plugin. Its `Notification` and `Stop` hooks answer with a
//! `terminalSequence`, an OSC 777 notify that Claude Code writes to its terminal, and stay
//! silent in terminals other than Marley's. [`install`] writes the plugin as a local
//! marketplace under Marley's data directory and installs it with `claude plugin`.

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

/// The plugin's files, by their path in the marketplace, and whether each is a program.
const FILES: [(&str, &str, bool); 4] = [
    (
        ".claude-plugin/marketplace.json",
        include_str!("../claude_plugin/.claude-plugin/marketplace.json"),
        false,
    ),
    (
        "marley/.claude-plugin/plugin.json",
        include_str!("../claude_plugin/marley/.claude-plugin/plugin.json"),
        false,
    ),
    (
        "marley/hooks/hooks.json",
        include_str!("../claude_plugin/marley/hooks/hooks.json"),
        false,
    ),
    (
        "marley/hooks/notify.sh",
        include_str!("../claude_plugin/marley/hooks/notify.sh"),
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
    /// Whether an install is running.
    pub installing: bool,
}

impl Global for ClaudePlugin {}

/// Uses Marley's data directory and Claude Code's own, `CLAUDE_CONFIG_DIR` or `~/.claude`.
/// [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    let config_dir = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map_or_else(|| util::paths::home_dir().join(".claude"), PathBuf::from);
    set_up(
        ClaudePlugin {
            marketplace_dir: paths::data_dir().join("claude-code"),
            config_dir,
            claude: None,
            installed: None,
            installing: false,
        },
        cx,
    );
}

/// Uses `plugin`, then reads off the main thread whether the plugin is installed, and finds
/// `claude` on the PATH unless `plugin` names one.
pub fn set_up(plugin: ClaudePlugin, cx: &mut App) {
    let config_dir = plugin.config_dir.clone();
    let find_claude = plugin.claude.is_none();
    cx.set_global(plugin);
    cx.spawn(async move |cx| {
        let (installed, claude) = cx
            .background_spawn(futures::future::lazy(move |_| {
                let claude = find_claude.then(|| which::which("claude").ok()).flatten();
                (installed_in(&config_dir), claude)
            }))
            .await;
        cx.update(|cx| {
            let state = cx.global_mut::<ClaudePlugin>();
            state.installed = Some(installed);
            state.claude = state.claude.take().or(claude);
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Writes the plugin and its marketplace into `dir`, the script executable.
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
                            "Marley's plugin for Claude Code is installed. New Claude Code \
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

async fn run_install(plugin: ClaudePlugin, cx: &AsyncApp) -> anyhow::Result<()> {
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
    if !known {
        run(
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
    run(
        &claude,
        &[
            OsStr::new("plugin"),
            OsStr::new("install"),
            OsStr::new(PLUGIN),
        ],
    )
    .await
}

/// Runs `claude` with `args`, an error carrying what it printed when it fails.
async fn run(claude: &Path, args: &[&OsStr]) -> anyhow::Result<()> {
    let output = util::command::new_command(claude)
        .args(args)
        .output()
        .await
        .context("running `claude`")?;
    anyhow::ensure!(
        output.status.success(),
        "`claude {}` failed: {}",
        args.iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

#[cfg(test)]
#[path = "claude_plugin_tests.rs"]
mod tests;
