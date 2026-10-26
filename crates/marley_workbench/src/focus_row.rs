//! The start of the bottom row says what has the focus (#833).
//!
//! A file shows its type and path; a terminal its title, folder, last command and its exit,
//! whether a command runs, and its blocks; an agent thread its agent, model, mode, what it does
//! and for how long, its context and the files it changed (#907). Whenever an agent
//! anywhere waits on the user, the row says so, with Allow and Open, as the rail's inbox does.

use std::path::Path;
use std::sync::Arc;

use agent_settings::AgentSettings;
use agent_ui::ConversationView;
use gpui::{
    AnyElement, App, Context, Entity, EntityId, FontWeight, IntoElement, Render, Subscription,
    WeakEntity, Window, px,
};
use marley_rail::{InboxAnswer, InboxEntry};
use settings::Settings as _;
use terminal_view::{MarleyBlockMarks, TerminalView};
use ui::{Button, ButtonStyle, Label, LabelSize, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::ItemHandle;
use workspace::{MultiWorkspace, StatusItemView, Workspace};

use crate::item_kind::{self, AgentMark, ItemKind, ItemState};
use crate::rail::Rail;
use crate::thread_activity::{self, ThreadActivity};
use crate::thread_tab::{self, ThreadTab};
use crate::tmux_sessions::{Listing, TmuxHosts};

/// The most of a waiting agent's request the row shows.
const ASK_MAX: usize = 80;
/// The widest a terminal's title and a waiting agent's request draw, so the items after the row
/// keep their places.
const TITLE_MAX: f32 = 100.;
const ASK_WIDTH: f32 = 320.;
/// The widest a terminal's folder and its failed command draw (#908).
const FOLDER_MAX: f32 = 110.;
const FAILED_MAX: f32 = 90.;
/// The widest a thread's activity draws (PR-claude-833).
const ACTIVITY_WIDTH: f32 = 280.;

/// What the active item is, as the row tells it.
enum Focus {
    None,
    File(Box<dyn ItemHandle>),
    Terminal(Box<dyn ItemHandle>, Entity<TerminalView>),
    Thread(Entity<ConversationView>),
    /// One of Marley's own screens (#909).
    Screen(crate::focus_screens::Screen),
}

/// The bottom row's focus item.
pub struct FocusRow {
    workspace: WeakEntity<Workspace>,
    focus: Focus,
    /// The active item's entity, observed so its state redraws the row.
    item_events: Option<Subscription>,
    /// A thread tab's thread view, thread and action log, observed once it has them, so the turn's
    /// timer, token use and edits redraw the row (#907).
    thread_events: Option<(EntityId, [Subscription; 3])>,
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
            thread_events: None,
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
                .children(kind.map(|kind| chip(kind, ItemKind::File, cx)))
                .child(Label::new(path).size(LabelSize::Small).truncate())
                .into_any_element(),
        )
    }

    /// A terminal as the `BottomRow` board draws it (#908): its tile and title, folder, branch with
    /// its changed count and a failed command as Ask an agent; at the right whether a command runs,
    /// its blocks and bookmarks, its size and the tmux sessions.
    fn render_terminal(
        &self,
        item: &dyn ItemHandle,
        view: &Entity<TerminalView>,
        cx: &App,
    ) -> AnyElement {
        let terminal = view.read(cx).terminal().read(cx);
        let folder = terminal.working_directory();
        let repository = folder.as_ref().and_then(|folder| {
            let workspace = self.workspace.upgrade()?;
            crate::agent_bar::branch_and_changes(workspace.read(cx).project(), folder, cx)
        });
        let last = crate::rail::command_snapshot(terminal);
        let live = terminal.marley_anchored().live();
        let blocks = live.len();
        let terminal_id = view.read(cx).terminal().entity_id();
        let bookmarked = cx
            .try_global::<MarleyBlockMarks>()
            .and_then(|marks| marks.bookmarks.get(&terminal_id))
            .map_or(0, collections::BTreeSet::len);
        // The failed block the chip asks about: the newest live one, as the block header's Ask an
        // agent finds it (#826).
        let failed = last
            .as_ref()
            .filter(|last| !last.running && last.exit_code.is_some_and(|code| code != 0))
            .zip(live.last())
            .map(|(last, block)| FailedBlock {
                text: last.text.clone(),
                exit: last.exit_code.unwrap_or_default(),
                index: block.index,
            });
        let running = last
            .as_ref()
            .filter(|last| last.running)
            .map(|last| last.text.clone());
        let bounds = terminal.last_content().terminal_bounds;
        let size = format!("{}\u{d7}{}", bounds.num_columns(), bounds.num_lines());
        let has_failed = failed.is_some();
        let left = h_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_1()
                    .child(chip(">_".to_string(), ItemKind::Shell, cx))
                    // A tab's title carries a program's arguments; the row keeps room for the rest.
                    .child(
                        div().max_w(px(TITLE_MAX)).min_w_0().child(
                            Label::new(item.tab_content_text(0, cx))
                                .size(LabelSize::Small)
                                .truncate(),
                        ),
                    ),
            )
            .children(folder.map(|folder| {
                let label = crate::block_headers::folder_label(&folder.to_string_lossy());
                div()
                    .max_w(px(FOLDER_MAX))
                    .min_w_0()
                    .child(
                        Label::new(label)
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    )
                    .into_any_element()
            }))
            .children(repository.map(|repository| {
                let words = match repository.changed {
                    0 => repository.branch.to_string(),
                    count => format!("{} \u{b7} {count} changed", repository.branch),
                };
                Label::new(words)
                    .size(LabelSize::Small)
                    .color(Color::Custom(cx.theme().colors().terminal_ansi_magenta))
                    .into_any_element()
            }))
            .children(failed.map(|failed| failed_chip(failed, view, cx)));
        // A failed chip already says nothing runs; its room goes to the tmux chip (#908).
        let idle_words = (!has_failed).then(|| muted("no command running".to_string()));
        let right = h_flex()
            .gap_2()
            .children(running.map_or(idle_words, |text| {
                Some(
                    Label::new(format!("running: {text}"))
                        .size(LabelSize::Small)
                        .color(Color::Accent)
                        .into_any_element(),
                )
            }))
            .child(muted(blocks_words(blocks, bookmarked)))
            .child(
                Label::new(size)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .buffer_font(cx),
            )
            .children(tmux_chip(cx));
        h_flex()
            .debug_selector(|| "marley-focus-terminal".into())
            .gap_3()
            .child(left)
            .child(right)
            .into_any_element()
    }

    /// Follows a conversation's thread view (its turn timer), its thread (token use, tool calls)
    /// and its action log (edits) (#907). `ThreadView` is `agent_ui`'s own, named by no export.
    fn follow_thread<V: 'static>(
        &mut self,
        view: &Entity<V>,
        thread: &Entity<acp_thread::AcpThread>,
        cx: &mut Context<Self>,
    ) {
        if self
            .thread_events
            .as_ref()
            .is_some_and(|(id, _)| *id == view.entity_id())
        {
            return;
        }
        let action_log = thread.read(cx).action_log().clone();
        self.thread_events = Some((
            view.entity_id(),
            [
                cx.observe(view, |_, _, cx| cx.notify()),
                cx.observe(thread, |_, _, cx| cx.notify()),
                cx.observe(&action_log, |_, _, cx| cx.notify()),
            ],
        ));
    }

    /// A thread as the Main and `BottomRow` boards draw it (#907): its agent, model and effort,
    /// mode, what it does or asks and for how long, its context, the files it changed, and Full
    /// width.
    fn render_thread(
        &mut self,
        conversation: &Entity<ConversationView>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let agent = conversation.read(cx).agent_key().id();
        let name = self.workspace.upgrade().map_or_else(
            || agent.0.clone(),
            |workspace| crate::agents::thread_agent_name(&agent, workspace.read(cx).project(), cx),
        );
        let mark = AgentMark::from_agent_id(&agent);
        let view = conversation.read(cx).root_thread_view();
        if let Some(view) = &view {
            let thread = view.read(cx).thread.clone();
            self.follow_thread(view, &thread, cx);
        }
        let facts = view
            .as_ref()
            .map(|view| view.read(cx).marley_session_facts(cx));
        let (usage, files) = view.as_ref().map_or_else(
            || (None, Vec::new()),
            |view| {
                let thread = view.read(cx).thread.read(cx);
                let usage = thread
                    .token_usage()
                    .map(|usage| (usage.used_tokens, usage.max_tokens));
                (usage, thread_tab::changed_files(thread, cx))
            },
        );
        let activity = thread_activity::thread_activity(conversation, cx);
        let (model, mode) = facts.map_or((None, None), |facts| {
            let model = facts.model.map(|model| {
                facts.effort.map_or_else(
                    || model.to_string(),
                    |effort| format!("{model} \u{b7} {effort}"),
                )
            });
            (model, facts.mode.map(|mode| mode.to_string()))
        });
        h_flex()
            .debug_selector(|| "marley-focus-thread".into())
            .gap_3()
            .child(
                h_flex()
                    .gap_1p5()
                    .child(item_kind::tile(
                        ItemKind::Thread(mark),
                        mark.glyph(),
                        px(22.),
                        cx,
                    ))
                    .child(
                        Label::new(name)
                            .size(LabelSize::Small)
                            .weight(FontWeight::SEMIBOLD),
                    ),
            )
            .children(model.map(muted))
            .children(mode.map(muted))
            .children(activity.map(|activity| activity_chip(&activity, cx)))
            .children(usage.and_then(|usage| context_meter(usage, cx)))
            .children(files_changed(&files))
            .child(self.render_full_width(cx))
            .into_any_element()
    }

    /// Full width (#907, D3): the thread's text the full width or a readable column, the one
    /// setting #822's button in the composer flips.
    fn render_full_width(&self, cx: &App) -> AnyElement {
        let limited = AgentSettings::get_global(cx).max_content_width.is_some();
        let fs = self
            .workspace
            .upgrade()
            .map(|workspace| Arc::clone(&workspace.read(cx).app_state().fs));
        Button::new("marley-focus-full-width", "Full width")
            .label_size(LabelSize::Small)
            .style(ButtonStyle::Subtle)
            .toggle_state(!limited)
            .on_click(move |_, _, cx| {
                let Some(fs) = fs.clone() else {
                    return;
                };
                settings::update_settings_file(fs, cx, move |settings, _| {
                    settings.agent.get_or_insert_default().limit_content_width = Some(!limited);
                });
            })
            .into_any_element()
    }

    /// The oldest request waiting on the user, with Allow when it takes one, Open, and how many
    /// more wait.
    /// `front` is the thread in front, by its key: when it is the one waiting, its own chip already
    /// says so (#907), and the row leaves the request out.
    fn render_waiting(rail: &Entity<Rail>, front: Option<&str>, cx: &App) -> Option<AnyElement> {
        let inbox = rail.read(cx).inbox();
        let entry: &InboxEntry = inbox.first()?;
        if let Some(front) = front
            && entry.key.starts_with(&format!("thread:{front}:"))
        {
            return None;
        }
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
            Focus::File(item) => Self::render_file(item.as_ref(), cx),
            Focus::Screen(screen) => Some(screen.render(window, cx)),
            Focus::Terminal(item, view) => Some(self.render_terminal(item.as_ref(), view, cx)),
            // A thread draws below, where the row's own context can follow its thread.
            Focus::None | Focus::Thread(_) => None,
        };
        let thread = match &self.focus {
            Focus::Thread(conversation) => Some(conversation.clone()),
            _ => None,
        };
        let focus =
            focus.or_else(|| thread.map(|conversation| self.render_thread(&conversation, cx)));
        let front = match &self.focus {
            Focus::Thread(conversation) => Some(conversation.read(cx).parent_id().to_key_string()),
            _ => None,
        };
        let waiting = rail.and_then(|rail| Self::render_waiting(&rail, front.as_deref(), cx));
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
        self.thread_events = None;
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
                } else if let Some(screen) = crate::focus_screens::Screen::of(item) {
                    self.item_events = screen.observe(cx);
                    Focus::Screen(screen)
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

/// A thread's state, what it does or asks and for how long, on its state's colour (#907).
fn activity_chip(activity: &ThreadActivity, cx: &App) -> AnyElement {
    let state = ItemState::from_thread(activity.status());
    let status = cx.theme().status();
    let background = match state {
        ItemState::Working => status.warning_background,
        ItemState::NeedsYou => status.conflict_background,
        ItemState::Failed => status.error_background,
        ItemState::Done | ItemState::Idle => cx.theme().colors().element_background,
    };
    let word = state.words();
    let mut parts: Vec<String> = vec![capitalized(word)];
    if activity.doing().as_ref() != word {
        parts.push(activity.doing().to_string());
    }
    if let Some(elapsed) = activity.elapsed() {
        parts.push(thread_activity::elapsed_words(elapsed));
    }
    h_flex()
        .debug_selector(|| "marley-focus-activity".into())
        .flex_none()
        .gap_1p5()
        .px_1p5()
        .rounded_sm()
        .bg(background)
        .child(div().size_1p5().rounded_full().bg(state.dot(cx)))
        .child(
            div().max_w(px(ACTIVITY_WIDTH)).min_w_0().child(
                Label::new(parts.join(" \u{b7} "))
                    .size(LabelSize::Small)
                    .color(state.line(cx))
                    .truncate(),
            ),
        )
        .into_any_element()
}

/// "Context", a small bar and "<used> of <max>" (#907), when the agent says its window.
fn context_meter((used, max): (u64, u64), cx: &App) -> Option<AnyElement> {
    if max == 0 {
        return None;
    }
    // In thousandths, which a u16 holds and an f32 takes whole.
    let thousandths = u16::try_from(used.min(max) * 1000 / max).unwrap_or(1000);
    let share = f32::from(thousandths) / 1000.;
    Some(
        h_flex()
            .flex_none()
            .gap_1p5()
            .child(muted("Context".to_string()))
            .child(
                div()
                    .w(px(48.))
                    .h_1()
                    .rounded_full()
                    .bg(cx.theme().colors().element_background)
                    .child(
                        div()
                            .h_full()
                            .w(relative(share))
                            .rounded_full()
                            .bg(cx.theme().colors().text_accent),
                    ),
            )
            .child(muted(format!(
                "{} of {}",
                thread_tab::humanize(used),
                thread_tab::humanize(max)
            )))
            .into_any_element(),
    )
}

/// "<N> files · +<added> −<removed>" (#907), when the thread changed any.
fn files_changed(files: &[thread_tab::ChangedFile]) -> Option<AnyElement> {
    if files.is_empty() {
        return None;
    }
    let added: u32 = files.iter().map(|file| file.added).sum();
    let removed: u32 = files.iter().map(|file| file.removed).sum();
    let count = if files.len() == 1 {
        "1 file".to_string()
    } else {
        format!("{} files", files.len())
    };
    Some(
        h_flex()
            .flex_none()
            .gap_1()
            .child(muted(format!("{count} \u{b7}")))
            .child(
                Label::new(format!("+{added}"))
                    .size(LabelSize::Small)
                    .color(Color::Created),
            )
            .child(
                Label::new(format!("\u{2212}{removed}"))
                    .size(LabelSize::Small)
                    .color(Color::Deleted),
            )
            .into_any_element(),
    )
}

/// `word` with its first letter in capitals: "needs you" → "Needs you".
fn capitalized(word: &str) -> String {
    let mut letters = word.chars();
    letters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(letters).collect()
    })
}

/// "N blocks · M bookmarked", the bookmarks only when there are some.
fn blocks_words(blocks: usize, bookmarked: usize) -> String {
    let blocks = if blocks == 1 {
        "1 block".to_string()
    } else {
        format!("{blocks} blocks")
    };
    if bookmarked == 0 {
        blocks
    } else {
        format!("{blocks} \u{b7} {bookmarked} bookmarked")
    }
}

/// The newest block, when it failed: its one-line command, exit code and index (#908).
struct FailedBlock {
    text: String,
    exit: i32,
    index: usize,
}

/// The failed command as a button that asks an agent to fix it (#908, D1): the block header's own
/// Ask an agent.
fn failed_chip(failed: FailedBlock, view: &Entity<TerminalView>, cx: &App) -> AnyElement {
    let status = cx.theme().status();
    let view = view.clone();
    let FailedBlock { text, exit, index } = failed;
    h_flex()
        .id("marley-focus-ask-agent")
        .debug_selector(|| "marley-focus-failed".into())
        .flex_none()
        .gap_1p5()
        .px_1p5()
        .rounded_sm()
        .bg(status.error_background)
        .cursor_pointer()
        .hover(|style| style.opacity(0.85))
        .child(div().size_1p5().rounded_full().bg(status.error))
        .child(
            div().max_w(px(FAILED_MAX)).min_w_0().child(
                Label::new(text)
                    .size(LabelSize::Small)
                    .color(Color::Error)
                    .truncate(),
            ),
        )
        .child(
            Label::new(format!("\u{b7} exit {exit} \u{b7} Ask an agent"))
                .size(LabelSize::Small)
                .color(Color::Error),
        )
        .tooltip(Tooltip::text("Ask an agent to fix it"))
        .on_click(move |_, window, cx| {
            crate::send_block::send(&view, index, crate::send_block::Request::Fix, window, cx);
        })
        .into_any_element()
}

/// "tmux: N · M detached" on the tmux tint (#908); a click opens the Sessions page, whether the
/// rail is open, shrunk or hidden (#930, D5).
fn tmux_chip(cx: &App) -> Option<AnyElement> {
    let sessions: Vec<bool> = cx.try_global::<TmuxHosts>().map_or_else(Vec::new, |tmux| {
        tmux.hosts
            .iter()
            .flat_map(|host| match &host.listing {
                Listing::Sessions(sessions) => sessions
                    .iter()
                    .map(|session| session.attached)
                    .collect::<Vec<_>>(),
                Listing::Waiting | Listing::Failed(_) => Vec::new(),
            })
            .collect()
    });
    if sessions.is_empty() {
        return None;
    }
    let detached = sessions.iter().filter(|attached| !**attached).count();
    let words = match detached {
        0 => format!("tmux: {}", sessions.len()),
        detached => format!("tmux: {} \u{b7} {detached} detached", sessions.len()),
    };
    Some(
        h_flex()
            .id("marley-focus-tmux")
            .debug_selector(|| "marley-focus-tmux".into())
            .flex_none()
            .px_1p5()
            .rounded_sm()
            .bg(ItemKind::Tmux.tint(cx))
            .cursor_pointer()
            .hover(|style| style.opacity(0.85))
            .child(
                Label::new(words)
                    .size(LabelSize::Small)
                    .color(Color::Custom(ItemKind::Tmux.ink(cx))),
            )
            .tooltip(Tooltip::text("Open the Sessions page"))
            .on_click(|_, window, cx| {
                // Deferred: the status bar is being updated, and the page opens in a workspace.
                window.defer(cx, |window, cx| {
                    window.dispatch_action(Box::new(crate::sessions_page::OpenSessionsPage), cx);
                });
            })
            .into_any_element(),
    )
}

/// A small rounded chip of short text on its kind's tint: a file's extension, `>_`.
pub(crate) fn chip(text: String, kind: ItemKind, cx: &App) -> AnyElement {
    div()
        .px_1()
        .rounded_sm()
        .bg(kind.tint(cx))
        .child(
            Label::new(text)
                .size(LabelSize::XSmall)
                .color(Color::Custom(kind.ink(cx)))
                .buffer_font(cx),
        )
        .into_any_element()
}

/// Small muted words, the row's facts.
pub(crate) fn muted(text: String) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(Color::Muted)
        .into_any_element()
}
