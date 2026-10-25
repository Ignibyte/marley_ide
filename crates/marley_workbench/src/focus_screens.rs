//! The bottom row for Marley's own screens (#909): the Threads page, a Browser tab, MCP Servers, the
//! Component Library, and Rusty's Tasks, Graph and brain pages, each with the chips its board draws.
//!
//! Each fact comes from state the screen or a global already holds; the row asks nothing of Rusty
//! or the browser itself (D5).

use gpui::{AnyElement, App, Context, Entity, Subscription, Window, px};
use marley_agent::AgentKind;
use marley_agent::versions::Found;
use ui::prelude::*;
use workspace::MultiWorkspace;
use workspace::item::ItemHandle;

use crate::browser::{BrowserHub, BrowserView};
use crate::component_library::ComponentLibrary;
use crate::focus_row::{FocusRow, chip, muted};
use crate::item_kind::ItemKind;
use crate::mcp_servers::McpServers;
use crate::rusty::brain_tab::BrainTab;
use crate::rusty::graph_tab::GraphView;
use crate::rusty::page::PageView;
use crate::rusty::tasks_tab::TasksView;
use crate::threads_page::ThreadsPage;

/// The widest a page's title, an address or a broken template's error draws (D3).
const TEXT_MAX: f32 = 220.;

/// A screen the row has words for, by its entity (D1).
pub(crate) enum Screen {
    Threads(Entity<ThreadsPage>),
    Browser(Entity<BrowserView>),
    McpServers(Entity<McpServers>),
    Components(Entity<ComponentLibrary>),
    Tasks(Entity<TasksView>),
    Graph(Entity<GraphView>),
    /// The Brain tab, drawn as the page it shows (D7).
    Brain(Entity<BrainTab>),
    Page(Entity<PageView>),
}

impl Screen {
    /// The screen `item` is, when the row has words for it.
    pub(crate) fn of(item: &dyn ItemHandle) -> Option<Self> {
        item.downcast::<ThreadsPage>()
            .map(Self::Threads)
            .or_else(|| item.downcast::<BrowserView>().map(Self::Browser))
            .or_else(|| item.downcast::<McpServers>().map(Self::McpServers))
            .or_else(|| item.downcast::<ComponentLibrary>().map(Self::Components))
            .or_else(|| item.downcast::<TasksView>().map(Self::Tasks))
            .or_else(|| item.downcast::<GraphView>().map(Self::Graph))
            .or_else(|| item.downcast::<BrainTab>().map(Self::Brain))
            .or_else(|| item.downcast::<PageView>().map(Self::Page))
    }

    /// Follows what the screen's row reads: a Browser tab's page lives in the hub, every other
    /// screen in its own entity.
    pub(crate) fn observe(&self, cx: &mut Context<FocusRow>) -> Option<Subscription> {
        Some(match self {
            Self::Threads(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Browser(_) => {
                let hub = BrowserHub::try_global(cx)?;
                cx.observe(&hub, |_, _, cx| cx.notify())
            }
            Self::McpServers(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Components(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Tasks(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Graph(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Brain(view) => cx.observe(view, |_, _, cx| cx.notify()),
            Self::Page(view) => cx.observe(view, |_, _, cx| cx.notify()),
        })
    }

    /// The screen's row.
    pub(crate) fn render(&self, window: &Window, cx: &App) -> AnyElement {
        let parts = match self {
            Self::Threads(_) => threads(window, cx),
            Self::Browser(view) => browser(view, cx),
            Self::McpServers(_) => mcp_servers(cx),
            Self::Components(_) => components(cx),
            Self::Tasks(view) => tasks(view, cx),
            Self::Graph(view) => graph(view, cx),
            Self::Brain(view) => view
                .read(cx)
                .page()
                .map_or_else(|| vec![name_chip("Brain", cx)], |page| brain_page(page, cx)),
            Self::Page(view) => brain_page(view, cx),
        };
        h_flex()
            .debug_selector(|| "marley-focus-screen".into())
            .flex_none()
            .gap_2()
            .children(parts)
            .into_any_element()
    }
}

/// The Threads page: its conversations, and how many are open in tabs.
fn threads(window: &Window, cx: &App) -> Vec<AnyElement> {
    let conversations = crate::threads_page::conversations("", cx).len();
    let open: usize = window
        .root::<MultiWorkspace>()
        .flatten()
        .map_or(0, |multi_workspace| {
            multi_workspace
                .read(cx)
                .workspaces()
                .map(|workspace| crate::thread_tab::tab_threads(workspace.read(cx), cx).len())
                .sum()
        });
    vec![
        name_chip("Threads", cx),
        muted(count_words(conversations, "conversation", "conversations")),
        muted(format!("{open} open")),
    ]
}

/// A Browser tab: the page's title and address, its console errors, the picks ready, and the
/// minute the recorder keeps.
fn browser(view: &Entity<BrowserView>, cx: &App) -> Vec<AnyElement> {
    let mut parts = Vec::new();
    let page = view
        .read(cx)
        .target()
        .map(str::to_string)
        .zip(BrowserHub::try_global(cx));
    let Some((target, hub)) = page else {
        parts.push(chip("www".to_string(), ItemKind::Browser, cx));
        return parts;
    };
    let hub = hub.read(cx);
    let title = hub
        .title(&target)
        .unwrap_or_else(|| SharedString::new_static("Browser"));
    parts.push(
        h_flex()
            .gap_1()
            .child(chip("www".to_string(), ItemKind::Browser, cx))
            .child(capped(Label::new(title).size(LabelSize::Small)))
            .into_any_element(),
    );
    if let Some(address) = hub.address(&target) {
        parts.push(capped(
            Label::new(address)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .buffer_font(cx),
        ));
    }
    let errors = hub
        .console_entries(&target)
        .iter()
        .filter(|entry| crate::browser::is_error(&entry.level))
        .count();
    if errors > 0 {
        parts.push(dot_chip(
            count_words(errors, "console error", "console errors"),
            Severity::Error,
            cx,
        ));
    }
    let picks = hub.pick_count(&target);
    if picks > 0 {
        parts.push(muted(count_words(picks, "pick ready", "picks ready")));
    }
    parts.push(muted("recording the last minute".to_string()));
    parts
}

/// MCP Servers: each agent CLI Marley found missing, and the tools Marley's server lists.
fn mcp_servers(cx: &App) -> Vec<AnyElement> {
    let mut parts = vec![name_chip("MCP Servers", cx)];
    for kind in AgentKind::ALL {
        if let Some(Found::Missing(_)) = crate::agent_versions::found(kind, cx) {
            parts.push(dot_chip(
                format!("{} is not on the PATH", kind.display_name()),
                Severity::NeedsYou,
                cx,
            ));
        }
    }
    parts.push(muted(format!(
        "marley \u{b7} {} tools",
        marley_mcp::served_tool_count()
    )));
    parts
}

/// The Component Library: the first broken template and its error, and how many components and
/// templates there are.
fn components(cx: &App) -> Vec<AnyElement> {
    let templates = crate::component_library::templates(cx);
    let mut parts = vec![name_chip("Component Library", cx)];
    let broken = templates.broken();
    if let Some(first) = broken.first() {
        let file = first.path.file_name().map_or_else(
            || first.path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        parts.push(dot_chip(
            format!("{file} \u{b7} {}", first.error),
            Severity::NeedsYou,
            cx,
        ));
        if broken.len() > 1 {
            parts.push(muted(format!("+{} more", broken.len() - 1)));
        }
    }
    parts.push(muted(format!(
        "{} \u{b7} {}",
        count_words(
            marley_components::catalog().len(),
            "component",
            "components"
        ),
        count_words(templates.templates().len(), "template", "templates")
    )));
    parts
}

/// Rusty's Tasks: the chosen list, the decisions overdue, and the list's open tasks.
fn tasks(view: &Entity<TasksView>, cx: &App) -> Vec<AnyElement> {
    let facts = view.read(cx).row_facts();
    let name = facts.list.map_or_else(
        || "Tasks".to_string(),
        |list| format!("Tasks \u{b7} {list}"),
    );
    let mut parts = vec![name_chip(name, cx)];
    if facts.overdue > 0 {
        parts.push(dot_chip(
            count_words(facts.overdue, "decision overdue", "decisions overdue"),
            Severity::NeedsYou,
            cx,
        ));
    }
    parts.push(muted(format!("{} open", facts.open)));
    parts
}

/// Rusty's Graph: its scope, depth and pages, and the embedding as Rusty last reported it (D6).
fn graph(view: &Entity<GraphView>, cx: &App) -> Vec<AnyElement> {
    let facts = view.read(cx).row_facts();
    let pages = count_words(facts.pages, "page", "pages");
    let shown = facts.depth.map_or_else(
        || pages.clone(),
        |depth| format!("depth {depth} \u{b7} {pages}"),
    );
    let mut parts = vec![
        name_chip(format!("Graph \u{b7} {}", facts.scope), cx),
        muted(shown),
    ];
    if let Some(status) = crate::rusty::semantic(cx) {
        let words = if status.provider.is_none() {
            "full-text only".to_string()
        } else if status.stale > 0 {
            format!(
                "{} embedded \u{b7} {} waiting",
                count_words(status.stats.pages, "page", "pages"),
                status.stale
            )
        } else {
            format!(
                "{} embedded",
                count_words(status.stats.pages, "page", "pages")
            )
        };
        parts.push(muted(words));
    }
    parts
}

/// A brain page: its slug, whether it is a favourite, and whether Rusty is connected.
fn brain_page(view: &Entity<PageView>, cx: &App) -> Vec<AnyElement> {
    let slug = view.read(cx).slug().to_string();
    let favourite = crate::rusty::is_favourite_page(&slug, cx);
    let mut parts = vec![
        name_chip("Brain", cx),
        capped(
            Label::new(slug)
                .size(LabelSize::Small)
                .color(Color::Muted)
                .buffer_font(cx),
        ),
    ];
    if favourite {
        parts.push(muted("\u{2605} favourite".to_string()));
    }
    parts.push(muted(if crate::rusty::is_connected(cx) {
        "Rusty connected".to_string()
    } else {
        "Rusty not connected".to_string()
    }));
    parts
}

/// What a dot chip says about its words: the user is needed, or something failed.
#[derive(Clone, Copy)]
enum Severity {
    NeedsYou,
    Error,
}

/// A chip with a dot in the needs-you pink or the error red (D2).
fn dot_chip(text: String, severity: Severity, cx: &App) -> AnyElement {
    let status = cx.theme().status();
    let (color, background, ink) = match severity {
        Severity::NeedsYou => (status.conflict, status.conflict_background, Color::Conflict),
        Severity::Error => (status.error, status.error_background, Color::Error),
    };
    h_flex()
        .flex_none()
        .gap_1p5()
        .px_1p5()
        .rounded_sm()
        .bg(background)
        .child(div().size_1p5().rounded_full().bg(color))
        .child(capped(Label::new(text).size(LabelSize::Small).color(ink)))
        .into_any_element()
}

/// The raised chip naming the screen (D2).
fn name_chip(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .bg(cx.theme().colors().element_background)
        .child(Label::new(text.into()).size(LabelSize::Small))
        .into_any_element()
}

/// `label`, cut at the row's widest text (D3).
fn capped(label: Label) -> AnyElement {
    div()
        .max_w(px(TEXT_MAX))
        .min_w_0()
        .child(label.truncate())
        .into_any_element()
}

/// "1 page", "3 pages".
fn count_words(count: usize, one: &str, many: &str) -> String {
    if count == 1 {
        format!("1 {one}")
    } else {
        format!("{count} {many}")
    }
}
