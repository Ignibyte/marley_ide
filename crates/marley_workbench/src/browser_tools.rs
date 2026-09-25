//! The browser family of Marley's MCP server (#492): the page in the Browser tab, seen and driven
//! by agents.
//!
//! Each call answers from a task of its own, since each waits on the browser. A read tool reads
//! the page the user sees. A write tool first brings the Browser tab to the front, so the user
//! watches, says what it does in the tab's Agent chip, and sends the same CDP events the user's
//! keys and mouse send (`marley_browser::input`), so the page cannot tell them apart.

use std::collections::HashSet;
use std::time::Duration;

use gpui::{App, AsyncApp, Entity, MouseButton};
use marley_browser::address;
use marley_browser::cdp::CdpError;
use marley_browser::input::{self, KeyPress};
use marley_browser::observe::redact_url;
use marley_browser::page::Page;
use marley_browser::snapshot::{self, FrameTree, RefTarget, Snapshot};
use marley_mcp::{AppCall, ToolAnswer, ToolImage};
use serde::Serialize;
use serde_json::{Value, json};
use ui::SharedString;

use crate::browser::{BrowserHub, HubState, show_for_agent};

/// How long a call waits for the browser to show its page.
const PAGE_WAIT: Duration = Duration::from_secs(20);

/// How often it looks.
const POLL: Duration = Duration::from_millis(100);

/// How long a scroll is given to land before its call answers.
const SCROLL_SETTLE: Duration = Duration::from_millis(300);

/// The tools that act in the page, which bring the Browser tab to the front.
const WRITES: &[&str] = &[
    "browser_navigate",
    "browser_back",
    "browser_click",
    "browser_type",
    "browser_press",
    "browser_scroll",
];

/// Answers `call`, a `browser_*` tool, from a task of its own.
pub fn answer(call: AppCall, cx: &mut App) {
    let hub = BrowserHub::global(cx);
    if WRITES.contains(&call.tool.as_str()) {
        show_for_agent(cx);
    }
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
    let page = page_of(hub, cx).await?;
    match tool {
        "browser_look" => look(&page, hub, cx).await,
        "browser_snapshot" => take_snapshot(&page, arguments, hub, cx).await,
        "browser_console" => entries(hub.read_with(cx, |hub, _| hub.console_entries())),
        "browser_network" => entries(hub.read_with(cx, |hub, _| hub.network_entries())),
        "browser_navigate" => navigate(arguments, hub, cx).await,
        "browser_back" => back(hub, cx).await,
        "browser_click" => click(&page, arguments, hub, cx).await,
        "browser_type" => type_text(&page, arguments, hub, cx).await,
        "browser_press" => press(&page, arguments, hub, cx).await,
        "browser_scroll" => scroll(&page, arguments, hub, cx).await,
        other => Err(format!("Marley answers no tool named {other}")),
    }
}

/// The page, once the hub shows it; the reason, if the browser could not start.
async fn page_of(hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<Page, String> {
    let mut waited = Duration::ZERO;
    loop {
        let (state, page) = hub.read_with(cx, |hub, _| (hub.state().clone(), hub.page()));
        match (state, page) {
            (HubState::Showing, Some(page)) => return Ok(page),
            (HubState::Failed(reason), _) => return Err(reason.to_string()),
            _ if waited >= PAGE_WAIT => {
                return Err("the browser did not show its page within 20 seconds".to_string());
            }
            _ => {}
        }
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
}

async fn look(page: &Page, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<ToolAnswer, String> {
    let (url, title, loading) =
        hub.read_with(cx, |hub, _| (hub.url(), hub.title(), hub.is_loading()));
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

/// What `browser_look` says of the page, beside its image.
#[derive(Serialize)]
struct Look {
    url: String,
    title: String,
    loading: bool,
    viewport: marley_browser::page::Viewport,
    focused: Option<marley_browser::page::FocusedElement>,
    selection: String,
}

async fn take_snapshot(
    page: &Page,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let full = arguments
        .get("full")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let iframes = hub.read_with(cx, |hub, _| hub.iframes());
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
    hub.update(cx, |hub, _| hub.set_refs(refs));
    Ok(ToolAnswer {
        structured: json!({ "snapshot": text, "refs": count, "cut": cut }),
        text: Some(text),
        image: None,
    })
}

fn entries<T: Serialize>(entries: Vec<T>) -> Result<ToolAnswer, String> {
    let entries = serde_json::to_value(entries).map_err(|error| error.to_string())?;
    Ok(ToolAnswer {
        structured: json!({ "entries": entries }),
        text: None,
        image: None,
    })
}

/// A write tool's answer: what it did, and where the page is now.
fn done(did: &str, hub: &Entity<BrowserHub>, cx: &AsyncApp) -> ToolAnswer {
    let (url, title) = hub.read_with(cx, |hub, _| (hub.url(), hub.title()));
    ToolAnswer {
        structured: json!({
            "did": did,
            "url": url.map(|url| redact_url(&url)).unwrap_or_default(),
            "title": title.map(|title| title.to_string()).unwrap_or_default(),
        }),
        text: None,
        image: None,
    }
}

/// Shows `doing` in the Agent chip, runs `action`, and shows `did`, or that it failed.
async fn acting<T>(
    hub: &Entity<BrowserHub>,
    doing: String,
    did: String,
    cx: &mut AsyncApp,
    action: impl AsyncFnOnce(&mut AsyncApp) -> Result<T, String>,
) -> Result<ToolAnswer, String> {
    hub.update(cx, |hub, cx| hub.agent_started(doing, cx));
    let result = action(cx).await;
    let shown = if result.is_ok() {
        did.clone()
    } else {
        format!("failed: {did}")
    };
    hub.update(cx, |hub, cx| hub.agent_ended(SharedString::from(shown), cx));
    result.map(|_| done(&did, hub, cx))
}

async fn navigate(
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    let url = arguments
        .get("url")
        .and_then(Value::as_str)
        .ok_or("give `url`, an http or https URL")?;
    let url = address::agent_url(url)?;
    let shown = redact_url(&url);
    let hub_for_action = hub.clone();
    acting(
        hub,
        format!("going to {shown}"),
        format!("went to {shown}"),
        cx,
        async move |cx| {
            let task = hub_for_action.update(cx, |hub, cx| hub.navigate_task(url, cx));
            task.await
        },
    )
    .await
}

async fn back(hub: &Entity<BrowserHub>, cx: &mut AsyncApp) -> Result<ToolAnswer, String> {
    let hub_for_action = hub.clone();
    acting(
        hub,
        "going back".to_string(),
        "went back".to_string(),
        cx,
        async move |cx| {
            let task = hub_for_action.update(cx, |hub, cx| hub.go_task(-1, cx));
            task.await
        },
    )
    .await
}

async fn click(
    page: &Page,
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
    let (point, what) = target_point(page, arguments, hub, cx).await?;
    let page = page.clone();
    acting(
        hub,
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
        let (point, what) = target_point(page, arguments, hub, cx).await?;
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
        format!("pressing {key}"),
        format!("pressed {key}"),
        cx,
        async move |_| send_press(&page, key_press).await,
    )
    .await
}

async fn scroll(
    page: &Page,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, String> {
    if let Some(reference) = arguments.get("ref").and_then(Value::as_str) {
        let target = ref_target(reference, hub, cx)?;
        let what = target.describe();
        let page = page.clone();
        let hub_for_action = hub.clone();
        return acting(
            hub,
            format!("scrolling {what} into view"),
            format!("scrolled {what} into view"),
            cx,
            async move |cx| place(&page, &target, &hub_for_action, cx).await.map(drop),
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
    reference: &str,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<RefTarget, String> {
    hub.read_with(cx, |hub, _| hub.ref_target(reference))
        .ok_or_else(|| format!("no {reference} in the newest snapshot; take a new one"))
}

/// Where a click goes, in the page's viewport, and what the chip calls it: a ref's element,
/// scrolled into view, or the point `x`, `y`.
async fn target_point(
    page: &Page,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<((f64, f64), String), String> {
    if let Some(reference) = arguments.get("ref").and_then(Value::as_str) {
        let target = ref_target(reference, hub, cx)?;
        let point = place(page, &target, hub, cx).await?;
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
    target: &RefTarget,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<(f64, f64), String> {
    let gone = |_: CdpError| {
        format!(
            "{} is no longer on the page, or has no box; take a new snapshot",
            target.id
        )
    };
    let session = target
        .session
        .clone()
        .unwrap_or_else(|| page.session_id().to_string());
    page.scroll_into_view(&session, target.backend_node_id)
        .await
        .map_err(gone)?;
    let offset = match &target.frame_id {
        Some(frame) => {
            let nested = hub.read_with(cx, |hub, _| {
                hub.iframes().iter().any(|iframe| {
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
    let (x, y) = page
        .box_center(&session, target.backend_node_id)
        .await
        .map_err(gone)?;
    Ok((x + offset.0, y + offset.1))
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
