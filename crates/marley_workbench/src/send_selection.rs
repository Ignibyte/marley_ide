//! Sending the editor's selection to a CLI agent in a terminal (#549).
//!
//! `marley::SendSelectionToAgent`, and in the Marley layout Zed's `agent::AddSelectionToThread`
//! (`ctrl->`) while the focus is in a file's editor and an agent runs in a terminal of the window,
//! type a reference to the selection at the agent's prompt, with no Enter: Claude Code's
//! `@src/auth.rs#L12-40`, and Zed's terminal form `src/auth.rs:12-40 ` for the other agents, the
//! path relative to the agent's folder when the file is inside it. One agent takes it; several
//! open a picker; none leaves the key to Zed's Agent Panel. Rich input open on the terminal takes
//! the reference instead, and nothing is typed while the agent waits on a permission or a
//! question, which a paste would answer.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use editor::Editor;
use fuzzy::{StringMatch, StringMatchCandidate};
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext as _, Context, DismissEvent, Entity, EntityId,
    EventEmitter, FocusHandle, Focusable, Global, Render, Task, WeakEntity, Window,
};
use language::Point;
use marley_agent::{AgentKind, claude_events};
use marley_fleet::State;
use picker::{Picker, PickerDelegate};
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use ui::{HighlightedLabel, ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};
use zed_actions::agent::AddSelectionToThread;

use crate::agent_bar::agent_in;
use crate::agent_events::AgentEvents;
use crate::{AskAgentToFix, SendSelectionToAgent};

/// How many terminals the focus order remembers.
const FOCUS_ORDER: usize = 64;

/// The terminals the focus entered, each once, the newest last: the picker lists them newest
/// first.
#[derive(Default)]
struct FocusOrder(Vec<EntityId>);

impl Global for FocusOrder {}

/// A terminal an agent runs in, and what a send needs of it.
#[derive(Clone)]
pub(crate) struct Target {
    pub(crate) view: Entity<TerminalView>,
    pub(crate) kind: AgentKind,
    pub(crate) project: String,
    /// The seat's state word, for Claude Code with Marley's plugin.
    pub(crate) status: Option<&'static str>,
    /// Whether the agent waits on a permission or a question.
    pub(crate) waiting: bool,
    /// Whether its seat says it is idle at its prompt (#522).
    pub(crate) ready: bool,
    /// The folder its foreground program runs in.
    pub(crate) cwd: Option<PathBuf>,
}

/// What a row of the agent picker stands for: an agent, or copying what would be sent (#522).
#[derive(Clone)]
pub(crate) enum Pick {
    Agent(Target),
    Copy,
}

/// A row of the agent picker: its words and what it stands for.
#[derive(Clone)]
pub(crate) struct Row {
    pub(crate) label: String,
    pub(crate) pick: Pick,
}

/// What the picker does with the row chosen, in the window it opened in.
pub(crate) type OnPick = Box<dyn FnMut(Pick, AnyWindowHandle, &mut App)>;

impl Target {
    pub(crate) fn label(&self) -> String {
        let mut label = format!("{} · {}", self.kind.display_name(), self.project);
        if let Some(status) = self.status {
            label.push_str(" · ");
            label.push_str(status);
        }
        label
    }
}

/// The part of a file a send refers to: its absolute path, and its lines from 1, none for a caret.
struct Selection {
    path: PathBuf,
    lines: Option<(u32, u32)>,
}

/// Registers the action and the key's capture on every workspace, and follows the terminals'
/// focus; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(FocusOrder::default());
    // Failed checks go to an agent through this picker (#718).
    crate::failed_checks::init(cx);
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else {
                return;
            };
            let focus_handle = view.focus_handle(cx);
            cx.on_focus_in(&focus_handle, window, |_, _, cx| {
                let id = cx.entity_id();
                let order = &mut cx.default_global::<FocusOrder>().0;
                order.retain(|seen| *seen != id);
                order.push(id);
                if order.len() > FOCUS_ORDER {
                    order.remove(0);
                }
            })
            .detach();
        },
    )
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &SendSelectionToAgent, window, cx| {
            if let Err(message) = send_selection(workspace, window, cx) {
                show_toast(workspace, message, cx);
            }
        });
        workspace.register_action(|workspace, _: &AskAgentToFix, window, cx| {
            if let Err(message) = ask_to_fix(workspace, window, cx) {
                show_toast(workspace, message, cx);
            }
        });
        workspace.register_action_renderer(|div, _, _, cx| {
            div.capture_action(cx.listener(capture_add_selection))
        });
    })
    .detach();
}

/// Zed's Add to Agent Thread, caught before Zed's handler: in the Marley layout it goes to a
/// terminal agent when the focus is in a file's editor and one runs; otherwise it goes on to the
/// Agent Panel as before.
fn capture_add_selection(
    workspace: &mut Workspace,
    _: &AddSelectionToThread,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if crate::marley_layout(cx) && send_selection(workspace, window, cx).is_ok() {
        cx.stop_propagation();
    }
}

/// Sends the focused editor's selection to the one agent in a terminal of the window, or opens
/// the picker for several; the message says why nothing was sent.
fn send_selection(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    send_lines(workspace, None, window, cx)
}

/// [`send_selection`], with `lines` in place of the selection's when given: inline assist keeps
/// its lines as a highlight and leaves the editor only a caret (#900).
fn send_lines(
    workspace: &mut Workspace,
    lines: Option<(u32, u32)>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let editor = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
        .filter(|editor| editor.focus_handle(cx).contains_focused(window, cx))
        .ok_or("Marley sends the selection of a file's editor: put the focus in one first.")?;
    let mut selection = selection_of(&editor, cx)
        .ok_or("Marley sends the selection of a file's editor: this one shows no file.")?;
    if lines.is_some() {
        selection.lines = lines;
    }
    let mut targets = agent_targets(workspace, cx);
    let window_handle = window.window_handle();
    match targets.len() {
        0 => Err("No agent runs in a terminal of this window."),
        1 => {
            let target = targets.remove(0);
            send(target, &selection, window_handle, cx);
            Ok(())
        }
        _ => {
            let rows = targets
                .into_iter()
                .map(|target| Row {
                    label: target.label(),
                    pick: Pick::Agent(target),
                })
                .collect();
            let on_pick: OnPick = Box::new(move |pick, window, cx| {
                if let Pick::Agent(target) = pick {
                    send(target, &selection, window, cx);
                }
            });
            workspace.toggle_modal(window, cx, |window, cx| {
                TargetPicker::new(
                    rows,
                    "Send the selection to…",
                    on_pick,
                    window_handle,
                    window,
                    cx,
                )
            });
            Ok(())
        }
    }
}

/// Asks an agent to fix the line under the caret of the active editor (#899): "Fix this:", the
/// line's reference and the error or warning there, typed with no Enter, to the one agent or
/// through the picker.
fn ask_to_fix(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let editor = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
        .ok_or("Ask an agent to fix this works in a file's editor.")?;
    let (path, line, problem) = line_and_problem(&editor, cx)
        .ok_or("Ask an agent to fix this works in a file's editor: this one shows no file.")?;
    ask_to_fix_at(workspace, path, line, problem, window, cx)
}

/// Asks an agent to fix the problem at `line` (from 1) of `path`, with its words when there are
/// any: the request #899's key makes, for a row of the drawer's Problems too (#922).
pub(crate) fn ask_to_fix_at(
    workspace: &mut Workspace,
    path: PathBuf,
    line: u32,
    problem: Option<String>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Result<(), &'static str> {
    let mut targets = agent_targets(workspace, cx);
    let window_handle = window.window_handle();
    let request = move |target: &Target| {
        let place = reference(
            target.kind,
            &path,
            target.cwd.as_deref(),
            Some((line, line)),
        );
        let place = place.trim_end();
        problem.as_ref().map_or_else(
            || format!("Fix this: {place} "),
            |problem| format!("Fix this: {place} ({problem}) "),
        )
    };
    match targets.len() {
        0 => Err("No agent runs in a terminal of this window."),
        1 => {
            let target = targets.remove(0);
            let text = request(&target);
            send_text(target, text, false, window_handle, cx);
            Ok(())
        }
        _ => {
            let rows = targets
                .into_iter()
                .map(|target| Row {
                    label: target.label(),
                    pick: Pick::Agent(target),
                })
                .collect();
            let on_pick: OnPick = Box::new(move |pick, window, cx| {
                if let Pick::Agent(target) = pick {
                    let text = request(&target);
                    send_text(target, text, false, window, cx);
                }
            });
            workspace.toggle_modal(window, cx, |window, cx| {
                TargetPicker::new(
                    rows,
                    "Ask which agent to fix it…",
                    on_pick,
                    window_handle,
                    window,
                    cx,
                )
            });
            Ok(())
        }
    }
}

/// The file and line, from 1, under `editor`'s caret, and the first line of the worst error or
/// warning on that line; `None` when the editor shows no file on this machine.
fn line_and_problem(
    editor: &Entity<Editor>,
    cx: &mut App,
) -> Option<(PathBuf, u32, Option<String>)> {
    let (head, snapshot) = editor.update(cx, |editor, cx| {
        let display = editor.display_snapshot(cx);
        let head = editor.selections.newest::<Point>(&display).head();
        (head, editor.buffer().read(cx).snapshot(cx))
    });
    let (buffer, point) = snapshot.point_to_buffer_point(head)?;
    let path = buffer.file()?.as_local()?.abs_path(cx);
    let row = point.row;
    let line = Point::new(row, 0)..Point::new(row, buffer.line_len(row));
    let problem = buffer
        .diagnostics_in_range::<_, Point>(line, false)
        .min_by_key(|entry| entry.diagnostic.severity)
        .and_then(|entry| {
            entry
                .diagnostic
                .message
                .as_str()
                .lines()
                .next()
                .map(str::to_string)
        });
    Some((path, row + 1, problem))
}

/// The newest selection of `editor` as a file and its lines, or `None` when its buffer is no
/// file on this machine.
fn selection_of(editor: &Entity<Editor>, cx: &mut App) -> Option<Selection> {
    let (selection, snapshot) = editor.update(cx, |editor, cx| {
        let display = editor.display_snapshot(cx);
        let newest = editor.selections.newest::<Point>(&display);
        (newest, editor.buffer().read(cx).snapshot(cx))
    });
    let (buffer, range) = snapshot
        .range_to_buffer_range(selection.range())
        .or_else(|| {
            // A selection across two excerpts has no one place: the cursor's buffer stands in.
            let (buffer, point) = snapshot.point_to_buffer_point(selection.head())?;
            Some((buffer, point..point))
        })?;
    let path = buffer.file()?.as_local()?.abs_path(cx);
    Some(Selection {
        path,
        lines: line_span(range.start, range.end),
    })
}

/// The lines, from 1, a selection from `start` to `end` covers; none for a caret. A selection
/// that ends at a line's start leaves that line out, as a selection of whole lines does.
fn line_span(start: Point, end: Point) -> Option<(u32, u32)> {
    if start == end {
        return None;
    }
    let first = start.row + 1;
    let last = if end.column == 0 && end.row > start.row {
        end.row
    } else {
        end.row + 1
    };
    Some((first, last))
}

/// What the agent is handed for `lines` of `file`: Claude Code's `@path#L<a>-<b>`, else Zed's
/// terminal form `path:<a>-<b> `, with a trailing space. The path is relative to `cwd`, the
/// agent's folder, when the file lies under it, else absolute.
fn reference(
    kind: AgentKind,
    file: &Path,
    cwd: Option<&Path>,
    lines: Option<(u32, u32)>,
) -> String {
    let shown = cwd
        .and_then(|cwd| file.strip_prefix(cwd).ok())
        .filter(|relative| !relative.as_os_str().is_empty())
        .unwrap_or(file)
        .to_string_lossy();
    let span = lines.map(|(first, last)| {
        if first == last {
            first.to_string()
        } else {
            format!("{first}-{last}")
        }
    });
    match (kind, span) {
        (AgentKind::Claude, Some(span)) => format!("@{shown}#L{span}"),
        (AgentKind::Claude, None) => format!("@{shown}"),
        (_, Some(span)) => format!("{shown}:{span} "),
        (_, None) => format!("{shown} "),
    }
}

/// Every terminal of the window `workspace` is in, the center panes' and the Terminal Panel's,
/// whose foreground program is a known agent, the one focused last first. `workspace` is read as
/// given, since this runs in its update; the window's other workspaces through their entities.
pub(crate) fn agent_targets(workspace: &Workspace, cx: &App) -> Vec<Target> {
    let current = workspace.weak_handle().entity_id();
    let others: Vec<Entity<Workspace>> = workspace
        .multi_workspace()
        .and_then(WeakEntity::upgrade)
        .map(|multi_workspace| {
            multi_workspace
                .read(cx)
                .workspaces()
                .filter(|other| other.entity_id() != current)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let mut views = terminal_views(workspace, cx);
    for other in &others {
        views.extend(terminal_views(other.read(cx), cx));
    }
    let order = cx
        .try_global::<FocusOrder>()
        .map(|order| order.0.clone())
        .unwrap_or_default();
    let focused_last = |view: &Entity<TerminalView>| {
        order
            .iter()
            .position(|id| *id == view.entity_id())
            .map_or(0, |position| position + 1)
    };
    views.sort_by_key(|(_, view)| std::cmp::Reverse(focused_last(view)));
    views
        .into_iter()
        .filter_map(|(project, view)| target_of(project, view, cx))
        .collect()
}

/// Inline assist's line under its prompt (#900): its lines and the agent a terminal of the
/// window runs, and a click sends those lines as `marley: send selection to agent` sends a
/// selection. No agent, no line.
pub(crate) fn inline_assist_line(
    workspace: &WeakEntity<Workspace>,
    lines: Range<u32>,
    cx: &App,
) -> Option<AnyElement> {
    let workspace = workspace.upgrade()?;
    let targets = agent_targets(workspace.read(cx), cx);
    let to = match targets.as_slice() {
        [] => return None,
        [target] => {
            // A terminal's title can be its whole command line; the line keeps to one row.
            let title = target.view.read(cx).tab_content_text(0, cx);
            let first = title.lines().next().unwrap_or_default().trim();
            let title = util::truncate_and_trailoff(first, 32);
            format!("{} in \u{201c}{title}\u{201d}", target.kind.display_name())
        }
        several => format!("one of the {} agents in this window", several.len()),
    };
    let span = (lines.start, lines.end - 1);
    let lines = if lines.len() <= 1 {
        format!("line {}", lines.start)
    } else {
        format!("lines {}\u{2013}{}", lines.start, lines.end - 1)
    };
    let workspace = workspace.downgrade();
    Some(
        h_flex()
            .id("marley-inline-assist-send")
            .debug_selector(|| "marley-inline-assist-send".into())
            .py_0p5()
            .gap_1()
            .cursor_pointer()
            .child(
                div().min_w_0().child(
                    Label::new(format!("Or send {lines} to {to}"))
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                ),
            )
            .child(
                div().flex_none().child(
                    Label::new("\u{b7} marley: send selection to agent")
                        .size(LabelSize::Small)
                        .color(Color::Disabled),
                ),
            )
            .on_click(move |_, window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        if let Err(message) = send_lines(workspace, Some(span), window, cx) {
                            show_toast(workspace, message, cx);
                        }
                    })
                    .log_err();
            })
            .into_any_element(),
    )
}

/// The terminal views of `workspace`, center and docked, with its project's name.
fn terminal_views(workspace: &Workspace, cx: &App) -> Vec<(String, Entity<TerminalView>)> {
    let project = workspace
        .project()
        .read(cx)
        .visible_worktrees(cx)
        .next()
        .map(|worktree| worktree.read(cx).root_name_str().to_string())
        .unwrap_or_default();
    let docked: Vec<Entity<TerminalView>> = workspace
        .panel::<TerminalPanel>(cx)
        .map(|panel| {
            panel
                .read(cx)
                .panes()
                .into_iter()
                .flat_map(|pane| pane.read(cx).items_of_type::<TerminalView>())
                .collect()
        })
        .unwrap_or_default();
    workspace
        .items_of_type::<TerminalView>(cx)
        .chain(docked)
        .map(|view| (project.clone(), view))
        .collect()
}

/// `view` as a target, when a known agent is its foreground program.
pub(crate) fn target_of(project: String, view: Entity<TerminalView>, cx: &App) -> Option<Target> {
    let terminal = view.read(cx).terminal().read(cx);
    let kind = agent_in(terminal)?;
    let cwd = terminal.working_directory();
    let seat = cx
        .try_global::<AgentEvents>()
        .and_then(|events| events.seat(view.entity_id()));
    Some(Target {
        kind,
        project,
        status: seat.map(|seat| claude_events::seat_status(seat.state).label()),
        waiting: seat.is_some_and(|seat| seat.state == State::Waiting),
        ready: seat.is_some_and(|seat| seat.state == State::Idle),
        cwd,
        view,
    })
}

/// Mentions the selection to `target` over Claude Code's IDE link when it has one that takes
/// mentions, its terminal brought to the front and focused (#653); otherwise types its reference
/// at the prompt, as [`send_text`] does. Rich input open on the terminal, and an agent that waits,
/// go to [`send_text`] as before.
fn send(target: Target, selection: &Selection, window: AnyWindowHandle, cx: &mut App) {
    if target.kind == AgentKind::Claude
        && !target.waiting
        && !crate::rich_input::is_open(&target.view, cx)
        && crate::claude_ide::mention(
            target.view.entity_id(),
            &selection.path,
            selection.lines,
            cx,
        )
    {
        cx.defer(move |cx| {
            window
                .update(cx, |_, window, cx| {
                    window.activate_window();
                    crate::browser::reveal_terminal(&target.view, window, cx);
                    window.focus(&target.view.focus_handle(cx), cx);
                })
                .log_err();
        });
        return;
    }
    let text = reference(
        target.kind,
        &selection.path,
        target.cwd.as_deref(),
        selection.lines,
    );
    send_text(target, text, false, window, cx);
}

/// Types `text` at `target`'s prompt after this update, as a pick's Send does: its window
/// activated, its tab brought to the front and focused, one paste and no Enter. Rich input open
/// on it takes the text instead; an agent that waits is left alone, with a toast.
pub(crate) fn send_text(
    target: Target,
    text: String,
    submit: bool,
    window: AnyWindowHandle,
    cx: &mut App,
) {
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                if crate::rich_input::is_open(&target.view, cx) {
                    // The terminal comes to the front, so the editor the text goes into shows.
                    window.activate_window();
                    crate::browser::reveal_terminal(&target.view, window, cx);
                    crate::rich_input::insert(&target.view, &text, window, cx);
                    return;
                }
                if target.waiting {
                    let message = format!(
                        "{} in {} waits on a permission or a question; nothing was sent.",
                        target.kind.display_name(),
                        target.project
                    );
                    if let Some(workspace) = target.view.read(cx).marley_workspace().upgrade() {
                        workspace.update(cx, |workspace, cx| show_toast(workspace, message, cx));
                    }
                    return;
                }
                window.activate_window();
                crate::browser::reveal_terminal(&target.view, window, cx);
                window.focus(&target.view.focus_handle(cx), cx);
                let terminal = target.view.read(cx).terminal().clone();
                if submit {
                    // A request asked at a prompt runs as the agent's next prompt (#557).
                    crate::terminal_drive::paste_then(
                        &terminal,
                        &text,
                        |terminal| terminal.input(b"\r".to_vec()),
                        cx,
                    )
                    .detach();
                } else {
                    terminal.update(cx, |terminal, _| terminal.paste(&text));
                }
            })
            .log_err();
    });
}

/// The language servers menu's Ask an Agent (#901): types the server's name and the first line of
/// its message at the window's terminal agent, or opens the picker for several; a toast says why
/// nothing was typed.
pub(crate) fn ask_about_server(
    workspace: &WeakEntity<Workspace>,
    server: &str,
    message: &str,
    window: &mut Window,
    cx: &mut App,
) {
    let line = message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .trim_end_matches('.');
    let text = format!("The language server {server} reports: {line}. Why, and how do I fix it? ");
    let window_handle = window.window_handle();
    workspace
        .update(cx, |workspace, cx| {
            let mut targets = agent_targets(workspace, cx);
            match targets.len() {
                0 => show_toast(workspace, "No agent runs in a terminal of this window.", cx),
                1 => send_text(targets.remove(0), text, false, window_handle, cx),
                _ => {
                    let rows = targets
                        .into_iter()
                        .map(|target| Row {
                            label: target.label(),
                            pick: Pick::Agent(target),
                        })
                        .collect();
                    let on_pick: OnPick = Box::new(move |pick, window, cx| {
                        if let Pick::Agent(target) = pick {
                            send_text(target, text.clone(), false, window, cx);
                        }
                    });
                    workspace.toggle_modal(window, cx, |window, cx| {
                        TargetPicker::new(
                            rows,
                            "Ask which agent about the server…",
                            on_pick,
                            window_handle,
                            window,
                            cx,
                        )
                    });
                }
            }
        })
        .log_err();
}

pub(crate) fn show_toast(
    workspace: &mut Workspace,
    message: impl Into<String>,
    cx: &mut Context<Workspace>,
) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<FocusOrder>(), message.into()),
        cx,
    );
}

/// The picker of an agent to send to: a selection's when several agents run (#549), review
/// notes' (#522), one row an agent, the one focused last first.
pub(crate) struct TargetPicker {
    picker: Entity<Picker<TargetDelegate>>,
}

impl TargetPicker {
    pub(crate) fn new(
        rows: Vec<Row>,
        placeholder: &'static str,
        on_pick: OnPick,
        window_handle: AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let delegate = TargetDelegate {
            modal: cx.entity().downgrade(),
            rows,
            placeholder,
            on_pick,
            window: window_handle,
            matches: Vec::new(),
            selected_index: 0,
        };
        Self {
            picker: cx.new(|cx| Picker::uniform_list(delegate, window, cx)),
        }
    }
}

impl ModalView for TargetPicker {}

impl EventEmitter<DismissEvent> for TargetPicker {}

impl Focusable for TargetPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for TargetPicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("SendSelectionPicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

struct TargetDelegate {
    modal: WeakEntity<TargetPicker>,
    rows: Vec<Row>,
    placeholder: &'static str,
    on_pick: OnPick,
    window: AnyWindowHandle,
    /// The targets the query matches, in order, each naming its target by index.
    matches: Vec<StringMatch>,
    selected_index: usize,
}

impl PickerDelegate for TargetDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "marley send selection"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected_index
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected_index = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        self.placeholder.into()
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let candidates: Vec<StringMatchCandidate> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| StringMatchCandidate::new(index, &row.label))
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps the targets in their focus order; fuzzy matching would sort
            // them by score.
            let matches = if query.is_empty() {
                candidates
                    .into_iter()
                    .map(|candidate| StringMatch {
                        candidate_id: candidate.id,
                        score: 0.,
                        positions: Vec::new(),
                        string: candidate.string,
                    })
                    .collect()
            } else {
                let cancelled = AtomicBool::new(false);
                fuzzy::match_strings(&candidates, &query, false, true, 100, &cancelled, executor)
                    .await
            };
            picker
                .update(cx, |picker, cx| {
                    picker.delegate.matches = matches;
                    picker.delegate.selected_index = 0;
                    cx.notify();
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let pick = self
            .matches
            .get(self.selected_index)
            .and_then(|found| self.rows.get(found.candidate_id))
            .map(|row| row.pick.clone());
        if let Some(pick) = pick {
            (self.on_pick)(pick, self.window, cx);
        }
        self.dismissed(window, cx);
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        self.modal
            .update(cx, |_, cx| cx.emit(DismissEvent))
            .log_err();
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let found = self.matches.get(index)?;
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(HighlightedLabel::new(
                    found.string.clone(),
                    found.positions.clone(),
                )),
        )
    }
}
