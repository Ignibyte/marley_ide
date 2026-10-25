//! A long command's end, and a password prompt, from a terminal the user is not looking at
//! (#551).
//!
//! Each terminal view's blocks are watched. A block that ends after `marley.long_command_seconds`
//! (30; 0 for never) posts one desktop notification titled with its command, `done in 45 s` or
//! `exit 1 after 4 m 12 s`, and marks the terminal unread. While the running block's PTY reads a
//! line with echo off, a password prompt, one notification says it waits for a password. Agent
//! terminals are left out: their session is one long block, and #538 words their banners.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{App, Context, Entity, EntityId, Global, Window};
use marley_terminal::BlockState;
use settings::Settings as _;
use terminal::{Event, Terminal};
use terminal_view::TerminalView;

use crate::MarleySettings;
use crate::agent_bar::agent_in;
use crate::notifications::{looking_at, mark_unread, notify};

/// What was already told for each terminal view: the last block whose end was looked at, and the
/// block a password prompt was told for.
#[derive(Default)]
struct Watch(HashMap<EntityId, Told>);

impl Global for Watch {}

#[derive(Default, Clone, Copy)]
struct Told {
    finished_through: Option<usize>,
    password_for: Option<usize>,
    /// The block whose end is told whatever the long-command setting says (#917, D5).
    tell_for: Option<usize>,
}

/// Marks the block at `index` of the terminal view `view` to be told of when it ends, whatever
/// the long-command setting says, or unmarks it (#917, D5).
pub(crate) fn toggle_tell(view: EntityId, index: usize, cx: &mut App) {
    let told = cx.default_global::<Watch>().0.entry(view).or_default();
    told.tell_for = if told.tell_for == Some(index) {
        None
    } else {
        Some(index)
    };
}

/// Whether the block at `index` of the terminal view `view` is marked to be told of (#917).
pub(crate) fn will_tell(view: EntityId, index: usize, cx: &App) -> bool {
    cx.try_global::<Watch>()
        .and_then(|watch| watch.0.get(&view))
        .is_some_and(|told| told.tell_for == Some(index))
}

/// What a check of a terminal's last block found to tell.
enum Tell {
    Finished { took: Duration, exit: Option<i32> },
    Password,
}

/// Watches every terminal view's blocks; [`crate::init`] calls it once.
pub fn init(cx: &App) {
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else { return };
            let terminal = view.terminal().clone();
            // A view made for a terminal with blocks, as a split is, tells nothing of the ends
            // before it.
            let finished_through = terminal
                .read(cx)
                .blocks()
                .last()
                .filter(|block| block.state == BlockState::Finished)
                .map(|block| block.index);
            let id = cx.entity_id();
            let _earlier = cx.default_global::<Watch>().0.insert(
                id,
                Told {
                    finished_through,
                    password_for: None,
                    tell_for: None,
                },
            );
            // A block's end stamps its times and notifies the terminal, with no event.
            cx.observe_in(&terminal, window, |view, terminal, window, cx| {
                check(view, &terminal, window, cx);
            })
            .detach();
            // The password flag changes with the foreground process's refresh, which emits this.
            cx.subscribe_in(&terminal, window, |view, terminal, event, window, cx| {
                if matches!(event, Event::TitleChanged) {
                    check(view, terminal, window, cx);
                }
            })
            .detach();
            cx.on_release(move |_, cx| {
                if cx.has_global::<Watch>() {
                    let _forgotten = cx.global_mut::<Watch>().0.remove(&id);
                }
            })
            .detach();
        },
    )
    .detach();
}

/// Tells what `terminal`'s last block has come to since it was last checked, unless an agent CLI
/// runs there or the user is looking at the terminal.
fn check(
    view: &TerminalView,
    terminal: &Entity<Terminal>,
    window: &Window,
    cx: &mut Context<TerminalView>,
) {
    let (index, command, tell) = {
        let terminal = terminal.read(cx);
        let anchored = terminal.marley_anchored();
        let Some(block) = anchored.blocks().last() else {
            return;
        };
        // An agent's block, whether it still runs or has ended, is the agent's session.
        if agent_in(terminal).is_some() || marley_agent::agent_kind_of(&block.command).is_some() {
            return;
        }
        let tell = if block.state == BlockState::Finished {
            let took = anchored
                .times(block.index)
                .and_then(|times| times.finished?.duration_since(times.started).ok())
                .unwrap_or_default();
            Tell::Finished {
                took,
                exit: block.exit_code.0,
            }
        } else if terminal.marley_foreground_reads_password() {
            Tell::Password
        } else {
            return;
        };
        (block.index, block.command.trim().to_string(), tell)
    };
    let threshold = MarleySettings::get_global(cx).long_command_seconds;
    let id = cx.entity_id();
    let told = cx.default_global::<Watch>().0.entry(id).or_default();
    // A finished block's time and exit, or none for a password's tell.
    let ending = match tell {
        Tell::Finished { took, exit } => {
            if told
                .finished_through
                .is_some_and(|through| through >= index)
            {
                return;
            }
            told.finished_through = Some(index);
            let asked = told.tell_for == Some(index);
            if asked {
                told.tell_for = None;
            }
            if !asked && (threshold == 0 || took < Duration::from_secs(threshold)) {
                return;
            }
            Some((
                marley_terminal::duration_label(took),
                exit.unwrap_or_default(),
            ))
        }
        Tell::Password => {
            if told.password_for == Some(index) {
                return;
            }
            told.password_for = Some(index);
            None
        }
    };
    if command.is_empty() || looking_at(view, window, cx) {
        return;
    }
    // As the Inbox board words it (#925): "<command> finished · exit N", and where it ran.
    let (title, body) = match ending {
        Some((took, code)) => (
            format!("{command} finished \u{b7} exit {code}"),
            ran_words(view, terminal, &took, window, cx),
        ),
        None => (command, "waiting for a password".to_string()),
    };
    mark_unread(cx);
    notify(view, Some(&title), &body, window, cx);
}

/// "<terminal> ran <time> while you were in <item>.", naming the active item only while the window
/// is active and shows another item; "<terminal> ran <time>." otherwise (#925, D5).
fn ran_words(
    view: &TerminalView,
    terminal: &Entity<Terminal>,
    took: &str,
    window: &Window,
    cx: &Context<TerminalView>,
) -> String {
    let tab = terminal.read(cx).title(true);
    let me = cx.entity_id();
    let elsewhere = window
        .is_window_active()
        .then(|| view.marley_workspace().upgrade())
        .flatten()
        .and_then(|workspace| workspace.read(cx).active_item(cx))
        .filter(|item| item.item_id() != me)
        .map(|item| item.tab_content_text(0, cx));
    elsewhere.map_or_else(
        || format!("{tab} ran {took}."),
        |item| format!("{tab} ran {took} while you were in {item}."),
    )
}
