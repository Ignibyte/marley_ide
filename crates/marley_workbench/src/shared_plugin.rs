//! The Claude Code plugin Marley shares with rustal-harness (#709), loaded in Marley's terminals.
//!
//! Its six files are rustal-harness's, carried unchanged at the revision with Marley's host (its
//! TICKET-108, MIT OR Apache-2.0). Their digest names a read-only folder under Marley's data
//! directory, and while `marley.claude_code_shared_plugin` is on and the installed Claude Code is
//! one the plugin was tested on (#648), each new local terminal puts that folder first in
//! `CLAUDE_CODE_PLUGIN_DIRS`. The plugin's mod then reports the session's state through
//! `MARLEY_BIN` (#652).

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use gpui::{App, AppContext as _};
use marley_agent::versions::CLAUDE_SHARED_PLUGIN;
use marley_terminal::identity::{set_shared_plugin, shared_plugin};
use settings::SettingsStore;
use sha2::{Digest as _, Sha256};
use util::ResultExt as _;

/// The plugin's files, by their path in the plugin, in rustal-harness's order, which the digest
/// follows.
const FILES: [(&str, &str); 6] = [
    (
        ".claude-plugin/plugin.json",
        include_str!("../claude_shared_plugin/.claude-plugin/plugin.json"),
    ),
    (
        "hooks/hooks.json",
        include_str!("../claude_shared_plugin/hooks/hooks.json"),
    ),
    (
        "hooks/register.js",
        include_str!("../claude_shared_plugin/hooks/register.js"),
    ),
    (
        "LICENSE-MIT",
        include_str!("../claude_shared_plugin/LICENSE-MIT"),
    ),
    (
        "LICENSE-APACHE",
        include_str!("../claude_shared_plugin/LICENSE-APACHE"),
    ),
    (
        "claude-code-versions.json",
        include_str!("../claude_shared_plugin/claude-code-versions.json"),
    ),
];

/// Whether Marley's local terminals load the shared plugin, from
/// `marley.claude_code_shared_plugin` (#709).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SharedPlugin {
    /// No terminal loads it.
    #[default]
    Off,
    /// New local terminals load it, while the installed Claude Code is one it was tested on.
    On,
}

impl SharedPlugin {
    /// The value of `marley.claude_code_shared_plugin`: off unless it is on.
    pub(crate) fn of(cx: &App) -> Self {
        match cx
            .global::<SettingsStore>()
            .merged_settings()
            .marley
            .as_ref()
            .and_then(|marley| marley.claude_code_shared_plugin)
        {
            Some(true) => Self::On,
            _ => Self::Off,
        }
    }
}

/// Follows the setting and the installed Claude Code's version; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    reconcile(cx);
    cx.observe_global::<SettingsStore>(reconcile).detach();
    crate::agent_versions::observe(cx, reconcile).detach();
}

fn wanted(cx: &App) -> bool {
    SharedPlugin::of(cx) == SharedPlugin::On
        && crate::agent_versions::is_on(&CLAUDE_SHARED_PLUGIN, cx)
}

/// Writes the plugin's folder when it is wanted and hands it to the terminals, or takes it away.
fn reconcile(cx: &mut App) {
    if !wanted(cx) {
        set_shared_plugin(None);
        return;
    }
    if shared_plugin().is_some() {
        return;
    }
    let shared = paths::data_dir().join("claude-code").join("shared");
    let written = cx.background_spawn(futures::future::lazy(move |_| install_in(&shared)));
    cx.spawn(async move |cx| {
        let Some(folder) = written.await.log_err() else {
            return;
        };
        // The setting may have changed while the folder was written.
        cx.update(|cx| {
            if wanted(cx) {
                set_shared_plugin(Some(folder));
            }
        });
    })
    .detach();
}

/// The plugin's digest, as `rh` computes it: each file's path, then its text, each preceded by its
/// length as a little-endian u64, in [`FILES`]' order.
fn digest() -> String {
    let mut hash = Sha256::new();
    for (path, text) in FILES {
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path);
        hash.update((text.len() as u64).to_le_bytes());
        hash.update(text);
    }
    format!("{:x}", hash.finalize())
}

/// The plugin's folder under `shared`, named by its digest: checked when it is there, else written,
/// read-only, by staging it beside and renaming it into place.
///
/// # Errors
///
/// When the folder can't be written, or one that is there no longer holds the plugin's files.
fn install_in(shared: &Path) -> io::Result<PathBuf> {
    let folder = shared.join(digest());
    if folder.exists() {
        check_in(&folder)?;
        return Ok(folder);
    }
    make_dir(shared)?;
    let staged = shared.join(format!(".staged-{}", uuid::Uuid::new_v4()));
    make_dir(&staged)?;
    for (path, text) in FILES {
        let file = staged.join(path);
        if let Some(parent) = file.parent() {
            make_dir(parent)?;
        }
        write_read_only(&file, text)?;
    }
    if let Err(error) = fs::rename(&staged, &folder) {
        // Another Marley may have put it there first.
        fs::remove_dir_all(&staged).log_err();
        if !folder.exists() {
            return Err(error);
        }
    }
    check_in(&folder)?;
    Ok(folder)
}

/// Whether `folder` still holds exactly the plugin's files.
fn check_in(folder: &Path) -> io::Result<()> {
    for (path, text) in FILES {
        if fs::read_to_string(folder.join(path))? != text {
            return Err(io::Error::other(format!(
                "the shared Claude Code plugin's {path} in {} was altered, so Marley doesn't load \
                 it; remove the folder to have Marley write it again",
                folder.display()
            )));
        }
    }
    Ok(())
}

/// Makes `dir`, readable by its owner alone, when it is missing.
fn make_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(dir)
}

/// Writes `text` to a new `file` its owner can only read.
fn write_read_only(file: &Path, text: &str) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o400);
    }
    let mut written = options.open(file)?;
    written.write_all(text.as_bytes())?;
    written.sync_all()
}
