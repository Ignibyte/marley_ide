//! The Sessions page (#861): every tmux session Marley lists, as a table.
//!
//! It reads the `TmuxHosts` global the rail's Sessions view reads (#832) and redraws when it
//! changes: a header with the counts and New Session, a table with its column headers and a row
//! per session with its state chip, Show or Attach and Close, the line on how the rail places a
//! session, and the hosts to look on as checkboxes (#930).

use std::collections::HashSet;

use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, PromptLevel, SharedString,
    Subscription, WeakEntity, Window, actions,
};
use ui::{Checkbox, ToggleState, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{MultiWorkspace, Workspace};

use crate::prompts::{Role, render_button};
use crate::tmux_sessions::{self, Listing, SessionKey, TmuxHosts};

actions!(
    marley,
    [
        /// Opens the Sessions page: every tmux session Marley lists, on every host, as a table.
        #[derive(Eq)]
        OpenSessionsPage
    ]
);

/// Registers the open action on every workspace.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenSessionsPage, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings the workspace's Sessions page forward, or opens one in the center.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<SessionsPage>(cx).next();
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let handle = workspace.weak_handle();
    let tab = cx.new(|cx| SessionsPage::new(handle, cx));
    workspace.add_item_to_center(Box::new(tab), window, cx);
}

/// One session as the table shows it.
struct Row {
    host: SharedString,
    host_label: SharedString,
    session: String,
    windows: u32,
    folder: String,
    state: SessionState,
    agent: Option<SharedString>,
}

/// Where a session is attached (#930).
#[derive(Clone, Copy, PartialEq, Eq)]
enum SessionState {
    /// In a terminal of this window.
    AttachedHere,
    /// By a client elsewhere: another terminal program, another machine.
    AttachedElsewhere,
    Detached,
}

/// The table's columns' widths, shared by the header row and the rows.
const SESSION_WIDTH: f32 = 200.;
const HOST_WIDTH: f32 = 140.;
const WINDOWS_WIDTH: f32 = 90.;
const STATE_WIDTH: f32 = 130.;

/// The Sessions page.
pub(crate) struct SessionsPage {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    _hosts: Subscription,
}

impl SessionsPage {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
            _hosts: cx.observe_global::<TmuxHosts>(|_, cx| cx.notify()),
        }
    }

    /// Every listed session, host by host in the setting's order; `here` holds those attached in
    /// this window.
    fn rows(here: &HashSet<SessionKey>, cx: &App) -> Vec<Row> {
        let Some(hosts) = cx.try_global::<TmuxHosts>() else {
            return Vec::new();
        };
        hosts
            .hosts
            .iter()
            .flat_map(|host| {
                let sessions = match &host.listing {
                    Listing::Sessions(sessions) => sessions.clone(),
                    Listing::Waiting | Listing::Failed(_) => Vec::new(),
                };
                let (name, label) = (host.name.clone(), host.label());
                sessions.into_iter().map(move |session| {
                    let key = SessionKey {
                        host: name.clone(),
                        session: session.name.clone(),
                    };
                    let state = if here.contains(&key) {
                        SessionState::AttachedHere
                    } else if session.attached {
                        SessionState::AttachedElsewhere
                    } else {
                        SessionState::Detached
                    };
                    Row {
                        host: name.clone(),
                        host_label: label.clone(),
                        session: session.name,
                        windows: session.windows,
                        folder: session.folder.display().to_string(),
                        state,
                        agent: session
                            .agent
                            .map(|agent| SharedString::new_static(agent.display_name())),
                    }
                })
            })
            .collect()
    }

    fn render_header(&self, rows: &[Row], cx: &Context<Self>) -> impl IntoElement + use<> {
        let hosts = rows
            .iter()
            .map(|row| row.host.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let count = |state: SessionState| rows.iter().filter(|row| row.state == state).count();
        let elsewhere = count(SessionState::AttachedElsewhere);
        // The parts add up to the total: elsewhere shows only when there is one (#930, D4).
        let mut parts = vec![
            format!(
                "{} on {hosts} {}",
                rows.len(),
                if hosts == 1 { "host" } else { "hosts" }
            ),
            format!("{} attached here", count(SessionState::AttachedHere)),
        ];
        if elsewhere > 0 {
            parts.push(format!("{elsewhere} attached elsewhere"));
        }
        parts.push(format!("{} detached", count(SessionState::Detached)));
        let summary = parts.join(" \u{b7} ");
        let workspace = self.workspace.clone();
        h_flex()
            .gap_3()
            .child(Headline::new("Sessions").size(HeadlineSize::Small))
            .child(
                Label::new(summary)
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(div().flex_1())
            .child(
                render_button(0, "New Session", Role::Primary, false, cx)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        workspace
                            .update(cx, |workspace, cx| {
                                let folder = workspace
                                    .project()
                                    .read(cx)
                                    .visible_worktrees(cx)
                                    .next()
                                    .map(|worktree| worktree.read(cx).abs_path().to_path_buf());
                                let result = folder.map_or_else(
                                    || {
                                        Err(anyhow::anyhow!(
                                            "this window has no project folder to start a \
                                             session in"
                                        ))
                                    },
                                    |folder| {
                                        tmux_sessions::new_session_in(
                                            &folder, None, workspace, window, cx,
                                        )
                                    },
                                );
                                tmux_sessions::report(result, window, cx);
                            })
                            .log_err();
                    }),
            )
    }

    /// The table's column headers, at the rows' widths (#930).
    fn render_columns(cx: &App) -> impl IntoElement + use<> {
        let heading = |words: &'static str| {
            Label::new(words)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
        };
        h_flex()
            .w_full()
            .gap_3()
            .px_3()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                div()
                    .w(px(SESSION_WIDTH))
                    .flex_none()
                    .child(heading("Session")),
            )
            .child(div().w(px(HOST_WIDTH)).flex_none().child(heading("Host")))
            .child(
                div()
                    .w(px(WINDOWS_WIDTH))
                    .flex_none()
                    .child(heading("Windows")),
            )
            .child(div().flex_1().min_w_0().child(heading("Folder")))
            .child(div().w(px(STATE_WIDTH)).flex_none().child(heading("State")))
            // The buttons' room.
            .child(div().w(px(150.)).flex_none())
    }

    fn render_row(&self, index: usize, row: Row, cx: &Context<Self>) -> AnyElement {
        let colors = cx.theme().colors();
        let windows = match row.windows {
            1 => "1 window".to_string(),
            count => format!("{count} windows"),
        };
        let shown = row.state == SessionState::AttachedHere;
        let (attach_host, attach_session) = (row.host.to_string(), row.session.clone());
        let (close_host, close_session) = (row.host.to_string(), row.session.clone());
        let workspace = self.workspace.clone();
        h_flex()
            .w_full()
            .gap_3()
            .px_3()
            .py_1p5()
            .border_b_1()
            .border_color(colors.border_variant)
            .child(
                v_flex()
                    .w(px(SESSION_WIDTH))
                    .flex_none()
                    .child(
                        Label::new(row.session.clone())
                            .size(LabelSize::Small)
                            .buffer_font(cx),
                    )
                    .children(row.agent.map(|agent| {
                        Label::new(agent)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                    })),
            )
            .child(
                div()
                    .w(px(HOST_WIDTH))
                    .flex_none()
                    .child(Label::new(row.host_label).size(LabelSize::Small)),
            )
            .child(
                div().w(px(WINDOWS_WIDTH)).flex_none().child(
                    Label::new(windows)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .child(
                div().flex_1().min_w_0().child(
                    Label::new(row.folder)
                        .size(LabelSize::XSmall)
                        .buffer_font(cx)
                        .color(Color::Muted)
                        .truncate(),
                ),
            )
            .child(
                h_flex()
                    .w(px(STATE_WIDTH))
                    .flex_none()
                    .child(state_chip(row.state, cx)),
            )
            .child(
                render_button(
                    index * 2 + 10,
                    if shown { "Show" } else { "Attach" },
                    Role::Plain,
                    false,
                    cx,
                )
                .cursor_pointer()
                .on_click(move |_, window, cx| {
                    show_or_attach(shown, &attach_host, &attach_session, &workspace, window, cx);
                }),
            )
            .child(
                render_button(index * 2 + 11, "Close…", Role::Quiet, false, cx)
                    .cursor_pointer()
                    .on_click(move |_, window, cx| {
                        close(close_host.clone(), close_session.clone(), window, cx);
                    }),
            )
            .into_any_element()
    }

    fn render_hosts(cx: &App) -> impl IntoElement {
        let errors: Vec<String> = cx
            .try_global::<TmuxHosts>()
            .map(|hosts| {
                hosts
                    .hosts
                    .iter()
                    .filter_map(tmux_sessions::error_words)
                    .collect()
            })
            .unwrap_or_default();
        // The ticks live here, the rail lists what is not ticked (#930, D2).
        v_flex()
            .debug_selector(|| "marley-sessions-hosts".into())
            .gap_1p5()
            .child(
                Label::new("HOSTS TO LOOK ON")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .children(tmux_sessions::host_choices(cx).into_iter().map(|choice| {
                let name = choice.name.clone();
                let ticked = if choice.listed {
                    ToggleState::Selected
                } else {
                    ToggleState::Unselected
                };
                h_flex()
                    .w(px(420.))
                    .gap_2()
                    .child(
                        Checkbox::new(
                            SharedString::from(format!("marley-sessions-host-{}", choice.name)),
                            ticked,
                        )
                        .on_click(move |state, _, cx| {
                            tmux_sessions::set_listed(&name, *state == ToggleState::Selected, cx);
                        }),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .child(Label::new(choice.label).size(LabelSize::Small))
                            .child(
                                Label::new(choice.from)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        Label::new(if choice.listed {
                            "looking"
                        } else {
                            "not listed"
                        })
                        .size(LabelSize::XSmall)
                        .color(if choice.listed {
                            Color::Success
                        } else {
                            Color::Muted
                        }),
                    )
            }))
            .child(
                Label::new("marley.tmux_hosts")
                    .size(LabelSize::XSmall)
                    .buffer_font(cx)
                    .color(Color::Muted),
            )
            .children(
                errors
                    .into_iter()
                    .map(|error| Label::new(error).size(LabelSize::Small).color(Color::Error)),
            )
    }
}

/// Show brings a session attached here forward in its tab (#930); Attach, or Show when its tab is
/// gone, attaches it in `workspace`.
fn show_or_attach(
    shown: bool,
    host: &str,
    session: &str,
    workspace: &WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    if shown {
        let workspaces = window_workspaces(window, cx);
        if let Some(tab) = tmux_sessions::attached_tab(&workspaces, host, session, cx) {
            crate::browser::reveal_terminal(&tab.view, window, cx);
            return;
        }
    }
    workspace
        .update(cx, |workspace, cx| {
            let result = tmux_sessions::attach(host, session, workspace, window, cx);
            tmux_sessions::report(result, window, cx);
        })
        .log_err();
}

/// A session's state as its chip (#930): attached here in the success colours, attached elsewhere
/// in the warning colours, detached quiet.
fn state_chip(state: SessionState, cx: &App) -> Div {
    let (colors, status) = (cx.theme().colors(), cx.theme().status());
    let (words, ink, fill) = match state {
        SessionState::AttachedHere => ("attached here", status.success, status.success_background),
        SessionState::AttachedElsewhere => (
            "attached elsewhere",
            status.warning,
            status.warning_background,
        ),
        SessionState::Detached => ("detached", colors.text_muted, colors.element_background),
    };
    div().flex().px_1p5().rounded_full().bg(fill).child(
        Label::new(words)
            .size(LabelSize::XSmall)
            .color(Color::Custom(ink)),
    )
}

/// The workspaces of `window`, for what is attached here.
fn window_workspaces(window: &Window, cx: &App) -> Vec<Entity<Workspace>> {
    window
        .root::<MultiWorkspace>()
        .flatten()
        .map(|multi_workspace| multi_workspace.read(cx).workspaces().cloned().collect())
        .unwrap_or_default()
}

/// Asks before ending `session` on the host named `host` (D1), then ends it; the list follows.
fn close(host: String, session: String, window: &mut Window, cx: &mut App) {
    let answer = window.prompt(
        PromptLevel::Warning,
        &format!("Close the tmux session {session}?"),
        Some("tmux ends the session and every program running in it."),
        &["Close", "Cancel"],
        cx,
    );
    window
        .spawn(cx, async move |cx| {
            if !matches!(answer.await, Ok(0)) {
                return anyhow::Ok(());
            }
            cx.update(|_, cx| tmux_sessions::kill(&host, &session, cx))?
                .await
        })
        .detach_and_log_err(cx);
}

impl Focusable for SessionsPage {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for SessionsPage {}

impl Render for SessionsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let here = tmux_sessions::attached_here(&window_workspaces(window, cx), cx);
        let rows = Self::rows(&here, cx);
        let columns = (!rows.is_empty()).then(|| Self::render_columns(cx));
        let header = self.render_header(&rows, cx);
        let empty = rows.is_empty().then(|| {
            crate::states::EmptyState::new(
                "tx",
                "No tmux sessions",
                "Start one with New Session, or from a project's + in the rail.",
            )
            .render(1, cx)
        });
        let table: Vec<AnyElement> = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| self.render_row(index, row, cx))
            .collect();
        v_flex()
            .id("marley-sessions-page")
            .key_context("MarleySessionsPage")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_6()
            .gap_4()
            .bg(colors.editor_background)
            .child(header)
            .child(
                v_flex()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.elevated_surface_background)
                    .children(columns)
                    .children(table)
                    .children(empty),
            )
            .child(
                Label::new(
                    "A session goes in the rail under the project whose folder holds the folder it \
                     started in; the rest go under Default.",
                )
                .size(LabelSize::Small)
                .color(Color::Muted),
            )
            .child(Self::render_hosts(cx))
    }
}

impl Item for SessionsPage {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Sessions")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::TerminalAlt))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Every tmux session Marley lists"))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
