//! The Browser tabs (prong 3 B0a, B1b): the pages Marley's own Chromium renders, in the main
//! area.
//!
//! One [`BrowserHub`] per app owns the connection to the browser and every page of it, so the
//! user and an agent attached to the same Chromium see the same pages. The hub starts Chromium
//! when no Marley Chromium answers, attaches to each page as the browser reports it, streams a
//! page as screencast frames while a tab draws it, and lays the page out at its tab's size. Each
//! [`BrowserView`] is one page's tab (#493): it draws the page's newest frame and frees each
//! frame from the window's GPU atlas two paints after it was first drawn, as Zed's screen-share
//! view does, since the window may present the last frame again.
//!
//! A page a page opens gets a tab beside its opener's, with the focus, as in a browser. A page an
//! agent or another client opens gets a tab that leaves the focus where it is: behind the tab in
//! front of a pane that has the focus, and, while no Browser tab is open and the pane with the
//! focus shows other work, in a new pane beside that one. Closing a tab closes its page, moving
//! it to another pane does not, and a page that closes closes its tab.
//!
//! Input (B0b, #489) goes to a page over CDP while its tab's page has the focus: the mouse and
//! the wheel from `PageElement`'s listeners, at CSS pixels taken from the frame's metadata; keys
//! from the tab's `key_down`, as `marley_browser::input` maps them; composed and input-method
//! text through the tab's input handler; and the system clipboard, which headless Chromium does
//! not share, through Marley.
//!
//! Navigation (B1a, #490) is the toolbar over the page: back, forward, reload or stop, and the
//! address bar, a Zed single-line editor that shows the page's URL whenever it does not have the
//! focus. The hub follows each main frame's loading, its commits and its history, and each
//! page's JavaScript dialogs, which headless Chromium does not draw: the tab draws each as a card
//! over the page, whose script waits until the card is answered.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use editor::Editor;
use editor::actions::SelectAll;
use futures::StreamExt as _;
use futures::channel::{mpsc, oneshot};
use gpui::{
    Anchor, AnyWindowHandle, App, AsyncApp, BackgroundExecutor, Bounds, ClipboardItem, Corners,
    DismissEvent, DispatchPhase, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, Global, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, KeyDownEvent, LayoutId, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderImage, ScrollWheelEvent,
    Size, Style, Subscription, Task, UTF16Selection, WeakEntity, WindowHandle, anchored, deferred,
    point, relative,
};
use marley_browser::cdp::{self, CdpError, Connection, Event};
use marley_browser::input::{self, KeyPress};
use marley_browser::observe::{ConsoleEntry, ConsoleLog, NetworkEntry, NetworkLog, redact_url};
use marley_browser::page::{
    DialogKind, FrameMetadata, JavaScriptDialog, NavigationHistory, Page, ScreencastFrame,
    TargetInfo,
};
use marley_browser::pick::{Listener, PageBox, PickBundle, ScriptInfo, SourcePosition};
use marley_browser::recorder::{self, Entry as RecordedEntry, Recorder, Recording};
use marley_browser::select::{self, SelectRequest};
use marley_browser::snapshot::{self, FrameTree, RefTarget};
use marley_browser::source_map::{self, MapLocation, OriginalPosition, SourceMap};
use marley_browser::{address, frame, service};
use project::{Project, ProjectPath};
use serde_json::Value;
use terminal_view::TerminalView;
use terminal_view::terminal_panel::TerminalPanel;
use ui::prelude::*;
use ui::{AlertModal, Chip, ContextMenu, IconPosition, Tooltip};
use util::ResultExt as _;
use util::paths::PathStyle;
use util::rel_path::RelPath;
use workspace::item::{Item, ItemEvent, SerializableItem};
use workspace::notifications::NotificationId;
use workspace::{ItemId, MultiWorkspace, Pane, SplitDirection, Toast, Workspace, WorkspaceId};

use crate::{
    Annotate, AnswerDialog, BrowserBack, BrowserForward, BrowserReload, DismissDialog,
    DropAnnotation, FocusAddressBar, GoToAddress, KeepAnnotation, NewBrowserTab, OpenBrowser,
    PickElement, RecordThis, RestoreAddress, SendPick,
};

/// How many times, a tenth of a second apart, a start waits for Chromium to write its endpoint
/// and answer.
const START_POLLS: u32 = 150;

/// The wait between two of those tries.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// How long a call waits for the browser to show its pages.
const SHOW_WAIT: Duration = Duration::from_secs(20);

/// The CSS pixels a wheel line scrolls: three lines a detent make the 100 a mouse wheel turns in
/// Chrome.
const WHEEL_LINE: f32 = 100.0 / 3.0;

/// How long the Agent chip stays after an agent's action ends (#492).
const AGENT_CHIP: Duration = Duration::from_secs(5);

/// How long a navigation an agent asked for may take to load before its call answers.
const LOAD_TIMEOUT: Duration = Duration::from_secs(15);

/// How long a tab waits for the pages a start found to be attached before it decides that its
/// page is not among them (#494).
const START_PAGES_WAIT: Duration = Duration::from_secs(10);

/// The URL of a blank page.
const BLANK: &str = "about:blank";

/// How often a tab waiting for its page looks at the browser's state.
const SHOWN_POLL: Duration = Duration::from_millis(250);

/// How soon after the user's press or key in the page a select that opens is taken as the
/// user's (#495); one an agent's click or key opens stays shut.
const USER_PRESS: Duration = Duration::from_secs(1);

/// How many of the pages the user focused the hub remembers, newest last (#574).
const FOCUS_HISTORY: usize = 64;

/// A cross-site iframe of a page, attached with a session of its own (#492).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iframe {
    /// Its session.
    pub session: String,
    /// The session it was attached from: its page's, or an outer iframe's.
    pub parent_session: String,
    /// Its frame's id, which is its target's.
    pub frame_id: String,
    /// Its URL.
    pub url: String,
}

/// What an agent did last in a page, for the Agent chip.
#[derive(Debug, Clone)]
struct AgentAction {
    text: SharedString,
    /// When it ended; none while it runs.
    ended: Option<Instant>,
}

/// What the hub is doing, as the Browser tabs show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubState {
    /// Finding or starting Chromium.
    Starting,
    /// Listing its pages.
    Connecting,
    /// Showing the pages.
    Showing,
    /// Stopped, for the reason given.
    Failed(SharedString),
}

/// What the hub tells the Browser tabs, each about one page, by its target id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserEvent {
    /// A page was attached: it wants a tab, beside its opener's when a page opened it, with the
    /// focus when `focus`.
    PageOpened {
        /// The page.
        target: String,
        /// The page that opened it, while the hub has it.
        opener: Option<String>,
        /// Whether its tab takes the focus.
        focus: bool,
        /// Whether a start found it, rather than something opening it since (#494): it gets a
        /// tab only when one claims it, such as a tab restored at launch.
        listed: bool,
    },
    /// A page went away: its tab closes.
    PageClosed {
        /// The page.
        target: String,
    },
    /// A page's title or URL changed, or the address it is going to.
    PageInfoChanged {
        /// The page.
        target: String,
    },
    /// A page opened a JavaScript dialog.
    DialogOpened {
        /// The page.
        target: String,
    },
    /// A page's dialog closed, answered here or by another client.
    DialogClosed {
        /// The page.
        target: String,
    },
    /// A page's `<select>` is opening (#495): its tab shows the list.
    SelectOpened {
        /// The page.
        target: String,
    },
    /// The user picked an element in a page (#496), staged as the pick `id`.
    PickStaged {
        /// The page.
        target: String,
        /// The pick.
        id: usize,
    },
}

/// An element the user picked (#496), staged in its tab until it is sent to the agent or
/// discarded.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Pick {
    /// Its number, from 1, which the agent's line and `browser_pick` name it by.
    pub id: usize,
    /// The page it was picked in, which the tools take as `tab`.
    pub tab: String,
    /// The page's URL at the pick, with secret-looking values hidden.
    pub url: String,
    /// The page's title at the pick.
    pub title: String,
    /// What the tray and the agent's line call it.
    pub summary: String,
    /// What the user said of it when sending it.
    pub caption: String,
    /// Whether it went to the agent.
    pub sent: bool,
    /// Whether the user took it out of the tray after sending it; the agent can still read it.
    #[serde(skip)]
    pub dismissed: bool,
    /// Whether its listeners' source maps were read (#497).
    #[serde(skip)]
    pub sources_read: bool,
    /// What the pick captured.
    pub bundle: PickBundle,
    /// The page around the element, a base64 JPEG.
    #[serde(skip)]
    pub crop: Option<String>,
}

/// Who drew an annotation (#498).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Maker {
    /// The user, in annotate mode.
    User,
    /// An agent, through `browser_annotate`.
    Agent,
}

/// A box and a note Marley draws over a page (#498), in the page's coordinates, the document's
/// CSS pixels, so it stays on what it marks as the page scrolls.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Annotation {
    /// Its number, from 1, across the session.
    pub id: usize,
    /// Its box in the page.
    #[serde(rename = "box")]
    pub page_box: PageBox,
    /// What it says.
    pub note: String,
    /// Who drew it.
    pub maker: Maker,
    /// When, in seconds since the Unix epoch.
    pub made_at: u64,
}

/// The viewport a tab asked for: its size in whole logical pixels, at the window's scale.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Viewport {
    width: u32,
    height: u32,
    scale: f32,
}

/// A select the page's listener caught opening (#495), with where the choice goes: the session
/// and the listener's world that reported it.
#[derive(Debug, Clone)]
struct SelectState {
    session: String,
    context: i64,
    request: SelectRequest,
}

/// One page of Marley's Chromium, as its tab shows it (#493).
struct PageState {
    page: Page,
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
    /// The select whose list the page's tab shows or is about to.
    select: Option<SelectState>,
    /// Whether the page is in pick mode (#496).
    picking: bool,
    /// The page's scripts, by id, as `Debugger.scriptParsed` named them, which a pick's
    /// listeners need: from the first pick mode on, when the Debugger domain comes on.
    scripts: Option<HashMap<String, ScriptInfo>>,
    /// Why the page's last pick could not be read.
    pick_error: Option<SharedString>,
    /// The boxes and notes drawn over the page (#498), oldest first, until it leaves its
    /// document.
    annotations: Vec<Annotation>,
    /// The page's last minute, kept while a tab draws the page (#499).
    recorder: Recorder,
    /// The page's cross-site iframes (#492).
    iframes: Vec<Iframe>,
    /// What the page logged since it was attached.
    console: ConsoleLog,
    /// What the page fetched since it was attached.
    network: NetworkLog,
    /// What the refs of the newest snapshot name.
    refs: Vec<RefTarget>,
    /// The agent's last action, for the Agent chip.
    agent: Option<AgentAction>,
    /// Calls waiting for the main frame to stop loading.
    load_waiters: Vec<oneshot::Sender<()>>,
    viewport: Option<Viewport>,
    /// How many tabs draw the page.
    viewers: usize,
    screencasting: bool,
    /// The mouse buttons held in the page, in CDP's bits, so a drag that leaves the tab still
    /// reaches the page, its release too.
    held_buttons: u32,
    /// When the oldest input that no frame has shown yet was sent.
    input_at: Option<Instant>,
}

impl PageState {
    fn new(page: Page) -> Self {
        Self {
            page,
            frame: None,
            metadata: None,
            title: None,
            url: None,
            pending_url: None,
            loading: false,
            history: None,
            dialog: None,
            select: None,
            picking: false,
            scripts: None,
            pick_error: None,
            annotations: Vec::new(),
            recorder: Recorder::default(),
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
        }
    }

    fn target(&self) -> &str {
        self.page.target_id()
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
}

/// What `browser_tabs` says of a page.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TabSummary {
    /// The page's id, which the tools take as `tab`.
    pub id: String,
    /// Its title.
    pub title: String,
    /// Its URL.
    pub url: String,
    /// Whether its main frame loads.
    pub loading: bool,
    /// Whether it is the page whose tab the user focused last, anywhere.
    pub focused: bool,
}

/// Marley's browser as the Browser tabs see it: the connection's state and every page.
pub struct BrowserHub {
    state: HubState,
    connection: Option<Connection>,
    pages: Vec<PageState>,
    /// The pages being attached, so a page is attached once.
    attaching: Vec<String>,
    /// The pages closing or gone while they were attached: an attach that ends for one drops it.
    closing: Vec<String>,
    /// The pages whose tabs the user focused, each once, the newest last (#574).
    focus_history: Vec<String>,
    /// The workspace each page an agent asked for gets its tab in, until the tab opens (#574).
    placements: Vec<(String, WeakEntity<Workspace>)>,
    /// The picks of the session, oldest first (#496).
    picks: Vec<Pick>,
    /// The next pick's number.
    next_pick: usize,
    /// The next annotation's number (#498).
    next_annotation: usize,
    /// Bumped at each start, so a superseded start's late results are dropped.
    generation: u64,
    run: Option<Task<()>>,
}

impl fmt::Debug for BrowserHub {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserHub")
            .field("state", &self.state)
            .field("pages", &self.pages.len())
            .field("focused", &self.focused())
            .finish_non_exhaustive()
    }
}

impl EventEmitter<BrowserEvent> for BrowserHub {}

struct HubHandle {
    hub: Entity<BrowserHub>,
    _tabs: Subscription,
}

impl Global for HubHandle {}

/// Every Browser tab, so a page finds its tab (#493).
#[derive(Default)]
struct BrowserViews(Vec<WeakEntity<BrowserView>>);

impl Global for BrowserViews {}

impl BrowserHub {
    /// The app's hub, created and started the first time it is asked for, with the tabs it opens
    /// and closes as pages come and go.
    pub fn global(cx: &mut App) -> Entity<Self> {
        if let Some(handle) = cx.try_global::<HubHandle>() {
            return handle.hub.clone();
        }
        let hub = cx.new(|cx| {
            let mut hub = Self {
                state: HubState::Starting,
                connection: None,
                pages: Vec::new(),
                attaching: Vec::new(),
                closing: Vec::new(),
                focus_history: Vec::new(),
                placements: Vec::new(),
                picks: Vec::new(),
                next_pick: 1,
                next_annotation: 1,
                generation: 0,
                run: None,
            };
            hub.start(cx);
            hub
        });
        let tabs = cx.subscribe(&hub, |hub, event: &BrowserEvent, cx| match event {
            BrowserEvent::PageOpened {
                target,
                opener,
                focus,
                listed,
            } => open_tab(&hub, target, opener.as_deref(), *focus, *listed, cx),
            BrowserEvent::PageClosed { target } => close_tabs(target, cx),
            _ => {}
        });
        cx.set_global(HubHandle {
            hub: hub.clone(),
            _tabs: tabs,
        });
        hub
    }

    /// What the hub is doing.
    #[must_use]
    pub const fn state(&self) -> &HubState {
        &self.state
    }

    fn page_state(&self, target: &str) -> Option<&PageState> {
        self.pages.iter().find(|page| page.target() == target)
    }

    fn page_state_mut(&mut self, target: &str) -> Option<&mut PageState> {
        self.pages.iter_mut().find(|page| page.target() == target)
    }

    /// The page `target`, while the hub has it.
    #[must_use]
    pub fn page(&self, target: &str) -> Option<Page> {
        self.page_state(target).map(|page| page.page.clone())
    }

    /// Whether the browser has pages, attached or being attached.
    #[must_use]
    pub const fn has_pages(&self) -> bool {
        !self.pages.is_empty() || !self.attaching.is_empty()
    }

    /// Whether a page is being attached.
    #[must_use]
    pub const fn is_attaching(&self) -> bool {
        !self.attaching.is_empty()
    }

    /// The page whose tab the user focused last while it lives, else the newest page.
    #[must_use]
    pub fn focused(&self) -> Option<String> {
        self.newest_focused(|_| true)
    }

    /// Among `targets`, the page whose tab the user focused last while it lives, else the newest
    /// of them (#574).
    #[must_use]
    pub fn focused_among(&self, targets: &HashSet<String>) -> Option<String> {
        self.newest_focused(|target| targets.contains(target))
    }

    /// The live page `wanted` takes that the user focused last, else the newest one it takes.
    fn newest_focused(&self, wanted: impl Fn(&str) -> bool) -> Option<String> {
        self.focus_history
            .iter()
            .rev()
            .map(String::as_str)
            .chain(self.pages.iter().rev().map(PageState::target))
            .find(|target| wanted(target) && self.page_state(target).is_some())
            .map(str::to_string)
    }

    fn set_focused(&mut self, target: &str) {
        if self.focus_history.last().map(String::as_str) == Some(target) {
            return;
        }
        self.focus_history.retain(|focused| focused != target);
        self.focus_history.push(target.to_string());
        if self.focus_history.len() > FOCUS_HISTORY {
            self.focus_history.remove(0);
        }
    }

    /// The workspace the page `target` gets its tab in, when an agent's call asked for one
    /// (#574); asked once.
    fn take_placement(&mut self, target: &str) -> Option<WeakEntity<Workspace>> {
        let index = self
            .placements
            .iter()
            .position(|(placed, _)| placed == target)?;
        Some(self.placements.remove(index).1)
    }

    /// Every page, oldest first, as `browser_tabs` lists them.
    #[must_use]
    pub fn tabs(&self) -> Vec<TabSummary> {
        let focused = self.focused();
        self.pages
            .iter()
            .map(|page| TabSummary {
                id: page.target().to_string(),
                title: page
                    .title
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                url: page
                    .url
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                loading: page.loading,
                focused: focused.as_deref() == Some(page.target()),
            })
            .collect()
    }

    /// The page's title, once it has one.
    #[must_use]
    pub fn title(&self, target: &str) -> Option<SharedString> {
        self.page_state(target)
            .and_then(|page| page.title.clone())
            .filter(|title| !title.is_empty())
    }

    /// The page's URL.
    #[must_use]
    pub fn url(&self, target: &str) -> Option<SharedString> {
        self.page_state(target).and_then(|page| page.url.clone())
    }

    /// What the page's address bar shows: where the page is going, else where it is.
    #[must_use]
    pub fn address(&self, target: &str) -> Option<SharedString> {
        self.page_state(target)
            .and_then(|page| page.pending_url.clone().or_else(|| page.url.clone()))
    }

    /// Whether the page's main frame is loading.
    #[must_use]
    pub fn is_loading(&self, target: &str) -> bool {
        self.page_state(target).is_some_and(|page| page.loading)
    }

    /// Whether the page's history has an entry `offset` steps from the one it shows: -1 back,
    /// 1 forward.
    #[must_use]
    pub fn can_go(&self, target: &str, offset: isize) -> bool {
        self.page_state(target).is_some_and(|page| {
            page.history
                .as_ref()
                .is_some_and(|history| history.entry_at(offset).is_some())
        })
    }

    /// The JavaScript dialog the page waits on.
    #[must_use]
    pub fn dialog(&self, target: &str) -> Option<&JavaScriptDialog> {
        self.page_state(target)
            .and_then(|page| page.dialog.as_ref())
    }

    /// Starts over when the hub stopped, and says whether it did.
    pub fn start_if_failed(&mut self, cx: &mut Context<Self>) -> bool {
        let failed = matches!(self.state, HubState::Failed(_));
        if failed {
            self.start(cx);
        }
        failed
    }

    /// Starts over: finds or starts Chromium, connects, attaches to each of its pages (a blank
    /// one made when it has none) and to each it reports later, and follows their events until
    /// the connection ends. The tabs of the pages it had close.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        self.state = HubState::Starting;
        self.connection = None;
        self.attaching.clear();
        self.closing.clear();
        self.placements.clear();
        for page in std::mem::take(&mut self.pages) {
            cx.emit(BrowserEvent::PageClosed {
                target: page.target().to_string(),
            });
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
                if this.generation == generation {
                    this.connection = Some(connection.clone());
                    this.state = HubState::Connecting;
                    cx.notify();
                }
            })
            .ok();
            match pages_at_start(&connection).await {
                Ok(pages) => {
                    this.update(cx, |this, cx| {
                        if this.generation == generation {
                            this.state = HubState::Showing;
                            // Discovery reports these again, and each is attached once.
                            for target in pages {
                                this.attach(generation, target, None, true, cx);
                            }
                            cx.notify();
                        }
                    })
                    .ok();
                }
                Err(error) => {
                    this.update(cx, |this, cx| {
                        this.fail(
                            generation,
                            format!("Could not list the browser's pages: {error}"),
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

    fn fail(&mut self, generation: u64, reason: String, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        log::warn!("browser: {reason}");
        self.state = HubState::Failed(reason.into());
        self.connection = None;
        self.attaching.clear();
        // The tabs stay, and say why, until the browser is opened again.
        for page in &mut self.pages {
            page.load_waiters.clear();
        }
        cx.notify();
    }

    /// Attaches to the page `target` in a task of its own, unless it is attached or attaching;
    /// once attached, its tab opens. A `listed` page is one a start found.
    fn attach(
        &mut self,
        generation: u64,
        target: String,
        opener: Option<String>,
        listed: bool,
        cx: &Context<Self>,
    ) {
        if generation != self.generation
            || self.page_state(&target).is_some()
            || self.attaching.contains(&target)
        {
            return;
        }
        let Some(connection) = self.connection.clone() else {
            return;
        };
        self.attaching.push(target.clone());
        cx.spawn(async move |this, cx| {
            let attached = Page::attach(&connection, &target).await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.attaching.retain(|attaching| *attaching != target);
                match attached {
                    Ok(page) => this.attached(page, opener, listed, cx),
                    Err(error) => {
                        this.closing.retain(|closing| *closing != target);
                        this.placements.retain(|(placed, _)| *placed != target);
                        log::warn!("browser: could not attach to a page: {error}");
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    fn attached(
        &mut self,
        page: Page,
        opener: Option<String>,
        listed: bool,
        cx: &mut Context<Self>,
    ) {
        let target = page.target_id().to_string();
        if let Some(index) = self.closing.iter().position(|closing| *closing == target) {
            self.closing.remove(index);
            return;
        }
        if self.page_state(&target).is_some() {
            return;
        }
        // A page a page opened takes the focus, as in a browser; one anyone else opened waits for
        // the user.
        let opener = opener.filter(|opener| self.page_state(opener).is_some());
        let focus = opener.is_some();
        // A page takes the size of the tab it opens beside, so a page behind another tab, an
        // agent's say, lays out as it will show.
        let beside = opener.clone().or_else(|| self.focused());
        let viewport = beside
            .and_then(|beside| self.page_state(&beside))
            .and_then(|beside| beside.viewport);
        self.pages.push(PageState::new(page.clone()));
        if let Some(viewport) = viewport {
            self.lay_out(&target, viewport, cx);
        }
        cx.notify();
        cx.emit(BrowserEvent::PageOpened {
            target: target.clone(),
            opener,
            focus,
            listed,
        });
        let generation = self.generation;
        cx.spawn(async move |this, cx| {
            page.watch_selects(page.session_id()).await.log_err();
            if let Some(info) = page.target_info().await.log_err() {
                this.update(cx, |this, cx| this.target_changed(generation, info, cx))
                    .ok();
            }
            let history = page.history().await.log_err();
            this.update(cx, |this, cx| {
                this.navigated(generation, &target, None, history, cx);
            })
            .ok();
        })
        .detach();
    }

    /// The page `target` went away: its tab closes.
    fn page_gone(&mut self, generation: u64, target: &str, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        if self.attaching.iter().any(|attaching| attaching == target) {
            if !self.closing.iter().any(|closing| closing == target) {
                self.closing.push(target.to_string());
            }
        } else {
            self.closing.retain(|closing| closing != target);
        }
        self.focus_history.retain(|focused| focused != target);
        self.placements.retain(|(placed, _)| placed != target);
        let before = self.pages.len();
        self.pages.retain(|page| page.target() != target);
        if self.pages.len() != before {
            cx.emit(BrowserEvent::PageClosed {
                target: target.to_string(),
            });
            cx.notify();
        }
    }

    /// Opens a new page at `url` and answers with its id once the browser has made it; the page
    /// attaches, and its tab opens as any page's does, a moment later (#493), in `place_in` when
    /// given (#574).
    pub fn create_page_task(
        &self,
        url: String,
        place_in: Option<WeakEntity<Workspace>>,
        cx: &Context<Self>,
    ) -> Task<Result<String, String>> {
        let Some(connection) = self.connection.clone() else {
            return Task::ready(Err("the browser is not connected".to_string()));
        };
        cx.spawn(async move |this, cx| {
            let target = Page::create(&connection, &url)
                .await
                .map_err(|error| error.to_string())?;
            // Kept before the page can attach, which takes several round trips more, so its tab
            // finds it.
            if let Some(workspace) = place_in {
                this.update(cx, |hub, _| {
                    hub.placements.push((target.clone(), workspace));
                })
                .ok();
            }
            Ok(target)
        })
    }

    /// Closes the page `target`, attached or not, as closing its tab in a browser does; its tab
    /// goes when the browser reports the page gone.
    pub fn close_page(&mut self, target: &str, cx: &Context<Self>) {
        let Some(connection) = self.connection.clone() else {
            return;
        };
        if !self.closing.iter().any(|closing| closing == target) {
            self.closing.push(target.to_string());
        }
        let target = target.to_string();
        cx.spawn(async move |_, _| {
            Page::close(&connection, &target).await.log_err();
        })
        .detach();
    }

    /// Navigates the page `target` to `url`, which its address bar shows until the navigation
    /// commits or ends. A dialog the page waits on is answered with Cancel first, as Chrome
    /// closes a page's dialog when the page is left.
    pub fn navigate(&mut self, target: &str, url: String, cx: &mut Context<Self>) {
        self.navigate_task(target, url, cx).detach();
    }

    /// Navigates as [`BrowserHub::navigate`] does, in a task that ends once the page has loaded,
    /// or 15 seconds later, with Chromium's reason when the navigation failed (#492).
    pub fn navigate_task(
        &mut self,
        target: &str,
        url: String,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let Some(page) = self.page(target) else {
            return Task::ready(Err(no_page(target)));
        };
        let dismiss = self.leave_dialog(target, cx);
        let Some(state) = self.page_state_mut(target) else {
            return Task::ready(Err(no_page(target)));
        };
        state.pending_url = Some(SharedString::from(&url));
        let loaded = state.load_waiter();
        cx.emit(BrowserEvent::PageInfoChanged {
            target: target.to_string(),
        });
        cx.notify();
        let generation = self.generation;
        let target = target.to_string();
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
                this.update(cx, |this, cx| {
                    this.navigation_ended(generation, &target, &url, cx);
                })
                .ok();
            }
            result.map_err(|error| error.to_string())?;
            wait_for_load(loaded, cx).await;
            Ok(())
        })
    }

    fn navigation_ended(
        &mut self,
        generation: u64,
        target: &str,
        url: &str,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        if let Some(page) = self.page_state_mut(target)
            && page.pending_url.as_deref() == Some(url)
        {
            page.pending_url = None;
            cx.emit(BrowserEvent::PageInfoChanged {
                target: target.to_string(),
            });
            cx.notify();
        }
    }

    /// Moves the page `target` `offset` entries through its history: -1 back, 1 forward.
    pub fn go(&mut self, target: &str, offset: isize, cx: &mut Context<Self>) {
        self.go_task(target, offset, cx).detach();
    }

    /// Moves as [`BrowserHub::go`] does, in a task that ends once the page has loaded, or
    /// 15 seconds later (#492).
    pub fn go_task(
        &mut self,
        target: &str,
        offset: isize,
        cx: &mut Context<Self>,
    ) -> Task<Result<(), String>> {
        let Some(page) = self.page(target) else {
            return Task::ready(Err(no_page(target)));
        };
        let entry = self.page_state_mut(target).and_then(|state| {
            let history = state.history.as_mut()?;
            let (index, id) = history
                .entry_at(offset)
                .map(|(index, entry)| (index, entry.id))?;
            // A second press before the page reports where it went goes on from here.
            history.current_index = index;
            Some(id)
        });
        let Some(id) = entry else {
            return Task::ready(Err("the page's history has no entry there".to_string()));
        };
        let dismiss = self.leave_dialog(target, cx);
        let Some(loaded) = self.page_state_mut(target).map(PageState::load_waiter) else {
            return Task::ready(Err(no_page(target)));
        };
        cx.notify();
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

    /// Loads the page `target` again.
    pub fn reload(&mut self, target: &str, cx: &mut Context<Self>) {
        let Some(page) = self.page(target) else {
            return;
        };
        let dismiss = self.leave_dialog(target, cx);
        cx.spawn(async move |_, _| {
            if dismiss {
                page.answer_dialog(false, None).await.log_err();
            }
            page.reload().await.log_err();
        })
        .detach();
    }

    /// Stops the page's loading.
    pub fn stop(&self, target: &str, cx: &Context<Self>) {
        let Some(page) = self.page(target) else {
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
        target: &str,
        accept: bool,
        prompt_text: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.page(target) else {
            return;
        };
        if !self.leave_dialog(target, cx) {
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
    fn leave_dialog(&mut self, target: &str, cx: &mut Context<Self>) -> bool {
        let had = self
            .page_state_mut(target)
            .is_some_and(|page| page.dialog.take().is_some());
        if had {
            cx.emit(BrowserEvent::DialogClosed {
                target: target.to_string(),
            });
            cx.notify();
        }
        had
    }

    fn dialog_opened(
        &mut self,
        generation: u64,
        target: &str,
        dialog: JavaScriptDialog,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        if let Some(page) = self.page_state_mut(target) {
            page.dialog = Some(dialog);
            cx.emit(BrowserEvent::DialogOpened {
                target: target.to_string(),
            });
            cx.notify();
        }
    }

    /// A page's listener reported a select opening (#495): the page keeps it for its tab.
    fn select_requested(
        &mut self,
        generation: u64,
        session: &str,
        params: &Value,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation
            || params.get("name").and_then(Value::as_str) != Some(select::BINDING)
        {
            return;
        }
        let Some(target) = self.target_of_session(session) else {
            return;
        };
        let (Some(payload), Some(context)) = (
            params.get("payload").and_then(Value::as_str),
            params.get("executionContextId").and_then(Value::as_i64),
        ) else {
            return;
        };
        let request = match SelectRequest::parse(payload) {
            Ok(request) => request,
            Err(error) => {
                log::warn!("browser: a select's report did not read: {error}");
                return;
            }
        };
        if let Some(page) = self.page_state_mut(&target) {
            page.select = Some(SelectState {
                session: session.to_string(),
                context,
                request,
            });
            cx.emit(BrowserEvent::SelectOpened { target });
        }
    }

    /// The select whose list the page's tab shows or is about to.
    fn select(&self, target: &str) -> Option<&SelectState> {
        self.page_state(target)?.select.as_ref()
    }

    /// Chooses option `index` of the page's open select.
    fn choose_option(&mut self, target: &str, index: usize, cx: &Context<Self>) {
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        let Some(select) = page.select.take() else {
            return;
        };
        let session = page.page.clone();
        cx.spawn(async move |_, _| {
            match session
                .choose_option(&select.session, select.context, index)
                .await
            {
                Ok(true) => {}
                Ok(false) => log::info!("browser: the select or its option went before the choice"),
                Err(error) => log::warn!("browser: choosing a select's option failed: {error}"),
            }
        })
        .detach();
    }

    /// Whether the page is in pick mode.
    #[must_use]
    pub fn is_picking(&self, target: &str) -> bool {
        self.page_state(target).is_some_and(|page| page.picking)
    }

    /// Turns pick mode on or off for the page (#496). The first time, the page's Debugger domain
    /// comes on, which names each script it loads from then on, for the picks' listeners.
    pub fn set_picking(&mut self, target: &str, on: bool, cx: &mut Context<Self>) {
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        if page.picking == on {
            return;
        }
        page.picking = on;
        let watch = on && page.scripts.is_none();
        if on {
            page.scripts.get_or_insert_with(HashMap::new);
            page.pick_error = None;
        }
        let session = page.page.clone();
        cx.spawn(async move |_, _| {
            if watch {
                session.watch_scripts().await.log_err();
            }
            session.set_inspect(on).await.log_err();
        })
        .detach();
        cx.notify();
    }

    fn script_parsed(&mut self, generation: u64, target: &str, params: &Value) {
        if generation != self.generation {
            return;
        }
        // A script with no URL, an `eval`'s say, names no place a pick could show.
        let (Some(id), Some(url)) = (
            params.get("scriptId").and_then(Value::as_str),
            params
                .get("url")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty()),
        ) else {
            return;
        };
        let source_map = params
            .get("sourceMapURL")
            .and_then(Value::as_str)
            .filter(|map| !map.is_empty())
            .map(str::to_string);
        if let Some(scripts) = self
            .page_state_mut(target)
            .and_then(|page| page.scripts.as_mut())
        {
            scripts.insert(
                id.to_string(),
                ScriptInfo {
                    url: url.to_string(),
                    source_map,
                },
            );
        }
    }

    /// The user clicked an element in pick mode: the mode ends, and the element is read into a
    /// pick at once, before the page can change it.
    fn pick_requested(
        &mut self,
        generation: u64,
        target: &str,
        backend_node_id: i64,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        let url = self.url(target);
        let title = self.title(target);
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        // Another client's inspect mode picks for itself.
        if !page.picking {
            return;
        }
        page.picking = false;
        let session = page.page.clone();
        let scripts = page.scripts.clone().unwrap_or_default();
        let target = target.to_string();
        cx.notify();
        cx.spawn(async move |this, cx| {
            session.set_inspect(false).await.log_err();
            let captured = session.capture_pick(backend_node_id, &scripts).await;
            let (crop, listeners) = match &captured {
                Ok(bundle) => (
                    session.crop(bundle.page_box).await.log_err(),
                    bundle.listeners.clone(),
                ),
                Err(_) => (None, Vec::new()),
            };
            let Ok(Some(id)) = this.update(cx, |this, cx| {
                this.pick_captured(&target, (url, title), captured, crop, cx)
            }) else {
                return;
            };
            // The tray shows the pick at once; the listeners' places follow.
            let positions = original_positions(&session, &listeners, cx).await;
            this.update(cx, |this, cx| this.pick_sources(id, positions, cx))
                .ok();
        })
        .detach();
    }

    /// Stages the pick the page's capture read, or says why it read none; the staged pick's id.
    fn pick_captured(
        &mut self,
        target: &str,
        (url, title): (Option<SharedString>, Option<SharedString>),
        captured: Result<PickBundle, CdpError>,
        crop: Option<String>,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let staged = match captured {
            Ok(bundle) => {
                let id = self.next_pick;
                self.next_pick += 1;
                self.picks.push(Pick {
                    id,
                    tab: target.to_string(),
                    url: url.map(|url| redact_url(&url)).unwrap_or_default(),
                    title: title.map(|title| title.to_string()).unwrap_or_default(),
                    summary: bundle.summary(),
                    caption: String::new(),
                    sent: false,
                    dismissed: false,
                    sources_read: false,
                    bundle,
                    crop,
                });
                cx.emit(BrowserEvent::PickStaged {
                    target: target.to_string(),
                    id,
                });
                Some(id)
            }
            Err(error) => {
                log::warn!("browser: a pick did not read: {error}");
                if let Some(page) = self.page_state_mut(target) {
                    page.pick_error = Some(format!("Could not read the element: {error}").into());
                }
                None
            }
        };
        cx.notify();
        staged
    }

    /// Sets each listener's place in its original source in the pick `id`, with the file in its
    /// tab's project that holds it (#497).
    fn pick_sources(
        &mut self,
        id: usize,
        positions: Vec<Option<OriginalPosition>>,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = self.pick(id).map(|pick| pick.tab.clone()) else {
            return;
        };
        let project = view_of(&tab, cx)
            .and_then(|view| view.read(cx).workspace.upgrade())
            .map(|workspace| workspace.read(cx).project().clone());
        let files: Vec<Option<String>> = positions
            .iter()
            .map(|position| {
                let (position, project) = (position.as_ref()?, project.as_ref()?);
                find_source(project.read(cx), &position.source, cx)
                    .map(|path| path.path.as_unix_str().to_string())
            })
            .collect();
        let Some(pick) = self.picks.iter_mut().find(|pick| pick.id == id) else {
            return;
        };
        for ((listener, position), file) in
            pick.bundle.listeners.iter_mut().zip(positions).zip(files)
        {
            listener.original = position.map(|position| SourcePosition {
                source: position.source,
                file,
                line: position.line.saturating_add(1),
                column: position.column.saturating_add(1),
            });
        }
        pick.sources_read = true;
        cx.notify();
    }

    fn inspect_canceled(&mut self, generation: u64, target: &str, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        if let Some(page) = self.page_state_mut(target)
            && page.picking
        {
            page.picking = false;
            cx.notify();
        }
    }

    /// The session's picks, oldest first.
    #[must_use]
    pub fn picks(&self) -> &[Pick] {
        &self.picks
    }

    /// The pick `id`.
    #[must_use]
    pub fn pick(&self, id: usize) -> Option<&Pick> {
        self.picks.iter().find(|pick| pick.id == id)
    }

    /// Why the page's last pick could not be read.
    fn pick_error(&self, target: &str) -> Option<SharedString> {
        self.page_state(target)?.pick_error.clone()
    }

    fn clear_pick_error(&mut self, target: &str, cx: &mut Context<Self>) {
        if let Some(page) = self.page_state_mut(target) {
            page.pick_error = None;
            cx.notify();
        }
    }

    /// Marks the pick `id` sent to the agent, with its caption.
    fn pick_sent(&mut self, id: usize, caption: String, cx: &mut Context<Self>) {
        if let Some(pick) = self.picks.iter_mut().find(|pick| pick.id == id) {
            pick.sent = true;
            pick.caption = caption;
            cx.notify();
        }
    }

    /// Drops the pick `id` from its tray: a pick not sent goes, and a sent one stays for the
    /// agent, whose line names it.
    fn discard_pick(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(pick) = self
            .picks
            .iter_mut()
            .find(|pick| pick.id == id && pick.sent)
        {
            pick.dismissed = true;
        } else {
            self.picks.retain(|pick| pick.id != id);
        }
        cx.notify();
    }

    /// Keeps `entry` in the page's minute, while a tab draws the page (#499).
    fn record_entry(&mut self, target: &str, entry: RecordedEntry) {
        if let Some(page) = self.page_state_mut(target)
            && page.viewers > 0
        {
            page.recorder.push(entry, Instant::now());
        }
    }

    fn record_navigation(&mut self, generation: u64, target: &str, url: Option<&str>) {
        if generation != self.generation {
            return;
        }
        if let Some(url) = url {
            self.record_entry(
                target,
                RecordedEntry::Navigation {
                    url: redact_url(url),
                },
            );
        }
    }

    /// Saves the page's minute, with a snapshot of the moment, as a recording in `dir`, off the
    /// main thread; the recording's id (#499).
    pub fn record(
        &self,
        target: &str,
        dir: PathBuf,
        cx: &Context<Self>,
    ) -> Task<anyhow::Result<String>> {
        let Some(page) = self.page_state(target) else {
            return Task::ready(Err(anyhow::anyhow!("the page is gone")));
        };
        let (entries, frames, seconds) = page.recorder.take(Instant::now());
        let mut recording = Recording {
            id: chrono::Local::now().format("%Y%m%d-%H%M%S").to_string(),
            tab: target.to_string(),
            url: page.url.as_deref().map(redact_url).unwrap_or_default(),
            title: page
                .title
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            recorded_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| since.as_secs()),
            seconds,
            frames: frames.len(),
            entries,
            snapshot: String::new(),
        };
        let session = page.page.clone();
        cx.spawn(async move |_, cx| {
            recording.snapshot = snapshot_text(&session).await.unwrap_or_default();
            cx.background_spawn(futures::future::lazy(move |_| {
                recorder::save_in(&dir, &mut recording, &frames).map(|_| recording.id)
            }))
            .await
            .map_err(anyhow::Error::from)
        })
    }

    /// The page's annotations, oldest first (#498).
    #[must_use]
    pub fn annotations(&self, target: &str) -> &[Annotation] {
        self.page_state(target)
            .map_or(&[], |page| page.annotations.as_slice())
    }

    /// Draws a box with a note over the page; the annotation's id, or none when the page is
    /// gone.
    pub fn add_annotation(
        &mut self,
        target: &str,
        page_box: PageBox,
        note: String,
        maker: Maker,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let id = self.next_annotation;
        let made_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        self.page_state_mut(target)?.annotations.push(Annotation {
            id,
            page_box,
            note,
            maker,
            made_at,
        });
        self.next_annotation += 1;
        cx.notify();
        Some(id)
    }

    fn remove_annotation(&mut self, target: &str, id: usize, cx: &mut Context<Self>) {
        if let Some(page) = self.page_state_mut(target) {
            page.annotations.retain(|annotation| annotation.id != id);
            cx.notify();
        }
    }

    /// Removes the agent's annotations from the page, the user's staying; how many went.
    pub fn clear_agent_annotations(&mut self, target: &str, cx: &mut Context<Self>) -> usize {
        let Some(page) = self.page_state_mut(target) else {
            return 0;
        };
        let before = page.annotations.len();
        page.annotations
            .retain(|annotation| annotation.maker != Maker::Agent);
        let removed = before - page.annotations.len();
        cx.notify();
        removed
    }

    /// The page's main frame shows another document: what was drawn over the old one goes.
    fn left_document(&mut self, generation: u64, target: &str, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        if let Some(page) = self.page_state_mut(target)
            && !page.annotations.is_empty()
        {
            page.annotations.clear();
            cx.notify();
        }
    }

    /// Forgets the page's open select: its list closed with no choice.
    fn dismiss_select(&mut self, target: &str) {
        if let Some(page) = self.page_state_mut(target) {
            page.select = None;
        }
    }

    fn dialog_closed(&mut self, generation: u64, target: &str, cx: &mut Context<Self>) {
        if generation == self.generation {
            self.leave_dialog(target, cx);
        }
    }

    /// The page's main frame, whose id is the page's, started or stopped loading.
    fn loading_changed(
        &mut self,
        generation: u64,
        target: &str,
        frame_id: Option<&str>,
        loading: bool,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation || frame_id != Some(target) {
            return;
        }
        if let Some(page) = self.page_state_mut(target)
            && page.loading != loading
        {
            page.loading = loading;
            if !loading {
                page.loaded();
            }
            cx.notify();
        }
    }

    /// The page's main frame committed a navigation, to `url` when the event said where: the
    /// address bar's navigation is over, and back and forward go from the new history.
    fn navigated(
        &mut self,
        generation: u64,
        target: &str,
        url: Option<String>,
        history: Option<NavigationHistory>,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        if history.is_some() {
            page.history = history;
        }
        let arrived = url.is_some() && page.pending_url.take().is_some();
        let moved = match url.map(SharedString::from) {
            Some(url) if page.url.as_ref() != Some(&url) => {
                page.url = Some(url);
                true
            }
            _ => false,
        };
        if arrived || moved {
            cx.emit(BrowserEvent::PageInfoChanged {
                target: target.to_string(),
            });
        }
        cx.notify();
    }

    fn within_document(&mut self, target: &str) {
        if let Some(page) = self.page_state_mut(target) {
            page.loaded();
        }
    }

    /// The page's cross-site iframes.
    #[must_use]
    pub fn iframes(&self, target: &str) -> Vec<Iframe> {
        self.page_state(target)
            .map(|page| page.iframes.clone())
            .unwrap_or_default()
    }

    /// What the page logged, oldest first.
    #[must_use]
    pub fn console_entries(&self, target: &str) -> Vec<ConsoleEntry> {
        self.page_state(target)
            .map(|page| page.console.entries().cloned().collect())
            .unwrap_or_default()
    }

    /// What the page fetched, oldest first.
    #[must_use]
    pub fn network_entries(&self, target: &str) -> Vec<NetworkEntry> {
        self.page_state(target)
            .map(|page| page.network.entries().cloned().collect())
            .unwrap_or_default()
    }

    /// Keeps the refs of the page's new snapshot, in place of the last one's.
    pub fn set_refs(&mut self, target: &str, refs: Vec<RefTarget>) {
        if let Some(page) = self.page_state_mut(target) {
            page.refs = refs;
        }
    }

    /// What the ref `id` of the page's newest snapshot names.
    #[must_use]
    pub fn ref_target(&self, target: &str, id: &str) -> Option<RefTarget> {
        self.page_state(target)?
            .refs
            .iter()
            .find(|reference| reference.id == id)
            .cloned()
    }

    /// Shows the page's Agent chip with what an agent is doing, until it ends.
    pub fn agent_started(
        &mut self,
        target: &str,
        text: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        if let Some(page) = self.page_state_mut(target) {
            page.agent = Some(AgentAction {
                text: text.into(),
                ended: None,
            });
            cx.notify();
        }
    }

    /// Shows the page's Agent chip with what an agent did, for five seconds more.
    pub fn agent_ended(
        &mut self,
        target: &str,
        text: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let text: SharedString = text.into();
        self.record_entry(
            target,
            RecordedEntry::Agent {
                did: text.to_string(),
            },
        );
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        page.agent = Some(AgentAction {
            text,
            ended: Some(Instant::now()),
        });
        cx.notify();
        let target = target.to_string();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(AGENT_CHIP).await;
            this.update(cx, |this, cx| {
                let Some(page) = this.page_state_mut(&target) else {
                    return;
                };
                let over = page.agent.as_ref().is_some_and(|action| {
                    action
                        .ended
                        .is_some_and(|ended| ended.elapsed() >= AGENT_CHIP)
                });
                if over {
                    page.agent = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// What the page's Agent chip says, while it shows.
    fn agent_chip(&self, target: &str) -> Option<SharedString> {
        self.page_state(target)?
            .agent
            .as_ref()
            .map(|action| action.text.clone())
    }

    /// The page an event from `session` belongs to: the page's own session, or one of its
    /// iframes'.
    fn target_of_session(&self, session: &str) -> Option<String> {
        self.pages
            .iter()
            .find(|page| {
                page.page.session_id() == session
                    || page.iframes.iter().any(|iframe| iframe.session == session)
            })
            .map(|page| page.target().to_string())
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
        let Some(target) = self.target_of_session(&parent_session) else {
            return;
        };
        let info = params.get("targetInfo");
        let kind = info
            .and_then(|info| info.get("type"))
            .and_then(Value::as_str);
        let (Some("iframe"), Some(session), Some(frame_id)) = (
            kind,
            params.get("sessionId").and_then(Value::as_str),
            info.and_then(|info| info.get("targetId"))
                .and_then(Value::as_str),
        ) else {
            return;
        };
        let url = info
            .and_then(|info| info.get("url"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let Some(page) = self.page_state_mut(&target) else {
            return;
        };
        page.iframes.retain(|iframe| iframe.session != session);
        page.iframes.push(Iframe {
            session: session.to_string(),
            parent_session,
            frame_id: frame_id.to_string(),
            url,
        });
        let session = session.to_string();
        let page = page.page.clone();
        cx.spawn(async move |_, _| {
            page.observe(&session).await.log_err();
            page.watch_selects(&session).await.log_err();
        })
        .detach();
    }

    fn iframe_detached(&mut self, generation: u64, params: &Value) {
        if generation != self.generation {
            return;
        }
        if let Some(session) = params.get("sessionId").and_then(Value::as_str) {
            for page in &mut self.pages {
                page.iframes.retain(|iframe| iframe.session != session);
            }
        }
    }

    fn observed(&mut self, generation: u64, session: &str, method: &str, params: &Value) {
        if generation != self.generation {
            return;
        }
        let Some(target) = self.target_of_session(session) else {
            return;
        };
        if let Some(page) = self.page_state_mut(&target) {
            let recording = page.viewers > 0;
            let now = Instant::now();
            if method.starts_with("Network.") {
                page.network.apply(method, params);
                if recording {
                    record_request(&mut page.recorder, method, params, now);
                }
            } else if let Some(entry) = page.console.apply(method, params)
                && recording
            {
                let entry = RecordedEntry::Console {
                    level: entry.level.clone(),
                    text: entry.text.clone(),
                };
                page.recorder.push(entry, now);
            }
        }
    }

    /// Lays the page `target` out at its tab's size: `size` in logical pixels, at the window's
    /// `scale`.
    fn resize(&mut self, target: &str, size: Size<Pixels>, scale: f32, cx: &Context<Self>) {
        let viewport = Viewport {
            width: whole_pixels(size.width),
            height: whole_pixels(size.height),
            scale,
        };
        self.lay_out(target, viewport, cx);
    }

    /// Lays the page `target` out at `viewport`, unless it is laid out so already.
    fn lay_out(&mut self, target: &str, viewport: Viewport, cx: &Context<Self>) {
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        if page.viewport == Some(viewport) {
            return;
        }
        page.viewport = Some(viewport);
        let session = page.page.clone();
        cx.spawn(async move |_, _| {
            session
                .set_viewport(viewport.width, viewport.height, viewport.scale)
                .await
                .log_err();
        })
        .detach();
        self.sync_screencast(target, cx);
    }

    fn add_viewer(&mut self, target: &str, cx: &Context<Self>) {
        if let Some(page) = self.page_state_mut(target) {
            page.viewers += 1;
        }
        self.sync_screencast(target, cx);
    }

    fn remove_viewer(&mut self, target: &str, cx: &Context<Self>) {
        if let Some(page) = self.page_state_mut(target) {
            page.viewers = page.viewers.saturating_sub(1);
        }
        self.sync_screencast(target, cx);
    }

    /// Streams the page while a tab draws it and its size is known, and stops the stream when
    /// no tab does.
    fn sync_screencast(&mut self, target: &str, cx: &Context<Self>) {
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        let wanted = page.viewers > 0 && page.viewport.is_some();
        if wanted == page.screencasting {
            return;
        }
        page.screencasting = wanted;
        let session = page.page.clone();
        cx.spawn(async move |_, _| {
            let result = if wanted {
                session.start_screencast().await
            } else {
                session.stop_screencast().await
            };
            result.log_err();
        })
        .detach();
    }

    /// Keeps a decoded frame the page of `session` streamed, unless a later start superseded
    /// the stream, and returns the page to acknowledge it to.
    /// Shows a frame the page sent, decoded, with its base64 JPEG, which the page's minute keeps
    /// every half second while a tab draws the page (#499).
    fn show(
        &mut self,
        generation: u64,
        session: &str,
        (decoded, jpeg): (anyhow::Result<Arc<RenderImage>>, String),
        metadata: FrameMetadata,
        cx: &mut Context<Self>,
    ) -> Option<Page> {
        if generation != self.generation {
            return None;
        }
        let page = self
            .pages
            .iter_mut()
            .find(|page| page.page.session_id() == session)?;
        match decoded {
            Ok(image) => {
                page.frame = Some(image);
                page.metadata = Some(metadata);
                if let Some(sent) = page.input_at.take() {
                    input::log_latency(sent);
                }
                let now = Instant::now();
                if page.viewers > 0 && page.recorder.wants_frame(now) {
                    page.recorder.push_frame(Arc::from(jpeg), now);
                }
                cx.notify();
            }
            Err(error) => log::warn!("browser: a screencast frame did not decode: {error:#}"),
        }
        Some(page.page.clone())
    }

    /// Sends `method` with `params` to the page. Each call's message leaves in the order the
    /// calls were made, so input keeps its order. A `timed` input starts the clock the next
    /// frame stops.
    fn send(
        &mut self,
        target: &str,
        method: &'static str,
        params: Value,
        timed: bool,
        cx: &Context<Self>,
    ) {
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        if timed {
            page.input_at.get_or_insert_with(Instant::now);
        }
        let session = page.page.clone();
        cx.spawn(async move |_, _| {
            session.call(method, params).await.log_err();
        })
        .detach();
    }

    fn send_key(&mut self, target: &str, press: KeyPress, cx: &Context<Self>) {
        self.record_entry(target, key_entry(&press));
        self.send(target, "Input.dispatchKeyEvent", press.down, true, cx);
        self.send(target, "Input.dispatchKeyEvent", press.up, false, cx);
    }

    fn insert_text(&mut self, target: &str, text: &str, cx: &Context<Self>) {
        self.record_entry(
            target,
            RecordedEntry::Typed {
                characters: text.chars().count(),
            },
        );
        self.send(
            target,
            "Input.insertText",
            serde_json::json!({ "text": text }),
            true,
            cx,
        );
    }

    fn set_composition(
        &mut self,
        target: &str,
        text: &str,
        selected: Range<usize>,
        cx: &Context<Self>,
    ) {
        self.send(
            target,
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
    fn copy_selection(&mut self, target: &str, press: KeyPress, cx: &Context<Self>) {
        self.record_entry(target, key_entry(&press));
        let Some(page) = self.page(target) else {
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
        target: &str,
        point: (f64, f64),
        button: MouseButton,
        click_count: usize,
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let (name, bit) = input::mouse_button(button);
        self.record_entry(
            target,
            RecordedEntry::Click {
                x: point.0,
                y: point.1,
                button: name.to_string(),
                count: click_count,
            },
        );
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        page.held_buttons |= bit;
        let params = input::mouse_event(
            "mousePressed",
            point,
            name,
            page.held_buttons,
            click_count,
            input::modifier_bits(modifiers),
        );
        self.send(target, "Input.dispatchMouseEvent", params, true, cx);
    }

    fn mouse_release(
        &mut self,
        target: &str,
        point: (f64, f64),
        button: MouseButton,
        click_count: usize,
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let (name, bit) = input::mouse_button(button);
        let Some(page) = self.page_state_mut(target) else {
            return;
        };
        page.held_buttons &= !bit;
        let params = input::mouse_event(
            "mouseReleased",
            point,
            name,
            page.held_buttons,
            click_count,
            input::modifier_bits(modifiers),
        );
        self.send(target, "Input.dispatchMouseEvent", params, false, cx);
    }

    fn mouse_move(
        &mut self,
        target: &str,
        point: (f64, f64),
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        let Some(held) = self.page_state(target).map(|page| page.held_buttons) else {
            return;
        };
        // A move names the button it drags with, the first one held.
        let button = [
            (1, "left"),
            (4, "middle"),
            (2, "right"),
            (8, "back"),
            (16, "forward"),
        ]
        .iter()
        .find(|(bit, _)| held & bit != 0)
        .map_or("none", |(_, name)| name);
        let params = input::mouse_event(
            "mouseMoved",
            point,
            button,
            held,
            0,
            input::modifier_bits(modifiers),
        );
        self.send(target, "Input.dispatchMouseEvent", params, false, cx);
    }

    fn wheel(
        &mut self,
        target: &str,
        point: (f64, f64),
        delta: (f64, f64),
        modifiers: Modifiers,
        cx: &Context<Self>,
    ) {
        self.record_entry(
            target,
            RecordedEntry::Scroll {
                dx: delta.0,
                dy: delta.1,
            },
        );
        let params = input::wheel_event(point, delta, input::modifier_bits(modifiers));
        self.send(target, "Input.dispatchMouseEvent", params, true, cx);
    }

    fn is_holding(&self, target: &str) -> bool {
        self.page_state(target)
            .is_some_and(|page| page.held_buttons != 0)
    }

    fn target_changed(&mut self, generation: u64, info: TargetInfo, cx: &mut Context<Self>) {
        if generation != self.generation {
            return;
        }
        for page in &mut self.pages {
            if let Some(iframe) = page
                .iframes
                .iter_mut()
                .find(|iframe| iframe.frame_id == info.target_id)
            {
                iframe.url = info.url;
                return;
            }
        }
        let Some(page) = self.page_state_mut(&info.target_id) else {
            return;
        };
        let title = Some(SharedString::from(info.title));
        let url = Some(SharedString::from(info.url));
        if title != page.title || url != page.url {
            page.title = title;
            page.url = url;
            cx.emit(BrowserEvent::PageInfoChanged {
                target: info.target_id,
            });
            cx.notify();
        }
    }
}

/// What a call on a page the hub does not have answers.
fn no_page(target: &str) -> String {
    format!("the browser has no tab {target}")
}

/// A length in the window's logical pixels from one CDP gives.
const fn window_pixels(length: f64) -> Pixels {
    #[allow(clippy::cast_possible_truncation)] // a length on a screen fits an f32
    let length = length as f32;
    px(length)
}

/// Rounds a length to the whole CSS pixels CDP takes, between 1 and 16,384.
fn whole_pixels(length: Pixels) -> u32 {
    let clamped = f32::from(length).round().clamp(1.0, 16_384.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 1..=16384
    let whole = clamped as u32;
    whole
}

/// The browser's pages once discovery is on. A start opens none (#494): a tab that wants a page
/// and finds none among these opens its own.
async fn pages_at_start(connection: &Connection) -> Result<Vec<String>, CdpError> {
    Page::discover(connection).await?;
    Page::page_ids(connection).await
}

/// Waits until the hub shows the browser's pages.
///
/// # Errors
///
/// The hub's reason when the browser could not start, or a note when 20 seconds pass first.
pub(crate) async fn showing(hub: &Entity<BrowserHub>, cx: &AsyncApp) -> Result<(), String> {
    let mut waited = Duration::ZERO;
    loop {
        match hub.read_with(cx, |hub, _| hub.state().clone()) {
            HubState::Showing => return Ok(()),
            HubState::Failed(reason) => return Err(reason.to_string()),
            HubState::Starting | HubState::Connecting if waited >= SHOW_WAIT => {
                return Err("the browser did not show its pages within 20 seconds".to_string());
            }
            HubState::Starting | HubState::Connecting => {}
        }
        cx.background_executor().timer(POLL_INTERVAL).await;
        waited += POLL_INTERVAL;
    }
}

/// Opens a page at `url` once the hub shows the browser's pages, and gives its id; its tab opens
/// as any page's does, in `place_in` when given (#574).
///
/// # Errors
///
/// As [`showing`], or the browser's reason when it made no page.
pub(crate) async fn new_page(
    hub: &Entity<BrowserHub>,
    url: String,
    place_in: Option<WeakEntity<Workspace>>,
    cx: &mut AsyncApp,
) -> Result<String, String> {
    showing(hub, cx).await?;
    let task = hub.update(cx, |hub, cx| hub.create_page_task(url, place_in, cx));
    task.await
}

/// Waits until the hub shows the browser's pages, however long that takes: a tab waiting for its
/// page shows why the browser stopped, and waits on for it to be opened again.
async fn shown(hub: &Entity<BrowserHub>, cx: &AsyncApp) {
    while hub.read_with(cx, |hub, _| hub.state() != &HubState::Showing) {
        cx.background_executor().timer(SHOWN_POLL).await;
    }
}

/// Waits, up to ten seconds, while the hub attaches pages a start found and `waiting` holds.
async fn wait_for_start_pages(
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
    waiting: impl Fn(&BrowserHub) -> bool,
) {
    let mut waited = Duration::ZERO;
    while waited < START_PAGES_WAIT
        && hub.read_with(cx, |hub, _| hub.is_attaching() && waiting(hub))
    {
        cx.background_executor().timer(POLL_INTERVAL).await;
        waited += POLL_INTERVAL;
    }
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

/// Follows the browser's events for the start `generation`: frames into their pages, the pages
/// that come and go, their titles and URLs. When the connection ends, the hub fails.
async fn follow(
    this: WeakEntity<BrowserHub>,
    mut events: mpsc::UnboundedReceiver<Event>,
    generation: u64,
    cx: &mut AsyncApp,
) {
    while let Some(event) = events.next().await {
        match event.method.as_str() {
            "Page.screencastFrame" => {
                let Some(session) = event.session_id.clone() else {
                    continue;
                };
                let Ok(frame) = serde_json::from_value::<ScreencastFrame>(event.params) else {
                    continue;
                };
                let number = frame.session_id;
                let metadata = frame.metadata;
                let data = frame.data;
                let (decoded, data) = cx
                    .background_spawn(futures::future::lazy(move |_| (frame::decode(&data), data)))
                    .await;
                match this.update(cx, |this, cx| {
                    this.show(generation, &session, (decoded, data), metadata, cx)
                }) {
                    Ok(Some(page)) => {
                        page.ack_frame(number).await.log_err();
                    }
                    Ok(None) => {}
                    Err(_) => return,
                }
            }
            // The document's own title arrives with its content, which no target event reports.
            "Page.domContentEventFired" | "Page.loadEventFired" => {
                if let Some(target) = page_of_event(&this, &event, cx) {
                    if event.method == "Page.loadEventFired" {
                        let this = this.clone();
                        let target = target.clone();
                        cx.spawn(async move |cx| {
                            record_snapshot(&this, generation, &target, cx).await;
                        })
                        .detach();
                    }
                    refresh_info(&this, generation, &target, cx).await;
                }
            }
            "Target.targetCreated" => {
                let info = event.params.get("targetInfo");
                let kind = info
                    .and_then(|info| info.get("type"))
                    .and_then(Value::as_str);
                let target = info
                    .and_then(|info| info.get("targetId"))
                    .and_then(Value::as_str);
                if let (Some("page"), Some(target)) = (kind, target) {
                    let opener = info
                        .and_then(|info| info.get("openerId"))
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let target = target.to_string();
                    this.update(cx, |this, cx| {
                        this.attach(generation, target, opener, false, cx);
                    })
                    .ok();
                }
            }
            "Target.targetInfoChanged" => {
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
                if let Some(target) = event.params.get("targetId").and_then(Value::as_str) {
                    let target = target.to_string();
                    this.update(cx, |this, cx| this.page_gone(generation, &target, cx))
                        .ok();
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

/// The page an event belongs to, by the session it came from.
fn page_of_event(this: &WeakEntity<BrowserHub>, event: &Event, cx: &AsyncApp) -> Option<String> {
    let session = event.session_id.as_deref()?;
    this.read_with(cx, |this, _| this.target_of_session(session))
        .ok()
        .flatten()
}

/// Follows each page's navigation for the start `generation`: its main frame's commits and
/// loading, and the JavaScript dialogs it opens and closes.
async fn follow_navigation(
    this: &WeakEntity<BrowserHub>,
    event: Event,
    generation: u64,
    cx: &mut AsyncApp,
) {
    let Some(target) = page_of_event(this, &event, cx) else {
        follow_observed(this, &event, generation, cx);
        return;
    };
    match event.method.as_str() {
        // A commit in the main frame, which has no parent. A page that failed to load commits
        // Chromium's error page, which names the URL it could not load.
        "Page.frameNavigated" => {
            if event.params.pointer("/frame/parentId").is_none() {
                let url = ["/frame/unreachableUrl", "/frame/url"]
                    .into_iter()
                    .find_map(|pointer| event.params.pointer(pointer).and_then(Value::as_str))
                    .map(str::to_string);
                this.update(cx, |this, cx| {
                    this.left_document(generation, &target, cx);
                    this.record_navigation(generation, &target, url.as_deref());
                })
                .ok();
                refresh_history(this, generation, &target, url, cx).await;
            }
        }
        // A fragment or the history API moved the main frame within its document.
        "Page.navigatedWithinDocument" => {
            let frame = event.params.get("frameId").and_then(Value::as_str);
            if frame == Some(target.as_str()) {
                let url = event
                    .params
                    .get("url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                // A move within the document is all the load a waiting call will see.
                this.update(cx, |this, _| this.within_document(&target))
                    .ok();
                refresh_history(this, generation, &target, url, cx).await;
                refresh_info(this, generation, &target, cx).await;
            }
        }
        "Page.frameStartedLoading" | "Page.frameStoppedLoading" => {
            let loading = event.method == "Page.frameStartedLoading";
            let frame = event.params.get("frameId").and_then(Value::as_str);
            this.update(cx, |this, cx| {
                this.loading_changed(generation, &target, frame, loading, cx);
            })
            .ok();
        }
        "Page.javascriptDialogOpening" => {
            match serde_json::from_value::<JavaScriptDialog>(event.params) {
                Ok(dialog) => {
                    this.update(cx, |this, cx| {
                        this.dialog_opened(generation, &target, dialog, cx);
                    })
                    .ok();
                }
                Err(error) => log::warn!("browser: a dialog the page opened: {error}"),
            }
        }
        "Page.javascriptDialogClosed" => {
            this.update(cx, |this, cx| this.dialog_closed(generation, &target, cx))
                .ok();
        }
        "Overlay.inspectNodeRequested" => {
            if let Some(node) = event.params.get("backendNodeId").and_then(Value::as_i64) {
                this.update(cx, |this, cx| {
                    this.pick_requested(generation, &target, node, cx);
                })
                .ok();
            }
        }
        "Overlay.inspectModeCanceled" => {
            this.update(cx, |this, cx| {
                this.inspect_canceled(generation, &target, cx);
            })
            .ok();
        }
        "Debugger.scriptParsed" => {
            this.update(cx, |this, _| {
                this.script_parsed(generation, &target, &event.params);
            })
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
            if let Some(session) = event.session_id.as_deref() {
                this.update(cx, |this, _| {
                    this.observed(generation, session, method, &event.params);
                })
                .ok();
            }
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
        "Runtime.bindingCalled" => {
            if let Some(session) = event.session_id.as_deref() {
                this.update(cx, |this, cx| {
                    this.select_requested(generation, session, &event.params, cx);
                })
                .ok();
            }
        }
        _ => {}
    }
}

/// Keeps the page's snapshot in its minute after a load, while a tab draws the page (#499).
async fn record_snapshot(
    this: &WeakEntity<BrowserHub>,
    generation: u64,
    target: &str,
    cx: &mut AsyncApp,
) {
    let page = this
        .read_with(cx, |this, _| {
            this.page_state(target)
                .filter(|page| this.generation == generation && page.viewers > 0)
                .map(|page| page.page.clone())
        })
        .ok()
        .flatten();
    let Some(page) = page else {
        return;
    };
    if let Some(text) = snapshot_text(&page).await {
        this.update(cx, |this, _| {
            this.record_entry(target, RecordedEntry::Snapshot { text });
        })
        .ok();
    }
}

/// The page's main frame as `snapshot::render` writes it for the recorder: its interactive
/// nodes, and no field's value.
async fn snapshot_text(page: &Page) -> Option<String> {
    let nodes = page
        .accessibility_tree(page.session_id(), None)
        .await
        .log_err()?;
    let trees = [FrameTree {
        session: None,
        frame_id: None,
        label: None,
        nodes,
    }];
    Some(snapshot::render(&trees, false).text)
}

/// What the recorder keeps of a request's events: its start, then its status or its failure.
fn record_request(recorder: &mut Recorder, method: &str, params: &Value, now: Instant) {
    let Some(id) = params.get("requestId").and_then(Value::as_str) else {
        return;
    };
    match method {
        "Network.requestWillBeSent" => {
            let url = params
                .pointer("/request/url")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let request_method = params
                .pointer("/request/method")
                .and_then(Value::as_str)
                .unwrap_or("GET");
            recorder.push(
                RecordedEntry::Request {
                    id: id.to_string(),
                    method: request_method.to_string(),
                    url: redact_url(url),
                    status: None,
                    failure: None,
                },
                now,
            );
        }
        "Network.responseReceived" => {
            if let Some(status) = params.pointer("/response/status").and_then(Value::as_u64) {
                recorder.request_ended(id, Ok(status));
            }
        }
        "Network.loadingFailed" => {
            let reason = params
                .get("errorText")
                .and_then(Value::as_str)
                .unwrap_or("failed");
            recorder.request_ended(id, Err(reason.to_string()));
        }
        _ => {}
    }
}

/// What the recorder keeps of a key (#499): a key that types a character counts as one, and the
/// character is never kept; a shortcut by its name and modifiers.
fn key_entry(press: &KeyPress) -> RecordedEntry {
    const ALT: u64 = 1;
    const CTRL: u64 = 2;
    const META: u64 = 4;
    const SHIFT: u64 = 8;
    let key = press
        .down
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let modifiers = press
        .down
        .get("modifiers")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let types = press
        .down
        .get("text")
        .and_then(Value::as_str)
        .is_some_and(|text| !text.is_empty() && text != "\r");
    let mut characters = key.chars();
    let single = characters.next().filter(|_| characters.next().is_none());
    // A character a layout's AltGr types arrives as a chord: it is typing too.
    let typed_by_chord = single.is_some_and(|character| !character.is_ascii_alphanumeric());
    if types || typed_by_chord {
        return RecordedEntry::Typed { characters: 1 };
    }
    let mut name: String = [
        (CTRL, "Ctrl+"),
        (ALT, "Alt+"),
        (META, "Meta+"),
        (SHIFT, "Shift+"),
    ]
    .iter()
    .filter(|(bit, _)| modifiers & bit != 0)
    .map(|(_, label)| *label)
    .collect();
    match single {
        Some(character) => name.push(character.to_ascii_uppercase()),
        None => name.push_str(key),
    }
    RecordedEntry::Key { key: name }
}

/// Where Record this saves the pages' minutes: Marley's data directory, never a project (#499).
pub(crate) fn recordings_dir() -> PathBuf {
    paths::data_dir().join("browser").join("recordings")
}

/// Reads the page's title and URL into the hub.
async fn refresh_info(
    this: &WeakEntity<BrowserHub>,
    generation: u64,
    target: &str,
    cx: &mut AsyncApp,
) {
    let page = this
        .read_with(cx, |this, _| this.page(target))
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
    target: &str,
    url: Option<String>,
    cx: &mut AsyncApp,
) {
    let page = this
        .read_with(cx, |this, _| this.page(target))
        .ok()
        .flatten();
    let Some(page) = page else {
        return;
    };
    let history = page.history().await.log_err();
    this.update(cx, |this, cx| {
        this.navigated(generation, target, url, history, cx);
    })
    .ok();
}

/// Waits until `loaded` hears the load end, or 15 seconds pass.
async fn wait_for_load(loaded: oneshot::Receiver<()>, cx: &AsyncApp) {
    let timeout = cx.background_executor().timer(LOAD_TIMEOUT);
    futures::future::select(loaded, timeout).await;
}

/// Every Browser tab that is still alive.
fn live_views(cx: &mut App) -> Vec<Entity<BrowserView>> {
    let views = cx.default_global::<BrowserViews>();
    views.0.retain(|view| view.upgrade().is_some());
    views.0.iter().filter_map(WeakEntity::upgrade).collect()
}

/// Each Browser tab in a pane that shows a page, with the page and the tab's workspace (#574).
pub(crate) fn tab_workspaces(cx: &mut App) -> Vec<(String, WeakEntity<Workspace>)> {
    live_views(cx)
        .iter()
        .filter(|view| in_a_pane(view, cx))
        .filter_map(|view| {
            let view = view.read(cx);
            Some((view.target.clone()?, view.workspace.clone()))
        })
        .collect()
}

/// The tab showing the page `target`.
fn view_of(target: &str, cx: &mut App) -> Option<Entity<BrowserView>> {
    live_views(cx)
        .into_iter()
        .find(|view| view.read(cx).target.as_deref() == Some(target))
}

/// The pane of `workspace` that holds `view`.
fn pane_of(
    workspace: &Entity<Workspace>,
    view: &Entity<BrowserView>,
    cx: &App,
) -> Option<Entity<Pane>> {
    workspace
        .read(cx)
        .panes()
        .iter()
        .find(|pane| pane.read(cx).index_for_item(view).is_some())
        .cloned()
}

/// Whether a pane of its workspace holds `view`.
fn in_a_pane(view: &Entity<BrowserView>, cx: &App) -> bool {
    view.read(cx)
        .workspace
        .upgrade()
        .is_some_and(|workspace| pane_of(&workspace, view, cx).is_some())
}

/// The window that holds `workspace`.
pub(crate) fn window_of(
    workspace: &Entity<Workspace>,
    cx: &App,
) -> Option<WindowHandle<MultiWorkspace>> {
    cx.windows()
        .into_iter()
        .filter_map(|window| window.downcast::<MultiWorkspace>())
        .find(|window| {
            window.read(cx).is_ok_and(|multi_workspace| {
                multi_workspace
                    .workspaces()
                    .any(|candidate| candidate == workspace)
            })
        })
}

/// The window a tab with no Browser tab to go beside opens in: the active window, else the
/// first.
fn active_multi_workspace(cx: &App) -> Option<WindowHandle<MultiWorkspace>> {
    cx.active_window()
        .and_then(|window| window.downcast::<MultiWorkspace>())
        .or_else(|| {
            cx.windows()
                .into_iter()
                .find_map(|window| window.downcast::<MultiWorkspace>())
        })
}

fn new_view(
    hub: Entity<BrowserHub>,
    target: Option<String>,
    adopts: bool,
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<BrowserView> {
    cx.new(|cx| BrowserView::new(hub, target, adopts, workspace, window, cx))
}

/// A page was attached. A tab that claims it shows it. A page a start found goes to a tab opened
/// while the browser started, or else waits without a tab until one claims it or the user opens
/// the browser (#494); any other page gets a tab of its own.
fn open_tab(
    hub: &Entity<BrowserHub>,
    target: &str,
    opener: Option<&str>,
    focus: bool,
    listed: bool,
    cx: &mut App,
) {
    let views = live_views(cx);
    if views
        .iter()
        .any(|view| view.read(cx).target.as_deref() == Some(target))
    {
        return;
    }
    if !listed {
        let beside = opener.and_then(|opener| {
            views
                .iter()
                .find(|view| view.read(cx).target.as_deref() == Some(opener))
                .cloned()
        });
        place_tab(hub, target, beside.as_ref(), focus, cx);
        return;
    }
    let waiting = views.iter().find(|view| {
        let view = view.read(cx);
        view.target.is_none() && view.adopts
    });
    if let Some(view) = waiting {
        let window = view.read(cx).window;
        let view = view.clone();
        let target = target.to_string();
        window
            .update(cx, |_, window, cx| {
                view.update(cx, |view, cx| view.show_page(target, window, cx));
            })
            .log_err();
    }
}

/// Opens a tab for the page `target`: beside `beside`, the tab of the page that opened it, or
/// else after the tab the user focused last, or else in the active workspace. A page an agent's
/// call asked for in a workspace (#574) looks only among that workspace's tabs, and else opens
/// in that workspace. A tab that does not take the focus leaves the tab that has it in front.
fn place_tab(
    hub: &Entity<BrowserHub>,
    target: &str,
    beside: Option<&Entity<BrowserView>>,
    focus: bool,
    cx: &mut App,
) {
    let placed_in = hub
        .update(cx, |hub, _| hub.take_placement(target))
        .and_then(|workspace| workspace.upgrade());
    let views: Vec<Entity<BrowserView>> = live_views(cx)
        .into_iter()
        .filter(|view| {
            placed_in
                .as_ref()
                .is_none_or(|placed_in| view.read(cx).workspace == *placed_in)
        })
        .collect();
    let focused = {
        let hub = hub.read(cx);
        if placed_in.is_some() {
            let targets = views
                .iter()
                .filter_map(|view| view.read(cx).target.clone())
                .collect();
            hub.focused_among(&targets)
        } else {
            hub.focused()
        }
    };
    let anchor = beside
        .cloned()
        .or_else(|| {
            views
                .iter()
                .find(|view| focused.is_some() && view.read(cx).target == focused)
                .cloned()
        })
        .or_else(|| {
            views
                .iter()
                .rev()
                .find(|view| view.read(cx).target.is_some())
                .cloned()
        });
    if let Some(anchor) = anchor {
        let (workspace, window) = {
            let anchor = anchor.read(cx);
            (anchor.workspace.clone(), anchor.window)
        };
        if let Some(workspace_entity) = workspace.upgrade()
            && let Some(pane) = pane_of(&workspace_entity, &anchor, cx)
        {
            let index = pane.read(cx).index_for_item(&anchor).map(|index| index + 1);
            let hub = hub.clone();
            let target = target.to_string();
            window
                .update(cx, |_, window, cx| {
                    let view = new_view(hub, Some(target), false, workspace, window, cx);
                    add_to_pane(&pane, view, index, focus, window, cx);
                })
                .log_err();
            return;
        }
    }
    let window = placed_in.as_ref().map_or_else(
        || active_multi_workspace(cx),
        |workspace| window_of(workspace, cx),
    );
    let Some(window) = window else {
        log::warn!("browser: no window to open a page's tab in");
        return;
    };
    let hub = hub.clone();
    let target = target.to_string();
    window
        .update(cx, |multi_workspace, window, cx| {
            let workspace = placed_in.unwrap_or_else(|| multi_workspace.workspace().clone());
            let view = new_view(hub, Some(target), false, workspace.downgrade(), window, cx);
            workspace.update(cx, |workspace, cx| {
                let pane = workspace.active_pane().clone();
                let busy = !focus
                    && pane.read(cx).active_item().is_some()
                    && pane.focus_handle(cx).contains_focused(window, cx);
                if busy {
                    // The pane with the focus shows other work, such as the agent's own
                    // terminal: the page opens beside it, and the focus stays.
                    let focused = window.focused(cx);
                    let new_pane = workspace.split_pane(pane, SplitDirection::Right, window, cx);
                    new_pane.update(cx, |pane, cx| {
                        pane.add_item(Box::new(view), false, false, None, window, cx);
                    });
                    if let Some(focused) = focused {
                        window.focus(&focused, cx);
                    }
                } else {
                    add_to_pane(&pane, view, None, focus, window, cx);
                }
            });
        })
        .log_err();
}

/// Adds `view` to `pane` at `index`, or after its active tab. A tab that does not take the
/// focus stays behind the pane's active tab while the pane has the focus: in front, it would
/// take the focus from it.
fn add_to_pane(
    pane: &Entity<Pane>,
    view: Entity<BrowserView>,
    index: Option<usize>,
    focus: bool,
    window: &mut Window,
    cx: &mut App,
) {
    pane.update(cx, |pane, cx| {
        let kept = if !focus && pane.focus_handle(cx).contains_focused(window, cx) {
            pane.active_item()
        } else {
            None
        };
        pane.add_item(Box::new(view), focus, focus, index, window, cx);
        if let Some(kept) = kept
            && let Some(index) = pane.index_for_item(kept.as_ref())
        {
            pane.activate_item(index, false, false, window, cx);
        }
    });
}

/// Brings `view` to the front of its pane, unless the pane has the focus, which it would take.
fn reveal(view: &Entity<BrowserView>, cx: &mut App) {
    let (workspace, window) = {
        let view = view.read(cx);
        (view.workspace.upgrade(), view.window)
    };
    let Some(pane) = workspace.and_then(|workspace| pane_of(&workspace, view, cx)) else {
        return;
    };
    let view = view.clone();
    window
        .update(cx, |_, window, cx| {
            pane.update(cx, |pane, cx| {
                let Some(index) = pane.index_for_item(&view) else {
                    return;
                };
                if index != pane.active_item_index()
                    && !pane.focus_handle(cx).contains_focused(window, cx)
                {
                    pane.activate_item(index, false, false, window, cx);
                }
            });
        })
        .log_err();
}

/// The page `target` closed: each tab of it closes.
fn close_tabs(target: &str, cx: &mut App) {
    for view in live_views(cx) {
        let (shows, workspace, window) = {
            let view = view.read(cx);
            (
                view.target.as_deref() == Some(target),
                view.workspace.upgrade(),
                view.window,
            )
        };
        if !shows {
            continue;
        }
        // The tab's removal must not close the page again.
        view.update(cx, |view, _| view.forget_page());
        let Some(pane) = workspace.and_then(|workspace| pane_of(&workspace, &view, cx)) else {
            continue;
        };
        let item_id = view.entity_id();
        window
            .update(cx, |_, window, cx| {
                pane.update(cx, |pane, cx| {
                    pane.remove_item(item_id, false, true, window, cx);
                });
            })
            .log_err();
    }
}

/// Opens a page at `url` for `view`, a tab in `window`, which shows it once the browser has made
/// it. A tab closed by then takes its page with it.
fn open_page_in(
    hub: &Entity<BrowserHub>,
    view: WeakEntity<BrowserView>,
    window: AnyWindowHandle,
    url: String,
    cx: &App,
) {
    let hub = hub.clone();
    cx.spawn(async move |cx| {
        let created = new_page(&hub, url, None, cx).await;
        let shown = window
            .update(cx, |_, window, cx| {
                view.update(cx, |view, cx| {
                    view.page_created(created.clone(), window, cx);
                })
            })
            .and_then(|shown| shown);
        if shown.is_err()
            && let Ok(target) = created
        {
            hub.update(cx, |hub, cx| hub.close_page(&target, cx));
        }
    })
    .detach();
}

/// The Browser tab: one page of Marley's Chromium, drawn from its newest frame, under a toolbar
/// with the address bar.
pub struct BrowserView {
    hub: Entity<BrowserHub>,
    /// The page it shows; none while it waits for one.
    target: Option<String>,
    /// Whether it takes the first page a start attaches: a tab opened while the browser starts.
    adopts: bool,
    /// Whether it is a tab restored at launch whose page is not back yet (#494).
    restoring: bool,
    /// The URL and title it was saved with, shown until its page is back.
    saved_url: Option<String>,
    saved_title: Option<SharedString>,
    /// The task that waits for the browser to give the tab its page, dropped with the tab.
    waiting: Option<Task<()>>,
    /// Why its page did not open.
    open_error: Option<SharedString>,
    /// An address the user went to before the page was attached, gone to once it is.
    queued_url: Option<String>,
    workspace: WeakEntity<Workspace>,
    window: AnyWindowHandle,
    /// Whether it counts among its page's viewers: from its first paint in front of its pane
    /// until it goes behind or closes.
    viewing: bool,
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
    /// Where the page's frame sat in the window at the last paint, which places a select's list.
    mapping: Option<PageMapping>,
    /// When the user last pressed in the page, or sent it a key: a select that opens right after
    /// is the user's.
    pressed_at: Option<Instant>,
    /// The list of the page's open select (#495).
    select_menu: Option<SelectMenu>,
    /// The caption fields of the page's staged picks, by pick (#496).
    captions: HashMap<usize, Entity<Editor>>,
    /// What went wrong with the last Send, or the last listener's file opened.
    tray_error: Option<SharedString>,
    /// Annotate mode, and the annotation being drawn (#498).
    annotate_mode: AnnotateMode,
    /// The annotation whose note the user clicked, which Delete removes.
    selected_annotation: Option<usize>,
    _subscriptions: Vec<Subscription>,
}

/// Annotate mode (#498): a drag in the page reaches the page, draws a box, or has drawn one that
/// waits for its note.
enum AnnotateMode {
    Off,
    On,
    /// A drag between two document points.
    Dragging {
        from: (f64, f64),
        to: (f64, f64),
    },
    /// A box and the field for its note.
    Noting {
        page_box: PageBox,
        note: Entity<Editor>,
    },
}

/// A pick as its tray row shows it.
struct TrayRow {
    id: usize,
    summary: String,
    /// The caption it was sent with, once sent.
    sent: Option<String>,
    place: Option<ListenerPlace>,
}

/// What a tray row says of its pick's listeners (#497): the first whose file is in the project,
/// else the first with a script.
struct ListenerPlace {
    event: String,
    /// Where it is, as the row shows it.
    label: String,
    /// The listener whose file a click opens.
    opens: Option<usize>,
    /// Every listener and its place, one a line.
    all: String,
}

/// A select's list, drawn by the tab over the page where the select's popup would be.
struct SelectMenu {
    menu: Entity<ContextMenu>,
    /// Its top left, in the window: under the select, at its left edge.
    position: Point<Pixels>,
    _dismissed: Subscription,
}

impl fmt::Debug for BrowserView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BrowserView")
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl BrowserView {
    fn new(
        hub: Entity<BrowserHub>,
        target: Option<String>,
        adopts: bool,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let address_bar = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Search or enter an address", window, cx);
            editor
        });
        let prompt_field = cx.new(|cx| Editor::single_line(window, cx));
        let focus_handle = cx.focus_handle();
        let subscriptions = vec![
            cx.observe(&hub, |_, _, cx| cx.notify()),
            cx.subscribe_in(&hub, window, |this, _, event, window, cx| {
                this.hub_event(event, window, cx);
            }),
            // The address bar shows the page's URL again once the focus leaves it.
            cx.on_focus_out(
                &address_bar.focus_handle(cx),
                window,
                |this, _, window, cx| this.show_address(window, cx),
            ),
            // While the page waits on a dialog, the page's focus goes to the dialog.
            cx.on_focus(&focus_handle, window, |this, window, cx| {
                if this.dialog(cx).is_some() {
                    this.focus_dialog(window, cx);
                }
            }),
            // The agent tools act on the page whose tab the user focused last.
            cx.on_focus_in(&focus_handle, window, |this, _, cx| this.mark_focused(cx)),
        ];
        let this = cx.weak_entity();
        cx.default_global::<BrowserViews>().0.push(this);
        // A tab opened while the browser starts takes a page the start finds, and opens a blank
        // one when the start brings none, after a failure and the start that follows it too.
        let waiting = adopts.then(|| {
            let hub = hub.clone();
            cx.spawn(async move |this, cx| {
                shown(&hub, cx).await;
                wait_for_start_pages(&hub, cx, |_| true).await;
                let Ok(Some(window)) = this.read_with(cx, |this, _| {
                    (this.target.is_none() && this.adopts).then_some(this.window)
                }) else {
                    return;
                };
                this.update(cx, |this, _| this.adopts = false).ok();
                cx.update(|cx| open_page_in(&hub, this, window, BLANK.to_string(), cx));
            })
        });
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
            if this.viewing
                && let Some(target) = &this.target
            {
                this.hub.update(cx, |hub, cx| hub.remove_viewer(target, cx));
            }
        })
        .detach();
        let view = Self {
            hub,
            target,
            adopts,
            restoring: false,
            saved_url: None,
            saved_title: None,
            waiting,
            open_error: None,
            queued_url: None,
            workspace,
            window: window_handle,
            viewing: false,
            focus_handle,
            address_bar,
            dialog_focus: cx.focus_handle(),
            prompt_field,
            current_frame: None,
            previous_frame: None,
            last_press: None,
            marked: String::new(),
            mapping: None,
            pressed_at: None,
            select_menu: None,
            captions: HashMap::new(),
            tray_error: None,
            annotate_mode: AnnotateMode::Off,
            selected_annotation: None,
            _subscriptions: subscriptions,
        };
        view.show_address(window, cx);
        view
    }

    /// Brings back a tab restored at launch (#494), which claims its saved page already: it
    /// shows the saved title and URL until its page is back, keeps the page when the browser
    /// still has it, and otherwise opens the saved URL in a new page.
    fn restore(&mut self, url: String, title: String, window: &mut Window, cx: &mut Context<Self>) {
        self.restoring = true;
        self.saved_title = (!title.is_empty()).then(|| SharedString::from(title));
        self.saved_url = Some(url.clone());
        self.show_address(window, cx);
        let Some(target) = self.target.clone() else {
            return;
        };
        let hub = self.hub.clone();
        let window_handle = self.window;
        self.waiting = Some(cx.spawn(async move |this, cx| {
            shown(&hub, cx).await;
            wait_for_start_pages(&hub, cx, |hub| hub.page(&target).is_none()).await;
            let back = hub.read_with(cx, |hub, _| hub.page(&target).is_some());
            let settled = window_handle.update(cx, |_, window, cx| {
                this.update(cx, |this, cx| {
                    this.restoring = false;
                    this.page_ready(window, cx);
                    cx.notify();
                })
            });
            if !back && matches!(settled, Ok(Ok(()))) {
                // The browser no longer has the page, after a restart say: its URL opens again.
                cx.update(|cx| open_page_in(&hub, this, window_handle, url, cx));
            }
        }));
    }

    /// What the tab saves: its page's id, and the page's URL and title, or the ones it was saved
    /// with while its page is not back.
    fn saved(&self, cx: &App) -> Option<(String, String, String)> {
        let target = self.target.clone()?;
        let hub = self.hub.read(cx);
        let url = hub
            .url(&target)
            .map(|url| url.to_string())
            .or_else(|| self.saved_url.clone())
            .unwrap_or_else(|| BLANK.to_string());
        let title = hub
            .title(&target)
            .or_else(|| self.saved_title.clone())
            .map(|title| title.to_string())
            .unwrap_or_default();
        Some((target, url, title))
    }

    /// Starts showing the page `target`, attached or about to be.
    fn show_page(&mut self, target: String, window: &mut Window, cx: &mut Context<Self>) {
        self.target = Some(target);
        self.adopts = false;
        self.open_error = None;
        if self.focus_handle.contains_focused(window, cx) {
            self.mark_focused(cx);
        }
        self.page_ready(window, cx);
        cx.emit(ItemEvent::UpdateTab);
        cx.notify();
    }

    fn page_created(
        &mut self,
        created: Result<String, String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match created {
            Ok(target) => self.show_page(target, window, cx),
            Err(error) => {
                self.open_error = Some(format!("Could not open a page: {error}").into());
                cx.notify();
            }
        }
    }

    /// Once the page is attached: goes where the user asked to go before it was, and shows the
    /// page's address.
    fn page_ready(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.attached_target(cx) else {
            return;
        };
        self.saved_url = None;
        self.saved_title = None;
        if let Some(url) = self.queued_url.take() {
            self.hub
                .update(cx, |hub, cx| hub.navigate(&target, url, cx));
        }
        self.show_address(window, cx);
    }

    /// The page it shows, once the hub has it.
    fn attached_target(&self, cx: &App) -> Option<String> {
        self.target
            .clone()
            .filter(|target| self.hub.read(cx).page_state(target).is_some())
    }

    fn mark_focused(&self, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, _| hub.set_focused(&target));
        }
    }

    fn dialog(&self, cx: &App) -> Option<JavaScriptDialog> {
        let target = self.target.as_deref()?;
        self.hub.read(cx).dialog(target).cloned()
    }

    fn hub_event(&mut self, event: &BrowserEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        match event {
            BrowserEvent::PageOpened { target: opened, .. } if *opened == target => {
                self.page_ready(window, cx);
                cx.emit(ItemEvent::UpdateTab);
            }
            BrowserEvent::PageInfoChanged { target: changed } if *changed == target => {
                cx.emit(ItemEvent::UpdateTab);
                self.show_address(window, cx);
            }
            BrowserEvent::DialogOpened { target: opened } if *opened == target => {
                let default = self
                    .dialog(cx)
                    .map(|dialog| dialog.default_prompt)
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
            BrowserEvent::SelectOpened { target: opened } if *opened == target => {
                self.open_select(window, cx);
            }
            // The user picked in this tab: the pick's caption takes the focus.
            BrowserEvent::PickStaged { target: picked, id } if *picked == target => {
                let caption = self.caption(*id, window, cx);
                if self.focus_handle.contains_focused(window, cx) {
                    window.focus(&caption.focus_handle(cx), cx);
                }
            }
            BrowserEvent::DialogClosed { target: closed }
                if *closed == target
                    && (self.dialog_focus.contains_focused(window, cx)
                        || self
                            .prompt_field
                            .focus_handle(cx)
                            .contains_focused(window, cx)) =>
            {
                window.focus(&self.focus_handle, cx);
            }
            _ => {}
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
        let address = self
            .queued_url
            .clone()
            .map(SharedString::from)
            .or_else(|| {
                let target = self.target.as_deref()?;
                self.hub.read(cx).address(target)
            })
            .or_else(|| self.saved_url.clone().map(SharedString::from))
            .unwrap_or_default();
        self.address_bar.update(cx, |editor, cx| {
            if editor.text(cx) != *address {
                editor.set_text(address.to_string(), window, cx);
            }
        });
    }

    /// Gives the focus to the page's dialog: its field for a `prompt`, else the card.
    fn focus_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let is_prompt = self
            .dialog(cx)
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
            match self.attached_target(cx) {
                Some(target) => self
                    .hub
                    .update(cx, |hub, cx| hub.navigate(&target, url, cx)),
                None => self.queued_url = Some(url),
            }
        }
        self.restore_address(&RestoreAddress, window, cx);
    }

    fn restore_address(&mut self, _: &RestoreAddress, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
        self.show_address(window, cx);
    }

    fn back(&mut self, _: &BrowserBack, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, cx| hub.go(&target, -1, cx));
        }
    }

    fn forward(&mut self, _: &BrowserForward, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, cx| hub.go(&target, 1, cx));
        }
    }

    fn reload(&mut self, _: &BrowserReload, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, cx| hub.reload(&target, cx));
        }
    }

    fn stop(&self, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, cx| hub.stop(&target, cx));
        }
    }

    fn answer_dialog(&mut self, _: &AnswerDialog, _: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let is_prompt = self
            .dialog(cx)
            .is_some_and(|dialog| dialog.kind == DialogKind::Prompt);
        let prompt_text = is_prompt.then(|| self.prompt_field.read(cx).text(cx));
        self.hub.update(cx, |hub, cx| {
            hub.answer_dialog(&target, true, prompt_text, cx);
        });
    }

    fn dismiss_dialog(&mut self, _: &DismissDialog, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub
                .update(cx, |hub, cx| hub.answer_dialog(&target, false, None, cx));
        }
    }

    /// Counts the tab among its page's viewers, from its first paint in front of its pane.
    fn start_viewing(&mut self, cx: &mut Context<Self>) {
        if self.viewing {
            return;
        }
        if let Some(target) = self.target.clone() {
            self.viewing = true;
            self.hub.update(cx, |hub, cx| hub.add_viewer(&target, cx));
        }
    }

    /// Lets go of a page that went away, as its tab closes.
    fn forget_page(&mut self) {
        self.target = None;
        self.viewing = false;
    }

    fn stop_viewing(&mut self, cx: &mut Context<Self>) {
        if !self.viewing {
            return;
        }
        self.viewing = false;
        if let Some(target) = self.target.clone() {
            self.hub
                .update(cx, |hub, cx| hub.remove_viewer(&target, cx));
        }
    }

    /// Shows the list of the select the page's listener reported (#495), when the user opened
    /// it: the user pressed in the page, or sent it a key, a moment before.
    fn open_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let Some(request) = self
            .hub
            .read(cx)
            .select(&target)
            .map(|select| select.request.clone())
        else {
            return;
        };
        let pressed = self
            .pressed_at
            .is_some_and(|pressed_at| pressed_at.elapsed() < USER_PRESS);
        let position = self.select_position(&request);
        let (Some(position), true) = (position, pressed) else {
            // An agent's click, say: the select stays shut, and focused for keys.
            self.hub.update(cx, |hub, _| hub.dismiss_select(&target));
            return;
        };
        let view = cx.weak_entity();
        let menu = ContextMenu::build(window, cx, |mut menu, _, _| {
            let mut group = None;
            for (index, option) in request.options.iter().enumerate() {
                if option.group != group {
                    menu = match &option.group {
                        Some(label) => menu.header(label.clone()),
                        None => menu.separator(),
                    };
                    group.clone_from(&option.group);
                }
                let view = view.clone();
                menu = menu.toggleable_entry_disabled_when(
                    option.text.clone(),
                    request.selected == Some(index),
                    option.disabled,
                    IconPosition::Start,
                    None,
                    move |window, cx| {
                        view.update(cx, |view, cx| view.choose_option(index, window, cx))
                            .ok();
                    },
                );
            }
            menu
        });
        menu.update(cx, |menu, cx| menu.select_toggled_or_first(window, cx));
        window.focus(&menu.focus_handle(cx), cx);
        let dismissed = cx.subscribe_in(&menu, window, |this, _, _: &DismissEvent, window, cx| {
            this.close_select(window, cx);
        });
        self.select_menu = Some(SelectMenu {
            menu,
            position,
            _dismissed: dismissed,
        });
        cx.notify();
    }

    /// Where a select's list goes in the window: under the select, at its left edge. The select's
    /// box is in its frame's viewport; a press, known in the page and in the frame, gives the
    /// frame's place in the page, and a key leaves the box as the main frame's.
    fn select_position(&self, request: &SelectRequest) -> Option<Point<Pixels>> {
        let mapping = self.mapping?;
        let (offset_x, offset_y) = match (request.press_x, request.press_y, self.last_press) {
            (Some(press_x), Some(press_y), Some(press)) => {
                let (page_x, page_y) = mapping.map(press);
                (page_x - press_x, page_y - press_y)
            }
            _ => (0.0, 0.0),
        };
        Some(mapping.to_window((
            request.left + offset_x,
            request.top + request.height + offset_y,
        )))
    }

    /// The user chose option `index` from a select's list.
    fn choose_option(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub
                .update(cx, |hub, cx| hub.choose_option(&target, index, cx));
        }
        self.select_menu = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// A select's list closed with no choice: the page keeps its value.
    fn close_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, _| hub.dismiss_select(&target));
        }
        self.select_menu = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Turns pick mode on or off for the tab's page, the page taking the focus so Escape ends it.
    fn pick_element(&mut self, _: &PickElement, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let on = !self.hub.read(cx).is_picking(&target);
        self.hub
            .update(cx, |hub, cx| hub.set_picking(&target, on, cx));
        if on {
            self.tray_error = None;
            window.focus(&self.focus_handle, cx);
        }
    }

    /// The caption field of the pick `id`, made the first time it is asked for.
    fn caption(
        &mut self,
        id: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<Editor> {
        self.captions
            .entry(id)
            .or_insert_with(|| {
                cx.new(|cx| {
                    let mut editor = Editor::single_line(window, cx);
                    editor.set_placeholder_text(
                        "What should the agent know? Enter sends",
                        window,
                        cx,
                    );
                    editor
                })
            })
            .clone()
    }

    /// Types the pick `id`'s reference and its caption into the terminal the user used last, and
    /// takes the user there.
    fn send_pick(&mut self, id: usize, window: &Window, cx: &mut Context<Self>) {
        let Some((summary, url)) = self
            .hub
            .read(cx)
            .pick(id)
            .map(|pick| (pick.summary.clone(), pick.url.clone()))
        else {
            return;
        };
        let terminal = cx
            .try_global::<LastTerminal>()
            .and_then(|last| Some((last.view.upgrade()?, last.window)));
        let Some((terminal, terminal_window)) = terminal else {
            self.tray_error = Some(SharedString::new_static(
                "No terminal to send to: click in one, then Send.",
            ));
            cx.notify();
            return;
        };
        let caption = self
            .captions
            .remove(&id)
            .map(|field| field.read(cx).text(cx).trim().to_string())
            .unwrap_or_default();
        let line = pick_line(id, &summary, &url, &caption);
        self.tray_error = None;
        self.hub
            .update(cx, |hub, cx| hub.pick_sent(id, caption, cx));
        let browser_window = window.window_handle();
        // The terminal's pane may hold this tab, which activating the terminal updates: so after
        // this update.
        cx.defer(move |cx| {
            terminal_window
                .update(cx, |_, window, cx| {
                    if terminal_window != browser_window {
                        window.activate_window();
                    }
                    reveal_terminal(&terminal, window, cx);
                    window.focus(&terminal.focus_handle(cx), cx);
                    let terminal = terminal.read(cx).terminal().clone();
                    terminal.update(cx, |terminal, _| terminal.paste(&line));
                })
                .log_err();
        });
    }

    /// Opens the file a pick's listener was written in, at its line, in the tab's workspace.
    fn open_pick_source(
        &mut self,
        id: usize,
        listener: usize,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let Some(original) = self
            .hub
            .read(cx)
            .pick(id)
            .and_then(|pick| pick.bundle.listeners.get(listener))
            .and_then(|listener| listener.original.clone())
        else {
            return;
        };
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let Some(path) = find_source(workspace.read(cx).project().read(cx), &original.source, cx)
        else {
            let name = original.file.unwrap_or(original.source);
            self.tray_error = Some(format!("{name} is not in the project any more.").into());
            cx.notify();
            return;
        };
        let point = language::Point::new(original.line.saturating_sub(1), 0);
        // Opening the file in this tab's pane updates the tab, so it happens after this update.
        cx.spawn_in(window, async move |this, cx| {
            let opened = async {
                let item = workspace
                    .update_in(cx, |workspace, window, cx| {
                        workspace.open_path(path, None, true, window, cx)
                    })?
                    .await?;
                if let Some(editor) = cx.update(|_, cx| item.act_as::<Editor>(cx))? {
                    editor.update_in(cx, |editor, window, cx| {
                        editor.go_to_singleton_buffer_point(point, window, cx);
                    })?;
                }
                anyhow::Ok(())
            }
            .await;
            if let Err(error) = opened {
                this.update(cx, |this, cx| {
                    this.tray_error = Some(format!("Could not open the file: {error}").into());
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn discard_pick(&mut self, id: usize, cx: &mut Context<Self>) {
        self.captions.remove(&id);
        self.hub.update(cx, |hub, cx| hub.discard_pick(id, cx));
    }

    fn dismiss_pick_error(&mut self, cx: &mut Context<Self>) {
        self.tray_error = None;
        if let Some(target) = self.target.clone() {
            self.hub
                .update(cx, |hub, cx| hub.clear_pick_error(&target, cx));
        }
        cx.notify();
    }

    /// Saves the page's last minute as a recording, and says so, or why not, in a toast (#499).
    fn record_this(&mut self, _: &RecordThis, _: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let saved = self
            .hub
            .update(cx, |hub, cx| hub.record(&target, recordings_dir(), cx));
        let workspace = self.workspace.clone();
        cx.spawn(async move |_, cx| {
            let message = match saved.await {
                Ok(id) => format!(
                    "Saved this page's last minute as recording {id}; agents read it with \
                     browser_recording."
                ),
                Err(error) => format!("Could not save the recording: {error:#}"),
            };
            workspace
                .update(cx, |workspace, cx| {
                    workspace.show_toast(
                        Toast::new(NotificationId::unique::<RecordThis>(), message),
                        cx,
                    );
                })
                .ok();
        })
        .detach();
    }

    /// Turns annotate mode on or off; the page takes the focus, so Escape ends it.
    fn annotate(&mut self, _: &Annotate, window: &mut Window, cx: &mut Context<Self>) {
        self.annotate_mode = if matches!(self.annotate_mode, AnnotateMode::Off) {
            window.focus(&self.focus_handle, cx);
            AnnotateMode::On
        } else {
            AnnotateMode::Off
        };
        self.selected_annotation = None;
        cx.notify();
    }

    const fn is_dragging(&self) -> bool {
        matches!(self.annotate_mode, AnnotateMode::Dragging { .. })
    }

    /// A press in annotate mode starts a box at the document point `from`; a box still waiting
    /// for its note goes.
    fn start_drag(&mut self, from: (f64, f64), cx: &mut Context<Self>) {
        if !matches!(self.annotate_mode, AnnotateMode::Off) {
            self.annotate_mode = AnnotateMode::Dragging { from, to: from };
            self.selected_annotation = None;
            cx.notify();
        }
    }

    fn drag_to(&mut self, point: (f64, f64), cx: &mut Context<Self>) {
        if let AnnotateMode::Dragging { to, .. } = &mut self.annotate_mode {
            *to = point;
            cx.notify();
        }
    }

    /// The release ends the box and opens the field for its note; a drag too short to be a box
    /// draws nothing.
    fn end_drag(&mut self, point: (f64, f64), window: &mut Window, cx: &mut Context<Self>) {
        let AnnotateMode::Dragging { from, .. } = self.annotate_mode else {
            return;
        };
        let page_box = PageBox {
            x: from.0.min(point.0),
            y: from.1.min(point.1),
            width: (point.0 - from.0).abs(),
            height: (point.1 - from.1).abs(),
        };
        if page_box.width < SHORTEST_BOX || page_box.height < SHORTEST_BOX {
            self.annotate_mode = AnnotateMode::On;
            cx.notify();
            return;
        }
        let note = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("A note for this box; Enter keeps it", window, cx);
            editor
        });
        window.focus(&note.focus_handle(cx), cx);
        self.annotate_mode = AnnotateMode::Noting { page_box, note };
        cx.notify();
    }

    /// Keeps the box drawn, with its note; annotate mode stays on for the next.
    fn keep_annotation(&mut self, _: &KeepAnnotation, window: &mut Window, cx: &mut Context<Self>) {
        let AnnotateMode::Noting { page_box, note } = &self.annotate_mode else {
            return;
        };
        let (page_box, note) = (*page_box, note.read(cx).text(cx).trim().to_string());
        if let Some(target) = self.target.clone() {
            self.hub.update(cx, |hub, cx| {
                hub.add_annotation(&target, page_box, note, Maker::User, cx);
            });
        }
        self.annotate_mode = AnnotateMode::On;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Drops the box being drawn, or waiting for its note.
    fn drop_annotation(&mut self, _: &DropAnnotation, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            self.annotate_mode,
            AnnotateMode::Dragging { .. } | AnnotateMode::Noting { .. }
        ) {
            self.annotate_mode = AnnotateMode::On;
            window.focus(&self.focus_handle, cx);
            cx.notify();
        }
    }

    /// A click on an annotation's note selects it, for Delete; the page takes the focus.
    fn select_annotation(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_annotation = Some(id);
        window.focus(&self.focus_handle, cx);
        cx.notify();
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
        let Some(target) = self.target.clone() else {
            return;
        };
        let keystroke = &event.keystroke;
        if keystroke.key == "escape" && self.hub.read(cx).is_picking(&target) {
            self.hub
                .update(cx, |hub, cx| hub.set_picking(&target, false, cx));
            cx.stop_propagation();
            return;
        }
        if keystroke.key == "escape" && !matches!(self.annotate_mode, AnnotateMode::Off) {
            self.annotate_mode = AnnotateMode::Off;
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if let Some(id) = self.selected_annotation
            && matches!(keystroke.key.as_str(), "delete" | "backspace")
        {
            self.selected_annotation = None;
            self.hub
                .update(cx, |hub, cx| hub.remove_annotation(&target, id, cx));
            cx.stop_propagation();
            return;
        }
        self.pressed_at = Some(Instant::now());
        let modifiers = &keystroke.modifiers;
        let ctrl_alone = modifiers.control && !modifiers.alt && !modifiers.platform;
        if ctrl_alone && keystroke.key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                self.hub
                    .update(cx, |hub, cx| hub.insert_text(&target, &text, cx));
            }
            cx.stop_propagation();
            return;
        }
        let Some(press) = input::key_press(keystroke, event.is_held) else {
            return;
        };
        if ctrl_alone && !modifiers.shift && matches!(keystroke.key.as_str(), "c" | "x") {
            self.hub
                .update(cx, |hub, cx| hub.copy_selection(&target, press, cx));
            cx.stop_propagation();
            return;
        }
        // A key with text ends a composition: a compose sequence's last key, say.
        if keystroke.key_char.is_some() && !self.marked.is_empty() {
            self.marked.clear();
            self.hub
                .update(cx, |hub, cx| hub.set_composition(&target, "", 0..0, cx));
        }
        self.hub
            .update(cx, |hub, cx| hub.send_key(&target, press, cx));
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
        let Some(target) = self.target.clone() else {
            return;
        };
        if !self.marked.is_empty() {
            let text = std::mem::take(&mut self.marked);
            self.hub
                .update(cx, |hub, cx| hub.insert_text(&target, &text, cx));
        }
    }

    fn replace_text_in_range(
        &mut self,
        _range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.target.clone() else {
            return;
        };
        // Inserting replaces the page's composition, if there is one.
        self.marked.clear();
        if !text.is_empty() {
            self.hub
                .update(cx, |hub, cx| hub.insert_text(&target, text, cx));
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
        let Some(target) = self.target.clone() else {
            return;
        };
        let length = new_text.encode_utf16().count();
        let selected = new_selected_range.unwrap_or(length..length);
        self.marked = new_text.to_string();
        self.hub.update(cx, |hub, cx| {
            hub.set_composition(&target, new_text, selected, cx);
        });
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
        // A tab still waiting for its page asks the hub of no page.
        let target = self.target.as_deref().unwrap_or_default();
        let showing = hub.state == HubState::Showing && hub.page_state(target).is_some();
        let loading = hub.is_loading(target);
        let picking = hub.is_picking(target);
        let annotating = !matches!(self.annotate_mode, AnnotateMode::Off);
        let colors = cx.theme().colors();
        let reload_or_stop = if loading {
            IconButton::new("browser-stop", IconName::Close)
                .tooltip(Tooltip::text("Stop"))
                .on_click(cx.listener(|this, _, _, cx| this.stop(cx)))
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
                    .disabled(!hub.can_go(target, -1))
                    .tooltip(Tooltip::for_action_title("Back", &BrowserBack))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.back(&BrowserBack, window, cx);
                    })),
            )
            .child(
                IconButton::new("browser-forward", IconName::ArrowRight)
                    .disabled(!hub.can_go(target, 1))
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
            .child(Self::render_pick_button(showing, picking, cx))
            .child(Self::render_annotate_button(showing, annotating, cx))
            .child(Self::render_record_button(showing, cx))
            // What an agent does in the page, while it does it and a moment after (#492).
            .when_some(hub.agent_chip(target), |this, action| {
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

    /// The toolbar's pick button, lit while the page is in pick mode.
    fn render_pick_button(showing: bool, picking: bool, cx: &Context<Self>) -> IconButton {
        IconButton::new("browser-pick", IconName::Crosshair)
            .disabled(!showing)
            .toggle_state(picking)
            .selected_icon_color(Color::Accent)
            .tooltip(Tooltip::for_action_title(
                if picking {
                    "Stop Picking"
                } else {
                    "Pick an Element for the Agent"
                },
                &PickElement,
            ))
            .on_click(cx.listener(|this, _, window, cx| {
                this.pick_element(&PickElement, window, cx);
            }))
    }

    /// The toolbar's annotate button, lit while a drag in the page draws a box (#498).
    fn render_annotate_button(showing: bool, annotating: bool, cx: &Context<Self>) -> IconButton {
        IconButton::new("browser-annotate", IconName::Pencil)
            .disabled(!showing)
            .toggle_state(annotating)
            .selected_icon_color(Color::Accent)
            .tooltip(Tooltip::for_action_title(
                if annotating {
                    "Stop Annotating"
                } else {
                    "Annotate the Page"
                },
                &Annotate,
            ))
            .on_click(cx.listener(|this, _, window, cx| {
                this.annotate(&Annotate, window, cx);
            }))
    }

    /// The toolbar's record button, a red dot: the recorder keeps the page's last minute while the
    /// tab draws it, and the button saves it (#499).
    fn render_record_button(showing: bool, cx: &Context<Self>) -> IconButton {
        IconButton::new("browser-record", IconName::Circle)
            .icon_size(IconSize::Small)
            .icon_color(Color::Error)
            .disabled(!showing)
            .tooltip(Tooltip::for_action_title(
                "Save the Last Minute of This Page",
                &RecordThis,
            ))
            .on_click(cx.listener(|this, _, window, cx| {
                this.record_this(&RecordThis, window, cx);
            }))
    }

    /// The page's annotations over the frame drawn now, each placed from that frame's own
    /// scroll and scale; then the box being drawn, and the field for its note (#498).
    fn render_annotations(
        &self,
        placement: Option<Placement>,
        cx: &Context<Self>,
    ) -> Vec<AnyElement> {
        let (Some(placement), Some(target)) = (placement, self.target.as_deref()) else {
            return Vec::new();
        };
        let user = cx.theme().status().warning;
        let mut elements: Vec<AnyElement> = self
            .hub
            .read(cx)
            .annotations(target)
            .iter()
            .filter_map(|annotation| self.render_annotation(annotation, placement, cx))
            .flatten()
            .collect();
        match &self.annotate_mode {
            AnnotateMode::Dragging { from, to } => {
                let page_box = PageBox {
                    x: from.0.min(to.0),
                    y: from.1.min(to.1),
                    width: (to.0 - from.0).abs(),
                    height: (to.1 - from.1).abs(),
                };
                elements.push(
                    annotation_box(placement.place(page_box), user, false, true).into_any_element(),
                );
            }
            AnnotateMode::Noting { page_box, note } => {
                let bounds = placement.place(*page_box);
                elements.push(annotation_box(bounds, user, false, true).into_any_element());
                elements.push(Self::render_note_field(bounds, placement, note, cx));
            }
            AnnotateMode::Off | AnnotateMode::On => {}
        }
        elements
    }

    /// An annotation's box and its note's chip, which a click selects; none when the box is out
    /// of the frame.
    fn render_annotation(
        &self,
        annotation: &Annotation,
        placement: Placement,
        cx: &Context<Self>,
    ) -> Option<[AnyElement; 2]> {
        let bounds = placement.place(annotation.page_box);
        if !placement.shows(&bounds) {
            return None;
        }
        let theme = cx.theme();
        let (color, maker) = match annotation.maker {
            Maker::User => (theme.status().warning, "You"),
            Maker::Agent => (theme.colors().text_accent, "Agent"),
        };
        let selected = self.selected_annotation == Some(annotation.id);
        let id = annotation.id;
        let label = if annotation.note.is_empty() {
            maker.to_string()
        } else {
            annotation.note.clone()
        };
        let chip_top = if bounds.origin.y > px(NOTE_CHIP_HEIGHT) {
            bounds.origin.y - px(NOTE_CHIP_HEIGHT)
        } else {
            bounds.origin.y
        };
        let chip = h_flex()
            .id(("browser-annotation", id))
            .absolute()
            .left(bounds.origin.x)
            .top(chip_top)
            .h(px(NOTE_CHIP_HEIGHT))
            .max_w(rems(20.))
            .gap_1()
            .px_1()
            .rounded_sm()
            .border_1()
            .border_color(color)
            .bg(theme.colors().elevated_surface_background)
            .occlude()
            .cursor_pointer()
            .when(annotation.maker == Maker::Agent, |this| {
                this.child(
                    Icon::new(IconName::Sparkle)
                        .size(IconSize::XSmall)
                        .color(Color::Custom(color)),
                )
            })
            .child(
                Label::new(label)
                    .size(LabelSize::Small)
                    .color(Color::Custom(color))
                    .truncate(),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_annotation(id, window, cx);
            }));
        Some([
            annotation_box(bounds, color, selected, false).into_any_element(),
            chip.into_any_element(),
        ])
    }

    /// The field for a drawn box's note: under the box, or over it when it would not fit.
    fn render_note_field(
        bounds: Bounds<Pixels>,
        placement: Placement,
        note: &Entity<Editor>,
        cx: &Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors();
        let below = bounds.origin.y + bounds.size.height + px(4.);
        let top = if f64::from(below) + f64::from(NOTE_FIELD_HEIGHT) > placement.height {
            bounds.origin.y - px(4. + NOTE_FIELD_HEIGHT)
        } else {
            below
        };
        div()
            .absolute()
            .left(bounds.origin.x)
            .top(top)
            .w(rems(20.))
            .key_context("MarleyAnnotationNote")
            .occlude()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(colors.border)
            .bg(colors.editor_background)
            .child(note.clone())
            .into_any_element()
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

    /// The page's picks under the toolbar, newest first, each with its caption, Send and Discard,
    /// and what went wrong with the last pick or Send (#496).
    fn render_tray(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let target = self.target.clone()?;
        let (picks, error) = {
            let hub = self.hub.read(cx);
            let picks: Vec<TrayRow> = hub
                .picks()
                .iter()
                .rev()
                .filter(|pick| pick.tab == target && !pick.dismissed)
                .map(|pick| TrayRow {
                    id: pick.id,
                    summary: pick.summary.clone(),
                    sent: pick.sent.then(|| pick.caption.clone()),
                    place: listener_place(pick),
                })
                .collect();
            (picks, hub.pick_error(&target))
        };
        self.captions
            .retain(|id, _| picks.iter().any(|row| row.id == *id && row.sent.is_none()));
        let error = error.or_else(|| self.tray_error.clone());
        if picks.is_empty() && error.is_none() {
            return None;
        }
        let colors = cx.theme().colors();
        let (border_variant, background) = (colors.border_variant, colors.panel_background);
        let error = error.map(|error| Self::render_pick_error(error, cx));
        let rows: Vec<_> = picks
            .into_iter()
            .map(|row| self.render_pick_row(row, window, cx))
            .collect();
        Some(
            v_flex()
                .id("browser-picks")
                .flex_none()
                .w_full()
                .max_h(rems(9.))
                .overflow_y_scroll()
                .border_b_1()
                .border_color(border_variant)
                .bg(background)
                .children(error)
                .children(rows)
                .into_any_element(),
        )
    }

    fn render_pick_error(error: SharedString, cx: &Context<Self>) -> impl IntoElement + use<> {
        h_flex()
            .w_full()
            .gap_1()
            .px_2()
            .py_1()
            .child(
                Icon::new(IconName::Warning)
                    .size(IconSize::Small)
                    .color(Color::Warning),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(error).size(LabelSize::Small).truncate()),
            )
            .child(
                IconButton::new("browser-pick-error-dismiss", IconName::Close)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Dismiss"))
                    .on_click(cx.listener(|this, _, _, cx| this.dismiss_pick_error(cx))),
            )
    }

    /// A pick's row: its number and summary, then its caption field and Send, or what was sent.
    fn render_pick_row(
        &mut self,
        row: TrayRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let TrayRow {
            id,
            summary,
            sent,
            place,
        } = row;
        let sent_already = sent.is_some();
        // One height for every row, sent or not, so the page below moves by whole rows.
        let head = h_flex()
            .w_full()
            .h_9()
            .gap_2()
            .px_2()
            .child(
                Label::new(format!("Pick {id}"))
                    .size(LabelSize::Small)
                    .color(Color::Accent),
            )
            .child(
                div()
                    .flex_none()
                    .max_w(rems(20.))
                    .child(Label::new(summary).size(LabelSize::Small).truncate()),
            )
            .when_some(place, |row, place| {
                row.child(Self::render_listener_place(id, place, cx))
            });
        let body = if let Some(caption) = sent {
            let said = if caption.is_empty() {
                "Sent to the terminal".to_string()
            } else {
                format!("Sent: {caption}")
            };
            head.child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_1()
                    .child(
                        Icon::new(IconName::Check)
                            .size(IconSize::Small)
                            .color(Color::Success),
                    )
                    .child(
                        Label::new(said)
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .truncate(),
                    ),
            )
        } else {
            let field = self.caption(id, window, cx);
            let colors = cx.theme().colors();
            head.child(
                div()
                    .key_context("MarleyPickCaption")
                    .on_action(cx.listener(move |this, _: &SendPick, window, cx| {
                        this.send_pick(id, window, cx);
                    }))
                    .flex_1()
                    .min_w_0()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(field),
            )
            .child(
                Button::new(("browser-pick-send", id), "Send")
                    .style(ButtonStyle::Filled)
                    .tooltip(Tooltip::text("Type it into the terminal you used last"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.send_pick(id, window, cx);
                    })),
            )
        };
        body.child(
            IconButton::new(("browser-pick-discard", id), IconName::Close)
                .icon_size(IconSize::Small)
                .tooltip(Tooltip::text(if sent_already {
                    "Remove from the tray; the agent can still read it"
                } else {
                    "Discard"
                }))
                .on_click(cx.listener(move |this, _, _, cx| this.discard_pick(id, cx))),
        )
    }

    /// A pick's listener and where it is: a link that opens its file when the file is in the
    /// project, else the place muted; every listener in the tooltip.
    fn render_listener_place(
        id: usize,
        place: ListenerPlace,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let ListenerPlace {
            event,
            label,
            opens,
            all,
        } = place;
        let location = match opens {
            Some(listener) => Button::new(("browser-pick-source", id), label)
                .style(ButtonStyle::Transparent)
                .label_size(LabelSize::Small)
                .color(Color::Accent)
                .truncate(true)
                .tooltip(Tooltip::text(all))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.open_pick_source(id, listener, window, cx);
                }))
                .into_any_element(),
            None => div()
                .id(("browser-pick-place", id))
                .min_w_0()
                .tooltip(Tooltip::text(all))
                .child(
                    Label::new(label)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .truncate(),
                )
                .into_any_element(),
        };
        h_flex()
            .flex_none()
            .max_w(rems(18.))
            .gap_1()
            .child(Label::new(event).size(LabelSize::Small).color(Color::Muted))
            .child(location)
    }

    /// What the tab says over the page while it has none to show, or when the browser stopped.
    fn message(
        &self,
        state: HubState,
        attached: bool,
    ) -> Option<(SharedString, Option<SharedString>)> {
        match state {
            HubState::Failed(reason) => Some((
                reason,
                Some(SharedString::new_static(
                    "Run \u{201c}marley: open browser\u{201d} to try again.",
                )),
            )),
            _ if attached => None,
            HubState::Starting => Some((SharedString::new_static("Starting Chromium…"), None)),
            HubState::Connecting => {
                Some((SharedString::new_static("Connecting to Chromium…"), None))
            }
            HubState::Showing => Some((
                self.open_error
                    .clone()
                    .unwrap_or_else(|| SharedString::new_static("Opening a page…")),
                None,
            )),
        }
    }
}

impl Render for BrowserView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (state, frame, metadata, dialog, attached) = {
            let hub = self.hub.read(cx);
            let page = self
                .target
                .as_deref()
                .and_then(|target| hub.page_state(target));
            (
                hub.state.clone(),
                page.and_then(|page| page.frame.clone()),
                page.and_then(|page| page.metadata),
                page.and_then(|page| page.dialog.clone()),
                page.is_some(),
            )
        };
        if let Some(frame) = &frame {
            self.drew(frame, window);
        }
        let message = self.message(state, attached);
        let placement = frame
            .as_deref()
            .zip(metadata)
            .and_then(|(frame, metadata)| Placement::new(frame, metadata, window));
        let annotations = self.render_annotations(placement, cx);
        let annotating = !matches!(self.annotate_mode, AnnotateMode::Off);
        let tray = self.render_tray(window, cx);
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
            .on_action(cx.listener(Self::pick_element))
            .on_action(cx.listener(Self::annotate))
            .on_action(cx.listener(Self::record_this))
            .on_action(cx.listener(Self::keep_annotation))
            .on_action(cx.listener(Self::drop_annotation))
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(toolbar)
            .children(tray)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_hidden()
                    .child(PageElement {
                        hub: self.hub.clone(),
                        view: cx.entity(),
                        target: self.target.clone().filter(|_| attached),
                        focus_handle: self.focus_handle.clone(),
                        frame,
                        metadata,
                        annotating,
                    })
                    .children(annotations)
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
            // Over everything, as Chromium's own popup would be.
            .children(self.select_menu.as_ref().map(|select| {
                deferred(
                    anchored()
                        .position(select.position)
                        .anchor(Anchor::TopLeft)
                        .snap_to_window_with_margin(px(8.))
                        .child(select.menu.clone()),
                )
                .with_priority(1)
            }))
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
        self.target
            .as_deref()
            .and_then(|target| self.hub.read(cx).title(target))
            .or_else(|| self.saved_title.clone())
            .unwrap_or_else(|| SharedString::new_static("Browser"))
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::ToolWeb))
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<SharedString> {
        self.target
            .as_deref()
            .and_then(|target| self.hub.read(cx).url(target))
            .or_else(|| self.saved_url.clone().map(SharedString::from))
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        None
    }

    fn deactivated(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.stop_viewing(cx);
    }

    fn on_removed(&self, cx: &mut Context<Self>) {
        let Some(target) = self.target.clone() else {
            return;
        };
        let hub = self.hub.clone();
        // A move to another pane removes the tab and adds it again before the effects run: the
        // page closes only when no pane holds a tab of it once they have.
        cx.defer(move |cx| {
            let views = live_views(cx);
            let shown = views.iter().any(|view| {
                view.read(cx).target.as_deref() == Some(target.as_str()) && in_a_pane(view, cx)
            });
            if !shown {
                hub.update(cx, |hub, cx| hub.close_page(&target, cx));
            }
        });
    }
}

/// A Browser tab is saved with its workspace (#494). The workspace's layout holds only the item;
/// the tab's page id, URL and title go in the tab's own table.
impl SerializableItem for BrowserView {
    fn serialized_item_kind() -> &'static str {
        "MarleyBrowserTab"
    }

    fn cleanup(
        workspace_id: WorkspaceId,
        alive_items: Vec<ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        workspace::delete_unloaded_items(
            alive_items,
            workspace_id,
            "marley_browser_tabs",
            &persistence::MarleyBrowserTabsDb::global(cx),
            cx,
        )
    }

    fn deserialize(
        _project: Entity<Project>,
        workspace: WeakEntity<Workspace>,
        workspace_id: WorkspaceId,
        item_id: ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        let db = persistence::MarleyBrowserTabsDb::global(cx);
        window.spawn(cx, async move |cx| {
            let (target, url, title) = db
                .get_tab(item_id, workspace_id)?
                .context("no Browser tab was saved for the item")?;
            cx.update(|window, cx| {
                let hub = BrowserHub::global(cx);
                // The tab claims its page before the browser's start can report it, so the page
                // gets no second tab.
                let view = cx.new(|cx| Self::new(hub, Some(target), false, workspace, window, cx));
                view.update(cx, |view, cx| view.restore(url, title, window, cx));
                view
            })
        })
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: ItemId,
        _closing: bool,
        cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        let workspace_id = workspace.database_id()?;
        let (target, url, title) = self.saved(cx)?;
        let db = persistence::MarleyBrowserTabsDb::global(cx);
        Some(cx.background_spawn(async move {
            db.save_tab(item_id, workspace_id, target, url, title).await
        }))
    }

    fn should_serialize(&self, event: &ItemEvent) -> bool {
        matches!(event, ItemEvent::UpdateTab)
    }
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;
    use workspace::{ItemId, WorkspaceDb, WorkspaceId};

    /// The Browser tabs' own table: each tab's page, and the URL and title it had.
    pub(super) struct MarleyBrowserTabsDb(ThreadSafeConnection);

    impl Domain for MarleyBrowserTabsDb {
        const NAME: &str = stringify!(MarleyBrowserTabsDb);

        const MIGRATIONS: &[&str] = &[
            sql!(
                CREATE TABLE marley_browser_tabs (
                    workspace_id INTEGER,
                    item_id INTEGER UNIQUE,
                    target_id TEXT NOT NULL,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL,

                    PRIMARY KEY(workspace_id, item_id),
                    FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                    ON DELETE CASCADE
                ) STRICT;
            ),
            // An item id is its entity id, which repeats across launches, so it keys a row only
            // with its workspace: with `UNIQUE(item_id)`, one workspace's tab replaced another's
            // (#576). SQLite drops a constraint by building the table again.
            sql!(
                CREATE TABLE marley_browser_tabs2 (
                    workspace_id INTEGER,
                    item_id INTEGER,
                    target_id TEXT NOT NULL,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL,

                    PRIMARY KEY(workspace_id, item_id),
                    FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                    ON DELETE CASCADE
                ) STRICT;

                INSERT INTO marley_browser_tabs2 (workspace_id, item_id, target_id, url, title)
                SELECT workspace_id, item_id, target_id, url, title FROM marley_browser_tabs;

                DROP TABLE marley_browser_tabs;

                ALTER TABLE marley_browser_tabs2 RENAME TO marley_browser_tabs;
            ),
        ];
    }

    db::static_connection!(MarleyBrowserTabsDb, [WorkspaceDb]);

    impl MarleyBrowserTabsDb {
        query! {
            pub(super) async fn save_tab(
                item_id: ItemId,
                workspace_id: WorkspaceId,
                target_id: String,
                url: String,
                title: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_browser_tabs(item_id, workspace_id, target_id, url, title)
                VALUES (?, ?, ?, ?, ?)
            }
        }

        query! {
            pub(super) fn get_tab(
                item_id: ItemId,
                workspace_id: WorkspaceId
            ) -> Result<Option<(String, String, String)>> {
                SELECT target_id, url, title
                FROM marley_browser_tabs
                WHERE item_id = ? AND workspace_id = ?
            }
        }
    }
}

/// The page's frame, drawn from the tab's top left at its own size (a frame from before a
/// resize is neither stretched nor squeezed), the tab's size reported to the hub, and the mouse
/// and the tab's input handler, which send what they get to the page.
struct PageElement {
    hub: Entity<BrowserHub>,
    view: Entity<BrowserView>,
    /// The page, once the hub has it.
    target: Option<String>,
    focus_handle: FocusHandle,
    frame: Option<Arc<RenderImage>>,
    metadata: Option<FrameMetadata>,
    /// Whether a drag draws an annotation rather than reaching the page (#498).
    annotating: bool,
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
        if let Some(target) = &self.target {
            let scale = window.scale_factor();
            self.hub
                .update(cx, |hub, cx| hub.resize(target, bounds.size, scale, cx));
            self.view.update(cx, BrowserView::start_viewing);
        }
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
        if let Some(target) = self.target.clone() {
            let to_page = PageMapping::new(bounds, self.frame.as_deref(), self.metadata, window);
            self.view.update(cx, |view, _| view.mapping = Some(to_page));
            if self.annotating {
                self.listen_for_a_box(hitbox, to_page, window);
            } else {
                self.listen(&target, hitbox, to_page, window);
            }
            self.listen_for_the_wheel(target, hitbox, to_page, window);
        }
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

    /// Where a point of the page, in CSS pixels, falls in the window.
    fn to_window(self, (x, y): (f64, f64)) -> Point<Pixels> {
        point(
            self.origin.x + window_pixels(x / self.scale_x),
            self.origin.y + window_pixels(y / self.scale_y),
        )
    }
}

impl PageElement {
    /// Sends the mouse to the page `target`: a press in the page focuses the tab first, and
    /// while a button the page got is held, moves and the release reach it wherever the pointer
    /// is.
    fn listen(&self, target: &str, hitbox: &Hitbox, to_page: PageMapping, window: &mut Window) {
        window.on_mouse_event({
            let hub = self.hub.clone();
            let view = self.view.clone();
            let focus_handle = self.focus_handle.clone();
            let hitbox = hitbox.clone();
            let target = target.to_string();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                window.focus(&focus_handle, cx);
                view.update(cx, |view, cx| {
                    view.last_press = Some(event.position);
                    view.pressed_at = Some(Instant::now());
                    if view.selected_annotation.take().is_some() {
                        cx.notify();
                    }
                });
                hub.update(cx, |hub, cx| {
                    hub.mouse_press(
                        &target,
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
            let target = target.to_string();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || !(hitbox.is_hovered(window) || hub.read(cx).is_holding(&target))
                {
                    return;
                }
                hub.update(cx, |hub, cx| {
                    hub.mouse_release(
                        &target,
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
            let target = target.to_string();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || !(hitbox.is_hovered(window) || hub.read(cx).is_holding(&target))
                {
                    return;
                }
                hub.update(cx, |hub, cx| {
                    hub.mouse_move(&target, to_page.map(event.position), event.modifiers, cx);
                });
            }
        });
    }

    /// Sends the wheel to the page, in annotate mode too.
    fn listen_for_the_wheel(
        &self,
        target: String,
        hitbox: &Hitbox,
        to_page: PageMapping,
        window: &mut Window,
    ) {
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
                    hub.wheel(
                        &target,
                        to_page.map(event.position),
                        delta,
                        event.modifiers,
                        cx,
                    );
                });
                cx.stop_propagation();
            }
        });
    }

    /// In annotate mode, a drag in the page draws a box, its corners in document points: the
    /// viewport point over the pinch scale, plus the scroll of the frame drawn now (#498).
    fn listen_for_a_box(&self, hitbox: &Hitbox, to_page: PageMapping, window: &mut Window) {
        let (zoom, scroll_x, scroll_y, top) =
            self.metadata.map_or((1.0, 0.0, 0.0, 0.0), |metadata| {
                (
                    positive_or_one(metadata.page_scale_factor),
                    metadata.scroll_offset_x,
                    metadata.scroll_offset_y,
                    metadata.offset_top,
                )
            });
        let to_document = move |position: Point<Pixels>| {
            let (x, y) = to_page.map(position);
            (x / zoom + scroll_x, (y - top) / zoom + scroll_y)
        };
        window.on_mouse_event({
            let view = self.view.clone();
            let focus_handle = self.focus_handle.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Left
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                window.focus(&focus_handle, cx);
                view.update(cx, |view, cx| {
                    view.start_drag(to_document(event.position), cx);
                });
            }
        });
        window.on_mouse_event({
            let view = self.view.clone();
            move |event: &MouseMoveEvent, phase, _, cx| {
                if phase == DispatchPhase::Bubble && view.read(cx).is_dragging() {
                    view.update(cx, |view, cx| {
                        view.drag_to(to_document(event.position), cx);
                    });
                }
            }
        });
        window.on_mouse_event({
            let view = self.view.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble
                    && event.button == MouseButton::Left
                    && view.read(cx).is_dragging()
                {
                    view.update(cx, |view, cx| {
                        view.end_drag(to_document(event.position), window, cx);
                    });
                }
            }
        });
    }
}

/// How long a side of a drawn box must be, in CSS pixels, for the drag to draw one.
const SHORTEST_BOX: f64 = 4.0;

/// The height of an annotation's note chip, in window pixels.
const NOTE_CHIP_HEIGHT: f32 = 20.0;

/// The height of the note field, in window pixels, which goes above its box when it would not
/// fit below.
const NOTE_FIELD_HEIGHT: f32 = 32.0;

/// A pinch scale as a divisor: one when the frame reports none.
fn positive_or_one(value: f64) -> f64 {
    if value > 0.0 { value } else { 1.0 }
}

/// An annotation's box: a border in its maker's color over a light fill, thicker when selected,
/// dashed while it is drawn.
fn annotation_box(bounds: Bounds<Pixels>, color: gpui::Hsla, selected: bool, drawing: bool) -> Div {
    div()
        .absolute()
        .left(bounds.origin.x)
        .top(bounds.origin.y)
        .w(bounds.size.width)
        .h(bounds.size.height)
        .rounded_sm()
        .map(|this| {
            if selected {
                this.border_4()
            } else {
                this.border_2()
            }
        })
        .when(drawing, Styled::border_dashed)
        .border_color(color)
        .bg(color.opacity(0.08))
}

/// Where the document's points fall in the page area, from the frame drawn now (#498): its
/// scroll, its pinch scale and its top offset, and its viewport in DIP over the size it is drawn
/// at.
#[derive(Clone, Copy)]
struct Placement {
    scroll_x: f64,
    scroll_y: f64,
    zoom: f64,
    top: f64,
    scale_x: f64,
    scale_y: f64,
    /// The frame's size as drawn, in window pixels.
    width: f64,
    height: f64,
}

impl Placement {
    fn new(frame: &RenderImage, metadata: FrameMetadata, window: &Window) -> Option<Self> {
        let drawn = frame.size(0).to_pixels(window.scale_factor());
        let (width, height) = (f64::from(drawn.width), f64::from(drawn.height));
        (width > 0.0 && height > 0.0).then(|| Self {
            scroll_x: metadata.scroll_offset_x,
            scroll_y: metadata.scroll_offset_y,
            zoom: positive_or_one(metadata.page_scale_factor),
            top: metadata.offset_top,
            scale_x: metadata.device_width / width,
            scale_y: metadata.device_height / height,
            width,
            height,
        })
    }

    /// A box of the document as a box of the page area, in window pixels from its top left.
    fn place(&self, page_box: PageBox) -> Bounds<Pixels> {
        let x = (page_box.x - self.scroll_x) * self.zoom / self.scale_x;
        let y = (page_box.y - self.scroll_y).mul_add(self.zoom, self.top) / self.scale_y;
        Bounds::new(
            point(window_pixels(x), window_pixels(y)),
            gpui::size(
                window_pixels(page_box.width * self.zoom / self.scale_x),
                window_pixels(page_box.height * self.zoom / self.scale_y),
            ),
        )
    }

    /// Whether any of `bounds` shows in the frame.
    fn shows(&self, bounds: &Bounds<Pixels>) -> bool {
        let right = f64::from(bounds.origin.x + bounds.size.width);
        let bottom = f64::from(bounds.origin.y + bounds.size.height);
        right > 0.0
            && bottom > 0.0
            && f64::from(bounds.origin.x) < self.width
            && f64::from(bounds.origin.y) < self.height
    }
}

/// Installs `marley::OpenBrowser` and `marley::NewBrowserTab` on every workspace, and saves and
/// restores the Browser tabs with their workspaces. [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    workspace::register_serializable_item::<BrowserView>(cx);
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenBrowser, window, cx| {
            open(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &NewBrowserTab, window, cx| {
            new_tab(workspace, window, cx);
        });
    })
    .detach();
    track_terminals(cx);
}

/// The terminal the user focused last, where Send types a pick (#496).
struct LastTerminal {
    view: WeakEntity<TerminalView>,
    window: AnyWindowHandle,
}

impl Global for LastTerminal {}

/// Keeps [`LastTerminal`]: each terminal view, center or docked, marks itself when the focus
/// enters it.
fn track_terminals(cx: &App) {
    cx.observe_new(
        |view: &mut TerminalView, window, cx: &mut Context<TerminalView>| {
            let Some(window) = window else {
                return;
            };
            let focus_handle = view.focus_handle(cx);
            cx.on_focus_in(&focus_handle, window, |_, window, cx| {
                cx.set_global(LastTerminal {
                    view: cx.weak_entity(),
                    window: window.window_handle(),
                });
            })
            .detach();
        },
    )
    .detach();
}

/// Brings `terminal`'s tab to the front wherever its window keeps it: a center pane or the
/// Terminal Panel, of the workspace shown or another.
pub(crate) fn reveal_terminal(terminal: &Entity<TerminalView>, window: &mut Window, cx: &mut App) {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return;
    };
    let workspaces: Vec<Entity<Workspace>> =
        multi_workspace.read(cx).workspaces().cloned().collect();
    for workspace in workspaces {
        let holds = |pane: &&Entity<Pane>| pane.read(cx).index_for_item(terminal).is_some();
        let center = workspace.read(cx).panes().iter().find(holds).cloned();
        let docked = workspace
            .read(cx)
            .panel::<TerminalPanel>(cx)
            .and_then(|panel| panel.read(cx).panes().into_iter().find(holds).cloned());
        let Some(pane) = center.clone().or(docked) else {
            continue;
        };
        if multi_workspace.read(cx).workspace() != &workspace {
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace.clone(), None, window, cx);
            });
        }
        if center.is_none() {
            workspace.update(cx, |workspace, cx| {
                workspace.open_panel::<TerminalPanel>(window, cx);
            });
        }
        pane.update(cx, |pane, cx| {
            if let Some(index) = pane.index_for_item(terminal) {
                pane.activate_item(index, true, true, window, cx);
            }
        });
        return;
    }
}

/// Each listener's place in its original source, through its script's source map (#497): each
/// map loaded once for the pick, and parsed and scanned off the main thread.
async fn original_positions(
    session: &Page,
    listeners: &[Listener],
    cx: &AsyncApp,
) -> Vec<Option<OriginalPosition>> {
    let mut maps: Vec<(&str, &str, Option<Arc<SourceMap>>)> = Vec::new();
    for listener in listeners {
        let (Some(script), Some(map_url)) = (&listener.script, &listener.source_map) else {
            continue;
        };
        if maps
            .iter()
            .any(|(known_script, known_map, _)| known_script == script && known_map == map_url)
        {
            continue;
        }
        let map = load_map(session, script, map_url, cx).await;
        maps.push((script, map_url, map));
    }
    let mut positions = Vec::with_capacity(listeners.len());
    for listener in listeners {
        let map = maps
            .iter()
            .find(|(script, map_url, _)| {
                listener.script.as_deref() == Some(*script)
                    && listener.source_map.as_deref() == Some(*map_url)
            })
            .and_then(|(_, _, map)| map.clone());
        let position = match map {
            Some(map) => {
                let (line, column) = (
                    listener.line.saturating_sub(1),
                    listener.column.saturating_sub(1),
                );
                cx.background_spawn(futures::future::lazy(move |_| map.original(line, column)))
                    .await
            }
            None => None,
        };
        positions.push(position);
    }
    positions
}

/// The source map `script` names, loaded through the page or read from its `data:` URL.
async fn load_map(
    session: &Page,
    script: &str,
    map_url: &str,
    cx: &AsyncApp,
) -> Option<Arc<SourceMap>> {
    let (text, base) = match source_map::map_location(script, map_url)? {
        MapLocation::Inline(text) => (text, script.to_string()),
        MapLocation::Remote(url) => match session.load_resource(&url).await {
            Ok(text) => (text, url),
            Err(error) => {
                log::info!("browser: {script}'s source map did not load: {error}");
                return None;
            }
        },
    };
    cx.background_spawn(futures::future::lazy(move |_| {
        SourceMap::parse(&text, &base)
    }))
    .await
    .inspect_err(|error| log::info!("browser: {script}'s source map: {error}"))
    .ok()
    .map(Arc::new)
}

/// The file in `project` that a map's `source` names (#497): an absolute path inside a worktree,
/// else the longest suffix of its path, down to two components, that is a file in one; a lone
/// name only when that is all the source names.
fn find_source(project: &Project, source: &str, cx: &App) -> Option<ProjectPath> {
    let path = source_map::source_path(source);
    let worktrees: Vec<_> = project.visible_worktrees(cx).collect();
    if path.absolute {
        let absolute = PathBuf::from(format!("/{}", path.components.join("/")));
        for worktree in &worktrees {
            let worktree = worktree.read(cx);
            let Ok(relative) = absolute.strip_prefix(worktree.abs_path()) else {
                continue;
            };
            let Ok(relative) = RelPath::new(relative, PathStyle::local()) else {
                continue;
            };
            if worktree
                .entry_for_path(&relative)
                .is_some_and(project::Entry::is_file)
            {
                return Some(ProjectPath {
                    worktree_id: worktree.id(),
                    path: relative.into_arc(),
                });
            }
        }
    }
    let count = path.components.len();
    let shortest = if count == 1 { 1 } else { 2 };
    for length in (shortest..=count).rev() {
        let suffix = path.components.get(count - length..)?.join("/");
        let Ok(relative) = RelPath::from_unix_str(&suffix) else {
            continue;
        };
        for worktree in &worktrees {
            let worktree = worktree.read(cx);
            if worktree
                .entry_for_path(relative)
                .is_some_and(project::Entry::is_file)
            {
                return Some(ProjectPath {
                    worktree_id: worktree.id(),
                    path: relative.into_arc(),
                });
            }
        }
    }
    None
}

/// What a pick's tray row says of its listeners: the first whose file is in the project, else
/// the first with a script, and every listener for the tooltip.
fn listener_place(pick: &Pick) -> Option<ListenerPlace> {
    let listeners = &pick.bundle.listeners;
    let opens = listeners.iter().position(|listener| {
        listener
            .original
            .as_ref()
            .is_some_and(|original| original.file.is_some())
    });
    let index = opens.or_else(|| {
        listeners
            .iter()
            .position(|listener| listener.script.is_some())
    })?;
    let listener = listeners.get(index)?;
    let reading = !pick.sources_read && listener.source_map.is_some();
    let label = if reading {
        "\u{2026}".to_string()
    } else {
        listener_where(listener)
    };
    let all = listeners
        .iter()
        .map(|listener| {
            format!(
                "{} on {}: {}",
                listener.event,
                listener.on,
                listener_where(listener)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(ListenerPlace {
        event: listener.event.clone(),
        label,
        opens,
        all,
    })
}

/// Where a listener is: its file and line in the project, else its original source's name and
/// line, else its script's name and line.
fn listener_where(listener: &Listener) -> String {
    match (&listener.original, &listener.script) {
        (
            Some(SourcePosition {
                file: Some(file),
                line,
                ..
            }),
            _,
        ) => format!("{file}:{line}"),
        (Some(original), _) => format!("{}:{}", file_name(&original.source), original.line),
        (None, Some(script)) => format!("{}:{}", file_name(script), listener.line),
        (None, None) => "a script with no URL".to_string(),
    }
}

/// A URL's file name: its last path segment, without the query or the fragment.
fn file_name(url: &str) -> &str {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
}

/// The line Send types into the terminal: the pick's reference, which `browser_pick` reads, then
/// the user's caption.
fn pick_line(id: usize, summary: &str, url: &str, caption: &str) -> String {
    let reference = format!(
        "[browser pick {id}: {summary} on {}; browser_pick id {id}]",
        short_address(url)
    );
    if caption.is_empty() {
        reference
    } else {
        format!("{reference} {caption}")
    }
}

/// A URL as a pick's line names it: its host and path, with no scheme, query or fragment.
fn short_address(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let end = rest.find(['?', '#']).unwrap_or(rest.len());
    let address = rest.get(..end).unwrap_or(rest);
    address.strip_suffix('/').unwrap_or(address)
}

/// Shows the page `target` for an agent that acts in it (#492): its tab comes to the front of its
/// pane, unless that would take the focus from where the user is, and a page with no tab gets
/// one.
pub(crate) fn show_for_agent(target: &str, cx: &mut App) {
    if let Some(view) = view_of(target, cx) {
        reveal(&view, cx);
        return;
    }
    let hub = BrowserHub::global(cx);
    place_tab(&hub, target, None, false, cx);
}

/// Shows the Browser: tabs for the pages that have none, the first with the focus; else the
/// workspace's tab of the page the user focused last, or its first; else a new tab. A hub that
/// stopped starts again, and the tabs of its old pages close.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let hub = BrowserHub::global(cx);
    hub.update(cx, BrowserHub::start_if_failed);
    let this = cx.weak_entity();
    let untabbed = untabbed(&hub, cx);
    if !untabbed.is_empty() {
        let mut first = None;
        for target in untabbed {
            let view = new_view(hub.clone(), Some(target), false, this.clone(), window, cx);
            first.get_or_insert_with(|| view.clone());
            workspace.add_item_to_active_pane(Box::new(view), None, false, window, cx);
        }
        if let Some(first) = first {
            workspace.activate_item(&first, true, true, window, cx);
        }
        return;
    }
    let focused = hub.read(cx).focused();
    let views: Vec<Entity<BrowserView>> = workspace
        .items_of_type::<BrowserView>(cx)
        .filter(|view| {
            // A tab waiting for its page counts, a restored one too; one whose page is gone is
            // closing.
            let view = view.read(cx);
            view.restoring
                || view
                    .target
                    .as_ref()
                    .is_none_or(|target| hub.read(cx).page_state(target).is_some())
        })
        .collect();
    let shown = views
        .iter()
        .find(|view| focused.is_some() && view.read(cx).target == focused)
        .or_else(|| views.first())
        .cloned();
    if let Some(view) = shown {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let showing = hub.read(cx).state == HubState::Showing;
    let view = new_view(hub.clone(), None, !showing, this, window, cx);
    workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
    if showing {
        let window_handle = view.read(cx).window;
        open_page_in(&hub, view.downgrade(), window_handle, BLANK.to_string(), cx);
    }
}

/// Opens a blank page in a new tab after the active one, with the focus in its address bar: what
/// Ctrl+T does, and the rail's New Browser Tab (#500).
pub(crate) fn new_tab(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let hub = BrowserHub::global(cx);
    hub.update(cx, BrowserHub::start_if_failed);
    let view = new_view(hub.clone(), None, false, cx.weak_entity(), window, cx);
    workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
    view.update(cx, |view, cx| {
        view.focus_address_bar(&FocusAddressBar, window, cx);
    });
    let window_handle = view.read(cx).window;
    open_page_in(&hub, view.downgrade(), window_handle, BLANK.to_string(), cx);
}

/// Opens `url` in a Browser tab of `workspace` with the focus, or brings forward the tab of
/// `workspace` already on it (#503).
pub(crate) fn open_url_tab(
    workspace: &mut Workspace,
    url: String,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let this = cx.weak_entity();
    let hub = BrowserHub::global(cx);
    let same = |address: &str| {
        url::Url::parse(address)
            .ok()
            .zip(url::Url::parse(&url).ok())
            .is_some_and(|(address, url)| address == url)
    };
    let showing = live_views(cx).into_iter().find(|view| {
        let view = view.read(cx);
        view.workspace == this
            && view
                .target
                .as_deref()
                .and_then(|target| hub.read(cx).address(target))
                .is_some_and(|address| same(&address))
    });
    if let Some(view) = showing {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    hub.update(cx, BrowserHub::start_if_failed);
    let view = new_view(hub.clone(), None, false, this, window, cx);
    workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
    let window_handle = view.read(cx).window;
    open_page_in(&hub, view.downgrade(), window_handle, url, cx);
}

/// The pages no tab shows: those a start found that no tab claimed, and those whose tab closed
/// with its window.
fn untabbed(hub: &Entity<BrowserHub>, cx: &mut App) -> Vec<String> {
    let shown: Vec<String> = live_views(cx)
        .iter()
        .filter_map(|view| view.read(cx).target.clone())
        .collect();
    hub.read(cx)
        .pages
        .iter()
        .map(|page| page.target().to_string())
        .filter(|target| !shown.contains(target))
        .collect()
}
