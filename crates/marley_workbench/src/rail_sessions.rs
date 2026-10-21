//! The rail's Sessions view (#832): every tmux session on every host Marley looks on, by host, with
//! where each is placed and whether it is attached, and the hosts to look on as checkboxes.

use gpui::{AnyElement, App, Context, PromptLevel, SharedString, Task, Window};
use ui::{
    Checkbox, ContextMenu, Icon, IconSize, Label, LabelSize, ToggleState, prelude::*,
    right_click_menu,
};
use util::ResultExt as _;
use workspace::notifications::DetachAndPromptErr as _;

use super::{Rail, RowIcon, RowKind, RowLine, row_card};
use crate::tmux_sessions::{Listing, TmuxHost, TmuxHosts, TmuxSession};

/// Whether the Sessions view shows, in place of the projects' rows or the threads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SessionsView {
    #[default]
    Hidden,
    Shown,
}

impl SessionsView {
    pub(super) const fn shown(self) -> bool {
        matches!(self, Self::Shown)
    }
}

impl Rail {
    pub(super) fn render_sessions(&self, cx: &Context<Self>) -> impl IntoElement {
        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(tmux) = cx.try_global::<TmuxHosts>() {
            let attached = self.tmux_attached_here(tmux, cx);
            for host in &tmux.hosts {
                children.push(host_heading(host, cx));
                if let Some(words) = crate::tmux_sessions::error_words(host) {
                    children.push(
                        div()
                            .px_2()
                            .pb_1()
                            .child(
                                Label::new(words)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Error),
                            )
                            .into_any_element(),
                    );
                }
                if let Listing::Sessions(sessions) = &host.listing {
                    for session in sessions {
                        let here = attached.contains(&(host.name.clone(), session.name.clone()));
                        children.push(self.session_row(host, session, here, cx));
                    }
                }
            }
            if tmux.hosts.is_empty() {
                children.push(
                    div()
                        .px_2()
                        .py_1()
                        .child(
                            Label::new("No host is looked on. Tick one below.")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                        .into_any_element(),
                );
            }
        }
        children.push(host_choices(cx));
        v_flex()
            .id("marley-rail-sessions")
            .flex_1()
            .overflow_y_scroll()
            .px_2()
            .py_1p5()
            .gap_0p5()
            .children(children)
    }

    /// A session's row: its name, and where it is placed, whether it is attached and its windows.
    fn session_row(
        &self,
        host: &TmuxHost,
        session: &TmuxSession,
        here: bool,
        cx: &Context<Self>,
    ) -> AnyElement {
        let under = self
            .tmux_place(host, session, cx)
            .and_then(|index| self.snapshot.rail.projects.get(index))
            .map_or_else(|| "Home".to_string(), |project| project.name.clone());
        let state = if here {
            "attached here"
        } else if session.attached {
            "attached elsewhere"
        } else {
            "detached"
        };
        let windows = match session.windows {
            1 => "1 window".to_string(),
            count => format!("{count} windows"),
        };
        let icon = session.agent.map_or_else(
            || {
                Label::new(">_")
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .buffer_font(cx)
                    .into_any_element()
            },
            |agent| {
                Icon::new(crate::agents::cli_icon(agent))
                    .size(IconSize::Small)
                    .color(Color::Muted)
                    .into_any_element()
            },
        );
        let title = Label::new(session.name.clone())
            .size(LabelSize::Small)
            .truncate()
            .into_any_element();
        let element = SharedString::from(format!("{}-{}", host.name, session.name));
        let (host_name, name) = (host.name.clone(), session.name.clone());
        let card = row_card(
            SharedString::from(format!("marley-rail-session-{element}")),
            format!("marley-rail-session-icon-{element}"),
            false,
            RowIcon::new(icon, RowKind::Tmux, None),
            title,
            vec![RowLine::muted(format!("{under} · {state} · {windows}"))],
            cx,
        )
        .on_click(cx.listener({
            let (host_name, name) = (host_name.clone(), name.clone());
            move |rail, _, window, cx| rail.open_session(&host_name, &name, window, cx)
        }));
        let rail = cx.weak_entity();
        let label = host.label();
        right_click_menu(SharedString::from(format!(
            "marley-rail-session-menu-{element}"
        )))
        .trigger(move |_, _, _| card)
        .menu(move |window, cx| {
            let (rail, host_name, name, label) =
                (rail.clone(), host_name.clone(), name.clone(), label.clone());
            ContextMenu::build(window, cx, move |menu, _, _| {
                let (open_rail, open_host, open_name) =
                    (rail.clone(), host_name.clone(), name.clone());
                menu.entry(
                    if here { "Show" } else { "Attach" },
                    None,
                    move |window, cx| {
                        open_rail
                            .update(cx, |rail, cx| {
                                rail.open_session(&open_host, &open_name, window, cx);
                            })
                            .log_err();
                    },
                )
                .entry("Close Session…", None, move |window, cx| {
                    close_session(&host_name, &label, &name, window, cx);
                })
            })
        })
        .into_any_element()
    }

    /// Shows the tab a session is attached in here, or attaches it in the project it is placed
    /// under.
    fn open_session(
        &self,
        host: &SharedString,
        session: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(tmux) = cx.try_global::<TmuxHosts>() else {
            return;
        };
        let attached = self.snapshot.terminals.values().find(|entry| {
            entry.view.upgrade().is_some_and(|view| {
                let terminal = view.read(cx).terminal().read(cx);
                terminal.task().is_some_and(|task| {
                    tmux.attach_of(&task.spawned_task.full_label)
                        .is_some_and(|attach| attach.host == *host && attach.session == session)
                })
            })
        });
        if let Some(entry) = attached {
            let (workspace, view) = (entry.workspace.clone(), entry.view.clone());
            self.activate_terminal(&workspace, &view, window, cx)
                .log_err();
            return;
        }
        let place = tmux
            .hosts
            .iter()
            .find(|known| known.name == *host)
            .and_then(|known| match &known.listing {
                Listing::Sessions(sessions) => sessions
                    .iter()
                    .find(|listed| listed.name == session)
                    .and_then(|listed| self.tmux_place(known, listed, cx)),
                Listing::Waiting | Listing::Failed(_) => None,
            });
        if let Some(index) = place {
            self.attach_tmux(host, session, index, window, cx);
        }
    }
}

/// A host's heading: a dot for its listing's state, its name and how many sessions it has.
fn host_heading(host: &TmuxHost, cx: &App) -> AnyElement {
    let (dot, count) = match &host.listing {
        Listing::Sessions(sessions) => (cx.theme().status().success, sessions.len().to_string()),
        Listing::Failed(_) => (cx.theme().status().error, String::new()),
        Listing::Waiting => (cx.theme().colors().text_muted, String::new()),
    };
    h_flex()
        .debug_selector({
            let name = host.name.clone();
            move || format!("marley-rail-session-host-{name}")
        })
        .px_2()
        .pt_2()
        .pb_1()
        .gap_1p5()
        .child(div().size_1p5().flex_none().rounded_full().bg(dot))
        .child(
            Label::new(host.label())
                .size(LabelSize::Small)
                .weight(gpui::FontWeight::SEMIBOLD),
        )
        .child(
            Label::new(count)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .into_any_element()
}

/// HOSTS TO LOOK ON: this machine and each saved ssh host, ticked while Marley looks on it.
fn host_choices(cx: &App) -> AnyElement {
    let choices = crate::tmux_sessions::host_choices(cx);
    v_flex()
        .debug_selector(|| "marley-rail-session-hosts".into())
        .pt_4()
        .gap_1()
        .child(
            div().px_2().child(
                Label::new("HOSTS TO LOOK ON")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            ),
        )
        .children(choices.into_iter().map(|choice| {
            let name = choice.name.clone();
            let state = if choice.listed {
                ToggleState::Selected
            } else {
                ToggleState::Unselected
            };
            h_flex()
                .px_2()
                .gap_2()
                .child(
                    Checkbox::new(
                        SharedString::from(format!("marley-rail-tmux-host-{}", choice.name)),
                        state,
                    )
                    .on_click(move |state, _, cx| {
                        crate::tmux_sessions::set_listed(
                            &name,
                            *state == ToggleState::Selected,
                            cx,
                        );
                    }),
                )
                .child(Label::new(choice.label).size(LabelSize::Small))
                .child(
                    Label::new(choice.from)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
        }))
        .into_any_element()
}

/// Asks, then ends the session `session` on the host `host` and lists the hosts again.
fn close_session(host: &str, label: &str, session: &str, window: &mut Window, cx: &mut App) {
    let answer = window.prompt(
        PromptLevel::Warning,
        &format!("End the tmux session {session} on {label}?"),
        Some("Its programs stop."),
        &["End Session", "Cancel"],
        cx,
    );
    let (host, session) = (host.to_string(), session.to_string());
    window
        .spawn(cx, async move |cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            let Ok(killed) = cx.update(|_, cx| crate::tmux_sessions::kill(&host, &session, cx))
            else {
                return;
            };
            let result = killed.await;
            cx.update(|window, cx| match result {
                Ok(()) => crate::tmux_sessions::list_soon(cx),
                Err(error) => Task::ready(Err::<(), _>(error)).detach_and_prompt_err(
                    "Could not end the tmux session",
                    window,
                    cx,
                    |_, _, _| None,
                ),
            })
            .log_err();
        })
        .detach();
}
