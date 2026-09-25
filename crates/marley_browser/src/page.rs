//! One page of the browser, attached as a flat session: its viewport and its screencast, its
//! navigation and history, and the JavaScript dialogs it opens.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cdp::{CdpError, Connection};
use crate::snapshot::AxNode;

/// The screencast's JPEG quality: text stays crisp and a frame stays small.
const SCREENCAST_QUALITY: u8 = 85;

/// The script that reads what is selected: the focused text field's selection, which the
/// document's selection leaves out, or else the document's.
const SELECTED_TEXT: &str = "(() => {
  const field = document.activeElement;
  if (field && typeof field.selectionStart === 'number' && typeof field.value === 'string') {
    return field.value.slice(field.selectionStart, field.selectionEnd);
  }
  return String(getSelection());
})()";

/// The script, run in an isolated world, that gives the focused element: the page's own scripts
/// neither see it nor change what it finds.
const ACTIVE_ELEMENT: &str = "(() => { const element = document.activeElement; \
return element && element !== document.body && element !== document.documentElement ? element : null; })()";

/// What the focused element says of itself; a password field keeps its value.
const DESCRIBE_ELEMENT: &str = "function () { return { tag: this.tagName.toLowerCase(), \
kind: typeof this.type === 'string' ? this.type : null, \
value: this.type === 'password' || typeof this.value !== 'string' ? null : this.value }; }";

/// A page target, attached with its own session on the browser's connection.
#[derive(Debug, Clone)]
pub struct Page {
    connection: Connection,
    target_id: String,
    session_id: String,
}

/// What the browser says of one of its targets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    /// The target's id.
    pub target_id: String,
    /// Its kind: `page`, `iframe`, `service_worker`, `browser_ui` and so on.
    #[serde(rename = "type")]
    pub kind: String,
    /// The page's title.
    pub title: String,
    /// The page's URL.
    pub url: String,
}

/// A screencast frame: the JPEG as base64, where the page was when it was drawn, and the
/// number its acknowledgement names.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreencastFrame {
    /// The JPEG, base64.
    pub data: String,
    /// The page's geometry for this frame.
    pub metadata: FrameMetadata,
    /// The number [`Page::ack_frame`] sends back.
    pub session_id: i64,
}

/// The page's geometry when a frame was drawn, as `Page.screencastFrame` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameMetadata {
    /// The top offset, in DIP.
    pub offset_top: f64,
    /// The pinch zoom.
    pub page_scale_factor: f64,
    /// The viewport's width, in DIP.
    pub device_width: f64,
    /// The viewport's height, in DIP.
    pub device_height: f64,
    /// The horizontal scroll, in CSS pixels.
    pub scroll_offset_x: f64,
    /// The vertical scroll, in CSS pixels.
    pub scroll_offset_y: f64,
    /// When the frame was swapped, in seconds since the epoch.
    #[serde(default)]
    pub timestamp: Option<f64>,
}

/// The page's session history, as `Page.getNavigationHistory` reports it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationHistory {
    /// The index of the entry the page shows.
    pub current_index: usize,
    /// The entries, oldest first.
    pub entries: Vec<HistoryEntry>,
}

/// One entry of the session history.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HistoryEntry {
    /// The number [`Page::go_to_history_entry`] takes.
    pub id: i64,
    /// The entry's URL.
    pub url: String,
    /// The entry's title.
    #[serde(default)]
    pub title: String,
}

impl NavigationHistory {
    /// The entry `offset` steps from the one the page shows, -1 back and 1 forward, with its
    /// index.
    #[must_use]
    pub fn entry_at(&self, offset: isize) -> Option<(usize, &HistoryEntry)> {
        let index = self.current_index.checked_add_signed(offset)?;
        self.entries.get(index).map(|entry| (index, entry))
    }
}

/// A JavaScript dialog the page opened, as `Page.javascriptDialogOpening` reports it. Headless
/// Chromium draws none, and the page's script waits until one is answered.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaScriptDialog {
    /// The URL of the frame that opened it.
    pub url: String,
    /// Its message.
    pub message: String,
    /// Which dialog it is.
    #[serde(rename = "type")]
    pub kind: DialogKind,
    /// The text a `prompt` starts with.
    #[serde(default)]
    pub default_prompt: String,
}

impl JavaScriptDialog {
    /// The host, with its port, of the frame that opened the dialog, which the dialog names as
    /// the one asking; none for a page with no host, such as `about:blank`.
    #[must_use]
    pub fn origin(&self) -> Option<String> {
        let url = url::Url::parse(&self.url).ok()?;
        let host = url.host_str()?;
        Some(
            url.port()
                .map_or_else(|| host.to_string(), |port| format!("{host}:{port}")),
        )
    }
}

/// The kinds of JavaScript dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DialogKind {
    /// `alert`: a message and OK.
    Alert,
    /// `confirm`: a message, OK and Cancel.
    Confirm,
    /// `prompt`: a message, a field for the answer, OK and Cancel.
    Prompt,
    /// The page asks before it is left.
    #[serde(rename = "beforeunload")]
    BeforeUnload,
}

/// The page's viewport, as `Page.getLayoutMetrics` gives it, in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Viewport {
    /// Its width.
    pub width: f64,
    /// Its height.
    pub height: f64,
    /// How far the page is scrolled right.
    pub scroll_x: f64,
    /// How far the page is scrolled down.
    pub scroll_y: f64,
    /// The pinch zoom.
    pub scale: f64,
}

/// The element that has the focus, as an agent sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FocusedElement {
    /// Its tag, lowercased.
    pub tag: String,
    /// An input's type.
    pub kind: Option<String>,
    /// Its accessible role.
    pub role: Option<String>,
    /// Its accessible name.
    pub name: Option<String>,
    /// A field's value; never a password field's.
    pub value: Option<String>,
}

/// A frame of the page, from its frame tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInfo {
    /// The frame's id.
    pub id: String,
    /// Its URL.
    pub url: String,
    /// Whether it is the main frame.
    pub main: bool,
}

impl Page {
    /// Attaches to the browser's first page, or to a new blank one when it has none, and turns
    /// on what the Browser tab needs: the page's events, focus as if the page had it, and every
    /// target's title and URL as they change.
    ///
    /// # Errors
    ///
    /// When a call fails or the browser's answers lack what attaching needs.
    pub async fn attach_first(connection: &Connection) -> Result<Self, CdpError> {
        connection
            .call(
                "Target.setDiscoverTargets",
                json!({ "discover": true }),
                None,
            )
            .await?;
        let targets = connection
            .call("Target.getTargets", json!({}), None)
            .await?;
        let infos: Vec<TargetInfo> = targets
            .get("targetInfos")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| CdpError::Unexpected(format!("the browser's targets: {error}")))?
            .unwrap_or_default();
        let first_page = infos.into_iter().find(|info| info.kind == "page");
        let target_id = if let Some(info) = first_page {
            info.target_id
        } else {
            let created = connection
                .call("Target.createTarget", json!({ "url": "about:blank" }), None)
                .await?;
            string_field(&created, "targetId")?
        };
        let attached = connection
            .call(
                "Target.attachToTarget",
                json!({ "targetId": target_id, "flatten": true }),
                None,
            )
            .await?;
        let page = Self {
            connection: connection.clone(),
            session_id: string_field(&attached, "sessionId")?,
            target_id,
        };
        page.call("Page.enable", json!({})).await?;
        page.call(
            "Emulation.setFocusEmulationEnabled",
            json!({ "enabled": true }),
        )
        .await?;
        Ok(page)
    }

    /// The page's target id.
    #[must_use]
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// The page's flat session.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// What the browser says of the page now. Its title is only here: Chromium reports a new
    /// URL as a target change, with the URL as the title, but not the title the document sets.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer is not a target's.
    pub async fn target_info(&self) -> Result<TargetInfo, CdpError> {
        let answer = self
            .connection
            .call(
                "Target.getTargetInfo",
                json!({ "targetId": self.target_id }),
                None,
            )
            .await?;
        answer
            .get("targetInfo")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| CdpError::Unexpected(format!("the page's target: {error}")))?
            .ok_or_else(|| CdpError::Unexpected("the browser's answer has no targetInfo".into()))
    }

    /// Sends `method` with `params` to the page's session.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn call(&self, method: &str, params: Value) -> Result<Value, CdpError> {
        self.connection
            .call(method, params, Some(&self.session_id))
            .await
    }

    /// Lays the page out in a viewport of `width` by `height` CSS pixels, at `scale` device
    /// pixels to the CSS pixel.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn set_viewport(&self, width: u32, height: u32, scale: f32) -> Result<(), CdpError> {
        self.call(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "width": width,
                "height": height,
                "deviceScaleFactor": scale,
                "mobile": false,
            }),
        )
        .await
        .map(drop)
    }

    /// Starts streaming the page as JPEG frames.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn start_screencast(&self) -> Result<(), CdpError> {
        self.call(
            "Page.startScreencast",
            json!({ "format": "jpeg", "quality": SCREENCAST_QUALITY }),
        )
        .await
        .map(drop)
    }

    /// Stops the stream.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn stop_screencast(&self) -> Result<(), CdpError> {
        self.call("Page.stopScreencast", json!({})).await.map(drop)
    }

    /// What is selected in the page's main frame, read in an isolated world, where the page's
    /// own scripts can neither see the read nor change what it returns.
    ///
    /// # Errors
    ///
    /// When a call fails or the page has no main frame.
    pub async fn selected_text(&self) -> Result<String, CdpError> {
        let context = self.isolated_context().await?;
        let evaluated = self
            .call(
                "Runtime.evaluate",
                json!({ "expression": SELECTED_TEXT, "contextId": context, "returnByValue": true }),
            )
            .await?;
        Ok(evaluated
            .pointer("/result/value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string())
    }

    /// Navigates the page to `url`. Chromium answers once the navigation commits or fails.
    ///
    /// # Errors
    ///
    /// When the call fails, or the navigation did (Chromium's `errorText`: a stopped one is
    /// `net::ERR_ABORTED`).
    pub async fn navigate(&self, url: &str) -> Result<(), CdpError> {
        let answer = self.call("Page.navigate", json!({ "url": url })).await?;
        match answer.get("errorText").and_then(Value::as_str) {
            Some(error) if !error.is_empty() => Err(CdpError::Protocol {
                method: "Page.navigate".to_string(),
                message: error.to_string(),
            }),
            _ => Ok(()),
        }
    }

    /// Loads the page again.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn reload(&self) -> Result<(), CdpError> {
        self.call("Page.reload", json!({})).await.map(drop)
    }

    /// Stops the page's loading.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn stop_loading(&self) -> Result<(), CdpError> {
        self.call("Page.stopLoading", json!({})).await.map(drop)
    }

    /// The page's session history.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer is not a history.
    pub async fn history(&self) -> Result<NavigationHistory, CdpError> {
        let answer = self.call("Page.getNavigationHistory", json!({})).await?;
        serde_json::from_value(answer)
            .map_err(|error| CdpError::Unexpected(format!("the page's history: {error}")))
    }

    /// Moves the page to the history entry `id`.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn go_to_history_entry(&self, id: i64) -> Result<(), CdpError> {
        self.call("Page.navigateToHistoryEntry", json!({ "entryId": id }))
            .await
            .map(drop)
    }

    /// Answers the page's JavaScript dialog: OK when `accept`, else Cancel, with
    /// `prompt_text` as a `prompt`'s answer.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`]; a page with no dialog open answers with an error.
    pub async fn answer_dialog(
        &self,
        accept: bool,
        prompt_text: Option<&str>,
    ) -> Result<(), CdpError> {
        let mut params = json!({ "accept": accept });
        if let (Some(text), Value::Object(fields)) = (prompt_text, &mut params) {
            fields.insert("promptText".into(), Value::from(text));
        }
        self.call("Page.handleJavaScriptDialog", params)
            .await
            .map(drop)
    }

    /// Sends `method` with `params` to `session`, one of this browser's: the page's, or a
    /// cross-site iframe's.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn call_in(
        &self,
        session: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, CdpError> {
        self.connection.call(method, params, Some(session)).await
    }

    /// Turns on, in `session`, what the agent tools read: console messages and uncaught errors,
    /// the network, the browser's log, and each cross-site iframe as a session of its own.
    ///
    /// # Errors
    ///
    /// When a call fails.
    pub async fn observe(&self, session: &str) -> Result<(), CdpError> {
        for (method, params) in [
            ("Runtime.enable", json!({})),
            ("Network.enable", json!({})),
            ("Log.enable", json!({})),
            (
                "Target.setAutoAttach",
                json!({ "autoAttach": true, "waitForDebuggerOnStart": false, "flatten": true }),
            ),
        ] {
            self.call_in(session, method, params).await?;
        }
        Ok(())
    }

    /// The page as a JPEG, base64, at its viewport's size.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer holds no image.
    pub async fn screenshot(&self) -> Result<String, CdpError> {
        let answer = self
            .call(
                "Page.captureScreenshot",
                json!({ "format": "jpeg", "quality": SCREENCAST_QUALITY }),
            )
            .await?;
        string_field(&answer, "data")
    }

    /// The page's viewport and scroll.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer lacks the viewports.
    pub async fn viewport(&self) -> Result<Viewport, CdpError> {
        let answer = self.call("Page.getLayoutMetrics", json!({})).await?;
        let number = |pointer: &str| {
            answer
                .pointer(pointer)
                .and_then(Value::as_f64)
                .ok_or_else(|| CdpError::Unexpected(format!("the layout metrics lack {pointer}")))
        };
        Ok(Viewport {
            width: number("/cssLayoutViewport/clientWidth")?,
            height: number("/cssLayoutViewport/clientHeight")?,
            scroll_x: number("/cssLayoutViewport/pageX")?,
            scroll_y: number("/cssLayoutViewport/pageY")?,
            scale: number("/cssVisualViewport/scale").unwrap_or(1.0),
        })
    }

    /// The page's frames, the main one first: the ones in its own process, and the cross-site
    /// ones by id and URL.
    ///
    /// # Errors
    ///
    /// When the call fails.
    pub async fn frames(&self) -> Result<Vec<FrameInfo>, CdpError> {
        let tree = self.call("Page.getFrameTree", json!({})).await?;
        let mut frames = Vec::new();
        let mut pending = vec![(tree.get("frameTree").cloned().unwrap_or_default(), true)];
        while let Some((node, main)) = pending.pop() {
            if let Some(id) = node.pointer("/frame/id").and_then(Value::as_str) {
                frames.push(FrameInfo {
                    id: id.to_string(),
                    url: node
                        .pointer("/frame/url")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    main,
                });
            }
            if let Some(children) = node.get("childFrames").and_then(Value::as_array) {
                pending.extend(children.iter().rev().cloned().map(|child| (child, false)));
            }
        }
        Ok(frames)
    }

    /// The accessibility tree of `frame` in `session`, or of the session's main frame.
    ///
    /// # Errors
    ///
    /// When the call fails or its answer is not a tree.
    pub async fn accessibility_tree(
        &self,
        session: &str,
        frame: Option<&str>,
    ) -> Result<Vec<AxNode>, CdpError> {
        let params = frame.map_or_else(|| json!({}), |frame| json!({ "frameId": frame }));
        let answer = self
            .call_in(session, "Accessibility.getFullAXTree", params)
            .await?;
        answer
            .get("nodes")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| CdpError::Unexpected(format!("the accessibility tree: {error}")))?
            .ok_or_else(|| CdpError::Unexpected("the accessibility tree has no nodes".into()))
    }

    /// Scrolls the DOM node `backend_node_id` of `session` into view.
    ///
    /// # Errors
    ///
    /// When the call fails, as it does for a node that is gone.
    pub async fn scroll_into_view(
        &self,
        session: &str,
        backend_node_id: i64,
    ) -> Result<(), CdpError> {
        self.call_in(
            session,
            "DOM.scrollIntoViewIfNeeded",
            json!({ "backendNodeId": backend_node_id }),
        )
        .await
        .map(drop)
    }

    /// The middle of the DOM node `backend_node_id` of `session`, in its frame's viewport.
    ///
    /// # Errors
    ///
    /// When the call fails, as it does for a node that is gone or has no box.
    pub async fn box_center(
        &self,
        session: &str,
        backend_node_id: i64,
    ) -> Result<(f64, f64), CdpError> {
        let quad = self.content_quad(session, backend_node_id).await?;
        let xs = [quad[0], quad[2], quad[4], quad[6]];
        let ys = [quad[1], quad[3], quad[5], quad[7]];
        Ok((xs.iter().sum::<f64>() / 4.0, ys.iter().sum::<f64>() / 4.0))
    }

    /// Where the cross-site iframe `frame` shows its content in the page's viewport: the top left
    /// of its owner element's content box, scrolled into view first.
    ///
    /// # Errors
    ///
    /// When a call fails.
    pub async fn frame_origin(&self, frame: &str) -> Result<(f64, f64), CdpError> {
        let owner = self
            .call("DOM.getFrameOwner", json!({ "frameId": frame }))
            .await?;
        let backend = owner
            .get("backendNodeId")
            .and_then(Value::as_i64)
            .ok_or_else(|| CdpError::Unexpected("the frame has no owner element".into()))?;
        self.scroll_into_view(&self.session_id, backend).await?;
        let quad = self.content_quad(&self.session_id, backend).await?;
        Ok((quad[0], quad[1]))
    }

    async fn content_quad(
        &self,
        session: &str,
        backend_node_id: i64,
    ) -> Result<[f64; 8], CdpError> {
        let answer = self
            .call_in(
                session,
                "DOM.getBoxModel",
                json!({ "backendNodeId": backend_node_id }),
            )
            .await?;
        let points: Vec<f64> = answer
            .pointer("/model/content")
            .and_then(Value::as_array)
            .map(|points| points.iter().filter_map(Value::as_f64).collect())
            .unwrap_or_default();
        points
            .try_into()
            .map_err(|_| CdpError::Unexpected("the element's box has no content quad".into()))
    }

    /// The element that has the focus in the main frame, if one does; read in an isolated world.
    ///
    /// # Errors
    ///
    /// When a call fails.
    pub async fn focused_element(&self) -> Result<Option<FocusedElement>, CdpError> {
        let context = self.isolated_context().await?;
        let evaluated = self
            .call(
                "Runtime.evaluate",
                json!({ "expression": ACTIVE_ELEMENT, "contextId": context }),
            )
            .await?;
        let Some(object) = evaluated
            .pointer("/result/objectId")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            return Ok(None);
        };
        let described = self
            .call(
                "Runtime.callFunctionOn",
                json!({
                    "objectId": object,
                    "functionDeclaration": DESCRIBE_ELEMENT,
                    "returnByValue": true,
                }),
            )
            .await;
        let node = self
            .call("DOM.describeNode", json!({ "objectId": object }))
            .await;
        self.call("Runtime.releaseObject", json!({ "objectId": object }))
            .await?;
        let described = described?;
        let fields = described
            .pointer("/result/value")
            .cloned()
            .unwrap_or_default();
        let text = |name: &str| fields.get(name).and_then(Value::as_str).map(str::to_string);
        let mut element = FocusedElement {
            tag: text("tag").unwrap_or_default(),
            kind: text("kind"),
            role: None,
            name: None,
            value: text("value"),
        };
        if let Some(backend) = node?.pointer("/node/backendNodeId").and_then(Value::as_i64) {
            let tree = self
                .call(
                    "Accessibility.getPartialAXTree",
                    json!({ "backendNodeId": backend, "fetchRelatives": false }),
                )
                .await?;
            let first = tree.pointer("/nodes/0");
            let value = |pointer: &str| {
                first
                    .and_then(|node| node.pointer(pointer))
                    .and_then(Value::as_str)
                    .map(|text| text.trim().to_string())
            };
            element.role = value("/role/value");
            element.name = value("/name/value");
        }
        Ok(Some(element))
    }

    /// An isolated world in the main frame, where Marley's own scripts run unseen by the page's.
    async fn isolated_context(&self) -> Result<i64, CdpError> {
        let tree = self.call("Page.getFrameTree", json!({})).await?;
        let frame_id = tree
            .pointer("/frameTree/frame/id")
            .and_then(Value::as_str)
            .ok_or_else(|| CdpError::Unexpected("the page has no main frame".to_string()))?;
        let world = self
            .call(
                "Page.createIsolatedWorld",
                json!({ "frameId": frame_id, "worldName": "marley" }),
            )
            .await?;
        world
            .get("executionContextId")
            .and_then(Value::as_i64)
            .ok_or_else(|| CdpError::Unexpected("the isolated world has no context".to_string()))
    }

    /// Acknowledges frame `frame`, which lets Chromium send the next: an unacknowledged stream
    /// stops.
    ///
    /// # Errors
    ///
    /// As [`Connection::call`].
    pub async fn ack_frame(&self, frame: i64) -> Result<(), CdpError> {
        self.call("Page.screencastFrameAck", json!({ "sessionId": frame }))
            .await
            .map(drop)
    }
}

/// The string field `name` of an answer.
fn string_field(answer: &Value, name: &str) -> Result<String, CdpError> {
    answer
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| CdpError::Unexpected(format!("the browser's answer has no {name}")))
}
