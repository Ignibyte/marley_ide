//! The pause before a consequential click (#571), which waits for the user's Allow or Refuse.
//!
//! An agent's click in a Browser tab that pays, deletes, sends in the user's name or changes an
//! account waits, by default only for agents that run without a permission prompt of their own.
//!
//! `marley_browser::consequence` decides from the element, its form and the page, and a rule's
//! pause is a `rules` row. What the rules leave open is asked of the System One layer, which may
//! add a pause in `act` and never removes one. The pause is a card under the tab's toolbar and a
//! toast with Show; the card never takes the focus by itself, so a key meant for the page never
//! answers it. After 25 seconds, under the transport's 30, the click is refused, and Allow clicks
//! only the same element on the same page.

use std::pin::pin;
use std::time::{Duration, Instant};

use futures::future::{self, Either};
use gpui::{App, AsyncApp, Entity};
use marley_agent::claude_events::PERMISSION_MODE_LABEL;
use marley_browser::cdp::CdpError;
use marley_browser::consequence::{self, Class, ClickFacts, Verdict};
use marley_browser::observe::redact_url;
use marley_browser::page::{NodeFacts, Page};
use marley_browser::snapshot::RefTarget;
use marley_mcp::AppCall;
use marley_system_one::CLICK_CONSEQUENCE;
use marley_system_one::reading::{Reading, Signal};
use settings::{MarleyClickPauseAgents, Settings as _, SystemOneMode, ToolPermissionMode};
use ui::SharedString;

use crate::MarleySettings;
use crate::agent_events::AgentEvents;
use crate::browser::{self, BrowserHub};
use crate::system_one::{self, Asking};

/// The use's name, which its mode is set under.
const USE: &str = "click_consequence";

/// How long a pause waits for the user: under the transport's 30 seconds, so the agent reads the
/// refusal rather than a timeout.
const WAIT: Duration = Duration::from_secs(25);

/// The name Zed's agent gives Marley's server at `initialize`.
const ZED_CLIENT: &str = "Zed";

/// How Zed names Marley's `browser_click` among its tools' permissions.
const ZED_TOOL: &str = "mcp:marley:browser_click";

/// Claude Code's permission modes that ask for no tool call.
const PROMPTLESS_MODES: [&str; 2] = ["bypassPermissions", "dontAsk"];

/// What a write tool says while a click waits in its tab.
pub(crate) const PAUSED_WRITE: &str = "a click is paused in this tab; wait for the user";

/// Who made a browser call, as the pause sorts callers (the spec's D2).
#[derive(Debug, Clone)]
pub(crate) struct Who {
    /// The caller in words, for the card: `Claude Code`, `Zed's agent`, a client's name.
    words: String,
    /// Whether the caller asks the user before a tool of Marley's runs.
    prompts: bool,
    /// An outside client's name, for the recorder (#524).
    by: Option<SharedString>,
}

impl Who {
    /// Sorts the caller of `call`. An outside client (#524) is unknown. A call from a Marley
    /// terminal (#520) asks first unless its Claude Code runs in a mode that asks for nothing; one
    /// with no Claude Code there is unknown. With no terminal, Zed's agent asks first unless Zed
    /// lets `browser_click` run without asking; any other caller is unknown. Unknown callers do not
    /// ask. The names are courtesies, never an authority: `all_agents` pauses everyone.
    pub(crate) fn of(call: &AppCall, cx: &App) -> Self {
        let unknown = |words: &str| Self {
            words: words.to_string(),
            prompts: false,
            by: None,
        };
        if let Some(name) = call.principal().client_name() {
            return Self {
                by: Some(SharedString::from(name.to_string())),
                ..unknown(name)
            };
        }
        let caller = call.caller();
        if let Some((_, view)) = crate::mcp::caller_terminal(caller, cx) {
            let mode = cx
                .try_global::<AgentEvents>()
                .and_then(|events| events.seat(view.entity_id()))
                .and_then(|seat| seat.labels.get(PERMISSION_MODE_LABEL).cloned());
            return mode.map_or_else(
                || unknown("An agent in a terminal"),
                |mode| Self {
                    prompts: !PROMPTLESS_MODES.contains(&mode.as_str()),
                    ..unknown("Claude Code")
                },
            );
        }
        match caller.client.as_deref() {
            Some(ZED_CLIENT) => Self {
                prompts: !zed_runs_without_asking(cx),
                ..unknown("Zed's agent")
            },
            Some(name) => unknown(name),
            None => unknown("An agent"),
        }
    }
}

/// Whether Zed's agent runs Marley's `browser_click` without asking: that tool's own rule, else
/// Zed's default, allows it.
fn zed_runs_without_asking(cx: &App) -> bool {
    let permissions = &agent_settings::AgentSettings::get_global(cx).tool_permissions;
    let mode = permissions
        .tools
        .get(ZED_TOOL)
        .and_then(|rules| rules.default)
        .unwrap_or(permissions.default);
    mode == ToolPermissionMode::Allow
}

/// An agent's click in a tab, before it is made.
pub(crate) struct Click<'a> {
    /// The tab's page.
    pub(crate) page: &'a Page,
    /// The tab.
    pub(crate) tab: &'a str,
    /// The ref it names, when it names one.
    pub(crate) target: Option<&'a RefTarget>,
    /// Where it goes in the page's viewport.
    pub(crate) point: (f64, f64),
    /// What the tab's chip calls it: `button “Place order”`, `the point 120, 300`.
    pub(crate) what: &'a str,
}

/// What the pause decided for a click.
pub(crate) enum Checked {
    /// Click at once.
    Go,
    /// The user allowed it and the element is as it was: place it again and click.
    Allowed,
}

/// How a pause ended.
enum Answer {
    /// The user allowed the click, or refused it.
    Given(bool),
    /// The page went, and its pause with it.
    Gone,
    /// No answer came in time.
    Expired,
}

/// The element a paused click names, as it was when the click waited, read again at Allow.
struct Identity {
    url: Option<SharedString>,
    node: Option<(String, i64)>,
    read: Option<NodeFacts>,
}

/// Before `who`'s `click`: reads the element, decides, and when the click must wait, holds it
/// for the user (#571).
///
/// # Errors
///
/// The refusal the agent reads: the user refused, did not answer in time, or the page changed,
/// or another click already waits in the tab.
pub(crate) async fn before_click(
    click: &Click<'_>,
    who: &Who,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<Checked, String> {
    let Click {
        page,
        tab,
        target,
        point,
        what,
    } = *click;
    let (mode, all_agents) = cx.update(|cx| {
        (
            system_one::use_mode(USE, cx),
            MarleySettings::get_global(cx).browser_click_pause_agents
                == MarleyClickPauseAgents::AllAgents,
        )
    });
    if mode == SystemOneMode::Off || (who.prompts && !all_agents) {
        return Ok(Checked::Go);
    }
    let read = read_element(page, target, point).await;
    let (node, read) = match read {
        Ok((node, read)) => (Some(node), Some(read)),
        Err(error) => {
            log::debug!("browser: the element under an agent's click did not read: {error}");
            (None, None)
        }
    };
    let (url, title) = hub.read_with(cx, |hub, _| (hub.url(tab), hub.title(tab)));
    let facts = click_facts(target, read.as_ref(), url.as_deref().unwrap_or_default());
    let mut verdict = consequence::classify(&facts);
    // What could not be read is the layer's to read, unless the rules already pause it.
    if read.is_none() && !verdict.class.pauses() {
        verdict = Verdict {
            class: Class::Open,
            because: "the element could not be read".to_string(),
        };
    }
    if verdict.class == Class::Plain {
        return Ok(Checked::Go);
    }
    let asking = cx.update(|cx| asking(tab, &facts, &verdict, title.as_ref(), cx));
    let (why, call) = match verdict.class {
        Class::Open => {
            let asked = cx
                .update(|cx| system_one::ask(CLICK_CONSEQUENCE, &asking, cx))
                .await;
            let call = asked.row.as_ref().map(|row| row.id.clone());
            let Some((class, probability)) = held_noul(&asked.reading) else {
                return Ok(Checked::Go);
            };
            match asked.mode {
                SystemOneMode::Act => (
                    format!(
                        "which {} as the model reads it ({probability:.2})",
                        class.words()
                    ),
                    call,
                ),
                SystemOneMode::Suggest => {
                    let notice = format!(
                        "{} clicked {what}, which the model reads as a click that {} \
                         ({probability:.2})",
                        who.words,
                        class.words()
                    );
                    cx.update(|cx| browser::show_click_notice(tab, notice, cx));
                    return Ok(Checked::Go);
                }
                SystemOneMode::Off | SystemOneMode::Shadow => return Ok(Checked::Go),
            }
        }
        class => {
            let mut asking = asking;
            asking.verdict = class
                .noul()
                .map(|noul| system_one::noul_verdict(noul, true));
            let asked = cx.update(|cx| system_one::record(CLICK_CONSEQUENCE, &asking, cx));
            (
                format!("which {} ({})", class.words(), verdict.because),
                asked.row.map(|row| row.id),
            )
        }
    };
    let host = host_of(url.as_deref().unwrap_or_default());
    let sentence = format!("{} wants to click {what}, {why}, on {host}", who.words);
    let identity = Identity { url, node, read };
    hold(click, &sentence, &identity, who, call, hub, cx).await
}

/// Holds `click`, which `sentence` describes, until the user answers, or 25 seconds pass, and
/// logs how it ended as the outcome of the call that decided it.
async fn hold(
    click: &Click<'_>,
    sentence: &str,
    identity: &Identity,
    who: &Who,
    call: Option<String>,
    hub: &Entity<BrowserHub>,
    cx: &mut AsyncApp,
) -> Result<Checked, String> {
    let tab = click.tab;
    let receiver = hub.update(cx, |hub, cx| {
        hub.pause_click(
            tab,
            SharedString::from(sentence.to_string()),
            who.by.clone(),
            cx,
        )
    });
    let Some(receiver) = receiver else {
        return Err(PAUSED_WRITE.to_string());
    };
    cx.update(|cx| {
        browser::show_pause_toast(
            tab,
            format!("{sentence}. Allow it or refuse it in the tab."),
            cx,
        );
    });
    let started = Instant::now();
    let timer = pin!(cx.background_executor().timer(WAIT));
    // The page's going drops the answer's sender.
    let answer = match future::select(receiver, timer).await {
        Either::Left((Ok(allow), _)) => Answer::Given(allow),
        Either::Left((Err(_), _)) => Answer::Gone,
        Either::Right(_) => Answer::Expired,
    };
    cx.update(|cx| browser::dismiss_pause_toast(tab, cx));
    let paused = format!("Marley paused this click: {sentence}");
    let (how, result) = match answer {
        Answer::Given(true) if same_element(click, identity, hub, cx).await => {
            ("allowed", Ok(Checked::Allowed))
        }
        Answer::Given(true) => (
            "changed",
            Err(format!(
                "{paused}. The page changed while it waited, so nothing was clicked"
            )),
        ),
        Answer::Given(false) => (
            "refused",
            Err(format!(
                "{paused}. The user refused it; ask them before you try again"
            )),
        ),
        Answer::Gone => (
            "gone",
            Err(format!(
                "{paused}. The page went away while it waited, so nothing was clicked"
            )),
        ),
        Answer::Expired => (
            "expired",
            Err(format!(
                "{paused}. The user did not answer within {} seconds; ask them before you try \
                 again",
                WAIT.as_secs()
            )),
        ),
    };
    hub.update(cx, |hub, cx| hub.end_pause(tab, how, cx));
    if let Some(call) = call {
        let seconds = started.elapsed().as_secs();
        cx.update(|cx| system_one::outcome(&call, format!("{how} after {seconds} s"), cx));
    }
    result
}

/// Whether the element `click` names is as it was: the same page, and the same element with the
/// same role, name and tag under the ref or the point.
async fn same_element(
    click: &Click<'_>,
    identity: &Identity,
    hub: &Entity<BrowserHub>,
    cx: &AsyncApp,
) -> bool {
    let url = hub.read_with(cx, |hub, _| hub.url(click.tab));
    if url != identity.url {
        return false;
    }
    let Some((node, before)) = identity.node.as_ref().zip(identity.read.as_ref()) else {
        // Nothing was read before, so there is nothing to compare but the page.
        return true;
    };
    match read_element(click.page, click.target, click.point).await {
        Ok((now_node, now)) => {
            now_node == *node
                && now.role == before.role
                && now.name == before.name
                && now.tag == before.tag
        }
        Err(_) => false,
    }
}

/// The element under a click, read from the page: its session and node, and what it says of
/// itself.
async fn read_element(
    page: &Page,
    target: Option<&RefTarget>,
    point: (f64, f64),
) -> Result<((String, i64), NodeFacts), CdpError> {
    let (session, node) = match target {
        Some(target) => (
            target
                .session
                .clone()
                .unwrap_or_else(|| page.session_id().to_string()),
            target.backend_node_id,
        ),
        None => (
            page.session_id().to_string(),
            page.node_at(point.0, point.1).await?,
        ),
    };
    let read = page.node_facts(&session, node).await?;
    Ok(((session, node), read))
}

/// What code knows of the element: the snapshot's role and name for a ref, else the element's
/// own, and what the page says of its form, its link and the text around it.
fn click_facts(target: Option<&RefTarget>, read: Option<&NodeFacts>, url: &str) -> ClickFacts {
    let read = read.cloned().unwrap_or_default();
    let role = target
        .map(|target| target.role.clone())
        .filter(|role| !role.is_empty())
        .unwrap_or(read.role);
    let name = target
        .map(|target| target.name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or(read.name);
    ClickFacts {
        role,
        name,
        tag: read.tag,
        input_type: read.input_type,
        in_form: read.in_form,
        form_action: read.form_action,
        form_method: read.form_method,
        form_has_text_area: read.form_has_text_area,
        form_fields: read.form_fields,
        link: read.link,
        context: read.context,
        page_url: url.to_string(),
    }
}

/// What the layer is asked about a click: code's facts (the role, the tag, the input's type,
/// whether a form is around it and how it posts, the page's host, what the rules found) and the
/// page's own words as text (the element's name, the form's target, the page's path and title,
/// the text around it), which a metadata-only project keeps back.
fn asking(
    tab: &str,
    facts: &ClickFacts,
    verdict: &Verdict,
    title: Option<&SharedString>,
    cx: &mut App,
) -> Asking {
    let (folders, local) = browser::tab_workspace(tab, cx).map_or_else(
        || (Vec::new(), true),
        |workspace| system_one::project_of(workspace.read(cx), cx),
    );
    let project = system_one::project_name(&folders);
    let words = |value: &str| {
        if value.is_empty() {
            "none".to_string()
        } else {
            value.to_string()
        }
    };
    let texts = [
        ("name", facts.name.clone()),
        ("rules found", verdict.because.clone()),
        ("form posts to", path_of(&facts.form_action)),
        ("page path", path_of(&facts.page_url)),
        (
            "page title",
            title.map(ToString::to_string).unwrap_or_default(),
        ),
        ("text around it", facts.context.clone()),
    ]
    .into_iter()
    .filter(|(_, text)| !text.is_empty())
    .collect();
    Asking {
        subject: tab.to_string(),
        project: project.clone(),
        folders,
        local,
        facts: vec![
            ("project", project),
            ("role", words(&facts.role)),
            ("tag", words(&facts.tag)),
            (
                "input type",
                words(facts.input_type.as_deref().unwrap_or("")),
            ),
            (
                "in a form",
                if facts.in_form { "yes" } else { "no" }.to_string(),
            ),
            ("form method", words(&facts.form_method)),
            ("page host", host_of(&facts.page_url)),
            ("rules class", verdict.class.words().to_string()),
        ],
        texts,
        verdict: None,
    }
}

/// The class the reading's nouls hold, the likeliest first, with its probability.
fn held_noul(reading: &Reading) -> Option<(Class, f64)> {
    let Reading::Model(reads) = reading else {
        return None;
    };
    reads
        .iter()
        .filter_map(|read| match &read.signal {
            Signal::Noul {
                holds: true,
                probability,
            } => Class::from_noul(&read.key).map(|class| (class, *probability)),
            _ => None,
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
}

/// A URL's host and port, as the card names the page.
fn host_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            let host = parsed.host_str()?.to_string();
            Some(
                parsed
                    .port()
                    .map_or_else(|| host.clone(), |port| format!("{host}:{port}")),
            )
        })
        .unwrap_or_else(|| "this page".to_string())
}

/// A URL's path, its secret-looking parts hidden, without its query.
fn path_of(url: &str) -> String {
    url::Url::parse(&redact_url(url))
        .map(|parsed| parsed.path().to_string())
        .unwrap_or_default()
}
