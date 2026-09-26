//! The browser family of Marley's MCP server (#492, #493): the pages in the Browser tabs, seen
//! and driven by agents.
//!
//! Each call answers from a task of its own, since each waits on the browser, and acts on the
//! page its `tab` names, a page's id from `browser_tabs`, or else on the page whose tab the user
//! focused last. A read tool reads the page as the user sees it. A write tool first brings the
//! page's tab to the front where that leaves the focus alone, so the user watches, says what it
//! does in the tab's Agent chip, and sends the same CDP events the user's keys and mouse send
//! (`marley_browser::input`), so the page cannot tell them apart.

use std::collections::HashSet;
use std::time::Duration;

use base64::Engine as _;

use gpui::{App, AppContext as _, AsyncApp, Entity, MouseButton};
use marley_browser::address;
use marley_browser::cdp::CdpError;
use marley_browser::input::{self, KeyPress};
use marley_browser::observe::redact_url;
use marley_browser::page::Page;
use marley_browser::pick::PageBox;
use marley_browser::recorder;
use marley_browser::snapshot::{self, FrameTree, RefTarget, Snapshot};
use marley_mcp::{AppCall, ToolAnswer, ToolImage};
use serde::Serialize;
use serde_json::{Value, json};
use ui::SharedString;

use crate::browser::{BrowserHub, Maker, new_page, recordings_dir, show_for_agent, showing};

/// How long a call waits, once the browser shows its pages, for the page it acts on to attach.
const ATTACH_WAIT: Duration = Duration::from_secs(5);

/// How often it looks.
const POLL: Duration = Duration::from_millis(100);

/// How long a scroll is given to land before its call answers.
const SCROLL_SETTLE: Duration = Duration::from_millis(300);

/// The tools that act in a page they find, which bring its tab to the front first.
const WRITES: &[&str] = &[
    "browser_annotate",
    "browser_back",
    "browser_click",
    "browser_type",
    "browser_press",
    "browser_scroll",
];

/// Answers `call`, a `browser_*` tool, from a task of its own. A browser that stopped starts
/// again, as `marley: open browser` starts it.
pub fn answer(call: AppCall, cx: &mut App) {
    let hub = BrowserHub::global(cx);
    hub.update(cx, BrowserHub::start_if_failed);
    cx.spawn(async move |cx| {
        let result = run(&call.tool, &call.arguments, &hub, cx).await;
        call.answer(result);
    })
    .detach();
}

async fn run(
    tool: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    // Picks are Marley's, kept while the browser restarts, and recordings are files.
    match tool {
        "browser_picks" => return Ok(picks(hub, cx)),
        "browser_pick" => return pick(arguments, hub, cx),
        "browser_recordings" => return recordings(cx).await,
        "browser_recording" => return recording(arguments, cx).await,
        _ => {}
    }
    showing(hub, cx).await?;
    let named_tab = arguments.get("tab").and_then(Value::as_str);
    match tool {
        "browser_tabs" => return tabs(hub, cx).await,
        "browser_navigate" => return navigate(arguments, named_tab, hub, cx).await,
        _ => {}
    }
    let (tab, page) = page_of(hub, named_tab, cx).await?;
    if WRITES.contains(&tool) {
        cx.update(|cx| show_for_agent(&tab, cx));
    }
    match tool {
        "browser_look" => look(&page, &tab, hub, cx).await,
        "browser_snapshot" => take_snapshot(&page, &tab, arguments, hub, cx).await,
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
        "browser_annotate" => annotate(&page, &tab, arguments, hub, cx).await,
        "browser_back" => back(&tab, hub, cx).await,
        "browser_click" => click(&page, &tab, arguments, hub, cx).await,
        "browser_type" => type_text(&page, &tab, arguments, hub, cx).await,
        "browser_press" => press(&page, &tab, arguments, hub, cx).await,
        "browser_scroll" => scroll(&page, &tab, arguments, hub, cx).await,
        other => Err(format!("Marley answers no tool named {other}")),
    }
}

/// The page `named_tab` names, or the page whose tab the user focused last, with its id, once
/// it is attached.
async fn page_of(
    hub: &Entity<BrowserHub>,
    named_tab: Option<&str>,
    cx: &AsyncApp,
) -> Result<(String, Page), String> {
    let mut waited = Duration::ZERO;
    loop {
        let (found, has_pages) = hub.read_with(cx, |hub, _| {
            let tab = named_tab.map(str::to_string).or_else(|| hub.focused());
            let found = tab.and_then(|tab| hub.page(&tab).map(|page| (tab, page)));
            (found, hub.has_pages())
        });
        if let Some(found) = found {
            return Ok(found);
        }
        // A page just made attaches in a moment; with none at all there is nothing to wait for.
        if (named_tab.is_none() && !has_pages) || waited >= ATTACH_WAIT {
            return Err(named_tab.map_or_else(
                || "the browser has no tabs; browser_navigate opens one".to_string(),
                |tab| format!("the browser has no tab {tab}; browser_tabs lists them"),
            ));
        }
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
}

/// `browser_tabs`: every page, once the pages being attached are.
async fn tabs(hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let mut waited = Duration::ZERO;
    while hub.read_with(cx, |hub, _| hub.is_attaching()) && waited < ATTACH_WAIT {
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
    let tabs: Vec<_> = hub
        .read_with(cx, |hub, _| hub.tabs())
        .into_iter()
        .map(|mut tab| {
            tab.url = redact_url(&tab.url);
            tab
        })
        .collect();
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
    let (timeline, jpeg) = cx
        .background_spawn(futures::future::lazy(move |_| {
            let timeline = recorder::read_in(&dir, &id)?;
            let jpeg = frame
                .map(|frame| recorder::frame_in(&dir, &id, frame))
                .transpose()?;
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

/// `browser_picks`: each pick of the session, without its bundle.
fn picks(hub: &Entity<BrowserHub>, cx: &AsyncApp) -> ToolAnswer {
    let picks: Vec<Value> = hub.read_with(cx, |hub, _| {
        hub.picks()
            .iter()
            .map(|pick| {
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

/// `browser_pick`: the pick `id` with its bundle, and its crop as the image.
fn pick(arguments: &Value, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let id = arguments
        .get("id")
        .and_then(Value::as_u64)
        .and_then(|id| usize::try_from(id).ok())
        .ok_or_else(|| "browser_pick needs the pick's id, from browser_picks".to_string())?;
    let pick = hub
        .read_with(cx, |hub, _| hub.pick(id).cloned())
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
    let Snapshot { text, refs, cut } = snapshot::render(&trees, full);
    let count = refs.len();
    hub.update(cx, |hub, _| hub.set_refs(tab, refs));
    Ok(ToolAnswer {
        structured: json!({ "tab": tab, "snapshot": text, "refs": count, "cut": cut }),
        text: Some(text),
        image: None,
    })
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

/// Shows `doing` in the tab's Agent chip, runs `action`, and shows `did`, or that it failed.
async fn acting<T>(
    hub: &Entity<BrowserHub>,
    tab: &str,
    doing: String,
    did: String,
    cx: &mut AsyncApp,
    action: impl AsyncFnOnce(&mut AsyncApp) -> Result<T, String>,
) -> Result<ToolAnswer, String> {
    hub.update(cx, |hub, cx| hub.agent_started(tab, doing, cx));
    let result = action(cx).await;
    let shown = if result.is_ok() {
        did.clone()
    } else {
        format!("failed: {did}")
    };
    hub.update(cx, |hub, cx| {
        hub.agent_ended(tab, SharedString::from(shown), cx);
    });
    result.map(|_| done(&did, tab, hub, cx))
}

/// `browser_navigate`: in the tab named or focused last, or in a new tab with `new_tab` or when
/// the browser has none.
async fn navigate(
    arguments: &Value,
    named_tab: Option<&str>,
    hub: &Entity<BrowserHub>,
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
    let has_pages = hub.read_with(cx, |hub, _| hub.has_pages());
    let tab = if new_tab || (named_tab.is_none() && !has_pages) {
        let created = new_page(hub, "about:blank".to_string(), cx).await?;
        page_of(hub, Some(&created), cx).await?.0
    } else {
        page_of(hub, named_tab, cx).await?.0
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
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let hub_for_action = hub.clone();
    let tab_for_action = tab.to_string();
    acting(
        hub,
        tab,
        "going back".to_string(),
        "went back".to_string(),
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
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    if arguments.get("clear").and_then(Value::as_bool) == Some(true) {
        let removed = hub.update(cx, |hub, cx| hub.clear_agent_annotations(tab, cx));
        let did = format!("cleared {removed} annotations");
        hub.update(cx, |hub, cx| {
            hub.agent_ended(tab, SharedString::from(did.clone()), cx);
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
        hub.agent_ended(tab, SharedString::from(did.clone()), cx);
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
