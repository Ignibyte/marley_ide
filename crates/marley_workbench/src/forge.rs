//! Ignibyte Forge as a fleet provider (#841).
//!
//! A `forge` entry in `marley.fleet.providers` names Forge's MCP endpoint. The fleet's poll calls
//! its `agent-list` tool, once per project the entry names or once for the token's own, and reads
//! each running session as an agent of the fleet contract: its harness, its ticket and folder, its
//! host, and Forge's own state, where a session running and silent past three minutes needs
//! attention. The token is the keyring's entry for the endpoint, which `marley: set forge token`
//! writes, else the variable the entry names.

use editor::Editor;
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    Render, Window,
};
use marley_sdk::{AgentSummary, Attention, State, WorkItem};
use serde::Deserialize;
use settings::{FleetProviderContent, Settings as _};
use ui::prelude::*;
use workspace::notifications::{MarleyToastKind, NotificationId};
use workspace::{ModalView, Toast, Workspace};

use crate::{MarleySettings, SetForgeToken};

/// How often Forge is asked, in seconds, when the entry names no `poll_seconds`.
pub(crate) const DEFAULT_POLL_S: u64 = 15;

/// How long a session's heartbeat may be silent before its row reads stale: Forge leaves the
/// heartbeat's cadence to each harness.
pub(crate) const STALE_AFTER_S: u64 = 300;

/// The user name the keyring entry is written under.
const KEYRING_USER: &str = "forge";

/// `agent-list`'s answer, the fields Marley reads.
#[derive(Debug, Deserialize)]
pub(crate) struct ForgeAgents {
    #[serde(default)]
    pub(crate) agents: Vec<ForgeAgent>,
}

/// One of Forge's agent sessions (`AgentSessionDto`), the fields Marley reads.
#[derive(Debug, Deserialize)]
pub(crate) struct ForgeAgent {
    id: String,
    #[serde(default)]
    agent_name: String,
    #[serde(default)]
    folder: String,
    #[serde(default)]
    host: String,
    #[serde(default)]
    harness: String,
    #[serde(default)]
    status: String,
    current_ticket: Option<String>,
    last_output_at: Option<String>,
    last_heartbeat_at: Option<String>,
    #[serde(default)]
    state: String,
}

/// The running sessions in `agents` as the fleet contract's agents; stopped ones are left out.
pub(crate) fn summaries(agents: ForgeAgents) -> Vec<AgentSummary> {
    let now_ms = u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or_default();
    agents
        .agents
        .into_iter()
        .filter(|agent| agent.status != "stopped")
        .map(|agent| summary(agent, now_ms))
        .collect()
}

fn summary(agent: ForgeAgent, now_ms: u64) -> AgentSummary {
    // Forge derives the state when the list is read: silent past 180 s reads needs_attention.
    let state = match agent.state.as_str() {
        "working" => State::Working,
        "needs_attention" => State::Waiting,
        _ => State::Idle,
    };
    let silent_since = agent.last_output_at.as_deref().and_then(epoch_ms);
    let mut parts: Vec<String> = Vec::new();
    parts.extend(agent.current_ticket.clone());
    if !agent.folder.is_empty() {
        parts.push(agent.folder.clone());
    }
    if state == State::Waiting
        && let Some(since) = silent_since
    {
        parts.push(format!("silent {}", silent_for(since, now_ms)));
    }
    let key = parts.join(" · ");
    let work_item = (!key.is_empty()).then(|| WorkItem {
        id: agent.current_ticket.clone().unwrap_or_default(),
        key,
        title: agent.current_ticket.clone().unwrap_or_default(),
        status: None,
        url: None,
        summary: None,
    });
    AgentSummary {
        id: agent.id,
        name: agent.agent_name,
        runtime: agent.harness,
        state,
        // Its last output is when its silence began; its heartbeat, when Forge last heard of it.
        state_since_ms: silent_since,
        last_seen_ms: agent.last_heartbeat_at.as_deref().and_then(epoch_ms),
        host_id: (!agent.host.is_empty()).then_some(agent.host),
        work_item,
        phase: None,
        attention: Attention::None,
        question: None,
    }
}

/// How long since `since_ms`, in whole minutes, or hours past two.
fn silent_for(since_ms: u64, now_ms: u64) -> String {
    let minutes = now_ms.saturating_sub(since_ms) / 60_000;
    if minutes >= 120 {
        format!("{} h", minutes / 60)
    } else {
        format!("{minutes} m")
    }
}

/// An RFC 3339 time as milliseconds since the epoch.
fn epoch_ms(time: &str) -> Option<u64> {
    let parsed = chrono::DateTime::parse_from_rfc3339(time).ok()?;
    u64::try_from(parsed.timestamp_millis()).ok()
}

/// Registers `marley: set forge token`. [`crate::init`] calls it once.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &SetForgeToken, window, cx| {
            let Some(url) = forge_url(cx) else {
                struct NoForge;
                workspace.show_toast(
                    Toast::new(
                        NotificationId::unique::<NoForge>(),
                        "No Forge provider is set. Add { \"kind\": \"forge\", \"url\": … } to \
                         marley.fleet.providers first.",
                    ),
                    cx,
                );
                return;
            };
            let weak_workspace = cx.weak_entity();
            workspace.toggle_modal(window, cx, |window, cx| {
                ForgeToken::new(url, weak_workspace, window, cx)
            });
        });
    })
    .detach();
}

/// The first Forge provider's endpoint.
fn forge_url(cx: &App) -> Option<String> {
    MarleySettings::get_global(cx)
        .fleet_providers
        .iter()
        .find_map(|entry| match entry {
            FleetProviderContent::Forge { url, .. } => Some(url.clone()),
            _ => None,
        })
}

/// `marley: set forge token`: a masked field whose Enter writes the token to the keyring under
/// Forge's endpoint.
pub(crate) struct ForgeToken {
    url: String,
    field: Entity<Editor>,
    workspace: gpui::WeakEntity<Workspace>,
}

impl ForgeToken {
    fn new(
        url: String,
        workspace: gpui::WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_masked(true, cx);
            editor.set_placeholder_text("Paste Forge's token, then press Enter", window, cx);
            editor
        });
        Self {
            url,
            field,
            workspace,
        }
    }

    fn save(&mut self, _: &menu::Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        let token = self.field.read(cx).text(cx);
        let token = token.trim();
        if token.is_empty() {
            return;
        }
        let write = cx.write_credentials(&self.url, KEYRING_USER, token.as_bytes());
        let workspace = self.workspace.clone();
        cx.spawn(async move |_, cx| {
            let (message, kind) = match write.await {
                Ok(()) => (
                    "The Forge token is in the keyring; the fleet uses it at its next connection."
                        .to_string(),
                    MarleyToastKind::Success,
                ),
                Err(error) => (
                    format!("Could not write the Forge token: {error:#}"),
                    MarleyToastKind::Error,
                ),
            };
            workspace
                .update(cx, |workspace, cx| {
                    struct TokenWritten;
                    workspace.show_toast(
                        Toast::new(NotificationId::unique::<TokenWritten>(), message)
                            .marley_kind(kind),
                        cx,
                    );
                })
                .ok();
        })
        .detach();
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for ForgeToken {}

impl ModalView for ForgeToken {}

impl Focusable for ForgeToken {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.field.focus_handle(cx)
    }
}

impl Render for ForgeToken {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .debug_selector(|| "marley-forge-token".into())
            .key_context("ForgeToken")
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(px(440.))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .child(Label::new("Forge's token").weight(gpui::FontWeight::SEMIBOLD))
            .child(
                Label::new(format!("Kept in the system keyring for {}.", self.url))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.field.clone()),
            )
    }
}
