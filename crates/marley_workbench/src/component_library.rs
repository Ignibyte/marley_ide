//! The Component Library tab (#799).
//!
//! It lists the components an agent can ask Marley to draw (`marley_components`), by group, each
//! with its description, its example drawn in the theme, and the example's JSON. A scratch box
//! at the top draws whatever JSON is typed or pasted into it, or shows why it cannot, so a tree
//! can be tried without an agent. It opens from `marley: open component library`, Home's
//! Configure card and the Marley Settings page; one tab per workspace, brought forward when open.
//!
//! The templates in `components/` of Marley's config folder (#801) are loaded here into
//! `MarleyTemplates` and loaded again when a file there changes; the library shows them in their
//! own group, the refused files with why, and the scratch box and the agent tools use them.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use editor::{Editor, EditorEvent};
use fs::Fs;
use futures::StreamExt as _;
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Global,
    SharedString, Subscription, Window, actions,
};
use marley_components::{
    Entry, Group, ParseError, Templates, Tree, catalog, parse, parse_str_with, parse_with,
};
use ui::prelude::*;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

actions!(
    marley,
    [
        /// Opens the Component Library: every component an agent can ask Marley to draw, with its
        /// JSON, and a box to try one.
        #[derive(Eq)]
        OpenComponentLibrary
    ]
);

/// The templates of `components/` in Marley's config folder, as last loaded (#801).
#[derive(Clone, Debug, Default)]
pub(crate) struct MarleyTemplates(pub(crate) Arc<Templates>);

impl Global for MarleyTemplates {}

/// The templates as last loaded; none before the first load.
pub(crate) fn templates(cx: &App) -> Arc<Templates> {
    cx.try_global::<MarleyTemplates>()
        .map(|templates| Arc::clone(&templates.0))
        .unwrap_or_default()
}

/// The folder templates are read from.
pub(crate) fn templates_dir() -> PathBuf {
    paths::config_dir().join("components")
}

/// Loads the templates, and again whenever a file in their folder changes. The folder is made
/// first, in Marley's own config folder, so there is one to watch and to open.
fn watch_templates(cx: &mut App) {
    cx.set_global(MarleyTemplates::default());
    let fs = <dyn Fs>::global(cx);
    let dir = templates_dir();
    cx.spawn(async move |cx| {
        fs.create_dir(&dir).await?;
        let (events, _watcher) = fs.watch(&dir, Duration::from_millis(200)).await;
        futures::pin_mut!(events);
        loop {
            let loaded = smol::unblock({
                let dir = dir.clone();
                move || Templates::load_in(&dir)
            })
            .await;
            cx.update(|cx| cx.set_global(MarleyTemplates(Arc::new(loaded))));
            if events.next().await.is_none() {
                return anyhow::Ok(());
            }
        }
    })
    .detach_and_log_err(cx);
}

/// Registers the open action on every workspace and starts loading the templates.
pub(crate) fn init(cx: &mut App) {
    watch_templates(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenComponentLibrary, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings the workspace's library tab forward, or opens one in the center.
pub(crate) fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<ComponentLibrary>(cx).next();
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let tab = cx.new(|cx| ComponentLibrary::new(window, cx));
    workspace.add_item_to_center(Box::new(tab), window, cx);
}

/// One catalog entry as the tab shows it.
struct Shown {
    entry: Entry,
    drawn: Result<Tree, ParseError>,
    json: Vec<SharedString>,
}

/// The Component Library tab.
pub(crate) struct ComponentLibrary {
    shown: Vec<Shown>,
    scratch: Entity<Editor>,
    /// The navigation column's find field (#849).
    find: Entity<Editor>,
    tried: Option<Result<Tree, ParseError>>,
    focus_handle: FocusHandle,
    _find_edits: Subscription,
    _scratch_edits: Subscription,
    _templates_changed: Subscription,
}

impl ComponentLibrary {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shown = catalog()
            .into_iter()
            .map(|entry| {
                let drawn = parse(&entry.example);
                let json = serde_json::to_string_pretty(&entry.example)
                    .unwrap_or_default()
                    .lines()
                    .map(|line| SharedString::from(line.to_owned()))
                    .collect();
                Shown { entry, drawn, json }
            })
            .collect();
        let scratch = cx.new(|cx| {
            let mut editor = Editor::auto_height(4, 16, window, cx);
            editor.set_placeholder_text(
                "Type or paste a component's JSON here to draw it",
                window,
                cx,
            );
            editor
        });
        let find = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Find a component", window, cx);
            editor
        });
        let find_edits = cx.subscribe(&find, |_, _, event, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        });
        let scratch_edits = cx.subscribe(&scratch, |library, _, event, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                library.try_scratch(cx);
            }
        });
        // A template added or changed redraws its group and what the scratch box holds.
        let templates_changed = cx.observe_global::<MarleyTemplates>(|library, cx| {
            library.try_scratch(cx);
        });
        Self {
            shown,
            scratch,
            find,
            tried: None,
            focus_handle: cx.focus_handle(),
            _find_edits: find_edits,
            _scratch_edits: scratch_edits,
            _templates_changed: templates_changed,
        }
    }

    /// What the find field holds, lower-cased.
    fn query(&self, cx: &App) -> String {
        self.find.read(cx).text(cx).trim().to_lowercase()
    }

    /// The navigation column, as the Components board draws it (#849): the find field over each
    /// group's component names; a name clicked narrows the page to it.
    fn render_navigation(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let query = self.query(cx);
        let groups = Group::ALL.into_iter().filter_map(|group| {
            let names: Vec<&'static str> = self
                .shown
                .iter()
                .filter(|shown| shown.entry.group == group && matches(&shown.entry, &query))
                .map(|shown| shown.entry.name)
                .collect();
            (!names.is_empty()).then(|| {
                v_flex()
                    .gap_0p5()
                    .child(
                        div().px_2().pt_2().child(
                            Label::new(group.title())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                    )
                    .children(names.into_iter().map(|name| {
                        h_flex()
                            .id(SharedString::from(format!("marley-library-nav-{name}")))
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|style| style.bg(colors.ghost_element_hover))
                            .on_click(cx.listener(move |library, _, window, cx| {
                                library
                                    .find
                                    .update(cx, |find, cx| find.set_text(name, window, cx));
                            }))
                            .child(Label::new(name).size(LabelSize::Small).buffer_font(cx))
                    }))
            })
        });
        v_flex()
            .debug_selector(|| "marley-library-navigation".into())
            .id("marley-library-navigation")
            .flex_none()
            .w(px(220.))
            .h_full()
            .p_2()
            .gap_1()
            .overflow_y_scroll()
            .border_r_1()
            .border_color(colors.border_variant)
            .bg(colors.panel_background)
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .child(self.find.clone()),
            )
            .children(groups)
    }

    fn try_scratch(&mut self, cx: &mut Context<Self>) {
        let text = self.scratch.read(cx).text(cx);
        self.tried = (!text.trim().is_empty()).then(|| parse_str_with(&text, &templates(cx)));
        cx.notify();
    }
}

impl Focusable for ComponentLibrary {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for ComponentLibrary {}

impl Render for ComponentLibrary {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let tried = match &self.tried {
            None => None,
            Some(Ok(tree)) => Some(marley_components::render(tree, "marley-library-tried", cx)),
            Some(Err(error)) => Some(marley_components::render_error(error)),
        };
        let query = self.query(cx);
        let main = v_flex()
            .id("marley-component-library")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .child(Headline::new("Component Library").size(HeadlineSize::Large))
            .child(
                Label::new(
                    "What an agent can draw in a thread with component_show. Each component \
                     below is drawn from the JSON beside it.",
                )
                .color(Color::Muted),
            )
            .child(
                v_flex()
                    .debug_selector(|| "marley-library-scratch".into())
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .child(
                        Label::new("Try it")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        div()
                            .p_2()
                            .rounded_sm()
                            .border_1()
                            .border_color(colors.border_variant)
                            .bg(colors.editor_background)
                            .child(self.scratch.clone()),
                    )
                    .children(tried),
            )
            .children(Group::ALL.into_iter().filter_map(|group| {
                let cards: Vec<_> = self
                    .shown
                    .iter()
                    .filter(|shown| shown.entry.group == group && matches(&shown.entry, &query))
                    .map(|shown| entry_card(shown, cx))
                    .collect();
                (!cards.is_empty()).then(|| {
                    v_flex()
                        .gap_3()
                        .child(Headline::new(group.title()).size(HeadlineSize::Medium))
                        .children(cards)
                })
            }))
            .child(templates_group(cx));
        h_flex()
            .key_context("MarleyComponentLibrary")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.editor_background)
            .child(self.render_navigation(cx))
            .child(main)
    }
}

/// Whether `entry` matches the find field's `query` (D1): its name or description holds it.
fn matches(entry: &Entry, query: &str) -> bool {
    query.is_empty()
        || entry.name.contains(query)
        || entry.description.to_lowercase().contains(query)
}

/// The Templates group: each template drawn from its example with its file, each refused file
/// with why, and a button that opens the folder.
fn templates_group(cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let templates = templates(cx);
    let dir = templates_dir();
    v_flex()
        .debug_selector(|| "marley-library-templates".into())
        .gap_3()
        .child(
            h_flex()
                .gap_3()
                .child(Headline::new("Templates").size(HeadlineSize::Medium))
                .child(
                    Button::new("marley-library-open-templates", "Open Folder")
                        .start_icon(Icon::new(IconName::Folder).size(IconSize::Small))
                        .style(ButtonStyle::Filled)
                        .on_click(move |_, _, cx| cx.open_with_system(&dir)),
                ),
        )
        .child(
            Label::new(SharedString::from(format!(
                "A JSON file in {} defines a component from the ones above. Marley reads the \
                 folder again when a file changes.",
                templates_dir().display()
            )))
            .size(LabelSize::Small)
            .color(Color::Muted),
        )
        .when(
            templates.templates().is_empty() && templates.broken().is_empty(),
            |group| {
                group.child(
                    Label::new("No templates yet.")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            },
        )
        .children(templates.templates().iter().map(|template| {
            let name = template.name.clone();
            let drawn = match parse_with(&template.example_tree(), &templates) {
                Ok(tree) => marley_components::render(&tree, format!("marley-template-{name}"), cx),
                Err(error) => marley_components::render_error(&error),
            };
            v_flex()
                .debug_selector(move || format!("marley-template-{name}"))
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(colors.border)
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Label::new(SharedString::from(template.name.clone()))
                                .buffer_font(cx)
                                .weight(gpui::FontWeight::BOLD),
                        )
                        .child(
                            Label::new(SharedString::from(template.description.clone()))
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        ),
                )
                .child(
                    Label::new(SharedString::from(template.path.display().to_string()))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                )
                .child(drawn)
        }))
        .children(templates.broken().iter().map(|broken| {
            ui::Callout::new()
                .severity(Severity::Error)
                .icon(IconName::XCircle)
                .title(SharedString::from(format!(
                    "{} is not used",
                    broken.path.display()
                )))
                .description(SharedString::from(broken.error.clone()))
        }))
}

/// An entry's card: its name and description over its drawn example, the JSON beside it.
fn entry_card(shown: &Shown, cx: &App) -> impl IntoElement {
    let colors = cx.theme().colors();
    let name = shown.entry.name;
    let drawn = match &shown.drawn {
        Ok(tree) => marley_components::render(tree, format!("marley-library-{name}"), cx),
        Err(error) => marley_components::render_error(error),
    };
    v_flex()
        .debug_selector(move || format!("marley-library-{name}"))
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(colors.border)
        .child(
            h_flex()
                .gap_2()
                .child(
                    Label::new(name)
                        .buffer_font(cx)
                        .weight(gpui::FontWeight::BOLD),
                )
                .child(
                    Label::new(shown.entry.description)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
        )
        .child(
            h_flex()
                .gap_4()
                .items_start()
                .child(div().flex_1().min_w_0().child(drawn))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .p_2()
                        .rounded_sm()
                        .bg(colors.surface_background)
                        .children(shown.json.iter().map(|line| {
                            Label::new(line.clone())
                                .size(LabelSize::XSmall)
                                .buffer_font(cx)
                                .color(Color::Muted)
                        })),
                ),
        )
}

impl Item for ComponentLibrary {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Component Library")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Blocks))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "The components an agent can draw with component_show",
        ))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
