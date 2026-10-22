//! Marley's pages as buttons (#739, #746, #778), and the Debug panel's button (#774).
//!
//! Home at the right of the status bar; Rusty's and the Marley agent's conversations in the title
//! bar beside the guide's `?` (#746: Chad, 2026-10-11, "Rusty and Marley go to the top right next
//! to (?) and Avatar instead"; #778: Chad, 2026-10-12, "The top right rusty button is suppose to
//! open the rusty agent"). Each opens its tab in the group the window shows, in either layout.
//! Chad, 2026-10-10: "they just open in a new tab." Rusty's home page and its button left when Home
//! took Rusty's cards (#782). Threads left for the rail's header and the Home page (#745). In the
//! Marley layout the Debug panel's button sits at the status bar's left (#774).

use std::sync::Arc;

use gpui::{
    App, AppContext as _, Context, Entity, Focusable as _, Render, Subscription, WeakEntity, Window,
};
use project::DisableAiSettings;
use settings::{Settings as _, SettingsStore};
use title_bar::{MarleyProjectMark, MarleyTitleBarButtons, MarleyTitleBarCenter};
use ui::{IconButtonShape, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::dock::{Dock, PanelButtons, PanelHandle};
use workspace::{HideStatusItem, ItemHandle, StatusItemView, Workspace};

/// The Debug panel's persistent name. The panel stays in its dock and is found by this name, so the
/// crate needs no `debugger_ui`.
const DEBUG_PANEL: &str = "DebugPanel";

/// Adds Home and Debug to every workspace's status bar, and hands the title bar
/// the view that draws Rusty and Marley. [`crate::init`] calls it once, before any window opens.
pub(crate) fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let Some(window) = window else {
            return;
        };
        let handle = workspace.weak_handle();
        let handle_for_focus = handle.clone();
        let home = cx.new(|_| HomeButton { workspace: handle });
        let docks = workspace.all_docks().map(Entity::clone);
        let debug = cx.new(|cx| DebugButton::new(docks, cx));
        let focus = cx.new(|cx| crate::focus_row::FocusRow::new(handle_for_focus, cx));
        let forge = cx.new(crate::forge_watch::ForgeItem::new);
        let panels = cx.new(|cx| crate::panel_group::PanelGroup::new(workspace, cx));
        workspace.status_bar().update(cx, |status_bar, cx| {
            // Every dock's panels, named, after Home (#862): the right items draw from the end, so
            // the group goes in first.
            status_bar.add_right_item(panels, window, cx);
            status_bar.add_right_item(home, window, cx);
            // Debug at the left end, after the left dock's buttons, which `Workspace::new` adds
            // before any other status item.
            match status_bar.position_of_item::<PanelButtons>() {
                Some(position) => status_bar.insert_item_after(position, debug, window, cx),
                None => status_bar.add_left_item(debug, window, cx),
            }
            // What has the focus, right after Debug (#833).
            match status_bar.position_of_item::<DebugButton>() {
                Some(position) => status_bar.insert_item_after(position, focus, window, cx),
                None => status_bar.add_left_item(focus, window, cx),
            }
            // Forge's agents and its newest QA failure, after what has the focus (#843).
            match status_bar.position_of_item::<crate::focus_row::FocusRow>() {
                Some(position) => status_bar.insert_item_after(position, forge, window, cx),
                None => status_bar.add_left_item(forge, window, cx),
            }
        });
    })
    .detach();
    cx.set_global(MarleyTitleBarButtons(Arc::new(|workspace, _, cx| {
        cx.new(|cx| TitleButtons::new(workspace, cx)).into()
    })));
    cx.set_global(MarleyProjectMark(Arc::new(|workspace, _, cx| {
        cx.new(|cx| TitleProjectMark::new(workspace, cx)).into()
    })));
    cx.set_global(MarleyTitleBarCenter(Arc::new(|_, _, cx| {
        cx.new(|cx| {
            cx.observe_global::<SettingsStore>(|_, cx| cx.notify())
                .detach();
            TitleCenter
        })
        .into()
    })));
}

/// The palette field's width in the title bar (#834).
const PALETTE_FIELD_WIDTH: f32 = 440.;

/// The title bar's middle in the Marley layout (#834): a field that opens the command palette.
#[derive(Debug)]
pub struct TitleCenter;

impl Render for TitleCenter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if crate::MarleySettings::get_global(cx).layout != settings::MarleyLayout::Marley {
            return div().into_any_element();
        }
        let colors = cx.theme().colors();
        h_flex()
            .id("marley-title-palette")
            .debug_selector(|| "marley-title-palette".into())
            // Centered on the title bar's middle: the hook's slot starts there.
            .w(px(PALETTE_FIELD_WIDTH))
            .ml(px(-PALETTE_FIELD_WIDTH / 2.))
            .h(px(26.))
            .px_2()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(colors.border_variant)
            .bg(colors.element_background)
            .hover(|style| style.bg(colors.element_hover))
            .cursor_pointer()
            .child(
                Icon::new(IconName::MagnifyingGlass)
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            )
            .child(
                div().flex_1().min_w_0().child(
                    Label::new("Search, run a command, or ask an agent")
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
            )
            .child(ui::KeyBinding::for_action(
                &zed_actions::command_palette::Toggle,
                cx,
            ))
            .on_click(|_, window, cx| {
                window.dispatch_action(Box::new(zed_actions::command_palette::Toggle), cx);
            })
            .into_any_element()
    }
}

/// The title bar's project mark (#831): the shown project's own image before its name. A project
/// with only its letter shows nothing here, since its name beside the mark says the same.
pub struct TitleProjectMark {
    workspace: WeakEntity<Workspace>,
    project_events: Option<Subscription>,
}

impl std::fmt::Debug for TitleProjectMark {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TitleProjectMark")
            .finish_non_exhaustive()
    }
}

impl TitleProjectMark {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<crate::project_icons::ProjectMarks>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<SettingsStore>(|_, cx| cx.notify())
            .detach();
        // The title bar makes this view while the workspace is being updated, so the workspace
        // is read only once that update is over. The hook runs once per title bar, so a folder
        // added or removed is followed here.
        let view = cx.weak_entity();
        cx.defer(move |cx| {
            view.update(cx, Self::follow_project).log_err();
        });
        Self {
            workspace,
            project_events: None,
        }
    }

    /// Follows the workspace's project for folders added or removed.
    fn follow_project(&mut self, cx: &mut Context<Self>) {
        let project = self
            .workspace
            .upgrade()
            .map(|workspace| workspace.read(cx).project().clone());
        self.project_events = project.map(|project| {
            cx.subscribe(&project, |_, _, event: &project::Event, cx| {
                if matches!(
                    event,
                    project::Event::WorktreeAdded(_) | project::Event::WorktreeRemoved(_)
                ) {
                    cx.notify();
                }
            })
        });
        cx.notify();
    }

    /// The shown project's first folder, while the project is local (a remote one's mark is its
    /// letter until #860).
    fn root(&self, cx: &App) -> Option<std::path::PathBuf> {
        let key = self.workspace.upgrade()?.read(cx).project_group_key(cx);
        if key.host().is_some() {
            return None;
        }
        key.path_list().paths().first().cloned()
    }
}

impl Render for TitleProjectMark {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(root) = self.root(cx) else {
            return div().into_any_element();
        };
        let wanted = root.clone();
        cx.defer(move |cx| crate::project_icons::want(&wanted, cx));
        match crate::project_icons::mark(&root, cx) {
            crate::project_icons::ProjectMark::Image(image) => div()
                .debug_selector(|| "marley-title-project-mark".into())
                .flex_none()
                .pr_1()
                .child(gpui::img(image).size_4().rounded_sm())
                .into_any_element(),
            crate::project_icons::ProjectMark::Letter => div().into_any_element(),
        }
    }
}

/// The status bar's Home button, drawn only while its workspace is shown.
pub struct HomeButton {
    workspace: WeakEntity<Workspace>,
}

impl std::fmt::Debug for HomeButton {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("HomeButton").finish_non_exhaustive()
    }
}

impl Render for HomeButton {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // A plain closure over the weak workspace: opening a tab updates the workspace, which reads
        // its items (PR-claude-701).
        let workspace = self.workspace.clone();
        IconButton::new("marley-status-home", IconName::MarleyHome)
            .icon_size(IconSize::Small)
            .shape(IconButtonShape::Square)
            .tooltip(Tooltip::text("Home"))
            .on_click(move |_, window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        crate::home_page::open_here(workspace, window, cx);
                    })
                    .log_err();
            })
    }
}

impl StatusItemView for HomeButton {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// Home is Marley's own page, always there.
    fn hide_setting(&self, _cx: &App) -> Option<HideStatusItem> {
        None
    }
}

/// The Debug panel's button at the status bar's left, in the Marley layout (#774).
///
/// Chad, 2026-10-11: "yes debug moves to bottom left and stays the exception". The panel stays in
/// its dock; the button is drawn and clicked as the dock's own `PanelButtons` draws it, which the
/// Marley layout's `debugger.button` default turns off.
pub struct DebugButton {
    docks: [Entity<Dock>; 3],
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for DebugButton {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DebugButton")
            .finish_non_exhaustive()
    }
}

impl DebugButton {
    fn new(docks: [Entity<Dock>; 3], cx: &mut Context<Self>) -> Self {
        // What the button's look rests on: the docks (open, and on which panel) and the settings
        // (the layout, `debugger.button`).
        let mut subscriptions: Vec<Subscription> = docks
            .iter()
            .map(|dock| cx.observe(dock, |_, _, cx| cx.notify()))
            .collect();
        subscriptions.push(cx.observe_global::<SettingsStore>(|_, cx| cx.notify()));
        Self {
            docks,
            _subscriptions: subscriptions,
        }
    }

    /// The dock holding the Debug panel, and the panel.
    fn debug_panel(&self, cx: &App) -> Option<(Entity<Dock>, Arc<dyn PanelHandle>)> {
        self.docks.iter().find_map(|dock| {
            let panel = Arc::clone(dock.read(cx).panel_for_persistent_name(DEBUG_PANEL)?);
            Some((dock.clone(), panel))
        })
    }
}

/// Whether the user's own settings turn the Debug button off, as Hide Button does.
fn debug_button_hidden_by_user(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.debugger.as_ref())
        .and_then(|debugger| debugger.button)
        == Some(false)
}

impl Render for DebugButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some((dock, panel)) = self.debug_panel(cx) else {
            return div().into_any_element();
        };
        // Zed's own button draws while `debugger.button` is on: a user who turns it on gets that
        // one among the dock's buttons, not a second here.
        if !crate::marley_layout(cx)
            || panel.icon(window, cx).is_some()
            || debug_button_hidden_by_user(cx)
        {
            return div().into_any_element();
        }
        let (shown, focus_handle, close) = {
            let dock = dock.read(cx);
            let shown = dock.is_open()
                && dock
                    .active_panel()
                    .is_some_and(|active| active.panel_id() == panel.panel_id());
            (shown, dock.focus_handle(cx), dock.toggle_action())
        };
        let action = if shown {
            close
        } else {
            panel.toggle_action(window, cx)
        };
        IconButton::new(("marley-status-debug", u64::from(shown)), IconName::Debug)
            .icon_size(IconSize::Small)
            .toggle_state(shown)
            .on_click({
                let action = action.boxed_clone();
                move |_, window, cx| {
                    window.focus(&focus_handle, cx);
                    window.dispatch_action(action.boxed_clone(), cx);
                }
            })
            .when(!shown, |button| {
                button.tooltip(move |_, cx| Tooltip::for_action("Debug Panel", &*action, cx))
            })
            .into_any_element()
    }
}

impl StatusItemView for DebugButton {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    /// The Debug panel's own Hide Button, which writes `debugger.button: false`.
    fn hide_setting(&self, cx: &App) -> Option<HideStatusItem> {
        let (_, panel) = self.debug_panel(cx)?;
        panel.hide_button_setting(cx)
    }
}

/// Rusty's and the Marley agent's conversations in one workspace's title bar (#746, #778).
pub struct TitleButtons {
    workspace: WeakEntity<Workspace>,
    /// The window's rail, which the agent pills count from (#834), observed once found.
    rail: Option<(WeakEntity<crate::Rail>, Subscription)>,
}

impl std::fmt::Debug for TitleButtons {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TitleButtons")
            .finish_non_exhaustive()
    }
}

impl TitleButtons {
    fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        // What each button's presence rests on: the settings (AI, Rusty's switch), Rusty's
        // connection, and the Marley and Rusty entries.
        cx.observe_global::<SettingsStore>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::rusty::Rusty>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::assistant::Assistant>(|_, cx| cx.notify())
            .detach();
        Self {
            workspace,
            rail: None,
        }
    }
}

impl TitleButtons {
    /// The window's rail, observed once found, as Home and the bottom row find it.
    fn rail(&mut self, window: &Window, cx: &mut Context<Self>) -> Option<Entity<crate::Rail>> {
        let multi_workspace = window.root::<workspace::MultiWorkspace>().flatten()?;
        let rail = multi_workspace
            .read(cx)
            .sidebar()?
            .to_any()
            .downcast::<crate::Rail>()
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

    /// The agents at work and those waiting on the user, counted as pills (#834).
    fn agent_pills(&mut self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        if crate::MarleySettings::get_global(cx).layout != settings::MarleyLayout::Marley {
            return None;
        }
        let rail = self.rail(window, cx)?;
        let at_work = rail.read(cx).agents_at_work();
        let working: Vec<String> = at_work
            .iter()
            .filter(|agent| agent.state == "working")
            .map(|agent| format!("{} · {}", agent.title, agent.project))
            .collect();
        let waiting = at_work
            .iter()
            .filter(|agent| matches!(agent.state, "waiting" | "failed"))
            .count();
        if working.is_empty() && waiting == 0 {
            return None;
        }
        let status = cx.theme().status();
        let pill = |id: &'static str, color: gpui::Hsla, text: String| {
            h_flex()
                .id(id)
                .debug_selector(move || id.to_string())
                .h(px(22.))
                .px_2()
                .gap_1p5()
                .rounded_full()
                .bg(color.opacity(0.15))
                .child(div().size_1p5().rounded_full().bg(color))
                .child(
                    Label::new(text)
                        .size(LabelSize::XSmall)
                        .color(Color::Custom(color)),
                )
        };
        let working_words = match working.len() {
            1 => "1 agent working".to_string(),
            count => format!("{count} agents working"),
        };
        let names = working.join("\n");
        let rail = rail.downgrade();
        Some(
            h_flex()
                .gap_1()
                .mr_1()
                .when(!working.is_empty(), |pills| {
                    pills.child(
                        pill("marley-title-working", status.warning, working_words)
                            .tooltip(Tooltip::text(names)),
                    )
                })
                .when(waiting > 0, |pills| {
                    pills.child(
                        pill(
                            "marley-title-needs-you",
                            status.conflict,
                            format!("{waiting} needs you"),
                        )
                        .cursor_pointer()
                        .tooltip(Tooltip::text("Open the oldest request"))
                        .on_click(move |_, window, cx| {
                            rail.update(cx, |rail, cx| {
                                if let Some(key) =
                                    rail.inbox().first().map(|entry| entry.key.clone())
                                {
                                    rail.open_inbox_entry(&key, window, cx);
                                }
                            })
                            .log_err();
                        }),
                    )
                })
                .into_any_element(),
        )
    }
}

impl Render for TitleButtons {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pills = self.agent_pills(window, cx);
        let ai = !DisableAiSettings::get_global(cx).disable_ai;
        // Rusty's entry is there only while Rusty is on and connected and an agent runs it.
        let rusty = ai && crate::assistant::rusty_present(cx);
        let marley = ai && crate::assistant::marley_present(cx);
        let rusty_workspace = self.workspace.clone();
        let marley_workspace = self.workspace.clone();
        h_flex()
            .gap_1()
            .children(pills)
            .when(rusty, |buttons| {
                buttons.child(
                    IconButton::new("marley-title-rusty", crate::rusty::RUSTY_ICON)
                        .icon_size(IconSize::Small)
                        .tooltip(|_, cx| {
                            Tooltip::for_action("Talk to Rusty", &crate::assistant::TalkToRusty, cx)
                        })
                        .on_click(move |_, window, cx| {
                            rusty_workspace
                                .update(cx, |workspace, cx| {
                                    crate::assistant::talk_to_rusty(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
            })
            .when(marley, |buttons| {
                buttons.child(
                    IconButton::new("marley-title-marley", IconName::MarleyCat)
                        .icon_size(IconSize::Small)
                        .tooltip(|_, cx| {
                            Tooltip::for_action(
                                "Talk to Marley",
                                &crate::assistant::TalkToMarley,
                                cx,
                            )
                        })
                        .on_click(move |_, window, cx| {
                            marley_workspace
                                .update(cx, |workspace, cx| {
                                    crate::assistant::talk_to_marley(workspace, window, cx);
                                })
                                .log_err();
                        }),
                )
            })
    }
}
