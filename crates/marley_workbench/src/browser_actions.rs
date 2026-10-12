//! The browser tools #756 added, seven that act and one that waits.
//!
//! They hover, choose in a select list, drag, upload a file, answer a dialog, lay the page out as
//! a device and force its colour scheme; `browser_wait` waits for the page.
//!
//! The acting ones go through `browser_tools::acting`, so each shows in the tab's Agent chip,
//! brings the tab forward and is refused while a click is paused there, as the older write tools
//! are. Their refusals carry codes (`no_option`, `not_a_file_input`, `no_dialog`, `not_met`,
//! `outside_projects`, `no_file`, `secret_file`). A select list's options are set through a fixed
//! function on the node, so Marley's own list stays shut as an agent's click keeps it (#495). An
//! upload takes files from the open projects' folders only, never one a secret glob matches.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::channel::oneshot;
use gpui::{App, AppContext as _, AsyncApp, Entity};
use marley_browser::devices::{DEVICES, device_named};
use marley_browser::input;
use marley_browser::page::Page;
use marley_mcp::{Refusal, ToolAnswer};
use serde_json::{Value, json};
use ui::SharedString;
use util::ResultExt as _;

use crate::browser::{BrowserHub, Emulated, Scheme};
use crate::browser_tools::{acting, done, ref_origin, ref_target, target_point};

/// How many moves a drag makes between its press and its release.
const DRAG_STEPS: u32 = 10;

/// How long a drag waits after its last move for Chromium to say it intercepted an HTML drag.
const DRAG_SETTLE: Duration = Duration::from_millis(300);

/// How often a wait looks at the page.
const WAIT_STEP: Duration = Duration::from_millis(250);

/// How long a wait looks when the call names no time, and the most it may name: within the
/// server's 30 seconds (L-claude-682).
const WAIT_DEFAULT_MS: u64 = 10_000;
const WAIT_MOST_MS: u64 = 20_000;

/// Answers `tool` when it is one of #756's, else `None`.
pub(crate) async fn run(
    tool: &str,
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Option<Result<ToolAnswer, Refusal>> {
    let answer = match tool {
        "browser_hover" => hover(page, tab, arguments, hub, by, cx).await,
        "browser_select" => select(page, tab, arguments, hub, by, cx).await,
        "browser_drag" => drag(page, tab, arguments, hub, by, cx).await,
        "browser_upload" => upload(page, tab, arguments, hub, by, cx).await,
        "browser_dialog" => dialog(tab, arguments, hub, by, cx),
        "browser_resize" => resize(tab, arguments, hub, by, cx).await,
        "browser_color_scheme" => color_scheme(tab, arguments, hub, by, cx).await,
        "browser_wait" => wait(page, tab, arguments, hub, cx).await,
        _ => return None,
    };
    Some(answer)
}

fn bad_argument(reason: impl Into<String>) -> Refusal {
    Refusal::new("bad_argument", reason.into())
}

/// The strings of the array argument `name`, when it is given.
fn strings(arguments: &Value, name: &str) -> Option<Vec<String>> {
    arguments.get(name).and_then(Value::as_array).map(|items| {
        items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    })
}

/// One mouse event, as the user's mouse sends it.
async fn mouse(
    page: &Page,
    kind: &str,
    point: (f64, f64),
    button: &str,
    buttons: u32,
) -> Result<(), String> {
    page.call(
        "Input.dispatchMouseEvent",
        input::mouse_event(kind, point, button, buttons, 0, 0),
    )
    .await
    .map(drop)
    .map_err(|error| error.to_string())
}

/// `browser_hover`: the pointer onto a ref's element or a point, left there.
async fn hover(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let (point, what) = target_point(page, tab, arguments, hub, cx).await?;
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("hovering over {what}"),
        format!("hovered over {what}"),
        by,
        cx,
        async move |_| mouse(&page, "mouseMoved", point, "none", 0).await,
    )
    .await
    .map_err(Refusal::from)
}

/// `browser_select`: options of a ref's select list, by value or by label.
async fn select(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let reference = arguments
        .get("ref")
        .and_then(Value::as_str)
        .ok_or_else(|| bad_argument("give `ref`, the select list's ref from browser_snapshot"))?;
    let (by_labels, wanted) = match (strings(arguments, "values"), strings(arguments, "labels")) {
        (Some(values), None) if !values.is_empty() => (false, values),
        (None, Some(labels)) if !labels.is_empty() => (true, labels),
        _ => {
            return Err(bad_argument(
                "give `values` or `labels`, one of them, with an option at least",
            ));
        }
    };
    let target = ref_target(tab, reference, hub, cx)?;
    let (session, _) = ref_origin(page, tab, &target, hub, cx).await?;
    let what = target.describe();
    let chosen = wanted.join(", ");
    let page = page.clone();
    let node = target.backend_node_id;
    let described = what.clone();
    acting(
        hub,
        tab,
        format!("choosing {chosen} in {what}"),
        format!("chose {chosen} in {what}"),
        by,
        cx,
        async move |_| {
            let answer = page
                .select_options(&session, node, by_labels, &wanted)
                .await
                .map_err(|error| Refusal::from(error.to_string()))?;
            match answer.get("error").and_then(Value::as_str) {
                None => Ok(()),
                Some("no_option") => {
                    let missing = strings(&answer, "missing").unwrap_or_default().join(", ");
                    let options: Vec<String> = answer
                        .get("options")
                        .and_then(Value::as_array)
                        .map(|options| {
                            options
                                .iter()
                                .map(|option| {
                                    format!(
                                        "{} ({})",
                                        option["label"].as_str().unwrap_or_default(),
                                        option["value"].as_str().unwrap_or_default()
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Err(Refusal::new(
                        "no_option",
                        format!(
                            "{described} has no option {missing}; its options are {}",
                            options.join(", ")
                        ),
                    ))
                }
                Some("not_a_select") => Err(Refusal::new(
                    "not_a_select",
                    format!("{described} is not a select list"),
                )),
                Some("not_multiple") => Err(Refusal::new(
                    "not_multiple",
                    format!("{described} takes one option"),
                )),
                Some(other) => Err(Refusal::from(other.to_string())),
            }
        },
    )
    .await
}

/// One end of a drag: a ref's element by `<end>_ref`, or the point `<end>_x`, `<end>_y`.
async fn drag_end(
    end: &str,
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<((f64, f64), String), Refusal> {
    let reference = arguments.get(format!("{end}_ref"));
    let x = arguments.get(format!("{end}_x"));
    let y = arguments.get(format!("{end}_y"));
    let named = match (reference, x, y) {
        (Some(reference), _, _) => json!({ "ref": reference }),
        (None, Some(x), Some(y)) => json!({ "x": x, "y": y }),
        _ => {
            return Err(bad_argument(format!(
                "give `{end}_ref`, or `{end}_x` and `{end}_y`"
            )));
        }
    };
    Ok(target_point(page, tab, &named, hub, cx).await?)
}

/// `browser_drag`: press at one end, move, release at the other; an HTML drag the page starts is
/// finished with its drop events.
async fn drag(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let (from, from_what) = drag_end("from", page, tab, arguments, hub, cx).await?;
    let (to, to_what) = drag_end("to", page, tab, arguments, hub, cx).await?;
    let intercepted = hub.update(cx, |hub, _| hub.watch_drag(tab));
    let page = page.clone();
    acting(
        hub,
        tab,
        format!("dragging {from_what} to {to_what}"),
        format!("dragged {from_what} to {to_what}"),
        by,
        cx,
        async move |cx| drag_between(&page, from, to, intercepted, cx).await,
    )
    .await
    .map_err(Refusal::from)
}

/// The drag itself: with Chromium holding the page's HTML drag for Marley, as Playwright's
/// Chromium driver does, since headless Chromium never finishes a native drag; a drag the page
/// follows with pointer events alone ends with the release.
async fn drag_between(
    page: &Page,
    from: (f64, f64),
    to: (f64, f64),
    mut intercepted: oneshot::Receiver<Value>,
    cx: &AsyncApp,
) -> Result<(), String> {
    page.call("Input.setInterceptDrags", json!({ "enabled": true }))
        .await
        .map_err(|error| error.to_string())?;
    let dragged = async {
        mouse(page, "mouseMoved", from, "none", 0).await?;
        mouse(page, "mousePressed", from, "left", 1).await?;
        let mut data = None;
        for step in 1..=DRAG_STEPS {
            let along = f64::from(step) / f64::from(DRAG_STEPS);
            let point = (
                (to.0 - from.0).mul_add(along, from.0),
                (to.1 - from.1).mul_add(along, from.1),
            );
            mouse(page, "mouseMoved", point, "left", 1).await?;
            if let Ok(Some(held)) = intercepted.try_recv() {
                data = Some(held);
                break;
            }
        }
        if data.is_none() {
            cx.background_executor().timer(DRAG_SETTLE).await;
            data = intercepted.try_recv().ok().flatten();
        }
        match data {
            Some(data) => {
                for kind in ["dragEnter", "dragOver", "drop"] {
                    page.call(
                        "Input.dispatchDragEvent",
                        json!({ "type": kind, "x": to.0, "y": to.1, "data": data, "modifiers": 0 }),
                    )
                    .await
                    .map_err(|error| error.to_string())?;
                }
                Ok(())
            }
            None => mouse(page, "mouseReleased", to, "left", 0).await,
        }
    }
    .await;
    page.call("Input.setInterceptDrags", json!({ "enabled": false }))
        .await
        .log_err();
    dragged
}

/// `browser_upload`: files from the open projects onto a ref's file input.
async fn upload(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let reference = arguments.get("ref").and_then(Value::as_str);
    // No ref: the file chooser the page waits on (#767).
    if reference.is_none() && hub.read_with(cx, |hub, _| hub.chooser(tab).is_none()) {
        return Err(Refusal::new(
            "no_chooser",
            "the page waits on no file chooser, and no `ref` names a file input",
        )
        .next("Give `ref`, the file input's ref from browser_snapshot."));
    }
    let paths = strings(arguments, "paths")
        .filter(|paths| !paths.is_empty())
        .ok_or_else(|| bad_argument("give `paths`, absolute paths of files to upload"))?;
    if let Some(relative) = paths.iter().find(|path| !Path::new(path).is_absolute()) {
        return Err(bad_argument(format!("{relative} is not an absolute path")));
    }
    // A link inside a project can point anywhere, so each file is checked where it really is.
    let real = cx
        .background_spawn(futures::future::lazy(move |_| {
            paths
                .iter()
                .map(|path| match std::fs::canonicalize(path) {
                    Ok(real) if real.is_file() => Ok(real),
                    _ => Err(path.clone()),
                })
                .collect::<Vec<_>>()
        }))
        .await;
    let mut files = Vec::new();
    for found in real {
        match found {
            Ok(real) => files.push(real),
            Err(path) => {
                return Err(Refusal::new("no_file", format!("{path} is not a file")));
            }
        }
    }
    cx.update(|cx| allowed_uploads(&files, cx))?;
    let Some(reference) = reference else {
        return answer_chooser(tab, files, hub, by, cx).await;
    };
    let target = ref_target(tab, reference, hub, cx)?;
    let (session, _) = ref_origin(page, tab, &target, hub, cx).await?;
    let what = target.describe();
    let names: Vec<String> = files
        .iter()
        .filter_map(|file| file.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    let shown = names.join(", ");
    let files: Vec<String> = files
        .iter()
        .map(|file| file.to_string_lossy().into_owned())
        .collect();
    let page = page.clone();
    let node = target.backend_node_id;
    let described = what.clone();
    acting(
        hub,
        tab,
        format!("uploading {shown} to {what}"),
        format!("uploaded {shown} to {what}"),
        by,
        cx,
        async move |_| {
            page.set_file_input(&session, node, &files)
                .await
                .map_err(|error| {
                    if error.to_string().contains("file input") {
                        Refusal::new(
                            "not_a_file_input",
                            format!("{described} is not a file input"),
                        )
                    } else {
                        Refusal::from(error.to_string())
                    }
                })
        },
    )
    .await
}

/// `browser_upload` with no ref: `files` answer the file chooser the page waits on (#767).
async fn answer_chooser(
    tab: &str,
    files: Vec<PathBuf>,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let shown = files
        .iter()
        .filter_map(|file| file.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(", ");
    let files: Vec<String> = files
        .iter()
        .map(|file| file.to_string_lossy().into_owned())
        .collect();
    let hub_for_action = hub.clone();
    let tab_for_action = tab.to_string();
    acting(
        hub,
        tab,
        format!("choosing {shown} for the page's file chooser"),
        format!("chose {shown} for the page's file chooser"),
        by,
        cx,
        async move |cx| {
            let task =
                hub_for_action.update(cx, |hub, cx| hub.answer_chooser(&tab_for_action, files, cx));
            task.await
        },
    )
    .await
    .map_err(Refusal::from)
}

/// Refuses a file outside every open project, or one a secret glob matches.
fn allowed_uploads(files: &[PathBuf], cx: &App) -> Result<(), Refusal> {
    for file in files {
        if crate::editor_tools::project_holding(file, cx).is_none() {
            return Err(Refusal::new(
                "outside_projects",
                format!("{} is in none of the open projects", file.display()),
            )
            .next("Upload a file from a folder of a project open in Marley."));
        }
        crate::editor_tools::not_secret(file, cx)?;
    }
    Ok(())
}

/// `browser_dialog`: the tab's open dialog answered, as its card's OK or Cancel would.
fn dialog(
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let accept = arguments
        .get("accept")
        .and_then(Value::as_bool)
        .ok_or_else(|| bad_argument("give `accept`: true for OK, false for Cancel"))?;
    let text = arguments
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_string);
    let open = hub.read_with(cx, |hub, _| hub.dialog(tab).cloned());
    let Some(open) = open else {
        return Err(Refusal::new("no_dialog", "the tab shows no dialog")
            .next("A tool whose action opens a dialog names it in its answer."));
    };
    let kind = crate::browser_tools::dialog_json(&open, cx)["kind"]
        .as_str()
        .unwrap_or("JavaScript")
        .to_string();
    let did = format!(
        "{} the {kind} dialog",
        if accept { "accepted" } else { "dismissed" }
    );
    hub.update(cx, |hub, cx| {
        hub.answer_dialog(tab, accept, text, cx);
        hub.agent_ended(tab, SharedString::from(did.clone()), by.cloned(), cx);
    });
    Ok(done(&did, tab, hub, cx))
}

/// `browser_resize`: the page laid out as a device, at a size, or back at the tab's.
async fn resize(
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    if arguments.get("fill").and_then(Value::as_bool) == Some(true) {
        hub.update(cx, |hub, cx| hub.fill(tab, cx));
        let did = "laid the page out at the tab's size".to_string();
        hub.update(cx, |hub, cx| {
            hub.agent_ended(tab, SharedString::from(did.clone()), by.cloned(), cx);
        });
        return Ok(done(&did, tab, hub, cx));
    }
    let emulated = if let Some(preset) = arguments.get("preset").and_then(Value::as_str) {
        let device = device_named(preset).ok_or_else(|| {
            let names: Vec<&str> = DEVICES.iter().map(|device| device.name).collect();
            Refusal::new(
                "no_preset",
                format!(
                    "no device named {preset}; the devices are {}",
                    names.join(", ")
                ),
            )
        })?;
        Emulated {
            preset: Some(device.name),
            width: device.width,
            height: device.height,
            scale: device.scale,
            mobile: device.mobile,
        }
    } else {
        let size = |name: &str, range: std::ops::RangeInclusive<u64>| {
            arguments
                .get(name)
                .and_then(Value::as_u64)
                .filter(|size| range.contains(size))
                .and_then(|size| u32::try_from(size).ok())
        };
        match (size("width", 320..=3840), size("height", 240..=2160)) {
            (Some(width), Some(height)) => Emulated {
                preset: None,
                width,
                height,
                scale: 1.0,
                mobile: false,
            },
            _ => {
                return Err(bad_argument(
                    "give `preset`, or `width` (320 to 3840) and `height` (240 to 2160), or \
                     `fill`",
                ));
            }
        }
    };
    let size = format!("{}×{}", emulated.width, emulated.height);
    let named = emulated
        .preset
        .map_or_else(|| size.clone(), |preset| format!("{preset} ({size})"));
    let hub_for_action = hub.clone();
    let tab_for_action = tab.to_string();
    acting(
        hub,
        tab,
        format!("laying the page out as {named}"),
        format!("laid the page out as {named}"),
        by,
        cx,
        async move |cx| {
            let task =
                hub_for_action.update(cx, |hub, cx| hub.emulate(&tab_for_action, emulated, cx));
            task.await
        },
    )
    .await
    .map_err(Refusal::from)
}

/// `browser_color_scheme`: `prefers-color-scheme` forced, or the system's again.
async fn color_scheme(
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    by: Option<&SharedString>,
    cx: &mut AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let scheme = match arguments.get("scheme").and_then(Value::as_str) {
        Some("dark") => Some(Scheme::Dark),
        Some("light") => Some(Scheme::Light),
        Some("system") => None,
        _ => return Err(bad_argument("give `scheme`: dark, light or system")),
    };
    let named = scheme.map_or("the system's", Scheme::css);
    let hub_for_action = hub.clone();
    let tab_for_action = tab.to_string();
    acting(
        hub,
        tab,
        format!("switching the page to {named} colours"),
        format!("switched the page to {named} colours"),
        by,
        cx,
        async move |cx| {
            let task =
                hub_for_action.update(cx, |hub, cx| hub.set_scheme(&tab_for_action, scheme, cx));
            task.await
        },
    )
    .await
    .map_err(Refusal::from)
}

/// `browser_wait`: until every condition given holds, or the time is up.
async fn wait(
    page: &Page,
    tab: &str,
    arguments: &Value,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let text_of = |name: &str| {
        arguments
            .get(name)
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    };
    let (shown, gone, role, name, url) = (
        text_of("text"),
        text_of("text_gone"),
        text_of("role"),
        text_of("name"),
        text_of("url"),
    );
    if shown.is_none() && gone.is_none() && role.is_none() && url.is_none() {
        return Err(bad_argument(
            "give what to wait for: `text`, `text_gone`, `role` (with `name`) or `url`",
        ));
    }
    let timeout = Duration::from_millis(
        arguments
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(WAIT_DEFAULT_MS)
            .min(WAIT_MOST_MS),
    );
    let started = Instant::now();
    loop {
        let mut held = Vec::new();
        let mut missing = Vec::new();
        if shown.is_some() || gone.is_some() {
            let text = page
                .page_text()
                .await
                .map_err(|error| Refusal::from(error.to_string()))?;
            if let Some(shown) = &shown {
                let line = format!("the text \"{shown}\" shows");
                if text.contains(shown.as_str()) {
                    held.push(line);
                } else {
                    missing.push(line);
                }
            }
            if let Some(gone) = &gone {
                let line = format!("the text \"{gone}\" is gone");
                if text.contains(gone.as_str()) {
                    missing.push(line);
                } else {
                    held.push(line);
                }
            }
        }
        if let Some(role) = &role {
            let line = name.as_ref().map_or_else(
                || format!("a {role} is there"),
                |name| format!("a {role} named \"{name}\" is there"),
            );
            let found = page
                .has_element(role, name.as_deref())
                .await
                .map_err(|error| Refusal::from(error.to_string()))?;
            if found {
                held.push(line);
            } else {
                missing.push(line);
            }
        }
        if let Some(url) = &url {
            let line = format!("the URL holds \"{url}\"");
            let now = hub.read_with(cx, |hub, _| hub.url(tab));
            if now.is_some_and(|now| now.contains(url.as_str())) {
                held.push(line);
            } else {
                missing.push(line);
            }
        }
        let waited = started.elapsed();
        let waited_ms = u64::try_from(waited.as_millis()).unwrap_or(u64::MAX);
        if missing.is_empty() {
            return Ok(ToolAnswer {
                structured: json!({ "tab": tab, "held": held, "waited_ms": waited_ms }),
                text: None,
                image: None,
            });
        }
        if waited >= timeout {
            return Err(Refusal::new(
                "not_met",
                format!("after {waited_ms} ms, not yet: {}", missing.join("; ")),
            )
            .next("Take a snapshot with browser_snapshot to see the page as it is."));
        }
        cx.background_executor().timer(WAIT_STEP).await;
    }
}
