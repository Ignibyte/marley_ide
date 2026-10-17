//! `component_list` and `component_show` (#800): an agent draws a catalog component
//! (`marley_components`, #799) for the user.
//!
//! In an Agent Panel thread, the thread draws the tree in the call's row from the call's input
//! (`tool_call_view`, through `agent_ui`'s `MarleyToolCallView`), so every agent shows it the
//! same way whether its client keeps structured results or not: Zed's own agent keeps only the
//! text. A call from an agent in a Marley terminal has no row to draw into, so the tree opens in a
//! Component tab beside that terminal. Either way the tool checks the tree first and answers the
//! error, with its JSON path, for the agent to fix.

use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, SharedString, Window,
};
use marley_components::{Tree, catalog, parse_with, render, render_error};
use marley_mcp::{AppCall, Caller, Refusal, ToolAnswer};
use serde_json::{Value, json};
use ui::prelude::*;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

/// Gives `agent_ui` Marley's drawing of a `component_show` call.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(agent_ui::MarleyToolCallView {
        draw: tool_call_view,
    });
}

/// The thread's drawing of a `component_show` call, from its input: the tree, or why it cannot
/// be drawn. `None` for any other tool, which the thread draws as Zed does.
pub(crate) fn tool_call_view(
    tool_name: &str,
    raw_input: &Value,
    id: SharedString,
    cx: &App,
) -> Option<AnyElement> {
    if !is_component_show(tool_name) {
        return None;
    }
    let parsed;
    let input = match raw_input {
        // Some agents report a call's input as its JSON text.
        Value::String(text) => {
            parsed = serde_json::from_str::<Value>(text).ok()?;
            &parsed
        }
        input => input,
    };
    let component = input.get("component")?;
    Some(
        match parse_with(component, &crate::component_library::templates(cx)) {
            Ok(tree) => render(&tree, id, cx),
            Err(error) => render_error(&error),
        },
    )
}

/// Whether an agent's name for a tool is Marley's `component_show`: Claude Code reports
/// `mcp__<server>__component_show`, other clients `<server>.component_show` or the bare name.
fn is_component_show(tool_name: &str) -> bool {
    let bare = tool_name
        .rsplit_once("__")
        .map_or(tool_name, |(_, tail)| tail);
    let bare = bare.rsplit_once('.').map_or(bare, |(_, tail)| tail);
    bare == "component_show"
}

/// Answers a `component_*` call.
pub(crate) fn answer(call: AppCall, cx: &mut App) {
    let verb = call.tool.strip_prefix("component_").unwrap_or_default();
    let answer = match verb {
        "list" => Ok(list(cx)),
        "show" => show(&call.arguments, call.caller(), cx),
        _ => Err(Refusal::new(
            "unknown_tool",
            format!("no tool {}", call.tool),
        )),
    };
    call.answer(answer);
}

fn list(cx: &App) -> ToolAnswer {
    let entries = catalog();
    let templates = crate::component_library::templates(cx);
    let components = entries
        .iter()
        .map(|entry| {
            json!({
                "type": entry.name,
                "group": entry.group.title(),
                "description": entry.description,
                "schema": entry.schema,
                "example": entry.example,
            })
        })
        .chain(templates.templates().iter().map(|template| {
            json!({
                "type": template.name,
                "group": "Templates",
                "description": template.description,
                "schema": template.schema(),
                "example": template.example_tree(),
            })
        }))
        .collect::<Vec<_>>();
    let names = entries
        .iter()
        .map(|entry| entry.name.to_owned())
        .chain(
            templates
                .templates()
                .iter()
                .map(|template| template.name.clone()),
        )
        .collect::<Vec<_>>()
        .join(", ");
    ToolAnswer {
        structured: json!({ "components": components }),
        text: Some(format!(
            "Marley draws {} component types: {names}. Each entry gives its fields' JSON Schema \
             and an example tree.",
            components.len()
        )),
        image: None,
    }
}

fn show(arguments: &Value, caller: &Caller, cx: &mut App) -> Result<ToolAnswer, Refusal> {
    let component = arguments.get("component").ok_or_else(|| {
        Refusal::new("bad_argument", "`component` is missing")
            .next("set `component` to one JSON object with a `type`")
            .next("component_list gives each type's fields and an example")
    })?;
    let tree = match parse_with(component, &crate::component_library::templates(cx)) {
        Ok(tree) => tree,
        Err(error) => {
            return Ok(ToolAnswer {
                structured: json!({
                    "shown": false,
                    "error": { "path": error.path, "message": error.message },
                }),
                text: Some(format!(
                    "Marley did not draw the tree. {error} Fix the tree and call component_show \
                     again."
                )),
                image: None,
            });
        }
    };
    let title = arguments
        .get("title")
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("Component")
        .to_owned();
    if let Some((workspace, terminal)) = crate::mcp::caller_terminal(caller, cx)
        && let Some(window) = crate::browser::window_of(&workspace, cx)
    {
        let opened = AnyWindowHandle::from(window).update(cx, |_, window, cx| {
            workspace.update(cx, |workspace, cx| {
                open_tab(workspace, &terminal, title.clone(), tree, window, cx);
            });
        });
        if opened.is_ok() {
            return Ok(ToolAnswer {
                structured: json!({ "shown": true, "where": "tab", "title": title }),
                text: Some(format!(
                    "Marley opened the tree in the tab \"{title}\" beside your terminal."
                )),
                image: None,
            });
        }
    }
    Ok(ToolAnswer {
        structured: json!({ "shown": true, "where": "thread" }),
        text: Some("Marley draws the tree in this call's row of the thread.".to_owned()),
        image: None,
    })
}

/// Opens the tree in a Component tab in the terminal's pane, or replaces the tree of the tab with
/// the same title.
fn open_tab(
    workspace: &mut Workspace,
    terminal: &Entity<terminal_view::TerminalView>,
    title: String,
    tree: Tree,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let open = workspace
        .items_of_type::<ComponentTab>(cx)
        .find(|tab| tab.read(cx).title.as_ref() == title);
    if let Some(tab) = open {
        tab.update(cx, |tab, cx| {
            tab.tree = tree;
            cx.notify();
        });
        workspace.activate_item(&tab, true, false, window, cx);
        return;
    }
    let tab = cx.new(|cx| ComponentTab {
        title: SharedString::from(title),
        tree,
        focus_handle: cx.focus_handle(),
    });
    match workspace.pane_for(terminal) {
        Some(pane) => pane.update(cx, |pane, cx| {
            pane.add_item(Box::new(tab), true, false, None, window, cx);
        }),
        None => {
            workspace.add_item_to_center(Box::new(tab), window, cx);
        }
    }
}

/// A tree an agent in a terminal showed.
pub(crate) struct ComponentTab {
    title: SharedString,
    tree: Tree,
    focus_handle: FocusHandle,
}

impl Focusable for ComponentTab {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for ComponentTab {}

impl Render for ComponentTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("marley-component-tab")
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .gap_3()
            .bg(cx.theme().colors().editor_background)
            .child(Headline::new(self.title.clone()).size(HeadlineSize::Medium))
            .child(render(&self.tree, "marley-component-tab", cx))
    }
}

impl Item for ComponentTab {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        self.title.clone()
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Blocks))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::from(format!(
            "A component an agent showed: {}",
            self.title
        )))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
