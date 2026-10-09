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

use agent_ui::thread_metadata_store::ThreadId;
use agent_ui::{AgentPanel, AgentPanelEvent, AgentThreadSource, ConversationView};
use gpui::{
    Action, App, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString, Subscription,
    Window, actions,
};
use ui::prelude::*;
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

/// A center tab showing an Agent Panel thread.
pub struct ThreadTab {
    conversation_view: Entity<ConversationView>,
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

pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenThreadInCenter, window, cx| {
            open_in_center(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &MoveThreadToPanel, window, cx| {
            move_to_panel(workspace, window, cx);
        });
    })
    .detach();
}

impl ThreadTab {
    fn new(
        conversation_view: Entity<ConversationView>,
        panel: &Entity<AgentPanel>,
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
    let tab = cx.new(|cx| ThreadTab::new(view.clone(), &panel, cx));
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

fn refuse(workspace: &mut Workspace, message: &'static str, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ThreadTabToast>(), message),
        cx,
    );
}
