//! The Brain tab (#678): Rusty's vault with a navigation of its own, as the Tasks tab has its lists.
//!
//! The Brain view (#644), which took the rail's place until #678, sits in a column on the tab's
//! left: the search, the favourites and the tree. The page it opens shows on the tab's right, one
//! page view the tab navigates, with its own back, forward, Edit, outline and star. The tab opens
//! in the window's Rusty group (#675), one per window.

use std::any::TypeId;
use std::sync::Arc;

use gpui::{
    AnyEntity, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels,
    Render, SharedString, Subscription, WeakEntity, Window, px,
};
use marley_rusty::page::Visit;
use ui::prelude::*;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use super::brain::BrainView;
use super::page::{PageEvent, PageView};

/// The navigation column's width, the Tasks tab's lists' (#658).
const NAVIGATION_WIDTH: Pixels = px(280.);

/// Opens the window's Brain tab, or brings it forward, once the update in progress ends; it opens
/// in the Rusty group, and while Rusty is off or not connected a toast says why instead.
pub(crate) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    super::in_rusty_group(workspace, window, cx, |workspace, window, cx| {
        open(workspace, window, cx);
    });
}

/// Opens the Brain tab on page `slug`, for the home page's rows (#679).
pub(crate) fn open_page_later(
    workspace: WeakEntity<Workspace>,
    slug: String,
    window: &Window,
    cx: &mut App,
) {
    super::in_rusty_group(workspace, window, cx, move |workspace, window, cx| {
        if let Some(tab) = open(workspace, window, cx) {
            // `show_page` reads the workspace, which this runs inside the update of.
            window.defer(cx, move |window, cx| {
                tab.update(cx, |tab, cx| tab.show_page(slug, true, window, cx));
            });
        }
    });
}

/// Opens the Brain tab and shows today's note in it, which Rusty makes when it is missing.
pub(crate) fn open_today_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    super::in_rusty_group(workspace, window, cx, |workspace, window, cx| {
        if let Some(tab) = open(workspace, window, cx) {
            tab.update(cx, |tab, cx| tab.today(window, cx));
        }
    });
}

/// Brings the workspace's Brain tab forward with the focus in its navigation, or adds one to the
/// active pane.
fn open(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Option<Entity<BrainTab>> {
    if !super::capture::ready(workspace, cx) {
        return None;
    }
    let shown = workspace.items_of_type::<BrainTab>(cx).next();
    if let Some(open) = shown {
        workspace.activate_item(&open, true, true, window, cx);
        return Some(open);
    }
    let weak_workspace = cx.weak_entity();
    let tab = cx.new(|cx| BrainTab::new(weak_workspace, window, cx));
    workspace.add_item_to_active_pane(Box::new(tab.clone()), None, true, window, cx);
    Some(tab)
}

/// The Brain tab: the vault's navigation on the left, the page it opened on the right.
pub(crate) struct BrainTab {
    workspace: WeakEntity<Workspace>,
    navigation: Entity<BrainView>,
    page: Option<Entity<PageView>>,
    focus_handle: FocusHandle,
    /// The page's tab events, which the tab passes on.
    page_events: Option<Subscription>,
}

/// The tab's events, which Zed's pane reads as item events.
pub(crate) enum BrainTabEvent {
    /// The page it shows changed, which its followers read through the tab.
    UpdateTab,
}

impl BrainTab {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let multi_workspace = window
            .root::<workspace::MultiWorkspace>()
            .flatten()
            .map_or_else(WeakEntity::new_invalid, |multi_workspace| {
                multi_workspace.downgrade()
            });
        let host = cx.weak_entity();
        let navigation = cx.new(|cx| BrainView::new(multi_workspace, host, window, cx));
        Self {
            workspace,
            navigation,
            page: None,
            focus_handle: cx.focus_handle(),
            page_events: None,
        }
    }

    /// The page the tab shows, which the Knowledge panel, the Graph tab and the page picker read
    /// as the page in front.
    pub(crate) const fn page(&self) -> Option<&Entity<PageView>> {
        self.page.as_ref()
    }

    /// Shows page `slug` on the tab's right: the tab's page view is made the first time and
    /// navigated after, so back and forward walk the pages the tab showed. `focus` moves the
    /// keyboard into the page.
    pub(crate) fn show_page(
        &mut self,
        slug: String,
        focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        super::page_picker::opened(&slug, cx);
        let page = if let Some(page) = &self.page {
            page.update(cx, |page, cx| page.navigate(Visit::page(slug), window, cx));
            page.clone()
        } else {
            let Some(languages) = self
                .workspace
                .upgrade()
                .map(|workspace| Arc::clone(workspace.read(cx).project().read(cx).languages()))
            else {
                return;
            };
            let workspace = self.workspace.clone();
            let page =
                cx.new(|cx| PageView::new(Visit::page(slug), workspace, languages, window, cx));
            self.page_events = Some(cx.subscribe(&page, |_, _, event: &PageEvent, cx| {
                if matches!(event, PageEvent::UpdateTab) {
                    cx.emit(BrainTabEvent::UpdateTab);
                }
            }));
            self.page = Some(page.clone());
            page
        };
        if focus {
            window.focus(&page.focus_handle(cx), cx);
        }
        cx.emit(BrainTabEvent::UpdateTab);
        cx.notify();
    }

    /// Today: today's note, made when missing, revealed in the tree and shown on the right.
    fn today(&self, window: &Window, cx: &mut Context<Self>) {
        self.navigation
            .update(cx, |_, cx| BrainView::today(window, cx));
    }
}

impl EventEmitter<BrainTabEvent> for BrainTab {}

impl Focusable for BrainTab {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.navigation.focus_handle(cx)
    }
}

impl Render for BrainTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        h_flex()
            .id("marley-brain-tab")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.editor_background)
            .child(
                v_flex()
                    .debug_selector(|| "marley-brain-tab-navigation".into())
                    .flex_none()
                    .w(NAVIGATION_WIDTH)
                    .h_full()
                    .border_r_1()
                    .border_color(colors.border)
                    .bg(colors.panel_background)
                    .child(self.navigation.clone()),
            )
            .child(
                div()
                    .debug_selector(|| "marley-brain-tab-page".into())
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .map(|area| match &self.page {
                        Some(page) => area.child(page.clone()),
                        None => {
                            area.child(
                                v_flex().size_full().items_center().justify_center().child(
                                    Label::new("Pick a page on the left.").color(Color::Muted),
                                ),
                            )
                        }
                    }),
            )
    }
}

impl Item for BrainTab {
    type Event = BrainTabEvent;

    /// The page on the right answers for the tab where a follower asks for a page (#678).
    fn act_as_type<'a>(
        &'a self,
        type_id: TypeId,
        self_handle: &'a Entity<Self>,
        _: &'a App,
    ) -> Option<AnyEntity> {
        if type_id == TypeId::of::<Self>() {
            Some(self_handle.clone().into())
        } else if type_id == TypeId::of::<PageView>() {
            self.page.as_ref().map(|page| page.clone().into())
        } else {
            None
        }
    }

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Brain")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::BookCopy))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        match event {
            BrainTabEvent::UpdateTab => f(ItemEvent::UpdateTab),
        }
    }
}

impl std::fmt::Debug for BrainTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrainTab")
            .field("page", &self.page.is_some())
            .finish_non_exhaustive()
    }
}
