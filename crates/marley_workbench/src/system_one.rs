//! The System One layer in the app (#565): the settings, the key, the network, the files and the
//! gate around `marley_system_one`'s pure core.
//!
//! `ask` is the one way a use asks. With the layer or the use off it answers
//! [`Reading::Off`] and does nothing else: no request, no key read, no file. Otherwise it decides
//! what the project may send, builds the masked state, and asks the configured provider inside the
//! use's deadline. Every call, refused and failed ones included, becomes a row under
//! `<data dir>/system_one/`, and the `SystemOne` global keeps this session's rows for the
//! Decisions view. The check (`marley: system one check`) is the first use: it asks whether the
//! last command of the terminal used last failed, and shows the answer in a toast. The stop kind
//! (#566) is the second: `agent_events` asks it when a Claude Code turn stops, logs what its own
//! rules settle through `record`, and logs the user's next prompt through `outcome`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use futures::channel::mpsc;
use futures::future::{Either, select};
use futures::{AsyncReadExt as _, StreamExt as _};
use gpui::{
    App, AppContext as _, BackgroundExecutor, Context, Entity, Global, SharedString, Task, Window,
};
use http_client::{AsyncBody, Host, HttpClient, HttpRequestExt as _, Method, Request, Url};
use marley_mcp::redact::marker;
use marley_system_one::files::{self, CallRow, OutcomeRow, Replay, Row};
use marley_system_one::policy::{self, Refusal};
use marley_system_one::reading::{self, Read, Reading, Signal};
use marley_system_one::request::{self, Answer, Answers};
use marley_system_one::state::{Detail, State, StateBuilder};
use marley_system_one::{CHECK, DEFAULT_MODEL, UseSpec};
use settings::{
    Settings as _, SettingsStore, SystemOneMode, SystemOneProvider, SystemOneSettingsContent,
};
use terminal_view::TerminalView;
use util::ResultExt as _;
use workspace::item::Item as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use crate::{MarleySettings, OpenDecisions, SystemOneCheck, browser, decisions, mcp};

/// The variable the key is read from first.
pub(crate) const KEY_VARIABLE: &str = "MARLEY_SYSTEM_ONE_KEY";

/// The endpoint of the `typesafe` provider, which is also where the keyring keeps its key.
const TYPESAFE_URL: &str = "https://api.typesafe.ai/v1/systemone";

/// The keyring's username for the key.
const KEY_USERNAME: &str = "system-one";

/// What `api.typesafe.ai` charges for Jev, in thousandths of a cent per million input tokens
/// (0.042 USD).
const TYPESAFE_PRICE: u64 = 4_200;

/// Statuses a call is tried again on, once, while the deadline leaves time.
const RETRIED: [u16; 3] = [429, 503, 529];

/// How long to wait before trying again.
const RETRY_PAUSE: Duration = Duration::from_millis(200);

/// How long past the deadline the timer that backs up the request's own timeout fires.
const BACKSTOP: Duration = Duration::from_millis(250);

/// The rows of this session the global keeps for the Decisions view.
const KEPT_ROWS: usize = 500;

/// Numbers each call of this process, so two calls in one millisecond keep apart.
static CALLS: AtomicU64 = AtomicU64::new(0);

/// The System One layer's settings, as resolved (#565).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemOneSettings {
    /// Whether the layer asks at all.
    pub enabled: bool,
    /// Who answers.
    pub provider: SystemOneProvider,
    /// The `compatible` provider's URL.
    pub endpoint: Option<String>,
    /// The model asked.
    pub model: String,
    /// Folders whose projects may send their state.
    pub projects: Vec<PathBuf>,
    /// Folders whose projects send facts alone.
    pub metadata_only_projects: Vec<PathBuf>,
    /// The day's budget, in cents.
    pub daily_budget_cents: u64,
    /// A `compatible` provider's price, in thousandths of a cent per million input tokens.
    pub price: u64,
    /// Each use's mode, by the use's name.
    pub uses: BTreeMap<String, SystemOneMode>,
}

impl SystemOneSettings {
    /// The settings from `marley.system_one`, each one missing at its default.
    pub(crate) fn from_content(content: Option<&SystemOneSettingsContent>) -> Self {
        let text = |value: Option<&String>| {
            value
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        };
        let folders = |list: Option<&Vec<String>>| {
            list.map(|list| {
                list.iter()
                    .filter_map(|folder| folder_path(folder))
                    .collect()
            })
            .unwrap_or_default()
        };
        Self {
            enabled: content.and_then(|content| content.enabled).unwrap_or(false),
            provider: content
                .and_then(|content| content.provider)
                .unwrap_or_default(),
            endpoint: text(content.and_then(|content| content.endpoint.as_ref())),
            model: text(content.and_then(|content| content.model.as_ref()))
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            projects: folders(content.and_then(|content| content.projects.as_ref())),
            metadata_only_projects: folders(
                content.and_then(|content| content.metadata_only_projects.as_ref()),
            ),
            daily_budget_cents: content
                .and_then(|content| content.daily_budget_cents)
                .unwrap_or(50),
            price: content
                .and_then(|content| content.price_cents_per_million_tokens)
                .map_or(0, thousandths),
            uses: content
                .and_then(|content| content.uses.clone())
                .unwrap_or_default(),
        }
    }
}

/// A folder from the settings, with `~/` as the home directory; an empty one is none.
fn folder_path(folder: &str) -> Option<PathBuf> {
    let folder = folder.trim();
    if folder.is_empty() {
        return None;
    }
    Some(folder.strip_prefix("~/").map_or_else(
        || PathBuf::from(folder),
        |rest| util::paths::home_dir().join(rest),
    ))
}

/// Cents per million tokens as thousandths of a cent, rounded. A price that is not a number, or
/// below zero, is none; one too large for a `u64` is the most there is.
fn thousandths(cents: f32) -> u64 {
    let thousandths = f64::from(cents) * 1000.0;
    if thousandths.is_nan() || thousandths < 0.0 {
        return 0;
    }
    // `f64` has no lossless conversion to `u64`: the rounded decimal parses to one when it fits.
    format!("{thousandths:.0}").parse().unwrap_or(u64::MAX)
}

/// The key, which prints as `Key(***)` and leaves only as a request's header.
#[derive(Clone)]
struct Key(String);

impl fmt::Debug for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Key(***)")
    }
}

impl Key {
    /// The `Authorization` header's value.
    fn bearer(&self) -> String {
        format!("Bearer {}", self.0)
    }

    /// `text` with the key hidden: #516's rules do not name `MARLEY_SYSTEM_ONE_KEY`.
    fn mask(&self, text: &str) -> String {
        text.replace(&self.0, &marker("secret"))
    }
}

/// Where the key came from, as the Decisions view names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeySource {
    /// The layer is off, or its provider needs no key.
    NotNeeded,
    /// `MARLEY_SYSTEM_ONE_KEY`.
    Environment,
    /// The system keyring, at the provider's URL.
    Keyring,
    /// Being read from the keyring.
    Reading,
    /// There is none, and why.
    Missing(String),
}

impl fmt::Display for KeySource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotNeeded => formatter.write_str("not needed"),
            Self::Environment => write!(formatter, "environment ({KEY_VARIABLE})"),
            Self::Keyring => formatter.write_str("keyring"),
            Self::Reading => formatter.write_str("reading the keyring"),
            Self::Missing(reason) => write!(formatter, "none ({reason})"),
        }
    }
}

/// The System One layer's state: the settings it applied, the key, the gate, the recorded
/// answers, and this session's rows.
pub(crate) struct SystemOne {
    settings: SystemOneSettings,
    key: Option<Key>,
    key_source: KeySource,
    /// Whether the provider refused the key; calls wait until the key changes.
    key_refused: bool,
    /// Counts the key's loads, so a keyring read that finishes after a newer load is dropped.
    key_load: u64,
    gate: policy::Gate,
    replay: Replay,
    rows: Vec<CallRow>,
    /// Where rows go to be written, once the first call is made.
    log: Option<mpsc::UnboundedSender<(String, Row)>>,
    started: Instant,
}

impl Default for SystemOne {
    fn default() -> Self {
        Self {
            settings: SystemOneSettings::from_content(None),
            key: None,
            key_source: KeySource::NotNeeded,
            key_refused: false,
            key_load: 0,
            gate: policy::Gate::default(),
            replay: Replay::default(),
            rows: Vec::new(),
            log: None,
            started: Instant::now(),
        }
    }
}

impl Global for SystemOne {}

impl SystemOne {
    /// The settings the layer applies.
    pub(crate) const fn settings(&self) -> &SystemOneSettings {
        &self.settings
    }

    /// Where the key came from.
    pub(crate) fn key_source(&self) -> KeySource {
        if self.key_refused {
            KeySource::Missing(format!(
                "the provider refused the key from {}",
                self.key_source
            ))
        } else {
            self.key_source.clone()
        }
    }

    /// This session's rows, oldest first.
    pub(crate) fn rows(&self) -> &[CallRow] {
        &self.rows
    }

    /// What today's calls cost, in billionths of a cent.
    pub(crate) fn spent_today(&self) -> u64 {
        self.gate.spent_on(&today())
    }

    /// Whether the breaker holds calls back.
    pub(crate) fn breaker_open(&self) -> bool {
        self.gate.breaker_open(self.now())
    }

    /// Milliseconds since the layer's state was made: the gate's clock.
    fn now(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

/// Installs the layer: the settings it follows, the check and the Decisions view.
pub(crate) fn init(cx: &mut App) {
    apply_settings(cx);
    cx.observe_global::<SettingsStore>(apply_settings).detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &SystemOneCheck, window, cx| {
            run_check(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &OpenDecisions, window, cx| {
            decisions::open(workspace, window, cx);
        });
    })
    .detach();
}

/// Applies `marley.system_one` when it changed, reading the key and the recorded answers again
/// when what they depend on changed.
fn apply_settings(cx: &mut App) {
    let settings = MarleySettings::get_global(cx).system_one.clone();
    let before = cx
        .try_global::<SystemOne>()
        .map(|layer| layer.settings.clone());
    if before.as_ref() == Some(&settings) {
        return;
    }
    let key_changed = before.as_ref().is_none_or(|before| {
        before.enabled != settings.enabled
            || before.provider != settings.provider
            || before.endpoint != settings.endpoint
    });
    let replay_changed = before.as_ref().is_none_or(|before| {
        before.enabled != settings.enabled || before.provider != settings.provider
    });
    cx.default_global::<SystemOne>().settings = settings;
    if key_changed {
        load_key(cx);
    }
    if replay_changed {
        load_replay(cx);
    }
}

/// Reads the key the settings call for: `MARLEY_SYSTEM_ONE_KEY`, else the keyring at the
/// provider's URL. With the layer off, or a provider that sends nothing, nothing is read, so a
/// user who leaves the layer off never meets the keyring's unlock prompt.
pub(crate) fn load_key(cx: &mut App) {
    let settings = {
        let layer = cx.default_global::<SystemOne>();
        layer.key_load += 1;
        layer.key = None;
        layer.key_refused = false;
        layer.settings.clone()
    };
    if !settings.enabled || !needs_key(settings.provider) {
        cx.default_global::<SystemOne>().key_source = KeySource::NotNeeded;
        return;
    }
    if let Some(value) = env_var::EnvVar::new(SharedString::new_static(KEY_VARIABLE))
        .value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        let layer = cx.default_global::<SystemOne>();
        layer.key = Some(Key(value));
        layer.key_source = KeySource::Environment;
        return;
    }
    let url = match endpoint(&settings) {
        Ok(url) => url,
        Err(reason) => {
            cx.default_global::<SystemOne>().key_source = KeySource::Missing(reason);
            return;
        }
    };
    let load = {
        let layer = cx.default_global::<SystemOne>();
        layer.key_source = KeySource::Reading;
        layer.key_load
    };
    let read = cx.read_credentials(url.as_str());
    cx.spawn(async move |cx| {
        let found = read.await;
        cx.update(|cx| {
            let layer = cx.default_global::<SystemOne>();
            if layer.key_load != load {
                return;
            }
            layer.key_source = match found {
                Ok(Some((_, secret))) => match String::from_utf8(secret) {
                    Ok(key) if !key.trim().is_empty() => {
                        layer.key = Some(Key(key.trim().to_string()));
                        KeySource::Keyring
                    }
                    _ => KeySource::Missing("the keyring's item is not a key".to_string()),
                },
                Ok(None) => {
                    KeySource::Missing(format!("set {KEY_VARIABLE}, or Set Key in Decisions"))
                }
                Err(error) => {
                    KeySource::Missing(format!("the keyring could not be read: {error:#}"))
                }
            };
        });
    })
    .detach();
}

/// Writes `key` to the keyring at the provider's URL, then reads the key again.
pub(crate) fn store_key(key: &str, cx: &mut App) -> Task<anyhow::Result<()>> {
    let settings = cx.default_global::<SystemOne>().settings.clone();
    let url = match endpoint(&settings) {
        Ok(url) => url,
        Err(reason) => return Task::ready(Err(anyhow::anyhow!(reason))),
    };
    let write = cx.write_credentials(url.as_str(), KEY_USERNAME, key.trim().as_bytes());
    cx.spawn(async move |cx| {
        write.await?;
        cx.update(load_key);
        Ok(())
    })
}

/// Removes the key from the keyring at the provider's URL, then reads the key again.
pub(crate) fn forget_key(cx: &mut App) -> Task<anyhow::Result<()>> {
    let settings = cx.default_global::<SystemOne>().settings.clone();
    let url = match endpoint(&settings) {
        Ok(url) => url,
        Err(reason) => return Task::ready(Err(anyhow::anyhow!(reason))),
    };
    let delete = cx.delete_credentials(url.as_str());
    cx.spawn(async move |cx| {
        delete.await?;
        cx.update(load_key);
        Ok(())
    })
}

/// Loads `replay.jsonl` when the provider is `replay`, off the main thread.
fn load_replay(cx: &mut App) {
    let settings = cx.default_global::<SystemOne>().settings.clone();
    if !settings.enabled || settings.provider != SystemOneProvider::Replay {
        cx.default_global::<SystemOne>().replay = Replay::default();
        return;
    }
    let dir = folder();
    let loading = cx.background_spawn(futures::future::lazy(move |_| Replay::load_in(&dir)));
    cx.spawn(async move |cx| {
        let loaded = loading.await;
        cx.update(|cx| match loaded {
            Ok(replay) => cx.default_global::<SystemOne>().replay = replay,
            Err(error) => log::warn!("system one: reading the recorded answers: {error:#}"),
        });
    })
    .detach();
}

/// The layer's folder under Marley's data directory.
pub(crate) fn folder() -> PathBuf {
    paths::data_dir().join("system_one")
}

/// Today in local time, as the log's file names and the budget count days.
pub(crate) fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Whether `provider` sends requests, and so needs a key.
const fn needs_key(provider: SystemOneProvider) -> bool {
    matches!(
        provider,
        SystemOneProvider::Typesafe | SystemOneProvider::Compatible
    )
}

/// The provider's name, as rows and settings write it.
pub(crate) const fn provider_name(provider: SystemOneProvider) -> &'static str {
    match provider {
        SystemOneProvider::Typesafe => "typesafe",
        SystemOneProvider::Compatible => "compatible",
        SystemOneProvider::Rules => "rules",
        SystemOneProvider::Replay => "replay",
    }
}

/// The mode's name, as rows and settings write it.
const fn mode_name(mode: SystemOneMode) -> &'static str {
    match mode {
        SystemOneMode::Off => "off",
        SystemOneMode::Shadow => "shadow",
        SystemOneMode::Suggest => "suggest",
        SystemOneMode::Act => "act",
    }
}

/// Where the provider's requests go. A `compatible` endpoint must be `https`, or `http` on this
/// machine, so the key never crosses a network in the clear.
fn endpoint(settings: &SystemOneSettings) -> Result<Url, String> {
    match settings.provider {
        SystemOneProvider::Typesafe => Url::parse(TYPESAFE_URL).map_err(|error| error.to_string()),
        SystemOneProvider::Compatible => {
            let endpoint = settings.endpoint.as_deref().ok_or_else(|| {
                "set marley.system_one.endpoint for the compatible provider".to_string()
            })?;
            let url = Url::parse(endpoint).map_err(|_| format!("`{endpoint}` is not a URL"))?;
            let on_this_machine = match url.host() {
                Some(Host::Domain(domain)) => domain == "localhost",
                Some(Host::Ipv4(address)) => address.is_loopback(),
                Some(Host::Ipv6(address)) => address.is_loopback(),
                None => false,
            };
            if url.scheme() == "https" || (url.scheme() == "http" && on_this_machine) {
                Ok(url)
            } else {
                Err("the endpoint must be https, or http on this machine".to_string())
            }
        }
        SystemOneProvider::Rules | SystemOneProvider::Replay => {
            Err("this provider makes no request".to_string())
        }
    }
}

/// What a use asks about.
pub(crate) struct Asking {
    /// What the call is about, for the repeat check: a terminal, a tab, an entry.
    pub(crate) subject: String,
    /// The project's name.
    pub(crate) project: String,
    /// The project's folders.
    pub(crate) folders: Vec<PathBuf>,
    /// Whether the project is on this machine.
    pub(crate) local: bool,
    /// Facts computed in code, which every state keeps.
    pub(crate) facts: Vec<(&'static str, String)>,
    /// Text, which a metadata-only project's state leaves out.
    pub(crate) texts: Vec<(&'static str, String)>,
    /// The use's own verdict, which the `rules` provider answers with.
    pub(crate) verdict: Option<Answers>,
}

/// What came of an ask.
pub(crate) struct Asked {
    /// What the use may act on.
    pub(crate) reading: Reading,
    /// The use's mode.
    pub(crate) mode: SystemOneMode,
    /// Whether the layer was on.
    pub(crate) layer_on: bool,
    /// The row the call left, when one was made.
    pub(crate) row: Option<CallRow>,
    /// The answers the reading came from, when a provider or the replay gave some: a choice's
    /// probability for each option, which a use can rank by (#567).
    pub(crate) answers: Option<Answers>,
}

/// A row being filled in as a call goes on.
struct Draft {
    day: String,
    row: CallRow,
    subject: String,
    answers: Option<Answers>,
}

impl Draft {
    fn new(
        spec: &UseSpec,
        settings: &SystemOneSettings,
        asking: &Asking,
        mode: SystemOneMode,
    ) -> Self {
        let now = chrono::Local::now();
        let number = CALLS.fetch_add(1, Ordering::Relaxed);
        Self {
            day: now.format("%Y-%m-%d").to_string(),
            subject: format!("{}:{}", spec.name, asking.subject),
            answers: None,
            row: CallRow {
                id: format!("{}-{number}", now.format("%Y%m%dT%H%M%S%3f")),
                time: now.to_rfc3339(),
                use_name: spec.name.to_string(),
                set: spec.set.id.to_string(),
                model: settings.model.clone(),
                provider: provider_name(settings.provider).to_string(),
                project: Some(asking.project.clone()),
                mode: mode_name(mode).to_string(),
                verdict: asking.verdict.as_ref().map(|verdict| verdict.raw.clone()),
                state: None,
                state_hash: None,
                questions: spec
                    .set
                    .questions
                    .iter()
                    .map(|question| question.key().to_string())
                    .collect(),
                answers: None,
                reading: String::new(),
                failed: false,
                thresholds: None,
                latency_ms: None,
                input_tokens: None,
                cost: None,
                error: None,
            },
        }
    }

    fn answered(&mut self, answers: &Answers) {
        self.row.answers = Some(answers.raw.clone());
        self.row.thresholds = Some(files::thresholds());
        self.row.input_tokens = Some(answers.input_tokens);
        self.answers = Some(answers.clone());
    }
}

/// Asks `spec`'s questions about what `asking` describes.
///
/// With the layer or the use off the answer is [`Reading::Off`] and nothing else happens. Every
/// other ask leaves a row, and model trouble comes back as a reading that says so. A `UseSpec` is
/// `Copy` and its references are `'static`, so a use may make one at call time (#567).
pub(crate) fn ask(spec: UseSpec, asking: &Asking, cx: &mut App) -> Task<Asked> {
    let settings = cx.try_global::<SystemOne>().map_or_else(
        || SystemOneSettings::from_content(None),
        |layer| layer.settings.clone(),
    );
    if !settings.enabled {
        return Task::ready(Asked {
            reading: Reading::Off,
            mode: SystemOneMode::Off,
            layer_on: false,
            row: None,
            answers: None,
        });
    }
    let mode = settings
        .uses
        .get(spec.name)
        .copied()
        .unwrap_or(SystemOneMode::Off);
    if mode == SystemOneMode::Off {
        return Task::ready(Asked {
            reading: Reading::Off,
            mode,
            layer_on: true,
            row: None,
            answers: None,
        });
    }
    let mut draft = Draft::new(&spec, &settings, asking, mode);
    let detail = match policy::may_send(
        &asking.folders,
        asking.local,
        &settings.projects,
        &settings.metadata_only_projects,
    ) {
        Ok(detail) => detail,
        Err(refusal) => return Task::ready(finish(draft, refused(refusal), mode, cx)),
    };
    let Some(state) = state_for(asking, detail, cx) else {
        return Task::ready(finish(
            draft,
            Reading::Refused("nothing to ask about".to_string()),
            mode,
            cx,
        ));
    };
    let key = cx
        .try_global::<SystemOne>()
        .and_then(|layer| layer.key.clone());
    draft.row.state = Some(state.text.clone());
    draft.row.state_hash = Some(state.hash.clone());
    match settings.provider {
        SystemOneProvider::Rules => {
            let reads = asking.verdict.as_ref().map_or_else(
                || nothing(&spec, "the use gave no verdict"),
                |verdict| reading::read(spec.set, verdict),
            );
            Task::ready(finish(draft, Reading::Rules(reads), mode, cx))
        }
        SystemOneProvider::Replay => {
            let answered = cx
                .default_global::<SystemOne>()
                .replay
                .answer(spec.set.id, &state);
            let reading = match answered {
                Ok(Some(answers)) => {
                    draft.answered(&answers);
                    Reading::Model(reading::read(spec.set, &answers))
                }
                Ok(None) => Reading::Model(nothing(&spec, "no replay row")),
                Err(error) => {
                    Reading::Unavailable(format!("the replay row does not read: {error}"))
                }
            };
            Task::ready(finish(draft, reading, mode, cx))
        }
        SystemOneProvider::Typesafe | SystemOneProvider::Compatible => {
            send(spec, &settings, draft, &state, key, mode, cx)
        }
    }
}

/// The state of `asking` at `detail`, each value masked with #516's rules and the user's patterns,
/// whatever agents' redaction says, and with the key's value; `None` when it holds nothing.
fn state_for(asking: &Asking, detail: Detail, cx: &App) -> Option<State> {
    let redactor = mcp::model_redactor(cx);
    let key = cx
        .try_global::<SystemOne>()
        .and_then(|layer| layer.key.clone());
    let mask = |text: &str| {
        let masked = redactor.redact(text).text;
        match &key {
            Some(key) => key.mask(&masked),
            None => masked,
        }
    };
    let facts = asking.facts.iter().fold(
        StateBuilder::new(detail, &mask),
        |builder, (label, value)| builder.fact(label, value),
    );
    asking
        .texts
        .iter()
        .fold(facts, |builder, (label, value)| builder.text(label, value))
        .build()
}

/// A use's mode as the layer applies it: `off` while the layer is off.
pub(crate) fn use_mode(name: &str, cx: &App) -> SystemOneMode {
    cx.try_global::<SystemOne>()
        .filter(|layer| layer.settings.enabled)
        .and_then(|layer| layer.settings.uses.get(name).copied())
        .unwrap_or(SystemOneMode::Off)
}

/// What `asking`'s project may send: its facts and text, its facts alone, or nothing and why. A
/// use that asks different questions by detail reads it before it picks its set (#566).
pub(crate) fn detail(asking: &Asking, cx: &App) -> Result<Detail, Refusal> {
    cx.try_global::<SystemOne>().map_or_else(
        || Err(Refusal::Refused("the layer is off".to_string())),
        |layer| {
            policy::may_send(
                &asking.folders,
                asking.local,
                &layer.settings.projects,
                &layer.settings.metadata_only_projects,
            )
        },
    )
}

/// Logs the verdict `asking` carries, which the use's own rules settled, so that no provider is
/// asked whatever the settings name: every verdict is a row (#565's D6), under the `rules`
/// provider. The row keeps the state the project may send, and none when it may send nothing.
pub(crate) fn record(spec: UseSpec, asking: &Asking, cx: &mut App) -> Asked {
    let mode = use_mode(spec.name, cx);
    let Some(settings) = cx
        .try_global::<SystemOne>()
        .map(|layer| layer.settings.clone())
        .filter(|_| mode != SystemOneMode::Off)
    else {
        return Asked {
            reading: Reading::Off,
            mode: SystemOneMode::Off,
            layer_on: cx
                .try_global::<SystemOne>()
                .is_some_and(|layer| layer.settings.enabled),
            row: None,
            answers: None,
        };
    };
    let mut draft = Draft::new(&spec, &settings, asking, mode);
    draft.row.provider = provider_name(SystemOneProvider::Rules).to_string();
    if let Some(state) = detail(asking, cx)
        .ok()
        .and_then(|detail| state_for(asking, detail, cx))
    {
        draft.row.state = Some(state.text);
        draft.row.state_hash = Some(state.hash);
    }
    let reads = asking.verdict.as_ref().map_or_else(
        || nothing(&spec, "the use gave no verdict"),
        |verdict| reading::read(spec.set, verdict),
    );
    finish(draft, Reading::Rules(reads), mode, cx)
}

/// Logs what came after the call `call`, as its use learned it, such as the user's next prompt:
/// the outcomes a report can fit thresholds on (#566).
pub(crate) fn outcome(call: &str, outcome: String, cx: &mut App) {
    let now = chrono::Local::now();
    let row = OutcomeRow {
        call: call.to_string(),
        time: now.to_rfc3339(),
        outcome,
    };
    let day = now.format("%Y-%m-%d").to_string();
    if let Err(error) = log_sender(cx).unbounded_send((day, Row::Outcome(row))) {
        log::warn!("system one: an outcome came as Marley shut down: {error}");
    }
}

/// Each of `spec`'s questions with no signal, for `reason`.
fn nothing(spec: &UseSpec, reason: &str) -> Vec<Read> {
    spec.set
        .questions
        .iter()
        .map(|question| Read {
            key: question.key().to_string(),
            signal: Signal::Nothing(reason.to_string()),
        })
        .collect()
}

/// The reading of a refusal.
fn refused(refusal: Refusal) -> Reading {
    match refusal {
        Refusal::Refused(reason) => Reading::Refused(reason),
        Refusal::Unavailable(reason) => Reading::Unavailable(reason),
    }
}

/// Sends the request to a `typesafe` or `compatible` provider, past the key's and the gate's
/// checks, and reads what comes back.
fn send(
    spec: UseSpec,
    settings: &SystemOneSettings,
    mut draft: Draft,
    state: &State,
    key: Option<Key>,
    mode: SystemOneMode,
    cx: &mut App,
) -> Task<Asked> {
    let url = match endpoint(settings) {
        Ok(url) => url,
        Err(reason) => return Task::ready(finish(draft, Reading::Refused(reason), mode, cx)),
    };
    let layer = cx.default_global::<SystemOne>();
    let key = match key {
        Some(_) if layer.key_refused => Err("the provider refused the key; set a new one"),
        Some(key) => Ok(key),
        None if layer.key_source == KeySource::Reading => {
            Err("the key is still being read from the keyring")
        }
        None => Err("no key: set MARLEY_SYSTEM_ONE_KEY, or Set Key in Decisions"),
    };
    let key = match key {
        Ok(key) => key,
        Err(reason) => {
            return Task::ready(finish(
                draft,
                Reading::Unavailable(reason.to_string()),
                mode,
                cx,
            ));
        }
    };
    let now = layer.now();
    if let Err(refusal) = layer.gate.admit(
        &draft.subject,
        &state.hash,
        &draft.day,
        now,
        policy::budget(settings.daily_budget_cents),
    ) {
        return Task::ready(finish(draft, refused(refusal), mode, cx));
    }
    let price = match settings.provider {
        SystemOneProvider::Typesafe => TYPESAFE_PRICE,
        _ => settings.price,
    };
    let body = request::build(&settings.model, &state.text, spec.set).to_string();
    let posting = cx.background_spawn(post(
        cx.http_client(),
        cx.background_executor().clone(),
        url.to_string(),
        key.bearer(),
        body,
        spec.deadline,
    ));
    let started = Instant::now();
    let hash = state.hash.clone();
    cx.spawn(async move |cx| {
        let posted = posting.await;
        let latency = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        cx.update(|cx| {
            draft.row.latency_ms = Some(latency);
            let reading = read_posted(posted, &spec, &key, &hash, price, &mut draft, cx);
            finish(draft, reading, mode, cx)
        })
    })
}

/// What came of a request.
enum Posted {
    /// The provider answered with this status and body.
    Answered { status: u16, body: String },
    /// No answer came within the deadline.
    TimedOut,
    /// The provider could not be reached, and why.
    Unreachable(String),
}

/// Posts `body` to `url` with the key's header, trying once more on 429, 503 or 529 while the
/// deadline leaves time. Off the main thread.
async fn post(
    client: Arc<dyn HttpClient>,
    executor: BackgroundExecutor,
    url: String,
    bearer: String,
    body: String,
    deadline: Duration,
) -> Posted {
    let started = Instant::now();
    let mut retried = false;
    loop {
        let remaining = deadline.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Posted::TimedOut;
        }
        let request = match Request::builder()
            .method(Method::POST)
            .uri(url.as_str())
            .header("Authorization", bearer.as_str())
            .header("Content-Type", "application/json")
            .timeout(remaining)
            .body(AsyncBody::from(body.clone()))
        {
            Ok(request) => request,
            Err(error) => return Posted::Unreachable(error.to_string()),
        };
        let sending = async {
            let mut response = client.send(request).await?;
            let status = response.status().as_u16();
            let mut text = String::new();
            response.body_mut().read_to_string(&mut text).await?;
            anyhow::Ok((status, text))
        };
        // The request's own timeout covers its body; this backs it up.
        let backstop = executor.timer(remaining + BACKSTOP);
        match select(pin!(sending), pin!(backstop)).await {
            Either::Left((Ok((status, text)), _)) => {
                let time_left = deadline.saturating_sub(started.elapsed());
                if !retried && RETRIED.contains(&status) && time_left > RETRY_PAUSE * 2 {
                    retried = true;
                    executor.timer(RETRY_PAUSE).await;
                    continue;
                }
                return Posted::Answered { status, body: text };
            }
            Either::Left((Err(error), _)) => {
                return if started.elapsed() >= deadline {
                    Posted::TimedOut
                } else {
                    Posted::Unreachable(format!("{error:#}"))
                };
            }
            Either::Right(_) => return Posted::TimedOut,
        }
    }
}

/// Reads what a request came back with, and tells the gate: an answer spends, a failure counts
/// toward the breaker, and a refused key waits for a new one.
fn read_posted(
    posted: Posted,
    spec: &UseSpec,
    key: &Key,
    hash: &str,
    price: u64,
    draft: &mut Draft,
    cx: &mut App,
) -> Reading {
    let layer = cx.default_global::<SystemOne>();
    let now = layer.now();
    match posted {
        Posted::Answered { status, body } if (200..300).contains(&status) => {
            match request::parse(&body) {
                Ok(answers) => {
                    let cost = policy::cost(answers.input_tokens, price);
                    layer.gate.answered(&draft.subject, hash, cost);
                    draft.answered(&answers);
                    draft.row.cost = Some(cost);
                    Reading::Model(reading::read(spec.set, &answers))
                }
                Err(error) => {
                    layer.gate.failed(now);
                    draft.row.error = Some(key.mask(&request::error_excerpt(&body)));
                    Reading::Unavailable(error.to_string())
                }
            }
        }
        Posted::Answered { status: 401, body } => {
            layer.key_refused = true;
            draft.row.error = Some(key.mask(&request::error_excerpt(&body)));
            Reading::Unavailable("the provider refused the key".to_string())
        }
        Posted::Answered { status, body } => {
            layer.gate.failed(now);
            draft.row.error = Some(key.mask(&request::error_excerpt(&body)));
            Reading::Unavailable(format!("the provider answered {status}"))
        }
        Posted::TimedOut => {
            layer.gate.failed(now);
            Reading::Unavailable(format!("no answer within {}", seconds(spec.deadline)))
        }
        Posted::Unreachable(reason) => {
            layer.gate.failed(now);
            draft.row.error = Some(key.mask(&request::error_excerpt(&reason)));
            Reading::Unavailable("the provider could not be reached".to_string())
        }
    }
}

/// A deadline as the reading says it: `2 s`, or `600 ms` under a second.
fn seconds(duration: Duration) -> String {
    if duration.subsec_millis() == 0 {
        format!("{} s", duration.as_secs())
    } else {
        format!("{} ms", duration.as_millis())
    }
}

/// Finishes the call's row with `reading`, keeps it for the Decisions view and sends it to be
/// written.
fn finish(mut draft: Draft, reading: Reading, mode: SystemOneMode, cx: &mut App) -> Asked {
    draft.row.reading = reading.summary();
    draft.row.failed = reading.failed();
    let row = draft.row;
    let log = log_sender(cx);
    if let Err(error) = log.unbounded_send((draft.day, Row::Call(Box::new(row.clone())))) {
        log::warn!("system one: a row came as Marley shut down: {error}");
    }
    let layer = cx.default_global::<SystemOne>();
    layer.rows.push(row.clone());
    if layer.rows.len() > KEPT_ROWS {
        let excess = layer.rows.len() - KEPT_ROWS;
        layer.rows.drain(..excess).for_each(drop);
    }
    Asked {
        reading,
        mode,
        layer_on: true,
        row: Some(row),
        answers: draft.answers,
    }
}

/// Where rows go to be written: a task, started with the first call, that appends each one to
/// its day's file off the main thread.
fn log_sender(cx: &mut App) -> mpsc::UnboundedSender<(String, Row)> {
    if let Some(log) = cx
        .try_global::<SystemOne>()
        .and_then(|layer| layer.log.clone())
    {
        return log;
    }
    let (log, mut rows) = mpsc::unbounded::<(String, Row)>();
    let dir = folder();
    cx.background_spawn(async move {
        while let Some((day, row)) = rows.next().await {
            files::append_in(&dir, &day, &row).log_err();
        }
    })
    .detach();
    cx.default_global::<SystemOne>().log = Some(log.clone());
    log
}

/// Runs the check on the terminal used last and shows what came back.
fn run_check(workspace: &mut Workspace, window: &Window, cx: &mut Context<Workspace>) {
    let asking = browser::last_terminal(cx)
        .ok_or_else(|| {
            "System One: no terminal to check. Click in one, run a command, then run the check."
                .to_string()
        })
        .and_then(|view| {
            // The check runs inside this workspace's update, so a terminal of this workspace takes
            // its folders from here: reading the workspace entity now would panic.
            let owner = view.read(cx).marley_workspace().clone();
            let (folders, local) = if owner.entity_id() == cx.entity_id() {
                project_of(workspace, cx)
            } else {
                let other = owner
                    .upgrade()
                    .ok_or_else(|| "System One: the terminal's project is closed.".to_string())?;
                project_of(other.read(cx), cx)
            };
            check_asking(&view, folders, local, cx)
        });
    let asking = match asking {
        Ok(asking) => asking,
        Err(message) => {
            show_toast(workspace, message, cx);
            return;
        }
    };
    let asked = ask(CHECK, &asking, cx);
    cx.spawn_in(window, async move |workspace, cx| {
        let asked = asked.await;
        workspace
            .update(cx, |workspace, cx| {
                show_toast(workspace, check_toast(&asked), cx);
            })
            .log_err();
    })
    .detach();
}

/// A project's name in a state: its first folder's, or `a project`.
pub(crate) fn project_name(folders: &[PathBuf]) -> String {
    folders
        .first()
        .and_then(|folder| folder.file_name())
        .map_or_else(
            || "a project".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}

/// A workspace's folders, and whether its project is on this machine.
pub(crate) fn project_of(workspace: &Workspace, cx: &App) -> (Vec<PathBuf>, bool) {
    let folders = workspace
        .root_paths(cx)
        .into_iter()
        .map(|root| root.to_path_buf())
        .collect();
    (folders, workspace.project().read(cx).is_local())
}

/// What the check asks about `view`, a terminal of the project in `folders`: the project, the
/// terminal's title and its last command, with the command's exit code and program.
fn check_asking(
    view: &Entity<TerminalView>,
    folders: Vec<PathBuf>,
    local: bool,
    cx: &App,
) -> Result<Asking, String> {
    let terminal_view = view.read(cx);
    let project = project_name(&folders);
    let title = terminal_view.tab_content_text(0, cx).to_string();
    let terminal = terminal_view.terminal().read(cx);
    let block = terminal
        .blocks()
        .last()
        .ok_or_else(|| "System One: the terminal has run no command to check yet.".to_string())?;
    let exit = block.exit_code.0;
    Ok(Asking {
        subject: view.entity_id().to_string(),
        facts: vec![
            ("project", project.clone()),
            ("block", block.index.to_string()),
            (
                "exit code",
                exit.map_or_else(|| "none".to_string(), |code| code.to_string()),
            ),
            ("program", program(&block.command)),
        ],
        texts: vec![("terminal", title), ("last command", block.command.clone())],
        verdict: exit.map(|code| noul_verdict("command_failed", code != 0)),
        project,
        folders,
        local,
    })
}

/// A use's own yes or no as answers, which the `rules` provider gives back.
pub(crate) fn noul_verdict(key: &str, holds: bool) -> Answers {
    verdict(
        key,
        Answer::Noul {
            noul: if holds { 1.0 } else { 0.0 },
        },
    )
}

/// A use's own yes to each of the nouls `keys` as answers, which a `rules` row keeps (#568).
pub(crate) fn nouls_verdict(keys: &[&str]) -> Answers {
    let by_key: BTreeMap<String, Answer> = keys
        .iter()
        .map(|key| ((*key).to_string(), Answer::Noul { noul: 1.0 }))
        .collect();
    Answers {
        model: None,
        raw: serde_json::to_value(&by_key).unwrap_or(serde_json::Value::Null),
        by_key,
        unreadable: Vec::new(),
        input_tokens: 0,
    }
}

/// A use's own choice of `option` as answers, which a `rules` row keeps (#566).
pub(crate) fn choice_verdict(key: &str, option: &str) -> Answers {
    verdict(
        key,
        Answer::Choice {
            choice: option.to_string(),
            confidence: 1.0,
            probabilities: BTreeMap::from([(option.to_string(), 1.0)]),
        },
    )
}

/// A use's own `answer` to its question `key`, as answers.
fn verdict(key: &str, answer: Answer) -> Answers {
    let by_key = BTreeMap::from([(key.to_string(), answer)]);
    Answers {
        model: None,
        raw: serde_json::to_value(&by_key).unwrap_or(serde_json::Value::Null),
        by_key,
        unreadable: Vec::new(),
        input_tokens: 0,
    }
}

/// The program a command runs: its first word after any `NAME=value` assignments, without its
/// folder, so `GITHUB_TOKEN=… /usr/bin/git push` is `git`.
fn program(command: &str) -> String {
    command
        .split_whitespace()
        .find(|word| !is_assignment(word))
        .map(|word| word.rsplit('/').next().unwrap_or(word).to_string())
        .unwrap_or_default()
}

/// Whether `word` is a shell's `NAME=value` assignment.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty()
            && !name.starts_with(|character: char| character.is_ascii_digit())
            && name
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
    })
}

/// The check's toast: `System One: command failed: yes (0.92) · compatible · 1,200 tokens ·
/// 30 ms`.
fn check_toast(asked: &Asked) -> String {
    let Some(row) = asked.row.as_ref() else {
        return if asked.layer_on {
            "System One's check is off (marley.system_one.uses.check).".to_string()
        } else {
            "System One is off. Turn it on in the System One section of the Marley settings."
                .to_string()
        };
    };
    let failed = asked.reading.failed();
    if asked.mode == SystemOneMode::Shadow && !failed {
        return "System One logged the check in shadow mode. Open Decisions to see it.".to_string();
    }
    let mut parts = vec![asked.reading.summary()];
    if !failed {
        parts.push(row.provider.clone());
    }
    if let Some(tokens) = row.input_tokens.filter(|tokens| *tokens > 0) {
        parts.push(format!("{} tokens", thousands(tokens)));
    }
    if let Some(latency) = row.latency_ms {
        parts.push(format!("{latency} ms"));
    }
    format!("System One: {}", parts.join(" · "))
}

/// Shows `message` in the workspace's toast for the layer.
fn show_toast(workspace: &mut Workspace, message: String, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<SystemOne>(), message),
        cx,
    );
}

/// `number` with commas between each three digits: `1,200`.
pub(crate) fn thousands(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// Billionths of a cent as cents, with up to five decimals: `0.00504¢`.
pub(crate) fn cents(billionths: u64) -> String {
    let whole = billionths / 1_000_000_000;
    let fraction = (billionths % 1_000_000_000) / 10_000;
    if fraction == 0 {
        format!("{whole}¢")
    } else {
        let decimals = format!("{fraction:05}");
        format!("{whole}.{}¢", decimals.trim_end_matches('0'))
    }
}
