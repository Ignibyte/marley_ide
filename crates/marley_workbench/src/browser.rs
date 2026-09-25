//! The Browser tab (prong 3 B0a): the page Marley's own Chromium renders, in the main area.
//!
//! One [`BrowserHub`] per app owns the connection to the browser and the page the Browser tabs
//! show, so the user and an agent attached to the same Chromium see one page. The hub starts
//! Chromium when no Marley Chromium answers, streams the page as screencast frames while a tab
//! shows it, and lays the page out at the tab's size. [`BrowserView`] is the tab: it draws the
//! newest frame and frees each frame from the window's GPU atlas two paints after it was first
//! drawn, as Zed's screen-share view does, since the window may present the last frame again.
//!
//! Input (B0b, #489) goes to the page over CDP while the tab has the focus: the mouse and the
//! wheel from `PageElement`'s listeners, at CSS pixels taken from the frame's metadata; keys
//! from the tab's `key_down`, as `marley_browser::input` maps them; composed and input-method
//! text through the tab's input handler; and the system clipboard, which headless Chromium does
//! not share, through Marley.

use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui::{
    App, AsyncApp, BackgroundExecutor, Bounds, ClipboardItem, Corners, DispatchPhase, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, Global, GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, KeyDownEvent,
    LayoutId, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    RenderImage, ScrollWheelEvent, Size, Style, Subscription, Task, UTF16Selection, WeakEntity,
    relative,
};
use marley_browser::cdp::{self, Connection, Event};
use marley_browser::input::{self, KeyPress};
use marley_browser::page::{FrameMetadata, Page, ScreencastFrame, TargetInfo};
use marley_browser::{frame, service};
use serde_json::Value;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

use crate::OpenBrowser;

/// How many times, a tenth of a second apart, a start waits for Chromium to write its endpoint
/// and answer.
const START_POLLS: u32 = 150;

/// The wait between two of those tries.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// The CSS pixels a wheel line scrolls: three lines a detent make the 100 a mouse wheel turns in
/// Chrome.
const WHEEL_LINE: f32 = 100.0 / 3.0;

/// What the hub is doing, as the Browser tab shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubState {
    /// Finding or starting Chromium.
    Starting,
    /// Attaching to its page.
    Connecting,
    /// Showing the page.
    Showing,
    /// Stopped, for the reason given.
    Failed(SharedString),
}

/// The page's title or URL changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageInfoChanged;

/// The viewport a tab asked for: its size in whole logical pixels, at the window's scale.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Viewport {
    width: u32,
    height: u32,
    scale: f32,
}

/// Marley's browser as the Browser tabs see it: the connection's state, the page, its newest
/// frame, and how many tabs show it.
pub struct BrowserHub {
    state: HubState,
    page: Option<Page>,
    frame: Option<Arc<RenderImage>>,
    /// The newest frame's geometry, which maps a point in the tab to the page.
    metadata: Option<FrameMetadata>,
    title: Option<SharedString>,
    url: Option<SharedString>,
    viewport: Option<Viewport>,
    viewers: usize,
    screencasting: bool,
    /// The mouse buttons held in the page, in CDP's bits, so a drag that leaves the tab still
    /// reaches the page, its release too.
    held_buttons: u32,
    /// When the oldest input that no frame has shown yet was sent.
    input_at: Option<Instant>,
    /// Bumped at each start, so a superseded start's late results are dropped.
    generation: u64,
    run: Option<Task<()>>,
}

impl fmt::Debug for BrowserHub {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserHub")
            .field("state", &self.state)
            .field("url", &self.url)
            .field("viewers", &self.viewers)
            .field("screencasting", &self.screencasting)
            .finish_non_exhaustive()
    }
}

impl EventEmitter<PageInfoChanged> for BrowserHub {}

struct HubHandle(Entity<BrowserHub>);

impl Global for HubHandle {}

impl BrowserHub {
    /// The app's hub, created and started the first time it is asked for.
    pub fn global(cx: &mut App) -> Entity<Self> {
        if let Some(handle) = cx.try_global::<HubHandle>() {
            return handle.0.clone();
        }
        let hub = cx.new(|cx| {
            let mut hub = Self {
                state: HubState::Starting,
                page: None,
                frame: None,
                metadata: None,
                title: None,
                url: None,
                viewport: None,
                viewers: 0,
                screencasting: false,
                held_buttons: 0,
                input_at: None,
                generation: 0,
                run: None,
            };
            hub.start(cx);
            hub
        });
        cx.set_global(HubHandle(hub.clone()));
        hub
    }

    /// What the hub is doing.
    #[must_use]
    pub const fn state(&self) -> &HubState {
        &self.state
    }

    /// The page's title, once it has one.
    #[must_use]
    pub fn title(&self) -> Option<SharedString> {
        self.title.clone().filter(|title| !title.is_empty())
    }

    /// The page's URL.
    #[must_use]
    pub fn url(&self) -> Option<SharedString> {
        self.url.clone()
    }

    /// Starts over: finds or starts Chromium, connects, attaches to its page, and follows the
    /// page's events until the connection ends.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        self.state = HubState::Starting;
        self.page = None;
        self.frame = None;
        self.metadata = None;
        self.held_buttons = 0;
        self.screencasting = false;
        if self.title.take().is_some() | self.url.take().is_some() {
            cx.emit(PageInfoChanged);
        }
        cx.notify();
        let profile = paths::data_dir().join("browser").join("profile");
        let executor = cx.background_executor().clone();
        self.run = Some(cx.spawn(async move |this, cx| {
            let (connection, events) = match open_browser(&profile, &executor, cx).await {
                Ok(opened) => opened,
                Err(error) => {
                    this.update(cx, |this, cx| {
                        this.fail(generation, format!("{error:#}"), cx);
                    })
                    .ok();
                    return;
                }
            };
            this.update(cx, |this, cx| {
                this.set_state(generation, HubState::Connecting, cx);
            })
            .ok();
            match Page::attach_first(&connection).await {
                Ok(page) => {
                    if this
                        .update(cx, |this, cx| this.attached(generation, page, cx))
                        .is_err()
                    {
                        return;
                    }
                }
                Err(error) => {
                    this.update(cx, |this, cx| {
                        this.fail(
                            generation,
                            format!("Could not attach to the browser's page: {error}"),
                            cx,
                        );
                    })
                    .ok();
                    return;
                }
            }
            follow(this, events, generation, cx).await;
        }));
    }

    fn set_state(&mut self, generation: u64, state: HubState, cx: &mut Context<Self>) {
        if generation == self.generation {
            self.state = state;
            cx.notify();
        }
    }

    fn fail(&mut self, generation: u64, reason: String, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        log::warn!("browser: {reason}");
        self.state = HubState::Failed(reason.into());
        self.page = None;
        self.frame = None;
        self.metadata = None;
        self.held_buttons = 0;
        self.screencasting = false;
        cx.notify();
    }

    fn attached(&mut self, generation: u64, page: Page, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        self.page = Some(page.clone());
        self.state = HubState::Showing;
        cx.notify();
        if let Some(viewport) = self.viewport {
            cx.spawn(async move |_, _| {
                page.set_viewport(viewport.width, viewport.height, viewport.scale)
                    .await
                    .log_err();
            })
            .detach();
        }
        self.sync_screencast(cx);
    }

    /// Lays the page out at a tab's size: `size` in logical pixels, at the window's `scale`.
    pub fn resize(&mut self, size: Size<Pixels>, scale: f32, cx: &Context<Self>) {
        let viewport = Viewport {
            width: whole_pixels(size.width),
            height: whole_pixels(size.height),
            scale,
        };
        if self.viewport == Some(viewport) {
            return;
        }
        self.viewport = Some(viewport);
        if let Some(page) = self.page.clone() {
            cx.spawn(async move |_, _| {
                page.set_viewport(viewport.width, viewport.height, viewport.scale)
                    .await
                    .log_err();
            })
            .detach();
        }
        self.sync_screencast(cx);
    }

    fn add_viewer(&mut self, cx: &mut Context<Self>) {
        self.viewers += 1;
        self.sync_screencast(cx);
    }

    fn remove_viewer(&mut self, cx: &mut Context<Self>) {
        self.viewers = self.viewers.saturating_sub(1);
        self.sync_screencast(cx);
    }

    /// Streams the page while a tab shows it and its size is known, and stops the stream when
    /// the last tab goes.
    fn sync_screencast(&mut self, cx: &Context<Self>) {
        let Some(page) = self.page.clone() else {
            return;
        };
        let wanted = self.viewers > 0 && self.viewport.is_some();
        if wanted == self.screencasting {
            return;
        }
        self.screencasting = wanted;
        cx.spawn(async move |_, _| {
            let result = if wanted {
                page.start_screencast().await
            } else {
                page.stop_screencast().await
            };
            result.log_err();
        })
        .detach();
    }

    /// Keeps a decoded frame, unless a later start superseded the stream it came from, and
    /// returns the page to acknowledge it to.
    fn show(
        &mut self,
        generation: u64,
        decoded: anyhow::Result<Arc<RenderImage>>,
        metadata: FrameMetadata,
        cx: &mut Context<Self>,
    ) -> Option<Page> {
        if generation != self.generation {
            return None;
        }
        match decoded {
            Ok(image) => {
                self.frame = Some(image);
                self.metadata = Some(metadata);
                if let Some(sent) = self.input_at.take() {
                    input::log_latency(sent);
                }
                cx.notify();
            }
            Err(error) => log::warn!("browser: a screencast frame did not decode: {error:#}"),
        }
        self.page.clone()
    }

    /// Sends `method` with `params` to the page. Each call's message leaves in the order the
    /// calls were made, so input keeps its order. A `timed` input starts the clock the next
    /// frame stops.
    fn send(&mut self, method: &'static str, params: Value, timed: bool, cx: &Context<Self>) {
        let Some(page) = self.page.clone() else {
            return;
        };
        if timed {
            self.input_at.get_or_insert_with(Instant::now);
        }
        cx.spawn(async move |_, _| {
            page.call(method, params).await.log_err();
        })
        .detach();
    }

    fn send_key(&mut self, press: KeyPress, cx: &Context<Self>) {
        self.send("Input.dispatchKeyEvent", press.down, true, cx);
        self.send("Input.dispatchKeyEvent", press.up, false, cx);
    }

    fn insert_text(&mut self, text: &str, cx: &Context<Self>) {
        self.send(
            "Input.insertText",
            serde_json::json!({ "text": text }),
            true,
            cx,
        );
    }

    fn set_composition(&mut self, text: &str, selected: Range<usize>, cx: &Context<Self>) {
        self.send(
            "Input.imeSetComposition",
            serde_json::json!({
                "text": text,
                "selectionStart": selected.start,
                "selectionEnd": selected.end,
            }),
            true,
            cx,
        );
    }

    /// Puts the page's selection on the system clipboard, then sends `press`, the Ctrl+C or
    /// Ctrl+X that asked for it: a cut must not empty the selection before it is read.
    fn copy_selection(&self, press: KeyPress, cx: &Context<Self>) {
        let Some(page) = self.page.clone() else {
            return;
        };
        cx.spawn(async move |_, cx| {
            match page.selected_text().await {
                Ok(text) if !text.is_empty() => {
                    cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string(text)));
                }
                Ok(_) => {}
                Err(error) => log::warn!("browser: reading the page's selection failed: {error}"),
            }
            page.call("Input.dispatchKeyEvent", press.down)
                .await
                .log_err();
            page.call("Input.dispatchKeyEvent", press.up)
                .await
                .log_err();
        })
        .detach();
    }

    /// Presses `button` at `point`, in CSS pixels.
    fn mouse_press(
        &mut self,
        point: (f64, f64),
        button: MouseButton,
        click_count: usize,
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let (name, bit) = input::mouse_button(button);
        self.held_buttons |= bit;
        let params = input::mouse_event(
            "mousePressed",
            point,
            name,
            self.held_buttons,
            click_count,
            input::modifier_bits(modifiers),
        );
        self.send("Input.dispatchMouseEvent", params, true, cx);
    }

    fn mouse_release(
        &mut self,
        point: (f64, f64),
        button: MouseButton,
        click_count: usize,
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let (name, bit) = input::mouse_button(button);
        self.held_buttons &= !bit;
        let params = input::mouse_event(
            "mouseReleased",
            point,
            name,
            self.held_buttons,
            click_count,
            input::modifier_bits(modifiers),
        );
        self.send("Input.dispatchMouseEvent", params, false, cx);
    }

    fn mouse_move(&mut self, point: (f64, f64), modifiers: Modifiers, cx: &Context<Self>) {
        // A move names the button it drags with, the first one held.
        let button = [
            (1, "left"),
            (4, "middle"),
            (2, "right"),
            (8, "back"),
            (16, "forward"),
        ]
        .iter()
        .find(|(bit, _)| self.held_buttons & bit != 0)
        .map_or("none", |(_, name)| name);
        let params = input::mouse_event(
            "mouseMoved",
            point,
            button,
            self.held_buttons,
            0,
            input::modifier_bits(modifiers),
        );
        self.send("Input.dispatchMouseEvent", params, false, cx);
    }

    fn wheel(
        &mut self,
        point: (f64, f64),
        delta: (f64, f64),
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let params = input::wheel_event(point, delta, input::modifier_bits(modifiers));
        self.send("Input.dispatchMouseEvent", params, true, cx);
    }

    const fn is_holding(&self) -> bool {
        self.held_buttons != 0
    }

    fn target_changed(&mut self, generation: u64, info: TargetInfo, cx: &mut Context<Self>) {
        if generation != self.generation
            || self
                .page
                .as_ref()
                .is_none_or(|page| page.target_id() != info.target_id)
        {
            return;
        }
        let title = Some(SharedString::from(info.title));
        let url = Some(SharedString::from(info.url));
        if title != self.title || url != self.url {
            self.title = title;
            self.url = url;
            cx.emit(PageInfoChanged);
        }
    }

    fn is_page(&self, target_id: Option<&str>) -> bool {
        self.page
            .as_ref()
            .is_some_and(|page| Some(page.target_id()) == target_id)
    }
}

/// Rounds a length to the whole CSS pixels CDP takes, between 1 and 16,384.
fn whole_pixels(length: Pixels) -> u32 {
    let clamped = f32::from(length).round().clamp(1.0, 16_384.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 1..=16384
    let whole = clamped as u32;
    whole
}

/// Connects to the Marley Chromium whose profile is `profile`, starting it first when none
/// answers.
async fn open_browser(
    profile: &Path,
    executor: &BackgroundExecutor,
    cx: &AsyncApp,
) -> anyhow::Result<(Connection, mpsc::UnboundedReceiver<Event>)> {
    if let Some(opened) = try_connect(profile, executor, cx).await {
        return Ok(opened);
    }
    let unit = service::unit_name(profile);
    if !service::unit_state(&unit).await?.is_up() {
        let stale = profile.to_path_buf();
        let binary_override = std::env::var_os(service::BINARY_OVERRIDE).map(PathBuf::from);
        let binary = cx
            .background_spawn(futures::future::lazy(move |_| {
                service::remove_endpoint_in(&stale)?;
                service::find_binary(binary_override.as_deref())
            }))
            .await?;
        service::start(&unit, &binary, profile).await?;
    }
    for poll in 1..=START_POLLS {
        if let Some(opened) = try_connect(profile, executor, cx).await {
            return Ok(opened);
        }
        // Once a second: a unit that is down again stopped as it started.
        if poll % 10 == 0 && !service::unit_state(&unit).await?.is_up() {
            anyhow::bail!(
                "Chromium stopped as it started; `journalctl --user -u {unit}` says why."
            );
        }
        executor.timer(POLL_INTERVAL).await;
    }
    anyhow::bail!("Chromium did not answer within 15 seconds; see `journalctl --user -u {unit}`.")
}

/// The connection to the Chromium `DevToolsActivePort` in `profile` names, when it answers.
async fn try_connect(
    profile: &Path,
    executor: &BackgroundExecutor,
    cx: &AsyncApp,
) -> Option<(Connection, mpsc::UnboundedReceiver<Event>)> {
    let profile = profile.to_path_buf();
    // A file Chromium is still writing reads as no endpoint yet, and the next try reads it again.
    let endpoint = cx
        .background_spawn(futures::future::lazy(move |_| {
            service::endpoint_in(&profile).ok().flatten()
        }))
        .await?;
    cdp::connect(endpoint.port, &endpoint.path, executor.clone())
        .await
        .ok()
}

/// Follows the browser's events for the start `generation`: frames into the hub, the page's
/// title and URL, and the page going away. When the connection ends, the hub fails.
async fn follow(
    this: WeakEntity<BrowserHub>,
    mut events: mpsc::UnboundedReceiver<Event>,
    generation: u64,
    cx: &mut AsyncApp,
) {
    while let Some(event) = events.next().await {
        match event.method.as_str() {
            "Page.screencastFrame" => {
                let Ok(frame) = serde_json::from_value::<ScreencastFrame>(event.params) else {
                    continue;
                };
                let number = frame.session_id;
                let metadata = frame.metadata;
                let data = frame.data;
                let decoded = cx
                    .background_spawn(futures::future::lazy(move |_| frame::decode(&data)))
                    .await;
                match this.update(cx, |this, cx| this.show(generation, decoded, metadata, cx)) {
                    Ok(Some(page)) => {
                        page.ack_frame(number).await.log_err();
                    }
                    Ok(None) => {}
                    Err(_) => return,
                }
            }
            // The document's own title arrives with its content, which no target event reports.
            "Page.domContentEventFired"
            | "Page.loadEventFired"
            | "Page.navigatedWithinDocument" => {
                let page = this
                    .read_with(cx, |this, _| this.page.clone())
                    .ok()
                    .flatten();
                if let Some(page) = page
                    && let Some(info) = page.target_info().await.log_err()
                {
                    this.update(cx, |this, cx| this.target_changed(generation, info, cx))
                        .ok();
                }
            }
            // Discovery reports the targets that exist already as created.
            "Target.targetCreated" | "Target.targetInfoChanged" => {
                let info = event
                    .params
                    .get("targetInfo")
                    .cloned()
                    .and_then(|info| serde_json::from_value::<TargetInfo>(info).ok());
                if let Some(info) = info {
                    this.update(cx, |this, cx| this.target_changed(generation, info, cx))
                        .ok();
                }
            }
            "Target.targetDestroyed" | "Target.targetCrashed" => {
                let target = event.params.get("targetId").and_then(|id| id.as_str());
                let ours = this
                    .update(cx, |this, _| this.is_page(target))
                    .unwrap_or(false);
                if ours {
                    // The page went away: start over on the browser's first page, in a task of
                    // its own, since starting replaces this one.
                    this.update(cx, |_, cx| {
                        cx.spawn(async move |this, cx| {
                            this.update(cx, BrowserHub::start).ok();
                        })
                        .detach();
                    })
                    .ok();
                    return;
                }
            }
            _ => {}
        }
    }
    this.update(cx, |this, cx| {
        this.fail(
            generation,
            "The browser closed its connection.".to_string(),
            cx,
        );
    })
    .ok();
}

/// The Browser tab: the page Marley's Chromium shows, drawn from the hub's newest frame.
pub struct BrowserView {
    hub: Entity<BrowserHub>,
    focus_handle: FocusHandle,
    /// The frame this tab drew last, which the window may present again.
    current_frame: Option<Arc<RenderImage>>,
    /// The frame drawn before it, freed when the next new frame is drawn.
    previous_frame: Option<Arc<RenderImage>>,
    /// Where the last press in the page was, in the window: an input method opens its window
    /// there, since CDP reports no caret.
    last_press: Option<Point<Pixels>>,
    /// The input method's text not yet committed, shown as the page's composition.
    marked: String,
    _subscriptions: [Subscription; 2],
}

impl fmt::Debug for BrowserView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserView")
            .finish_non_exhaustive()
    }
}

impl BrowserView {
    fn new(hub: Entity<BrowserHub>, window: &Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = [
            cx.observe(&hub, |_, _, cx| cx.notify()),
            cx.subscribe(&hub, |_, _, _: &PageInfoChanged, cx| {
                cx.emit(ItemEvent::UpdateTab);
            }),
        ];
        hub.update(cx, BrowserHub::add_viewer);
        let window_handle = window.window_handle();
        cx.on_release(move |this: &mut Self, cx| {
            for frame in [this.previous_frame.take(), this.current_frame.take()]
                .into_iter()
                .flatten()
            {
                window_handle
                    .update(cx, |_, window, _| window.drop_image(frame).log_err())
                    .ok();
            }
            this.hub.update(cx, BrowserHub::remove_viewer);
        })
        .detach();
        Self {
            hub,
            focus_handle: cx.focus_handle(),
            current_frame: None,
            previous_frame: None,
            last_press: None,
            marked: String::new(),
            _subscriptions: subscriptions,
        }
    }

    /// Records `frame` as drawn now, and frees the frame drawn two paints ago.
    fn drew(&mut self, frame: &Arc<RenderImage>, window: &mut Window) {
        if self
            .current_frame
            .as_ref()
            .is_some_and(|current| current.id == frame.id)
        {
            return;
        }
        if let Some(previous) = self.previous_frame.take() {
            window.drop_image(previous).log_err();
        }
        self.previous_frame = self.current_frame.replace(Arc::clone(frame));
    }

    /// Sends a key the tab receives to the page, after Zed's own bindings had their turn.
    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let modifiers = &keystroke.modifiers;
        let ctrl_alone = modifiers.control && !modifiers.alt && !modifiers.platform;
        if ctrl_alone && keystroke.key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                self.hub.update(cx, |hub, cx| hub.insert_text(&text, cx));
            }
            cx.stop_propagation();
            return;
        }
        let Some(press) = input::key_press(keystroke, event.is_held) else {
            return;
        };
        if ctrl_alone && !modifiers.shift && matches!(keystroke.key.as_str(), "c" | "x") {
            self.hub.update(cx, |hub, cx| hub.copy_selection(press, cx));
            cx.stop_propagation();
            return;
        }
        // A key with text ends a composition: a compose sequence's last key, say.
        if keystroke.key_char.is_some() && !self.marked.is_empty() {
            self.marked.clear();
            self.hub
                .update(cx, |hub, cx| hub.set_composition("", 0..0, cx));
        }
        self.hub.update(cx, |hub, cx| hub.send_key(press, cx));
        cx.stop_propagation();
    }
}

impl EntityInputHandler for BrowserView {
    fn text_for_range(
        &mut self,
        _range: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        None
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        // An input method places its window only where some selection is.
        Some(UTF16Selection {
            range: 0..0,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        (!self.marked.is_empty()).then(|| 0..self.marked.encode_utf16().count())
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.marked.is_empty() {
            let text = std::mem::take(&mut self.marked);
            self.hub.update(cx, |hub, cx| hub.insert_text(&text, cx));
        }
    }

    fn replace_text_in_range(
        &mut self,
        _range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Inserting replaces the page's composition, if there is one.
        self.marked.clear();
        if !text.is_empty() {
            self.hub.update(cx, |hub, cx| hub.insert_text(text, cx));
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let length = new_text.encode_utf16().count();
        let selected = new_selected_range.unwrap_or(length..length);
        self.marked = new_text.to_string();
        self.hub
            .update(cx, |hub, cx| hub.set_composition(new_text, selected, cx));
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let origin = self.last_press.unwrap_or(element_bounds.origin);
        Some(Bounds::new(origin, gpui::size(px(2.), px(20.))))
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl Render for BrowserView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (state, frame, metadata) = {
            let hub = self.hub.read(cx);
            (hub.state.clone(), hub.frame.clone(), hub.metadata)
        };
        if let Some(frame) = &frame {
            self.drew(frame, window);
        }
        let message = match state {
            HubState::Starting => Some((SharedString::new_static("Starting Chromium…"), None)),
            HubState::Connecting => {
                Some((SharedString::new_static("Connecting to Chromium…"), None))
            }
            HubState::Showing => None,
            HubState::Failed(reason) => Some((
                reason,
                Some(SharedString::new_static(
                    "Run \u{201c}marley: open browser\u{201d} to try again.",
                )),
            )),
        };
        div()
            .track_focus(&self.focus_handle)
            .key_context("MarleyBrowser")
            .on_key_down(cx.listener(Self::key_down))
            .relative()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(PageElement {
                hub: self.hub.clone(),
                view: cx.entity(),
                focus_handle: self.focus_handle.clone(),
                frame,
                metadata,
            })
            .when_some(message, |this, (headline, hint)| {
                this.child(
                    v_flex()
                        .absolute()
                        .inset_0()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .px_8()
                        .child(
                            div()
                                .max_w(rems(48.))
                                .text_center()
                                .child(Label::new(headline).color(Color::Muted)),
                        )
                        .when_some(hint, |this, hint| {
                            this.child(Label::new(hint).size(LabelSize::Small).color(Color::Muted))
                        }),
                )
            })
    }
}

impl EventEmitter<ItemEvent> for BrowserView {}

impl Focusable for BrowserView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for BrowserView {
    type Event = ItemEvent;

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.hub
            .read(cx)
            .title()
            .unwrap_or_else(|| SharedString::new_static("Browser"))
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ToolWeb))
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<SharedString> {
        self.hub.read(cx).url()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        None
    }
}

/// The page's frame, drawn from the tab's top left at its own size (a frame from before a
/// resize is neither stretched nor squeezed), the tab's size reported to the hub, and the
/// mouse and the tab's input handler, which send what they get to the page.
struct PageElement {
    hub: Entity<BrowserHub>,
    view: Entity<BrowserView>,
    focus_handle: FocusHandle,
    frame: Option<Arc<RenderImage>>,
    metadata: Option<FrameMetadata>,
}

impl IntoElement for PageElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for PageElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let scale = window.scale_factor();
        self.hub
            .update(cx, |hub, cx| hub.resize(bounds.size, scale, cx));
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.handle_input(
            &self.focus_handle,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );
        let to_page = PageMapping::new(bounds, self.frame.as_deref(), self.metadata, window);
        self.listen(hitbox, to_page, window);
        let Some(frame) = self.frame.clone() else {
            return;
        };
        let size = frame.size(0).to_pixels(window.scale_factor());
        let image_bounds = Bounds {
            origin: bounds.origin,
            size,
        };
        window
            .paint_image(bounds, image_bounds, Corners::default(), frame, 0, false)
            .log_err();
    }
}

/// Where a point in the window falls in the page, in CSS pixels: the offset from the tab's top
/// left, scaled by the frame's viewport width in DIP over the width the frame is drawn at.
#[derive(Clone, Copy)]
struct PageMapping {
    origin: Point<Pixels>,
    scale_x: f64,
    scale_y: f64,
}

impl PageMapping {
    fn new(
        bounds: Bounds<Pixels>,
        frame: Option<&RenderImage>,
        metadata: Option<FrameMetadata>,
        window: &Window,
    ) -> Self {
        let (scale_x, scale_y) = frame.zip(metadata).map_or((1.0, 1.0), |(frame, metadata)| {
            let drawn = frame.size(0).to_pixels(window.scale_factor());
            let width = f64::from(drawn.width);
            let height = f64::from(drawn.height);
            if width > 0.0 && height > 0.0 {
                (
                    metadata.device_width / width,
                    metadata.device_height / height,
                )
            } else {
                (1.0, 1.0)
            }
        });
        Self {
            origin: bounds.origin,
            scale_x,
            scale_y,
        }
    }

    fn map(self, position: Point<Pixels>) -> (f64, f64) {
        let offset = position - self.origin;
        (
            f64::from(offset.x) * self.scale_x,
            f64::from(offset.y) * self.scale_y,
        )
    }
}

impl PageElement {
    /// Sends the mouse to the page: a press in the page focuses the tab first, and while a
    /// button the page got is held, moves and the release reach it wherever the pointer is.
    fn listen(&self, hitbox: &Hitbox, to_page: PageMapping, window: &mut Window) {
        window.on_mouse_event({
            let hub = self.hub.clone();
            let view = self.view.clone();
            let focus_handle = self.focus_handle.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                window.focus(&focus_handle, cx);
                view.update(cx, |view, _| view.last_press = Some(event.position));
                hub.update(cx, |hub, cx| {
                    hub.mouse_press(
                        to_page.map(event.position),
                        event.button,
                        event.click_count,
                        event.modifiers,
                        cx,
                    );
                });
            }
        });
        window.on_mouse_event({
            let hub = self.hub.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || !(hitbox.is_hovered(window) || hub.read(cx).is_holding())
                {
                    return;
                }
                hub.update(cx, |hub, cx| {
                    hub.mouse_release(
                        to_page.map(event.position),
                        event.button,
                        event.click_count,
                        event.modifiers,
                        cx,
                    );
                });
            }
        });
        window.on_mouse_event({
            let hub = self.hub.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || !(hitbox.is_hovered(window) || hub.read(cx).is_holding())
                {
                    return;
                }
                hub.update(cx, |hub, cx| {
                    hub.mouse_move(to_page.map(event.position), event.modifiers, cx);
                });
            }
        });
        window.on_mouse_event({
            let hub = self.hub.clone();
            let hitbox = hitbox.clone();
            move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                // gpui's positive y scrolls up; CDP's scrolls down.
                let delta = event.delta.pixel_delta(px(WHEEL_LINE));
                let delta = (
                    -f64::from(delta.x) * to_page.scale_x,
                    -f64::from(delta.y) * to_page.scale_y,
                );
                hub.update(cx, |hub, cx| {
                    hub.wheel(to_page.map(event.position), delta, event.modifiers, cx);
                });
                cx.stop_propagation();
            }
        });
    }
}

/// Installs `marley::OpenBrowser` on every workspace. [`crate::init`] calls it once.
pub fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenBrowser, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Shows the Browser tab: the workspace's own when it has one, else a new one in the active
/// pane. A hub that stopped starts again.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let hub = BrowserHub::global(cx);
    hub.update(cx, |hub, cx| {
        if matches!(hub.state, HubState::Failed(_)) {
            hub.start(cx);
        }
    });
    if let Some(view) = workspace.item_of_type::<BrowserView>(cx) {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let view = cx.new(|cx| BrowserView::new(hub, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}
