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
use marley_terminal::BlockState;
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

/// A repository's branch and how many files changed in it (#908).
pub(crate) struct BranchChanges {
    pub(crate) branch: SharedString,
    /// The repository's changed files, as Zed's title bar counts them.
    pub(crate) changed: usize,
}

/// The branch of `project`'s innermost repository that holds `folder`, with its changed count, when
/// it is on one (#908).
pub(crate) fn branch_and_changes(
    project: &Entity<Project>,
    folder: &Path,
    cx: &App,
) -> Option<BranchChanges> {
    let repository = project
        .read(cx)
        .git_store()
        .read(cx)
        .repositories()
        .values()
        .filter(|repository| folder.starts_with(&repository.read(cx).work_directory_abs_path))
        .max_by_key(|repository| {
            repository
                .read(cx)
                .work_directory_abs_path
                .components()
                .count()
        })?
        .read(cx);
    let branch = repository.branch.as_ref()?.name().to_string().into();
    Some(BranchChanges {
        branch,
        changed: repository.status_summary().count,
    })
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
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    let drive = crate::terminal_drive::footer(context, cx);
    let prompt_editor = rich_input::element(context, rich_input::PromptFrame::Card, cx);
    let colors = cx.theme().colors();
    // The prompt editor as the boards' prompt card, which says where the command runs, as the
    // agent bar does (#826, #917).
    let card = prompt_editor.map(|editor| prompt_card(context, editor, window, cx));
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
    let mut parts: Vec<AnyElement> = [drive, card, strip].into_iter().flatten().collect();
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

/// The shell's prompt editor as the boards' prompt card (#917): the folder, the branch, the changed
/// files and a failed last exit over the line, and under it the prompt's keys as the keymap binds
/// them and Start an agent here.
fn prompt_card(
    context: &MarleyFooterContext,
    editor: AnyElement,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let colors = cx.theme().colors();
    let terminal = context.terminal.read(cx);
    let folder = terminal.working_directory();
    let repository = folder.as_deref().and_then(|folder| {
        let project = context.project.upgrade()?;
        branch_and_changes(&project, folder, cx)
    });
    let changed = repository
        .as_ref()
        .map(|repository| repository.changed)
        .filter(|changed| *changed > 0);
    let failed = last_exit(terminal);
    let chips = h_flex()
        .debug_selector(|| "marley-prompt-place".into())
        .w_full()
        .min_w_0()
        .gap_3()
        .children(folder.as_ref().map(|folder| {
            // Short, as the block headers name it (#630).
            let text = crate::block_headers::folder_label(&folder.to_string_lossy());
            colored_chip(
                IconName::Folder,
                text.into(),
                Color::Custom(colors.terminal_ansi_cyan),
            )
            .debug_selector(|| "marley-prompt-place-folder".into())
        }))
        .children(repository.map(|repository| {
            colored_chip(
                IconName::GitBranch,
                repository.branch,
                Color::Custom(colors.terminal_ansi_magenta),
            )
            .debug_selector(|| "marley-prompt-place-branch".into())
        }))
        .children(changed.map(|changed| {
            Label::new(format!("{changed} changed"))
                .size(LabelSize::Small)
                .color(Color::Muted)
        }))
        .children(failed.map(|code| {
            Label::new(format!("last: exit {code}"))
                .size(LabelSize::Small)
                .color(Color::Error)
        }));
    let start_here = folder.map(|folder| {
        Button::new("marley-start-agent-here", "Start an agent here")
            .style(ButtonStyle::Subtle)
            .label_size(LabelSize::XSmall)
            .color(Color::Accent)
            .on_click(move |_, window, cx| {
                window.dispatch_action(
                    Box::new(crate::agents::NewAgent {
                        folder: Some(folder.clone()),
                    }),
                    cx,
                );
            })
    });
    let hints = h_flex()
        .w_full()
        .gap_3()
        .children(key_hints(&context.view, window, cx))
        .child(div().flex_1())
        .children(start_here);
    v_flex()
        .debug_selector(|| "marley-prompt-card".into())
        .flex_none()
        .m_2()
        .px_3()
        .py_2()
        .gap_1p5()
        .rounded(px(14.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.editor_background)
        .child(chips)
        .child(editor)
        .child(hints)
        .into_any_element()
}

/// The newest live block's exit while it failed and the shell waits at its prompt (#917).
fn last_exit(terminal: &Terminal) -> Option<i32> {
    let anchored = terminal.marley_anchored();
    let block = anchored.live().last()?;
    let code = block.exit_code.0.filter(|code| *code != 0)?;
    (block.state == BlockState::Finished && anchored.at_prompt()).then_some(code)
}

/// The prompt's keys, each with what it does, read from the keymap from the prompt editor; a key
/// shows only while its action is bound (#917, D2).
fn key_hints(view: &WeakEntity<TerminalView>, window: &Window, cx: &App) -> Vec<AnyElement> {
    let Some(focus) = rich_input::editor_focus(view, cx) else {
        return Vec::new();
    };
    let hints: [(&dyn gpui::Action, &'static str); 4] = [
        (&crate::SendRichInput, "runs"),
        (&crate::AskAgent, "asks the agent"),
        (&crate::RichInputUp, "history"),
        (&RichInput, "a bigger editor"),
    ];
    let mut shown = Vec::new();
    for (action, words) in hints {
        let binding = ui::KeyBinding::for_action_in(action, &focus, cx);
        if !binding.has_binding(window) {
            continue;
        }
        shown.push(
            h_flex()
                .gap_1()
                .child(binding.size(rems_from_px(11_f32)))
                .child(
                    Label::new(words)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
                .into_any_element(),
        );
    }
    shown
}

/// A chip with its icon and name in `color` (#917).
fn colored_chip(icon: IconName, text: SharedString, color: Color) -> Div {
    h_flex()
        .min_w_0()
        .gap_1()
        .child(Icon::new(icon).size(IconSize::XSmall).color(color))
        .child(
            Label::new(text)
                .size(LabelSize::Small)
                .color(color)
                .truncate(),
        )
}

/// A folder's or a branch's chip: its icon and its muted name (#477, #826).
fn place_chip(icon: IconName, text: SharedString) -> Div {
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
}

fn render(context: &MarleyFooterContext, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
    let offer = links::offer(context, cx).map(|url| links::offer_button(context, url));
    let Some(BarContents {
        agent,
        folder,
        branch,
    }) = contents(context, cx)
    else {
        return footer_without_agent(context, offer, window, cx);
    };
    let microphone = microphone(context, cx);
    let prompt_editor = rich_input::element(context, rich_input::PromptFrame::Strip, cx);
    let colors = cx.theme().colors();
    let chip = place_chip;
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
                .child(rich_input_button(context, agent, cx))
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

/// Rich Input, which opens the terminal's editor for `agent`'s prompt (#481), and while the editor
/// is open shows pressed and hides it, keeping the draft (#677).
fn rich_input_button(context: &MarleyFooterContext, agent: AgentKind, cx: &App) -> AnyElement {
    let view = context.view.clone();
    let open = view
        .upgrade()
        .is_some_and(|view| rich_input::is_open_for_agent(&view, cx));
    let title = if open {
        "Hide Rich Input"
    } else {
        "Rich Input"
    };
    div()
        .debug_selector(|| "marley-rich-input-button".into())
        .child(
            IconButton::new("marley-rich-input", IconName::Pencil)
                .icon_size(IconSize::Small)
                .icon_color(Color::Muted)
                .toggle_state(open)
                .tooltip(Tooltip::for_action_title(title, &RichInput))
                .on_click(move |_, window, cx| {
                    let Some(view) = view.upgrade() else {
                        return;
                    };
                    // The editor's state when the click lands, not when the bar was drawn.
                    if rich_input::is_open_for_agent(&view, cx) {
                        rich_input::hide(&view, window, cx);
                    } else {
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
    let look = microphone_look(voice.state);
    voice::follow_once_drawn(&voxtype, cx);
    let (workspace, focus_handle) = (context.workspace.clone(), context.focus_handle.clone());
    let toggle = move |_: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
        window.focus(&focus_handle, cx);
        voice::toggle(voxtype.clone(), workspace.clone(), cx);
    };
    // While Voxtype works, the button says what it does in a pill, as the board draws it (#929).
    let button = match look.words {
        Some(words) => {
            let ink = look.color.color(cx);
            h_flex()
                .id("marley-microphone")
                .h(px(22.))
                .px_2()
                .gap_1p5()
                .rounded_full()
                .bg(ink.opacity(0.12))
                .cursor_pointer()
                .child(div().flex_none().size(px(6.)).rounded_full().bg(ink))
                .child(Label::new(words).size(LabelSize::XSmall).color(look.color))
                .tooltip(Tooltip::text(look.tooltip))
                .on_click(toggle)
                .into_any_element()
        }
        None => IconButton::new("marley-microphone", IconName::Mic)
            .icon_size(IconSize::Small)
            .icon_color(look.color)
            .tooltip(Tooltip::text(look.tooltip))
            .on_click(toggle)
            .into_any_element(),
    };
    Some(
        div()
            .debug_selector(|| "marley-microphone".into())
            .child(button)
            .into_any_element(),
    )
}

/// How the microphone looks while Voxtype is in one state (#929, D5).
struct MicrophoneLook {
    color: Color,
    tooltip: &'static str,
    /// The pill's words while Voxtype works; none while it waits, when the icon shows.
    words: Option<&'static str>,
}

/// The microphone's look while Voxtype is in `state`.
const fn microphone_look(state: VoiceState) -> MicrophoneLook {
    match state {
        VoiceState::Idle => MicrophoneLook {
            color: Color::Muted,
            tooltip: "Dictate with Voxtype",
            words: None,
        },
        VoiceState::Recording => MicrophoneLook {
            color: Color::Error,
            tooltip: "Stop and transcribe",
            words: Some("Listening \u{b7} Voxtype"),
        },
        VoiceState::Transcribing => MicrophoneLook {
            color: Color::Warning,
            tooltip: "Transcribing\u{2026}",
            words: Some("Transcribing \u{b7} Voxtype"),
        },
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
