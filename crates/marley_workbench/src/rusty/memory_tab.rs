//! Rusty's long-term memories in a center tab (#664): the screen Rusty's app draws as
//! `MemoryPage.qml`.
//!
//! `list_memories` gives them high importance first, newest first within each, and the tab draws
//! that order. A field at the top stores one; the Category menu shows one category; a click on a
//! row opens a form that saves or deletes it. The tab reads when it opens, when Rusty connects, on
//! Rusty's announcement while it shows (else when it next shows), after each of its own writes and
//! on Read again after a failure.

use std::hash::{DefaultHasher, Hash, Hasher};

use chrono::{DateTime, Local};
use editor::{Editor, EditorEvent};
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, PromptLevel, Render,
    SharedString, Subscription, Task, WeakEntity, Window, actions,
};
use marley_rusty::memories::{
    Importance, LIST_MEMORIES, Memory, MemoryWrite, categories, count_line, memories_from_answer,
};
use serde_json::json;
use ui::{
    Chip, ContextMenu, DropdownMenu, DropdownStyle, ListItem, ListItemSpacing, ToggleButtonGroup,
    ToggleButtonGroupStyle, ToggleButtonSimple, prelude::*,
};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

use super::decisions_tab::{Link, ReadDue};

actions!(
    rusty,
    [
        /// Opens Memory: what Rusty remembers across conversations, to add to, edit and delete.
        #[derive(Eq)]
        OpenMemory,
    ]
);

/// Registers `rusty: open memory` on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|_, _: &OpenMemory, window, cx| {
            open_later(cx.weak_entity(), window, cx);
        });
    })
    .detach();
}

/// Opens the Memory tab once the update in progress ends; the rail's Brain view calls it.
pub(super) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    super::in_rusty_group(workspace, window, cx, |workspace, window, cx| {
        open(workspace, window, cx);
    });
}

/// Brings the workspace's Memory tab forward with the focus, or adds one to the active pane; with
/// Rusty off, says so and opens nothing.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        workspace.show_toast(
            Toast::new(
                NotificationId::unique::<BrainMemoryView>(),
                reason.to_string(),
            ),
            cx,
        );
        return;
    }
    let open = workspace.items_of_type::<BrainMemoryView>(cx).next();
    if let Some(view) = open {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let weak = cx.entity().downgrade();
    let view = cx.new(|cx| BrainMemoryView::new(weak, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// The Memory tab.
pub(super) struct BrainMemoryView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// The last answer drawn; none before the first.
    listed: Option<Vec<Memory>>,
    /// A failed read's words, shown with Read again over the list kept.
    failure: Option<SharedString>,
    reading: Option<Task<()>>,
    due: ReadDue,
    link: Link,
    /// The category shown; none shows every memory.
    filter: Option<String>,
    add_content: Entity<Editor>,
    add_category: Entity<Editor>,
    add_importance: Importance,
    adding: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl BrainMemoryView {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe_global::<super::Announced>(|this, cx| {
                if this.showing(cx) {
                    this.read(cx);
                } else if this.due == ReadDue::No {
                    this.due = ReadDue::WhenShown;
                }
            }),
            cx.observe_global::<super::Rusty>(Self::rusty_changed),
        ];
        let add_content = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Remember something and press Enter", window, cx);
            editor
        });
        let add_category = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("context", window, cx);
            editor
        });
        let mut view = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            listed: None,
            failure: None,
            reading: None,
            due: ReadDue::No,
            link: Link::now(cx),
            filter: None,
            add_content,
            add_category,
            add_importance: Importance::Normal,
            adding: None,
            _subscriptions: subscriptions,
        };
        view.read(cx);
        view
    }

    /// Whether the tab is its pane's active item, asked each time (PR-607).
    fn showing(&self, cx: &Context<Self>) -> bool {
        let Some(workspace) = self.workspace.upgrade() else {
            return false;
        };
        let id = cx.entity_id();
        workspace.read(cx).panes().iter().any(|pane| {
            pane.read(cx)
                .active_item()
                .is_some_and(|item| item.item_id() == id)
        })
    }

    /// Off drops the list and calls nothing; the link coming up reads (L-658).
    fn rusty_changed(&mut self, cx: &mut Context<Self>) {
        let link = Link::now(cx);
        if link == self.link {
            return;
        }
        self.link = link;
        match link {
            Link::Off => {
                self.listed = None;
                self.failure = None;
                self.reading = None;
                self.due = ReadDue::No;
            }
            Link::Up => self.read(cx),
            Link::Down => {}
        }
        cx.notify();
    }

    /// Reads `list_memories`; with one read running, one more after it.
    fn read(&mut self, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        if self.reading.is_some() {
            self.due = ReadDue::AfterThis;
            return;
        }
        self.due = ReadDue::No;
        let asking = super::call_tool(LIST_MEMORIES, json!({}), cx);
        self.reading = Some(cx.spawn(async move |this, cx| {
            let read = asking.await.and_then(|text| {
                memories_from_answer(&text)
                    .map_err(|error| format!("{LIST_MEMORIES}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| this.take_read(read, cx))
                .log_err();
        }));
    }

    /// Reads again after a write: with the service connection Rusty announces nothing (AD-662).
    fn read_again(&mut self, cx: &mut Context<Self>) {
        self.read(cx);
        cx.notify();
    }

    fn take_read(&mut self, read: Result<Vec<Memory>, String>, cx: &mut Context<Self>) {
        self.reading = None;
        match read {
            Ok(memories) => {
                // A category that went away shows every memory again.
                if let Some(filter) = &self.filter
                    && !memories.iter().any(|memory| &memory.category == filter)
                {
                    self.filter = None;
                }
                self.listed = Some(memories);
                self.failure = None;
            }
            Err(error) => {
                let line = error.lines().next().unwrap_or_default().to_string();
                self.failure = Some(line.into());
            }
        }
        if self.due == ReadDue::AfterThis {
            self.read(cx);
        }
        cx.notify();
    }

    /// The memories the filter shows, in Rusty's order.
    fn shown(&self) -> Vec<&Memory> {
        self.listed
            .iter()
            .flatten()
            .filter(|memory| {
                self.filter
                    .as_ref()
                    .is_none_or(|category| &memory.category == category)
            })
            .collect()
    }

    /// Stores what the add field holds, with the category typed and the importance chosen.
    fn add(&mut self, window: &Window, cx: &Context<Self>) {
        let content = self.add_content.read(cx).text(cx);
        if content.trim().is_empty() || self.adding.is_some() {
            return;
        }
        let write = MemoryWrite::Store {
            content,
            category: self.add_category.read(cx).text(cx),
            importance: self.add_importance,
        };
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.adding = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update_in(cx, |this, window, cx| {
                this.adding = None;
                match answer {
                    Ok(_) => {
                        this.add_content
                            .update(cx, |editor, cx| editor.set_text("", window, cx));
                        this.read_again(cx);
                    }
                    Err(error) => this.toast(format!("Could not remember it: {error}"), cx),
                }
            })
            .log_err();
        }));
    }

    fn toast(&self, message: String, cx: &mut App) {
        self.workspace
            .update(cx, |workspace, cx| {
                workspace.show_toast(Toast::new(NotificationId::unique::<Self>(), message), cx);
            })
            .log_err();
    }

    fn open_form(&self, memory: &Memory, window: &Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let tab = cx.entity().downgrade();
        let memory = memory.clone();
        // The form opens in the workspace's modal layer, outside this tab's update.
        window.defer(cx, move |window, cx| {
            workspace.update(cx, |workspace, cx| {
                workspace.toggle_modal(window, cx, |window, cx| {
                    MemoryForm::new(tab, &memory, window, cx)
                });
            });
        });
    }

    fn render_add(&self, cx: &Context<Self>) -> impl IntoElement {
        let field = |editor: &Entity<Editor>| {
            div()
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().colors().border)
                .child(editor.clone())
        };
        let buttons = Importance::ALL.map(|importance| {
            ToggleButtonSimple::new(
                importance.label(),
                cx.listener(move |this, _, _, cx| {
                    this.add_importance = importance;
                    cx.notify();
                }),
            )
        });
        let chosen = Importance::ALL
            .iter()
            .position(|each| *each == self.add_importance)
            .unwrap_or_default();
        h_flex()
            .key_context("RustyMemoryAdd menu")
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.add(window, cx)))
            .gap_2()
            .child(div().flex_1().min_w_0().child(field(&self.add_content)))
            .child(
                div()
                    .flex_none()
                    .w(rems(10.))
                    .child(field(&self.add_category)),
            )
            // The group fills its parent, so it gets a width of its own and the field the rest.
            .child(
                div().flex_none().w(rems(16.)).child(
                    ToggleButtonGroup::single_row("rusty-memory-add-importance", buttons)
                        .style(ToggleButtonGroupStyle::Outlined)
                        .selected_index(chosen),
                ),
            )
    }

    /// The count of the memories shown, and the Category menu.
    fn render_filter(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let all = categories(self.listed.as_deref().unwrap_or_default());
        let filter = self.filter.clone();
        let mut key = DefaultHasher::new();
        (&all, &filter).hash(&mut key);
        let view = cx.entity().downgrade();
        let menu = window.use_keyed_state(
            ("rusty-memory-filter", key.finish()),
            cx,
            move |window, cx| {
                ContextMenu::new(window, cx, move |mut menu, _, _| {
                    let choices = std::iter::once(None).chain(all.iter().cloned().map(Some));
                    for choice in choices {
                        let label = choice.clone().unwrap_or_else(|| "All".to_string());
                        let view = view.clone();
                        menu = menu.toggleable_entry(
                            label,
                            choice == filter,
                            IconPosition::End,
                            None,
                            move |_, cx| {
                                let choice = choice.clone();
                                view.update(cx, |this, cx| {
                                    this.filter = choice;
                                    cx.notify();
                                })
                                .log_err();
                            },
                        );
                    }
                    menu
                })
            },
        );
        h_flex()
            .gap_2()
            .justify_between()
            .child(
                Label::new(count_line(self.shown().len()))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                DropdownMenu::new(
                    "rusty-memory-category",
                    self.filter.clone().map_or_else(
                        || "Category: All".to_string(),
                        |category| format!("Category: {category}"),
                    ),
                    menu,
                )
                .style(DropdownStyle::Outlined),
            )
    }

    fn render_row(index: usize, memory: &Memory, cx: &Context<Self>) -> AnyElement {
        let meta = [
            memory.importance.clone(),
            memory.source.clone(),
            day(memory.updated_at),
        ]
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        let row = memory.clone();
        ListItem::new(("rusty-memory", index))
            .spacing(ListItemSpacing::Sparse)
            .child(
                v_flex()
                    .min_w_0()
                    .gap_1()
                    .py_1()
                    .child(Label::new(memory.content.clone()).line_clamp(3))
                    .child(
                        h_flex()
                            .gap_2()
                            .when(!memory.category.is_empty(), |line| {
                                line.child(Chip::new(memory.category.clone()))
                            })
                            .child(Label::new(meta).size(LabelSize::Small).color(Color::Muted)),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| this.open_form(&row, window, cx)))
            .into_any_element()
    }

    fn render_state(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if !super::is_on(cx) {
            return Some(line(
                "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
                Color::Muted,
            ));
        }
        if !super::is_connected(cx) {
            let reason = super::unavailable(cx).unwrap_or_default();
            return Some(line(
                format!("Marley is not connected to Rusty: {reason}"),
                Color::Muted,
            ));
        }
        if let Some(failure) = &self.failure {
            return Some(
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(failure.clone())
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    )
                    .child(
                        Button::new("rusty-memory-read-again", "Read again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.read_again(cx))),
                    )
                    .into_any_element(),
            );
        }
        match &self.listed {
            None => Some(line("Reading Rusty's memories…", Color::Muted)),
            Some(listed) if listed.is_empty() => Some(line(
                "Nothing remembered yet. Type something above and press Enter.",
                Color::Muted,
            )),
            Some(_) => None,
        }
    }
}

/// The day a memory last changed, in this machine's time zone.
fn day(seconds: i64) -> String {
    DateTime::from_timestamp(seconds, 0)
        .filter(|_| seconds > 0)
        .map(|time| time.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn line(text: impl Into<SharedString>, color: Color) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(color)
        .into_any_element()
}

impl Focusable for BrainMemoryView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for BrainMemoryView {}

impl Render for BrainMemoryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A change announced while the tab was hidden is read now that it draws (AD-609).
        if self.due == ReadDue::WhenShown && self.reading.is_none() {
            self.read(cx);
        }
        let state = self.render_state(cx);
        let listing = super::is_on(cx) && self.listed.as_ref().is_some_and(|list| !list.is_empty());
        let rows: Vec<AnyElement> = if listing {
            self.shown()
                .into_iter()
                .enumerate()
                .map(|(index, memory)| Self::render_row(index, memory, cx))
                .collect()
        } else {
            Vec::new()
        };
        let connected = super::is_on(cx) && super::is_connected(cx);
        v_flex()
            .key_context("RustyMemory")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .px_4()
                    .pt_3()
                    .pb_2()
                    .gap_2()
                    .child(Label::new("Memory").size(LabelSize::Large))
                    .child(
                        Label::new(
                            "What Rusty remembers across conversations; its agents read the \
                             high ones first.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    )
                    .when(connected, |header| header.child(self.render_add(cx)))
                    .children(state)
                    .when(listing, |header| {
                        header.child(self.render_filter(window, cx))
                    }),
            )
            .child(
                v_flex()
                    .id("rusty-memories")
                    .flex_1()
                    .min_h_0()
                    .px_2()
                    .overflow_y_scroll()
                    .children(rows),
            )
    }
}

impl Item for BrainMemoryView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Memory")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Book))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("What Rusty remembers"))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    /// With the service connection no change is announced, so each showing reads (#647's D9).
    fn deactivated(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            cx.global::<super::Rusty>().source,
            super::Source::Service(_)
        ) {
            self.due = ReadDue::WhenShown;
        }
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // L-613: an item moved to another workspace follows it.
        self.workspace = workspace.weak_handle();
    }
}

/// The form a row opens: the memory's content, category and importance, saved or deleted.
pub(super) struct MemoryForm {
    tab: WeakEntity<BrainMemoryView>,
    id: String,
    content: Entity<Editor>,
    category: Entity<Editor>,
    /// None while the memory holds a word other than Rusty's three and none is picked.
    importance: Option<Importance>,
    sending: Option<Task<()>>,
    /// Rusty's words when it refused the last write.
    refusal: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

/// The form's width in rems, #660's form's.
const FORM_WIDTH: f32 = 34.;

impl MemoryForm {
    fn new(
        tab: WeakEntity<BrainMemoryView>,
        memory: &Memory,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = cx.new(|cx| {
            let mut editor = Editor::auto_height(3, 10, window, cx);
            editor.set_text(memory.content.clone(), window, cx);
            editor
        });
        let category = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_text(memory.category.clone(), window, cx);
            editor.set_placeholder_text("context", window, cx);
            editor
        });
        // Save's state follows the content's words, not its caret's blink.
        let subscriptions = vec![cx.subscribe(&content, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        })];
        Self {
            tab,
            id: memory.id.clone(),
            content,
            category,
            importance: memory.importance(),
            sending: None,
            refusal: None,
            _subscriptions: subscriptions,
        }
    }

    fn content_empty(&self, cx: &App) -> bool {
        self.content.read(cx).text(cx).trim().is_empty()
    }

    fn save(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.sending.is_some() || self.content_empty(cx) {
            return;
        }
        let write = MemoryWrite::Update {
            id: self.id.clone(),
            content: self.content.read(cx).text(cx),
            category: self.category.read(cx).text(cx),
            importance: self.importance,
        };
        self.send(&write, window, cx);
    }

    /// Asks first, naming the memory, then deletes it.
    fn delete(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sending.is_some() {
            return;
        }
        let text = self.content.read(cx).text(cx);
        let first = text.lines().next().unwrap_or_default().to_string();
        let answer = window.prompt(
            PromptLevel::Warning,
            "Forget this memory?",
            Some(&first),
            &["Delete", "Cancel"],
            cx,
        );
        let id = self.id.clone();
        cx.spawn_in(window, async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            this.update_in(cx, |this, window, cx| {
                this.send(&MemoryWrite::Delete { id }, window, cx);
            })
            .log_err();
        })
        .detach();
    }

    fn send(&mut self, write: &MemoryWrite, window: &Window, cx: &mut Context<Self>) {
        self.refusal = None;
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.sending = Some(cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await;
            this.update(cx, |this, cx| {
                this.sending = None;
                match answer {
                    Ok(_) => {
                        this.tab.update(cx, BrainMemoryView::read_again).log_err();
                        cx.emit(DismissEvent);
                    }
                    Err(error) => {
                        let line = error.lines().next().unwrap_or_default().to_string();
                        this.refusal = Some(line.into());
                    }
                }
                cx.notify();
            })
            .log_err();
        }));
        cx.notify();
    }

    fn render_importance(&self, cx: &Context<Self>) -> impl IntoElement {
        let buttons = Importance::ALL.map(|importance| {
            ToggleButtonSimple::new(
                importance.label(),
                cx.listener(move |this, _, _, cx| {
                    this.importance = Some(importance);
                    cx.notify();
                }),
            )
        });
        let chosen = self
            .importance
            .and_then(|importance| Importance::ALL.iter().position(|each| *each == importance))
            // An index past the three lights none (#660's D3).
            .unwrap_or(Importance::ALL.len());
        ToggleButtonGroup::single_row("rusty-memory-importance", buttons)
            .style(ToggleButtonGroupStyle::Outlined)
            .selected_index(chosen)
    }
}

fn field_label(text: &'static str) -> Label {
    Label::new(text).size(LabelSize::Small).color(Color::Muted)
}

impl ModalView for MemoryForm {}

impl EventEmitter<DismissEvent> for MemoryForm {}

impl Focusable for MemoryForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.content.focus_handle(cx)
    }
}

impl Render for MemoryForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sending = self.sending.is_some();
        let field = |editor: &Entity<Editor>| {
            div()
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(cx.theme().colors().border)
                .child(editor.clone())
        };
        v_flex()
            .key_context("RustyMemoryForm menu")
            .w(rems(FORM_WIDTH))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .child(Label::new("Edit Memory"))
            .child(field_label("Content"))
            .child(field(&self.content))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_1()
                            .child(field_label("Category"))
                            .child(field(&self.category)),
                    )
                    .child(
                        v_flex()
                            .flex_none()
                            .w(rems(14.))
                            .gap_1()
                            .child(field_label("Importance"))
                            .child(self.render_importance(cx)),
                    ),
            )
            .children(self.refusal.clone().map(|refusal| {
                Label::new(refusal)
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }))
            .child(
                h_flex()
                    .gap_2()
                    .justify_between()
                    .child(
                        Button::new("rusty-memory-delete", "Delete")
                            .style(ButtonStyle::Subtle)
                            .disabled(sending)
                            .on_click(cx.listener(|this, _, window, cx| this.delete(window, cx))),
                    )
                    .child(
                        Button::new(
                            "rusty-memory-save",
                            if sending { "Saving…" } else { "Save" },
                        )
                        .style(ButtonStyle::Filled)
                        .disabled(sending || self.content_empty(cx))
                        .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
