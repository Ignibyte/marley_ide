//! The start of the bottom row says what has the focus (#833).
//!
//! A file shows its type and path; a terminal its title, folder, last command and its exit,
//! whether a command runs, and its blocks; an agent thread its agent and state. Whenever an agent
//! anywhere waits on the user, the row says so, with Allow and Open, as the rail's inbox does.

use std::path::Path;

use agent_ui::ConversationView;
use gpui::{
    AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, Subscription, WeakEntity,
    Window, px,
};
use marley_rail::{InboxAnswer, InboxEntry};
use terminal_view::{MarleyBlockMarks, TerminalView};
use ui::{Button, ButtonStyle, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::ItemHandle;
use workspace::{MultiWorkspace, StatusItemView, Workspace};

use crate::rail::Rail;
use crate::thread_tab::ThreadTab;
use crate::tmux_sessions::{Listing, TmuxHosts};

/// The most of a waiting agent's request the row shows.
const ASK_MAX: usize = 80;
/// The widest a terminal's title and a waiting agent's request draw, so the items after the row
/// keep their places.
const TITLE_MAX: f32 = 200.;
const ASK_WIDTH: f32 = 320.;

/// What the active item is, as the row tells it.
enum Focus {
    None,
    File(Box<dyn ItemHandle>),
    Terminal(Box<dyn ItemHandle>, Entity<TerminalView>),
    Thread(Entity<ConversationView>),
}

/// The bottom row's focus item.
pub struct FocusRow {
    workspace: WeakEntity<Workspace>,
    focus: Focus,
    /// The active item's entity, observed so its state redraws the row.
    item_events: Option<Subscription>,
    /// The rail the needs-you part reads, observed.
    rail: Option<(WeakEntity<Rail>, Subscription)>,
    _tmux: Subscription,
}

impl std::fmt::Debug for FocusRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("FocusRow").finish_non_exhaustive()
    }
}

impl FocusRow {
    /// A row item for `workspace`'s status bar, which reads nothing of the workspace until it draws
    /// (PR-claude-831).
    pub fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        Self {
            workspace,
            focus: Focus::None,
            item_events: None,
            rail: None,
            _tmux: cx.observe_global::<TmuxHosts>(|_, cx| cx.notify()),
        }
    }

    /// The window's rail, observed once found, as Home finds it (#713).
    fn rail(&mut self, window: &Window, cx: &mut Context<Self>) -> Option<Entity<Rail>> {
        let multi_workspace = window.root::<MultiWorkspace>().flatten()?;
        let rail = multi_workspace
            .read(cx)
            .sidebar()?
            .to_any()
            .downcast::<Rail>()
            .ok()?;
        let followed = self
            .rail
            .as_ref()
            .is_some_and(|(followed, _)| followed.entity_id() == rail.entity_id());
        if !followed {
            let subscription = cx.observe(&rail, |_, _, cx| cx.notify());
            self.rail = Some((rail.downgrade(), subscription));
        }
        Some(rail)
    }

    fn render_file(item: &dyn ItemHandle, cx: &App) -> Option<AnyElement> {
        let path = item.project_path(cx)?;
        let path = path.path.as_unix_str().to_string();
        let kind = Path::new(&path)
            .extension()
            .map(|extension| extension.to_string_lossy().into_owned());
        Some(
            h_flex()
                .debug_selector(|| "marley-focus-file".into())
                .gap_1()
                .children(kind.map(|kind| chip(kind, cx)))
                .child(Label::new(path).size(LabelSize::Small).truncate())
                .into_any_element(),
        )
    }

    fn render_terminal(
        &self,
        item: &dyn ItemHandle,
        view: &Entity<TerminalView>,
        cx: &App,
    ) -> AnyElement {
        let terminal = view.read(cx).terminal().read(cx);
        let root = self.workspace.upgrade().and_then(|workspace| {
            workspace
                .read(cx)
                .project()
                .read(cx)
                .visible_worktrees(cx)
                .next()
                .map(|worktree| worktree.read(cx).abs_path().to_path_buf())
        });
        let folder = marley_rail::working_directory_label(
            terminal.working_directory().as_deref(),
            root.as_deref(),
            Some(paths::home_dir()),
        );
        let last = crate::rail::command_snapshot(terminal);
        let live = terminal.marley_anchored().live();
        let blocks = live.len();
        let terminal_id = view.read(cx).terminal().entity_id();
        let bookmarked = cx
            .try_global::<MarleyBlockMarks>()
            .and_then(|marks| marks.bookmarks.get(&terminal_id))
            .map_or(0, collections::BTreeSet::len);
        let sessions: usize = cx.try_global::<TmuxHosts>().map_or(0, |tmux| {
            tmux.hosts
                .iter()
                .map(|host| match &host.listing {
                    Listing::Sessions(sessions) => sessions.len(),
                    Listing::Waiting | Listing::Failed(_) => 0,
                })
                .sum()
        });
        let (command, running) = match &last {
            Some(last) if last.running => (None, Some(last.text.clone())),
            Some(last) => (Some((last.text.clone(), last.exit_code)), None),
            None => (None, None),
        };
        h_flex()
            .debug_selector(|| "marley-focus-terminal".into())
            .gap_2()
            .child(
                h_flex()
                    .gap_1()
                    .child(chip(">_".to_string(), cx))
                    // A tab's title carries a program's arguments; the row keeps room for the rest.
                    .child(
                        div().max_w(px(TITLE_MAX)).min_w_0().child(
                            Label::new(item.tab_content_text(0, cx))
                                .size(LabelSize::Small)
                                .truncate(),
                        ),
                    ),
            )
            .children(folder.map(muted))
            .children(command.map(|(text, exit)| {
                let failed = exit.is_some_and(|code| code != 0);
                let words = match exit {
                    Some(code) => format!("{text} · exit {code}"),
                    None => text,
                };
                Label::new(words)
                    .size(LabelSize::Small)
                    .color(if failed { Color::Error } else { Color::Muted })
                    .into_any_element()
            }))
            .child(running.map_or_else(
                || muted("no command running".to_string()),
                |text| {
                    Label::new(format!("running: {text}"))
                        .size(LabelSize::Small)
                        .color(Color::Accent)
                        .into_any_element()
                },
            ))
            .child(muted(match (blocks, bookmarked) {
                (1, 0) => "1 block".to_string(),
                (blocks, 0) => format!("{blocks} blocks"),
                (blocks, bookmarked) => format!("{blocks} blocks · {bookmarked} bookmarked"),
            }))
            .when(sessions > 0, |row| {
                row.child(muted(format!("tmux {sessions}")))
            })
            .into_any_element()
    }

    fn render_thread(&self, conversation: &Entity<ConversationView>, cx: &App) -> AnyElement {
        let conversation = conversation.read(cx);
        let agent = conversation.agent_key().id();
        let name = self.workspace.upgrade().map_or_else(
            || agent.0.clone(),
            |workspace| crate::agents::thread_agent_name(&agent, workspace.read(cx).project(), cx),
        );
        let status = conversation.root_thread_view().map(|view| {
            let thread = view.read(cx).thread.read(cx);
            marley_rail::thread_status(
                conversation.root_thread_has_pending_tool_call(cx),
                thread.had_error(),
                thread.status() == acp_thread::ThreadStatus::Generating,
            )
        });
        h_flex()
            .debug_selector(|| "marley-focus-thread".into())
            .gap_2()
            .child(
                Label::new(name)
                    .size(LabelSize::Small)
                    .weight(FontWeight::SEMIBOLD),
            )
            .children(status.map(|status| {
                let color = match status {
                    marley_rail::ThreadStatus::Running => Color::Warning,
                    marley_rail::ThreadStatus::Waiting => Color::Conflict,
                    marley_rail::ThreadStatus::Error => Color::Error,
                    marley_rail::ThreadStatus::Done => Color::Muted,
                };
                Label::new(status.label())
                    .size(LabelSize::Small)
                    .color(color)
                    .into_any_element()
            }))
            .child(muted(conversation.title(cx).to_string()))
            .into_any_element()
    }

    /// The oldest request waiting on the user, with Allow when it takes one, Open, and how many
    /// more wait.
    fn render_waiting(rail: &Entity<Rail>, cx: &App) -> Option<AnyElement> {
        let inbox = rail.read(cx).inbox();
        let entry: &InboxEntry = inbox.first()?;
        let more = inbox.len() - 1;
        let ask = if entry.ask.chars().count() > ASK_MAX {
            let mut cut: String = entry.ask.chars().take(ASK_MAX - 1).collect();
            cut.push('…');
            cut
        } else {
            entry.ask.clone()
        };
        let allows = entry.answers.contains(&InboxAnswer::Allow);
        let (allow_rail, open_rail) = (rail.downgrade(), rail.downgrade());
        let (allow_key, open_key) = (entry.key.clone(), entry.key.clone());
        Some(
            h_flex()
                .debug_selector(|| "marley-focus-waiting".into())
                .flex_none()
                .gap_1p5()
                .px_1p5()
                .rounded_sm()
                .bg(cx.theme().status().conflict_background)
                .child(
                    div()
                        .size_1p5()
                        .rounded_full()
                        .bg(cx.theme().status().conflict),
                )
                .child(
                    div().max_w(px(ASK_WIDTH)).min_w_0().child(
                        Label::new(format!("{} in {}: {ask}", entry.agent, entry.project))
                            .size(LabelSize::Small)
                            .truncate(),
                    ),
                )
                .when(allows, |row| {
                    row.child(
                        Button::new("marley-focus-allow", "Allow")
                            .label_size(LabelSize::Small)
                            .style(ButtonStyle::Filled)
                            .on_click(move |_, window, cx| {
                                allow_rail
                                    .update(cx, |rail, cx| {
                                        rail.answer_inbox(
                                            &allow_key,
                                            InboxAnswer::Allow,
                                            window,
                                            cx,
                                        );
                                    })
                                    .log_err();
                            }),
                    )
                })
                .child(
                    Button::new("marley-focus-open", "Open")
                        .label_size(LabelSize::Small)
                        .style(ButtonStyle::Subtle)
                        .tooltip(Tooltip::text("Show where it waits"))
                        .on_click(move |_, window, cx| {
                            open_rail
                                .update(cx, |rail, cx| {
                                    rail.open_inbox_entry(&open_key, window, cx);
                                })
                                .log_err();
                        }),
                )
                .when(more > 0, |row| {
                    row.child(muted(format!("+{more} more waiting")))
                })
                .into_any_element(),
        )
    }
}

impl Render for FocusRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rail = self.rail(window, cx);
        let focus = match &self.focus {
            Focus::None => None,
            Focus::File(item) => Self::render_file(item.as_ref(), cx),
            Focus::Terminal(item, view) => Some(self.render_terminal(item.as_ref(), view, cx)),
            Focus::Thread(conversation) => Some(self.render_thread(conversation, cx)),
        };
        let waiting = rail.and_then(|rail| Self::render_waiting(&rail, cx));
        // The row keeps its width, its title and request already capped; Zed's items after it
        // (the language servers' message) are the ones that shorten.
        h_flex()
            .id("marley-focus-row")
            .flex_none()
            .gap_3()
            .children(focus)
            .children(waiting)
    }
}

impl StatusItemView for FocusRow {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.item_events = None;
        self.focus = match active_pane_item {
            None => Focus::None,
            Some(item) => {
                if let Some(view) = item.downcast::<TerminalView>() {
                    let terminal = view.read(cx).terminal().clone();
                    self.item_events = Some(cx.observe(&terminal, |_, _, cx| cx.notify()));
                    Focus::Terminal(item.boxed_clone(), view)
                } else if let Some(tab) = item.downcast::<ThreadTab>() {
                    let conversation = tab.read(cx).conversation().clone();
                    self.item_events = Some(cx.observe(&conversation, |_, _, cx| cx.notify()));
                    Focus::Thread(conversation)
                } else if item.project_path(cx).is_some() {
                    Focus::File(item.boxed_clone())
                } else {
                    Focus::None
                }
            }
        };
        cx.notify();
    }

    fn hide_setting(&self, _cx: &App) -> Option<workspace::HideStatusItem> {
        None
    }
}

/// A small rounded chip of short text: a file's extension, `>_`.
fn chip(text: String, cx: &App) -> AnyElement {
    div()
        .px_1()
        .rounded_sm()
        .bg(cx.theme().colors().element_background)
        .child(
            Label::new(text)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .buffer_font(cx),
        )
        .into_any_element()
}

fn muted(text: String) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(Color::Muted)
        .into_any_element()
}
