//! Rusty's tools for Zed's agents (#633).
//!
//! Where `rusty-mcp` is on the search path, Marley offers it to Zed's agents as the context server
//! `rusty`, beside its own `marley` server (#501), so an agent in Marley reaches Rusty's brain loop
//! and its other tools.
//!
//! The server is a stdio `rusty-mcp` with no arguments, as Rusty's own `.mcp.json` names it, added
//! to Zed's default settings, so a `context_servers.rusty` of the user's own wins. Zed asks before
//! each call, as for every context server. `marley.rusty_tools` turns it off, which takes Marley's
//! entry back out.

use std::path::PathBuf;

use gpui::{App, AppContext as _, BorrowAppContext as _, Global};
use settings::settings_content::{ContextServerCommand, ContextServerSettingsContent};
use settings::{Settings as _, SettingsStore};

use crate::{MarleySettings, RustyTools};

/// The context server Zed's agents know Rusty's tools by.
const CONTEXT_SERVER: &str = "rusty";

/// The program Rusty's MCP server is.
const PROGRAM: &str = "rusty-mcp";

/// The `rusty-mcp` Marley last offered, if any.
#[derive(Default)]
struct RustyOffer(Option<PathBuf>);

impl Global for RustyOffer {}

/// Offers Rusty's server where it is installed, and follows the setting; [`crate::init`] calls it
/// once.
pub fn init(cx: &mut App) {
    cx.set_global(RustyOffer::default());
    offer(cx);
    cx.observe_global::<SettingsStore>(offer).detach();
}

/// Looks for `rusty-mcp` while the setting is on, off the main thread, then offers or withdraws.
fn offer(cx: &mut App) {
    let wanted = MarleySettings::get_global(cx).rusty_tools == RustyTools::Offered;
    let search_path = crate::agents::launcher(cx).search_path;
    let found = cx.background_spawn(futures::future::lazy(move |_| {
        wanted
            .then(|| which::which_in(PROGRAM, search_path, "/").ok())
            .flatten()
    }));
    cx.spawn(async move |cx| {
        let found = found.await;
        cx.update(|cx| settle(found.as_ref(), cx));
    })
    .detach();
}

/// Puts `found` into Zed's defaults as `rusty`, or takes Marley's entry out, when that changes
/// what was offered. The defaults' change notifies the settings again, which finds it unchanged.
fn settle(found: Option<&PathBuf>, cx: &mut App) {
    if cx.global::<RustyOffer>().0.as_ref() == found {
        return;
    }
    cx.global_mut::<RustyOffer>().0 = found.cloned();
    cx.update_global::<SettingsStore, _>(|store, cx| {
        store.update_default_settings(cx, |defaults| match found {
            Some(program) => {
                log::info!(
                    "rusty: Zed's agents get Rusty's tools through the context server \
                     {CONTEXT_SERVER}, which runs {}",
                    program.display()
                );
                defaults.project.context_servers.insert(
                    CONTEXT_SERVER.into(),
                    ContextServerSettingsContent::Stdio {
                        enabled: true,
                        remote: false,
                        command: ContextServerCommand {
                            path: program.clone(),
                            args: Vec::new(),
                            env: None,
                            timeout: None,
                        },
                    },
                );
            }
            None => {
                defaults.project.context_servers.remove(CONTEXT_SERVER);
            }
        });
    });
}
