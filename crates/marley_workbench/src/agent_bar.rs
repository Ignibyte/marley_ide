//! The agent bar (T7a), under a terminal running a CLI agent.
//!
//! The agent is at its left, and at its right the folder the agent works in and that folder's
//! git branch, where Warp puts them. Zed's `TerminalView` draws whatever [`MarleyTerminalFooter`] renders below its grid, and
//! takes the rows it needs from the grid; [`init`] sets it to this bar.
//!
//! Beside the agent, Attach File (T7c) types the paths of the files chosen in a file chooser into
//! the terminal, as dropping the files on it does, for the agent to read, and the microphone
//! (T7d) dictates through Voxtype while Voice is on (#642).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, PathPromptOptions, SharedString, WeakEntity, Window};
use marley_agent::AgentKind;
use project::{DirectoryLister, Project};
use settings::Settings as _;
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, MarleyTerminalFooter, TerminalView};
use ui::{Button, Icon, IconButton, IconName, IconSize, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use util::paths::PathExt as _;
use workspace::Workspace;

use crate::agents::cli_icon;
use crate::blocks::focused_terminal;
use crate::claude_plugin::{self, ClaudePlugin};
use crate::links;
use crate::rich_input;
use crate::voice::{self, Voice, VoiceState};
use crate::{AttachFile, Dictation, MarleySettings, RichInput};

/// Puts the agent bar under every terminal, and Attach File on every workspace for the focused
/// terminal. [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyTerminalFooter(Arc::new(render)));
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &AttachFile, window, cx| {
            if let Some(view) = focused_terminal(workspace, window, cx) {
                attach(view.downgrade(), workspace, window, cx);
            }
        });
    })
    .detach();
}

/// Asks for files, in the desktop's chooser or Zed's own path prompt as the settings and the
/// project say, and types the chosen paths into `view`'s terminal; a cancelled chooser types
/// nothing.
fn attach(
    view: WeakEntity<TerminalView>,
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    // The project's lister, so a remote project's chooser lists the machine its terminals run
    // on.
    let lister = DirectoryLister::Project(workspace.project().clone());
    let paths = workspace.prompt_for_open_path(
        PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(SharedString::new_static("Attach")),
        },
        lister,
        window,
        cx,
    );
    cx.spawn_in(window, async move |_, cx| {
        let Some(paths) = paths.await.log_err().flatten() else {
            return;
        };
        view.update_in(cx, |view, window, cx| {
            view.add_paths_to_terminal(&paths, window, cx);
        })
        .log_err();
    })
    .detach();
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
    let agent = agent_in(terminal)?;
    let folder = terminal.working_directory();
    let branch = folder.as_deref().and_then(|folder| {
        let project = context.project.upgrade()?;
        branch_of(&project, folder, cx)
    });
    Some(BarContents {
        agent,
        folder,
        branch,
    })
}

/// The branch of `project`'s innermost repository that holds `folder`, when it is on one (#477; the
/// block headers' too, #630).
pub(crate) fn branch_of(
    project: &Entity<Project>,
    folder: &Path,
    cx: &App,
) -> Option<SharedString> {
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
}

/// The CLI agent in `terminal`'s foreground, if one is.
pub(crate) fn agent_in(terminal: &Terminal) -> Option<AgentKind> {
    terminal
        .foreground_process_command_name()
        .as_deref()
        .and_then(marley_agent::agent_kind_of)
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

/// The footer of a terminal with no agent CLI in its foreground: the card or bar of an agent
/// typing into its program (#525), the shell's prompt editor while it is open (#624), and the
/// strip while it offers a URL (#503).
fn footer_without_agent(
    context: &MarleyFooterContext,
    offer: Option<AnyElement>,
    cx: &App,
) -> Option<AnyElement> {
    let drive = crate::terminal_drive::footer(context, cx);
    let prompt_editor = rich_input::element(context, cx);
    let colors = cx.theme().colors();
    let strip = offer.map(|offer| {
        h_flex()
            .debug_selector(|| "marley-served-url-strip".into())
            .flex_none()
            .w_full()
            .justify_end()
            .px_2()
            .py_1()
            .border_t_1()
            .border_color(colors.border_variant)
            .bg(colors.terminal_background)
            .child(offer)
            .into_any_element()
    });
    let mut parts: Vec<AnyElement> = [drive, prompt_editor, strip]
        .into_iter()
        .flatten()
        .collect();
    match parts.len() {
        0 => None,
        1 => parts.pop(),
        _ => Some(
            v_flex()
                .w_full()
                .flex_none()
                .children(parts)
                .into_any_element(),
        ),
    }
}

fn render(context: &MarleyFooterContext, _: &mut Window, cx: &mut App) -> Option<AnyElement> {
    let offer = links::offer(context, cx).map(|url| links::offer_button(context, url));
    let Some(BarContents {
        agent,
        folder,
        branch,
    }) = contents(context, cx)
    else {
        return footer_without_agent(context, offer, cx);
    };
    let microphone = microphone(context, cx);
    let prompt_editor = rich_input::element(context, cx);
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
    let bar = h_flex()
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
                .child(attach_button(context))
                .child(rich_input_button(context, agent))
                .children(microphone)
                .children(
                    (agent == AgentKind::Claude)
                        .then(|| claude_plugin_chip(context, cx))
                        .flatten(),
                )
                // Codex's and OpenCode's notifications, set up in a click (#552).
                .children(crate::agent_notify::chip(agent, context, cx))
                // An integration off on an untested agent version, and why (#648).
                .children(crate::agent_versions::chip(agent, context, cx)),
        )
        .child(
            h_flex()
                .min_w_0()
                .gap_3()
                .children(offer)
                .children(folder.map(|folder| {
                    let text = folder.compact().to_string_lossy().into_owned();
                    chip(IconName::Folder, text.into())
                        .debug_selector(|| "marley-agent-bar-folder".into())
                }))
                .children(branch.map(|branch| {
                    chip(IconName::GitBranch, branch)
                        .debug_selector(|| "marley-agent-bar-branch".into())
                })),
        );
    // The rich input's editor, while it is open, sits above the bar (#481).
    Some(
        v_flex()
            .flex_none()
            .w_full()
            .children(prompt_editor)
            .child(bar)
            .into_any_element(),
    )
}

/// Rich Input, which opens the terminal's editor for `agent`'s prompt (#481).
fn rich_input_button(context: &MarleyFooterContext, agent: AgentKind) -> AnyElement {
    let view = context.view.clone();
    div()
        .debug_selector(|| "marley-rich-input-button".into())
        .child(
            IconButton::new("marley-rich-input", IconName::Pencil)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .tooltip(Tooltip::for_action_title("Rich Input", &RichInput))
                .on_click(move |_, window, cx| {
                    if let Some(view) = view.upgrade() {
                        rich_input::open(&view, agent, window, cx);
                    }
                }),
        )
        .into_any_element()
}

/// Attach File, before the plugin's chip so it stays put when the chip goes.
fn attach_button(context: &MarleyFooterContext) -> AnyElement {
    let (view, workspace) = (context.view.clone(), context.workspace.clone());
    div()
        .debug_selector(|| "marley-attach-file".into())
        .child(
            IconButton::new("marley-attach-file", IconName::Plus)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .tooltip(Tooltip::for_action_title("Attach File", &AttachFile))
                .on_click(move |_, window, cx| {
                    workspace
                        .update(cx, |workspace, cx| {
                            attach(view.clone(), workspace, window, cx);
                        })
                        .log_err();
                }),
        )
        .into_any_element()
}

/// The microphone, where Voice is on and Voxtype is on the PATH. It shows what Voxtype is doing,
/// and a click puts the focus on its terminal, where Voxtype types, and starts or stops a
/// dictation.
fn microphone(context: &MarleyFooterContext, cx: &mut App) -> Option<AnyElement> {
    if MarleySettings::get_global(cx).dictation == Dictation::Off {
        return None;
    }
    let voice = cx.try_global::<Voice>()?;
    let voxtype = voice.voxtype.clone()?;
    let (color, tooltip) = microphone_look(voice.state);
    voice::follow_once_drawn(&voxtype, cx);
    let (workspace, focus_handle) = (context.workspace.clone(), context.focus_handle.clone());
    Some(
        div()
            .debug_selector(|| "marley-microphone".into())
            .child(
                IconButton::new("marley-microphone", IconName::Mic)
                    .icon_size(IconSize::Small)
                    .icon_color(color)
                    .tooltip(Tooltip::text(tooltip))
                    .on_click(move |_, window, cx| {
                        window.focus(&focus_handle, cx);
                        voice::toggle(voxtype.clone(), workspace.clone(), cx);
                    }),
            )
            .into_any_element(),
    )
}

/// The microphone's color and tooltip while Voxtype is in `state`.
const fn microphone_look(state: VoiceState) -> (Color, &'static str) {
    match state {
        VoiceState::Idle => (Color::Muted, "Dictate with Voxtype"),
        VoiceState::Recording => (Color::Error, "Stop and transcribe"),
        VoiceState::Transcribing => (Color::Warning, "Transcribing…"),
    }
}

/// The chip that installs Marley's plugin for Claude Code, while the plugin is known not to be
/// installed (#482).
fn claude_plugin_chip(context: &MarleyFooterContext, cx: &App) -> Option<AnyElement> {
    let plugin = cx.try_global::<ClaudePlugin>()?;
    // An install older than the plugin Marley ships lacks its newer hooks (#547).
    let update = plugin.needs_update();
    if plugin.installed != Some(false) && !update {
        return None;
    }
    let busy = if update {
        plugin
            .updating
            .then_some("Updating Marley's plugin for Claude Code…")
    } else {
        plugin
            .installing
            .then_some("Installing Marley's plugin for Claude Code…")
    };
    if let Some(busy) = busy {
        return Some(
            Label::new(busy)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .into_any_element(),
        );
    }
    let (plugin, workspace) = (plugin.clone(), context.workspace.clone());
    let button = if update {
        Button::new("marley-update-claude-plugin", "Update Marley's plugin")
            .start_icon(Icon::new(IconName::ArrowCircle).size(IconSize::XSmall))
            .tooltip(Tooltip::text(
                "Updates Marley's plugin for Claude Code, which then tells the rail what Claude \
                 Code is doing: your prompt, the tool it runs, what it waits on",
            ))
            .on_click(move |_, _, cx| {
                claude_plugin::update(plugin.clone(), workspace.clone(), cx);
            })
    } else {
        Button::new(
            "marley-enable-claude-notifications",
            "Connect Claude Code to Marley",
        )
        .start_icon(Icon::new(IconName::Download).size(IconSize::XSmall))
        .tooltip(Tooltip::text(
            "Installs Marley's plugin for Claude Code: notifications, and Marley's tools \
             for its terminals and Browser tabs, so the agent can open a page and drive \
             it while you watch",
        ))
        .on_click(move |_, _, cx| {
            claude_plugin::install(plugin.clone(), workspace.clone(), cx);
        })
    };
    Some(
        div()
            .debug_selector(|| "marley-claude-plugin-chip".into())
            .child(button.label_size(LabelSize::Small))
            .into_any_element(),
    )
}

#[cfg(test)]
#[path = "agent_bar_tests.rs"]
mod tests;
