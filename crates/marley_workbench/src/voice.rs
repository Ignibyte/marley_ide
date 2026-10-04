//! Voice input through Voxtype (T7d), the dictation daemon Omarchy ships.
//!
//! The agent bar's microphone runs `voxtype record toggle`: Voxtype records, transcribes when
//! toggled again, and types the text into the focused window, where the microphone has put the
//! focus on its terminal. [`Voice`] follows `voxtype status --follow --format json`, so the
//! microphone shows what Voxtype is doing however the dictation started: from the microphone,
//! from `marley::ToggleDictation` or from Omarchy's keys. Marley never touches the audio.
//!
//! All of it waits for `marley.voice.enabled` (#642): off, there is no microphone, the action
//! says dictation is off, and Marley starts no `voxtype`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use futures::{AsyncBufReadExt as _, StreamExt as _};
use gpui::{App, AppContext as _, AsyncApp, Context, Global, Task, WeakEntity};
use settings::{Settings as _, SettingsStore};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::{Dictation, MarleySettings, ToggleDictation};

/// What Voxtype is doing, as its status last said.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    /// Waiting for a dictation, or not running.
    #[default]
    Idle,
    /// Recording; Voxtype's streaming engines also transcribe as they go.
    Recording,
    /// Turning a finished recording into text.
    Transcribing,
}

/// Voxtype as the agent bar's microphone sees it.
#[derive(Debug, Default)]
pub struct Voice {
    /// The `voxtype` program, found on the PATH when Marley starts.
    pub voxtype: Option<PathBuf>,
    /// What Voxtype is doing.
    pub state: VoiceState,
    /// Whether `voxtype status --follow` runs.
    following: bool,
    /// The task that reads the status, and stops it when dropped.
    follower: Option<Task<()>>,
}

impl Global for Voice {}

/// Finds `voxtype` on the PATH, off the main thread, and installs `marley::ToggleDictation` on
/// every workspace.
///
/// It also stops following Voxtype when Voice turns off. [`crate::init`] calls it once, before
/// any window opens, so its observer runs before the terminals redraw their bars.
pub fn init(cx: &mut App) {
    cx.set_global(Voice::default());
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &ToggleDictation, _, cx| {
            if MarleySettings::get_global(cx).dictation == Dictation::Off {
                workspace.show_toast(
                    Toast::new(
                        NotificationId::unique::<Voice>(),
                        "Dictation is off. Turn it on in the Voice section of the Marley settings.",
                    ),
                    cx,
                );
                return;
            }
            match cx.global::<Voice>().voxtype.clone() {
                Some(voxtype) => toggle(voxtype, workspace.weak_handle(), cx),
                None => workspace.show_error(
                    anyhow::anyhow!("Voxtype, which Marley dictates with, is not on the PATH"),
                    cx,
                ),
            }
        });
    })
    .detach();
    cx.observe_global::<SettingsStore>(|cx| {
        if MarleySettings::get_global(cx).dictation == Dictation::Off {
            stop_following(cx);
        }
    })
    .detach();
    cx.spawn(async move |cx| {
        let voxtype = cx
            .background_spawn(futures::future::lazy(|_| which::which("voxtype").ok()))
            .await;
        cx.update(|cx| {
            cx.global_mut::<Voice>().voxtype = voxtype;
            cx.refresh_windows();
        });
    })
    .detach();
}

/// Starts following Voxtype's status once the frame is drawn, the first time a microphone is.
///
/// A status that ended is followed again only by a toggle, so a status that exits at once
/// cannot start over with every frame.
pub fn follow_once_drawn(voxtype: &Path, cx: &mut App) {
    if cx.global::<Voice>().follower.is_none() {
        let voxtype = voxtype.to_path_buf();
        cx.defer(move |cx| follow(voxtype, cx));
    }
}

/// Starts or stops a dictation, following Voxtype's status first if it does not run; a
/// failure shows in `workspace`.
pub fn toggle(voxtype: PathBuf, workspace: WeakEntity<Workspace>, cx: &mut App) {
    follow(voxtype.clone(), cx);
    cx.spawn(async move |cx| {
        let result =
            crate::process::run_program(&voxtype, &[OsStr::new("record"), OsStr::new("toggle")])
                .await;
        if let Err(error) = result {
            workspace
                .update(cx, |workspace, cx| workspace.show_error(error, cx))
                .log_err();
        }
    })
    .detach();
}

/// Drops the status follower, which ends `voxtype status`. The dropped task never reaches its own
/// reset, so this resets the state; the next microphone drawn starts a follower again.
fn stop_following(cx: &mut App) {
    let voice = cx.global_mut::<Voice>();
    voice.follower = None;
    voice.following = false;
    voice.state = VoiceState::Idle;
}

/// Runs `voxtype status --follow` unless it runs already. When it ends, Voxtype counts as idle.
fn follow(voxtype: PathBuf, cx: &mut App) {
    let voice = cx.global_mut::<Voice>();
    if voice.following {
        return;
    }
    voice.following = true;
    let follower = cx.spawn(async move |cx| {
        let result = read_status(&voxtype, cx).await;
        cx.update(|cx| {
            let voice = cx.global_mut::<Voice>();
            voice.following = false;
            voice.state = VoiceState::Idle;
            cx.refresh_windows();
        });
        result.log_err();
    });
    cx.global_mut::<Voice>().follower = Some(follower);
}

/// Reads `voxtype status --follow --format json` into [`Voice::state`], line by line, until it
/// ends.
async fn read_status(voxtype: &Path, cx: &AsyncApp) -> anyhow::Result<()> {
    let mut status = crate::process::follow(voxtype, &["status", "--follow", "--format", "json"])
        .context("running `voxtype status`")?;
    let output = status
        .stdout
        .take()
        .context("`voxtype status` has no output")?;
    let mut lines = futures::io::BufReader::new(output).lines();
    while let Some(line) = lines.next().await {
        if let Some(state) = state_of(&line?) {
            cx.update(|cx| {
                cx.global_mut::<Voice>().state = state;
                cx.refresh_windows();
            });
        }
    }
    Ok(())
}

/// The state a line of `voxtype status --format json` reports in its `class`; a daemon that
/// stopped, and a state this Voxtype does not know, count as idle.
fn state_of(line: &str) -> Option<VoiceState> {
    let status: serde_json::Value = serde_json::from_str(line).ok()?;
    Some(match status.get("class")?.as_str()? {
        "recording" | "streaming" => VoiceState::Recording,
        "transcribing" => VoiceState::Transcribing,
        _ => VoiceState::Idle,
    })
}
