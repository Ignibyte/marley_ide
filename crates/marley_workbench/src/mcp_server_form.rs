//! Adding, changing and removing Claude Code's and Codex's MCP servers from MCP Servers (#768).
//!
//! A write goes through `safe_write::replace_in` with a backup and changes one entry: Codex's
//! `config.toml` through `toml_edit`, so its comments and other tables stay; `.claude.json` and a
//! folder's `.mcp.json` as JSON, keeping their key order, two-space indentation and final
//! newline. Only the entry is compared before the write: an entry that changed since the tab read
//! it, or a name added that is there now, refuses it. Claude Code rewrites `.claude.json` often
//! while it runs, read-modify-write, so the rest of the file moving is no conflict, and an entry
//! written between two of its saves is kept.
//!
//! Values are write-only. The form names an existing env variable or header and keeps its value
//! unless a new one is typed; it never reads one. Stored arguments and a URL show as the tab shows
//! them, keys and tokens hidden, and a field left as shown keeps what is stored.

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use editor::Editor;
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    PromptLevel, SharedString, Task, WeakEntity, Window,
};
use serde_json::{Map, Value};
use toml_edit::{Item, TableLike};
use ui::{ButtonStyle, Headline, HeadlineSize, prelude::*};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

use crate::mcp_servers::McpServers;

/// Where a server lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Place {
    /// `mcpServers` in `.claude.json`.
    ClaudeUser,
    /// That folder's entry in `.claude.json`'s `projects`.
    ClaudeLocal(PathBuf),
    /// That folder's `.mcp.json`.
    ClaudeProject(PathBuf),
    /// `[mcp_servers]` in Codex's `config.toml`.
    Codex,
}

impl Place {
    /// The file the server is written in.
    pub(crate) fn file(&self) -> PathBuf {
        match self {
            Self::ClaudeUser | Self::ClaudeLocal(_) => crate::mcp_servers::claude_json(),
            Self::ClaudeProject(folder) => folder.join(".mcp.json"),
            Self::Codex => crate::agent_notify::codex_config(),
        }
    }

    /// The scope in words, as the tab's rows name it.
    pub(crate) fn words(&self) -> String {
        match self {
            Self::ClaudeUser | Self::Codex => "user".to_string(),
            Self::ClaudeLocal(folder) => format!("local, {}", folder_name(folder)),
            Self::ClaudeProject(folder) => format!("project, {}", folder_name(folder)),
        }
    }

    const fn cli(&self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeUser | Self::ClaudeLocal(_) | Self::ClaudeProject(_) => "Claude Code",
        }
    }
}

/// A folder's last part, as the tab names a project.
pub(crate) fn folder_name(folder: &Path) -> String {
    folder.file_name().map_or_else(
        || folder.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// How a server runs, as stored or as the form saves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    Command { command: String, args: Vec<String> },
    Url { url: String },
}

/// What an edit of a stored server starts from. It holds the command, its arguments and the URL
/// as stored, and never an env or header value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Stored {
    pub(crate) place: Place,
    pub(crate) kind: Kind,
    /// The names of its env variables (a command) or headers (a URL) that the form edits.
    pub(crate) pairs: Vec<String>,
    /// The entry as the tab read it, hashed ([`entry_hash`]).
    pub(crate) seen: u64,
}

/// A pair's value: the stored one kept, or a new one.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pair {
    Keep,
    Set(String),
}

/// A server as the form saves it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Server {
    kind: Kind,
    pairs: Vec<(String, Pair)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Change {
    Put(Server),
    Remove,
}

/// What an entry is compared by: a hash of its JSON or TOML text as parsed.
pub(crate) fn entry_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// `text`, the file of `place`, with `change` made to the server `name`. `seen` is the entry's
/// hash as the tab read it, `None` for a server added.
fn change_text(
    text: &str,
    place: &Place,
    name: &str,
    change: &Change,
    seen: Option<u64>,
) -> anyhow::Result<String> {
    match place {
        Place::Codex => codex_change(text, name, change, seen),
        Place::ClaudeUser | Place::ClaudeLocal(_) | Place::ClaudeProject(_) => {
            claude_change(text, place, name, change, seen)
        }
    }
}

/// Refuses a change to an entry that is not as the tab read it.
fn check(found: Option<u64>, seen: Option<u64>, name: &str) -> anyhow::Result<()> {
    match (found, seen) {
        (None, None) => Ok(()),
        (Some(found), Some(seen)) if found == seen => Ok(()),
        (Some(_), None) => bail!("{name} is there already; Refresh to see it"),
        (_, Some(_)) => bail!("{name} changed since the tab read it; Refresh and try again"),
    }
}

fn claude_change(
    text: &str,
    place: &Place,
    name: &str,
    change: &Change,
    seen: Option<u64>,
) -> anyhow::Result<String> {
    let mut root: Value = if text.trim().is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str(text).context("reading the file as JSON")?
    };
    let servers = claude_servers(&mut root, place)?;
    check(
        servers
            .get(name)
            .map(|entry| entry_hash(&entry.to_string())),
        seen,
        name,
    )?;
    match change {
        Change::Remove => {
            servers.shift_remove(name);
        }
        Change::Put(server) => {
            let mut entry = match servers.get(name) {
                Some(Value::Object(entry)) => entry.clone(),
                _ => Map::new(),
            };
            put_claude(&mut entry, server)?;
            servers.insert(name.to_string(), Value::Object(entry));
        }
    }
    let mut written = serde_json::to_string_pretty(&root).context("writing the JSON")?;
    if text.ends_with('\n') {
        written.push('\n');
    }
    Ok(written)
}

/// The `mcpServers` object `place` names in `root`, made when missing. A folder Claude Code has
/// not run in has no entry to hold a local server: Claude Code fills a new entry's other keys
/// itself, and Marley does not write half of one.
fn claude_servers<'a>(
    root: &'a mut Value,
    place: &Place,
) -> anyhow::Result<&'a mut Map<String, Value>> {
    let root = root
        .as_object_mut()
        .context("the file is not a JSON object")?;
    let holder = match place {
        Place::ClaudeLocal(folder) => {
            let key = folder.to_string_lossy();
            root.get_mut("projects")
                .and_then(Value::as_object_mut)
                .and_then(|projects| projects.get_mut(key.as_ref()))
                .and_then(Value::as_object_mut)
                .with_context(|| {
                    format!(
                        "Claude Code has not run in {} yet, so it has no local scope there",
                        folder.display()
                    )
                })?
        }
        Place::ClaudeUser | Place::ClaudeProject(_) | Place::Codex => root,
    };
    holder
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .context("mcpServers is not an object")
}

/// Writes `server` into a Claude Code entry, keeping the keys the form does not know.
fn put_claude(entry: &mut Map<String, Value>, server: &Server) -> anyhow::Result<()> {
    let pairs_key = match &server.kind {
        Kind::Command { command, args } => {
            entry.insert("type".to_string(), Value::from("stdio"));
            entry.insert("command".to_string(), Value::from(command.clone()));
            if args.is_empty() {
                entry.shift_remove("args");
            } else {
                entry.insert("args".to_string(), Value::from(args.clone()));
            }
            entry.shift_remove("url");
            entry.shift_remove("headers");
            "env"
        }
        Kind::Url { url } => {
            let transport = match entry.get("type").and_then(Value::as_str) {
                Some(transport @ ("http" | "sse")) => transport.to_string(),
                _ => "http".to_string(),
            };
            entry.insert("type".to_string(), Value::from(transport));
            entry.insert("url".to_string(), Value::from(url.clone()));
            entry.shift_remove("command");
            entry.shift_remove("args");
            entry.shift_remove("env");
            "headers"
        }
    };
    let pairs = server
        .pairs
        .iter()
        .map(|(name, pair)| {
            let value = match pair {
                Pair::Set(value) => Value::from(value.clone()),
                Pair::Keep => entry
                    .get(pairs_key)
                    .and_then(|stored| stored.get(name))
                    .cloned()
                    .with_context(|| format!("{name} has no stored value to keep"))?,
            };
            Ok((name.clone(), value))
        })
        .collect::<anyhow::Result<Map<String, Value>>>()?;
    if pairs.is_empty() {
        entry.shift_remove(pairs_key);
    } else {
        entry.insert(pairs_key.to_string(), Value::Object(pairs));
    }
    Ok(())
}

fn codex_change(
    text: &str,
    name: &str,
    change: &Change,
    seen: Option<u64>,
) -> anyhow::Result<String> {
    let mut document: toml_edit::DocumentMut = text.parse().context("reading the file as TOML")?;
    let found = document
        .get("mcp_servers")
        .and_then(|servers| servers.get(name))
        .map(|entry| entry_hash(&entry.to_string()));
    check(found, seen, name)?;
    let servers = document
        .entry("mcp_servers")
        .or_insert_with(|| {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            Item::Table(table)
        })
        .as_table_like_mut()
        .context("mcp_servers is not a table")?;
    match change {
        Change::Remove => {
            servers.remove(name);
        }
        Change::Put(server) => {
            if !servers.contains_key(name) {
                servers.insert(name, Item::Table(toml_edit::Table::new()));
            }
            let entry = servers
                .get_mut(name)
                .and_then(Item::as_table_like_mut)
                .with_context(|| format!("{name} is not a table"))?;
            put_codex(entry, server)?;
        }
    }
    Ok(document.to_string())
}

/// Writes `server` into a Codex table, keeping `enabled`, `env_vars`, `bearer_token_env_var` and
/// every other key the form does not know.
fn put_codex(entry: &mut dyn TableLike, server: &Server) -> anyhow::Result<()> {
    let pairs_key = match &server.kind {
        Kind::Command { command, args } => {
            entry.insert("command", toml_edit::value(command.as_str()));
            if args.is_empty() {
                entry.remove("args");
            } else {
                let args: toml_edit::Array = args.iter().map(String::as_str).collect();
                entry.insert("args", toml_edit::value(args));
            }
            entry.remove("url");
            entry.remove("http_headers");
            "env"
        }
        Kind::Url { url } => {
            entry.insert("url", toml_edit::value(url.as_str()));
            entry.remove("command");
            entry.remove("args");
            entry.remove("env");
            "http_headers"
        }
    };
    if server.pairs.is_empty() {
        entry.remove(pairs_key);
        return Ok(());
    }
    // A stored table, inline or not, is edited in place so its style and order stay.
    if let Some(stored) = entry.get_mut(pairs_key).and_then(Item::as_table_like_mut) {
        let names: HashSet<&str> = server.pairs.iter().map(|(name, _)| name.as_str()).collect();
        let stale: Vec<String> = stored
            .iter()
            .map(|(name, _)| name.to_string())
            .filter(|name| !names.contains(name.as_str()))
            .collect();
        for name in stale {
            stored.remove(&name);
        }
        for (name, pair) in &server.pairs {
            match pair {
                Pair::Set(value) => {
                    stored.insert(name, toml_edit::value(value.as_str()));
                }
                Pair::Keep if stored.contains_key(name) => {}
                Pair::Keep => bail!("{name} has no stored value to keep"),
            }
        }
        return Ok(());
    }
    let mut table = toml_edit::InlineTable::new();
    for (name, pair) in &server.pairs {
        match pair {
            Pair::Set(value) => {
                table.insert(name, value.as_str().into());
            }
            Pair::Keep => bail!("{name} has no stored value to keep"),
        }
    }
    entry.insert(pairs_key, toml_edit::value(table));
    Ok(())
}

/// After a write: the tab reads the files again, and a toast says when the CLI sees it.
fn saved(
    page: &WeakEntity<McpServers>,
    workspace: &WeakEntity<Workspace>,
    message: String,
    cx: &mut gpui::AsyncWindowContext,
) {
    page.update_in(cx, |page, window, cx| page.refresh(window, cx))
        .log_err();
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<McpServerForm>(), message),
                cx,
            );
        })
        .log_err();
}

/// Asks, then removes the server `name` from where `stored` says it lives.
pub(crate) fn remove(
    page: WeakEntity<McpServers>,
    workspace: WeakEntity<Workspace>,
    name: String,
    stored: Stored,
    window: &mut Window,
    cx: &mut App,
) {
    let question = format!(
        "Remove {name} from {}'s servers ({})?",
        stored.place.cli(),
        stored.place.words()
    );
    let file = stored.place.file();
    let file_name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let detail = format!(
        "A copy of {file_name} as it is now stays beside it, as {file_name}.marley-backup."
    );
    let answer = window.prompt(
        PromptLevel::Warning,
        &question,
        Some(&detail),
        &["Remove", "Cancel"],
        cx,
    );
    window
        .spawn(cx, async move |cx| {
            if !matches!(answer.await, Ok(0)) {
                return;
            }
            let file = stored.place.file();
            let written = cx
                .background_spawn(futures::future::lazy({
                    let name = name.clone();
                    let place = stored.place.clone();
                    move |_| {
                        crate::safe_write::replace_in(&file, true, |text| {
                            change_text(text, &place, &name, &Change::Remove, Some(stored.seen))
                        })
                    }
                }))
                .await;
            match written {
                Ok(_) => saved(
                    &page,
                    &workspace,
                    format!(
                        "{name} removed. {} reads its servers at its next start.",
                        stored.place.cli()
                    ),
                    cx,
                ),
                Err(error) => {
                    workspace
                        .update(cx, |workspace, cx| workspace.show_error(error, cx))
                        .log_err();
                }
            }
        })
        .detach();
}

/// Whether the form runs a command or names a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormKind {
    Command,
    Url,
}

/// A pair's name: one stored, whose value the form never reads, or a new one being typed.
enum PairName {
    Stored(String),
    New(Entity<Editor>),
}

/// One env variable or header in the form.
struct PairField {
    name: PairName,
    value: Entity<Editor>,
}

/// The form over MCP Servers: a new server for a CLI, or one stored server changed.
pub(crate) struct McpServerForm {
    page: WeakEntity<McpServers>,
    workspace: WeakEntity<Workspace>,
    /// The server edited, by name, and what the tab read of it; `None` adds one.
    editing: Option<(String, Stored)>,
    /// Where an added server can go; one place when editing.
    places: Vec<Place>,
    place: Place,
    kind: FormKind,
    name: Entity<Editor>,
    command: Entity<Editor>,
    arguments: Entity<Editor>,
    url: Entity<Editor>,
    /// The stored arguments or URL as the fields first showed them, when that hid something: a
    /// field left so keeps what is stored.
    shown_arguments: Option<String>,
    shown_url: Option<String>,
    env: Vec<PairField>,
    headers: Vec<PairField>,
    status: Option<SharedString>,
    saving: Option<Task<()>>,
}

impl McpServerForm {
    /// A form that adds a server to one of `places`, or, with `editing`, changes a stored one.
    pub(crate) fn new(
        page: WeakEntity<McpServers>,
        workspace: WeakEntity<Workspace>,
        editing: Option<(String, Stored)>,
        places: Vec<Place>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = |placeholder: &str, text: &str, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_placeholder_text(placeholder, window, cx);
                if !text.is_empty() {
                    editor.set_text(text, window, cx);
                }
                editor
            })
        };
        let stored = editing.as_ref().map(|(_, stored)| stored);
        let (kind, command, arguments, url) = match stored.map(|stored| &stored.kind) {
            Some(Kind::Url { url }) => (FormKind::Url, String::new(), String::new(), url.clone()),
            Some(Kind::Command { command, args }) => (
                FormKind::Command,
                command.clone(),
                shlex::try_join(args.iter().map(String::as_str)).unwrap_or_else(|_| args.join(" ")),
                String::new(),
            ),
            None => (
                FormKind::Command,
                String::new(),
                String::new(),
                String::new(),
            ),
        };
        let masked = marley_mcp::redact::Redactor::new(&[]).0.redact(&arguments);
        let shown_arguments = (masked.count > 0).then_some(masked.text);
        let redacted_url = marley_browser::observe::redact_url(&url);
        let shown_url = (redacted_url != url).then_some(redacted_url);
        let stored_pairs: Vec<PairField> = stored
            .map(|stored| stored.pairs.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|name| PairField {
                name: PairName::Stored(name),
                value: masked_field("set; type to replace", window, cx),
            })
            .collect();
        let (env, headers) = match kind {
            FormKind::Command => (stored_pairs, Vec::new()),
            FormKind::Url => (Vec::new(), stored_pairs),
        };
        let place = stored
            .map(|stored| stored.place.clone())
            .or_else(|| places.first().cloned())
            .unwrap_or(Place::ClaudeUser);
        Self {
            page,
            workspace,
            name: field("A name: letters, digits, - and _", "", window, cx),
            command: field("The program, such as npx or a path", &command, window, cx),
            arguments: field(
                "Its arguments, quoted as in a shell",
                shown_arguments.as_deref().unwrap_or(&arguments),
                window,
                cx,
            ),
            url: field(
                "https://…",
                shown_url.as_deref().unwrap_or(&url),
                window,
                cx,
            ),
            shown_arguments,
            shown_url,
            editing,
            places,
            place,
            kind,
            env,
            headers,
            status: None,
            saving: None,
        }
    }

    fn stored_kind(&self) -> Option<&Kind> {
        self.editing.as_ref().map(|(_, stored)| &stored.kind)
    }

    const fn pairs(&self) -> &Vec<PairField> {
        match self.kind {
            FormKind::Command => &self.env,
            FormKind::Url => &self.headers,
        }
    }

    const fn pairs_mut(&mut self) -> &mut Vec<PairField> {
        match self.kind {
            FormKind::Command => &mut self.env,
            FormKind::Url => &mut self.headers,
        }
    }

    /// The form as a change, or what is wrong with it.
    fn collect(&self, cx: &App) -> Result<(String, Server), String> {
        let name = if let Some((name, _)) = &self.editing {
            name.clone()
        } else {
            let name = self.name.read(cx).text(cx).trim().to_string();
            if name.is_empty()
                || name.len() > 64
                || !name
                    .chars()
                    .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_'))
            {
                return Err("A name is letters, digits, - and _, up to 64 of them".to_string());
            }
            name
        };
        let kind = match self.kind {
            FormKind::Command => {
                let command = self.command.read(cx).text(cx).trim().to_string();
                if command.is_empty() {
                    return Err("The server needs a program to run".to_string());
                }
                let typed = self.arguments.read(cx).text(cx);
                let args = match (self.shown_arguments.as_deref(), self.stored_kind()) {
                    (Some(shown), Some(Kind::Command { args, .. })) if typed == shown => {
                        args.clone()
                    }
                    (Some(_), _) if typed.contains("[redacted: ") => {
                        return Err(
                            "An argument is hidden: leave the arguments as they were, or \
                                    type them all again"
                                .to_string(),
                        );
                    }
                    _ => shlex::split(&typed).ok_or_else(|| {
                        "The arguments have a quote that is not closed".to_string()
                    })?,
                };
                Kind::Command { command, args }
            }
            FormKind::Url => {
                let typed = self.url.read(cx).text(cx).trim().to_string();
                let url = match (self.shown_url.as_deref(), self.stored_kind()) {
                    (Some(shown), Some(Kind::Url { url })) if typed == shown => url.clone(),
                    (Some(_), _) if typed.contains('…') => {
                        return Err(
                            "The URL has a hidden value: leave it as it was, or type it \
                                    whole again"
                                .to_string(),
                        );
                    }
                    _ => typed,
                };
                match url::Url::parse(&url) {
                    Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => {}
                    _ => return Err("The URL needs to start with http:// or https://".to_string()),
                }
                Kind::Url { url }
            }
        };
        let what = match self.kind {
            FormKind::Command => "variable",
            FormKind::Url => "header",
        };
        let mut seen = HashSet::new();
        let mut pairs = Vec::new();
        for field in self.pairs() {
            let value = field.value.read(cx).text(cx);
            let (name, pair) = match &field.name {
                PairName::Stored(name) if value.is_empty() => (name.clone(), Pair::Keep),
                PairName::Stored(name) => (name.clone(), Pair::Set(value)),
                PairName::New(editor) => {
                    let name = editor.read(cx).text(cx).trim().to_string();
                    let well_formed = !name.is_empty()
                        && name.chars().all(|letter| {
                            letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_' | '.')
                        });
                    if !well_formed {
                        return Err(format!("A {what}'s name is letters, digits, -, _ and ."));
                    }
                    if value.is_empty() {
                        return Err(format!("{name} needs a value"));
                    }
                    (name, Pair::Set(value))
                }
            };
            if !seen.insert(name.clone()) {
                return Err(format!("{name} is named twice"));
            }
            pairs.push((name, pair));
        }
        Ok((name, Server { kind, pairs }))
    }

    /// Writes the form off the main thread; success refreshes the tab and closes the form, a
    /// refusal stays in it.
    fn save(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.saving.is_some() {
            return;
        }
        let (name, server) = match self.collect(cx) {
            Ok(change) => change,
            Err(why) => {
                self.status = Some(why.into());
                cx.notify();
                return;
            }
        };
        let place = self.place.clone();
        let seen = self.editing.as_ref().map(|(_, stored)| stored.seen);
        self.status = Some(SharedString::new_static("Saving…"));
        cx.notify();
        let page = self.page.clone();
        let workspace = self.workspace.clone();
        self.saving = Some(cx.spawn_in(window, async move |form, cx| {
            let file = place.file();
            let written = cx
                .background_spawn(futures::future::lazy({
                    let name = name.clone();
                    let place = place.clone();
                    move |_| {
                        crate::safe_write::replace_in(&file, true, |text| {
                            change_text(text, &place, &name, &Change::Put(server), seen)
                        })
                    }
                }))
                .await;
            match written {
                Ok(_) => {
                    saved(
                        &page,
                        &workspace,
                        format!(
                            "{name} saved. {} reads its servers at its next start.",
                            place.cli()
                        ),
                        cx,
                    );
                    form.update(cx, |_, cx| cx.emit(DismissEvent)).log_err();
                }
                Err(error) => {
                    form.update(cx, |form, cx| {
                        form.saving = None;
                        form.status = Some(format!("{error:#}").into());
                        cx.notify();
                    })
                    .log_err();
                }
            }
        }));
    }

    fn render_field(label: &'static str, editor: &Entity<Editor>, cx: &App) -> AnyElement {
        v_flex()
            .gap_0p5()
            .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
            .child(boxed(editor, cx))
            .into_any_element()
    }

    fn render_places(&self, cx: &Context<Self>) -> AnyElement {
        v_flex()
            .gap_0p5()
            .child(
                Label::new("Where")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(self.places.iter().enumerate().map(|(index, place)| {
                        let chosen = place.clone();
                        Button::new(("mcp-server-place", index), place.words())
                            .style(ButtonStyle::Filled)
                            .toggle_state(self.place == *place)
                            .on_click(cx.listener(move |form, _, _, cx| {
                                form.place = chosen.clone();
                                cx.notify();
                            }))
                    })),
            )
            .into_any_element()
    }

    fn render_kinds(&self, cx: &Context<Self>) -> AnyElement {
        let kind_button = |id: &'static str, label: &'static str, kind: FormKind| {
            Button::new(id, label)
                .style(ButtonStyle::Filled)
                .toggle_state(self.kind == kind)
                .on_click(cx.listener(move |form, _, _, cx| {
                    form.kind = kind;
                    cx.notify();
                }))
        };
        h_flex()
            .gap_2()
            .child(kind_button(
                "mcp-server-command",
                "Command",
                FormKind::Command,
            ))
            .child(kind_button("mcp-server-url", "URL", FormKind::Url))
            .into_any_element()
    }

    fn render_pairs(&self, cx: &Context<Self>) -> AnyElement {
        let (label, add) = match self.kind {
            FormKind::Command => ("Env variables", "Add Variable"),
            FormKind::Url => ("Headers", "Add Header"),
        };
        v_flex()
            .gap_1()
            .child(Label::new(label).size(LabelSize::Small).color(Color::Muted))
            .children(self.pairs().iter().enumerate().map(|(index, field)| {
                let name = match &field.name {
                    PairName::Stored(name) => div()
                        .w(rems(12.))
                        .child(Label::new(name.clone()).size(LabelSize::Small))
                        .into_any_element(),
                    PairName::New(editor) => div()
                        .w(rems(12.))
                        .child(boxed(editor, cx))
                        .into_any_element(),
                };
                h_flex()
                    .gap_2()
                    .child(name)
                    .child(div().flex_1().child(boxed(&field.value, cx)))
                    .child(
                        Button::new(("mcp-server-clear", index), "Clear")
                            .style(ButtonStyle::Subtle)
                            .on_click(cx.listener(move |form, _, _, cx| {
                                let pairs = form.pairs_mut();
                                if index < pairs.len() {
                                    pairs.remove(index);
                                }
                                cx.notify();
                            })),
                    )
            }))
            .child(
                h_flex().child(
                    Button::new("mcp-server-add-pair", add)
                        .style(ButtonStyle::Subtle)
                        .on_click(cx.listener(|form, _, window, cx| {
                            let name = cx.new(|cx| {
                                let mut editor = Editor::single_line(window, cx);
                                editor.set_placeholder_text("NAME", window, cx);
                                editor
                            });
                            let value = masked_field("value", window, cx);
                            window.focus(&name.focus_handle(cx), cx);
                            form.pairs_mut().push(PairField {
                                name: PairName::New(name),
                                value,
                            });
                            cx.notify();
                        })),
                ),
            )
            .into_any_element()
    }
}

/// A single-line editor whose text shows as dots: a value is typed, never shown.
fn masked_field(placeholder: &str, window: &mut Window, cx: &mut App) -> Entity<Editor> {
    cx.new(|cx| {
        let mut editor = Editor::single_line(window, cx);
        editor.set_placeholder_text(placeholder, window, cx);
        editor.set_masked(true, cx);
        editor
    })
}

/// An editor in a bordered box, as the forms draw a field.
fn boxed(editor: &Entity<Editor>, cx: &App) -> AnyElement {
    let colors = cx.theme().colors();
    div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .border_1()
        .border_color(colors.border)
        .bg(colors.editor_background)
        .child(editor.clone())
        .into_any_element()
}

impl ModalView for McpServerForm {}

impl EventEmitter<DismissEvent> for McpServerForm {}

impl Focusable for McpServerForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match (&self.editing, self.kind) {
            (None, _) => self.name.focus_handle(cx),
            (Some(_), FormKind::Command) => self.command.focus_handle(cx),
            (Some(_), FormKind::Url) => self.url.focus_handle(cx),
        }
    }
}

impl Render for McpServerForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = match &self.editing {
            Some((name, stored)) => format!(
                "Edit {name} ({}, {})",
                stored.place.cli(),
                stored.place.words()
            ),
            None => format!("Add a Server to {}", self.place.cli()),
        };
        v_flex()
            .key_context("McpServerForm")
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|form, _: &menu::Confirm, window, cx| form.save(window, cx)))
            .w(rems(40.))
            .elevation_3(cx)
            .p_3()
            .gap_3()
            .child(Headline::new(title).size(HeadlineSize::Small))
            .when(self.editing.is_none(), |form| {
                form.child(Self::render_field("Name", &self.name, cx))
                    .when(self.places.len() > 1, |form| {
                        form.child(self.render_places(cx))
                    })
            })
            .child(self.render_kinds(cx))
            .map(|form| match self.kind {
                FormKind::Command => form
                    .child(Self::render_field("Command", &self.command, cx))
                    .child(Self::render_field("Arguments", &self.arguments, cx)),
                FormKind::Url => form.child(Self::render_field("URL", &self.url, cx)),
            })
            .child(self.render_pairs(cx))
            .when_some(self.status.clone(), |form, status| {
                form.child(
                    Label::new(status)
                        .size(LabelSize::Small)
                        .color(Color::Warning),
                )
            })
            .child(
                h_flex().justify_end().child(
                    Button::new("mcp-server-save", "Save")
                        .style(ButtonStyle::Filled)
                        .disabled(self.saving.is_some())
                        .on_click(cx.listener(|form, _, window, cx| form.save(window, cx))),
                ),
            )
    }
}
