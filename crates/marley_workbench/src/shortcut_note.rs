//! A note the first time one of Marley's keys takes a key from a terminal's program (#563).
//!
//! Marley binds a few keys in a terminal that a program would otherwise get: Ctrl-G while an
//! agent CLI runs (Rich Input), Ctrl-Up and Ctrl-Down (the block keys), Ctrl-Alt-N (New Agent)
//! and Ctrl-I once an agent typed or ran there (the take-over). The first time each takes its
//! key, a toast names the key as bound now, what it did, and Open Keymap, since the user's keymap
//! wins over Marley's and gives the key back. Each action's note shows once per data directory,
//! remembered in Zed's key-value store; a handler that lets the key through says nothing.

use collections::HashSet;
use db::kvp::KeyValueStore;
use gpui::{Action, App, AppContext as _, Entity, FocusHandle, Global, SharedString, Window};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

/// The store's scope for the actions whose note was shown.
const SCOPE: &str = "marley-shortcut-note";

/// The actions whose note this session showed or found shown.
#[derive(Default)]
struct Shown(HashSet<SharedString>);

impl Global for Shown {}

/// `action` took its key from the terminal whose focus handle is `terminal` and `did` what it
/// does, in words (`opened Marley's Rich Input`): the note, the first time on this data
/// directory.
pub(crate) fn taken(
    action: &dyn Action,
    did: &'static str,
    terminal: &FocusHandle,
    workspace: &Entity<Workspace>,
    window: &Window,
    cx: &mut App,
) {
    let name = SharedString::from(action.name());
    if cx
        .try_global::<Shown>()
        .is_some_and(|shown| shown.0.contains(&name))
    {
        return;
    }
    // The key as the terminal's own contexts bind it now, the user's keymap last: the window's
    // own lookup reads the frame's root contexts, where a `Terminal` binding is not.
    let Some(binding) = window.bindings_for_action_in(action, terminal).pop() else {
        return;
    };
    let key = ui::text_for_keybinding_keystrokes(binding.keystrokes(), cx);
    cx.default_global::<Shown>().0.insert(name.clone());
    let store = KeyValueStore::global(cx);
    if store
        .scoped(SCOPE)
        .read(&name)
        .log_err()
        .flatten()
        .is_some()
    {
        return;
    }
    let message = format!("{key} {did}; the program in this terminal did not get the key.");
    let id = NotificationId::composite::<Shown>(name.clone());
    let workspace = workspace.downgrade();
    // The handler runs while the workspace is being updated; the toast waits for that to end.
    window.defer(cx, move |_, cx| {
        let toast = Toast::new(id, message).on_click("Open Keymap", |window, cx| {
            window.dispatch_action(zed_actions::OpenKeymapFile.boxed_clone(), cx);
        });
        workspace
            .update(cx, |workspace, cx| workspace.show_toast(toast, cx))
            .log_err();
    });
    cx.background_spawn(async move {
        store
            .scoped(SCOPE)
            .write(name.to_string(), "shown".to_string())
            .await
            .log_err();
    })
    .detach();
}
