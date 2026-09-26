//! Where a URL clicked in a terminal opens, and the dev server URL a terminal offers (#503).
//!
//! Zed's terminal finds the URL under a Ctrl+click, or an OSC 8 link's target under a plain
//! click, and asks [`MarleyTerminalUrl`] where it goes. A local URL (`localhost`, a loopback
//! address) opens in a Browser tab of the terminal's project, where agents see the page too, and
//! any other in the system browser, as `marley.terminal_links` says; Shift+Ctrl+click takes the
//! other place. A terminal whose project is remote, or whose foreground program is an SSH client,
//! sends every URL to the system browser: its `localhost` is another machine.
//!
//! While a local terminal prints, its last lines are read for local URLs, and the newest one
//! whose port something listens on shows in the terminal's footer ([`offer`], [`offer_button`]).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Anchor, AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, EntityId, Global,
    Task, WeakEntity, Window,
};
use marley_browser::address::{self, LocalUrl};
use project::Project;
use settings::{MarleyTerminalLinks, Settings as _};
use terminal::Terminal;
use terminal_view::{MarleyFooterContext, MarleyTerminalUrl, TerminalView};
use ui::{
    ButtonLike, ContextMenu, Icon, IconName, IconSize, Label, LabelSize, PopoverMenu, SplitButton,
    SplitButtonStyle, Tooltip, prelude::*,
};
use util::ResultExt as _;
use workspace::Workspace;

use crate::{MarleySettings, browser};

/// The programs whose terminal reaches another machine.
const SSH_CLIENTS: &[&str] = &["ssh", "mosh-client", "autossh", "et"];
/// How many of a terminal's last lines a scan reads.
const SCANNED_LINES: usize = 200;
/// How long a terminal's output is left before its lines are read: at most one scan in that
/// time, and one after the output stops.
const SCAN_DELAY: Duration = Duration::from_millis(500);
/// How often the listening ports are read while any terminal holds a printed URL.
const PORTS_POLL: Duration = Duration::from_secs(2);
/// How many printed URLs a terminal keeps, the newest.
const KEPT_URLS: usize = 16;
/// Where the machine's TCP tables are.
const PROC_NET: &str = "/proc/net";

/// Each terminal view's printed local URLs, and the ports that listen.
#[derive(Default)]
struct ServedUrls {
    terminals: HashMap<EntityId, Printed>,
    listening: BTreeSet<u16>,
    /// Whether the ports are being read.
    watching: bool,
}

impl Global for ServedUrls {}

/// One terminal's printed local URLs, each port's newest print, the newest last.
struct Printed {
    view: WeakEntity<TerminalView>,
    urls: Vec<LocalUrl>,
    scan: Option<Task<()>>,
}

impl Printed {
    fn remember(&mut self, url: LocalUrl) {
        self.urls.retain(|kept| kept.port != url.port);
        self.urls.push(url);
        let excess = self.urls.len().saturating_sub(KEPT_URLS);
        self.urls.drain(..excess);
    }

    fn offer(&self, listening: &BTreeSet<u16>) -> Option<&LocalUrl> {
        self.urls
            .iter()
            .rev()
            .find(|url| listening.contains(&url.port))
    }
}

/// Where a clicked URL goes.
enum Destination {
    /// A Browser tab of the terminal's workspace, on this URL.
    BrowserTab(String),
    /// The system browser, on this URL.
    SystemBrowser(String),
}

/// Routes the URLs clicked in terminals and follows every terminal view for printed URLs.
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(MarleyTerminalUrl(Arc::new(open_clicked)));
    cx.set_global(ServedUrls::default());
    cx.observe_new(|_: &mut TerminalView, _, cx: &mut Context<TerminalView>| follow(cx))
        .detach();
}

/// The hook Zed's terminal view asks: opens `url` where it goes and says so, or leaves a URL
/// that is not http or https to Zed.
fn open_clicked(
    context: &MarleyFooterContext,
    url: &str,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let over_ssh = over_ssh(context.project.upgrade(), context.terminal, cx);
    let Some(destination) = destination(url, over_ssh, window.modifiers().shift, cx) else {
        return false;
    };
    open(destination, context.workspace.clone(), window, cx);
    true
}

/// Where `url` opens from a terminal, or `None` for a URL that is not http or https. Over SSH,
/// every URL goes to the system browser as printed; `inverted`, for Shift+Ctrl+click, sends it to
/// the place the setting would not.
fn destination(url: &str, over_ssh: bool, inverted: bool, cx: &App) -> Option<Destination> {
    let parsed = url::Url::parse(url).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    if over_ssh {
        return Some(Destination::SystemBrowser(url.to_string()));
    }
    let local = address::local_url(url);
    let in_tab = match MarleySettings::get_global(cx).terminal_links {
        MarleyTerminalLinks::LocalInBrowserTab => local.is_some(),
        MarleyTerminalLinks::AllInBrowserTab => true,
        MarleyTerminalLinks::SystemBrowser => false,
    };
    let url = local.map_or_else(|| url.to_string(), |local| local.url);
    Some(if in_tab == inverted {
        Destination::SystemBrowser(url)
    } else {
        Destination::BrowserTab(url)
    })
}

/// The URL to open in a Browser tab for a program that opened `url` through `BROWSER` (#561), or
/// none when `marley.terminal_links` sends it to the system browser. The opener runs on this
/// machine, so SSH plays no part, and no key is held.
pub(crate) fn browser_tab_url(url: &str, cx: &App) -> Option<String> {
    match destination(url, false, false, cx)? {
        Destination::BrowserTab(url) => Some(url),
        Destination::SystemBrowser(_) => None,
    }
}

fn open(destination: Destination, workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    match destination {
        Destination::SystemBrowser(url) => cx.open_url(&url),
        // A click's URL arrives while its terminal view is being updated, and a new tab
        // deactivates the pane's front item, which is that view: the tab opens after.
        Destination::BrowserTab(url) => window.defer(cx, move |window, cx| {
            workspace
                .update(cx, |workspace, cx| {
                    browser::open_url_tab(workspace, url, window, cx);
                })
                .log_err();
        }),
    }
}

/// Whether `terminal`'s `localhost` is another machine: its project is remote, or its foreground
/// program is an SSH client.
fn over_ssh(project: Option<Entity<Project>>, terminal: &Entity<Terminal>, cx: &App) -> bool {
    project.is_some_and(|project| !project.read(cx).is_local())
        || terminal
            .read(cx)
            .foreground_process_command_name()
            .is_some_and(|name| SSH_CLIENTS.contains(&name.as_str()))
}

/// Reads a terminal view's lines after its output, and forgets its URLs when it goes.
fn follow(cx: &mut Context<TerminalView>) {
    cx.subscribe_self(|_, event: &terminal::Event, cx| {
        if matches!(event, terminal::Event::Wakeup) {
            schedule_scan(cx);
        }
    })
    .detach();
    let id = cx.entity_id();
    cx.on_release(move |_, cx| {
        cx.default_global::<ServedUrls>().terminals.remove(&id);
    })
    .detach();
}

fn schedule_scan(cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    // Read without `default_global`, which tells the global's observers it changed, on every
    // wakeup.
    if cx
        .try_global::<ServedUrls>()
        .and_then(|served| served.terminals.get(&id))
        .is_some_and(|printed| printed.scan.is_some())
    {
        return;
    }
    let view = cx.weak_entity();
    let scan = cx.spawn(async move |this, cx| {
        cx.background_executor().timer(SCAN_DELAY).await;
        this.update(cx, |view, cx| scan(view, cx)).log_err();
    });
    cx.default_global::<ServedUrls>()
        .terminals
        .entry(id)
        .or_insert_with(|| Printed {
            view,
            urls: Vec::new(),
            scan: None,
        })
        .scan = Some(scan);
}

/// Reads the view's last lines for local URLs, redraws its footer when its offer changed, and
/// starts reading the ports when a terminal holds a URL.
fn scan(view: &TerminalView, cx: &mut Context<TerminalView>) {
    let id = cx.entity_id();
    let project = view
        .marley_workspace()
        .upgrade()
        .map(|workspace| workspace.read(cx).project().clone());
    let found: Vec<LocalUrl> = if over_ssh(project, view.terminal(), cx) {
        Vec::new()
    } else {
        view.terminal()
            .read(cx)
            .last_n_non_empty_lines(SCANNED_LINES)
            .iter()
            .flat_map(|line| address::printed_local_urls(line))
            .collect()
    };
    let weak = cx.weak_entity();
    let served = cx.default_global::<ServedUrls>();
    let listening = served.listening.clone();
    let printed = served.terminals.entry(id).or_insert_with(|| Printed {
        view: weak,
        urls: Vec::new(),
        scan: None,
    });
    // The task running this scan: it ends by itself.
    if let Some(running) = printed.scan.take() {
        running.detach();
    }
    let before = printed.offer(&listening).cloned();
    for url in found {
        printed.remember(url);
    }
    let changed = printed.offer(&listening) != before.as_ref();
    let start_watching = !served.watching
        && served
            .terminals
            .values()
            .any(|printed| !printed.urls.is_empty());
    if changed {
        cx.notify();
    }
    if start_watching {
        watch_ports(cx);
    }
}

/// Reads the listening ports every [`PORTS_POLL`] while any terminal holds a printed URL, and
/// redraws each terminal whose offer changed.
fn watch_ports(cx: &mut App) {
    cx.default_global::<ServedUrls>().watching = true;
    cx.spawn(async move |cx| {
        loop {
            let ports = cx
                .background_spawn(futures::future::lazy(|_| {
                    marley_browser::ports::listening_ports_in(Path::new(PROC_NET))
                }))
                .await
                .log_err()
                .unwrap_or_default();
            if !cx.update(|cx| take_ports(ports, cx)) {
                break;
            }
            cx.background_executor().timer(PORTS_POLL).await;
        }
    })
    .detach();
}

/// Keeps `ports` as the listening ones and redraws each terminal whose offer they change; says
/// whether any terminal still holds a URL, and stops the watch when none does.
fn take_ports(ports: BTreeSet<u16>, cx: &mut App) -> bool {
    let served = cx.default_global::<ServedUrls>();
    let changed: Vec<WeakEntity<TerminalView>> = served
        .terminals
        .values()
        .filter(|printed| printed.offer(&served.listening) != printed.offer(&ports))
        .map(|printed| printed.view.clone())
        .collect();
    served.listening = ports;
    let more = served
        .terminals
        .values()
        .any(|printed| !printed.urls.is_empty());
    served.watching = more;
    for view in changed {
        if let Some(view) = view.upgrade() {
            view.update(cx, |_, cx| cx.notify());
        }
    }
    more
}

/// The URL `context`'s terminal offers: its newest printed local URL whose port listens, unless
/// the terminal reaches another machine.
pub fn offer(context: &MarleyFooterContext, cx: &App) -> Option<LocalUrl> {
    if over_ssh(context.project.upgrade(), context.terminal, cx) {
        return None;
    }
    let served = cx.try_global::<ServedUrls>()?;
    served
        .terminals
        .get(&context.view.entity_id())?
        .offer(&served.listening)
        .cloned()
}

/// The offer as a split button: its label opens the URL as a Ctrl+click on it would, and its
/// arrow lists Open in Browser Tab, Open in System Browser and Copy URL.
#[must_use]
pub fn offer_button(context: &MarleyFooterContext, url: LocalUrl) -> AnyElement {
    let id = context.view.entity_id();
    let (workspace, menu_workspace) = (context.workspace.clone(), context.workspace.clone());
    let (open_url, menu_url) = (url.url.clone(), url.url.clone());
    let left = ButtonLike::new_rounded_left(("marley-served-url", id))
        .child(
            h_flex()
                .gap_1()
                .child(
                    Icon::new(IconName::ToolWeb)
                        .size(IconSize::XSmall)
                        .color(Color::Muted),
                )
                .child(Label::new(url.label).size(LabelSize::Small)),
        )
        .tooltip(Tooltip::text(format!("Open {}", url.url)))
        .on_click(move |_, window, cx| {
            if let Some(destination) = destination(&open_url, false, false, cx) {
                open(destination, workspace.clone(), window, cx);
            }
        });
    let right = PopoverMenu::new(("marley-served-url-menu", id))
        .trigger(
            ButtonLike::new_rounded_right(("marley-served-url-more", id))
                .child(Icon::new(IconName::ChevronDown).size(IconSize::XSmall)),
        )
        .menu(move |window, cx| {
            let (tab_url, system_url, copied_url) =
                (menu_url.clone(), menu_url.clone(), menu_url.clone());
            let workspace = menu_workspace.clone();
            Some(ContextMenu::build(window, cx, move |menu, _, _| {
                menu.entry("Open in Browser Tab", None, move |window, cx| {
                    open(
                        Destination::BrowserTab(tab_url.clone()),
                        workspace.clone(),
                        window,
                        cx,
                    );
                })
                .entry("Open in System Browser", None, move |_, cx| {
                    cx.open_url(&system_url);
                })
                .entry("Copy URL", None, move |_, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copied_url.clone()));
                })
            }))
        })
        // The footer is at the terminal's bottom, so the menu opens upward.
        .anchor(Anchor::BottomRight)
        .into_any_element();
    div()
        .debug_selector(|| "marley-served-url".into())
        .child(SplitButton::new(left, right).style(SplitButtonStyle::Outlined))
        .into_any_element()
}
