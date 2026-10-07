//! The settings and actions tools of Marley's MCP server (#681).
//!
//! `settings_schema`, `settings_read` and `actions_list` let an agent say what a setting does,
//! what it is set to and in which file, and which key runs a command.
//!
//! The schema is Zed's own (`SettingsStore::json_schema`), built off the main thread with the
//! actions' names and documentation; it carries no defaults, so those come from the default
//! settings. "Which file wins" walks `SettingsStore::get_all_files`, highest precedence first,
//! reading each file's content at the key path, rather than the Settings UI's typed pickers, which
//! take a field and not a path.

use collections::HashMap;
use gpui::App;
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Map, Value, json};
use settings::{SettingsFile, SettingsJsonSchemaParams, SettingsStore};
use workspace::MultiWorkspace;

/// The most actions `actions_list` answers.
const MAX_ACTIONS: usize = 50;

/// The most keys a listing of an object's keys, or a refusal's next steps, names.
const MAX_KEYS: usize = 60;

/// The most bytes a `settings_read` answer holds, a page's (#680).
const MAX_ANSWER_BYTES: usize = 12_000;

/// Words a query is made of that say nothing about what it looks for.
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "can", "do", "does", "for", "how", "i", "in", "is", "it", "me", "my",
    "of", "on", "or", "the", "to", "what", "when", "where", "which", "with",
];

/// What a hidden setting's value becomes.
const HIDDEN: &str = "[redacted: setting]";

/// Answers `call`, one of `settings_schema`, `settings_read` and `actions_list`. The schema is
/// built off the main thread; the other two read the app's state here.
pub(crate) fn answer(call: AppCall, cx: &App) {
    let tool = call.tool.clone();
    match tool.as_str() {
        "settings_schema" => schema(call, cx),
        "settings_read" => {
            let result = read(&call.arguments, cx);
            call.answer(result);
        }
        "actions_list" => {
            let result = actions_list(&call.arguments, cx);
            call.answer(result);
        }
        other => call.answer(Err(Refusal::from(format!(
            "Marley answers no tool named {other}"
        )))),
    }
}

/// The words of a query that say what it looks for: lower case, two characters or more, the stop
/// words out.
pub(crate) fn query_words(query: &str) -> Vec<String> {
    let mut words: Vec<String> = query
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .map(str::to_lowercase)
        .filter(|word| word.chars().count() >= 2 && !STOP_WORDS.contains(&word.as_str()))
        .collect();
    words.dedup();
    words
}

/// An action's name as the command palette shows it: the namespace with its underscores as
/// spaces, a colon, then the name's words in lower case (`editor::GoToDefinition` is
/// `editor: go to definition`).
pub(crate) fn palette_name(action: &str) -> String {
    let (namespace, name) = action.rsplit_once("::").unwrap_or(("", action));
    let mut words = String::new();
    let mut previous: Option<char> = None;
    for character in name.chars() {
        let starts_word = character.is_uppercase()
            && previous.is_some_and(|previous| previous.is_lowercase() || previous.is_numeric());
        if starts_word {
            words.push(' ');
        }
        words.extend(character.to_lowercase());
        previous = Some(character);
    }
    if namespace.is_empty() {
        words
    } else {
        format!(
            "{}: {words}",
            namespace.replace("::", " ").replace('_', " ")
        )
    }
}

/// One key binding of an action, as text.
#[derive(Debug, Clone)]
pub(crate) struct BoundKey {
    pub(crate) keystrokes: String,
    pub(crate) context: Option<String>,
}

/// Every action's key bindings, highest precedence first, read from the keymap.
pub(crate) fn bound_keys(cx: &App) -> HashMap<&'static str, Vec<BoundKey>> {
    let keymap = cx.key_bindings();
    let keymap = keymap.borrow();
    let mut keys: HashMap<&'static str, Vec<BoundKey>> = HashMap::default();
    // The keymap lists its bindings lowest precedence first.
    for binding in keymap.bindings().rev() {
        let name = binding.action().name();
        if name == "zed::NoAction" || name == "zed::Unbind" {
            continue;
        }
        keys.entry(name).or_default().push(BoundKey {
            keystrokes: ui::text_for_keybinding_keystrokes(binding.keystrokes(), cx),
            context: binding.predicate().map(|predicate| predicate.to_string()),
        });
    }
    keys
}

/// `actions_list`: the actions whose name, palette name or documentation holds every word of the
/// query, those whose name holds them first.
fn actions_list(arguments: &Value, cx: &App) -> Result<ToolAnswer, Refusal> {
    let query = arguments
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let words = query_words(query);
    if words.is_empty() {
        return Err(
            Refusal::new("bad_argument", "`query` holds no word to look for").next(
                "give `query`, words the action's name or documentation holds, such as \"save\"",
            ),
        );
    }
    let documentation = cx.action_documentation();
    let keys = bound_keys(cx);
    let mut found: Vec<(bool, &'static str)> = cx
        .all_action_names()
        .iter()
        .filter_map(|&name| {
            let palette = palette_name(name);
            let named = format!("{} {palette}", name.to_lowercase());
            let held = format!(
                "{named} {}",
                documentation
                    .get(name)
                    .map(|text| text.to_lowercase())
                    .unwrap_or_default()
            );
            words
                .iter()
                .all(|word| held.contains(word.as_str()))
                .then(|| (words.iter().all(|word| named.contains(word.as_str())), name))
        })
        .collect();
    // Those whose name holds every word first, the shortest first, so `save` finds
    // `workspace::Save` before `editor::SaveLocation`.
    found.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then(left.1.len().cmp(&right.1.len()))
            .then(left.1.cmp(right.1))
    });
    let matched = found.len();
    let listed: Vec<Value> = found
        .into_iter()
        .take(MAX_ACTIONS)
        .map(|(_, name)| {
            let bindings: Vec<Value> = keys
                .get(name)
                .map(|bound| {
                    bound
                        .iter()
                        .map(|key| json!({ "keystrokes": key.keystrokes, "context": key.context }))
                        .collect()
                })
                .unwrap_or_default();
            json!({
                "name": name,
                "palette": palette_name(name),
                "documentation": documentation
                    .get(name)
                    .and_then(|text| text.lines().find(|line| !line.trim().is_empty()))
                    .map(str::trim),
                "keys": bindings,
            })
        })
        .collect();
    Ok(ToolAnswer {
        structured: json!({ "matched": matched, "actions": listed }),
        text: None,
        image: None,
    })
}

/// `settings_schema`: what the setting at the key path is. The schema is built off the main
/// thread, from the actions' names and documentation read here.
fn schema(call: AppCall, cx: &App) {
    let key = key_argument(&call.arguments);
    let action_names: Vec<&'static str> = cx.all_action_names().to_vec();
    let documentation = cx.action_documentation().clone();
    let deprecations = cx.deprecated_actions_to_preferred_actions().clone();
    let deprecation_messages = cx.action_deprecation_messages().clone();
    let defaults = serde_json::to_value(cx.global::<SettingsStore>().raw_default_settings())
        .unwrap_or_default();
    cx.background_executor()
        .spawn(futures::future::lazy(move |_| {
            let schema = SettingsStore::json_schema(&SettingsJsonSchemaParams {
                language_names: &[],
                font_names: &[],
                theme_names: &[],
                icon_theme_names: &[],
                lsp_adapter_names: &[],
                action_names: &action_names,
                action_documentation: &documentation,
                deprecations: &deprecations,
                deprecation_messages: &deprecation_messages,
            });
            call.answer(describe(&schema, &defaults, &key));
        }))
        .detach();
}

/// The key path a call names, its surrounding space and dots trimmed.
fn key_argument(arguments: &Value) -> String {
    arguments
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .trim_matches('.')
        .to_string()
}

/// The setting at `key` in `schema`, with its default from `defaults`, or the top-level keys
/// for an empty key.
fn describe(schema: &Value, defaults: &Value, key: &str) -> Result<ToolAnswer, Refusal> {
    let mut node = schema;
    let mut description = None;
    let mut walked: Vec<&str> = Vec::new();
    for segment in key.split('.').filter(|segment| !segment.is_empty()) {
        let object = resolve(schema, node);
        let next = object
            .get("properties")
            .and_then(|properties| properties.get(segment))
            .or_else(|| {
                object
                    .get("additionalProperties")
                    .filter(|value| value.is_object())
            });
        let Some(next) = next else {
            let place = if walked.is_empty() {
                "the top level".to_string()
            } else {
                format!("`{}`", walked.join("."))
            };
            let keys = property_names(schema, object);
            let refusal = Refusal::new("no_setting", format!("{place} has no key `{segment}`"));
            let refusal = if keys.is_empty() {
                refusal.next(format!("{place} takes no named keys"))
            } else {
                refusal.next(format!("the keys of {place}: {}", keys.join(", ")))
            };
            return Err(refusal.next("settings_schema with an empty `key` lists the top level"));
        };
        description = next
            .get("description")
            .and_then(Value::as_str)
            .or(description);
        node = next;
        walked.push(segment);
    }
    let resolved = resolve(schema, node);
    let description = resolved
        .get("description")
        .and_then(Value::as_str)
        .or(description);
    let default = key
        .split('.')
        .filter(|segment| !segment.is_empty())
        .try_fold(defaults, |value, segment| value.get(segment))
        .cloned()
        .unwrap_or(Value::Null);
    let keys: Vec<Value> = resolved
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .take(MAX_KEYS)
                .map(|(name, property)| {
                    let line = property
                        .get("description")
                        .or_else(|| resolve(schema, property).get("description"))
                        .and_then(Value::as_str)
                        .and_then(|text| text.lines().next());
                    json!({ "key": name, "description": line })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(ToolAnswer {
        structured: json!({
            "key": key,
            "type": type_of(resolved),
            "description": description,
            "values": allowed_values(schema, resolved),
            "default": default,
            "keys": keys,
        }),
        text: None,
        image: None,
    })
}

/// A schema node with its reference followed and an optional's null branch dropped, as many times
/// as that takes.
fn resolve<'a>(schema: &'a Value, mut node: &'a Value) -> &'a Value {
    for _ in 0..16 {
        if let Some(target) = node
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|reference| reference.strip_prefix("#/$defs/"))
            .and_then(|name| schema.get("$defs")?.get(name))
        {
            node = target;
            continue;
        }
        let branches = node
            .get("anyOf")
            .or_else(|| node.get("allOf"))
            .and_then(Value::as_array);
        let not_null: Vec<&Value> = branches
            .map(|branches| {
                branches
                    .iter()
                    .filter(|branch| branch.get("type").and_then(Value::as_str) != Some("null"))
                    .collect()
            })
            .unwrap_or_default();
        match not_null.as_slice() {
            [only] => node = only,
            _ => return node,
        }
    }
    node
}

/// The names of an object node's keys.
fn property_names(schema: &Value, node: &Value) -> Vec<String> {
    resolve(schema, node)
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.keys().take(MAX_KEYS).cloned().collect())
        .unwrap_or_default()
}

/// A node's type as words: its `type`, or `one of` for a list of choices.
fn type_of(node: &Value) -> Option<String> {
    match node.get("type") {
        Some(Value::String(name)) => Some(name.clone()),
        Some(Value::Array(names)) => Some(
            names
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" or "),
        ),
        _ if node.get("enum").is_some() || node.get("oneOf").is_some() => {
            Some("one of".to_string())
        }
        _ => None,
    }
}

/// The values a node allows when it lists them: its `enum`, or its choices' constants, each with
/// the first line of its description.
fn allowed_values(schema: &Value, node: &Value) -> Vec<Value> {
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return values.clone();
    }
    node.get("oneOf")
        .or_else(|| node.get("anyOf"))
        .and_then(Value::as_array)
        .map(|choices| {
            choices
                .iter()
                .flat_map(|choice| {
                    let choice = resolve(schema, choice);
                    let line = choice
                        .get("description")
                        .and_then(Value::as_str)
                        .and_then(|text| text.lines().next());
                    let constants: Vec<Value> = choice
                        .get("const")
                        .map(|constant| vec![constant.clone()])
                        .or_else(|| choice.get("enum").and_then(Value::as_array).cloned())
                        .unwrap_or_default();
                    constants.into_iter().map(move |constant| match line {
                        Some(line) => json!({ "value": constant, "description": line }),
                        None => constant,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `settings_read`: the key's value in each settings file that sets it, highest precedence first,
/// which one wins, and the value in effect; secrets hidden.
fn read(arguments: &Value, cx: &App) -> Result<ToolAnswer, Refusal> {
    let key = key_argument(arguments);
    if key.is_empty() {
        return Err(Refusal::new("bad_argument", "`key` is empty")
            .next("give `key`, a key path such as `terminal.font_size`")
            .next("settings_schema with an empty `key` lists the top-level keys"));
    }
    // The key's own last name counts too: `env.API_TOKEN` names a secret.
    let last = key.rsplit('.').next();
    let store = cx.global::<SettingsStore>();
    let mut values = Vec::new();
    for file in store.get_all_files() {
        let Some(content) = store.get_content_for_file(file.clone()) else {
            continue;
        };
        let Some(value) = value_at(&serde_json::to_value(content).unwrap_or_default(), &key) else {
            continue;
        };
        values.push(json!({ "file": file_name(&file, cx), "value": hidden(value, last) }));
    }
    // The merged settings leave project files out, which apply only inside their project: they
    // are the value where no project file sets the key.
    let global = value_at(
        &serde_json::to_value(store.merged_settings()).unwrap_or_default(),
        &key,
    )
    .map(|value| hidden(value, last));
    if values.is_empty() && global.is_none() {
        return Err(Refusal::new(
            "no_setting",
            format!("no settings file sets `{key}`, and it has no default"),
        )
        .next("settings_schema tells whether the key exists and what it takes"));
    }
    let set_in = values
        .first()
        .and_then(|value| value.get("file"))
        .cloned()
        .unwrap_or(Value::Null);
    // A single value's winner is the highest-precedence file's; an object merges across files, so
    // its value in effect is the merged one.
    let effective = values
        .first()
        .and_then(|value| value.get("value"))
        .filter(|value| !value.is_object())
        .cloned()
        .or_else(|| global.clone());
    let redactor = crate::mcp::agent_redactor(cx);
    let mut structured = json!({
        "key": key,
        "effective": effective,
        "global": global,
        "set_in": set_in,
        "values": values,
    });
    redact_strings(&mut structured, redactor.as_deref());
    let size = structured.to_string().len();
    if size > MAX_ANSWER_BYTES {
        let keys: Vec<String> = structured
            .get("global")
            .and_then(Value::as_object)
            .map(|object| object.keys().take(MAX_KEYS).cloned().collect())
            .unwrap_or_default();
        let refusal = Refusal::new(
            "too_large",
            format!("`{key}` reads as {size} bytes, more than an answer holds"),
        );
        return Err(if keys.is_empty() {
            refusal.next("name a key under it")
        } else {
            refusal.next(format!(
                "name a key under it: {}",
                keys.iter()
                    .map(|name| format!("{key}.{name}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        });
    }
    Ok(ToolAnswer {
        structured,
        text: None,
        image: None,
    })
}

/// The value at a key path in a settings file's content, when the file sets it.
fn value_at(content: &Value, key: &str) -> Option<Value> {
    key.split('.')
        .try_fold(content, |value, segment| value.get(segment))
        .filter(|value| !value.is_null())
        .cloned()
}

/// Whether a key's name reads as a secret's.
fn secret_name(name: &str) -> bool {
    let name = name.to_lowercase();
    [
        "token",
        "secret",
        "password",
        "passwd",
        "credential",
        "auth",
        "api_key",
        "apikey",
    ]
    .iter()
    .any(|word| name.contains(word))
        || name == "key"
        || name.ends_with("_key")
}

/// `value` with each value under a secret's name hidden; `name` is the key it sits under.
fn hidden(value: Value, name: Option<&str>) -> Value {
    if name.is_some_and(secret_name) && !value.is_object() && !value.is_array() {
        return Value::String(HIDDEN.to_string());
    }
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    let value = hidden(value, Some(&key));
                    (key, value)
                })
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|item| hidden(item, name)).collect())
        }
        other => other,
    }
}

/// Runs every string in `value` through the agents' redaction (#516).
fn redact_strings(value: &mut Value, redactor: Option<&marley_mcp::redact::Redactor>) {
    match value {
        Value::String(text) => *text = crate::mcp::for_agents(text, redactor).text,
        Value::Array(items) => {
            for item in items {
                redact_strings(item, redactor);
            }
        }
        Value::Object(object) => {
            for item in object.values_mut() {
                redact_strings(item, redactor);
            }
        }
        _ => {}
    }
}

/// A settings file as an agent reads its name: `default`, `user <path>`, `project <folder>`.
fn file_name(file: &SettingsFile, cx: &App) -> String {
    match file {
        SettingsFile::Default => "default".to_string(),
        SettingsFile::Global => "global".to_string(),
        SettingsFile::Server => "server".to_string(),
        SettingsFile::User => format!("user {}", paths::settings_file().display()),
        SettingsFile::Project((worktree_id, directory)) => {
            let root = cx
                .windows()
                .into_iter()
                .filter_map(|window| window.downcast::<MultiWorkspace>()?.read(cx).ok())
                .flat_map(|multi_workspace| {
                    multi_workspace.workspaces().cloned().collect::<Vec<_>>()
                })
                .find_map(|workspace| {
                    let worktree = workspace
                        .read(cx)
                        .project()
                        .read(cx)
                        .worktree_for_id(*worktree_id, cx)?;
                    Some(worktree.read(cx).abs_path().display().to_string())
                })
                .unwrap_or_else(|| "a project".to_string());
            let folder = if directory.is_empty() {
                root
            } else {
                format!("{root}/{}", directory.as_unix_str())
            };
            format!("project {folder}/.zed/settings.json")
        }
    }
}
