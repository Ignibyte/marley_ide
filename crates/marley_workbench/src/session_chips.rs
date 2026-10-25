//! The tab row's right end for a terminal tab (#917): the session the tab shows, its folder and its
//! branch, as the Terminal Session board names them.
//!
//! The pane's tab bar asks the `MarleyTabBarEnd` hook this module sets for an element before its
//! end buttons, given the active item; only a terminal tab gets one.

use std::path::Path;

use gpui::{AnyElement, App, Entity, Window};
use terminal::{TaskStatus, Terminal};
use terminal_view::TerminalView;
use ui::prelude::*;
use workspace::MarleyTabBarEnd;
use workspace::item::ItemHandle;

use crate::tmux_sessions::TmuxHosts;

/// Sets the tab bar's hook; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyTabBarEnd(tab_bar_end));
}

/// What a terminal tab shows.
enum Session {
    /// A local shell, by its program's name.
    Shell(SharedString),
    /// A tmux session, by its name, with its window count when the listing has it.
    Tmux {
        session: SharedString,
        windows: Option<u32>,
    },
    /// A shell on another machine over ssh, by its host, and whether the link is down.
    Ssh { host: SharedString, lost: bool },
}

/// The chips for `item` when it is a terminal: its session, then its folder and its branch.
fn tab_bar_end(item: &dyn ItemHandle, _: &mut Window, cx: &mut App) -> Option<AnyElement> {
    let view = item.act_as::<TerminalView>(cx)?;
    let terminal = view.read(cx).terminal().clone();
    let session = session_of(&terminal, cx);
    let folder = terminal.read(cx).working_directory();
    let branch = folder
        .as_deref()
        .and_then(|folder| branch_of(&view, folder, cx));
    let colors = cx.theme().colors();
    Some(
        h_flex()
            .debug_selector(|| "marley-session-chips".into())
            .flex_none()
            .gap_1p5()
            .pr_1()
            .children(session.map(|session| session_chip(&session, cx)))
            .children(folder.map(|folder| {
                chip(
                    IconName::Folder,
                    crate::block_headers::folder_label(&folder.to_string_lossy()).into(),
                    Color::Custom(colors.terminal_ansi_cyan),
                    cx,
                )
            }))
            .children(branch.map(|branch| {
                chip(
                    IconName::GitBranch,
                    branch,
                    Color::Custom(colors.terminal_ansi_magenta),
                    cx,
                )
            }))
            .into_any_element(),
    )
}

/// The session `terminal` shows: a tmux session it attached through Marley (#821), a remote shell
/// (#641), or the local shell, named by the program at its prompt, else by `$SHELL`.
fn session_of(terminal: &Entity<Terminal>, cx: &App) -> Option<Session> {
    if let Some(link) = crate::remote::link_of(terminal, cx) {
        return Some(Session::Ssh {
            host: link.host,
            lost: link.lost,
        });
    }
    let terminal = terminal.read(cx);
    if let Some(task) = terminal
        .task()
        .filter(|task| task.status == TaskStatus::Running)
        && let Some(tmux) = cx.try_global::<TmuxHosts>()
        && let Some(attach) = tmux.attach_of(&task.spawned_task.full_label)
    {
        let windows = tmux
            .session_of(&attach.host, &attach.session)
            .map(|session| session.windows);
        return Some(Session::Tmux {
            session: attach.session.clone().into(),
            windows,
        });
    }
    let at_prompt = terminal.marley_anchored().at_prompt();
    let name = at_prompt
        .then(|| terminal.foreground_process_command_name())
        .flatten()
        .or_else(|| {
            let shell = std::env::var_os("SHELL")?;
            Some(
                Path::new(&shell)
                    .file_name()?
                    .to_string_lossy()
                    .into_owned(),
            )
        })?;
    Some(Session::Shell(name.into()))
}

/// The branch of the folder in the view's project, as the block headers find it.
fn branch_of(view: &Entity<TerminalView>, folder: &Path, cx: &App) -> Option<SharedString> {
    let workspace = view.read(cx).marley_workspace().upgrade()?;
    let project = workspace.read(cx).project().clone();
    crate::agent_bar::branch_of(&project, folder, cx)
}

/// The session's chip: "bash", "tmux · build · 2 windows", or "ssh · host · connected", the lost
/// link in the error colour.
fn session_chip(session: &Session, cx: &App) -> AnyElement {
    let (words, color) = match session {
        Session::Shell(name) => (name.to_string(), Color::Default),
        Session::Tmux { session, windows } => (
            match windows {
                Some(1) => format!("tmux \u{b7} {session} \u{b7} 1 window"),
                Some(count) => format!("tmux \u{b7} {session} \u{b7} {count} windows"),
                None => format!("tmux \u{b7} {session}"),
            },
            Color::Default,
        ),
        Session::Ssh { host, lost: false } => (
            format!("ssh \u{b7} {host} \u{b7} connected"),
            Color::Default,
        ),
        Session::Ssh { host, lost: true } => {
            (format!("ssh \u{b7} {host} \u{b7} link lost"), Color::Error)
        }
    };
    chip(IconName::Terminal, words.into(), color, cx)
}

/// A chip: its icon and its words in `color`, on the element background.
fn chip(icon: IconName, words: SharedString, color: Color, cx: &App) -> AnyElement {
    h_flex()
        .h(px(22.))
        .px_2()
        .gap_1()
        .rounded(px(6.))
        .bg(cx.theme().colors().element_background)
        .child(Icon::new(icon).size(IconSize::XSmall).color(color))
        .child(
            Label::new(words)
                .size(LabelSize::XSmall)
                .color(color)
                .single_line(),
        )
        .into_any_element()
}
