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

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent::ThreadStore;
use agent_ui::thread_metadata_store::ThreadId;
use agent_ui::{
    Agent, AgentConnectionStore, AgentPanel, AgentPanelEvent, AgentThreadSource, ConversationView,
    MarleyThreadHost,
};
use fs::Fs;
use gpui::{
    Action, App, Context, Entity, EntityId, EventEmitter, FocusHandle, Focusable, SharedString,
    Subscription, Window, actions,
};
use project::{AgentId, DisableAiSettings, Project, Worktree};
use schemars::JsonSchema;
use serde::Deserialize;
use settings::Settings as _;
use ui::prelude::*;
use util::ResultExt as _;
use util::path_list::PathList;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

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
    let connection_store: Entity<AgentConnectionStore> = panel.read(cx).connection_store().clone();
    let threads = ThreadStore::global(cx);
    let server = agent.server(fs, threads.clone());
    // Only Zed's own agent keeps its threads in Zed's thread store, as the panel passes it.
    let thread_store = agent.is_native().then_some(threads);
    let project = workspace.project().clone();
    let handle = workspace.weak_handle();
    let view = cx.new(|cx| {
        let mut view = ConversationView::new(
            server,
            connection_store,
            agent,
            None,
            Some(ThreadId::new()),
            Some(PathList::new(&[folder])),
            None,
            None,
            handle,
            project,
            thread_store,
            AgentThreadSource::Sidebar,
            window,
            cx,
        );
        view.marley_own_folders = true;
        view
    });
    let tab = cx.new(|cx| ThreadTab::new(view, &panel, hidden, cx));
    workspace.add_item_to_center(Box::new(tab.clone()), window, cx);
    window.focus(&tab.read(cx).focus_handle(cx), cx);
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
