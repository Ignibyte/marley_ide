//! The Component Library tab (#799).
//!
//! It lists the components an agent can ask Marley to draw (`marley_components`), by group, each
//! with its description, its example drawn in the theme, and the example's JSON. A scratch box
//! at the top draws whatever JSON is typed or pasted into it, or shows why it cannot, so a tree
//! can be tried without an agent. It opens from `marley: open component library`, Home's
//! Configure card and the Marley Settings page; one tab per workspace, brought forward when open.

use editor::{Editor, EditorEvent};
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, SharedString,
    Subscription, Window, actions,
};
use marley_components::{Entry, Group, ParseError, Tree, catalog, parse, parse_str};
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

/// Registers the open action on every workspace.
pub(crate) fn init(cx: &App) {
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
    tried: Option<Result<Tree, ParseError>>,
    focus_handle: FocusHandle,
    _scratch_edits: Subscription,
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
        let scratch_edits = cx.subscribe(&scratch, |library, scratch, event, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                let text = scratch.read(cx).text(cx);
                library.tried = (!text.trim().is_empty()).then(|| parse_str(&text));
                cx.notify();
            }
        });
        Self {
            shown,
            scratch,
            tried: None,
            focus_handle: cx.focus_handle(),
            _scratch_edits: scratch_edits,
        }
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
        v_flex()
            .id("marley-component-library")
            .key_context("MarleyComponentLibrary")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_4()
            .bg(colors.editor_background)
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
            .children(Group::ALL.into_iter().map(|group| {
                v_flex()
                    .gap_3()
                    .child(Headline::new(group.title()).size(HeadlineSize::Medium))
                    .children(
                        self.shown
                            .iter()
                            .filter(|shown| shown.entry.group == group)
                            .map(|shown| entry_card(shown, cx)),
                    )
            }))
    }
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
