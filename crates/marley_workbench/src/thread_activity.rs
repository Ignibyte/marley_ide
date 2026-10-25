//! What an agent thread is doing or asking, and since when (#907).
//!
//! The bottom row says it for the thread in front, and the rail's thread rows (#910) take the same
//! words from here, so the two never disagree.

use std::time::{Duration, Instant};

use acp_thread::{AgentThreadEntry, ToolCallStatus};
use agent_ui::ConversationView;
use gpui::{App, Entity};
use marley_rail::ThreadStatus;
use ui::SharedString;

/// A thread's status, what it does or asks, and since when.
pub(crate) struct ThreadActivity {
    status: ThreadStatus,
    doing: SharedString,
    /// When the turn started, while the thread works or waits.
    since: Option<Instant>,
}

impl ThreadActivity {
    /// The thread's status, as the rail reads it.
    pub(crate) const fn status(&self) -> ThreadStatus {
        self.status
    }

    /// What the agent does or asks: the newest tool call still open, "thinking", the files a
    /// finished turn changed, "idle" or "failed".
    pub(crate) const fn doing(&self) -> &SharedString {
        &self.doing
    }

    /// How long the turn has run, while the thread works or waits.
    pub(crate) fn elapsed(&self) -> Option<Duration> {
        self.since.map(|since| since.elapsed())
    }
}

/// `conversation`'s activity, once it has a thread.
pub(crate) fn thread_activity(
    conversation: &Entity<ConversationView>,
    cx: &App,
) -> Option<ThreadActivity> {
    let conversation = conversation.read(cx);
    let view = conversation.root_thread_view()?;
    let view = view.read(cx);
    let thread = view.thread.read(cx);
    let status = marley_rail::thread_status(
        conversation.root_thread_has_pending_tool_call(cx),
        thread.had_error(),
        thread.status() == acp_thread::ThreadStatus::Generating,
    );
    let doing: SharedString = match status {
        ThreadStatus::Running | ThreadStatus::Waiting => open_tool_call(thread.entries(), cx)
            .map_or_else(|| SharedString::new_static("thinking"), SharedString::from),
        ThreadStatus::Done => {
            let changed = thread.action_log().read(cx).changed_buffers(cx).count();
            match changed {
                0 => SharedString::new_static("idle"),
                1 => SharedString::new_static("1 file changed"),
                count => format!("{count} files changed").into(),
            }
        }
        ThreadStatus::Error => SharedString::new_static("failed"),
    };
    let open = matches!(status, ThreadStatus::Running | ThreadStatus::Waiting);
    Some(ThreadActivity {
        status,
        doing,
        since: view.turn_fields.turn_started_at.filter(|_| open),
    })
}

/// The label of the newest tool call that has not finished, on one line, as the inbox words a
/// waiting one.
fn open_tool_call(entries: &[AgentThreadEntry], cx: &App) -> Option<String> {
    entries.iter().rev().find_map(|entry| match entry {
        AgentThreadEntry::ToolCall(call)
            if matches!(
                call.status(),
                ToolCallStatus::Pending
                    | ToolCallStatus::WaitingForConfirmation
                    | ToolCallStatus::InProgress
            ) =>
        {
            let label = call.label.read(cx).source();
            Some(label.split_whitespace().collect::<Vec<_>>().join(" "))
        }
        _ => None,
    })
}

/// An elapsed time as the row says it: "40 s", "12 min", "1 h 5 min".
pub(crate) fn elapsed_words(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    match seconds {
        0..60 => format!("{seconds} s"),
        60..3600 => format!("{} min", seconds / 60),
        _ => format!("{} h {} min", seconds / 3600, seconds % 3600 / 60),
    }
}
