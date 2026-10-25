//! A running build's own progress (#918): cargo's `Building [..] N/M` bar and a leading `[N/M]`
//! (ninja, meson), read from the cursor's line, where build tools redraw it.
//!
//! On a terminal's output, at most every 500 ms, the running block's cursor line goes through
//! `marley_terminal::progress::parse_progress`. The last value per block is kept in a global
//! nothing observes (L-claude-572), and the view is notified when it moves; a block that no longer
//! runs loses its value at the next read. Agent CLIs' and SSH clients' terminals are left out, as
//! #572 leaves them out.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{AnyElement, App, Context, Entity, EntityId, Global, Task, Window};
use marley_terminal::BlockState;
use marley_terminal::progress::{Progress, parse_progress};
use terminal::{Event, Terminal};
use terminal_view::TerminalView;
use ui::prelude::*;
use util::ResultExt as _;

use crate::agent_bar::agent_in;
use crate::links::ssh_in;

/// How long after output the cursor's line is read, at most once in that time.
const READ_DELAY: Duration = Duration::from_millis(500);

/// The last progress each running block showed, by its terminal's entity id and its index, and
/// the read each terminal view has coming.
#[derive(Default)]
struct Progresses {
    by_block: HashMap<(EntityId, usize), Progress>,
    reads: HashMap<EntityId, Task<()>>,
}

impl Global for Progresses {}

/// Watches every terminal view's running block; [`crate::init`] calls it once.
pub fn init(cx: &App) {
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else { return };
            let terminal = view.terminal().clone();
            cx.subscribe_in(&terminal, window, |_, _, event, window, cx| {
                if matches!(event, Event::Wakeup) {
                    schedule(window, cx);
                }
            })
            .detach();
            let id = cx.entity_id();
            let terminal_id = terminal.entity_id();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Progresses>() {
                    let progresses = cx.global_mut::<Progresses>();
                    progresses.reads.remove(&id);
                    progresses
                        .by_block
                        .retain(|(terminal, _), _| *terminal != terminal_id);
                }
            })
            .detach();
        },
    )
    .detach();
}

/// Reads the view's cursor line once [`READ_DELAY`] has passed, unless a read is already coming.
fn schedule(window: &Window, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    // Read without `default_global` first: it is written at every wakeup otherwise.
    if cx
        .try_global::<Progresses>()
        .is_some_and(|progresses| progresses.reads.contains_key(&id))
    {
        return;
    }
    let read_task = cx.spawn_in(window, async move |this, cx| {
        cx.background_executor().timer(READ_DELAY).await;
        this.update(cx, |view, cx| read(view, cx)).log_err();
    });
    cx.default_global::<Progresses>()
        .reads
        .insert(id, read_task);
}

/// Reads the running block's progress from the cursor's line and keeps it, notifying the view
/// when it moved.
fn read(view: &TerminalView, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    if let Some(running) = cx.default_global::<Progresses>().reads.remove(&id) {
        // The task running this read: it ends by itself.
        running.detach();
    }
    let terminal_id = view.terminal().entity_id();
    let (running, progress) = {
        let terminal = view.terminal().read(cx);
        let running = terminal
            .blocks()
            .last()
            .filter(|block| block.state == BlockState::Running)
            .map(|block| block.index)
            .filter(|_| agent_in(terminal).is_none() && !ssh_in(terminal));
        let line = running.and_then(|_| terminal.marley_cursor_line());
        (running, line.as_deref().and_then(parse_progress))
    };
    let progresses = cx.default_global::<Progresses>();
    progresses
        .by_block
        .retain(|(terminal, index), _| *terminal != terminal_id || Some(*index) == running);
    if let (Some(index), Some(progress)) = (running, progress)
        && progresses.by_block.insert((terminal_id, index), progress) != Some(progress)
    {
        cx.notify();
    }
}

/// The progress the block at `index` of `terminal` last showed while it runs.
pub(crate) fn progress(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<Progress> {
    let running = terminal
        .read(cx)
        .blocks()
        .get(index)
        .is_some_and(|block| block.state == BlockState::Running);
    running
        .then(|| cx.try_global::<Progresses>())
        .flatten()?
        .by_block
        .get(&(terminal.entity_id(), index))
        .copied()
}

/// "1781 of 1813" in the running colour, for the block's chip, while it shows progress.
pub(crate) fn chip(terminal: &Entity<Terminal>, index: usize, cx: &App) -> Option<AnyElement> {
    let progress = progress(terminal, index, cx)?;
    Some(
        Label::new(format!("{} of {}", progress.done, progress.total))
            .size(LabelSize::XSmall)
            .color(Color::Warning)
            .into_any_element(),
    )
}
