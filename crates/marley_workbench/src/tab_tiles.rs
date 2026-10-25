//! The Ctrl Tab switcher's rows as the Splits board draws them (#883), and the center tabs and the
//! split menu as the Chrome board draws them (#866).
//!
//! Each switcher row leads with a tile naming its tab's kind, its state a dot at the corner, and
//! says under its title the project and the state. In the Marley layout each tab leads with the
//! same tile and carries its state as a dot, and the split menu draws each split as a small
//! picture. The tiles' tints, glyphs and dots are `item_kind`'s, as the rail's are (#905).

use gpui::{App, Window};
use marley_terminal::BlockState;
use tab_switcher::{MarleySwitcherLook, MarleySwitcherRow};
use terminal::Terminal;
use terminal_view::{MarleyTerminalTabIcon, TerminalView};
use ui::prelude::*;
use workspace::item::ItemHandle;
use workspace::pane::{MarleySplitPictures, MarleyTabLook, MarleyTabMark};
use workspace::{MultiWorkspace, SplitDirection};

use crate::browser::{BrowserHub, BrowserView};
use crate::item_kind::{AgentMark, ItemKind, ItemState};
use crate::thread_tab::ThreadTab;
use crate::tmux_sessions::TmuxHosts;

/// Sets the switcher's look, the tabs' and the split menu's.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleySwitcherLook(switcher_row));
    cx.set_global(MarleyTabLook(tab_mark));
    cx.set_global(MarleySplitPictures(split_picture));
    // In the Marley layout a terminal tab's tile is its one mark (#905, D4).
    cx.set_global(MarleyTerminalTabIcon(|cx| !crate::marley_layout(cx)));
}

/// `item`'s tile and state in the Marley layout (#866, #905); none in Zed's, which keeps Zed's
/// tabs.
fn tab_mark(item: &dyn ItemHandle, _window: &mut Window, cx: &mut App) -> Option<MarleyTabMark> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let kind = kind_of(item, cx);
    Some(MarleyTabMark {
        tile: tile(kind, glyph_of(item, kind, cx), px(24.), cx).into_any_element(),
        dot: state_of(item, cx).map(|state| state.dot(cx)),
    })
}

/// What `item` stands for (#905): a thread by its agent, a terminal by the agent CLI it runs or
/// the tmux session it is attached to, a Browser tab, a file, else a page of Marley's or Zed's.
fn kind_of(item: &dyn ItemHandle, cx: &App) -> ItemKind {
    if let Some(tab) = item.downcast::<ThreadTab>() {
        let agent = tab.read(cx).conversation().read(cx).agent_key().id();
        return ItemKind::Thread(AgentMark::from_agent_id(&agent));
    }
    if let Some(view) = item.downcast::<TerminalView>() {
        let terminal = view.read(cx).terminal().read(cx);
        if tmux_attached(terminal, cx) {
            return ItemKind::Tmux;
        }
        let agent = crate::agent_bar::agent_in(terminal)
            .or_else(|| crate::agent_events::waiting_agent(view.entity_id(), cx));
        return agent.map_or(ItemKind::Shell, |agent| {
            ItemKind::AgentTerminal(AgentMark::from_cli(agent))
        });
    }
    if item.downcast::<BrowserView>().is_some() {
        ItemKind::Browser
    } else if item.project_path(cx).is_some() {
        ItemKind::File
    } else {
        ItemKind::Page
    }
}

/// Whether `terminal` is a task attached to a tmux session (#821).
fn tmux_attached(terminal: &Terminal, cx: &App) -> bool {
    let Some(task) = terminal.task() else {
        return false;
    };
    cx.try_global::<TmuxHosts>()
        .is_some_and(|tmux| tmux.attach_of(&task.spawned_task.full_label).is_some())
}

/// The glyph on `item`'s tile: an agent's letters, ">_" for a shell, "tx" for tmux, "www" for a
/// Browser tab, a file's extension, else the title's first letter.
fn glyph_of(item: &dyn ItemHandle, kind: ItemKind, cx: &App) -> SharedString {
    match kind {
        ItemKind::Thread(agent) | ItemKind::AgentTerminal(agent) => agent.glyph(),
        ItemKind::Shell => SharedString::new_static(">_"),
        ItemKind::Tmux => SharedString::new_static("tx"),
        ItemKind::Browser => SharedString::new_static("www"),
        ItemKind::File | ItemKind::Page | ItemKind::Port => item
            .project_path(cx)
            .and_then(|path| path.path.extension().map(str::to_string))
            .map_or_else(
                || crate::project_icons::initial(&item.tab_content_text(0, cx)).into(),
                |extension| extension.chars().take(3).collect::<String>().into(),
            ),
    }
}

/// `item`'s state (#866, #905, D3): a thread's; an agent waiting on the user; a tmux tab idle; a
/// terminal's as the rail shows it, else its running or failed command or its unread bell; a
/// Browser tab with a dialog open, loading or loaded.
fn state_of(item: &dyn ItemHandle, cx: &App) -> Option<ItemState> {
    if let Some(tab) = item.downcast::<ThreadTab>() {
        return thread_state(tab.read(cx), cx);
    }
    if let Some(view) = item.downcast::<TerminalView>() {
        if crate::agent_events::waiting_agent(view.entity_id(), cx).is_some() {
            return Some(ItemState::NeedsYou);
        }
        let terminal = view.read(cx).terminal().read(cx);
        if tmux_attached(terminal, cx) {
            return Some(ItemState::Idle);
        }
        // The rail's word, so the tab says what the row says (D3).
        if let Some(state) = crate::item_kind::terminal_state(view.entity_id(), cx) {
            return Some(state);
        }
        let last = terminal.marley_anchored().live().last();
        return match last.map(|block| (block.state, block.exit_code.0)) {
            Some((BlockState::Running, _)) => Some(ItemState::Working),
            Some((BlockState::Finished, Some(code))) if code != 0 => Some(ItemState::Failed),
            _ => crate::notifications::unread(view.entity_id(), cx).then_some(ItemState::Done),
        };
    }
    let view = item.downcast::<BrowserView>()?;
    let target = view.read(cx).target()?.to_string();
    let hub = BrowserHub::try_global(cx)?;
    let hub = hub.read(cx);
    Some(if hub.dialog(&target).is_some() {
        ItemState::NeedsYou
    } else if hub.is_loading(&target) {
        ItemState::Working
    } else {
        ItemState::Done
    })
}

/// A thread tab's state, once its conversation has a thread, as the focus row reads it.
fn thread_state(tab: &ThreadTab, cx: &App) -> Option<ItemState> {
    let conversation = tab.conversation().read(cx);
    let view = conversation.root_thread_view()?;
    let thread = view.read(cx).thread.read(cx);
    Some(ItemState::from_thread(marley_rail::thread_status(
        conversation.root_thread_has_pending_tool_call(cx),
        thread.had_error(),
        thread.status() == acp_thread::ThreadStatus::Generating,
    )))
}

/// A split's picture (#866): a small window frame with the new pane's side filled.
fn split_picture(direction: SplitDirection, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    // Each pane fills its share of the frame both ways: a row centres its children, which would
    // leave them no height.
    let new_pane = || {
        div()
            .flex_1()
            .size_full()
            .rounded_xs()
            .bg(colors.text_accent.opacity(0.7))
    };
    let old_pane = || div().flex_1().size_full().rounded_xs().bg(colors.border);
    let frame = match direction {
        SplitDirection::Right => h_flex().child(old_pane()).child(new_pane()),
        SplitDirection::Left => h_flex().child(new_pane()).child(old_pane()),
        SplitDirection::Up => v_flex().child(new_pane()).child(old_pane()),
        SplitDirection::Down => v_flex().child(old_pane()).child(new_pane()),
    };
    frame
        .flex_none()
        .w(px(22.))
        .h(px(16.))
        .p_0p5()
        .gap_0p5()
        .rounded_sm()
        .border_1()
        .border_color(colors.border)
        .into_any_element()
}

/// `glyph` on a tile `width` wide, in `kind`'s tint and ink.
fn tile(kind: ItemKind, glyph: SharedString, width: Pixels, cx: &App) -> Div {
    div()
        .flex_none()
        .w(width)
        .py_0p5()
        .flex()
        .justify_center()
        .rounded_sm()
        .bg(kind.tint(cx))
        .child(
            Label::new(glyph)
                .size(LabelSize::XSmall)
                .buffer_font(cx)
                .color(Color::Custom(kind.ink(cx))),
        )
}

/// `item`'s tile, its state a dot at the corner, and its line, "<project> · <state>" (#883,
/// #905, D5).
fn switcher_row(item: &dyn ItemHandle, window: &mut Window, cx: &mut App) -> MarleySwitcherRow {
    let kind = kind_of(item, cx);
    let state = state_of(item, cx);
    let background = cx.theme().colors().elevated_surface_background;
    let tile = tile(kind, glyph_of(item, kind, cx), px(30.), cx)
        .relative()
        .children(state.map(|state| {
            div()
                .absolute()
                .bottom(px(-2.))
                .right(px(-2.))
                .size_2()
                .rounded_full()
                .border_1()
                .border_color(background)
                .bg(state.dot(cx))
        }))
        .into_any_element();
    let project = project_of(item, window, cx);
    let line = state_words(item, kind, state, cx).map_or_else(
        || project.to_string(),
        |words| format!("{project} · {words}"),
    );
    MarleySwitcherRow {
        tile,
        line: Some(line.into()),
    }
}

/// The project `item` belongs to, by its worktree's name: the worktree a file or a terminal's
/// folder lies in, else the shown workspace's first, else Home.
fn project_of(item: &dyn ItemHandle, window: &Window, cx: &App) -> SharedString {
    let home = || SharedString::new_static("Home");
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return home();
    };
    let workspace = multi_workspace.read(cx).workspace().read(cx);
    let project = workspace.project().read(cx);
    let name_of = |worktree: &gpui::Entity<project::Worktree>| -> SharedString {
        worktree
            .read(cx)
            .root_name()
            .as_unix_str()
            .to_string()
            .into()
    };
    if let Some(path) = item.project_path(cx)
        && let Some(worktree) = project.worktree_for_id(path.worktree_id, cx)
    {
        return name_of(&worktree);
    }
    let folder = item
        .downcast::<TerminalView>()
        .and_then(|view| view.read(cx).terminal().read(cx).working_directory());
    let mut visible = project.visible_worktrees(cx);
    let found = match folder {
        Some(folder) => visible.find(|worktree| folder.starts_with(worktree.read(cx).abs_path())),
        None => visible.next(),
    };
    found.map_or_else(home, |worktree| name_of(&worktree))
}

/// What a switcher row's line says after the project (D5); none when the state says nothing more.
fn state_words(
    item: &dyn ItemHandle,
    kind: ItemKind,
    state: Option<ItemState>,
    cx: &App,
) -> Option<SharedString> {
    match kind {
        ItemKind::Thread(_) => state.map(|state| SharedString::new_static(state.words())),
        ItemKind::Tmux => Some(SharedString::new_static("tmux")),
        ItemKind::AgentTerminal(_) if state == Some(ItemState::NeedsYou) => {
            Some(SharedString::new_static(ItemState::NeedsYou.words()))
        }
        ItemKind::AgentTerminal(_) | ItemKind::Shell => {
            let view = item.downcast::<TerminalView>()?;
            let terminal = view.read(cx).terminal().read(cx);
            let last = terminal.marley_anchored().live().last()?;
            match (last.state, last.exit_code.0) {
                (BlockState::Running, _) => Some(last.command.trim().to_string().into()),
                (BlockState::Finished, Some(code)) if code != 0 => {
                    Some(format!("exit {code}").into())
                }
                _ => None,
            }
        }
        ItemKind::Browser => {
            if state == Some(ItemState::Working) {
                return Some(SharedString::new_static("loading"));
            }
            let view = item.downcast::<BrowserView>()?;
            let target = view.read(cx).target()?;
            let url = BrowserHub::try_global(cx)?.read(cx).url(target)?;
            let host = url::Url::parse(&url).ok()?.host_str()?.to_string();
            Some(host.into())
        }
        ItemKind::File => item
            .project_path(cx)
            .map(|path| path.path.as_unix_str().to_string().into()),
        ItemKind::Page | ItemKind::Port => None,
    }
}
