//! Welcome to Marley (#912): what a first start shows in place of Zed's onboarding page, as the
//! Dialogs board draws it.
//!
//! Four rows of option cards, Layout, Theme, Agents and Rusty, each choice written to the user's
//! settings at once and the card in effect lit; then Open a Folder and Go to Home. The screen is a
//! center tab of the Home group, never saved, so a restart does not bring it back (D1).

use std::sync::Arc;

use anyhow::Result;
use db::kvp::KeyValueStore;
use fs::Fs;
use gpui::{
    App, AppContext as _, Context, EventEmitter, FocusHandle, Focusable, FontWeight, Render,
    SharedString, Task, WeakEntity, Window, img, px,
};
use marley_agent::AgentKind;
use marley_agent::versions::Found;
use settings::{MarleyLayout, ThemeName, ThemeSelection};
use ui::prelude::*;
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{AppState, Workspace};

use crate::claude_plugin::ClaudePlugin;

/// Opens a first start's window with Welcome to Marley in its center (#912), and notes the first
/// start under `first_open`, Zed's onboarding key, as Zed's page does, when the screen opens (D6).
///
/// # Errors
///
/// When the window cannot be opened.
pub fn show_first_run(
    app_state: Arc<AppState>,
    first_open: &'static str,
    cx: &mut App,
) -> Task<Result<()>> {
    workspace::open_new(
        workspace::OpenOptions::default(),
        app_state,
        cx,
        move |workspace, window, cx| {
            let welcome = cx.new(|cx| MarleyWelcome::new(workspace.weak_handle(), cx));
            workspace.add_item_to_center(Box::new(welcome.clone()), window, cx);
            window.focus(&welcome.focus_handle(cx), cx);
            let kvp = KeyValueStore::global(cx);
            db::write_and_log(cx, move || async move {
                kvp.write_kvp(first_open.to_string(), "false".to_string())
                    .await
            });
        },
    )
}

/// `marley: show welcome`: the screen forward in `workspace`, or a new one in its center.
pub(crate) fn open_here(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace.items_of_type::<MarleyWelcome>(cx).next();
    if let Some(welcome) = open {
        workspace.activate_item(&welcome, true, true, window, cx);
        return;
    }
    let welcome = cx.new(|cx| MarleyWelcome::new(workspace.weak_handle(), cx));
    workspace.add_item_to_center(Box::new(welcome.clone()), window, cx);
    window.focus(&welcome.focus_handle(cx), cx);
}

/// The welcome screen.
pub struct MarleyWelcome {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
}

impl MarleyWelcome {
    fn new(workspace: WeakEntity<Workspace>, cx: &Context<Self>) -> Self {
        Self {
            workspace,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Closes the screen's tab (D5).
    fn close(cx: &mut Context<Self>) {
        cx.emit(ItemEvent::CloseItem);
    }
}

/// The Theme row's choices (#912).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Night,
    Day,
    /// Follow Omarchy's theme (#759).
    Omarchy,
}

impl ThemeChoice {
    /// The choice in effect: Omarchy while the setting follows it, else the theme's appearance.
    fn current(cx: &App) -> Self {
        if crate::omarchy_theme::wanted(cx) {
            Self::Omarchy
        } else if cx.theme().appearance().is_light() {
            Self::Day
        } else {
            Self::Night
        }
    }

    /// Writes the choice: Marley's theme by name with Omarchy off, or Omarchy on.
    fn choose(self, cx: &App) {
        settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
            let follow = self == Self::Omarchy;
            content.marley.get_or_insert_default().omarchy_theme = Some(follow);
            let name = match self {
                Self::Night => "Marley Night",
                Self::Day => "Marley Day",
                Self::Omarchy => return,
            };
            content.theme.theme = Some(ThemeSelection::Static(ThemeName(name.into())));
        });
    }
}

/// A row of option cards under its title.
struct WelcomeRow {
    title: &'static str,
    options: Vec<WelcomeOption>,
}

/// What a card's click writes.
type Choose = Box<dyn Fn(&mut App)>;

/// One option card: its name, a line about it, whether it is in effect, and what a click does.
struct WelcomeOption {
    name: &'static str,
    about: SharedString,
    on: bool,
    choose: Option<Choose>,
}

impl WelcomeOption {
    fn new(name: &'static str, about: impl Into<SharedString>, on: bool) -> Self {
        Self {
            name,
            about: about.into(),
            on,
            choose: None,
        }
    }

    fn on_choose(mut self, choose: impl Fn(&mut App) + 'static) -> Self {
        self.choose = Some(Box::new(choose));
        self
    }
}

/// What Marley found of an agent CLI, as its card says it.
fn agent_found(kind: AgentKind, cx: &App) -> String {
    match crate::agent_versions::found(kind, cx) {
        Some(Found::Version(version)) => format!("Found: {version}"),
        Some(Found::Missing(_)) => "Not on the PATH".to_string(),
        Some(Found::Unreadable(_)) => "Found; its version did not read".to_string(),
        None => "Looking…".to_string(),
    }
}

impl MarleyWelcome {
    fn rows(cx: &App) -> Vec<WelcomeRow> {
        let layout = crate::marley_layout(cx);
        let theme = ThemeChoice::current(cx);
        let rusty = crate::rusty::is_on(cx);
        let mut themes = vec![
            WelcomeOption::new(
                "Marley Night",
                "The cat's dark coat",
                theme == ThemeChoice::Night,
            )
            .on_choose(|cx| ThemeChoice::Night.choose(cx)),
            WelcomeOption::new(
                "Marley Day",
                "The same, in daylight",
                theme == ThemeChoice::Day,
            )
            .on_choose(|cx| ThemeChoice::Day.choose(cx)),
        ];
        // Only where Omarchy is: without it the setting does nothing (D4).
        if crate::omarchy_theme::installed() {
            themes.push(
                WelcomeOption::new(
                    "Follow Omarchy",
                    "Omarchy's theme, as it changes",
                    theme == ThemeChoice::Omarchy,
                )
                .on_choose(|cx| ThemeChoice::Omarchy.choose(cx)),
            );
        }
        let agents = [AgentKind::Claude, AgentKind::Codex]
            .into_iter()
            .map(|kind| WelcomeOption::new(kind.display_name(), agent_found(kind, cx), false))
            .collect();
        vec![
            WelcomeRow {
                title: "Layout",
                options: vec![
                    WelcomeOption::new("Marley", "A rail of projects and their terminals", layout)
                        .on_choose(|cx| crate::write_layout(MarleyLayout::Marley, cx)),
                    WelcomeOption::new("Zed", "Zed's docks, as Zed ships them", !layout)
                        .on_choose(|cx| crate::write_layout(MarleyLayout::Zed, cx)),
                ],
            },
            WelcomeRow {
                title: "Theme",
                options: themes,
            },
            WelcomeRow {
                title: "Agents",
                options: agents,
            },
            WelcomeRow {
                title: "Rusty",
                options: vec![
                    WelcomeOption::new("On", "Your brain, tasks and notes", rusty)
                        .on_choose(|cx| write_rusty(true, cx)),
                    WelcomeOption::new("Off", "No trace of it anywhere", !rusty)
                        .on_choose(|cx| write_rusty(false, cx)),
                ],
            },
        ]
    }

    /// Connect the plugin (#547), the Agents row's one action (D3): greyed with no `claude`
    /// found, "Connected" once Claude Code lists it.
    fn render_plugin(&self, cx: &Context<Self>) -> AnyElement {
        let plugin = cx.try_global::<ClaudePlugin>();
        let connected = plugin.is_some_and(|plugin| plugin.installed == Some(true));
        let installing = plugin.is_some_and(|plugin| plugin.installing);
        let missing = plugin.is_none_or(|plugin| plugin.claude.is_none());
        let label = if connected {
            "Connected"
        } else if installing {
            "Connecting…"
        } else {
            "Connect the plugin"
        };
        let workspace = self.workspace.clone();
        Button::new("marley-welcome-plugin", label)
            .style(ButtonStyle::Filled)
            .label_size(LabelSize::Small)
            .disabled(connected || installing || missing)
            .on_click(move |_, _, cx| {
                if let Some(plugin) = cx.try_global::<ClaudePlugin>().cloned() {
                    crate::claude_plugin::install(plugin, workspace.clone(), cx);
                }
            })
            .into_any_element()
    }
}

/// Writes `marley.rusty.enabled`.
fn write_rusty(on: bool, cx: &App) {
    settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
        content
            .marley
            .get_or_insert_default()
            .rusty
            .get_or_insert_default()
            .enabled = Some(on);
    });
}

/// One option card, lit while it is in effect.
fn option_card(id: (&'static str, usize), option: WelcomeOption, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    let accent = colors.text_accent;
    let WelcomeOption {
        name,
        about,
        on,
        choose,
    } = option;
    v_flex()
        .id(id)
        .flex_1()
        .min_w_0()
        .gap_0p5()
        .p_2p5()
        .rounded_md()
        .border_1()
        .map(|card| {
            if on {
                card.border_color(accent.opacity(0.7))
                    .bg(accent.opacity(0.12))
            } else {
                card.border_color(colors.border)
            }
        })
        .when(choose.is_some(), |card| {
            card.cursor_pointer()
                .hover(|style| style.bg(colors.ghost_element_hover))
        })
        .child(
            Label::new(name)
                .size(LabelSize::Small)
                .weight(FontWeight::SEMIBOLD),
        )
        .child(
            Label::new(about)
                .size(LabelSize::XSmall)
                .color(Color::Muted),
        )
        .when_some(choose, |card, choose| {
            card.on_click(move |_, _, cx| choose(cx))
        })
        .into_any_element()
}

impl Render for MarleyWelcome {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let rows = Self::rows(cx);
        let plugin = self.render_plugin(cx);
        let mut agents_action = Some(plugin);
        v_flex()
            .id("marley-welcome")
            .debug_selector(|| "marley-welcome".into())
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .items_center()
            .py_10()
            .px_6()
            .bg(colors.editor_background)
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(560.))
                    .gap_5()
                    .child(
                        v_flex()
                            .items_center()
                            .gap_2()
                            .child(img("images/marley_mark.svg").size(px(56.)))
                            .child(Headline::new("Welcome to Marley").size(HeadlineSize::Large))
                            .child(
                                Label::new("Four choices, each one changeable later in Settings.")
                                    .color(Color::Muted),
                            ),
                    )
                    .children(rows.into_iter().map(|WelcomeRow { title, options }| {
                        let action = (title == "Agents").then(|| agents_action.take()).flatten();
                        v_flex()
                            .gap_1p5()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        Label::new(title.to_uppercase())
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    )
                                    .children(action),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .children(options.into_iter().enumerate().map(
                                        |(index, option)| option_card((title, index), option, cx),
                                    )),
                            )
                    }))
                    .child(
                        h_flex()
                            .justify_center()
                            .gap_2()
                            .pt_2()
                            .child(
                                Button::new("marley-welcome-open-folder", "Open a Folder")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        window.dispatch_action(
                                            Box::new(workspace::Open::DEFAULT),
                                            cx,
                                        );
                                        Self::close(cx);
                                    })),
                            )
                            .child(
                                Button::new("marley-welcome-home", "Go to Home")
                                    .style(ButtonStyle::Subtle)
                                    .on_click(cx.listener(|welcome, _, window, cx| {
                                        let workspace = welcome.workspace.clone();
                                        // After this update: Home's open reads every item.
                                        window.defer(cx, move |window, cx| {
                                            workspace
                                                .update(cx, |workspace, cx| {
                                                    crate::home_page::open_here(
                                                        workspace, window, cx,
                                                    );
                                                })
                                                .log_err();
                                        });
                                        Self::close(cx);
                                    })),
                            ),
                    ),
            )
    }
}

impl Focusable for MarleyWelcome {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for MarleyWelcome {}

impl Item for MarleyWelcome {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Welcome")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::MarleyCat))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}

impl std::fmt::Debug for MarleyWelcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MarleyWelcome")
            .finish_non_exhaustive()
    }
}
