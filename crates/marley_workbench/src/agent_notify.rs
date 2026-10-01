//! Desktop notifications for Codex and `OpenCode` in one click (#552), as the agent bar connects
//! Claude Code.
//!
//! Codex's TUI notifies through its terminal once its `config.toml` asks for it: `notifications`,
//! `notification_condition = "always"` (Marley decides focus itself, #478) and
//! `notification_method = "osc9"` under `[tui]`. `OpenCode` takes a plugin file, which Marley writes
//! whole and versions in its first line. While either agent runs in a terminal and is not set up,
//! its bar offers the setup; the files are read off the main thread, again when a bar for the
//! agent draws after a while, and after each write. Codex's home is `CODEX_HOME` or `~/.codex`,
//! `OpenCode`'s plugins `XDG_CONFIG_HOME` or `~/.config`, then `opencode/plugins`, so a scenario
//! points both at its own folders.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use gpui::{AnyElement, App, Global, WeakEntity};
use marley_agent::AgentKind;
use terminal_view::MarleyFooterContext;
use toml_edit::{DocumentMut, Item, Table, value};
use ui::{Button, Icon, IconName, IconSize, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::Toast;
use workspace::Workspace;
use workspace::notifications::NotificationId;

/// The plugin Marley ships for `OpenCode`.
const OPENCODE_PLUGIN: &str = include_str!("../agent_plugins/opencode/marley.js");

/// The first line's words before the version, in the plugin Marley ships.
const PLUGIN_MARK: &str = "// marley-opencode-plugin ";

/// How long a read of the files stands before a bar that draws reads them again.
const FRESH: Duration = Duration::from_secs(2);

/// What Marley knows of the two agents' setups.
#[derive(Default)]
struct AgentNotify {
    /// Whether Codex's config asks for notifications; `None` until read.
    codex: Option<bool>,
    /// Marley's plugin for `OpenCode`; `None` until read.
    opencode: Option<Plugin>,
    /// The agent whose setup is being written.
    writing: Option<AgentKind>,
    reading: bool,
    read_at: Option<Instant>,
}

impl Global for AgentNotify {}

/// The state of Marley's plugin file for `OpenCode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plugin {
    Missing,
    Older,
    Current,
}

/// Reads both setups once; [`crate::init`] calls it.
pub fn init(cx: &mut App) {
    cx.set_global(AgentNotify::default());
    read(cx);
}

/// Where Codex keeps its config.
fn codex_config() -> PathBuf {
    let home = std::env::var_os("CODEX_HOME")
        .map_or_else(|| util::paths::home_dir().join(".codex"), PathBuf::from);
    home.join("config.toml")
}

/// Where `OpenCode` loads Marley's plugin from.
fn opencode_plugin() -> PathBuf {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map_or_else(|| util::paths::home_dir().join(".config"), PathBuf::from);
    config.join("opencode/plugins/marley.js")
}

/// Whether Codex's config `text` asks its TUI for notifications Marley shows: the three keys
/// under `[tui]`, a list of events for `notifications` counting as on.
fn codex_configured(text: &str) -> bool {
    let Ok(document) = text.parse::<DocumentMut>() else {
        return false;
    };
    let Some(tui) = document.get("tui").and_then(Item::as_table_like) else {
        return false;
    };
    let notifications = tui.get("notifications").is_some_and(|item| {
        item.as_bool() == Some(true) || item.as_array().is_some_and(|events| !events.is_empty())
    });
    let string = |key: &str| tui.get(key).and_then(Item::as_str);
    notifications
        && string("notification_condition") == Some("always")
        && string("notification_method") == Some("osc9")
}

/// Codex's config `text` with the three keys set under `[tui]`, every other key and comment kept.
/// A list of events for `notifications` stays as it is.
fn configure_codex(text: &str) -> anyhow::Result<String> {
    let mut document = text
        .parse::<DocumentMut>()
        .context("Codex's config.toml does not parse as TOML")?;
    if !document.contains_key("tui") {
        document.insert("tui", Item::Table(Table::new()));
    }
    let tui = document
        .get_mut("tui")
        .and_then(Item::as_table_like_mut)
        .context("`tui` in Codex's config.toml is not a table")?;
    let listed = tui
        .get("notifications")
        .is_some_and(|item| item.as_array().is_some());
    if !listed {
        tui.insert("notifications", value(true));
    }
    tui.insert("notification_condition", value("always"));
    tui.insert("notification_method", value("osc9"));
    Ok(document.to_string())
}

/// The version in a plugin file's first line.
fn plugin_version(text: &str) -> Option<u32> {
    text.lines()
        .next()?
        .strip_prefix(PLUGIN_MARK)?
        .trim()
        .parse()
        .ok()
}

/// The state of the plugin file whose text is `text`, against the one Marley ships; a file with
/// no version line is an older one, which an update rewrites.
fn plugin_state(text: Option<&str>) -> Plugin {
    let Some(text) = text else {
        return Plugin::Missing;
    };
    match (plugin_version(text), plugin_version(OPENCODE_PLUGIN)) {
        (Some(installed), Some(shipped)) if installed >= shipped => Plugin::Current,
        _ => Plugin::Older,
    }
}

/// Reads both files off the main thread, unless a read runs.
fn read(cx: &mut App) {
    let notify = cx.global_mut::<AgentNotify>();
    if notify.reading {
        return;
    }
    notify.reading = true;
    cx.spawn(async move |cx| {
        let (codex, opencode) = cx
            .background_spawn(futures::future::lazy(|_| {
                // A missing or unreadable config asks for nothing.
                let codex = std::fs::read_to_string(codex_config())
                    .is_ok_and(|text| codex_configured(&text));
                let plugin = std::fs::read_to_string(opencode_plugin()).ok();
                (codex, plugin_state(plugin.as_deref()))
            }))
            .await;
        cx.update(|cx| {
            let notify = cx.global_mut::<AgentNotify>();
            let changed = notify.codex != Some(codex) || notify.opencode != Some(opencode);
            notify.codex = Some(codex);
            notify.opencode = Some(opencode);
            notify.reading = false;
            notify.read_at = Some(Instant::now());
            // Only a change redraws, so a bar that draws does not read and redraw forever.
            if changed {
                cx.refresh_windows();
            }
        });
    })
    .detach();
}

/// Reads the files again when a bar for `kind` draws and the last read is stale, so a file the
/// user or another program changed is seen.
fn read_if_stale(cx: &mut App) {
    let stale = cx
        .try_global::<AgentNotify>()
        .is_some_and(|notify| notify.read_at.is_none_or(|at| at.elapsed() >= FRESH));
    if stale {
        read(cx);
    }
}

/// The chip under `kind`'s terminal while its notifications are not set up, a busy label while
/// Marley writes, and none once they are.
pub(crate) fn chip(
    kind: AgentKind,
    context: &MarleyFooterContext,
    cx: &mut App,
) -> Option<AnyElement> {
    if !matches!(kind, AgentKind::Codex | AgentKind::OpenCode) {
        return None;
    }
    read_if_stale(cx);
    let notify = cx.try_global::<AgentNotify>()?;
    if notify.writing == Some(kind) {
        let busy = match kind {
            AgentKind::Codex => "Turning on Codex notifications…",
            _ => "Writing Marley's plugin for OpenCode…",
        };
        return Some(
            Label::new(busy)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .into_any_element(),
        );
    }
    let (id, label, tooltip) = match kind {
        AgentKind::Codex if notify.codex == Some(false) => (
            "marley-codex-notifications",
            "Turn on Codex notifications",
            "Sets notifications, notification_condition = \"always\" and notification_method = \
             \"osc9\" under [tui] in Codex's config.toml, so Codex tells Marley when a turn ends \
             or it needs you; the file's other settings stay as they are",
        ),
        AgentKind::OpenCode if notify.opencode == Some(Plugin::Missing) => (
            "marley-opencode-plugin",
            "Connect OpenCode to Marley",
            "Writes Marley's plugin to OpenCode's plugin folder, so OpenCode tells Marley when a \
             session ends, needs your permission or fails",
        ),
        AgentKind::OpenCode if notify.opencode == Some(Plugin::Older) => (
            "marley-opencode-plugin-update",
            "Update Marley's plugin for OpenCode",
            "Rewrites Marley's plugin in OpenCode's plugin folder with the one this Marley ships",
        ),
        _ => return None,
    };
    let workspace = context.workspace.clone();
    Some(
        div()
            .debug_selector(move || id.into())
            .child(
                Button::new(id, label)
                    .start_icon(Icon::new(IconName::Download).size(IconSize::XSmall))
                    .label_size(LabelSize::Small)
                    .tooltip(Tooltip::text(tooltip))
                    .on_click(move |_, _, cx| set_up(kind, workspace.clone(), cx)),
            )
            .into_any_element(),
    )
}

/// Writes `kind`'s setup off the main thread; the outcome shows in `workspace`.
fn set_up(kind: AgentKind, workspace: WeakEntity<Workspace>, cx: &mut App) {
    cx.global_mut::<AgentNotify>().writing = Some(kind);
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = cx
            .background_spawn(futures::future::lazy(move |_| match kind {
                AgentKind::Codex => write_codex(),
                _ => write_plugin(),
            }))
            .await;
        cx.update(|cx| {
            cx.global_mut::<AgentNotify>().writing = None;
            read(cx);
            workspace
                .update(cx, |workspace, cx| match result {
                    Ok(()) => {
                        let message = match kind {
                            AgentKind::Codex => {
                                "Codex notifications are on. Restart a running Codex to pick \
                                 them up."
                            }
                            _ => {
                                "Marley's plugin for OpenCode is in place. Restart a running \
                                 OpenCode to load it."
                            }
                        };
                        workspace.show_toast(
                            Toast::new(NotificationId::unique::<AgentNotify>(), message),
                            cx,
                        );
                    }
                    Err(error) => workspace.show_error(error, cx),
                })
                .log_err();
        });
    })
    .detach();
}

/// Sets Codex's three keys, making the file when it is missing.
fn write_codex() -> anyhow::Result<()> {
    write_codex_in(&codex_config())
}

fn write_codex_in(path: &Path) -> anyhow::Result<()> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error).context(format!("reading {}", path.display())),
    };
    let configured = configure_codex(&text)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(format!("making {}", parent.display()))?;
    }
    std::fs::write(path, configured).context(format!("writing {}", path.display()))
}

/// Writes the plugin `OpenCode` loads, whole.
fn write_plugin() -> anyhow::Result<()> {
    write_plugin_in(&opencode_plugin())
}

fn write_plugin_in(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(format!("making {}", parent.display()))?;
    }
    std::fs::write(path, OPENCODE_PLUGIN).context(format!("writing {}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))
        .context(format!("setting {}'s permissions", path.display()))
}
