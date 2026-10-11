//! The Threads page (#737): every agent conversation Zed keeps, from every project, group and
//! folder, in a center tab.
//!
//! Zed keeps its thread history in its own sidebar (`ThreadsArchiveView`), which the Marley layout
//! replaces with the rail, so without this page an old conversation is found only under the
//! project it ran in. The page lists Zed's thread store, Marley's and Rusty's conversations first,
//! with a search over titles and folders and a chip per agent. A click brings a conversation forward
//! where it is open, else opens it in a tab of the page's group (#734, #736).

use agent_ui::AgentPanel;
use agent_ui::thread_metadata_store::{ThreadId, ThreadMetadata, ThreadMetadataStore};
use editor::{Editor, EditorEvent};
use gpui::{
    App, AppContext as _, Context, Empty, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Subscription, WeakEntity, Window, actions,
};
use project::{AgentId, DisableAiSettings, Project};
use settings::Settings as _;
use ui::{ButtonStyle, Headline, HeadlineSize, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{MultiWorkspace, Workspace};

use crate::agents::{self, AgentIcon};
use crate::rusty::home_tab::muted;

actions!(
    marley,
    [
        /// Opens the Threads page: every agent conversation, from every project and group,
        /// searchable, in a tab of the shown group.
        #[derive(Eq)]
        OpenThreads,
    ]
);

pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenThreads, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
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
    let handle = workspace.weak_handle();
    let page = cx.new(|cx| ThreadsPage::new(handle, window, cx));
    workspace.add_item_to_center(Box::new(page.clone()), window, cx);
    window.focus(&page.read(cx).search.focus_handle(cx), cx);
}

/// The page: the store's conversations, the search, and the agent the chips keep.
pub struct ThreadsPage {
    workspace: WeakEntity<Workspace>,
    search: Entity<Editor>,
    /// The agent whose chip is chosen; none lists every agent.
    agent: Option<AgentId>,
    /// Whether the archived conversations are listed.
    archived_open: bool,
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

/// One conversation as the page lists it.
struct Row {
    thread_id: ThreadId,
    agent: AgentId,
    title: SharedString,
    /// Its first folder, the home folder as `~`; none for a conversation with no folder.
    folder: Option<String>,
    when: String,
}

/// The page's rows, by section, and the agents the chips offer.
struct Sections {
    assistants: Vec<Row>,
    others: Vec<Row>,
    archived: Vec<Row>,
    agents: Vec<AgentId>,
}

impl ThreadsPage {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search conversations by title or folder…", window, cx);
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
        Self {
            workspace,
            search,
            agent: None,
            archived_open: false,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// The store's conversations the search and the chip keep, newest first, in their sections.
    fn sections(&self, cx: &App) -> Sections {
        let query = self.search.read(cx).text(cx).trim().to_lowercase();
        let mut sections = Sections {
            assistants: Vec::new(),
            others: Vec::new(),
            archived: Vec::new(),
            agents: Vec::new(),
        };
        let Some(store) = ThreadMetadataStore::try_global(cx) else {
            return sections;
        };
        let store = store.read(cx);
        let mut records: Vec<&ThreadMetadata> = store
            .entries()
            .filter(|record| !record.is_draft())
            .chain(store.archived_entries())
            .collect();
        records.sort_by_key(|record| std::cmp::Reverse(record.updated_at));
        for record in records {
            if !sections.agents.contains(&record.agent_id) {
                sections.agents.push(record.agent_id.clone());
            }
            if self
                .agent
                .as_ref()
                .is_some_and(|agent| *agent != record.agent_id)
                || !matches(record, &query)
            {
                continue;
            }
            let row = Row {
                thread_id: record.thread_id,
                agent: record.agent_id.clone(),
                title: record.display_title(),
                folder: record
                    .folder_paths()
                    .paths()
                    .first()
                    .map(|path| shown_folder(path)),
                when: record
                    .updated_at
                    .with_timezone(&chrono::Local)
                    .format("%b %-d, %H:%M")
                    .to_string(),
            };
            if record.archived {
                sections.archived.push(row);
            } else if crate::assistant::is_assistant(&record.agent_id) {
                sections.assistants.push(row);
            } else {
                sections.others.push(row);
            }
        }
        sections
    }

    fn render_chips(
        &self,
        agents: &[AgentId],
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let all = Button::new("marley-threads-chip-all", "All")
            .style(ButtonStyle::Subtle)
            .toggle_state(self.agent.is_none())
            .on_click(cx.listener(|page, _, _, cx| {
                page.agent = None;
                cx.notify();
            }));
        h_flex()
            .flex_wrap()
            .gap_1()
            .child(all)
            .children(agents.iter().enumerate().map(|(index, agent)| {
                let chosen = agent.clone();
                Button::new(
                    ("marley-threads-chip", index),
                    agents::thread_agent_name(agent, project, cx),
                )
                .style(ButtonStyle::Subtle)
                .toggle_state(self.agent.as_ref() == Some(agent))
                .on_click(cx.listener(move |page, _, _, cx| {
                    page.agent = Some(chosen.clone());
                    cx.notify();
                }))
            }))
    }

    fn render_section(
        &self,
        heading: &'static str,
        rows: Vec<Row>,
        project: &Entity<Project>,
        cx: &Context<Self>,
    ) -> AnyElement {
        if rows.is_empty() {
            return Empty.into_any_element();
        }
        v_flex()
            .w_full()
            .gap_0p5()
            .child(
                Label::new(heading)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .children(
                rows.into_iter()
                    .map(|row| self.render_row(row, project, cx)),
            )
            .into_any_element()
    }

    fn render_row(&self, row: Row, project: &Entity<Project>, cx: &Context<Self>) -> AnyElement {
        let icon = match agents::thread_icon(&row.agent, project, cx) {
            AgentIcon::Named(name) => Icon::new(name),
            AgentIcon::Svg(path) => Icon::from_external_svg(path),
        };
        let agent_name = agents::thread_agent_name(&row.agent, project, cx);
        let detail = row.folder.as_ref().map_or_else(
            || agent_name.to_string(),
            |folder| format!("{agent_name} · {folder}"),
        );
        // A plain closure over a weak handle: opening a conversation updates the workspace, which
        // reads its items, this page among them (PR-claude-701).
        let workspace = self.workspace.clone();
        let thread_id = row.thread_id;
        ListItem::new(SharedString::from(format!(
            "marley-threads-row-{}",
            row.thread_id.to_key_string()
        )))
        .spacing(ListItemSpacing::Sparse)
        .start_slot(icon.color(Color::Muted))
        .child(
            v_flex()
                .min_w_0()
                .child(Label::new(row.title).truncate())
                .child(
                    Label::new(detail)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
        )
        .end_slot(
            Label::new(row.when)
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .on_click(move |_, window, cx| {
            open_thread(&workspace, thread_id, window, cx);
        })
        .into_any_element()
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
/// panel that holds it shows it; otherwise it opens in a tab of `workspace` (#697's one-place
/// rule: a thread is never shown twice).
fn open_thread(
    workspace: &WeakEntity<Workspace>,
    thread_id: ThreadId,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return;
    };
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

impl Render for ThreadsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let Some(project) = self
            .workspace
            .upgrade()
            .map(|workspace| workspace.read(cx).project().clone())
        else {
            return div().into_any_element();
        };
        let sections = self.sections(cx);
        let empty = sections.assistants.is_empty()
            && sections.others.is_empty()
            && sections.archived.is_empty();
        let archived_count = sections.archived.len();
        let archived_open = self.archived_open;
        let archived = if archived_open {
            self.render_section("ARCHIVED", sections.archived, &project, cx)
        } else {
            Empty.into_any_element()
        };
        v_flex()
            .id("marley-threads")
            .debug_selector(|| "marley-threads".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(colors.editor_background)
            .child(Headline::new("Threads").size(HeadlineSize::Small))
            .child(
                div()
                    .w_full()
                    .px_2()
                    .py_1()
                    .border_1()
                    .border_color(colors.border)
                    .rounded_md()
                    .child(self.search.clone()),
            )
            .child(self.render_chips(&sections.agents, &project, cx))
            .when(empty, |page| page.child(muted("No conversation matches.")))
            .child(self.render_section("MARLEY AND RUSTY", sections.assistants, &project, cx))
            .child(self.render_section("ALL CONVERSATIONS", sections.others, &project, cx))
            .when(archived_count > 0, |page| {
                let label = if archived_open {
                    format!("Hide archived ({archived_count})")
                } else {
                    format!("Show archived ({archived_count})")
                };
                page.child(
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
