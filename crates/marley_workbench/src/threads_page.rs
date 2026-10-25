//! The Threads page (#737): every agent conversation Zed keeps, from every project, group and
//! folder, in a center tab.
//!
//! Zed keeps its thread history in its own sidebar (`ThreadsArchiveView`), which the Marley layout
//! replaces with the rail, so without this page an old conversation is found only under the
//! project it ran in. The page lists Zed's thread store, Marley's and Rusty's conversations first,
//! with a search over titles and folders and a chip per agent. A click brings a conversation forward
//! where it is open, else opens it in a tab of the page's group (#734, #736).
//!
//! Reopen Thread… (#772) is the same list as a picker: the unarchived conversations, newest first,
//! opened the page's way.
//!
//! The Claude Code and Codex sessions in the CLIs' own files are listed among them (#754, from
//! `cli_sessions`), with a chip per CLI; the page's search also finds one by its prompts, and a
//! click reopens it with its CLI's resume, or brings forward the terminal running it. A session
//! Zed's store already holds, by its session id, is listed once, as Zed's.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use agent_ui::thread_metadata_store::{ThreadId, ThreadMetadata, ThreadMetadataStore};
use agent_ui::{AgentPanel, ConversationView};
use chrono::{DateTime, Utc};
use editor::{Editor, EditorEvent};
use fuzzy::{StringMatch, StringMatchCandidate};
use gpui::{
    App, AppContext as _, Context, DismissEvent, Empty, Entity, EventEmitter, FocusHandle,
    Focusable, SharedString, Subscription, Task, WeakEntity, Window, actions,
};
use marley_agent::AgentKind;
use marley_agent::cli_sessions::{CliSession, Found};
use marley_mcp::redact::Redactor;
use picker::{Picker, PickerDelegate};
use project::{AgentId, DisableAiSettings, Project};
use settings::Settings as _;
use ui::{
    ButtonStyle, Headline, HeadlineSize, HighlightedLabel, ListItem, ListItemSpacing, Tooltip,
    prelude::*,
};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{ModalView, MultiWorkspace, Workspace};

use crate::agents::{self, AgentIcon};
use crate::home_page::muted;
use crate::item_kind::{self, AgentMark, ItemKind, ItemState};
use crate::rail::{ConversationState, ConversationStates, Rail};

actions!(
    marley,
    [
        /// Opens the Threads page: every agent conversation, from every project and group,
        /// searchable, in a tab of the shown group.
        #[derive(Eq)]
        OpenThreads,
        /// Reopens an earlier agent conversation: a picker over the saved ones, newest first, that
        /// shows the one chosen in a tab of the shown group.
        #[derive(Eq)]
        ReopenThread,
    ]
);

pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenThreads, window, cx| {
            open(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &ReopenThread, window, cx| {
            reopen(workspace, window, cx);
        });
    })
    .detach();
}

/// Opens Reopen Thread… over `workspace` (#772).
pub(crate) fn reopen(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if DisableAiSettings::get_global(cx).disable_ai {
        return;
    }
    crate::cli_sessions::refresh(cx);
    let handle = workspace.weak_handle();
    let project = workspace.project().clone();
    let rows = conversations("", cx);
    workspace.toggle_modal(window, cx, |window, cx| {
        ReopenThreadPicker::new(handle, project, rows, window, cx)
    });
}

/// The store's unarchived conversations, not drafts, and the CLI sessions (#754), newest first,
/// whose title or folder holds `query`: what Reopen Thread… (#772) and the rail's Threads view
/// (#745) list.
pub(crate) fn conversations(query: &str, cx: &App) -> Vec<Row> {
    let query = query.to_lowercase();
    let mut rows: Vec<Row> = ThreadMetadataStore::try_global(cx)
        .map(|store| {
            store
                .read(cx)
                .entries()
                .filter(|record| !record.is_draft() && !record.archived && matches(record, &query))
                .map(row_of)
                .collect()
        })
        .unwrap_or_default();
    rows.extend(cli_rows(cx).into_iter().filter_map(|(mut row, session)| {
        matches_cli(&mut row, &session, &query, &CliSearch::Titles).then_some(row)
    }));
    rows.sort_by_key(|row| std::cmp::Reverse(row.updated_at));
    rows
}

/// `record` as the page and the picker list it.
fn row_of(record: &ThreadMetadata) -> Row {
    Row {
        target: Target::Thread {
            thread_id: record.thread_id,
            agent: record.agent_id.clone(),
        },
        title: record.display_title(),
        folder: record
            .folder_paths()
            .paths()
            .first()
            .map(|path| shown_folder(path)),
        updated_at: record.updated_at,
        snippet: None,
        when: when(record.updated_at),
    }
}

/// The CLI sessions the last scan found (#754), as rows, each with its session, but for the ones
/// Zed's store holds by their session id.
fn cli_rows(cx: &App) -> Vec<(Row, Arc<CliSession>)> {
    let known: HashSet<String> = ThreadMetadataStore::try_global(cx)
        .map(|store| {
            store
                .read(cx)
                .entries()
                .filter_map(|record| record.session_id.as_ref().map(|id| id.0.to_string()))
                .collect()
        })
        .unwrap_or_default();
    crate::cli_sessions::listed(cx)
        .iter()
        .filter(|listed| !known.contains(&listed.session.id))
        .map(|listed| {
            let session = Arc::clone(&listed.session);
            let row = Row {
                target: Target::Cli(Arc::clone(&session)),
                title: session.shown_title().into(),
                folder: Some(shown_folder(std::path::Path::new(&session.folder))),
                updated_at: listed.modified,
                snippet: None,
                when: when(listed.modified),
            };
            (row, session)
        })
        .collect()
}

/// How a CLI session's row is searched: by its title and folder, or by its prompts too, whose
/// snippet goes through #516's redactor while agents' redaction is on.
enum CliSearch<'a> {
    Titles,
    Prompts(Option<&'a Redactor>),
}

/// Whether a CLI session's row matches `query` (lowercase): by its title or folder, or by one of
/// its prompts, which gives the row its snippet.
fn matches_cli(row: &mut Row, session: &CliSession, query: &str, search: &CliSearch<'_>) -> bool {
    match (session.search(query), search) {
        (Some(Found::Listed), _) => true,
        (Some(Found::Prompt(snippet)), CliSearch::Prompts(redactor)) => {
            row.snippet = Some(crate::mcp::for_agents(&snippet, *redactor).text);
            true
        }
        _ => false,
    }
}

/// When a row last changed, in the local clock: `Oct 11, 15:05`.
fn when(at: DateTime<Utc>) -> String {
    at.with_timezone(&chrono::Local)
        .format("%b %-d, %H:%M")
        .to_string()
}

/// Brings `workspace`'s Threads page forward, or opens one in its center.
pub(crate) fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if DisableAiSettings::get_global(cx).disable_ai {
        return;
    }
    let open_page = workspace.items_of_type::<ThreadsPage>(cx).next();
    if let Some(page) = open_page {
        workspace.activate_item(&page, true, true, window, cx);
        return;
    }
    crate::cli_sessions::refresh(cx);
    let handle = workspace.weak_handle();
    let page = cx.new(|cx| ThreadsPage::new(handle, window, cx));
    workspace.add_item_to_center(Box::new(page.clone()), window, cx);
    window.focus(&page.read(cx).search.focus_handle(cx), cx);
}

/// The page: the store's conversations, the search, and the agent the chips keep.
pub struct ThreadsPage {
    workspace: WeakEntity<Workspace>,
    search: Entity<Editor>,
    /// The chip chosen; none lists every agent.
    agent: Option<Chip>,
    /// Whether the archived conversations are listed.
    archived_open: bool,
    /// The folder chosen; none lists every folder (#836).
    folder: Option<String>,
    /// The row the preview shows, by its key (#836).
    selected: Option<String>,
    /// The window's rail, observed so the cards' states follow it (#919).
    rail: Option<(WeakEntity<Rail>, Subscription)>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for ThreadsPage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ThreadsPage")
            .finish_non_exhaustive()
    }
}

/// One conversation as the page lists it, and the rail's Threads view (#745).
pub(crate) struct Row {
    pub(crate) target: Target,
    pub(crate) title: SharedString,
    /// Its first folder, the home folder as `~`; none for a conversation with no folder.
    pub(crate) folder: Option<String>,
    /// When it last changed, which orders the lists.
    updated_at: DateTime<Utc>,
    /// The words around a prompt the page's search found it by (#754).
    snippet: Option<String>,
    when: String,
}

/// What a row opens: one of Zed's conversations, or a CLI session (#754).
#[derive(Clone)]
pub(crate) enum Target {
    Thread { thread_id: ThreadId, agent: AgentId },
    Cli(Arc<CliSession>),
}

impl Row {
    /// The row's identity in element ids.
    pub(crate) fn key(&self) -> String {
        match &self.target {
            Target::Thread { thread_id, .. } => thread_id.to_key_string(),
            Target::Cli(session) => format!("cli-{}", session.id),
        }
    }

    /// The icon of the row's agent: Zed's agent's, or the CLI's.
    pub(crate) fn icon(&self, project: &Entity<Project>, cx: &App) -> Icon {
        match &self.target {
            Target::Thread { agent, .. } => match agents::thread_icon(agent, project, cx) {
                AgentIcon::Named(name) => Icon::new(name),
                AgentIcon::Svg(path) => Icon::from_external_svg(path),
            },
            Target::Cli(session) => Icon::new(agents::cli_icon(session.agent)),
        }
    }

    /// The row's second line: its agent, and its folder when it has one.
    pub(crate) fn detail(&self, project: &Entity<Project>, cx: &App) -> String {
        let agent = match &self.target {
            Target::Thread { agent, .. } => agents::thread_agent_name(agent, project, cx),
            Target::Cli(session) => cli_name(session.agent).into(),
        };
        self.folder
            .as_ref()
            .map_or_else(|| agent.to_string(), |folder| format!("{agent} · {folder}"))
    }
}

/// A CLI's name beside Zed's agents of the same name: `Claude Code CLI`, `Codex CLI`.
fn cli_name(agent: AgentKind) -> String {
    format!("{} CLI", agent.display_name())
}

/// A chip of the page: one of Zed's agents, or a CLI (#754).
#[derive(Clone, PartialEq)]
enum Chip {
    Agent(AgentId),
    Cli(AgentKind),
}

/// The page's rows, the archived ones apart, and the chips and folders offered.
struct Sections {
    rows: Vec<Row>,
    archived: Vec<Row>,
    chips: Vec<Chip>,
    /// How many listed conversations each chip keeps, beside `chips`, whatever chip is chosen
    /// (#919).
    counts: Vec<usize>,
    folders: Vec<String>,
}

impl Sections {
    /// Offers `chip` when it is new.
    fn offer(&mut self, chip: &Chip) {
        if !self.chips.contains(chip) {
            self.chips.push(chip.clone());
            self.counts.push(0);
        }
    }

    /// Counts a listed conversation for `chip`.
    fn count(&mut self, chip: &Chip) {
        if let Some(index) = self.chips.iter().position(|offered| offered == chip)
            && let Some(count) = self.counts.get_mut(index)
        {
            *count += 1;
        }
    }
}

/// The day heading a row goes under, by when it last changed (#836).
fn day_of(updated_at: DateTime<Utc>) -> &'static str {
    let today = chrono::Local::now().date_naive();
    let day = updated_at.with_timezone(&chrono::Local).date_naive();
    match (today - day).num_days() {
        ..=0 => "TODAY",
        1 => "YESTERDAY",
        2..=6 => "THIS WEEK",
        _ => "EARLIER",
    }
}

impl ThreadsPage {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search titles, folders, and what you asked", window, cx);
            editor
        });
        let mut subscriptions = vec![cx.subscribe(&search, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        })];
        if let Some(store) = ThreadMetadataStore::try_global(cx) {
            subscriptions.push(cx.observe(&store, |_, _, cx| cx.notify()));
        }
        subscriptions.push(
            cx.observe_global::<crate::cli_sessions::CliSessions>(|_, cx| {
                cx.notify();
            }),
        );
        Self {
            workspace,
            search,
            agent: None,
            archived_open: false,
            folder: None,
            selected: None,
            rail: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// The store's conversations the search and the chip keep, newest first, in their sections.
    fn sections(&self, cx: &App) -> Sections {
        let query = self.search.read(cx).text(cx).trim().to_lowercase();
        let mut sections = Sections {
            rows: Vec::new(),
            archived: Vec::new(),
            chips: Vec::new(),
            counts: Vec::new(),
            folders: Vec::new(),
        };
        // The store's entries hold the archived conversations too, each once.
        let mut records: Vec<&ThreadMetadata> = ThreadMetadataStore::try_global(cx)
            .map(|store| {
                store
                    .read(cx)
                    .entries()
                    .filter(|record| !record.is_draft())
                    .collect()
            })
            .unwrap_or_default();
        records.sort_by_key(|record| std::cmp::Reverse(record.updated_at));
        for record in records {
            let chip = Chip::Agent(record.agent_id.clone());
            sections.offer(&chip);
            let row = row_of(record);
            if let Some(folder) = &row.folder
                && !sections.folders.contains(folder)
            {
                sections.folders.push(folder.clone());
            }
            let kept = matches(record, &query) && self.folder_keeps(&row);
            if kept && !record.archived {
                sections.count(&chip);
            }
            if !kept || self.agent.as_ref().is_some_and(|agent| *agent != chip) {
                continue;
            }
            if record.archived {
                sections.archived.push(row);
            } else {
                sections.rows.push(row);
            }
        }
        let redactor = crate::mcp::agent_redactor(cx);
        let search = CliSearch::Prompts(redactor.as_deref());
        for (mut row, session) in cli_rows(cx) {
            let chip = Chip::Cli(session.agent);
            sections.offer(&chip);
            if let Some(folder) = &row.folder
                && !sections.folders.contains(folder)
            {
                sections.folders.push(folder.clone());
            }
            if self.folder_keeps(&row) && matches_cli(&mut row, &session, &query, &search) {
                sections.count(&chip);
                if self.agent.as_ref().is_none_or(|agent| *agent == chip) {
                    sections.rows.push(row);
                }
            }
        }
        sections
            .rows
            .sort_by_key(|row| std::cmp::Reverse(row.updated_at));
        sections
    }

    /// Whether the chosen folder, if any, keeps `row` (#836).
    fn folder_keeps(&self, row: &Row) -> bool {
        self.folder
            .as_ref()
            .is_none_or(|folder| row.folder.as_ref() == Some(folder))
    }

    /// WHO: All and each agent, a row with its tile, its name and how many listed conversations
    /// it keeps (#919); a row is a toggle.
    fn render_chips(
        &self,
        chips: &[Chip],
        counts: &[usize],
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let total: usize = counts.iter().sum();
        let all = who_row(
            SharedString::new_static("marley-threads-chip-all"),
            item_kind::tile(ItemKind::Page, SharedString::new_static("*"), px(22.), cx),
            SharedString::new_static("All"),
            total,
            self.agent.is_none(),
            cx,
        )
        .on_click(cx.listener(|page, _, _, cx| {
            page.agent = None;
            cx.notify();
        }));
        v_flex()
            .w_full()
            .gap_0p5()
            .child(
                Label::new("WHO")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(all)
            .children(chips.iter().enumerate().map(|(index, chip)| {
                let chosen = chip.clone();
                let (name, kind, mark): (SharedString, ItemKind, AgentMark) = match chip {
                    Chip::Agent(agent) => {
                        let mark = AgentMark::from_agent_id(agent);
                        (
                            agents::thread_agent_name(agent, project, cx),
                            ItemKind::Thread(mark),
                            mark,
                        )
                    }
                    Chip::Cli(agent) => {
                        let mark = AgentMark::from_cli(*agent);
                        (cli_name(*agent).into(), ItemKind::AgentTerminal(mark), mark)
                    }
                };
                who_row(
                    SharedString::from(format!("marley-threads-chip-{index}")),
                    item_kind::tile(kind, mark.glyph(), px(22.), cx),
                    name,
                    counts.get(index).copied().unwrap_or_default(),
                    self.agent.as_ref() == Some(chip),
                    cx,
                )
                .on_click(cx.listener(move |page, _, _, cx| {
                    page.agent = Some(chosen.clone());
                    cx.notify();
                }))
            }))
    }

    /// The window's rail, observed while it is the window's, as Home follows it (#713, #919).
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

    /// FOLDER: All and each folder the listed conversations ran in (#836).
    fn render_folders(&self, folders: &[String], cx: &Context<Self>) -> impl IntoElement {
        let all = Button::new("marley-threads-folder-all", "All")
            .style(ButtonStyle::Subtle)
            .toggle_state(self.folder.is_none())
            .on_click(cx.listener(|page, _, _, cx| {
                page.folder = None;
                cx.notify();
            }));
        v_flex()
            .w_full()
            .items_start()
            .gap_0p5()
            .child(
                Label::new("FOLDER")
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(all)
            .children(folders.iter().enumerate().map(|(index, folder)| {
                let chosen = folder.clone();
                let full: SharedString = folder.clone().into();
                // The folder's own name, its whole path on hover: paths run past the column.
                let name = std::path::Path::new(folder).file_name().map_or_else(
                    || folder.clone(),
                    |name| name.to_string_lossy().into_owned(),
                );
                Button::new(("marley-threads-folder", index), name)
                    .style(ButtonStyle::Subtle)
                    .truncate(true)
                    .tooltip(Tooltip::text(full))
                    .toggle_state(self.folder.as_ref() == Some(folder))
                    .on_click(cx.listener(move |page, _, _, cx| {
                        page.folder = Some(chosen.clone());
                        cx.notify();
                    }))
            }))
    }

    /// The selected row's preview, or what to do with none (#836).
    fn render_preview(
        &self,
        row: Option<&Row>,
        live: &Live,
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let Some(row) = row else {
            return muted("Pick a conversation to see it here.");
        };
        let state = live.state_of(row, cx);
        let facts = live.facts_of(row, state.as_ref(), cx);
        let workspace = self.workspace.clone();
        let (open_target, split_target) = (row.target.clone(), row.target.clone());
        let split_workspace = self.workspace.clone();
        let thread = match &row.target {
            Target::Thread { thread_id, .. } => Some(*thread_id),
            Target::Cli(_) => None,
        };
        v_flex()
            .debug_selector(|| "marley-threads-preview".into())
            .w_full()
            .min_w_0()
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(row.icon(project, cx).color(Color::Muted))
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(row.detail(project, cx))
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate_start(),
                        ),
                    ),
            )
            .child(Headline::new(row.title.clone()).size(HeadlineSize::XSmall))
            // The state in its colour, in place of the last change, which the card keeps (D5).
            .children(state.map(|state| {
                let words = match (state.state, state.ask) {
                    (ItemState::NeedsYou, Some(ask)) => format!("Needs you \u{b7} {ask}"),
                    (ItemState::NeedsYou, None) => "Needs you".to_string(),
                    (ItemState::Working, _) => "Working".to_string(),
                    (ItemState::Done, _) => "Done".to_string(),
                    (ItemState::Failed, _) => "Failed".to_string(),
                    (ItemState::Idle, _) => "Idle".to_string(),
                };
                Label::new(words)
                    .size(LabelSize::Small)
                    .color(state.state.line(cx))
            }))
            .child(facts_grid(&facts, cx))
            .children(row.snippet.clone().map(|snippet| {
                Label::new(snippet)
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .italic()
            }))
            .child(
                h_flex()
                    .gap_1()
                    .flex_wrap()
                    .child(
                        Button::new("marley-threads-preview-open", "Open")
                            .style(ButtonStyle::Tinted(ui::TintColor::Accent))
                            .on_click(move |_, window, cx| {
                                open_row(&workspace, &open_target, window, cx);
                            }),
                    )
                    .child(
                        Button::new("marley-threads-preview-split", "Open in a Split")
                            .style(ButtonStyle::Subtle)
                            .on_click(move |_, window, cx| {
                                let split = split_workspace.update(cx, |workspace, cx| {
                                    let pane = workspace.active_pane().clone();
                                    let right = workspace.split_pane(
                                        pane,
                                        workspace::SplitDirection::Right,
                                        window,
                                        cx,
                                    );
                                    window.focus(&right.focus_handle(cx), cx);
                                });
                                if split.log_err().is_some() {
                                    open_row(&split_workspace, &split_target, window, cx);
                                }
                            }),
                    )
                    .children(thread.map(|thread_id| {
                        Button::new("marley-threads-preview-archive", "Archive")
                            .style(ButtonStyle::Subtle)
                            .on_click(move |_, _, cx| {
                                if let Some(store) = ThreadMetadataStore::try_global(cx) {
                                    store
                                        .update(cx, |store, cx| store.archive(thread_id, None, cx));
                                }
                            })
                    })),
            )
            .into_any_element()
    }

    /// The States board's cards for an empty list (#857): no conversations yet, with New Agent,
    /// or none matching the search, with Clear the search.
    fn render_empty(&self, cx: &Context<Self>) -> AnyElement {
        let query = self.search.read(cx).text(cx);
        let state = if query.trim().is_empty() {
            crate::states::EmptyState::new(
                "Th",
                "No conversations yet",
                "Start an agent and its thread shows here.",
            )
            .action("New Agent", crate::prompts::Role::Primary, |window, cx| {
                window.dispatch_action(Box::new(agent_ui::NewThread), cx);
            })
        } else {
            let search = self.search.downgrade();
            crate::states::EmptyState::new(
                "?",
                format!("Nothing matches \u{201c}{}\u{201d}", query.trim()),
                "The search looks at each conversation's title and its words.",
            )
            .action(
                "Clear the search",
                crate::prompts::Role::Plain,
                move |window, cx| {
                    search
                        .update(cx, |search, cx| search.set_text("", window, cx))
                        .log_err();
                },
            )
        };
        div().py_4().child(state.render(0, cx)).into_any_element()
    }

    fn render_section(
        &self,
        heading: &'static str,
        rows: Vec<Row>,
        live: &Live,
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> AnyElement {
        if rows.is_empty() {
            return Empty.into_any_element();
        }
        v_flex()
            .w_full()
            .gap_1p5()
            .child(
                Label::new(heading)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .children(
                rows.into_iter()
                    .map(|row| self.render_row(row, live, project, cx)),
            )
            .into_any_element()
    }

    /// A conversation as the board's card (#919): its agent's tile with the state's dot, its title
    /// and time on one line, the search's snippet, and its agent and folder as pills.
    fn render_row(
        &self,
        row: Row,
        live: &Live,
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors();
        let state = live.state_of(&row, cx).map(|state| state.state);
        let (kind, mark, agent): (ItemKind, AgentMark, SharedString) = match &row.target {
            Target::Thread { agent, .. } => {
                let mark = AgentMark::from_agent_id(agent);
                (
                    ItemKind::Thread(mark),
                    mark,
                    agents::thread_agent_name(agent, project, cx),
                )
            }
            Target::Cli(session) => {
                let mark = AgentMark::from_cli(session.agent);
                (
                    ItemKind::AgentTerminal(mark),
                    mark,
                    cli_name(session.agent).into(),
                )
            }
        };
        // A plain closure over a weak handle: opening a conversation updates the workspace, which
        // reads its items, this page among them (PR-claude-701).
        let workspace = self.workspace.clone();
        let target = row.target.clone();
        let key = row.key();
        let selected = self.selected.as_ref() == Some(&key);
        let page = cx.weak_entity();
        h_flex()
            .id(SharedString::from(format!("marley-threads-row-{key}")))
            .w_full()
            .items_start()
            .gap_2p5()
            .p_2p5()
            .rounded(px(12.))
            .border_1()
            .cursor_pointer()
            .map(|card| {
                if selected {
                    card.bg(colors.element_selected)
                        .border_color(colors.border_focused)
                } else {
                    card.border_color(colors.border_variant)
                        .hover(|style| style.bg(colors.ghost_element_hover))
                }
            })
            .child(card_tile(kind, mark.glyph(), state, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Label::new(row.title).truncate()),
                            )
                            .child(
                                Label::new(row.when)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .children(row.snippet.map(|snippet| {
                        Label::new(snippet)
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .italic()
                            .truncate()
                    }))
                    .child(
                        h_flex()
                            .gap_1()
                            .child(card_pill(agent, cx))
                            .children(row.folder.map(|folder| card_pill(folder.into(), cx))),
                    ),
            )
            // A click selects for the preview; a double-click opens, as Open does (#836).
            .on_click(move |event, window, cx| {
                if event.click_count() >= 2 {
                    open_row(&workspace, &target, window, cx);
                } else {
                    page.update(cx, |page, cx| {
                        page.selected = Some(key.clone());
                        cx.notify();
                    })
                    .log_err();
                }
            })
            .into_any_element()
    }
}

/// Opens what a row stands for: a conversation the page's way, or a CLI session with its resume
/// (#754).
pub(crate) fn open_row(
    workspace: &WeakEntity<Workspace>,
    target: &Target,
    window: &mut Window,
    cx: &mut App,
) {
    match target {
        Target::Thread { thread_id, .. } => open_thread(workspace, *thread_id, window, cx),
        Target::Cli(session) => crate::cli_sessions::reopen(workspace, session, window, cx),
    }
}

/// Whether `record`'s title or one of its folders holds `query`, which is lowercase.
fn matches(record: &ThreadMetadata, query: &str) -> bool {
    query.is_empty()
        || record.display_title().to_lowercase().contains(query)
        || record
            .folder_paths()
            .paths()
            .iter()
            .any(|path| path.to_string_lossy().to_lowercase().contains(query))
}

/// A folder as the page shows it: under the home folder as `~/…`.
fn shown_folder(path: &std::path::Path) -> String {
    path.strip_prefix(util::paths::home_dir()).map_or_else(
        |_| path.display().to_string(),
        |rest| {
            if rest.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~/{}", rest.display())
            }
        },
    )
}

/// Opens the conversation `thread_id`: its tab comes forward wherever in the window it is, or a
/// panel that holds it shows it, in the Marley layout moving it into a tab of that panel's
/// workspace (#772); otherwise it opens in a tab of `workspace` (#697's one-place rule: a thread
/// is never shown twice).
pub(crate) fn open_thread(
    workspace: &WeakEntity<Workspace>,
    thread_id: ThreadId,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return;
    };
    // Opening an archived thread brings it back to the lists, wherever it shows (#777).
    crate::thread_tab::unarchive(thread_id, cx);
    let members: Vec<Entity<Workspace>> = multi_workspace.read(cx).workspaces().cloned().collect();
    let in_tab = members.iter().find(|member| {
        crate::thread_tab::tab_threads(member.read(cx), cx)
            .iter()
            .any(|(id, _)| *id == thread_id)
    });
    if let Some(member) = in_tab.cloned() {
        multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.activate(member.clone(), None, window, cx);
        });
        member.update(cx, |member, cx| {
            crate::thread_tab::activate_for(member, thread_id, window, cx);
        });
        return;
    }
    let in_panel = members.iter().find_map(|member| {
        let panel = member.read(cx).panel::<AgentPanel>(cx)?;
        let holds = panel.read(cx).active_thread_id(cx) == Some(thread_id)
            || panel.read(cx).is_retained_thread(&thread_id);
        holds.then(|| (member.clone(), panel))
    });
    if let Some((member, panel)) = in_panel {
        multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.activate(member.clone(), None, window, cx);
        });
        if crate::marley_layout(cx) {
            member.update(cx, |member, cx| {
                crate::thread_tab::show_here(member, thread_id, window, cx);
            });
            return;
        }
        panel.update(cx, |panel, cx| {
            panel.activate_retained_thread(thread_id, true, window, cx);
        });
        member.update(cx, |member, cx| {
            member.focus_panel::<AgentPanel>(window, cx)
        });
        return;
    }
    let record = ThreadMetadataStore::try_global(cx)
        .and_then(|store| store.read(cx).entry(thread_id).cloned());
    let Some(record) = record else {
        return;
    };
    workspace
        .update(cx, |workspace, cx| {
            crate::thread_tab::open_saved(workspace, &record, window, cx);
        })
        .log_err();
}

impl ThreadsPage {
    /// The search field, with its magnifier (#919).
    fn render_search(&self, cx: &App) -> impl IntoElement {
        h_flex()
            .w_full()
            .gap_2()
            .px_2()
            .py_1()
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_md()
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::Small)
                    .color(Color::Muted),
            )
            .child(div().flex_1().min_w_0().child(self.search.clone()))
    }

    /// The rows under their day's heading, newest first, and the archived ones behind their toggle
    /// (#836).
    fn render_list(
        &self,
        rows: Vec<Row>,
        archived_rows: Vec<Row>,
        empty: bool,
        live: &Live,
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let archived_count = archived_rows.len();
        let archived_open = self.archived_open;
        let mut days: Vec<(&'static str, Vec<Row>)> = Vec::new();
        for row in rows {
            let day = day_of(row.updated_at);
            match days.last_mut() {
                Some((last, rows)) if *last == day => rows.push(row),
                _ => days.push((day, vec![row])),
            }
        }
        let archived = if archived_open {
            self.render_section("ARCHIVED", archived_rows, live, project, cx)
        } else {
            Empty.into_any_element()
        };
        v_flex()
            .id("marley-threads-list")
            // The board's `flex: 999 1 480px`: the list takes the room the other two leave.
            .flex_basis(px(480.))
            .flex_grow(999.)
            .flex_shrink_1()
            .min_w_0()
            // At most the row's height, not all of it: wrapped, the preview sits right under the
            // list rather than a screen below.
            .max_h_full()
            .overflow_y_scroll()
            .gap_3()
            .when(empty, |list| list.child(self.render_empty(cx)))
            .children(
                days.into_iter()
                    .map(|(day, rows)| self.render_section(day, rows, live, project, cx)),
            )
            .when(archived_count > 0, |list| {
                let label = if archived_open {
                    format!("Hide archived ({archived_count})")
                } else {
                    format!("Show archived ({archived_count})")
                };
                list.child(
                    Button::new("marley-threads-archived", label)
                        .style(ButtonStyle::Subtle)
                        .on_click(cx.listener(|page, _, _, cx| {
                            page.archived_open = !page.archived_open;
                            cx.notify();
                        })),
                )
            })
            .child(archived)
            .into_any_element()
    }
}

impl Render for ThreadsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(project) = self
            .workspace
            .upgrade()
            .map(|workspace| workspace.read(cx).project().clone())
        else {
            return div().into_any_element();
        };
        let live = Live {
            states: self
                .rail(window, cx)
                .map(|rail| rail.read(cx).conversation_states()),
            loaded: loaded_conversations(window, cx),
        };
        let colors = cx.theme().colors();
        let sections = self.sections(cx);
        let open = sections
            .rows
            .iter()
            .filter(|row| live.is_open(row, cx))
            .count();
        let listed = sections.rows.len();
        let empty = sections.rows.is_empty() && sections.archived.is_empty();
        let selected = self.selected.as_ref().and_then(|key| {
            sections
                .rows
                .iter()
                .chain(sections.archived.iter())
                .find(|row| row.key() == *key)
        });
        let preview = self.render_preview(selected, &live, &project, cx);
        // The board's columns (#906): each flexes from its basis, and the row wraps, so in a narrow
        // pane the preview drops under the list rather than squeezing it.
        let filters = v_flex()
            .flex_basis(px(230.))
            .flex_grow_0()
            .flex_shrink_1()
            .min_w(px(200.))
            .overflow_hidden()
            .gap_4()
            .child(self.render_chips(&sections.chips, &sections.counts, &project, cx))
            .child(self.render_folders(&sections.folders, cx));
        let list = self.render_list(sections.rows, sections.archived, empty, &live, &project, cx);
        v_flex()
            .id("marley-threads")
            .debug_selector(|| "marley-threads".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .p_4()
            .gap_3()
            .bg(colors.editor_background)
            .child(render_header(listed, open, cx))
            .child(self.render_search(cx))
            .child(
                h_flex()
                    .id("marley-threads-columns")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex_wrap()
                    // Wrapped in a narrow pane, the columns stack and scroll as one.
                    .overflow_y_scroll()
                    .items_start()
                    .gap_4()
                    .child(filters)
                    .child(list)
                    .child(
                        div()
                            .flex_basis(px(380.))
                            .flex_grow_0()
                            .flex_shrink_1()
                            .min_w(px(300.))
                            .overflow_hidden()
                            .pl_4()
                            .border_l_1()
                            .border_color(colors.border)
                            .child(preview),
                    ),
            )
            .into_any_element()
    }
}

/// The page's header (#919): "Threads", the count of conversations listed and of those open in
/// the window, and New Agent.
fn render_header(listed: usize, open: usize, cx: &App) -> impl IntoElement {
    h_flex()
        .w_full()
        .items_end()
        .gap_3()
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(Headline::new("Threads").size(HeadlineSize::Small))
                .child(
                    Label::new(format!(
                        "{listed} conversation{} \u{b7} {open} open",
                        if listed == 1 { "" } else { "s" }
                    ))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
                ),
        )
        .child(
            crate::prompts::render_button(0, "New Agent", crate::prompts::Role::Primary, false, cx)
                .on_click(|_, window, cx| {
                    window.dispatch_action(Box::new(agents::NewAgent { folder: None }), cx);
                }),
        )
}

/// What the window knows of the conversations live: the rail's states, and the conversations
/// loaded in its tabs and panels by thread key (#919, D1, D6).
struct Live {
    states: Option<ConversationStates>,
    loaded: HashMap<String, Entity<ConversationView>>,
}

/// The preview's facts; "—" for one the window does not have (D6).
struct Facts {
    folder: String,
    files: String,
    started: String,
    mode: String,
}

impl Live {
    /// `row`'s state: a thread's from the rail by its key, a CLI session's from the terminal that
    /// runs it; idle when the rail does not show it, none without a rail.
    fn state_of(&self, row: &Row, cx: &App) -> Option<ConversationState> {
        let states = self.states.as_ref()?;
        let found = match &row.target {
            Target::Thread { .. } => states.threads.get(&row.key()),
            Target::Cli(session) => {
                crate::agent_events::view_running(session.agent, &session.id, cx)
                    .and_then(|view| states.terminals.get(&view.as_u64()))
            }
        };
        Some(found.cloned().unwrap_or(ConversationState {
            state: ItemState::Idle,
            ask: None,
            bypass: false,
        }))
    }

    /// Whether `row` is loaded in this window: a thread in a tab or a panel, a CLI session running
    /// in a terminal (D2).
    fn is_open(&self, row: &Row, cx: &App) -> bool {
        match &row.target {
            Target::Thread { .. } => self.loaded.contains_key(&row.key()),
            Target::Cli(session) => {
                crate::agent_events::view_running(session.agent, &session.id, cx).is_some()
            }
        }
    }

    /// The preview's facts for `row`: its folder and start from the store, its changed files and
    /// mode from its loaded conversation, or a CLI's bypass from its terminal (D6).
    fn facts_of(&self, row: &Row, state: Option<&ConversationState>, cx: &App) -> Facts {
        let none = || "\u{2014}".to_string();
        let started = match &row.target {
            Target::Thread { thread_id, .. } => ThreadMetadataStore::try_global(cx)
                .and_then(|store| store.read(cx).entry(*thread_id)?.created_at)
                .map(when),
            Target::Cli(_) => None,
        };
        let view = self
            .loaded
            .get(&row.key())
            .and_then(|conversation| conversation.read(cx).root_thread_view());
        let files = view.as_ref().map(|view| {
            let changed = crate::thread_tab::changed_files(view.read(cx).thread.read(cx), cx);
            let added: u32 = changed.iter().map(|file| file.added).sum();
            let removed: u32 = changed.iter().map(|file| file.removed).sum();
            format!("{} \u{b7} +{added} \u{2212}{removed}", changed.len())
        });
        let mode = view
            .as_ref()
            .and_then(|view| view.read(cx).marley_session_facts(cx).mode)
            .map(String::from)
            .or_else(|| {
                state
                    .filter(|state| state.bypass)
                    .map(|_| "bypass".to_string())
            });
        Facts {
            folder: row.folder.clone().unwrap_or_else(none),
            files: files.unwrap_or_else(none),
            started: started.unwrap_or_else(none),
            mode: mode.unwrap_or_else(none),
        }
    }
}

/// The conversations loaded in the window's workspaces, by thread key (#919, D2).
fn loaded_conversations(window: &Window, cx: &App) -> HashMap<String, Entity<ConversationView>> {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return HashMap::default();
    };
    let mut loaded = HashMap::default();
    for workspace in multi_workspace.read(cx).workspaces() {
        for conversation in crate::thread_tab::conversations_of(workspace.read(cx), cx) {
            let key = conversation.read(cx).parent_id().to_key_string();
            loaded.insert(key, conversation);
        }
    }
    loaded
}

/// The preview's facts in a 2×2 grid, a muted label over each value (#919).
fn facts_grid(facts: &Facts, cx: &App) -> AnyElement {
    let cell = |label: &'static str, value: &str| {
        v_flex()
            .flex_1()
            .min_w_0()
            .p_2()
            .rounded_md()
            .bg(cx.theme().colors().element_background)
            .child(
                Label::new(label)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .child(
                Label::new(value.to_string())
                    .size(LabelSize::Small)
                    .truncate(),
            )
    };
    v_flex()
        .w_full()
        .gap_1p5()
        .child(
            h_flex()
                .gap_1p5()
                .child(cell("Folder", &facts.folder))
                .child(cell("Files changed", &facts.files)),
        )
        .child(
            h_flex()
                .gap_1p5()
                .child(cell("Started", &facts.started))
                .child(cell("Mode", &facts.mode)),
        )
        .into_any_element()
}

/// A WHO row: its tile, its name, and its count at the right; filled while chosen (#919).
fn who_row(
    id: SharedString,
    tile: Div,
    name: SharedString,
    count: usize,
    chosen: bool,
    cx: &App,
) -> gpui::Stateful<Div> {
    let colors = cx.theme().colors();
    h_flex()
        .id(id)
        .w_full()
        .gap_2()
        .px_1p5()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .map(|row| {
            if chosen {
                row.bg(colors.element_selected)
            } else {
                row.hover(|style| style.bg(colors.ghost_element_hover))
            }
        })
        .child(tile)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(Label::new(name).size(LabelSize::Small).truncate()),
        )
        .child(
            Label::new(count.to_string())
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
}

/// A card's agent tile, 32 px, its state's dot at the corner (#919, D4).
fn card_tile(kind: ItemKind, glyph: SharedString, state: Option<ItemState>, cx: &App) -> Div {
    let colors = cx.theme().colors();
    div()
        .relative()
        .flex_none()
        .size(px(32.))
        .rounded(px(8.))
        .bg(kind.tint(cx))
        .flex()
        .items_center()
        .justify_center()
        .child(
            Label::new(glyph)
                .size(LabelSize::Small)
                .buffer_font(cx)
                .color(Color::Custom(kind.ink(cx))),
        )
        .children(state.map(|state| {
            div()
                .absolute()
                .right(px(-2.))
                .bottom(px(-2.))
                .size(px(10.))
                .rounded_full()
                .border_2()
                .border_color(colors.editor_background)
                .bg(state.dot(cx))
        }))
}

/// A card's pill: an agent's name or a folder.
fn card_pill(text: SharedString, cx: &App) -> Div {
    h_flex()
        .h(px(20.))
        .px_2()
        .min_w_0()
        .rounded_full()
        .border_1()
        .border_color(cx.theme().colors().border_variant)
        .child(
            Label::new(text)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .truncate(),
        )
}

impl Focusable for ThreadsPage {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for ThreadsPage {}

impl Item for ThreadsPage {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Threads")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Thread))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Every agent conversation, from every project and group",
        ))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}

/// Reopen Thread… (#772): the saved conversations in the modal layer.
struct ReopenThreadPicker {
    picker: Entity<Picker<ReopenThreadDelegate>>,
}

impl ReopenThreadPicker {
    fn new(
        workspace: WeakEntity<Workspace>,
        project: Entity<Project>,
        rows: Vec<Row>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = ReopenThreadDelegate {
            modal: cx.entity().downgrade(),
            workspace,
            project,
            rows,
            matches: Vec::new(),
            selected_index: 0,
        };
        let picker = cx.new(|cx| Picker::uniform_list(delegate, window, cx));
        Self { picker }
    }
}

impl ModalView for ReopenThreadPicker {}

impl EventEmitter<DismissEvent> for ReopenThreadPicker {}

impl Focusable for ReopenThreadPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for ReopenThreadPicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("ReopenThreadPicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

struct ReopenThreadDelegate {
    modal: WeakEntity<ReopenThreadPicker>,
    /// The workspace the picker opened in, where a conversation with no open tab opens.
    workspace: WeakEntity<Workspace>,
    project: Entity<Project>,
    rows: Vec<Row>,
    matches: Vec<StringMatch>,
    selected_index: usize,
}

impl PickerDelegate for ReopenThreadDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley reopen thread"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        "Reopen a conversation…".into()
    }

    fn no_matches_text(&self, _: &mut Window, _: &mut App) -> Option<SharedString> {
        Some(SharedString::new_static("No saved conversation"))
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        // The title first, so a match's positions up to its length fall in the title.
        let candidates: Vec<StringMatchCandidate> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let text = row.folder.as_ref().map_or_else(
                    || row.title.to_string(),
                    |folder| format!("{} {folder}", row.title),
                );
                StringMatchCandidate::new(index, &text)
            })
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps the newest first; fuzzy matching would sort by score.
            let matches = if query.is_empty() {
                candidates
                    .into_iter()
                    .map(|candidate| StringMatch {
                        candidate_id: candidate.id,
                        score: 0.,
                        positions: Vec::new(),
                        string: candidate.string,
                    })
                    .collect()
            } else {
                let cancelled = AtomicBool::new(false);
                fuzzy::match_strings(&candidates, &query, false, true, 100, &cancelled, executor)
                    .await
            };
            picker
                .update(cx, |picker, cx| {
                    picker.delegate.matches = matches;
                    picker.delegate.selected_index = 0;
                    cx.notify();
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let target = self
            .matches
            .get(self.selected_index)
            .and_then(|found| self.rows.get(found.candidate_id))
            .map(|row| row.target.clone());
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
        let Some(target) = target else {
            return;
        };
        let workspace = self.workspace.clone();
        // The open reads every workspace, this one included, and may show a modal (PR-claude-735).
        window.defer(cx, move |window, cx| {
            open_row(&workspace, &target, window, cx);
        });
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let found = self.matches.get(index)?;
        let row = self.rows.get(found.candidate_id)?;
        let icon = row.icon(&self.project, cx);
        let detail = row.detail(&self.project, cx);
        let title_end = row.title.len();
        let positions = found
            .positions
            .iter()
            .copied()
            .filter(|position| *position < title_end)
            .collect();
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .start_slot(icon.color(Color::Muted))
                // Two lines, as the page's rows: on one, a long folder took the title's width.
                .child(
                    v_flex()
                        .min_w_0()
                        .child(HighlightedLabel::new(row.title.clone(), positions).truncate())
                        .child(
                            Label::new(detail)
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                )
                .end_slot(
                    Label::new(row.when.clone())
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
    }
}
