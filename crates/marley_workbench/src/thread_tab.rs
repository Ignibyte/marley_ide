//! An Agent Panel thread in a center tab (#697).
//!
//! `marley: open thread in center` moves the panel's active thread into a tab of the center pane:
//! the same `ConversationView`, still running, hosted as Zed's own tests host one
//! (`ThreadViewItem`). The panel turns to a new draft with `clear_base_view`, which keeps the
//! thread among its retained threads, so the view is drawn in one place. `marley: move thread to
//! panel`, from the palette or the tab's right-click menu, makes it the panel's again, or loads it
//! from its history once the panel has let it go, and the tab closes. The tab also closes when the
//! panel shows its thread again from the panel's own history, and the rail brings an open tab
//! forward instead of opening its thread in the panel.
//!
//! A thread can also start in a tab (#734): `marley::NewAgentThread` builds the `ConversationView`
//! itself, on the hosting workspace's project and its panel's connection store, working in a
//! folder Marley names. Zed answers an agent's file calls through the project and refuses a path
//! outside its worktrees, so a folder no worktree holds joins the project as a hidden worktree,
//! except the home folder, which Zed would scan and watch whole. The view is marked
//! (`marley_own_folders`) so Zed's thread store files the thread under that folder, and
//! `MarleyThreadHost` tells Zed when a tab shows a thread, so it neither notifies about a thread in
//! front nor opens a second copy in the panel from a notification.
//!
//! A tab comes back after a restart (#736): it saves its thread's id, agent and folders in its own
//! table, and restoring it loads the thread as the Agent Panel loads one from its history, once
//! the workspace's panel exists.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent::ThreadStore;
use agent_ui::thread_metadata_store::{ThreadId, ThreadMetadata, ThreadMetadataStore};
use agent_ui::{
    Agent, AgentConnectionStore, AgentPanel, AgentPanelEvent, AgentThreadSource, ConversationView,
    MarleyThreadHost,
};
use anyhow::Context as _;
use collections::HashMap;
use fs::Fs;
use gpui::{
    Action, App, AppContext as _, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    Global, SharedString, Subscription, Task, WeakEntity, Window, actions,
};
use project::{AgentId, DisableAiSettings, Project, Worktree};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::Settings as _;
use ui::prelude::*;
use util::ResultExt as _;
use util::path_list::PathList;
use workspace::item::{Item, ItemEvent, SerializableItem};
use workspace::notifications::NotificationId;
use workspace::{ItemId, Toast, Workspace, WorkspaceId};

use persistence::MarleyThreadTabsDb;

actions!(
    marley,
    [
        /// Moves the Agent Panel's thread into a tab of the center pane.
        #[derive(Eq)]
        OpenThreadInCenter,
        /// Moves the thread in the active center tab back into the Agent Panel.
        #[derive(Eq)]
        MoveThreadToPanel
    ]
);

/// Starts a thread in a center tab of the shown group (#734).
///
/// `agent` is an agent's id as the Agent Panel's New Thread menu runs it (`Zed Agent`,
/// `claude-acp`, `Marley`); without it, the panel's selected agent. `folder` is the folder the
/// thread works in; without it, the shown project's root, else the home folder.
#[derive(Clone, Debug, PartialEq, Eq, JsonSchema, Action)]
#[action(namespace = marley)]
pub struct NewAgentThread {
    /// The agent's id; none runs the Agent Panel's selected agent.
    pub agent: Option<String>,
    /// The folder the thread works in; none takes the shown project's root, else the home folder.
    pub folder: Option<PathBuf>,
}

/// `NewAgentThread`'s fields as a keymap writes them, read through this struct for the reason
/// `rusty::OpenPage` gives (clippy's `unsafe_derive_deserialize` on a derived `Deserialize`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAgentThreadFields {
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    folder: Option<PathBuf>,
}

impl<'de> Deserialize<'de> for NewAgentThread {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let NewAgentThreadFields { agent, folder } =
            NewAgentThreadFields::deserialize(deserializer)?;
        Ok(Self { agent, folder })
    }
}

/// A center tab showing an agent thread: one moved from the Agent Panel (#697), or one started here
/// (#734).
pub struct ThreadTab {
    conversation_view: Entity<ConversationView>,
    /// The hidden worktree of the thread's folder, when the tab made one (#734): a project holds a
    /// hidden worktree only weakly, so the tab keeps it while it shows the thread.
    _folder: Option<Entity<Worktree>>,
    _subscriptions: [Subscription; 2],
}

impl std::fmt::Debug for ThreadTab {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ThreadTab")
            .field("conversation_view", &self.conversation_view.entity_id())
            .finish_non_exhaustive()
    }
}

/// The toast a refused move shows.
struct ThreadTabToast;

pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyThreadHost {
        shows: shows_view,
        reveal: activate_for,
    });
    load_saved_tabs(cx);
    workspace::register_serializable_item::<ThreadTab>(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenThreadInCenter, window, cx| {
            open_in_center(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &MoveThreadToPanel, window, cx| {
            move_to_panel(workspace, window, cx);
        });
        workspace.register_action(|workspace, action: &NewAgentThread, window, cx| {
            let agent = action.agent.clone().map(AgentId::new);
            start(workspace, agent, action.folder.clone(), window, cx);
        });
    })
    .detach();
}

impl ThreadTab {
    /// The key of the thread the tab shows, as the rail keys its thread rows (#702).
    pub(crate) fn thread_key(&self, cx: &App) -> String {
        self.conversation_view.read(cx).parent_id().to_key_string()
    }

    /// What a restart needs to show this tab again: its thread, agent and folders (#736). `None`
    /// while the thread has no folders yet, before its agent connects and Zed records it.
    fn saved(&self, cx: &App) -> Option<SavedThreadTab> {
        let view = self.conversation_view.read(cx);
        let thread_id = view.parent_id();
        let folders = view
            .root_thread_view()
            .and_then(|thread| thread.read(cx).thread.read(cx).work_dirs().cloned())
            .or_else(|| {
                let store = ThreadMetadataStore::try_global(cx)?;
                let record = store.read(cx).entry(thread_id)?;
                Some(record.folder_paths().clone())
            })?;
        Some(SavedThreadTab {
            thread_id,
            agent: view.agent_key().id().0.to_string(),
            folders: folders.paths().to_vec(),
            own_folders: view.marley_own_folders,
            title: Some(view.title(cx).to_string()),
        })
    }

    fn new(
        conversation_view: Entity<ConversationView>,
        panel: &Entity<AgentPanel>,
        folder: Option<Entity<Worktree>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = [
            cx.observe(&conversation_view, |_, _, cx| {
                cx.emit(ItemEvent::UpdateTab);
                cx.notify();
            }),
            cx.subscribe(panel, |this, panel, event, cx| {
                if matches!(event, AgentPanelEvent::ActiveViewChanged)
                    && panel.read(cx).active_conversation_view() == Some(&this.conversation_view)
                {
                    cx.emit(ItemEvent::CloseItem);
                }
            }),
        ];
        Self {
            conversation_view,
            _folder: folder,
            _subscriptions: subscriptions,
        }
    }
}

impl EventEmitter<ItemEvent> for ThreadTab {}

impl Item for ThreadTab {
    type Event = ItemEvent;

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.conversation_view.read(cx).title(cx)
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ZedAssistant))
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<SharedString> {
        Some(
            format!(
                "Agent thread: {}",
                self.conversation_view.read(cx).title(cx)
            )
            .into(),
        )
    }

    fn tab_extra_context_menu_actions(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Vec<(SharedString, Box<dyn Action>)> {
        vec![(
            SharedString::new_static("Move Thread to Panel"),
            Box::new(MoveThreadToPanel),
        )]
    }
}

impl SerializableItem for ThreadTab {
    fn serialized_item_kind() -> &'static str {
        "MarleyThreadTab"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        cx.default_global::<SavedThreadTabs>()
            .0
            .retain(|(workspace, item), _| {
                *workspace != workspace_id || alive_items.contains(item)
            });
        workspace::delete_unloaded_items(
            alive_items,
            workspace_id,
            "marley_thread_tabs",
            &MarleyThreadTabsDb::global(cx),
            cx,
        )
    }

    fn deserialize(
        project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: WorkspaceId,
        item_id: ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        let Some(saved) = saved_tab(workspace_id, item_id, cx) else {
            return Task::ready(Err(anyhow::anyhow!("no thread tab was saved for the item")));
        };
        // A thread deleted since leaves no empty tab: Zed logs the refusal, and the cleanup
        // drops the row.
        let known = ThreadMetadataStore::try_global(cx)
            .is_some_and(|store| store.read(cx).entry(saved.thread_id).is_some());
        if !known {
            return Task::ready(Err(anyhow::anyhow!(
                "the thread of a saved tab is no longer kept"
            )));
        }
        window.spawn(cx, async move |cx| {
            // Zed adds the Agent Panel while it still restores items; its connection store is the
            // one a tab's thread shares (#734).
            let mut panel = None;
            for _ in 0..PANEL_WAITS {
                panel = workspace.read_with(cx, Workspace::panel::<AgentPanel>)?;
                if panel.is_some() {
                    break;
                }
                cx.background_executor().timer(PANEL_WAIT).await;
            }
            let panel = panel.context("the workspace has no Agent Panel for its thread tab")?;
            let hidden = if saved.own_folders
                && let Some(folder) = saved.folders.first()
                && cx.update(|_, cx| needs_worktree(&project, folder, cx))?
            {
                let (worktree, _) = project
                    .update(cx, |project, cx| {
                        project.find_or_create_worktree(folder, false, cx)
                    })
                    .await?;
                Some(worktree)
            } else {
                None
            };
            cx.update(|window, cx| {
                let fs = workspace
                    .upgrade()
                    .map(|workspace| Arc::clone(&workspace.read(cx).app_state().fs))
                    .context("the workspace closed before its thread tab was restored")?;
                let thread = ThreadSpec {
                    agent: Agent::from(AgentId::new(saved.agent)),
                    thread_id: saved.thread_id,
                    folders: PathList::new(&saved.folders),
                    title: saved.title.map(SharedString::from),
                    own_folders: saved.own_folders,
                };
                let view = build_view(project, workspace, &panel, thread, fs, window, cx);
                Ok(cx.new(|cx| Self::new(view, &panel, hidden, cx)))
            })?
        })
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: ItemId,
        _closing: bool,
        cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        let workspace_id = workspace.database_id()?;
        let tab = self.saved(cx)?;
        Some(save_tab(workspace_id, item_id, tab, cx))
    }

    fn should_serialize(&self, event: &ItemEvent) -> bool {
        matches!(event, ItemEvent::UpdateTab)
    }
}

/// How often, and how long apart, a restored tab looks for its workspace's Agent Panel: ten
/// seconds in all.
const PANEL_WAITS: usize = 100;
const PANEL_WAIT: Duration = Duration::from_millis(100);

/// A thread tab as a restart reads it (#736).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SavedThreadTab {
    thread_id: ThreadId,
    /// The agent's id, as `Agent::id` gives it.
    agent: String,
    folders: Vec<PathBuf>,
    /// Whether Marley chose the folders (#734), so the first joins the project again when it
    /// needs to.
    own_folders: bool,
    title: Option<String>,
}

/// The saved tabs, read once at start, so `deserialize` finds one at once.
#[derive(Default)]
struct SavedThreadTabs(HashMap<(WorkspaceId, ItemId), SavedThreadTab>);

impl Global for SavedThreadTabs {}

fn load_saved_tabs(cx: &mut App) {
    let saved = MarleyThreadTabsDb::global(cx)
        .all_tabs()
        .log_err()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(workspace_id, item_id, state)| {
            let tab = serde_json::from_str::<SavedThreadTab>(&state)
                .map_err(|error| anyhow::anyhow!("a saved thread tab did not parse: {error}"))
                .log_err()?;
            Some(((workspace_id, item_id), tab))
        })
        .collect();
    cx.set_global(SavedThreadTabs(saved));
}

fn saved_tab(workspace_id: WorkspaceId, item_id: ItemId, cx: &App) -> Option<SavedThreadTab> {
    cx.try_global::<SavedThreadTabs>()?
        .0
        .get(&(workspace_id, item_id))
        .cloned()
}

/// Keeps the tab for the item, in memory and in the table.
fn save_tab(
    workspace_id: WorkspaceId,
    item_id: ItemId,
    tab: SavedThreadTab,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    let state = match serde_json::to_string(&tab) {
        Ok(state) => state,
        Err(error) => return Task::ready(Err(error.into())),
    };
    let _previous = cx
        .default_global::<SavedThreadTabs>()
        .0
        .insert((workspace_id, item_id), tab);
    let db = MarleyThreadTabsDb::global(cx);
    cx.background_spawn(async move { db.save_tab(item_id, workspace_id, state).await })
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;
    use workspace::{ItemId, WorkspaceDb, WorkspaceId};

    /// The thread tabs' own table (#736): each tab's thread, agent and folders as a JSON object, so
    /// a later field needs no migration. Keyed by workspace and item, with no `UNIQUE(item_id)`:
    /// ids repeat across launches (#576).
    pub(super) struct MarleyThreadTabsDb(ThreadSafeConnection);

    impl Domain for MarleyThreadTabsDb {
        const NAME: &str = stringify!(MarleyThreadTabsDb);

        const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE marley_thread_tabs (
                workspace_id INTEGER,
                item_id INTEGER,
                state TEXT NOT NULL,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
        )];
    }

    db::static_connection!(MarleyThreadTabsDb, [WorkspaceDb]);

    impl MarleyThreadTabsDb {
        query! {
            pub(super) async fn save_tab(
                item_id: ItemId,
                workspace_id: WorkspaceId,
                state: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_thread_tabs(item_id, workspace_id, state)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub(super) fn all_tabs() -> Result<Vec<(WorkspaceId, ItemId, String)>> {
                SELECT workspace_id, item_id, state
                FROM marley_thread_tabs
            }
        }
    }
}

impl Focusable for ThreadTab {
    /// The thread's message editor, as the Agent Panel focuses on activation: the view's own handle
    /// takes no typing.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        let view = self.conversation_view.read(cx);
        view.active_thread().map_or_else(
            || view.focus_handle(cx),
            |thread| thread.read(cx).message_editor.focus_handle(cx),
        )
    }
}

impl Render for ThreadTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.conversation_view.clone())
    }
}

/// `marley: open thread in center`: the panel's active thread into a center tab, or its tab
/// forward when it has one. An empty draft stays: there is no thread yet, and the panel would show
/// the same draft again.
pub(crate) fn open_in_center(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let Some(panel) = workspace.panel::<AgentPanel>(cx) else {
        refuse(workspace, "This window has no Agent Panel.", cx);
        return;
    };
    let Some(view) = panel.read(cx).active_conversation_view().cloned() else {
        refuse(workspace, "Open a thread in the Agent Panel first.", cx);
        return;
    };
    if panel.read(cx).active_thread_is_draft(cx) {
        refuse(
            workspace,
            "Send the new thread its first message, then open it in the center.",
            cx,
        );
        return;
    }
    let thread_id = view.read(cx).parent_id();
    if activate_for(workspace, thread_id, window, cx) {
        return;
    }
    panel.update(cx, |panel, cx| panel.clear_base_view(window, cx));
    let tab = cx.new(|cx| ThreadTab::new(view.clone(), &panel, None, cx));
    workspace.add_item_to_center(Box::new(tab.clone()), window, cx);
    window.focus(&tab.read(cx).focus_handle(cx), cx);
}

/// Opens `thread_id` in a center tab once the panel shows it: what the rail's Open in Center
/// runs after opening the thread as a click does.
pub(crate) fn open_thread_in_center(
    workspace: &mut Workspace,
    thread_id: ThreadId,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if activate_for(workspace, thread_id, window, cx) {
        return;
    }
    let shown = workspace.panel::<AgentPanel>(cx).is_some_and(|panel| {
        panel
            .read(cx)
            .active_conversation_view()
            .is_some_and(|view| view.read(cx).parent_id() == thread_id)
    });
    if shown {
        open_in_center(workspace, window, cx);
    }
}

/// `marley: move thread to panel`: the active center tab's thread back into the panel, which
/// still holds it, or loads it from its history when it let it go; then the tab closes.
fn move_to_panel(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let Some(tab) = workspace.active_item_as::<ThreadTab>(cx) else {
        refuse(workspace, "The active tab is not an agent thread.", cx);
        return;
    };
    let Some(panel) = workspace.panel::<AgentPanel>(cx) else {
        refuse(workspace, "This window has no Agent Panel.", cx);
        return;
    };
    let view = tab.read(cx).conversation_view.read(cx);
    let (thread_id, agent, title) = (view.parent_id(), view.agent_key().clone(), view.title(cx));
    let retained = panel.read(cx).is_retained_thread(&thread_id);
    panel.update(cx, |panel, cx| {
        if retained {
            panel.activate_retained_thread(thread_id, true, window, cx);
        } else {
            panel.load_agent_thread(
                agent,
                thread_id,
                None,
                Some(title),
                true,
                AgentThreadSource::AgentPanel,
                window,
                cx,
            );
        }
    });
    tab.update(cx, |_, cx| cx.emit(ItemEvent::CloseItem));
    workspace.focus_panel::<AgentPanel>(window, cx);
}

/// Brings forward the center tab showing `thread_id`, if one does.
pub(crate) fn activate_for(
    workspace: &mut Workspace,
    thread_id: ThreadId,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> bool {
    let Some(tab) = workspace
        .items_of_type::<ThreadTab>(cx)
        .find(|tab| tab.read(cx).conversation_view.read(cx).parent_id() == thread_id)
    else {
        return false;
    };
    workspace.activate_item(&tab, true, true, window, cx);
    true
}

/// Starts a thread of `agent` (the panel's selected agent when `None`) in a center tab of
/// `workspace`, working in `folder` (the workspace's first root, else the home folder, when
/// `None`). The folder joins the project as a hidden worktree first when it needs one.
pub(crate) fn start(
    workspace: &mut Workspace,
    agent: Option<AgentId>,
    folder: Option<PathBuf>,
    window: &Window,
    cx: &mut Context<Workspace>,
) {
    if DisableAiSettings::get_global(cx).disable_ai {
        return;
    }
    let Some(panel) = workspace.panel::<AgentPanel>(cx) else {
        refuse(workspace, "This window has no Agent Panel.", cx);
        return;
    };
    let agent = agent.map_or_else(|| panel.read(cx).selected_agent(cx), Agent::from);
    let folder = folder.unwrap_or_else(|| default_folder(workspace, cx));
    let project = workspace.project().clone();
    let joins = needs_worktree(&project, &folder, cx);
    let fs = Arc::clone(&workspace.app_state().fs);
    cx.spawn_in(window, async move |workspace, cx| {
        if !fs.is_dir(&folder).await {
            workspace
                .update(cx, |workspace, cx| {
                    refuse(
                        workspace,
                        format!("{} is not a folder.", folder.display()),
                        cx,
                    );
                })
                .log_err();
            return;
        }
        let mut hidden = None;
        if joins {
            let joined = project
                .update(cx, |project, cx| {
                    project.find_or_create_worktree(&folder, false, cx)
                })
                .await;
            match joined {
                Ok((worktree, _)) => hidden = Some(worktree),
                Err(error) => {
                    workspace
                        .update(cx, |workspace, cx| {
                            refuse(
                                workspace,
                                format!("Could not open {}: {error}", folder.display()),
                                cx,
                            );
                        })
                        .log_err();
                    return;
                }
            }
        }
        workspace
            .update_in(cx, |workspace, window, cx| {
                open_thread(workspace, agent, folder, hidden, fs, window, cx);
            })
            .log_err();
    })
    .detach();
}

/// Builds the thread's view, as the Agent Panel builds one, on `workspace`'s project and its
/// panel's connection store, and shows it in a new center tab.
fn open_thread(
    workspace: &mut Workspace,
    agent: Agent,
    folder: PathBuf,
    hidden: Option<Entity<Worktree>>,
    fs: Arc<dyn Fs>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let Some(panel) = workspace.panel::<AgentPanel>(cx) else {
        refuse(workspace, "This window has no Agent Panel.", cx);
        return;
    };
    let thread = ThreadSpec {
        agent,
        thread_id: ThreadId::new(),
        folders: PathList::new(&[folder]),
        title: None,
        own_folders: true,
    };
    let view = build_view(
        workspace.project().clone(),
        workspace.weak_handle(),
        &panel,
        thread,
        fs,
        window,
        cx,
    );
    let tab = cx.new(|cx| ThreadTab::new(view, &panel, hidden, cx));
    workspace.add_item_to_center(Box::new(tab.clone()), window, cx);
    window.focus(&tab.read(cx).focus_handle(cx), cx);
}

/// Opens a saved conversation in a tab of `workspace` (#737): its agent, folders and session, the
/// first folder joining the project hidden when it needs to, as a restored tab does (#736). A
/// conversation with no folder works in the workspace's own, and one of a remote project is
/// refused: its folders are on another machine.
pub(crate) fn open_saved(
    workspace: &mut Workspace,
    record: &ThreadMetadata,
    window: &Window,
    cx: &mut Context<Workspace>,
) {
    if record.remote_connection.is_some() {
        refuse(
            workspace,
            "A remote project's conversation opens from that project.",
            cx,
        );
        return;
    }
    if workspace.panel::<AgentPanel>(cx).is_none() {
        refuse(workspace, "This window has no Agent Panel.", cx);
        return;
    }
    let folders = if record.folder_paths().paths().is_empty() {
        PathList::new(&[default_folder(workspace, cx)])
    } else {
        record.folder_paths().clone()
    };
    let project = workspace.project().clone();
    let joining = folders
        .paths()
        .first()
        .filter(|folder| needs_worktree(&project, folder, cx))
        .cloned();
    let fs = Arc::clone(&workspace.app_state().fs);
    let thread = ThreadSpec {
        agent: Agent::from(record.agent_id.clone()),
        thread_id: record.thread_id,
        folders,
        title: Some(record.display_title()),
        own_folders: true,
    };
    cx.spawn_in(window, async move |workspace, cx| {
        let hidden = match joining {
            Some(folder) => {
                let joined = project
                    .update(cx, |project, cx| {
                        project.find_or_create_worktree(&folder, false, cx)
                    })
                    .await;
                match joined {
                    Ok((worktree, _)) => Some(worktree),
                    Err(error) => {
                        workspace
                            .update(cx, |workspace, cx| {
                                refuse(
                                    workspace,
                                    format!("Could not open {}: {error}", folder.display()),
                                    cx,
                                );
                            })
                            .log_err();
                        return;
                    }
                }
            }
            None => None,
        };
        workspace
            .update_in(cx, |workspace, window, cx| {
                let Some(panel) = workspace.panel::<AgentPanel>(cx) else {
                    return;
                };
                let view = build_view(
                    workspace.project().clone(),
                    workspace.weak_handle(),
                    &panel,
                    thread,
                    fs,
                    window,
                    cx,
                );
                let tab = cx.new(|cx| ThreadTab::new(view, &panel, hidden, cx));
                workspace.add_item_to_center(Box::new(tab.clone()), window, cx);
                window.focus(&tab.read(cx).focus_handle(cx), cx);
            })
            .log_err();
    })
    .detach();
}

/// What a tab's thread view is built from: a new thread (#734), or a saved one (#736).
struct ThreadSpec {
    agent: Agent,
    thread_id: ThreadId,
    folders: PathList,
    title: Option<SharedString>,
    /// Whether Marley chose the folders, so Zed's store files the thread under them.
    own_folders: bool,
}

/// The `ConversationView` for `thread`, as the Agent Panel builds one: the panel's connection
/// store, Zed's thread store for Zed's own agent, and the session the thread's record names, so a
/// saved thread loads its history.
fn build_view(
    project: Entity<Project>,
    workspace: WeakEntity<Workspace>,
    panel: &Entity<AgentPanel>,
    thread: ThreadSpec,
    fs: Arc<dyn Fs>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ConversationView> {
    let connection_store: Entity<AgentConnectionStore> = panel.read(cx).connection_store().clone();
    let threads = ThreadStore::global(cx);
    let server = thread.agent.server(fs, threads.clone());
    // Only Zed's own agent keeps its threads in Zed's thread store, as the panel passes it.
    let thread_store = thread.agent.is_native().then_some(threads);
    let session = ThreadMetadataStore::try_global(cx).and_then(|store| {
        store
            .read(cx)
            .entry(thread.thread_id)
            .and_then(|record| record.session_id.clone())
    });
    cx.new(|cx| {
        let mut view = ConversationView::new(
            server,
            connection_store,
            thread.agent,
            session,
            Some(thread.thread_id),
            Some(thread.folders),
            thread.title,
            None,
            workspace,
            project,
            thread_store,
            AgentThreadSource::Sidebar,
            window,
            cx,
        );
        view.marley_own_folders = thread.own_folders;
        view
    })
}

/// The folder a thread works in when none is named: the workspace's first root, else the home
/// folder, as a group with no folder starts its agent CLIs (#600).
pub(crate) fn default_folder(workspace: &Workspace, cx: &App) -> PathBuf {
    workspace.root_paths(cx).first().map_or_else(
        || util::paths::home_dir().clone(),
        |root| root.to_path_buf(),
    )
}

/// Whether `folder` must join `project` as a hidden worktree for the agent's file calls to reach
/// it: no worktree holds it, the project is this machine's, and it neither is nor holds the home
/// folder, which Zed would scan and watch whole.
fn needs_worktree(project: &Entity<Project>, folder: &Path, cx: &App) -> bool {
    let project = project.read(cx);
    project.is_local()
        && project.find_worktree(folder, cx).is_none()
        && !util::paths::home_dir().starts_with(folder)
}

/// The views of every thread `workspace` holds: its Agent Panel's, shown or kept, and those of its
/// thread tabs the panel does not hold (#734).
pub(crate) fn conversations_of(workspace: &Workspace, cx: &App) -> Vec<Entity<ConversationView>> {
    let mut views = workspace
        .panel::<AgentPanel>(cx)
        .map(|panel| panel.read(cx).conversation_views())
        .unwrap_or_default();
    for tab in workspace.items_of_type::<ThreadTab>(cx) {
        let view = tab.read(cx).conversation_view.clone();
        if !views.contains(&view) {
            views.push(view);
        }
    }
    views
}

/// The thread tabs of `workspace`, each with its thread's id: where the rail lists a thread in a
/// tab (#734).
pub(crate) fn tab_threads(
    workspace: &Workspace,
    cx: &App,
) -> Vec<(ThreadId, Entity<ConversationView>)> {
    workspace
        .items_of_type::<ThreadTab>(cx)
        .map(|tab| {
            let view = tab.read(cx).conversation_view.clone();
            (view.read(cx).parent_id(), view)
        })
        .collect()
}

/// `MarleyThreadHost::shows`: whether `workspace`'s active item is a thread tab on the view
/// `view_id`.
fn shows_view(workspace: &Workspace, view_id: EntityId, cx: &App) -> bool {
    workspace
        .active_item_as::<ThreadTab>(cx)
        .is_some_and(|tab| tab.read(cx).conversation_view.entity_id() == view_id)
}

fn refuse(
    workspace: &mut Workspace,
    message: impl Into<Cow<'static, str>>,
    cx: &mut Context<Workspace>,
) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ThreadTabToast>(), message),
        cx,
    );
}
