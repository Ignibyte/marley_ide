//! The bottom row while a debug session is paused (#901), as the `BottomRow` board draws it.
//!
//! Where it stopped, the debugger's Continue, Step over, Step in, Step out and Stop, the frame it
//! stopped in, its line and column, and how many breakpoints are set.
//!
//! The row draws what it last read: the session's threads and frames are read through the
//! session, which may ask the adapter for them, so they are read when the debugger's store, a
//! session or the breakpoints change, never while the row draws. Its buttons dispatch
//! `debugger_ui`'s own actions, which the workspace answers while a thread is stopped.

use std::collections::HashMap;
use std::path::Path;

use gpui::{App, Context, Entity, Render, Subscription, Window};
use project::Project;
use project::debugger::session::{
    Session, SessionEvent, SessionStateEvent, ThreadId, ThreadStatus,
};
use ui::{Tooltip, prelude::*};
use workspace::item::ItemHandle;
use workspace::{StatusItemView, Workspace};

/// Where a paused session stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Paused {
    /// The file's name and line, when the adapter has sent the frame.
    place: Option<(String, u64)>,
    /// The frame's function.
    frame: Option<String>,
    column: Option<u64>,
}

/// The paused session's part of the bottom row.
pub struct DebugRow {
    project: Entity<Project>,
    paused: Option<Paused>,
    breakpoints: usize,
    /// Each session's two kinds of events, followed while it lasts.
    sessions: HashMap<gpui::EntityId, [Subscription; 2]>,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for DebugRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DebugRow")
            .field("paused", &self.paused)
            .field("breakpoints", &self.breakpoints)
            .finish_non_exhaustive()
    }
}

impl DebugRow {
    pub(crate) fn new(workspace: &Workspace, cx: &mut Context<Self>) -> Self {
        let project = workspace.project().clone();
        let (dap_store, breakpoint_store) = {
            let project = project.read(cx);
            (project.dap_store(), project.breakpoint_store())
        };
        let subscriptions = vec![
            cx.observe(&dap_store, |row, _, cx| row.refresh(cx)),
            cx.observe(&breakpoint_store, |row, _, cx| row.refresh(cx)),
        ];
        let mut row = Self {
            project,
            paused: None,
            breakpoints: 0,
            sessions: HashMap::default(),
            _subscriptions: subscriptions,
        };
        row.refresh(cx);
        row
    }

    /// Reads the sessions and the breakpoints again, and follows each session for its changes.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let project = self.project.read(cx);
        let sessions: Vec<Entity<Session>> =
            project.dap_store().read(cx).sessions().cloned().collect();
        self.breakpoints = project
            .breakpoint_store()
            .read(cx)
            .all_source_breakpoints(cx)
            .values()
            .map(Vec::len)
            .sum();
        self.sessions
            .retain(|id, _| sessions.iter().any(|session| session.entity_id() == *id));
        for session in &sessions {
            self.sessions.entry(session.entity_id()).or_insert_with(|| {
                [
                    cx.subscribe(session, |row, _, _: &SessionEvent, cx| row.refresh(cx)),
                    cx.subscribe(session, |row, _, _: &SessionStateEvent, cx| row.refresh(cx)),
                ]
            });
        }
        let paused = sessions
            .iter()
            .find(|session| {
                let session = session.read(cx);
                !session.is_terminated() && session.any_stopped_thread()
            })
            .map(|session| paused_in(session, cx));
        self.paused = paused;
        cx.notify();
    }
}

/// Where `session`'s first stopped thread stopped: its top frame, once the adapter sent it.
fn paused_in(session: &Entity<Session>, cx: &mut App) -> Paused {
    session.update(cx, |session, cx| {
        let stopped = session
            .threads(cx)
            .into_iter()
            .find(|(_, status)| *status == ThreadStatus::Stopped)
            .map(|(thread, _)| ThreadId::from(thread.id));
        let top = stopped
            .and_then(|thread| session.stack_frames(thread, cx).ok())
            .and_then(|frames| frames.into_iter().next());
        let Some(top) = top else {
            return Paused {
                place: None,
                frame: None,
                column: None,
            };
        };
        let frame = top.dap;
        let file = frame.source.as_ref().and_then(|source| {
            source
                .path
                .as_deref()
                .and_then(|path| Path::new(path).file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .or_else(|| source.name.clone())
        });
        Paused {
            place: file.map(|file| (file, frame.line)),
            frame: Some(frame.name),
            column: Some(frame.column),
        }
    })
}

/// The debugger's controls, as the board draws them: a glyph and the action's name.
const CONTROLS: [(&str, &str, &str); 5] = [
    ("\u{25b6}", "Continue", "debugger::Continue"),
    ("\u{21b7}", "Step Over", "debugger::StepOver"),
    ("\u{2193}", "Step In", "debugger::StepInto"),
    ("\u{2191}", "Step Out", "debugger::StepOut"),
    ("\u{25a0}", "Stop", "debugger::Stop"),
];

impl Render for DebugRow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(paused) = self.paused.clone() else {
            return div().into_any_element();
        };
        let warning = cx.theme().status().warning;
        let place = paused.place.as_ref().map_or_else(
            || "Paused".to_string(),
            |(file, line)| format!("Paused at {file}:{line}"),
        );
        let controls = CONTROLS.iter().map(|(glyph, name, action)| {
            let action = *action;
            Button::new(
                SharedString::from(format!("marley-debug-row-{name}")),
                *glyph,
            )
            .label_size(LabelSize::Small)
            .tooltip(Tooltip::text(*name))
            .on_click(move |_, window, cx| match cx.build_action(action, None) {
                Ok(action) => window.dispatch_action(action, cx),
                Err(error) => log::error!("the debug row's {action}: {error:#}"),
            })
        });
        let breakpoints = match self.breakpoints {
            1 => "1 breakpoint".to_string(),
            count => format!("{count} breakpoints"),
        };
        h_flex()
            .id("marley-debug-row")
            .debug_selector(|| "marley-debug-row".into())
            .gap_2()
            .child(
                h_flex()
                    .gap_1p5()
                    .px_2()
                    .h_6()
                    .rounded_md()
                    .bg(warning.opacity(0.15))
                    .child(div().size_1p5().rounded_full().bg(warning))
                    .child(
                        Label::new(place)
                            .size(LabelSize::Small)
                            .color(Color::Warning),
                    ),
            )
            .child(h_flex().gap_0p5().children(controls))
            .children(
                paused
                    .frame
                    .map(|frame| Label::new(frame).size(LabelSize::Small).color(Color::Muted)),
            )
            .child(
                Label::new(breakpoints)
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(
                paused
                    .place
                    .as_ref()
                    .zip(paused.column)
                    .map(|((_, line), column)| {
                        Label::new(format!("{line}:{column}"))
                            .size(LabelSize::Small)
                            .buffer_font(cx)
                            .color(Color::Muted)
                    }),
            )
            .into_any_element()
    }
}

impl StatusItemView for DebugRow {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// The row shows only while a session is paused, so it has no switch of its own.
    fn hide_setting(&self, _cx: &App) -> Option<workspace::HideStatusItem> {
        None
    }
}
