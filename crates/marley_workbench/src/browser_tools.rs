//! The browser family of Marley's MCP server (#492, #493): the pages in the Browser tabs, seen
//! and driven by agents.
//!
//! Each call answers from a task of its own, since each waits on the browser, and acts on the
//! page its `tab` names, a page's id from `browser_tabs`. A call that names none acts in the
//! caller's project (#574), the project of the terminal it comes from or of the folder it runs
//! in: on the page whose tab of that project the user focused last. A caller in no project of
//! Marley's gets the page whose tab the user focused last anywhere. A read tool reads the page as
//! the user sees it. A write tool first brings the page's tab to the front where that leaves the
//! focus alone, so the user watches, says what it does in the tab's Agent chip, and sends the
//! same CDP events the user's keys and mouse send (`marley_browser::input`), so the page cannot
//! tell them apart.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use base64::Engine as _;

use gpui::{
    AnyWindowHandle, App, AppContext as _, AsyncApp, Entity, EntityId, MouseButton, WeakEntity,
};
use marley_browser::address;
use marley_browser::cdp::CdpError;
use marley_browser::input::{self, KeyPress};
use marley_browser::observe::redact_url;
use marley_browser::page::Page;
use marley_browser::pick::{self, PageBox, PickBundle};
use marley_browser::playwright;
use marley_browser::recorder;
use marley_browser::snapshot::{self, FrameTree, RefTarget, Snapshot};
use marley_mcp::redact::Redactor;
use marley_mcp::{AppCall, Caller, ToolAnswer, ToolImage};
use serde::Serialize;
use serde_json::{Value, json};
use ui::SharedString;
use workspace::{MultiWorkspace, ProjectGroup, Workspace};

use crate::browser::{
    BrowserHub, BrowserProject, Maker, Pick, SCROLL_SETTLE, TabSummary, new_page, open_url_tab,
    recordings_dir, settled, show_for_agent, tab_workspaces, window_of,
};
use crate::links;

/// How long a call waits, once the browser shows its pages, for the page it acts on to attach.
const ATTACH_WAIT: Duration = Duration::from_secs(5);

/// How often it looks.
const POLL: Duration = Duration::from_millis(100);

/// The most of a fill's text an agent reads in a recording (#506), cut after the redaction.
const FILL_BUDGET: usize = 1_000;

/// The names Playwright looks for its config under, in the project's root.
const PLAYWRIGHT_CONFIGS: [&str; 6] = [
    "playwright.config.ts",
    "playwright.config.js",
    "playwright.config.mjs",
    "playwright.config.cjs",
    "playwright.config.mts",
    "playwright.config.cts",
];

/// The tools that act in a page they find, which bring its tab to the front first.
const WRITES: &[&str] = &[
    "browser_annotate",
    "browser_back",
    "browser_click",
    "browser_type",
    "browser_press",
    "browser_scroll",
];

/// The project a call comes from (#574), as the rail groups projects: a call that names no tab
/// acts among its tabs.
struct Scope {
    /// The group's workspaces.
    workspaces: HashSet<EntityId>,
    /// The workspace the caller runs in, where a tab opened for it goes.
    home: WeakEntity<Workspace>,
    /// The group's name, as the rail shows it.
    name: String,
}

/// Answers `call`, a `browser_*` tool, from a task of its own.
///
/// The browser of the caller's project starts again when it stopped for a reason, as `marley:
/// open browser` starts it; asking starts no browser that is not running (#507).
pub fn answer(call: AppCall, cx: &mut App) {
    let hub = BrowserHub::global(cx);
    let scope = caller_scope(call.caller(), cx);
    if let Some(project) = caller_project(scope.as_ref(), cx) {
        hub.update(cx, |hub, cx| hub.restart_if_failed(&project.key, cx));
    }
    // An outside client's name marks what it does in the tab (#524).
    let by = call.principal().client_name().map(SharedString::from);
    cx.spawn(async move |cx| {
        let result = run(
            &call.tool,
            &call.arguments,
            scope.as_ref(),
            by.as_ref(),
            &hub,
            cx,
        )
        .await;
        call.answer(result);
    })
    .detach();
}

/// Refuses the call of a client the user cut off after its call reached Marley (#524): the call
/// waited for the main thread or for the browser, and acts no more.
fn still_allowed(by: Option<&SharedString>, cx: &AsyncApp) -> Result<(), String> {
    match by {
        Some(name) if !cx.update(|cx| crate::clients::is_allowed(name, cx)) => Err(format!(
            "the user cut the client {name} off, so its call was not carried out"
        )),
        _ => Ok(()),
    }
}

/// The project `caller` comes from (#574): the group of its terminal, else of the workspace one
/// of whose folders holds its `Marley-Project`, else its `Marley-Cwd`, the longest folder winning.
/// None for a caller in no project of Marley's.
fn caller_scope(caller: &Caller, cx: &App) -> Option<Scope> {
    let home = crate::mcp::caller_terminal(caller, cx)
        .map(|(workspace, _)| workspace)
        .or_else(|| holding(caller.project.as_deref()?, cx))
        .or_else(|| holding(caller.cwd.as_deref()?, cx))?;
    let multi_workspace = multi_workspaces(cx).into_iter().find(|multi_workspace| {
        multi_workspace
            .workspaces()
            .any(|workspace| *workspace == home)
    })?;
    let key = home.read(cx).project_group_key(cx);
    let workspaces = multi_workspace
        .workspaces()
        .filter(|workspace| workspace.read(cx).project_group_key(cx) == key)
        .map(Entity::entity_id)
        .collect();
    let name = names_in(multi_workspace, cx)
        .remove(&home.entity_id())
        .unwrap_or_default();
    Some(Scope {
        workspaces,
        home: home.downgrade(),
        name,
    })
}

/// The project a page a call opens goes to (#507): the caller's, else the one the active window
/// shows. None with no window.
fn caller_project(scope: Option<&Scope>, cx: &App) -> Option<BrowserProject> {
    let home = scope.and_then(|scope| scope.home.upgrade()).or_else(|| {
        cx.active_window()
            .and_then(|window| window.downcast::<MultiWorkspace>())
            .or_else(|| {
                cx.windows()
                    .into_iter()
                    .find_map(|window| window.downcast::<MultiWorkspace>())
            })
            .and_then(|window| window.read(cx).ok())
            .map(|multi_workspace| multi_workspace.workspace().clone())
    })?;
    Some(BrowserProject::of_workspace(&home, cx))
}

/// The local workspace one of whose folders holds `path`, the longest folder winning. Folders are
/// each workspace's own, so a linked worktree's finds its workspace, not its repository's.
fn holding(path: &str, cx: &App) -> Option<Entity<Workspace>> {
    let path = Path::new(path);
    multi_workspaces(cx)
        .into_iter()
        .flat_map(MultiWorkspace::workspaces)
        .filter(|workspace| workspace.read(cx).project().read(cx).is_local())
        .flat_map(|workspace| {
            workspace
                .read(cx)
                .root_paths(cx)
                .into_iter()
                .filter(|root| path.starts_with(root))
                .map(|root| (root.components().count(), workspace.clone()))
                .collect::<Vec<_>>()
        })
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, workspace)| workspace)
}

/// Each of Marley's windows.
fn multi_workspaces(cx: &App) -> Vec<&MultiWorkspace> {
    cx.windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
        .collect()
}

/// The name of each workspace's project in `multi_workspace`, as the rail shows it; a workspace
/// the rail does not list is named for its folders.
fn names_in(multi_workspace: &MultiWorkspace, cx: &App) -> HashMap<EntityId, String> {
    let groups: Vec<ProjectGroup> = multi_workspace
        .project_groups(cx)
        .into_iter()
        .filter(|group| !group.workspaces.is_empty())
        .collect();
    let names = crate::group_names(&groups);
    multi_workspace
        .workspaces()
        .map(|workspace| {
            let key = workspace.read(cx).project_group_key(cx);
            let name = groups
                .iter()
                .zip(&names)
                .find(|(group, _)| group.key == key)
                .map_or_else(
                    || key.display_name(&HashMap::new()).to_string(),
                    |(_, name)| name.clone(),
                );
            (workspace.entity_id(), name)
        })
        .collect()
}

/// The pages of the scope's Browser tabs.
fn targets_in(scope: &Scope, cx: &mut App) -> HashSet<String> {
    tab_workspaces(cx)
        .into_iter()
        .filter(|(_, workspace)| scope.workspaces.contains(&workspace.entity_id()))
        .map(|(target, _)| target)
        .collect()
}

/// The page a call that names no tab acts on: in the caller's project, the page of its tab the
/// user focused last, else of its newest; for a caller in no project, the page whose tab the user
/// focused last anywhere.
fn default_tab(hub: &Entity<BrowserHub>, scope: Option<&Scope>, cx: &mut App) -> Option<String> {
    match scope {
        Some(scope) => {
            let targets = targets_in(scope, cx);
            hub.read(cx).focused_among(&targets)
        }
        None => hub.read(cx).focused(),
    }
}

async fn run(
    tool: &str,
    arguments: &Value,
    scope: Option<&Scope>,
    by: Option<&SharedString>,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    still_allowed(by, cx)?;
    // Picks are Marley's, kept while the browser restarts, and recordings are files.
    match tool {
        "browser_picks" => return Ok(picks(hub, cx)),
        "browser_pick" => return pick(arguments, hub, cx),
        "browser_recordings" => return recordings(cx).await,
        "browser_recording" => return recording(arguments, cx).await,
        "browser_draft_test" => return draft_test(arguments, cx).await,
        // Before the browser is up: a new tab waits for it by itself.
        "browser_open_url" => return Ok(cx.update(|cx| open_url(arguments, cx))),
        _ => {}
    }
    // Each project's browser has its own pages (#507): a call sees them once none starts.
    settled(hub, cx).await;
    still_allowed(by, cx)?;
    let named_tab = arguments.get("tab").and_then(Value::as_str);
    match tool {
        "browser_tabs" => return tabs(hub, scope, cx).await,
        "browser_navigate" => return navigate(arguments, named_tab, scope, hub, by, cx).await,
        "browser_check_pick" => return check_pick(arguments, hub, by, cx).await,
        _ => {}
    }
    let (tab, page) = page_of(hub, named_tab, scope, cx).await?;
    if WRITES.contains(&tool) {
        cx.update(|cx| show_for_agent(&tab, cx));
    }
    match tool {
        "browser_look" => look(&page, &tab, hub, cx).await,
        "browser_snapshot" => take_snapshot(&page, &tab, arguments, hub, cx).await,
        "browser_find" => find_element(&page, &tab, arguments, hub, cx).await,
        "browser_console" => {
            let mut entries_read = hub.read_with(cx, |hub, _| hub.console_entries(&tab));
            let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
            if let Some(redactor) = redactor {
                for entry in &mut entries_read {
                    entry.text = redactor.redact(&entry.text).text;
                }
            }
            entries(&tab, entries_read)
        }
        "browser_network" => entries(&tab, hub.read_with(cx, |hub, _| hub.network_entries(&tab))),
        "browser_annotations" => Ok(annotations(&tab, hub, cx)),
        "browser_annotate" => annotate(&page, &tab, arguments, hub, by, cx).await,
        "browser_back" => back(&tab, hub, by, cx).await,
        "browser_click" => click(&page, &tab, arguments, hub, by, cx).await,
        "browser_type" => type_text(&page, &tab, arguments, hub, by, cx).await,
        "browser_press" => press(&page, &tab, arguments, hub, by, cx).await,
        "browser_scroll" => scroll(&page, &tab, arguments, hub, by, cx).await,
        other => Err(format!("Marley answers no tool named {other}")),
    }
}

/// `browser_open_url` (#561): a URL a program in `directory` asked its opener to open, in a
/// Browser tab of the project one of whose folders holds `directory`, with the focus, when
/// `marley.terminal_links` sends it to a Browser tab. It answers at once, since the program waits
/// on its opener, and opens nothing for a URL it declines: the opener sends that one to the
/// system browser itself.
fn open_url(arguments: &Value, cx: &mut App) -> ToolAnswer {
    let answer = |structured: Value| ToolAnswer {
        structured,
        text: None,
        image: None,
    };
    let declined = |reason: &str| answer(json!({ "opened": false, "reason": reason }));
    let text = |name: &str| {
        arguments
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
    };
    let Ok(url) = address::agent_url(text("url")) else {
        return declined("not an http or https URL");
    };
    let Some(url) = links::browser_tab_url(&url, cx) else {
        return declined("marley.terminal_links sends this URL to the system browser");
    };
    let Some(workspace) = holding(text("directory"), cx) else {
        return declined("no project of Marley's holds the directory");
    };
    let Some(window) = window_of(&workspace, cx) else {
        return declined("the project's window is gone");
    };
    let project = window
        .read(cx)
        .ok()
        .and_then(|multi_workspace| names_in(multi_workspace, cx).remove(&workspace.entity_id()))
        .unwrap_or_default();
    // Through the window's handle, which leaves its root alone, so the workspace is shown and its
    // tab added each in an update of its own.
    let shown = AnyWindowHandle::from(window).update(cx, |_, window, cx| {
        if let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten()
            && multi_workspace.read(cx).workspace() != &workspace
        {
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            });
        }
        workspace.update(cx, |workspace, cx| open_url_tab(workspace, url, window, cx));
    });
    match shown {
        Ok(()) => answer(json!({ "opened": true, "project": project })),
        Err(_) => declined("the project's window is gone"),
    }
}

/// The page `named_tab` names, or else the page a call that names no tab acts on
/// ([`default_tab`]), with its id, once it is attached.
async fn page_of(
    hub: &Entity<BrowserHub>,
    named_tab: Option<&str>,
    scope: Option<&Scope>,
    cx: &AsyncApp,
) -> Result<(String, Page), String> {
    let mut waited = Duration::ZERO;
    loop {
        let (found, has_pages) = cx.update(|cx| {
            let (tab, has_pages) = match (named_tab, scope) {
                (Some(tab), _) => (Some(tab.to_string()), true),
                (None, Some(scope)) => {
                    let targets = targets_in(scope, cx);
                    (hub.read(cx).focused_among(&targets), !targets.is_empty())
                }
                (None, None) => {
                    let hub = hub.read(cx);
                    (hub.focused(), hub.has_pages())
                }
            };
            let found = tab.and_then(|tab| hub.read(cx).page(&tab).map(|page| (tab, page)));
            (found, has_pages)
        });
        if let Some(found) = found {
            return Ok(found);
        }
        // A page just made attaches in a moment; with none at all there is nothing to wait for.
        if !has_pages || waited >= ATTACH_WAIT {
            return Err(match (named_tab, scope) {
                (Some(tab), _) => format!("the browser has no tab {tab}; browser_tabs lists them"),
                (None, Some(scope)) => format!(
                    "no Browser tab of the project {} shows a page; browser_navigate opens one \
                     there",
                    scope.name
                ),
                (None, None) => "the browser has no tabs; browser_navigate opens one".to_string(),
            });
        }
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
}

/// What `browser_tabs` says of a tab: the hub's summary, the tab's project, and whether a call
/// from this caller that names no tab acts on it (#574).
#[derive(Serialize)]
struct TabRow {
    #[serde(flatten)]
    summary: TabSummary,
    project: Option<String>,
    default: bool,
}

/// `browser_tabs`: every page, once the pages being attached are.
async fn tabs(
    hub: &Entity<BrowserHub>,
    scope: Option<&Scope>,
    cx: &AsyncApp,
) -> Result<ToolAnswer, String> {
    let mut waited = Duration::ZERO;
    while hub.read_with(cx, |hub, _| hub.is_attaching()) && waited < ATTACH_WAIT {
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
    let tabs: Vec<TabRow> = cx.update(|cx| {
        let placed = tab_workspaces(cx);
        let names: HashMap<EntityId, String> = multi_workspaces(cx)
            .into_iter()
            .flat_map(|multi_workspace| names_in(multi_workspace, cx))
            .collect();
        let default = default_tab(hub, scope, cx);
        hub.read(cx)
            .tabs()
            .into_iter()
            .map(|mut summary| {
                summary.url = redact_url(&summary.url);
                // A page no tab shows yet is named by its browser's project (#507).
                let project = placed
                    .iter()
                    .find(|(target, _)| *target == summary.id)
                    .and_then(|(_, workspace)| names.get(&workspace.entity_id()).cloned())
                    .or_else(|| {
                        hub.read(cx)
                            .project_of(&summary.id)
                            .map(|project| project.name.to_string())
                    });
                TabRow {
                    default: default.as_deref() == Some(summary.id.as_str()),
                    project,
                    summary,
                }
            })
            .collect()
    });
    let tabs = serde_json::to_value(tabs).map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured: json!({ "tabs": tabs }),
        text: None,
        image: None,
    })
}

async fn look(
    page: &Page,
    tab: &str,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<ToolAnswer, String> {
    let (url, title, loading) = hub.read_with(cx, |hub, _| {
        (hub.url(tab), hub.title(tab), hub.is_loading(tab))
    });
    let viewport = page.viewport().await.map_err(|error| error.to_string())?;
    let focused = page
        .focused_element()
        .await
        .map_err(|error| error.to_string())?;
    // A password field's selection is part of the password.
    let in_password = focused
        .as_ref()
        .is_some_and(|element| element.kind.as_deref() == Some("password"));
    let selection = if in_password {
        String::new()
    } else {
        page.selected_text()
            .await
            .map_err(|error| error.to_string())?
    };
    let image = page.screenshot().await.map_err(|error| error.to_string())?;
    let structured = serde_json::to_value(Look {
        tab: tab.to_string(),
        url: url.map(|url| redact_url(&url)).unwrap_or_default(),
        title: title.map(|title| title.to_string()).unwrap_or_default(),
        loading,
        viewport,
        focused,
        selection,
    })
    .map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured,
        text: None,
        image: Some(ToolImage {
            mime_type: "image/jpeg".to_string(),
            data: image,
        }),
    })
}

/// `browser_recordings`: the recordings Record this saved, read off the main thread (#499).
async fn recordings(cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let dir = recordings_dir();
    let recordings = cx
        .background_spawn(futures::future::lazy(move |_| recorder::list_in(&dir)))
        .await
        .map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured: json!({ "recordings": recordings }),
        text: None,
        image: None,
    })
}

/// `browser_recording`: a recording's timeline, and one of its frames as the image (#499).
async fn recording(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "browser_recording needs a recording's id, from browser_recordings".to_string()
        })?
        .to_string();
    let frame = arguments
        .get("frame")
        .and_then(Value::as_u64)
        .and_then(|frame| usize::try_from(frame).ok());
    let dir = recordings_dir();
    let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
    let (timeline, jpeg) = cx
        .background_spawn(futures::future::lazy(move |_| {
            let timeline = recorder::read_in(&dir, &id)?;
            let jpeg = frame
                .map(|frame| recorder::frame_in(&dir, &id, frame))
                .transpose()?;
            let timeline = timeline_for_agents(timeline, redactor.as_deref(), false);
            Ok::<_, recorder::RecordingError>((timeline, jpeg))
        }))
        .await
        .map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured: timeline,
        text: None,
        image: jpeg.map(|bytes| ToolImage {
            mime_type: "image/jpeg".to_string(),
            data: base64::engine::general_purpose::STANDARD.encode(bytes),
        }),
    })
}

/// `browser_draft_test` (#506): a Playwright test drafted from a recording, with where it would
/// go in the recording's project; it writes nothing, and the agent puts the test in place.
async fn draft_test(arguments: &Value, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "browser_draft_test needs a recording's id, from browser_recordings".to_string()
        })?
        .to_string();
    let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
    let dir = recordings_dir();
    let structured = cx
        .background_spawn(futures::future::lazy(move |_| {
            draft_in(&dir, &id, redactor.as_deref())
        }))
        .await?;
    Ok(ToolAnswer {
        structured,
        text: None,
        image: None,
    })
}

/// The test drafted from the recording `id` in `dir`, as `browser_draft_test` answers it: the
/// test, its suggested path in the recording's project, the variables it reads, what it left
/// out, where it starts and how to run it.
fn draft_in(dir: &Path, id: &str, redactor: Option<&Redactor>) -> Result<Value, String> {
    let timeline = recorder::read_in(dir, id).map_err(|error| error.to_string())?;
    let timeline = timeline_for_agents(timeline, redactor, true);
    let draft = playwright::draft(&timeline).map_err(|error| error.to_string())?;
    let project = timeline
        .get("project")
        .and_then(Value::as_str)
        .map(PathBuf::from);
    let configured = project.as_deref().and_then(configured_test_dir);
    let test_dir = configured
        .as_ref()
        .and_then(|(_, test_dir)| test_dir.clone())
        .unwrap_or_else(|| "tests".to_string());
    let title = timeline
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let relative = Path::new(&test_dir).join(format!("{}-{id}.spec.ts", slug(title)));
    let note = match (&project, &configured) {
        (None, _) => Some(
            "The recording names no project: put the test where the project's Playwright config \
             looks for tests.",
        ),
        (Some(_), None) => Some(
            "The project has no playwright.config; `npm init playwright@latest` sets Playwright \
             up.",
        ),
        (Some(_), Some(_)) => None,
    };
    Ok(json!({
        "id": id,
        "test": draft.text,
        "path": project.as_ref().map_or_else(
            || relative.to_string_lossy().into_owned(),
            |root| root.join(&relative).to_string_lossy().into_owned(),
        ),
        "project": project.map(|root| root.to_string_lossy().into_owned()),
        "env": draft.env,
        "skipped": draft.skipped,
        "start": draft.start,
        "run": format!("npx playwright test {}", relative.to_string_lossy()),
        "note": note,
    }))
}

/// The recording `timeline` as an agent may read it (#506): every string passed through the
/// redactor whole. For a draft, a fill whose text the redactor changed becomes a secret one,
/// its text gone, which the draft reads from the environment. For `browser_recording`, each
/// fill's text is then cut to [`FILL_BUDGET`].
fn timeline_for_agents(mut timeline: Value, redactor: Option<&Redactor>, for_draft: bool) -> Value {
    let redact = |text: &str| {
        redactor.map_or_else(|| text.to_string(), |redactor| redactor.redact(text).text)
    };
    if let Some(fields) = timeline.as_object_mut() {
        for (key, value) in fields.iter_mut() {
            if key != "entries" {
                redact_strings(value, &redact);
            }
        }
    }
    let Some(entries) = timeline.get_mut("entries").and_then(Value::as_array_mut) else {
        return timeline;
    };
    for entry in entries {
        let fill = entry.get("kind").and_then(Value::as_str) == Some("action")
            && entry.get("action").and_then(Value::as_str) == Some("fill");
        let flagged = fill
            && entry
                .get_mut("text")
                .is_some_and(|text| redact_strings(text, &redact));
        redact_strings(entry, &redact);
        if !fill {
            continue;
        }
        if for_draft && flagged {
            entry["text"] = Value::Null;
            entry["secret"] = Value::Bool(true);
        } else if !for_draft && let Some(Value::String(text)) = entry.get_mut("text") {
            *text = pick::within(text, FILL_BUDGET);
        }
    }
    timeline
}

/// Passes each string in `value` through `redact` whole; whether any changed.
fn redact_strings(value: &mut Value, redact: &impl Fn(&str) -> String) -> bool {
    match value {
        Value::String(text) => {
            let redacted = redact(text);
            let changed = redacted != *text;
            *text = redacted;
            changed
        }
        Value::Array(items) => items.iter_mut().fold(false, |changed, item| {
            redact_strings(item, redact) | changed
        }),
        Value::Object(fields) => fields.values_mut().fold(false, |changed, item| {
            redact_strings(item, redact) | changed
        }),
        _ => false,
    }
}

/// The Playwright config in the project `root`, when it has one, and the `testDir` it sets as a
/// plain string inside the project.
fn configured_test_dir(root: &Path) -> Option<(PathBuf, Option<String>)> {
    PLAYWRIGHT_CONFIGS.iter().find_map(|name| {
        let path = root.join(name);
        let text = std::fs::read_to_string(&path).ok()?;
        Some((path, test_dir_in(&text)))
    })
}

/// The directory a Playwright config's `testDir: '<dir>'` names, when it is a plain relative
/// path inside the project.
fn test_dir_in(config: &str) -> Option<String> {
    let after = &config[config.find("testDir")? + "testDir".len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let quote = after
        .chars()
        .next()
        .filter(|quote| "'\"`".contains(*quote))?;
    let rest = &after[quote.len_utf8()..];
    let dir = &rest[..rest.find(quote)?];
    let dir = dir.trim_start_matches("./").trim_end_matches('/');
    let inside = !dir.is_empty()
        && !dir.starts_with('/')
        && Path::new(dir)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)));
    inside.then(|| dir.to_string())
}

/// `title` as a file name's start: lower case letters and digits, dashes between, at most 40
/// characters; `recording` when nothing is left.
fn slug(title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
        if slug.len() >= 40 {
            break;
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.is_empty() {
        "recording".to_string()
    } else {
        slug.to_string()
    }
}

/// `browser_picks`: each pick of the session, without its bundle.
fn picks(hub: &Entity<BrowserHub>, cx: &AsyncApp) -> ToolAnswer {
    let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
    let picks: Vec<Value> = hub.read_with(cx, |hub, _| {
        hub.picks()
            .iter()
            .map(|pick| {
                let pick = pick_for_agents(pick, redactor.as_deref());
                json!({
                    "id": pick.id,
                    "tab": pick.tab,
                    "url": pick.url,
                    "title": pick.title,
                    "summary": pick.summary,
                    "caption": pick.caption,
                    "sent": pick.sent,
                })
            })
            .collect()
    });
    ToolAnswer {
        structured: json!({ "picks": picks }),
        text: None,
        image: None,
    }
}

/// The pick id `arguments` give a pick tool, `tool`.
fn pick_id(arguments: &Value, tool: &str) -> Result<usize, String> {
    arguments
        .get("id")
        .and_then(Value::as_u64)
        .and_then(|id| usize::try_from(id).ok())
        .ok_or_else(|| format!("{tool} needs the pick's id, from browser_picks"))
}

/// `browser_check_pick` (#505): the pick's element found again in the pick's own tab, brought
/// forward as the write tools bring theirs, cropped and compared with the pick; what changed, the
/// element now and its crop as the image. The check stays on the pick for `browser_pick` and the
/// tray, whose row shows its verdict.
async fn check_pick(
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let id = pick_id(arguments, "browser_check_pick")?;
    let tab = hub
        .read_with(cx, |hub, _| hub.pick(id).map(|pick| pick.tab.clone()))
        .ok_or_else(|| format!("the user made no pick {id}; browser_picks lists them"))?;
    let (tab, _) = page_of(hub, Some(&tab), None, cx)
        .await
        .map_err(|_| format!("pick {id}'s tab is closed"))?;
    cx.update(|cx| show_for_agent(&tab, cx));
    let did = format!("checked pick {id}");
    hub.update(cx, |hub, cx| {
        hub.agent_started(&tab, format!("checking pick {id}"), by.cloned(), cx);
    });
    let checked = hub.update(cx, |hub, cx| hub.check_pick(id, cx)).await;
    let shown = if checked.is_ok() {
        did
    } else {
        format!("failed: {did}")
    };
    hub.update(cx, |hub, cx| {
        hub.agent_ended(&tab, SharedString::from(shown), by.cloned(), cx);
    });
    checked?;
    let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
    let check = hub
        .read_with(cx, |hub, _| {
            hub.pick(id)
                .map(|pick| pick_for_agents(pick, redactor.as_deref()))
        })
        .and_then(|pick| pick.check)
        .ok_or_else(|| format!("pick {id} went while it was checked"))?;
    let image = check.crop.clone().map(|data| ToolImage {
        mime_type: "image/jpeg".to_string(),
        data,
    });
    Ok(ToolAnswer {
        structured: json!({
            "id": id,
            "tab": tab,
            "found": check.found_by.is_some(),
            "found_by": check.found_by,
            "changes": check.changes,
            "bundle": check.bundle,
            "checked_at": check.checked_at,
        }),
        text: None,
        image,
    })
}

/// `browser_pick`: the pick `id` with its bundle, and its crop as the image.
fn pick(arguments: &Value, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let id = pick_id(arguments, "browser_pick")?;
    let redactor = cx.update(|cx| crate::mcp::agent_redactor(cx));
    let pick = hub
        .read_with(cx, |hub, _| {
            hub.pick(id)
                .map(|pick| pick_for_agents(pick, redactor.as_deref()))
        })
        .ok_or_else(|| format!("the user made no pick {id}; browser_picks lists them"))?;
    let image = pick.crop.clone().map(|data| ToolImage {
        mime_type: "image/jpeg".to_string(),
        data,
    });
    let structured = serde_json::to_value(&pick).map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured,
        text: None,
        image,
    })
}

/// The pick as an agent may read it (#518). Each text the page gave is redacted whole and only
/// then cut to its budget, so no rule misses a secret a cut split
/// (PR-claude-redact-the-whole-text-before-cutting-it-001); the summary is made again from what
/// is left, and a listener's script URL loses its secret-looking values. With redaction off, the
/// texts are only cut. A check's bundle goes the same way, and its change lines, which quote the
/// page's texts, are made again from the two redacted bundles before each is cut (#505).
fn pick_for_agents(pick: &Pick, redactor: Option<&Redactor>) -> Pick {
    let redact = |text: &str| {
        redactor.map_or_else(|| text.to_string(), |redactor| redactor.redact(text).text)
    };
    let mut pick = pick.clone();
    pick.title = redact(&pick.title);
    pick.caption = redact(&pick.caption);
    let before = redacted(&pick.bundle, &redact);
    if let Some(check) = pick.check.as_mut() {
        let after = check
            .bundle
            .as_ref()
            .map(|bundle| redacted(bundle, &redact));
        check.changes = after
            .as_ref()
            .map(|after| {
                pick::changes(&before, after)
                    .iter()
                    .map(|line| pick::within(line, pick::CHANGE_BUDGET))
                    .collect()
            })
            .unwrap_or_default();
        check.bundle = after.map(within_budgets);
    }
    pick.bundle = within_budgets(before);
    pick.summary = pick.bundle.summary();
    pick
}

/// `bundle` with each text the page gave redacted whole, nothing cut yet.
fn redacted(bundle: &PickBundle, redact: &impl Fn(&str) -> String) -> PickBundle {
    let mut bundle = bundle.clone();
    bundle.name = bundle.name.as_deref().map(redact);
    bundle.text = redact(&bundle.text);
    bundle.html = redact(&bundle.html);
    for text in &mut bundle.nearby_text {
        *text = redact(text);
    }
    bundle.selected_text = bundle.selected_text.as_deref().map(redact);
    for locator in &mut bundle.locators {
        locator.value = redact(&locator.value);
    }
    for blocker in &mut bundle.blockers {
        *blocker = redact(blocker);
    }
    for listener in &mut bundle.listeners {
        listener.on = redact(&listener.on);
        listener.script = listener.script.as_deref().map(redact_url);
    }
    bundle
}

/// `bundle`'s texts cut to what an agent gets of each (#518).
fn within_budgets(mut bundle: PickBundle) -> PickBundle {
    bundle.text = pick::within(&bundle.text, pick::TEXT_BUDGET);
    bundle.html = pick::within(&bundle.html, pick::HTML_BUDGET);
    for text in &mut bundle.nearby_text {
        *text = pick::within(text, pick::TEXT_BUDGET);
    }
    bundle.selected_text = bundle
        .selected_text
        .as_deref()
        .map(|text| pick::within(text, pick::SELECTION_BUDGET));
    bundle
}

/// What `browser_look` says of the page, beside its image.
#[derive(Serialize)]
struct Look {
    tab: String,
    url: String,
    title: String,
    loading: bool,
    viewport: marley_browser::page::Viewport,
    focused: Option<marley_browser::page::FocusedElement>,
    selection: String,
}

async fn take_snapshot(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let full = arguments
        .get("full")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Snapshot { text, refs, cut } = read_snapshot(page, tab, full, hub, cx).await?;
    let count = refs.len();
    hub.update(cx, |hub, _| hub.set_refs(tab, refs));
    Ok(ToolAnswer {
        structured: json!({ "tab": tab, "snapshot": text, "refs": count, "cut": cut }),
        text: Some(text),
        image: None,
    })
}

/// The page's snapshot: its main tree, the same-site frames that tree leaves out, and the
/// cross-site iframes, every node when `full`. `browser_snapshot` and `browser_find` (#567)
/// read the page the same way, so their refs are the same currency.
async fn read_snapshot(
    page: &Page,
    tab: &str,
    full: bool,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<Snapshot, String> {
    let iframes = hub.read_with(cx, |hub, _| hub.iframes(tab));
    let main = page
        .accessibility_tree(page.session_id(), None)
        .await
        .map_err(|error| error.to_string())?;
    // A frame whose document the main tree holds already is not read again.
    let covered: HashSet<String> = main
        .iter()
        .filter_map(|node| node.frame_id.clone())
        .collect();
    let mut trees = vec![FrameTree {
        session: None,
        frame_id: None,
        label: None,
        nodes: main,
    }];
    for frame in page.frames().await.map_err(|error| error.to_string())? {
        let cross_site = iframes.iter().any(|iframe| iframe.frame_id == frame.id);
        if frame.main || cross_site || covered.contains(&frame.id) {
            continue;
        }
        match page
            .accessibility_tree(page.session_id(), Some(&frame.id))
            .await
        {
            Ok(nodes) => trees.push(FrameTree {
                session: None,
                frame_id: None,
                label: Some(frame.url),
                nodes,
            }),
            Err(error) => log::debug!("browser: a frame's accessibility tree: {error}"),
        }
    }
    for iframe in &iframes {
        match page.accessibility_tree(&iframe.session, None).await {
            Ok(nodes) => trees.push(FrameTree {
                session: Some(iframe.session.clone()),
                frame_id: Some(iframe.frame_id.clone()),
                label: Some(iframe.url.clone()),
                nodes,
            }),
            Err(error) => log::debug!("browser: an iframe's accessibility tree: {error}"),
        }
    }
    Ok(snapshot::render(&trees, full))
}

/// `browser_find` (#567): the element of the page `query` names, from a snapshot taken now, whose
/// refs the hub keeps, so `browser_click` takes the answer. The query's words come first, then
/// the System One layer for what they leave open (`crate::find`).
async fn find_element(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let query = find_query(arguments)?;
    let full = arguments
        .get("full")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Snapshot { refs, .. } = read_snapshot(page, tab, full, hub, cx).await?;
    let items: Vec<String> = refs.iter().map(RefTarget::describe).collect();
    let listed: Vec<(String, String, String)> = refs
        .iter()
        .map(|target| (target.id.clone(), target.role.clone(), target.name.clone()))
        .collect();
    hub.update(cx, |hub, _| hub.set_refs(tab, refs));
    let place = hub.read_with(cx, |hub, _| {
        hub.project_of(tab).map_or_else(
            || crate::find::Place {
                project: "no project".to_string(),
                folders: Vec::new(),
                local: true,
            },
            |project| crate::find::Place {
                project: project.name.to_string(),
                folders: project.paths.clone(),
                local: project.host.is_none(),
            },
        )
    });
    let found = crate::find::find_items(
        "browser_find",
        &format!("{tab}:{query}"),
        &query,
        &items,
        place,
        cx,
    )
    .await;
    let element = |index: usize| listed.get(index);
    let candidates: Vec<Value> = found
        .candidates
        .iter()
        .filter_map(|(index, probability)| {
            let (id, role, name) = element(*index)?;
            Some(json!({ "ref": id, "role": role, "name": name, "probability": probability }))
        })
        .collect();
    let top = found.top.and_then(element).map(|(id, _, _)| id.clone());
    let lines: Vec<String> = found
        .candidates
        .iter()
        .filter_map(|(index, probability)| {
            let item = items.get(*index)?;
            let id = &element(*index)?.0;
            Some(probability.map_or_else(
                || format!("{id} {item}"),
                |probability| format!("{id} {item} ({probability:.2})"),
            ))
        })
        .collect();
    let text = found_text(&found, top.as_deref(), &lines, "browser_snapshot");
    Ok(ToolAnswer {
        structured: json!({
            "tab": tab,
            "query": query,
            "source": found.source,
            "sure": found.sure,
            "ref": top,
            "candidates": candidates,
            "present": found.present,
            "verify": found.verify,
            "next": (!found.sure).then_some("browser_snapshot"),
            "note": found.note,
        }),
        text: Some(text),
        image: None,
    })
}

/// A find's `query`: its text, trimmed, of 200 characters at most (#567).
pub(crate) fn find_query(arguments: &Value) -> Result<String, String> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .ok_or_else(|| "give `query`, what to find in words".to_string())?;
    if query.chars().count() > 200 {
        return Err("`query` holds at most 200 characters".to_string());
    }
    Ok(query.to_string())
}

/// A find's answer as text (#567): the item to act on, or why there is none, the candidates, and
/// where to read when the answer is not sure.
pub(crate) fn found_text(
    found: &crate::find::Found,
    top: Option<&str>,
    candidates: &[String],
    next: &str,
) -> String {
    let head = match top {
        Some(top) => format!("{top}, by {}", found.source),
        None if candidates.is_empty() => "Nothing found".to_string(),
        None => "Not sure".to_string(),
    };
    let present = found
        .present
        .map(|present| format!("; the model reads a match as {present}"))
        .unwrap_or_default();
    let verify = if found.verify {
        "; the model's suggestions, to look at first"
    } else {
        ""
    };
    let mut lines = vec![format!("{head}{present}{verify}")];
    lines.extend(candidates.iter().map(|candidate| format!("- {candidate}")));
    lines.extend(found.note.clone());
    if !found.sure {
        lines.push(format!("Read with {next} to be sure."));
    }
    lines.join("\n")
}

fn entries<T: Serialize>(tab: &str, entries: Vec<T>) -> Result<ToolAnswer, String> {
    let entries = serde_json::to_value(entries).map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured: json!({ "tab": tab, "entries": entries }),
        text: None,
        image: None,
    })
}

/// A write tool's answer: what it did, in which tab, and where the page is now.
fn done(did: &str, tab: &str, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> ToolAnswer {
    let (url, title) = hub.read_with(cx, |hub, _| (hub.url(tab), hub.title(tab)));
    ToolAnswer {
        structured: json!({
            "did": did,
            "tab": tab,
            "url": url.map(|url| redact_url(&url)).unwrap_or_default(),
            "title": title.map(|title| title.to_string()).unwrap_or_default(),
        }),
        text: None,
        image: None,
    }
}

/// Shows `doing` in the tab's Agent chip, runs `action`, and shows `did`, or that it failed; the
/// chip names the outside client `by` when there is one (#524).
async fn acting<T>(
    hub: &Entity<BrowserHub>,
    tab: &str,
    doing: String,
    did: String,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
    action: impl AsyncFnOnce(&mut AsyncApp) -> Result<T, String>,
) -> Result<ToolAnswer, String> {
    hub.update(cx, |hub, cx| hub.agent_started(tab, doing, by.cloned(), cx));
    let result = action(cx).await;
    let shown = if result.is_ok() {
        did.clone()
    } else {
        format!("failed: {did}")
    };
    hub.update(cx, |hub, cx| {
        hub.agent_ended(tab, SharedString::from(shown), by.cloned(), cx);
    });
    result.map(|_| done(&did, tab, hub, cx))
}

/// `browser_navigate`: in the tab named, else in the tab a call that names no tab acts on; in a
/// new tab with `new_tab`, or when the caller's project has no tab showing a page, or, for a
/// caller in no project, when the browser has none. A new tab for a caller's project opens in the
/// caller's workspace (#574), its page in that project's browser, which starts when it is not
/// running; a caller in no project gets a page of the project the active window shows (#507).
async fn navigate(
    arguments: &Value,
    named_tab: Option<&str>,
    scope: Option<&Scope>,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let url = arguments
        .get("url")
        .and_then(Value::as_str)
        .ok_or("give `url`, an http or https URL")?;
    let url = address::agent_url(url)?;
    let new_tab = arguments
        .get("new_tab")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_pages = cx.update(|cx| match scope {
        Some(_) => default_tab(hub, scope, cx).is_some(),
        None => hub.read(cx).has_pages(),
    });
    let tab = if new_tab || (named_tab.is_none() && !has_pages) {
        let home = scope.map(|scope| scope.home.clone());
        let project = cx
            .update(|cx| caller_project(scope, cx))
            .ok_or("Marley has no window to open a page in")?;
        hub.update(cx, |hub, cx| hub.browser_for(&project, cx));
        let created = new_page(hub, &project.key, "about:blank".to_string(), home, cx).await?;
        page_of(hub, Some(&created), None, cx).await?.0
    } else {
        page_of(hub, named_tab, scope, cx).await?.0
    };
    cx.update(|cx| show_for_agent(&tab, cx));
    let shown = redact_url(&url);
    let hub_for_action = hub.clone();
    let tab_for_action = tab.clone();
    acting(
        hub,
        &tab,
        format!("going to {shown}"),
        format!("went to {shown}"),
        by,
        cx,
        async move |cx| {
            let task =
                hub_for_action.update(cx, |hub, cx| hub.navigate_task(&tab_for_action, url, cx));
            task.await
        },
    )
    .await
}

async fn back(
    tab: &str,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let hub_for_action = hub.clone();
    let tab_for_action = tab.to_string();
    acting(
        hub,
        tab,
        "going back".to_string(),
        "went back".to_string(),
        by,
        cx,
        async move |cx| {
            let task = hub_for_action.update(cx, |hub, cx| hub.go_task(&tab_for_action, -1, cx));
            task.await
        },
    )
    .await
}

async fn click(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let button = match arguments
        .get("button")
        .and_then(Value::as_str)
        .unwrap_or("left")
    {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        other => return Err(format!("no button named {other}; left, right or middle")),
    };
    let count = arguments
        .get("count")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .unwrap_or(1)
        .clamp(1, 3);
    let (point, what) = target_point(page, tab, arguments, hub, cx).await?;
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("clicking {what}"),
        format!("clicked {what}"),
        by,
        cx,
        async move |_| {
            click_at(&page, point, button, count)
                .await
                .map_err(|error| error.to_string())
        },
    )
    .await
}

async fn type_text(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let text = arguments
        .get("text")
        .and_then(Value::as_str)
        .ok_or("give `text` to type")?
        .to_string();
    let submit = arguments
        .get("submit")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let into = if arguments.get("ref").is_some() {
        let (point, what) = target_point(page, tab, arguments, hub, cx).await?;
        click_at(page, point, MouseButton::Left, 1)
            .await
            .map_err(|error| error.to_string())?;
        format!("into {what}")
    } else {
        "where the focus is".to_string()
    };
    // The chip counts what was typed and never shows it: it may be a password.
    let count = text.chars().count();
    let then = if submit { " and pressed Enter" } else { "" };
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("typing {into}"),
        format!("typed {count} characters {into}{then}"),
        by,
        cx,
        async move |_| {
            for character in text.chars() {
                send_press(&page, input::char_press(character)).await?;
            }
            if submit {
                send_press(&page, input::char_press('\n')).await?;
            }
            Ok(())
        },
    )
    .await
}

async fn press(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let key = arguments
        .get("key")
        .and_then(Value::as_str)
        .ok_or("give `key`, such as Enter or Ctrl+A")?;
    let key_press = input::chord(key)?;
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("pressing {key}"),
        format!("pressed {key}"),
        by,
        cx,
        async move |_| send_press(&page, key_press).await,
    )
    .await
}

async fn scroll(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    if let Some(reference) = arguments.get("ref").and_then(Value::as_str) {
        let target = ref_target(tab, reference, hub, cx)?;
        let what = target.describe();
        let page = page.clone();
        let hub_for_action = hub.clone();
        let tab_for_action = tab.to_string();
        return acting(
            hub,
            tab,
            format!("scrolling {what} into view"),
            format!("scrolled {what} into view"),
            by,
            cx,
            async move |cx| {
                place(&page, &tab_for_action, &target, &hub_for_action, cx)
                    .await
                    .map(drop)
            },
        )
        .await;
    }
    let delta_y = arguments.get("dy").and_then(Value::as_f64).unwrap_or(0.0);
    let delta_x = arguments.get("dx").and_then(Value::as_f64).unwrap_or(0.0);
    if delta_x == 0.0 && delta_y == 0.0 {
        return Err("give `dy` or `dx` in pixels, or `ref`".to_string());
    }
    let viewport = page.viewport().await.map_err(|error| error.to_string())?;
    let center = (viewport.width / 2.0, viewport.height / 2.0);
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("scrolling {}", scroll_words(delta_x, delta_y)),
        format!("scrolled {}", scroll_words(delta_x, delta_y)),
        by,
        cx,
        async move |cx| {
            page.call(
                "Input.dispatchMouseEvent",
                input::wheel_event(center, (delta_x, delta_y), 0),
            )
            .await
            .map_err(|error| error.to_string())?;
            cx.background_executor().timer(SCROLL_SETTLE).await;
            Ok(())
        },
    )
    .await
}

/// A scroll as the chip says it: `400 pixels down`.
fn scroll_words(delta_x: f64, delta_y: f64) -> String {
    let part = |amount: f64, forward: &str, backward: &str| {
        let direction = if amount > 0.0 { forward } else { backward };
        format!("{:.0} pixels {direction}", amount.abs())
    };
    match (delta_x != 0.0, delta_y != 0.0) {
        (true, true) => format!(
            "{} and {}",
            part(delta_y, "down", "up"),
            part(delta_x, "right", "left")
        ),
        (true, false) => part(delta_x, "right", "left"),
        _ => part(delta_y, "down", "up"),
    }
}

fn ref_target(
    tab: &str,
    reference: &str,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<RefTarget, String> {
    hub.read_with(cx, |hub, _| hub.ref_target(tab, reference))
        .ok_or_else(|| format!("no {reference} in the tab's newest snapshot; take a new one"))
}

/// Where a click goes, in the page's viewport, and what the chip calls it: a ref's element,
/// scrolled into view, or the point `x`, `y`.
async fn target_point(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<((f64, f64), String), String> {
    if let Some(reference) = arguments.get("ref").and_then(Value::as_str) {
        let target = ref_target(tab, reference, hub, cx)?;
        let point = place(page, tab, &target, hub, cx).await?;
        return Ok((point, target.describe()));
    }
    match (
        arguments.get("x").and_then(Value::as_f64),
        arguments.get("y").and_then(Value::as_f64),
    ) {
        (Some(x), Some(y)) => Ok(((x, y), format!("the point {x:.0}, {y:.0}"))),
        _ => Err("give `ref` from browser_snapshot, or `x` and `y`".to_string()),
    }
}

/// Scrolls a ref's element into view and gives its middle in the page's viewport: an element in
/// a cross-site iframe is placed through the iframe's owner element.
async fn place(
    page: &Page,
    tab: &str,
    target: &RefTarget,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<(f64, f64), String> {
    let (session, offset) = ref_origin(page, tab, target, hub, cx).await?;
    let (x, y) = page
        .box_center(&session, target.backend_node_id)
        .await
        .map_err(|_| gone(target))?;
    Ok((x + offset.0, y + offset.1))
}

/// Scrolls a ref's element into view and gives its border box in the page's coordinates (#498).
async fn element_box(
    page: &Page,
    tab: &str,
    target: &RefTarget,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<PageBox, String> {
    let (session, offset) = ref_origin(page, tab, target, hub, cx).await?;
    let (left, top, width, height) = page
        .border_box(&session, target.backend_node_id)
        .await
        .map_err(|_| gone(target))?;
    let viewport = page.viewport().await.map_err(|error| error.to_string())?;
    Ok(PageBox {
        x: left + offset.0 + viewport.scroll_x,
        y: top + offset.1 + viewport.scroll_y,
        width,
        height,
    })
}

/// What a call says of a ref whose element went.
fn gone(target: &RefTarget) -> String {
    format!(
        "{} is no longer on the page, or has no box; take a new snapshot",
        target.id
    )
}

/// A ref's element scrolled into view: the session that holds it, and where its frame's viewport
/// sits in the page's, through a cross-site iframe's owner element.
async fn ref_origin(
    page: &Page,
    tab: &str,
    target: &RefTarget,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<(String, (f64, f64)), String> {
    let session = target
        .session
        .clone()
        .unwrap_or_else(|| page.session_id().to_string());
    page.scroll_into_view(&session, target.backend_node_id)
        .await
        .map_err(|_| gone(target))?;
    let offset = match &target.frame_id {
        Some(frame) => {
            let nested = hub.read_with(cx, |hub, _| {
                hub.iframes(tab).iter().any(|iframe| {
                    iframe.frame_id == *frame && iframe.parent_session != page.session_id()
                })
            });
            if nested {
                return Err(
                    "the element is in a cross-site iframe inside another; Marley cannot place \
                     it yet"
                        .to_string(),
                );
            }
            page.frame_origin(frame)
                .await
                .map_err(|error| error.to_string())?
        }
        None => (0.0, 0.0),
    };
    Ok((session, offset))
}

/// `browser_annotations`: the tab's annotations, with their boxes in page coordinates (#498).
fn annotations(tab: &str, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> ToolAnswer {
    let annotations = hub.read_with(cx, |hub, _| hub.annotations(tab).to_vec());
    ToolAnswer {
        structured: json!({ "tab": tab, "annotations": annotations }),
        text: None,
        image: None,
    }
}

/// `browser_annotate`: a box of the agent's, with a note, around a ref's element or over an area
/// of the viewport; or, with `clear`, the agent's own boxes gone (#498).
async fn annotate(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    if arguments.get("clear").and_then(Value::as_bool) == Some(true) {
        let removed = hub.update(cx, |hub, cx| hub.clear_agent_annotations(tab, cx));
        let did = format!("cleared {removed} annotations");
        hub.update(cx, |hub, cx| {
            hub.agent_ended(tab, SharedString::from(did.clone()), by.cloned(), cx);
        });
        return Ok(done(&did, tab, hub, cx));
    }
    let note = arguments
        .get("note")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let (page_box, what) = if let Some(reference) = arguments.get("ref").and_then(Value::as_str) {
        let target = ref_target(tab, reference, hub, cx)?;
        (
            element_box(page, tab, &target, hub, cx).await?,
            target.describe(),
        )
    } else {
        let number = |name: &str| arguments.get(name).and_then(Value::as_f64);
        let (Some(x), Some(y), Some(width), Some(height)) =
            (number("x"), number("y"), number("width"), number("height"))
        else {
            return Err(
                "give `ref` from browser_snapshot, or `x`, `y`, `width` and `height`".to_string(),
            );
        };
        let viewport = page.viewport().await.map_err(|error| error.to_string())?;
        (
            PageBox {
                x: x + viewport.scroll_x,
                y: y + viewport.scroll_y,
                width,
                height,
            },
            format!("the area at {x:.0}, {y:.0}"),
        )
    };
    let id = hub
        .update(cx, |hub, cx| {
            hub.add_annotation(tab, page_box, note, Maker::Agent, cx)
        })
        .ok_or_else(|| format!("the browser has no tab {tab}"))?;
    let did = format!("annotated {what}");
    hub.update(cx, |hub, cx| {
        hub.agent_ended(tab, SharedString::from(did.clone()), by.cloned(), cx);
    });
    let mut answer = done(&did, tab, hub, cx);
    if let Some(structured) = answer.structured.as_object_mut() {
        structured.extend([
            ("id".to_string(), json!(id)),
            ("box".to_string(), json!(page_box)),
        ]);
    }
    Ok(answer)
}

/// A click at `point`, as the user's mouse makes one: the pointer moves there, then each press
/// and release, `count` times.
async fn click_at(
    page: &Page,
    point: (f64, f64),
    button: MouseButton,
    count: usize,
) -> Result<(), CdpError> {
    let (name, bit) = input::mouse_button(button);
    page.call(
        "Input.dispatchMouseEvent",
        input::mouse_event("mouseMoved", point, "none", 0, 0, 0),
    )
    .await?;
    for click in 1..=count {
        page.call(
            "Input.dispatchMouseEvent",
            input::mouse_event("mousePressed", point, name, bit, click, 0),
        )
        .await?;
        page.call(
            "Input.dispatchMouseEvent",
            input::mouse_event("mouseReleased", point, name, 0, click, 0),
        )
        .await?;
    }
    Ok(())
}

async fn send_press(page: &Page, key_press: KeyPress) -> Result<(), String> {
    page.call("Input.dispatchKeyEvent", key_press.down)
        .await
        .map_err(|error| error.to_string())?;
    page.call("Input.dispatchKeyEvent", key_press.up)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
