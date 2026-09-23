//! The rail's switcher: the window's terminals and threads, the most recently shown first. The
//! rail opens it on `ctrl-tab` in the sidebar overlay, as Zed's sidebar opens its thread
//! switcher. Each further press steps down the list, and letting go of the modifiers held to open
//! it opens the selected row.

use gpui::{
    AnyElement, ClickEvent, Context, EventEmitter, FocusHandle, Focusable, Modifiers,
    ModifiersChangedEvent, Render, ScrollHandle, Subscription, Window,
};
use marley_rail::{Selection, TerminalRow, ThreadRow};
use menu::{Cancel, Confirm};
use ui::{Icon, IconSize, Label, LabelSize, ListItem, prelude::*};
use zed_actions::agents_sidebar::ToggleThreadSwitcher;

use super::{terminal_icon, thread_item};
use crate::agents::AgentIcon;

/// A row of the switcher, with what it draws beside the rail's row.
#[derive(Debug, Clone)]
pub(super) enum SwitcherEntry {
    /// A terminal, and its project's name.
    Terminal {
        row: TerminalRow,
        project: SharedString,
    },
    /// A thread, its project's name and its agent's icon.
    Thread {
        row: ThreadRow,
        project: SharedString,
        icon: AgentIcon,
    },
}

impl SwitcherEntry {
    fn selection(&self) -> Selection {
        match self {
            Self::Terminal { row, .. } => Selection::Terminal(row.id),
            Self::Thread { row, .. } => Selection::Thread(row.key.clone()),
        }
    }
}

/// How the switcher closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SwitcherEvent {
    /// The row to open.
    Confirmed(Selection),
    /// Nothing to open. After Escape, focus goes back where it was; after focus moved away, it
    /// stays where it went.
    Cancelled { restore_focus: bool },
}

pub(super) struct RailSwitcher {
    entries: Vec<SwitcherEntry>,
    selected: usize,
    /// The modifiers held as it opened, whose release opens the selection. `None` when it opened
    /// with none held, from the command palette say, and once it has confirmed.
    opened_with: Option<Modifiers>,
    /// Whether it has said how it closed: a confirm moves focus away, which would cancel too.
    ended: bool,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    _focus_out: Subscription,
}

impl RailSwitcher {
    /// A switcher over `entries` on the second, the row shown before the current one, or on the
    /// last for `select_last`.
    pub(super) fn new(
        entries: Vec<SwitcherEntry>,
        select_last: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let last = entries.len().saturating_sub(1);
        let selected = if select_last { last } else { last.min(1) };
        let focus_handle = cx.focus_handle();
        let focus_out = cx.on_focus_out(&focus_handle, window, |switcher, _, _, cx| {
            switcher.end(
                SwitcherEvent::Cancelled {
                    restore_focus: false,
                },
                cx,
            );
        });
        let scroll_handle = ScrollHandle::new();
        scroll_handle.scroll_to_item(selected);
        Self {
            entries,
            selected,
            opened_with: window.modifiers().modified().then_some(window.modifiers()),
            ended: false,
            focus_handle,
            scroll_handle,
            _focus_out: focus_out,
        }
    }

    /// Moves the selection one down, or one up, wrapping at the ends.
    pub(super) fn step(&mut self, forward: bool, cx: &mut Context<Self>) {
        let count = self.entries.len();
        let next = if forward {
            self.selected + 1
        } else {
            self.selected + count.saturating_sub(1)
        };
        self.selected = next.checked_rem(count).unwrap_or(0);
        self.scroll_handle.scroll_to_item(self.selected);
        cx.notify();
    }

    fn toggle(&mut self, action: &ToggleThreadSwitcher, _: &mut Window, cx: &mut Context<Self>) {
        self.step(!action.select_last, cx);
    }

    fn confirm(&mut self, _: &Confirm, _: &mut Window, cx: &mut Context<Self>) {
        self.confirm_selected(cx);
    }

    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        self.end(
            SwitcherEvent::Cancelled {
                restore_focus: true,
            },
            cx,
        );
    }

    /// Letting go of any of the modifiers held to open it opens the selection.
    fn modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(opened_with) = self.opened_with
            && !opened_with.is_subset_of(&event.modifiers)
        {
            self.opened_with = None;
            self.confirm_selected(cx);
        }
    }

    fn click(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = index;
        self.confirm_selected(cx);
    }

    fn confirm_selected(&mut self, cx: &mut Context<Self>) {
        let event = self.entries.get(self.selected).map_or(
            SwitcherEvent::Cancelled {
                restore_focus: true,
            },
            |entry| SwitcherEvent::Confirmed(entry.selection()),
        );
        self.end(event, cx);
    }

    fn end(&mut self, event: SwitcherEvent, cx: &mut Context<Self>) {
        if !self.ended {
            self.ended = true;
            cx.emit(event);
        }
    }

    fn render_entry(&self, index: usize, entry: &SwitcherEntry, cx: &Context<Self>) -> AnyElement {
        let selected = index == self.selected;
        let row = match entry {
            SwitcherEntry::Terminal { row, project } => {
                ListItem::new(("marley-switcher-terminal", index))
                    .toggle_state(selected)
                    .rounded()
                    .start_slot(
                        Icon::new(terminal_icon(row))
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Label::new(row.title.clone()).size(LabelSize::Small))
                            .child(
                                Label::new(project.clone())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .into_any_element()
            }
            SwitcherEntry::Thread { row, project, icon } => thread_item(
                SharedString::from(format!("marley-switcher-thread-{}", row.key)),
                row.clone(),
                icon,
            )
            .selected(selected)
            .project_name(project.clone())
            .base_bg(cx.theme().colors().elevated_surface_background)
            .into_any_element(),
        };
        div()
            .id(("marley-switcher-entry", index))
            .debug_selector(move || format!("marley-switcher-entry-{index}"))
            .on_click(cx.listener(move |switcher, _: &ClickEvent, _, cx| {
                switcher.click(index, cx);
            }))
            .child(row)
            .into_any_element()
    }
}

#[cfg(test)]
impl RailSwitcher {
    /// The rows it lists, in its order.
    pub(super) fn listed(&self) -> Vec<Selection> {
        self.entries.iter().map(SwitcherEntry::selection).collect()
    }

    pub(super) const fn selected_index(&self) -> usize {
        self.selected
    }
}

impl EventEmitter<SwitcherEvent> for RailSwitcher {}

impl Focusable for RailSwitcher {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for RailSwitcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<AnyElement> = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| self.render_entry(index, entry, cx))
            .collect();
        v_flex()
            // Zed's thread switcher's context, so Zed's `ctrl-tab` bindings step this one too.
            .key_context("ThreadSwitcher")
            .track_focus(&self.focus_handle)
            .on_modifiers_changed(cx.listener(Self::modifiers_changed))
            .on_action(cx.listener(Self::toggle))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .w(rems(28.))
            .p_1()
            .elevation_3(cx)
            .child(
                v_flex()
                    .id("marley-switcher-entries")
                    .max_h(rems(30.))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .gap_px()
                    .children(rows),
            )
    }
}
