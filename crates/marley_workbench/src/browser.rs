//! The Browser tab (prong 3 B0a): the page Marley's own Chromium renders, in the main area.
//!
//! One [`BrowserHub`] per app owns the connection to the browser and the page the Browser tabs
//! show, so the user and an agent attached to the same Chromium see one page. The hub starts
//! Chromium when no Marley Chromium answers, streams the page as screencast frames while a tab
//! shows it, and lays the page out at the tab's size. [`BrowserView`] is the tab: it draws the
//! newest frame and frees each frame from the window's GPU atlas two paints after it was first
//! drawn, as Zed's screen-share view does, since the window may present the last frame again.
//!
//! Input (B0b, #489) goes to the page over CDP while the page has the focus: the mouse and the
//! wheel from `PageElement`'s listeners, at CSS pixels taken from the frame's metadata; keys
//! from the tab's `key_down`, as `marley_browser::input` maps them; composed and input-method
//! text through the tab's input handler; and the system clipboard, which headless Chromium does
//! not share, through Marley.
//!
//! Navigation (B1a, #490) is the toolbar over the page: back, forward, reload or stop, and the
//! address bar, a Zed single-line editor that shows the page's URL whenever it does not have the
//! focus. The hub follows the main frame's loading, its commits and its history, and the page's
//! JavaScript dialogs, which headless Chromium does not draw: the tab draws each as a card over
//! the page, whose script waits until the card is answered.

use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use editor::Editor;
use editor::actions::SelectAll;
use futures::StreamExt as _;
use futures::channel::{mpsc, oneshot};
use gpui::{
    App, AsyncApp, BackgroundExecutor, Bounds, ClipboardItem, Corners, DispatchPhase, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, Global, GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, KeyDownEvent,
    LayoutId, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    RenderImage, ScrollWheelEvent, Size, Style, Subscription, Task, UTF16Selection, WeakEntity,
    relative,
};
use marley_browser::cdp::{self, CdpError, Connection, Event};
use marley_browser::input::{self, KeyPress};
use marley_browser::observe::{ConsoleEntry, ConsoleLog, NetworkEntry, NetworkLog};
use marley_browser::page::{
    DialogKind, FrameMetadata, JavaScriptDialog, NavigationHistory, Page, ScreencastFrame,
    TargetInfo,
};
use marley_browser::snapshot::RefTarget;
use marley_browser::{address, frame, service};
use serde_json::Value;
use ui::prelude::*;
use ui::{AlertModal, Chip, Tooltip};
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::{MultiWorkspace, Workspace};

use crate::{
    AnswerDialog, BrowserBack, BrowserForward, BrowserReload, DismissDialog, FocusAddressBar,
    GoToAddress, OpenBrowser, RestoreAddress,
};

/// How many times, a tenth of a second apart, a start waits for Chromium to write its endpoint
/// and answer.
const START_POLLS: u32 = 150;

/// The wait between two of those tries.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// The CSS pixels a wheel line scrolls: three lines a detent make the 100 a mouse wheel turns in
/// Chrome.
const WHEEL_LINE: f32 = 100.0 / 3.0;

/// How long the Agent chip stays after an agent's action ends (#492).
const AGENT_CHIP: Duration = Duration::from_secs(5);

/// How long a navigation an agent asked for may take to load before its call answers.
const LOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// A cross-site iframe of the page, attached with a session of its own (#492).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iframe {
    /// Its session.
    pub session: String,
    /// The session it was attached from: the page's, or an outer iframe's.
    pub parent_session: String,
    /// Its frame's id, which is its target's.
    pub frame_id: String,
    /// Its URL.
    pub url: String,
}

/// What an agent did last in the page, for the Agent chip.
#[derive(Debug, Clone)]
struct AgentAction {
    text: SharedString,
    /// When it ended; none while it runs.
    ended: Option<Instant>,
}

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

/// What the hub tells the Browser tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserEvent {
    /// The page's title or URL changed, or the address it is going to.
    PageInfoChanged,
    /// The page opened a JavaScript dialog.
    DialogOpened,
    /// The page's dialog closed, answered here or by another client.
    DialogClosed,
}

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
    /// Where a navigation the tab asked for goes, from the ask until it commits or ends.
    pending_url: Option<SharedString>,
    /// Whether the main frame is loading.
    loading: bool,
    history: Option<NavigationHistory>,
    /// The JavaScript dialog the page waits on.
    dialog: Option<JavaScriptDialog>,
    /// The page's cross-site iframes (#492).
    iframes: Vec<Iframe>,
    /// What the page logged since the connection began.
    console: ConsoleLog,
    /// What the page fetched since the connection began.
    network: NetworkLog,
    /// What the refs of the newest snapshot name.
    refs: Vec<RefTarget>,
    /// The agent's last action, for the Agent chip.
    agent: Option<AgentAction>,
    /// Calls waiting for the main frame to stop loading.
    load_waiters: Vec<oneshot::Sender<()>>,
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
            .field("loading", &self.loading)
            .field("viewers", &self.viewers)
            .field("screencasting", &self.screencasting)
            .finish_non_exhaustive()
    }
}

impl EventEmitter<BrowserEvent> for BrowserHub {}

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
                pending_url: None,
                loading: false,
                history: None,
                dialog: None,
                iframes: Vec::new(),
                console: ConsoleLog::default(),
                network: NetworkLog::default(),
                refs: Vec::new(),
                agent: None,
                load_waiters: Vec::new(),
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

    /// What the address bar shows: where the page is going, else where it is.
    #[must_use]
    pub fn address(&self) -> Option<SharedString> {
        self.pending_url.clone().or_else(|| self.url.clone())
    }

    /// Whether the main frame is loading.
    #[must_use]
    pub const fn is_loading(&self) -> bool {
        self.loading
    }

    /// Whether the page's history has an entry `offset` steps from the one it shows: -1 back,
    /// 1 forward.
    #[must_use]
    pub fn can_go(&self, offset: isize) -> bool {
        self.page.is_some()
            && self
                .history
                .as_ref()
                .is_some_and(|history| history.entry_at(offset).is_some())
    }

    /// The JavaScript dialog the page waits on.
    #[must_use]
    pub const fn dialog(&self) -> Option<&JavaScriptDialog> {
        self.dialog.as_ref()
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
        self.forget_page(cx);
        if self.title.take().is_some() | self.url.take().is_some() {
            cx.emit(BrowserEvent::PageInfoChanged);
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
                    // What the agent tools read comes on before the hub shows the page, since a
                    // navigation sent sooner loads before the network is watched (#492).
                    page.observe(page.session_id()).await.log_err();
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
        self.forget_page(cx);
        cx.notify();
    }

    /// Drops what the hub knew of a page that is gone: where it was going, its loading, its
    /// history and its dialog.
    fn forget_page(&mut self, cx: &mut Context<Self>) {
        self.loading = false;
        self.history = None;
        self.iframes.clear();
        self.console = ConsoleLog::default();
        self.network = NetworkLog::default();
        self.refs.clear();
        // A waiting call hears its waiter drop, and answers.
        self.load_waiters.clear();
        if self.pending_url.take().is_some() {
            cx.emit(BrowserEvent::PageInfoChanged);
        }
        self.leave_dialog(cx);
    }

    fn attached(&mut self, generation: u64, page: Page, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        self.page = Some(page.clone());
        self.state = HubState::Showing;
        cx.notify();
        if let Some(viewport) = self.viewport {
            let page = page.clone();
            cx.spawn(async move |_, _| {
                page.set_viewport(viewport.width, viewport.height, viewport.scale)
                    .await
                    .log_err();
            })
            .detach();
        }
        self.sync_screencast(cx);
        cx.spawn(async move |this, cx| {
            let history = page.history().await.log_err();
            this.update(cx, |this, cx| this.navigated(generation, None, history, cx))
                .ok();
        })
        .detach();
    }

    /// Navigates the page to `url`, which the address bar shows until the navigation commits or
    /// ends. A dialog the page waits on is answered with Cancel first, as Chrome closes a page's
    /// dialog when the page is left.
    pub fn navigate(&mut self, url: String, cx: &mut Context<Self>) {
        self.navigate_task(url, cx).detach();
    }

    /// Navigates as [`BrowserHub::navigate`] does, in a task that ends once the page has loaded,
    /// or 15 seconds later, with Chromium's reason when the navigation failed (#492).
    pub fn navigate_task(
        &mut self,
        url: String,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let Some(page) = self.page.clone() else {
            return Task::ready(Err("the browser has no page".to_string()));
        };
        let dismiss = self.leave_dialog(cx);
        self.pending_url = Some(SharedString::from(&url));
        cx.emit(BrowserEvent::PageInfoChanged);
        cx.notify();
        let generation = self.generation;
        let loaded = self.load_waiter();
        cx.spawn(async move |this, cx| {
            if dismiss {
                page.answer_dialog(false, None).await.log_err();
            }
            let result = page.navigate(&url).await;
            if let Err(error) = &result {
                log::info!("browser: a navigation ended early: {error}");
            }
            // A navigation outlives a call that timed out, and the address bar keeps its URL.
            if !matches!(result, Err(CdpError::Timeout(_))) {
                this.update(cx, |this, cx| this.navigation_ended(generation, &url, cx))
                    .ok();
            }
            result.map_err(|error| error.to_string())?;
            wait_for_load(loaded, cx).await;
            Ok(())
        })
    }

    /// A receiver that hears when the main frame next stops loading, or moves within its
    /// document; it is dropped if the page goes.
    fn load_waiter(&mut self) -> oneshot::Receiver<()> {
        let (sender, receiver) = oneshot::channel();
        self.load_waiters.push(sender);
        receiver
    }

    /// Tells the calls waiting for a load that it is over.
    fn loaded(&mut self) {
        for waiter in self.load_waiters.drain(..) {
            // A call that stopped waiting dropped its end.
            waiter.send(()).ok();
        }
    }

    fn navigation_ended(&mut self, generation: u64, url: &str, cx: &mut Context<Self>) {
        if generation == self.generation && self.pending_url.as_deref() == Some(url) {
            self.pending_url = None;
            cx.emit(BrowserEvent::PageInfoChanged);
            cx.notify();
        }
    }

    /// Moves the page `offset` entries through its history: -1 back, 1 forward.
    pub fn go(&mut self, offset: isize, cx: &mut Context<Self>) {
        self.go_task(offset, cx).detach();
    }

    /// Moves as [`BrowserHub::go`] does, in a task that ends once the page has loaded, or
    /// 15 seconds later (#492).
    pub fn go_task(&mut self, offset: isize, cx: &mut Context<Self>) -> Task<Result<(), String>> {
        let Some(page) = self.page.clone() else {
            return Task::ready(Err("the browser has no page".to_string()));
        };
        let Some(history) = self.history.as_mut() else {
            return Task::ready(Err("the page's history is not read yet".to_string()));
        };
        let Some((index, id)) = history
            .entry_at(offset)
            .map(|(index, entry)| (index, entry.id))
        else {
            return Task::ready(Err("the page's history has no entry there".to_string()));
        };
        // A second press before the page reports where it went goes on from here.
        history.current_index = index;
        let dismiss = self.leave_dialog(cx);
        cx.notify();
        let loaded = self.load_waiter();
        cx.spawn(async move |_, cx| {
            if dismiss {
                page.answer_dialog(false, None).await.log_err();
            }
            page.go_to_history_entry(id)
                .await
                .map_err(|error| error.to_string())?;
            wait_for_load(loaded, cx).await;
            Ok(())
        })
    }

    /// Loads the page again.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(page) = self.page.clone() else {
            return;
        };
        let dismiss = self.leave_dialog(cx);
        cx.spawn(async move |_, _| {
            if dismiss {
                page.answer_dialog(false, None).await.log_err();
            }
            page.reload().await.log_err();
        })
        .detach();
    }

    /// Stops the page's loading.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        let Some(page) = self.page.clone() else {
            return;
        };
        cx.spawn(async move |_, _| {
            page.stop_loading().await.log_err();
        })
        .detach();
    }

    /// Answers the page's dialog: OK when `accept`, else Cancel, with `prompt_text` as a
    /// `prompt`'s answer.
    pub fn answer_dialog(
        &mut self,
        accept: bool,
        prompt_text: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.page.clone() else {
            return;
        };
        if !self.leave_dialog(cx) {
            return;
        }
        cx.spawn(async move |_, _| {
            page.answer_dialog(accept, prompt_text.as_deref())
                .await
                .log_err();
        })
        .detach();
    }

    /// Forgets the page's dialog, if it has one, and says whether it had.
    fn leave_dialog(&mut self, cx: &mut Context<Self>) -> bool {
        if self.dialog.take().is_none() {
            return false;
        }
        cx.emit(BrowserEvent::DialogClosed);
        cx.notify();
        true
    }

    fn dialog_opened(&mut self, generation: u64, dialog: JavaScriptDialog, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        self.dialog = Some(dialog);
        cx.emit(BrowserEvent::DialogOpened);
        cx.notify();
    }

    fn dialog_closed(&mut self, generation: u64, cx: &mut Context<Self>) {
        if generation == self.generation {
            self.leave_dialog(cx);
        }
    }

    /// The main frame started or stopped loading.
    fn loading_changed(
        &mut self,
        generation: u64,
        frame_id: Option<&str>,
        loading: bool,
        cx: &mut Context<Self>,
    ) {
        // The main frame's id is the page's target id.
        if generation == self.generation && self.is_page(frame_id) && self.loading != loading {
            self.loading = loading;
            if !loading {
                self.loaded();
            }
            cx.notify();
        }
    }

    /// The main frame committed a navigation, to `url` when the event said where: the address
    /// bar's navigation is over, and back and forward go from the new history.
    fn navigated(
        &mut self,
        generation: u64,
        url: Option<String>,
        history: Option<NavigationHistory>,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        if history.is_some() {
            self.history = history;
        }
        let arrived = url.is_some() && self.pending_url.take().is_some();
        let moved = match url.map(SharedString::from) {
            Some(url) if self.url.as_ref() != Some(&url) => {
                self.url = Some(url);
                true
            }
            _ => false,
        };
        if arrived || moved {
            cx.emit(BrowserEvent::PageInfoChanged);
        }
        cx.notify();
    }

    /// The page, while the hub shows it (#492).
    #[must_use]
    pub fn page(&self) -> Option<Page> {
        self.page.clone()
    }

    /// The page's cross-site iframes.
    #[must_use]
    pub fn iframes(&self) -> Vec<Iframe> {
        self.iframes.clone()
    }

    /// What the page logged, oldest first.
    #[must_use]
    pub fn console_entries(&self) -> Vec<ConsoleEntry> {
        self.console.entries().cloned().collect()
    }

    /// What the page fetched, oldest first.
    #[must_use]
    pub fn network_entries(&self) -> Vec<NetworkEntry> {
        self.network.entries().cloned().collect()
    }

    /// Keeps the refs of a new snapshot, in place of the last one's.
    pub fn set_refs(&mut self, refs: Vec<RefTarget>) {
        self.refs = refs;
    }

    /// What the ref `id` of the newest snapshot names.
    #[must_use]
    pub fn ref_target(&self, id: &str) -> Option<RefTarget> {
        self.refs.iter().find(|target| target.id == id).cloned()
    }

    /// Shows the Agent chip with what an agent is doing, until it ends.
    pub fn agent_started(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.agent = Some(AgentAction {
            text: text.into(),
            ended: None,
        });
        cx.notify();
    }

    /// Shows the Agent chip with what an agent did, for five seconds more.
    pub fn agent_ended(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.agent = Some(AgentAction {
            text: text.into(),
            ended: Some(Instant::now()),
        });
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AGENT_CHIP).await;
            this.update(cx, |this, cx| {
                let over = this.agent.as_ref().is_some_and(|action| {
                    action
                        .ended
                        .is_some_and(|ended| ended.elapsed() >= AGENT_CHIP)
                });
                if over {
                    this.agent = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// What the Agent chip says, while it shows.
    fn agent_chip(&self) -> Option<SharedString> {
        self.agent.as_ref().map(|action| action.text.clone())
    }

    /// A cross-site iframe attached from `parent_session`: its tools' domains come on, and its
    /// own iframes attach in turn.
    fn iframe_attached(
        &mut self,
        generation: u64,
        parent_session: String,
        params: &Value,
        cx: &Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        let info = params.get("targetInfo");
        let kind = info
            .and_then(|info| info.get("type"))
            .and_then(Value::as_str);
        let (Some("iframe"), Some(session), Some(frame_id), Some(page)) = (
            kind,
            params.get("sessionId").and_then(Value::as_str),
            info.and_then(|info| info.get("targetId"))
                .and_then(Value::as_str),
            self.page.clone(),
        ) else {
            return;
        };
        let url = info
            .and_then(|info| info.get("url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        self.iframes.retain(|iframe| iframe.session != session);
        self.iframes.push(Iframe {
            session: session.to_string(),
            parent_session,
            frame_id: frame_id.to_string(),
            url,
        });
        let session = session.to_string();
        cx.spawn(async move |_, _| {
            page.observe(&session).await.log_err();
        })
        .detach();
    }

    fn iframe_detached(&mut self, generation: u64, params: &Value) {
        if generation != self.generation {
            return;
        }
        if let Some(session) = params.get("sessionId").and_then(Value::as_str) {
            self.iframes.retain(|iframe| iframe.session != session);
        }
    }

    fn observed(&mut self, generation: u64, method: &str, params: &Value) {
        if generation != self.generation {
            return;
        }
        if method.starts_with("Network.") {
            self.network.apply(method, params);
        } else {
            self.console.apply(method, params);
        }
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
        if generation != self.generation {
            return;
        }
        if let Some(iframe) = self
            .iframes
            .iter_mut()
            .find(|iframe| iframe.frame_id == info.target_id)
        {
            iframe.url = info.url;
            return;
        }
        if self
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
            cx.emit(BrowserEvent::PageInfoChanged);
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
            "Page.domContentEventFired" | "Page.loadEventFired" => {
                refresh_info(&this, generation, cx).await;
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
            _ => follow_navigation(&this, event, generation, cx).await,
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

/// Follows the page's navigation for the start `generation`: its main frame's commits and
/// loading, and the JavaScript dialogs it opens and closes.
async fn follow_navigation(
    this: &WeakEntity<BrowserHub>,
    event: Event,
    generation: u64,
    cx: &mut AsyncApp,
) {
    match event.method.as_str() {
        // A commit in the main frame, which has no parent. A page that failed to load commits
        // Chromium's error page, which names the URL it could not load.
        "Page.frameNavigated" => {
            if event.params.pointer("/frame/parentId").is_none() {
                let url = ["/frame/unreachableUrl", "/frame/url"]
                    .into_iter()
                    .find_map(|pointer| event.params.pointer(pointer).and_then(Value::as_str))
                    .map(str::to_string);
                refresh_history(this, generation, url, cx).await;
            }
        }
        // A fragment or the history API moved the main frame within its document.
        "Page.navigatedWithinDocument" => {
            let frame = event.params.get("frameId").and_then(Value::as_str);
            let main = this
                .read_with(cx, |this, _| this.is_page(frame))
                .unwrap_or(false);
            if main {
                let url = event
                    .params
                    .get("url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                // A move within the document is all the load a waiting call will see.
                this.update(cx, |this, _| this.loaded()).ok();
                refresh_history(this, generation, url, cx).await;
                refresh_info(this, generation, cx).await;
            }
        }
        "Page.frameStartedLoading" | "Page.frameStoppedLoading" => {
            let loading = event.method == "Page.frameStartedLoading";
            let frame = event.params.get("frameId").and_then(Value::as_str);
            this.update(cx, |this, cx| {
                this.loading_changed(generation, frame, loading, cx);
            })
            .ok();
        }
        "Page.javascriptDialogOpening" => {
            match serde_json::from_value::<JavaScriptDialog>(event.params) {
                Ok(dialog) => {
                    this.update(cx, |this, cx| this.dialog_opened(generation, dialog, cx))
                        .ok();
                }
                Err(error) => log::warn!("browser: a dialog the page opened: {error}"),
            }
        }
        "Page.javascriptDialogClosed" => {
            this.update(cx, |this, cx| this.dialog_closed(generation, cx))
                .ok();
        }
        _ => follow_observed(this, &event, generation, cx),
    }
}

/// Follows what the agent tools read (#492): console messages, errors and the browser's log, the
/// network, and the cross-site iframes that attach and detach.
fn follow_observed(
    this: &WeakEntity<BrowserHub>,
    event: &Event,
    generation: u64,
    cx: &mut AsyncApp,
) {
    let method = event.method.as_str();
    match method {
        "Runtime.consoleAPICalled"
        | "Runtime.exceptionThrown"
        | "Log.entryAdded"
        | "Network.requestWillBeSent"
        | "Network.responseReceived"
        | "Network.loadingFinished"
        | "Network.loadingFailed" => {
            this.update(cx, |this, _| {
                this.observed(generation, method, &event.params);
            })
            .ok();
        }
        "Target.attachedToTarget" => {
            if let Some(parent) = event.session_id.clone() {
                this.update(cx, |this, cx| {
                    this.iframe_attached(generation, parent, &event.params, cx);
                })
                .ok();
            }
        }
        "Target.detachedFromTarget" => {
            this.update(cx, |this, _| {
                this.iframe_detached(generation, &event.params);
            })
            .ok();
        }
        _ => {}
    }
}

/// Reads the page's title and URL into the hub.
async fn refresh_info(this: &WeakEntity<BrowserHub>, generation: u64, cx: &mut AsyncApp) {
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

/// Reads the page's history into the hub after its main frame went to `url`.
async fn refresh_history(
    this: &WeakEntity<BrowserHub>,
    generation: u64,
    url: Option<String>,
    cx: &mut AsyncApp,
) {
    let page = this
        .read_with(cx, |this, _| this.page.clone())
        .ok()
        .flatten();
    let Some(page) = page else {
        return;
    };
    let history = page.history().await.log_err();
    this.update(cx, |this, cx| this.navigated(generation, url, history, cx))
        .ok();
}

/// Waits until `loaded` hears the load end, or [`LOAD_TIMEOUT`] passes.
async fn wait_for_load(loaded: oneshot::Receiver<()>, cx: &AsyncApp) {
    let timeout = cx.background_executor().timer(LOAD_TIMEOUT);
    futures::future::select(loaded, timeout).await;
}

/// The Browser tab: the page Marley's Chromium shows, drawn from the hub's newest frame, under
/// a toolbar with the address bar.
pub struct BrowserView {
    hub: Entity<BrowserHub>,
    /// The page's focus: keys and text go to the page while it holds the focus itself.
    focus_handle: FocusHandle,
    address_bar: Entity<Editor>,
    /// The focus of a dialog card with no field.
    dialog_focus: FocusHandle,
    /// A `prompt` dialog's field.
    prompt_field: Entity<Editor>,
    /// The frame this tab drew last, which the window may present again.
    current_frame: Option<Arc<RenderImage>>,
    /// The frame drawn before it, freed when the next new frame is drawn.
    previous_frame: Option<Arc<RenderImage>>,
    /// Where the last press in the page was, in the window: an input method opens its window
    /// there, since CDP reports no caret.
    last_press: Option<Point<Pixels>>,
    /// The input method's text not yet committed, shown as the page's composition.
    marked: String,
    _subscriptions: [Subscription; 4],
}

impl fmt::Debug for BrowserView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserView")
            .finish_non_exhaustive()
    }
}

impl BrowserView {
    fn new(hub: Entity<BrowserHub>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let address_bar = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search or enter an address", window, cx);
            editor
        });
        let prompt_field = cx.new(|cx| Editor::single_line(window, cx));
        let focus_handle = cx.focus_handle();
        let subscriptions = [
            cx.observe(&hub, |_, _, cx| cx.notify()),
            cx.subscribe_in(&hub, window, |this, _, event, window, cx| {
                this.hub_event(*event, window, cx);
            }),
            // The address bar shows the page's URL again once the focus leaves it.
            cx.on_focus_out(
                &address_bar.focus_handle(cx),
                window,
                |this, _, window, cx| this.show_address(window, cx),
            ),
            // While the page waits on a dialog, the page's focus goes to the dialog.
            cx.on_focus(&focus_handle, window, |this, window, cx| {
                if this.hub.read(cx).dialog().is_some() {
                    this.focus_dialog(window, cx);
                }
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
        let view = Self {
            hub,
            focus_handle,
            address_bar,
            dialog_focus: cx.focus_handle(),
            prompt_field,
            current_frame: None,
            previous_frame: None,
            last_press: None,
            marked: String::new(),
            _subscriptions: subscriptions,
        };
        view.show_address(window, cx);
        view
    }

    fn hub_event(&self, event: BrowserEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            BrowserEvent::PageInfoChanged => {
                cx.emit(ItemEvent::UpdateTab);
                self.show_address(window, cx);
            }
            BrowserEvent::DialogOpened => {
                let default = self
                    .hub
                    .read(cx)
                    .dialog()
                    .map(|dialog| dialog.default_prompt.clone())
                    .unwrap_or_default();
                self.prompt_field.update(cx, |field, cx| {
                    field.set_text(default, window, cx);
                    field.select_all(&SelectAll, window, cx);
                });
                // A dialog takes the focus from inside the tab only: a page elsewhere waits
                // until the tab has the focus again.
                if self.focus_handle.contains_focused(window, cx) {
                    self.focus_dialog(window, cx);
                }
            }
            BrowserEvent::DialogClosed => {
                if self.dialog_focus.contains_focused(window, cx)
                    || self
                        .prompt_field
                        .focus_handle(cx)
                        .contains_focused(window, cx)
                {
                    window.focus(&self.focus_handle, cx);
                }
            }
        }
    }

    /// Shows where the page is, or is going, in the address bar, unless the user is typing there.
    fn show_address(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .address_bar
            .focus_handle(cx)
            .contains_focused(window, cx)
        {
            return;
        }
        let address = self.hub.read(cx).address().unwrap_or_default();
        self.address_bar.update(cx, |editor, cx| {
            if editor.text(cx) != *address {
                editor.set_text(address.to_string(), window, cx);
            }
        });
    }

    /// Gives the focus to the page's dialog: its field for a `prompt`, else the card.
    fn focus_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let is_prompt = self
            .hub
            .read(cx)
            .dialog()
            .is_some_and(|dialog| dialog.kind == DialogKind::Prompt);
        if is_prompt {
            window.focus(&self.prompt_field.focus_handle(cx), cx);
        } else {
            window.focus(&self.dialog_focus, cx);
        }
    }

    fn focus_address_bar(
        &mut self,
        _: &FocusAddressBar,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.address_bar.focus_handle(cx), cx);
        self.address_bar
            .update(cx, |editor, cx| editor.select_all(&SelectAll, window, cx));
    }

    fn go_to_address(&mut self, _: &GoToAddress, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.address_bar.read(cx).text(cx);
        if let Some(url) = address::url_for(&text) {
            self.hub.update(cx, |hub, cx| hub.navigate(url, cx));
        }
        self.restore_address(&RestoreAddress, window, cx);
    }

    fn restore_address(&mut self, _: &RestoreAddress, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.show_address(window, cx);
    }

    fn back(&mut self, _: &BrowserBack, _: &mut Window, cx: &mut Context<Self>) {
        self.hub.update(cx, |hub, cx| hub.go(-1, cx));
    }

    fn forward(&mut self, _: &BrowserForward, _: &mut Window, cx: &mut Context<Self>) {
        self.hub.update(cx, |hub, cx| hub.go(1, cx));
    }

    fn reload(&mut self, _: &BrowserReload, _: &mut Window, cx: &mut Context<Self>) {
        self.hub.update(cx, BrowserHub::reload);
    }

    fn answer_dialog(&mut self, _: &AnswerDialog, _: &mut Window, cx: &mut Context<Self>) {
        let is_prompt = self
            .hub
            .read(cx)
            .dialog()
            .is_some_and(|dialog| dialog.kind == DialogKind::Prompt);
        let prompt_text = is_prompt.then(|| self.prompt_field.read(cx).text(cx));
        self.hub
            .update(cx, |hub, cx| hub.answer_dialog(true, prompt_text, cx));
    }

    fn dismiss_dialog(&mut self, _: &DismissDialog, _: &mut Window, cx: &mut Context<Self>) {
        self.hub
            .update(cx, |hub, cx| hub.answer_dialog(false, None, cx));
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

    /// Sends a key the page receives to the page, after Zed's own bindings had their turn. A
    /// key typed in the address bar or a dialog passes here on its way up, and stays in Marley.
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            return;
        }
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

impl BrowserView {
    fn render_toolbar(&self, cx: &Context<Self>) -> impl IntoElement {
        let hub = self.hub.read(cx);
        let showing = hub.state == HubState::Showing;
        let loading = hub.is_loading();
        let colors = cx.theme().colors();
        let reload_or_stop = if loading {
            IconButton::new("browser-stop", IconName::Close)
                .tooltip(Tooltip::text("Stop"))
                .on_click(cx.listener(|this, _, _, cx| this.hub.update(cx, BrowserHub::stop)))
        } else {
            IconButton::new("browser-reload", IconName::RotateCw)
                .disabled(!showing)
                .tooltip(Tooltip::for_action_title("Reload", &BrowserReload))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.reload(&BrowserReload, window, cx);
                }))
        };
        h_flex()
            .relative()
            .flex_none()
            .w_full()
            .gap_1()
            .px_1p5()
            .py_1()
            .border_b_1()
            .border_color(colors.border_variant)
            .bg(colors.toolbar_background)
            .child(
                IconButton::new("browser-back", IconName::ArrowLeft)
                    .disabled(!hub.can_go(-1))
                    .tooltip(Tooltip::for_action_title("Back", &BrowserBack))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.back(&BrowserBack, window, cx);
                    })),
            )
            .child(
                IconButton::new("browser-forward", IconName::ArrowRight)
                    .disabled(!hub.can_go(1))
                    .tooltip(Tooltip::for_action_title("Forward", &BrowserForward))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.forward(&BrowserForward, window, cx);
                    })),
            )
            .child(reload_or_stop)
            .child(
                div()
                    .key_context("MarleyAddressBar")
                    .on_action(cx.listener(Self::go_to_address))
                    .on_action(cx.listener(Self::restore_address))
                    .flex_1()
                    .min_w_0()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(self.address_bar.clone()),
            )
            // What an agent does in the page, while it does it and a moment after (#492).
            .when_some(hub.agent_chip(), |this, action| {
                this.child(
                    h_flex()
                        .flex_none()
                        .max_w(rems(24.))
                        .gap_1()
                        .child(
                            Chip::new("Agent")
                                .icon(IconName::Sparkle)
                                .icon_color(Color::Accent)
                                .label_color(Color::Accent),
                        )
                        .child(
                            Label::new(action)
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                                .truncate(),
                        ),
                )
            })
            // Over the toolbar's edge, so a load never moves the page.
            .when(loading, |this| {
                this.child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(2.))
                        .bg(colors.text_accent),
                )
            })
    }

    /// The page's dialog as a card over the page, which takes no input while it shows.
    fn render_dialog(
        &self,
        dialog: &JavaScriptDialog,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let (title, message, accept) = if dialog.kind == DialogKind::BeforeUnload {
            (
                SharedString::new_static("Leave this page?"),
                SharedString::new_static("Changes you made may not be saved."),
                "Leave",
            )
        } else {
            let asker = dialog.origin().map_or_else(
                || "This page says".to_string(),
                |origin| format!("{origin} says"),
            );
            (
                SharedString::from(asker),
                SharedString::from(&dialog.message),
                "OK",
            )
        };
        let colors = cx.theme().colors();
        div()
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .justify_center()
            .items_start()
            .pt_16()
            .child(
                AlertModal::new("browser-dialog")
                    .key_context("MarleyBrowserDialog")
                    .track_focus(&self.dialog_focus)
                    .on_action(cx.listener(Self::answer_dialog))
                    .on_action(cx.listener(Self::dismiss_dialog))
                    .width(rems(28.))
                    .title(title)
                    .child(Label::new(message))
                    .when(dialog.kind == DialogKind::Prompt, |this| {
                        this.child(
                            div()
                                .mt_2()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .border_1()
                                .border_color(colors.border)
                                .bg(colors.editor_background)
                                .child(self.prompt_field.clone()),
                        )
                    })
                    .footer(
                        h_flex()
                            .p_3()
                            .justify_end()
                            .gap_1()
                            .when(dialog.kind != DialogKind::Alert, |this| {
                                this.child(
                                    Button::new("browser-dialog-cancel", "Cancel")
                                        .color(Color::Muted)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.dismiss_dialog(&DismissDialog, window, cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("browser-dialog-accept", accept)
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.answer_dialog(&AnswerDialog, window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

impl Render for BrowserView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (state, frame, metadata, dialog) = {
            let hub = self.hub.read(cx);
            (
                hub.state.clone(),
                hub.frame.clone(),
                hub.metadata,
                hub.dialog.clone(),
            )
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
        let toolbar = self.render_toolbar(cx);
        let dialog = dialog.map(|dialog| self.render_dialog(&dialog, cx));
        v_flex()
            .track_focus(&self.focus_handle)
            .key_context("MarleyBrowser")
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(Self::focus_address_bar))
            .on_action(cx.listener(Self::back))
            .on_action(cx.listener(Self::forward))
            .on_action(cx.listener(Self::reload))
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(toolbar)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
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
                                    this.child(
                                        Label::new(hint).size(LabelSize::Small).color(Color::Muted),
                                    )
                                }),
                        )
                    })
                    .children(dialog),
            )
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

/// Brings a Browser tab to the front for an agent (#492): the first one in any window, shown in
/// its pane, or else a new one in the active workspace's active pane. The focus stays where the
/// user types.
pub(crate) fn show_for_agent(cx: &mut App) {
    let hub = BrowserHub::global(cx);
    let windows: Vec<_> = cx
        .windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>())
        .collect();
    for window in &windows {
        let shown = window
            .update(cx, |multi_workspace, window, cx| {
                multi_workspace.workspaces().any(|workspace| {
                    workspace.update(cx, |workspace, cx| {
                        let Some(view) = workspace.item_of_type::<BrowserView>(cx) else {
                            return false;
                        };
                        workspace.activate_item(&view, false, false, window, cx);
                        true
                    })
                })
            })
            .unwrap_or(false);
        if shown {
            return;
        }
    }
    let active = cx
        .active_window()
        .and_then(|window| window.downcast::<MultiWorkspace>())
        .or_else(|| windows.first().copied());
    let Some(window) = active else {
        log::warn!("browser: no window to show an agent's page in");
        return;
    };
    window
        .update(cx, |multi_workspace, window, cx| {
            let workspace = multi_workspace.workspace().clone();
            workspace.update(cx, |workspace, cx| {
                let view = cx.new(|cx| BrowserView::new(hub, window, cx));
                workspace.add_item_to_active_pane(Box::new(view), None, false, window, cx);
            });
        })
        .log_err();
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
