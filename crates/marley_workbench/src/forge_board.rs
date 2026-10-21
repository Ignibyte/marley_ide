//! Forge's sprint board (#842).
//!
//! `marley: open forge board` opens a tab with the current sprint's tickets in five columns by
//! status, each card a ticket with its kind, phase and the agent on it, and the selected ticket's
//! detail at the right. Claim, Move to… and Comment write through Forge's tools; Start here in a
//! worktree hands the ticket to New Agent in Worktree (#510). The board reaches Forge as the
//! fleet's first `forge` provider does (#841), with the same URL and token, and reads again after
//! each action and every 30 seconds.

use std::time::Duration;

use chrono::{DateTime, Utc};
use editor::{Editor, EditorEvent};
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, Render, SharedString, Subscription, Task, WeakEntity, Window, px,
};
use marley_agent::AgentKind;
use marley_sdk::AgentSummary;
use serde::Deserialize;
use serde_json::json;
use settings::{FleetProviderContent, Settings as _};
use ui::{ContextMenu, PopoverMenu, Tooltip, prelude::*};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::fleet_providers::{self, Client};
use crate::{MarleySettings, OpenForgeBoard, forge, worktree_agents};

/// The columns and the statuses each holds (D1); `won-t-fix` and `abandoned` show nowhere.
const COLUMNS: [(&str, &[&str]); 5] = [
    ("Open", &["open"]),
    ("In progress", &["in-progress"]),
    ("In review", &["in-review"]),
    ("QA", &["qa-review", "qa-failure", "qa-pass"]),
    ("Done", &["done", "closed"]),
];

/// The statuses Move to… offers.
const MOVES: [&str; 8] = [
    "open",
    "in-progress",
    "in-review",
    "qa-review",
    "qa-failure",
    "qa-pass",
    "done",
    "closed",
];

/// How often the board reads Forge again while it is open.
const REFRESH: Duration = Duration::from_secs(30);

/// The most tickets one read lists, Forge's own limit.
const TICKET_LIMIT: u32 = 200;

/// A ticket, the fields the board reads.
#[derive(Debug, Clone, Deserialize)]
struct Ticket {
    id: String,
    number: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    status: String,
    phase: Option<String>,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct Tickets {
    #[serde(default)]
    tickets: Vec<Ticket>,
}

/// The current sprint, the fields the board reads.
#[derive(Debug, Clone, Deserialize)]
struct Sprint {
    id: String,
    number: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    github_release_branch: String,
    ends_at: Option<String>,
}

#[derive(Deserialize)]
struct SprintAnswer {
    sprint: Option<Sprint>,
}

#[derive(Debug, Clone, Deserialize)]
struct Comment {
    #[serde(default)]
    body: String,
    created_at: Option<String>,
}

/// `ticket-get`'s answer with its comments.
#[derive(Debug, Clone, Deserialize)]
struct Detail {
    ticket: Ticket,
    #[serde(default)]
    comments: Vec<Comment>,
}

/// One read of Forge: the sprint, its tickets and the running agents.
struct Read {
    sprint: Option<Sprint>,
    tickets: Vec<Ticket>,
    agents: Vec<AgentSummary>,
}

/// Where the board stands with Forge.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Standing {
    Loading,
    NoProvider,
    Failed(String),
    Ready,
}

/// Registers `marley: open forge board`. [`crate::init`] calls it once.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenForgeBoard, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings the board forward, or opens one in the center.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open_board = workspace.items_of_type::<ForgeBoard>(cx).next();
    if let Some(board) = open_board {
        workspace.activate_item(&board, true, true, window, cx);
        return;
    }
    let handle = workspace.weak_handle();
    let board = cx.new(|cx| ForgeBoard::new(handle, window, cx));
    workspace.add_item_to_center(Box::new(board), window, cx);
}

/// The fleet's first Forge provider.
fn forge_entry(cx: &App) -> Option<FleetProviderContent> {
    MarleySettings::get_global(cx)
        .fleet_providers
        .iter()
        .find(|entry| matches!(entry, FleetProviderContent::Forge { .. }))
        .cloned()
}

/// The board's tab.
pub struct ForgeBoard {
    workspace: WeakEntity<Workspace>,
    client: Option<Client>,
    standing: Standing,
    sprint: Option<Sprint>,
    tickets: Vec<Ticket>,
    agents: Vec<AgentSummary>,
    /// The selected ticket's id, and its detail once read.
    selected: Option<String>,
    detail: Option<Detail>,
    filter: Entity<Editor>,
    comment: Entity<Editor>,
    focus_handle: FocusHandle,
    /// Counts the reads, so an older one never overwrites a newer.
    generation: u64,
    _refresh: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl std::fmt::Debug for ForgeBoard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ForgeBoard")
            .field("standing", &self.standing)
            .field("tickets", &self.tickets.len())
            .finish_non_exhaustive()
    }
}

impl ForgeBoard {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = |placeholder: &'static str, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_placeholder_text(placeholder, window, cx);
                editor
            })
        };
        let filter = field("Filter tickets", window, cx);
        let comment = field("Comment, then press Enter", window, cx);
        let subscriptions = vec![cx.subscribe(&filter, |_, _, event: &EditorEvent, cx| {
            if matches!(event, EditorEvent::BufferEdited) {
                cx.notify();
            }
        })];
        let refresh = cx.spawn(async move |board, cx| {
            loop {
                cx.background_executor().timer(REFRESH).await;
                if board.update(cx, Self::refresh).is_err() {
                    break;
                }
            }
        });
        let mut board = Self {
            workspace,
            client: None,
            standing: Standing::Loading,
            sprint: None,
            tickets: Vec::new(),
            agents: Vec::new(),
            selected: None,
            detail: None,
            filter,
            comment,
            focus_handle: cx.focus_handle(),
            generation: 0,
            _refresh: refresh,
            _subscriptions: subscriptions,
        };
        board.refresh(cx);
        board
    }

    /// Reads the sprint, its tickets, the agents and the selected ticket's detail.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(entry) = forge_entry(cx) else {
            self.standing = Standing::NoProvider;
            cx.notify();
            return;
        };
        self.generation += 1;
        let generation = self.generation;
        let client = self.client.clone();
        let selected = self.selected.clone();
        cx.spawn(async move |board, cx| {
            let read = async {
                let client = match client {
                    Some(client) => client,
                    None => fleet_providers::connect(&entry, "forge-board", cx).await?,
                };
                let read = read_forge(&client, cx).await?;
                let detail = match &selected {
                    Some(id) => Some(read_detail(&client, id, cx).await?),
                    None => None,
                };
                anyhow::Ok((client, read, detail))
            };
            let result = read.await;
            board
                .update(cx, |board, cx| {
                    if board.generation != generation {
                        return;
                    }
                    match result {
                        Ok((client, read, detail)) => {
                            board.client = Some(client);
                            board.standing = Standing::Ready;
                            board.sprint = read.sprint;
                            board.tickets = read.tickets;
                            board.agents = read.agents;
                            board.detail = detail;
                        }
                        Err(error) => {
                            board.client = None;
                            board.standing = Standing::Failed(format!("{error:#}"));
                        }
                    }
                    cx.notify();
                })
                .log_err();
        })
        .detach();
    }

    fn select(&mut self, id: String, cx: &mut Context<Self>) {
        self.selected = Some(id);
        self.detail = None;
        self.refresh(cx);
    }

    /// Calls `tool` with `arguments`, then reads the board again; a failure is a toast (D3).
    fn act(&self, tool: &'static str, arguments: serde_json::Value, cx: &Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let workspace = self.workspace.clone();
        cx.spawn(async move |board, cx| {
            let done: anyhow::Result<serde_json::Value> =
                fleet_providers::call_tool(&client, tool, arguments, cx).await;
            if let Err(error) = done {
                workspace
                    .update(cx, |workspace, cx| {
                        struct ForgeAction;
                        workspace.show_toast(
                            Toast::new(
                                NotificationId::unique::<ForgeAction>(),
                                format!("Forge did not take it: {error:#}"),
                            ),
                            cx,
                        );
                    })
                    .log_err();
            }
            board.update(cx, Self::refresh).log_err();
        })
        .detach();
    }

    fn claim(&self, cx: &Context<Self>) {
        if let Some(id) = self.selected.clone() {
            self.act("ticket-claim", json!({ "id": id }), cx);
        }
    }

    fn move_to(&self, status: &str, cx: &Context<Self>) {
        if let Some(id) = self.selected.clone() {
            self.act("ticket-update", json!({ "id": id, "status": status }), cx);
        }
    }

    fn send_comment(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if !self.comment.focus_handle(cx).is_focused(window) {
            return;
        }
        let body = self.comment.read(cx).text(cx).trim().to_string();
        let Some(id) = self.selected.clone() else {
            return;
        };
        if body.is_empty() {
            return;
        }
        self.comment
            .update(cx, |comment, cx| comment.set_text("", window, cx));
        self.act(
            "ticket-comment",
            json!({ "ticket_id": id, "body": body }),
            cx,
        );
    }

    /// New Agent in Worktree with the ticket in its prompt.
    fn start_in_worktree(&self, window: &mut Window, cx: &mut App) {
        let Some(detail) = &self.detail else {
            return;
        };
        let ticket = &detail.ticket;
        let seed = format!(
            "Forge ticket #{}: {}\n\n{}",
            ticket.number, ticket.title, ticket.description
        );
        if let Err(error) = worktree_agents::open_prompt_seeded(
            &self.workspace,
            AgentKind::Claude,
            Vec::new(),
            Some(seed),
            window,
            cx,
        ) {
            log::warn!("forge board: no worktree prompt: {error:#}");
        }
    }

    /// The tickets the filter field keeps.
    fn shown(&self, cx: &App) -> Vec<&Ticket> {
        let query = self.filter.read(cx).text(cx).to_lowercase();
        self.tickets
            .iter()
            .filter(|ticket| {
                query.is_empty()
                    || ticket.title.to_lowercase().contains(&query)
                    || ticket.number.to_string().contains(&query)
            })
            .collect()
    }

    /// The running agent on `ticket`: the one whose current ticket names its number.
    fn agent_of(&self, ticket: &Ticket) -> Option<&AgentSummary> {
        let number = ticket.number.to_string();
        self.agents.iter().find(|agent| {
            agent
                .work_item
                .as_ref()
                .is_some_and(|item| item.id.trim_start_matches('#') == number)
        })
    }
}

/// Reads the sprint, its tickets (every status) and the running agents.
async fn read_forge(client: &Client, cx: &gpui::AsyncApp) -> anyhow::Result<Read> {
    let sprint: SprintAnswer =
        fleet_providers::call_tool(client, "sprint-current", json!({}), cx).await?;
    let mut arguments = json!({ "status": "all", "limit": TICKET_LIMIT });
    if let Some(sprint) = &sprint.sprint {
        arguments["sprint_id"] = json!(sprint.id);
    }
    let tickets: Tickets = fleet_providers::call_tool(client, "ticket-list", arguments, cx).await?;
    let agents: forge::ForgeAgents =
        fleet_providers::call_tool(client, "agent-list", json!({}), cx).await?;
    Ok(Read {
        sprint: sprint.sprint,
        tickets: tickets.tickets,
        agents: forge::summaries(agents),
    })
}

async fn read_detail(client: &Client, id: &str, cx: &gpui::AsyncApp) -> anyhow::Result<Detail> {
    fleet_providers::call_tool(
        client,
        "ticket-get",
        json!({ "id": id, "include_comments": true }),
        cx,
    )
    .await
}

/// How long ago `time` was, in the board's short words.
fn age(time: Option<&str>) -> String {
    let Some(time) = time.and_then(|time| DateTime::parse_from_rfc3339(time).ok()) else {
        return String::new();
    };
    let minutes = Utc::now()
        .signed_duration_since(time.with_timezone(&Utc))
        .num_minutes();
    match minutes {
        ..1 => "now".to_string(),
        1..60 => format!("{minutes} m"),
        60..1440 => format!("{} h", minutes / 60),
        _ => format!("{} d", minutes / 1440),
    }
}

/// The days left in the sprint, from its end.
fn days_left(sprint: &Sprint) -> Option<String> {
    let ends = DateTime::parse_from_rfc3339(sprint.ends_at.as_deref()?).ok()?;
    let days = ends
        .with_timezone(&Utc)
        .signed_duration_since(Utc::now())
        .num_days();
    Some(match days {
        ..0 => "ended".to_string(),
        0 => "ends today".to_string(),
        1 => "1 day left".to_string(),
        days => format!("{days} days left"),
    })
}

impl ForgeBoard {
    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let done = self
            .tickets
            .iter()
            .filter(|ticket| matches!(ticket.status.as_str(), "done" | "closed"))
            .count();
        let sprint = self.sprint.as_ref();
        h_flex()
            .px_4()
            .py_2()
            .gap_3()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(Icon::new(IconName::ListTodo).color(Color::Accent))
            .child(Label::new("Forge").weight(FontWeight::SEMIBOLD))
            .when_some(sprint, |header, sprint| {
                let mut line = sprint.github_release_branch.clone();
                if let Some(left) = days_left(sprint) {
                    if !line.is_empty() {
                        line.push_str(" · ");
                    }
                    line.push_str(&left);
                }
                header
                    .child(Label::new(format!(
                        "Sprint {} · {}",
                        sprint.number, sprint.name
                    )))
                    .child(Label::new(line).size(LabelSize::Small).color(Color::Muted))
            })
            .child(
                Label::new(format!("{done} of {} done", self.tickets.len()))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(div().flex_1())
            .child(
                div()
                    .w(px(220.))
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.filter.clone()),
            )
    }

    fn render_columns(&self, cx: &Context<Self>) -> impl IntoElement {
        let shown = self.shown(cx);
        h_flex()
            .flex_1()
            .min_w_0()
            .items_start()
            .p_3()
            .gap_3()
            .children(
                COLUMNS
                    .iter()
                    .enumerate()
                    .map(|(column, (name, statuses))| {
                        let cards: Vec<&&Ticket> = shown
                            .iter()
                            .filter(|ticket| statuses.contains(&ticket.status.as_str()))
                            .collect();
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_2()
                            .child(
                                h_flex()
                                    .gap_1p5()
                                    .child(Label::new(*name).size(LabelSize::Small))
                                    .child(
                                        Label::new(cards.len().to_string())
                                            .size(LabelSize::XSmall)
                                            .color(Color::Muted),
                                    ),
                            )
                            .children(
                                cards
                                    .into_iter()
                                    .map(|ticket| self.render_card(column, ticket, cx)),
                            )
                    }),
            )
    }

    fn render_card(&self, column: usize, ticket: &Ticket, cx: &Context<Self>) -> AnyElement {
        let colors = cx.theme().colors();
        let selected = self.selected.as_deref() == Some(ticket.id.as_str());
        let agent = self.agent_of(ticket).map_or_else(
            || "not claimed".to_string(),
            |agent| {
                agent.host_id.as_ref().map_or_else(
                    || agent.name.clone(),
                    |host| format!("{} · {host}", agent.name),
                )
            },
        );
        // The QA column says which of its three statuses a card is in.
        let phase = if column == 3 {
            Some(ticket.status.clone())
        } else {
            ticket.phase.clone()
        };
        let id = ticket.id.clone();
        v_flex()
            .id(SharedString::from(format!("forge-card-{}", ticket.id)))
            .p_2()
            .gap_1()
            .rounded_lg()
            .border_1()
            .border_color(if selected {
                colors.border_focused
            } else {
                colors.border
            })
            .bg(if selected {
                colors.ghost_element_selected
            } else {
                colors.elevated_surface_background
            })
            .cursor_pointer()
            .on_click(cx.listener(move |board, _, _, cx| board.select(id.clone(), cx)))
            .child(
                h_flex()
                    .gap_1p5()
                    .child(
                        Label::new(format!("#{}", ticket.number))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new(ticket.kind.clone())
                            .size(LabelSize::XSmall)
                            .color(Color::Accent),
                    )
                    .when_some(phase, |row, phase| {
                        row.child(
                            Label::new(phase)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                    }),
            )
            .child(Label::new(ticket.title.clone()).size(LabelSize::Small))
            .child(
                h_flex()
                    .gap_1p5()
                    .child(
                        Label::new(agent)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(div().flex_1())
                    .child(
                        Label::new(age(ticket.updated_at.as_deref()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .into_any_element()
    }

    fn render_detail(&self, detail: &Detail, cx: &Context<Self>) -> impl IntoElement {
        let ticket = &detail.ticket;
        let agent = self
            .agent_of(ticket)
            .map_or_else(|| "not claimed".to_string(), |agent| agent.name.clone());
        let row = |label: &'static str, value: String| {
            h_flex()
                .gap_3()
                .child(
                    div()
                        .w_16()
                        .child(Label::new(label).size(LabelSize::Small).color(Color::Muted)),
                )
                .child(Label::new(value).size(LabelSize::Small))
        };
        v_flex()
            .debug_selector(|| "marley-forge-detail".into())
            .w(px(360.))
            .flex_none()
            .h_full()
            .p_4()
            .gap_3()
            .border_l_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new(format!("#{}", ticket.number)).color(Color::Muted))
                    .child(Label::new(ticket.kind.clone()).color(Color::Accent))
                    .child(Label::new(ticket.status.clone()).color(Color::Muted)),
            )
            .child(Label::new(ticket.title.clone()).weight(FontWeight::SEMIBOLD))
            .when(!ticket.description.is_empty(), |pane| {
                pane.child(
                    Label::new(ticket.description.clone())
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
            })
            .child(row("Agent", agent))
            .when_some(self.sprint.as_ref(), |pane, sprint| {
                pane.child(row(
                    "Sprint",
                    format!("{} · {}", sprint.number, sprint.name),
                ))
            })
            .child(self.render_comments(detail, cx))
            .child(Self::render_actions(cx))
    }

    fn render_comments(&self, detail: &Detail, cx: &Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(
                Label::new("Comments")
                    .size(LabelSize::Small)
                    .weight(FontWeight::SEMIBOLD),
            )
            .children(detail.comments.iter().map(|comment| {
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        Label::new(age(comment.created_at.as_deref()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(Label::new(comment.body.clone()).size(LabelSize::Small))
            }))
            .child(
                div()
                    .mt_1()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.comment.clone()),
            )
    }

    fn render_actions(cx: &Context<Self>) -> impl IntoElement {
        let board = cx.weak_entity();
        h_flex()
            .gap_1p5()
            .flex_wrap()
            .child(
                Button::new("forge-claim", "Claim")
                    .style(ButtonStyle::Outlined)
                    .on_click(cx.listener(|board, _, _, cx| board.claim(cx))),
            )
            .child(
                PopoverMenu::new("forge-move")
                    .trigger(
                        Button::new("forge-move-button", "Move to…").style(ButtonStyle::Outlined),
                    )
                    .menu(move |window, cx| {
                        let board = board.clone();
                        Some(ContextMenu::build(window, cx, move |mut menu, _, _| {
                            for status in MOVES {
                                let board = board.clone();
                                menu = menu.entry(status, None, move |_, cx| {
                                    board
                                        .update(cx, |board, cx| board.move_to(status, cx))
                                        .log_err();
                                });
                            }
                            menu
                        }))
                    }),
            )
            .child(
                Button::new("forge-worktree", "Start here in a worktree")
                    .style(ButtonStyle::Tinted(ui::TintColor::Accent))
                    .tooltip(Tooltip::text("New Agent in Worktree, with this ticket"))
                    .on_click(
                        cx.listener(|board, _, window, cx| board.start_in_worktree(window, cx)),
                    ),
            )
    }

    fn render_standing(&self) -> Option<AnyElement> {
        let message = match &self.standing {
            Standing::Ready => return None,
            Standing::Loading => "Reading Forge…".to_string(),
            Standing::NoProvider => {
                "No Forge provider is set. Add { \"kind\": \"forge\", \"url\": … } \
                                     to marley.fleet.providers, and its token with marley: set \
                                     forge token."
                    .to_string()
            }
            Standing::Failed(reason) => format!("Forge could not be read: {reason}"),
        };
        Some(
            div()
                .p_4()
                .child(Label::new(message).color(Color::Muted))
                .into_any_element(),
        )
    }
}

impl Render for ForgeBoard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let detail = self
            .detail
            .as_ref()
            .filter(|detail| self.selected.as_deref() == Some(detail.ticket.id.as_str()));
        v_flex()
            .debug_selector(|| "marley-forge-board".into())
            .key_context("ForgeBoard")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::send_comment))
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(self.render_header(cx))
            .children(self.render_standing())
            .when(self.standing == Standing::Ready, |board| {
                board.child(
                    h_flex()
                        .flex_1()
                        .min_h_0()
                        .items_start()
                        .child(self.render_columns(cx))
                        .when_some(detail, |row, detail| {
                            row.child(self.render_detail(detail, cx))
                        }),
                )
            })
    }
}

impl Focusable for ForgeBoard {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for ForgeBoard {}

impl Item for ForgeBoard {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Forge")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ListTodo))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Forge's sprint board"))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }
}
