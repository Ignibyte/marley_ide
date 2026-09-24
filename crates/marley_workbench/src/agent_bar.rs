//! The agent bar (T7a), under a terminal running a CLI agent.
//!
//! The agent is at its left, and at its right the folder the agent works in and that folder's
//! git branch, where Warp puts them. Zed's `TerminalView` draws whatever [`MarleyTerminalFooter`] renders below its grid, and
//! takes the rows it needs from the grid; [`init`] sets it to this bar.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{AnyElement, App, SharedString, Window};
use marley_agent::AgentKind;
use terminal_view::{MarleyFooterContext, MarleyTerminalFooter};
use ui::{Button, Icon, IconName, IconSize, Label, LabelSize, prelude::*};
use util::paths::PathExt as _;

use crate::agents::cli_icon;
use crate::claude_plugin::{self, ClaudePlugin};

/// Puts the agent bar under every terminal. [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyTerminalFooter(Arc::new(render)));
}

/// What the bar shows for one terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarContents {
    /// The agent in the terminal's foreground.
    pub agent: AgentKind,
    /// The folder the agent works in, when the terminal can tell.
    pub folder: Option<PathBuf>,
    /// The branch of the project's repository that holds the folder, when it is on one.
    pub branch: Option<SharedString>,
}

/// What the bar shows for the terminal in `context`, or `None` while no CLI agent is in its
/// foreground.
pub fn contents(context: &MarleyFooterContext, cx: &App) -> Option<BarContents> {
    let terminal = context.terminal.read(cx);
    let agent = terminal
        .foreground_process_command_name()
        .as_deref()
        .and_then(marley_agent::agent_kind_of)?;
    let folder = terminal.working_directory();
    let branch = folder.as_deref().and_then(|folder| {
        let project = context.project.upgrade()?;
        let repositories = project
            .read(cx)
            .git_store()
            .read(cx)
            .repositories()
            .values()
            .map(|repository| {
                let repository = repository.read(cx);
                let branch = repository
                    .branch
                    .as_ref()
                    .map(|branch| SharedString::from(branch.name().to_string()));
                (Arc::clone(&repository.work_directory_abs_path), branch)
            })
            .collect::<Vec<_>>();
        branch_for(
            folder,
            repositories
                .iter()
                .map(|(work_directory, branch)| (work_directory.as_ref(), branch.as_ref())),
        )
    });
    Some(BarContents {
        agent,
        folder,
        branch,
    })
}

/// The branch of the innermost repository whose work directory holds `folder`, if that
/// repository is on one.
fn branch_for<'a>(
    folder: &Path,
    repositories: impl IntoIterator<Item = (&'a Path, Option<&'a SharedString>)>,
) -> Option<SharedString> {
    repositories
        .into_iter()
        .filter(|(work_directory, _)| folder.starts_with(work_directory))
        .max_by_key(|(work_directory, _)| work_directory.components().count())
        .and_then(|(_, branch)| branch.cloned())
}

fn render(context: &MarleyFooterContext, _: &mut Window, cx: &mut App) -> Option<AnyElement> {
    let BarContents {
        agent,
        folder,
        branch,
    } = contents(context, cx)?;
    let colors = cx.theme().colors();
    let chip = |icon: IconName, text: SharedString| {
        h_flex()
            .min_w_0()
            .gap_1()
            .child(Icon::new(icon).size(IconSize::XSmall).color(Color::Muted))
            .child(
                Label::new(text)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .truncate(),
            )
    };
    Some(
        h_flex()
            .debug_selector(|| "marley-agent-bar".into())
            .flex_none()
            .w_full()
            .justify_between()
            .gap_4()
            .px_2()
            .py_1()
            .border_t_1()
            .border_color(colors.border_variant)
            .bg(colors.terminal_background)
            .child(
                h_flex()
                    .flex_none()
                    .gap_1p5()
                    .child(
                        Icon::new(cli_icon(agent))
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(agent.display_name())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .children(
                        (agent == AgentKind::Claude)
                            .then(|| claude_plugin_chip(context, cx))
                            .flatten(),
                    ),
            )
            .child(
                h_flex()
                    .min_w_0()
                    .gap_3()
                    .children(folder.map(|folder| {
                        let text = folder.compact().to_string_lossy().into_owned();
                        chip(IconName::Folder, text.into())
                            .debug_selector(|| "marley-agent-bar-folder".into())
                    }))
                    .children(branch.map(|branch| {
                        chip(IconName::GitBranch, branch)
                            .debug_selector(|| "marley-agent-bar-branch".into())
                    })),
            )
            .into_any_element(),
    )
}

/// The chip that installs Marley's plugin for Claude Code, while the plugin is known not to be
/// installed (#482).
fn claude_plugin_chip(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let plugin = cx.try_global::<ClaudePlugin>()?;
    if plugin.installed != Some(false) {
        return None;
    }
    if plugin.installing {
        return Some(
            Label::new("Installing Marley's plugin for Claude Code…")
                .size(LabelSize::Small)
                .color(Color::Muted)
                .into_any_element(),
        );
    }
    let (plugin, workspace) = (plugin.clone(), context.workspace.clone());
    Some(
        div()
            .debug_selector(|| "marley-claude-plugin-chip".into())
            .child(
                Button::new(
                    "marley-enable-claude-notifications",
                    "Enable Claude Code notifications",
                )
                .start_icon(Icon::new(IconName::Download).size(IconSize::XSmall))
                .label_size(LabelSize::Small)
                .on_click(move |_, _, cx| {
                    claude_plugin::install(plugin.clone(), workspace.clone(), cx);
                }),
            )
            .into_any_element(),
    )
}

#[cfg(test)]
#[path = "agent_bar_tests.rs"]
mod tests;
