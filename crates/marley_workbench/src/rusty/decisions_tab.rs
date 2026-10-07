//! Rusty's brain loop in a center tab (#659): the screen Rusty's app draws as `DecisionsPage.qml`.
//!
//! `brain_due` gives the follow-ups due and every decision; the tab draws the due ones first under
//! Due, then every decision with its status and dates, each row opening its page. It draws what
//! Rusty serves and computes no date. It reads when it opens, when Rusty connects, on Rusty's
//! announcement while it shows (else when it next shows), on Read again after a failure, and after
//! a follow-up recorded from its form (#660).

use gpui::{
    Anchor, AnyElement, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    MouseDownEvent, Pixels, Point, Render, SharedString, Subscription, Task, WeakEntity, Window,
    actions, anchored, deferred,
};
use marley_rusty::decisions::{
    BRAIN_DUE, DecisionStatus, DecisionSummary, Due, Entry, Section, count_line, entries, parse_due,
};
use serde_json::json;
use ui::{Chip, ContextMenu, ListItem, ListItemSpacing, ListSubHeader, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::Toast;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;

use super::follow_up::{self, Candidate};

actions!(
    rusty,
    [
        /// Opens Decisions: Rusty's brain loop, the follow-ups due first, then every decision
        /// with its status and dates.
        #[derive(Eq)]
        OpenDecisions,
    ]
);

/// Rusty's own line on the loop, under the title.
const LOOP_LINE: &str = "brain_ask before a choice, brain_decide when it is made, \
     brain_follow_up when its day comes.";

/// Registers `rusty: open decisions` on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|_, _: &OpenDecisions, window, cx| {
            open_later(cx.weak_entity(), window, cx);
        });
    })
    .detach();
}

/// Opens the Decisions tab once the update in progress ends; the rail's Brain view calls it.
pub(crate) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    super::in_rusty_group(workspace, window, cx, |workspace, window, cx| {
        open(workspace, window, cx);
    });
}

/// Brings the workspace's Decisions tab forward with the focus, or adds one to the active pane;
/// with Rusty off, says so and opens nothing.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        workspace.show_toast(
            Toast::new(
                NotificationId::unique::<BrainDecisionsView>(),
                reason.to_string(),
            ),
            cx,
        );
        return;
    }
    let open = workspace.items_of_type::<BrainDecisionsView>(cx).next();
    if let Some(view) = open {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let weak = cx.entity().downgrade();
    let view = cx.new(|cx| BrainDecisionsView::new(weak, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// Marley's link to Rusty, as the tab last saw it (L-658); the Memory tab's too (#664).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Link {
    Off,
    Down,
    Up,
}

impl Link {
    pub(super) fn now(cx: &App) -> Self {
        if !super::is_on(cx) {
            Self::Off
        } else if super::is_connected(cx) {
            Self::Up
        } else {
            Self::Down
        }
    }
}

/// A read the tab owes: none, one more after the one running, or one when it next shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadDue {
    No,
    AfterThis,
    WhenShown,
}

/// The Decisions tab.
pub(crate) struct BrainDecisionsView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// The last answer drawn; none before the first.
    listed: Option<Due>,
    /// A failed read's words, shown with Read again over the list kept.
    failure: Option<SharedString>,
    reading: Option<Task<()>>,
    due: ReadDue,
    link: Link,
    menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    _subscriptions: Vec<Subscription>,
}

impl BrainDecisionsView {
    fn new(workspace: WeakEntity<Workspace>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
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
        let mut view = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            listed: None,
            failure: None,
            reading: None,
            due: ReadDue::No,
            link: Link::now(cx),
            menu: None,
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

    /// Off drops the list and calls nothing; the link coming up reads. The global changes for
    /// more than the link, so only a change of the link counts (L-658).
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

    /// Reads `brain_due`; with one read running, one more after it.
    fn read(&mut self, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        if self.reading.is_some() {
            self.due = ReadDue::AfterThis;
            return;
        }
        self.due = ReadDue::No;
        let asking = super::call_tool(BRAIN_DUE, json!({ "days": 0 }), cx);
        self.reading = Some(cx.spawn(async move |this, cx| {
            let read = match asking.await {
                Ok(text) => cx
                    .background_spawn(futures::future::lazy(move |_| parse_due(&text)))
                    .await
                    .map_err(|error| format!("{BRAIN_DUE}'s answer did not parse: {error}")),
                Err(error) => Err(error),
            };
            this.update(cx, |this, cx| this.take_read(read, cx))
                .log_err();
        }));
    }

    /// Reads again after the follow-up form recorded one: with the service connection Rusty
    /// announces nothing (#647's D9).
    pub(super) fn read_again(&mut self, cx: &mut Context<Self>) {
        self.read(cx);
        cx.notify();
    }

    fn take_read(&mut self, read: Result<Due, String>, cx: &mut Context<Self>) {
        self.reading = None;
        match read {
            Ok(due) => {
                self.listed = Some(due);
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

    fn summary(&self, section: Section, index: usize) -> Option<&DecisionSummary> {
        let listed = self.listed.as_ref()?;
        match section {
            Section::Due => listed.due.get(index),
            Section::All => listed.all.get(index),
        }
    }

    fn render_row(&self, section: Section, index: usize, cx: &Context<Self>) -> Option<AnyElement> {
        let summary = self.summary(section, index)?;
        let slug = summary.slug.clone();
        let title = shown_title(summary);
        let id = SharedString::from(format!(
            "rusty-decision-{}-{}",
            section_word(section),
            summary.slug
        ));
        let tooltip = format!("{title}\n{slug}");
        let superseded = summary.status() == DecisionStatus::Superseded;
        Some(
            ListItem::new(id)
                .spacing(ListItemSpacing::Sparse)
                .tooltip(Tooltip::text(tooltip))
                .child(div().flex_1().min_w_0().child(Label::new(title).truncate()))
                .end_slot(self.render_end_slot(summary, section, index, cx))
                .on_click(cx.listener(move |this, _, window, cx| {
                    super::page::open_later(
                        this.workspace.clone(),
                        slug.clone(),
                        false,
                        true,
                        window,
                        cx,
                    );
                }))
                .on_secondary_mouse_down(cx.listener(
                    move |this, event: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        window.prevent_default();
                        // A replaced decision's next step is its successor's (D10).
                        let mut choices = Vec::new();
                        if !superseded && super::is_connected(cx) {
                            choices.push(("Follow Up…", MenuChoice::FollowUp(section, index)));
                        }
                        choices.push(("Open Page", MenuChoice::OpenPage(section, index)));
                        this.deploy_menu(event.position, choices, window, cx);
                    },
                ))
                .into_any_element(),
        )
    }

    /// A row's end: the status chip, its days as Rusty serves them, the successor of a replaced
    /// decision, and Follow Up on a Due row.
    fn render_end_slot(
        &self,
        summary: &DecisionSummary,
        section: Section,
        index: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let status = summary.status();
        let muted = |line: String| Label::new(line).size(LabelSize::Small).color(Color::Muted);
        let follow_up = summary.follow_up_line().map(|line| {
            Label::new(line)
                .size(LabelSize::Small)
                .color(if summary.overdue {
                    Color::Warning
                } else {
                    Color::Muted
                })
        });
        let successor = summary
            .successor()
            .filter(|_| status == DecisionStatus::Superseded)
            .map(|successor| self.render_successor(&summary.slug, successor, cx));
        let follow_up_button = (section == Section::Due
            && status != DecisionStatus::Superseded
            && super::is_connected(cx))
        .then(|| {
            Button::new(
                SharedString::from(format!("rusty-decision-follow-up-{}", summary.slug)),
                "Follow Up",
            )
            .label_size(LabelSize::Small)
            .on_click(cx.listener(move |this, _, window, cx| {
                this.open_follow_up(section, index, window, cx);
            }))
        });
        h_flex()
            .gap_3()
            .child(Chip::new(status.word().to_string()).label_color(
                if status == DecisionStatus::Superseded {
                    Color::Muted
                } else {
                    Color::Default
                },
            ))
            .children(summary.decided_line().map(muted))
            .children(summary.followed_up_line().map(muted))
            .children(follow_up)
            .children(successor)
            .children(follow_up_button)
    }

    /// "replaced by" and the successor's title, which opens its page and not the row's.
    fn render_successor(&self, slug: &str, successor: &str, cx: &Context<Self>) -> AnyElement {
        let title = self
            .listed
            .as_ref()
            .and_then(|listed| listed.all.iter().find(|each| each.slug == successor))
            .map_or_else(|| successor.to_string(), shown_title);
        let successor = successor.to_string();
        h_flex()
            .gap_1()
            .child(
                Label::new("replaced by")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                Button::new(
                    SharedString::from(format!("rusty-decision-successor-{slug}")),
                    title,
                )
                .label_size(LabelSize::Small)
                .color(Color::Accent)
                .on_click(cx.listener(move |this, _, window, cx| {
                    super::page::open_later(
                        this.workspace.clone(),
                        successor.clone(),
                        false,
                        true,
                        window,
                        cx,
                    );
                })),
            )
            .into_any_element()
    }

    /// Opens the follow-up form for a row, with every other decision as a possible successor.
    fn open_follow_up(
        &self,
        section: Section,
        index: usize,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(summary) = self.summary(section, index).cloned() else {
            return;
        };
        let candidates = self
            .listed
            .iter()
            .flat_map(|listed| listed.all.iter())
            .filter(|each| each.slug != summary.slug)
            .map(|each| Candidate {
                slug: each.slug.clone(),
                title: SharedString::from(shown_title(each)),
            })
            .collect();
        let tab = cx.entity().downgrade();
        let workspace = self.workspace.clone();
        // The form opens in the workspace's modal layer, after the update this click runs in.
        window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    follow_up::open(workspace, tab, &summary, candidates, window, cx);
                })
                .log_err();
        });
    }

    /// A right-click menu, deployed by hand at the pointer, as the Tasks tab's.
    fn deploy_menu(
        &mut self,
        position: Point<Pixels>,
        choices: Vec<(&'static str, MenuChoice)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        let menu = ContextMenu::build(window, cx, move |mut menu, _, _| {
            for (label, choice) in choices {
                let view = view.clone();
                menu = menu.entry(label, None, move |window, cx| {
                    view.update(cx, |this, cx| this.chose(choice, window, cx))
                        .log_err();
                });
            }
            menu
        });
        let subscription = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, _, cx| {
            this.menu = None;
            cx.notify();
        });
        window.focus(&menu.focus_handle(cx), cx);
        self.menu = Some((menu, position, subscription));
        cx.notify();
    }

    fn chose(&self, choice: MenuChoice, window: &Window, cx: &mut Context<Self>) {
        match choice {
            MenuChoice::FollowUp(section, index) => self.open_follow_up(section, index, window, cx),
            MenuChoice::OpenPage(section, index) => {
                if let Some(summary) = self.summary(section, index) {
                    super::page::open_later(
                        self.workspace.clone(),
                        summary.slug.clone(),
                        false,
                        true,
                        window,
                        cx,
                    );
                }
            }
        }
    }

    /// The lines over the list: Rusty off, not connected, reading, a failure, or none yet.
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
                        Button::new("rusty-decisions-read-again", "Read again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.read(cx);
                                cx.notify();
                            })),
                    )
                    .into_any_element(),
            );
        }
        match &self.listed {
            None => Some(line("Reading Rusty's decisions…", Color::Muted)),
            Some(listed) if listed.all.is_empty() && listed.due.is_empty() => Some(line(
                "No decisions yet. brain_ask, then brain_decide, writes the first one.",
                Color::Muted,
            )),
            Some(_) => None,
        }
    }
}

/// A right-click menu's entry, by what it does to which row.
#[derive(Clone, Copy, Debug)]
enum MenuChoice {
    FollowUp(Section, usize),
    OpenPage(Section, usize),
}

/// A decision's title, or its slug when it has none.
fn shown_title(summary: &DecisionSummary) -> String {
    if summary.title.is_empty() {
        summary.slug.clone()
    } else {
        summary.title.clone()
    }
}

const fn section_word(section: Section) -> &'static str {
    match section {
        Section::Due => "due",
        Section::All => "all",
    }
}

/// A section's header. `ListSubHeader` grows in a column, so it sits in a box that does not.
fn header(label: impl Into<SharedString>) -> AnyElement {
    div()
        .flex_none()
        .child(ListSubHeader::new(label))
        .into_any_element()
}

fn line(text: impl Into<SharedString>, color: Color) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(color)
        .into_any_element()
}

impl Focusable for BrainDecisionsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for BrainDecisionsView {}

impl Render for BrainDecisionsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A change announced while the tab was hidden is read now that it draws (AD-609).
        if self.due == ReadDue::WhenShown && self.reading.is_none() {
            self.read(cx);
        }
        let state = self.render_state(cx);
        let rows: Vec<AnyElement> = match (&self.listed, super::is_on(cx)) {
            (Some(listed), true) if !(listed.all.is_empty() && listed.due.is_empty()) => {
                entries(listed)
                    .into_iter()
                    .filter_map(|entry| match entry {
                        Entry::DueHeader => Some(header("Due")),
                        Entry::AllHeader { count } => Some(header(count_line(count))),
                        Entry::Row { section, index } => self.render_row(section, index, cx),
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        v_flex()
            .key_context("RustyDecisions")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .px_4()
                    .pt_3()
                    .pb_2()
                    .gap_1()
                    .child(Label::new("Decisions").size(LabelSize::Large))
                    .child(
                        Label::new(LOOP_LINE)
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .children(state),
            )
            .child(
                v_flex()
                    .id("rusty-decisions")
                    .flex_1()
                    .min_h_0()
                    .px_2()
                    .overflow_y_scroll()
                    .children(rows),
            )
            .children(self.menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Anchor::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
    }
}

impl Item for BrainDecisionsView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Decisions")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::CheckDouble))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Rusty's decisions and follow-ups"))
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
