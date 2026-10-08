//! `settings_change` (#682) and `keymap_change` (#686): an agent proposes a value for one key of
//! the user's settings, or a key binding for their keymap, and Marley writes it only when the user
//! accepts.
//!
//! The change is checked before anyone is asked: the key must be in Zed's settings schema, and the
//! file with the change must parse as Zed parses it, a parse error the file already had aside. The
//! question is an app notification with Apply and Decline, in every window, since the agent may run
//! where the user is not looking. The file is edited as text, as Zed's own settings writer edits it,
//! so comments and the other keys stay, and the write is refused when the file changed while the
//! user was asked. A binding goes through Zed's keymap updater (`KeymapFile::update_keybinding`,
//! the keymap editor's), after its action, keystrokes and context are checked.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fs::Fs;
use futures::FutureExt as _;
use futures::channel::oneshot;
use gpui::{
    App, AppContext as _, AsyncApp, KeyBindingContextPredicate, KeybindingKeystroke, Keystroke,
    SharedString,
};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Map, Value, json};
use settings::{
    KeybindUpdateOperation, KeybindUpdateTarget, KeymapFile, ParseStatus, RootUserSettings as _,
    UserSettingsContent,
};
use ui::prelude::*;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, dismiss_app_notification, show_app_notification};

use crate::settings_tools::{
    SchemaInputs, find_setting, hidden, key_argument, palette_name, value_at,
};

/// How long the user has to answer before the call is refused: under the 30 seconds the server
/// waits for the app's answer (`marley_mcp::APP_CALL_TIMEOUT_SECONDS`), as `terminal_type`'s 25.
const ANSWER_WAIT: Duration = Duration::from_secs(25);

/// The most characters of a value the question shows.
const SHOWN_CHARACTERS: usize = 200;

/// The indent a key Marley adds to the settings file gets, Zed's settings writer's.
const TAB_SIZE: usize = 2;

/// What an absent settings file reads as.
const EMPTY_SETTINGS: &str = "{}\n";

/// The most close action names a `no_action` refusal lists.
const CLOSE_ACTIONS: usize = 5;

/// The notifications' kind; each question has its own id within it.
struct SettingsChangeQuestion;

/// The next question's id.
static NEXT_QUESTION: AtomicUsize = AtomicUsize::new(0);

/// The user's answer, given once from whichever button is clicked; shared with the notification's
/// builder, which every window runs.
type AnswerSlot = Arc<Mutex<Option<oneshot::Sender<bool>>>>;

/// Answers `settings_change`: checks the change, asks the user, and writes it on Apply.
pub(crate) fn answer(call: AppCall, cx: &App) {
    let key = key_argument(&call.arguments);
    let value = call.arguments.get("value").cloned().unwrap_or(Value::Null);
    if key.is_empty() || value.is_null() {
        call.answer(Err(Refusal::new(
            "bad_argument",
            "give `key`, a key path such as `terminal.font_size`, and `value`, the JSON value to set",
        )
        .next("settings_schema tells what a key takes")));
        return;
    }
    let inputs = SchemaInputs::of(cx);
    let fs = <dyn Fs>::global(cx);
    let agent = call
        .caller()
        .client
        .clone()
        .unwrap_or_else(|| "An agent".to_string());
    cx.spawn(async move |cx| {
        let result = propose(&key, value, inputs, fs, &agent, cx).await;
        call.answer(result);
    })
    .detach();
}

/// Checks the change, asks the user and writes it.
async fn propose(
    key: &str,
    value: Value,
    inputs: SchemaInputs,
    fs: Arc<dyn Fs>,
    agent: &str,
    cx: &AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let checked = key.to_string();
    cx.background_executor()
        .spawn(futures::future::lazy(move |_| {
            find_setting(&inputs.schema(), &checked).map(|_| ())
        }))
        .await?;
    let path = paths::settings_file().clone();
    let text = load(&fs, &path, EMPTY_SETTINGS).await?;
    let (changed, before) = edit(&text, key, &value)?;
    if let Some(error) = parse_error(&changed)
        && parse_error(&text).as_ref() != Some(&error)
    {
        return Err(Refusal::new(
            "invalid_value",
            format!("the settings would not parse with it: {error}"),
        )
        .next("settings_schema tells the type and the values the key takes"));
    }
    let last = key.rsplit('.').next();
    let answer = |result: &str| ToolAnswer {
        structured: json!({
            "result": result,
            "key": key,
            "file": path.display().to_string(),
            "before": hidden(before.clone(), last),
            "after": hidden(value.clone(), last),
        }),
        text: None,
        image: None,
    };
    if changed == text {
        return Ok(answer("unchanged"));
    }
    let question = Question {
        headline: format!("{agent} wants to change your settings"),
        change: format!("{key}: {} → {}", shown(&before), shown(&value)),
        file: path.display().to_string(),
    };
    ask_then_write(question, &fs, &path, (&text, EMPTY_SETTINGS), changed, cx).await?;
    Ok(answer("applied"))
}

/// The file's text; an absent file reads as `absent`.
async fn load(fs: &Arc<dyn Fs>, path: &Path, absent: &str) -> Result<String, Refusal> {
    if !fs.is_file(path).await {
        return Ok(absent.to_string());
    }
    fs.load(path).await.map_err(|error| {
        Refusal::new(
            Refusal::REFUSED,
            format!("Marley could not read {}: {error:#}", path.display()),
        )
    })
}

/// The settings text with `key` set to `value`, edited as Zed's settings writer edits it, and the
/// key's value before.
fn edit(text: &str, key: &str, value: &Value) -> Result<(String, Value), Refusal> {
    let old_root: Value = settings::parse_json_with_comments(text).map_err(|error| {
        Refusal::new(
            Refusal::REFUSED,
            format!("the user's settings file does not parse as JSON: {error:#}"),
        )
        .next("the user fixes the file by hand first")
    })?;
    let before = value_at(&old_root, key).unwrap_or(Value::Null);
    let mut new_root = old_root.clone();
    set_at(&mut new_root, key, value.clone())?;
    let mut changed = text.to_string();
    let mut edits = Vec::new();
    settings::update_value_in_json_text(
        &mut changed,
        &mut Vec::new(),
        TAB_SIZE,
        &old_root,
        &new_root,
        &mut edits,
    );
    Ok((changed, before))
}

/// `root` with the value at the key path set, the objects on the way made where missing; refused
/// when a key on the way holds something other than an object.
fn set_at(root: &mut Value, key: &str, value: Value) -> Result<(), Refusal> {
    let segments: Vec<&str> = key.split('.').collect();
    let Some((last, parents)) = segments.split_last() else {
        return Err(Refusal::new("bad_argument", "`key` is empty"));
    };
    let mut node = root;
    for segment in parents {
        let Value::Object(object) = node else {
            return Err(Refusal::new(
                "invalid_value",
                format!("`{segment}` sits under a key that holds no object"),
            ));
        };
        node = object
            .entry((*segment).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    let Value::Object(object) = node else {
        return Err(Refusal::new(
            "invalid_value",
            format!("`{last}` sits under a key that holds no object"),
        ));
    };
    object.insert((*last).to_string(), value);
    Ok(())
}

/// The error Zed's parse of a settings text records, if any.
fn parse_error(text: &str) -> Option<String> {
    match UserSettingsContent::parse_json(text).1 {
        ParseStatus::Failed { error } => Some(error),
        ParseStatus::Success | ParseStatus::Unchanged => None,
    }
}

/// A value as the question shows it: compact JSON, cut to [`SHOWN_CHARACTERS`]; "not set" for none.
fn shown(value: &Value) -> String {
    if value.is_null() {
        return "not set".to_string();
    }
    let text = value.to_string();
    if text.chars().count() <= SHOWN_CHARACTERS {
        return text;
    }
    let cut: String = text.chars().take(SHOWN_CHARACTERS).collect();
    format!("{cut}…")
}

/// What the question shows: who asks, the change, the file.
#[derive(Clone)]
pub(crate) struct Question {
    pub(crate) headline: String,
    pub(crate) change: String,
    pub(crate) file: String,
}

/// Asks the user about a change to the file at `path`, read as `read`, and writes `changed` on
/// Apply, after checking the file still reads as it did.
///
/// # Errors
///
/// `declined`, `no_answer` after [`ANSWER_WAIT`], `changed` when the file moved, or the write's
/// own failure.
async fn ask_then_write(
    question: Question,
    fs: &Arc<dyn Fs>,
    path: &Path,
    (read, absent): (&str, &str),
    changed: String,
    cx: &AsyncApp,
) -> Result<(), Refusal> {
    ask_user(question, cx).await?;
    if load(fs, path, absent).await? != read {
        return Err(Refusal::new(
            "changed",
            "the file changed while the user was asked; nothing was written",
        )
        .next("propose the change again"));
    }
    fs.atomic_write(path.to_path_buf(), changed)
        .await
        .map_err(|error| {
            Refusal::new(
                Refusal::REFUSED,
                format!("Marley could not write {}: {error:#}", path.display()),
            )
        })
}

/// Asks the user `question` in every window and waits for Apply, at most [`ANSWER_WAIT`]; a
/// harness seat asks the same way (#692).
///
/// # Errors
///
/// `declined`, or `no_answer` when the wait ran out.
pub(crate) async fn ask_user(question: Question, cx: &AsyncApp) -> Result<(), Refusal> {
    let id = NotificationId::composite::<SettingsChangeQuestion>(
        NEXT_QUESTION.fetch_add(1, Ordering::Relaxed),
    );
    let (sender, receiver) = oneshot::channel();
    let slot: AnswerSlot = Arc::new(Mutex::new(Some(sender)));
    cx.update(|cx| ask(id.clone(), question, slot, cx));
    let timer = cx.background_executor().timer(ANSWER_WAIT);
    let answered = futures::select_biased! {
        answered = receiver.fuse() => answered.ok(),
        () = timer.fuse() => None,
    };
    cx.update(|cx| dismiss_app_notification(&id, cx));
    match answered {
        Some(true) => Ok(()),
        Some(false) => Err(Refusal::new("declined", "the user declined the change")
            .next("ask the user what they want instead")),
        None => Err(Refusal::new(
            "no_answer",
            format!(
                "the user did not answer within {} seconds; nothing was done",
                ANSWER_WAIT.as_secs()
            ),
        )
        .next("tell the user what you proposed, and propose it again when they are there")),
    }
}

/// Shows the question in every window, Apply and Decline answering through `slot`.
fn ask(id: NotificationId, question: Question, slot: AnswerSlot, cx: &mut App) {
    show_app_notification(id, cx, move |cx| {
        let question = question.clone();
        let apply = Arc::clone(&slot);
        let decline = Arc::clone(&slot);
        cx.new(|cx| {
            MessageNotification::new_from_builder(cx, move |_, _| {
                v_flex()
                    .gap_1()
                    .child(Label::new(SharedString::from(question.headline.clone())))
                    .child(
                        Label::new(SharedString::from(question.change.clone()))
                            .size(LabelSize::Small),
                    )
                    .child(
                        Label::new(SharedString::from(question.file.clone()))
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .into_any_element()
            })
            .primary_message("Apply")
            .primary_on_click(move |_, _| give(&apply, true))
            .secondary_message("Decline")
            .secondary_on_click(move |_, _| give(&decline, false))
        })
    });
}

/// Gives the user's answer, the first one only.
fn give(slot: &AnswerSlot, accepted: bool) {
    let sender = slot.lock().ok().and_then(|mut sender| sender.take());
    if let Some(sender) = sender
        && sender.send(accepted).is_err()
    {
        log::debug!("settings_change: the answer came after the call stopped waiting");
    }
}

/// Answers `keymap_change` (#686): checks the binding, asks the user, and adds it to their keymap
/// on Apply.
pub(crate) fn answer_keymap(call: AppCall, cx: &App) {
    let text = |name: &str| {
        call.arguments
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    };
    let (Some(typed), Some(action)) = (text("keystrokes"), text("action")) else {
        call.answer(Err(Refusal::new(
            "bad_argument",
            "give `keystrokes`, such as `ctrl-alt-m`, and `action`, an action's name",
        )
        .next("actions_list finds an action's name")));
        return;
    };
    let context = text("context");
    let arguments = call
        .arguments
        .get("arguments")
        .filter(|arguments| !arguments.is_null())
        .map(Value::to_string);
    if let Err(refusal) = check_binding(&typed, &action, context.as_deref(), cx) {
        call.answer(Err(refusal));
        return;
    }
    let fs = <dyn Fs>::global(cx);
    let agent = call
        .caller()
        .client
        .clone()
        .unwrap_or_else(|| "An agent".to_string());
    cx.spawn(async move |cx| {
        let binding = Binding {
            typed,
            action,
            context,
            arguments,
        };
        let result = propose_binding(binding, fs, &agent, cx).await;
        call.answer(result);
    })
    .detach();
}

/// A key binding an agent proposes.
struct Binding {
    /// The keystrokes as given, space between the steps of a sequence.
    typed: String,
    action: String,
    context: Option<String>,
    /// The action's arguments as JSON text.
    arguments: Option<String>,
}

/// Refuses a binding whose action the app lacks, whose keystrokes do not parse, or whose context
/// does not parse.
fn check_binding(
    typed: &str,
    action: &str,
    context: Option<&str>,
    cx: &App,
) -> Result<(), Refusal> {
    let names = cx.all_action_names();
    if !names.contains(&action) {
        let last = action.rsplit("::").next().unwrap_or(action).to_lowercase();
        let close: Vec<&str> = names
            .iter()
            .copied()
            .filter(|name| name.to_lowercase().contains(&last))
            .take(CLOSE_ACTIONS)
            .collect();
        let refusal = Refusal::new("no_action", format!("Marley has no action `{action}`"))
            .next("actions_list finds an action by words");
        return Err(if close.is_empty() {
            refusal
        } else {
            refusal.next(format!("actions with a name like it: {}", close.join(", ")))
        });
    }
    for step in typed.split_whitespace() {
        Keystroke::parse(step).map_err(|error| {
            Refusal::new(
                "bad_argument",
                format!("`{step}` is not a keystroke: {error}"),
            )
            .next("write keystrokes as Zed's keymap does: `ctrl-alt-m`, `cmd-k cmd-s`")
        })?;
    }
    if let Some(context) = context {
        KeyBindingContextPredicate::parse(context).map_err(|error| {
            Refusal::new(
                "bad_argument",
                format!("`{context}` is not a key context: {error:#}"),
            )
            .next("a context is a predicate such as `Workspace` or `Editor && mode == full`")
        })?;
    }
    Ok(())
}

/// Adds the binding through Zed's keymap updater, asks the user, and writes it.
async fn propose_binding(
    binding: Binding,
    fs: Arc<dyn Fs>,
    agent: &str,
    cx: &AsyncApp,
) -> Result<ToolAnswer, Refusal> {
    let path = paths::keymap_file().clone();
    let absent = settings::initial_keymap_content().to_string();
    let text = load(&fs, &path, &absent).await?;
    let changed = cx.update(|cx| {
        let mapper = std::rc::Rc::clone(cx.keyboard_mapper());
        let keystrokes: Vec<KeybindingKeystroke> = binding
            .typed
            .split_whitespace()
            .filter_map(|step| Keystroke::parse(step).ok())
            .map(|keystroke| {
                KeybindingKeystroke::new_with_mapper(keystroke, false, mapper.as_ref())
            })
            .collect();
        let operation = KeybindUpdateOperation::Add {
            source: KeybindUpdateTarget {
                context: binding.context.as_deref(),
                keystrokes: &keystrokes,
                action_name: &binding.action,
                action_arguments: binding.arguments.as_deref(),
            },
            from: None,
        };
        KeymapFile::update_keybinding(
            operation,
            text.clone(),
            TAB_SIZE,
            mapper.as_ref(),
            cx.deprecated_actions_to_preferred_actions(),
        )
    });
    let changed = changed.map_err(|error| {
        Refusal::new(
            "bad_argument",
            format!("Zed's keymap updater refused the binding: {error:#}"),
        )
    })?;
    let where_ = binding.context.as_deref().unwrap_or("every context");
    let question = Question {
        headline: format!("{agent} wants to bind a key"),
        change: format!(
            "{} → {} ({}) in {where_}",
            binding.typed,
            palette_name(&binding.action),
            binding.action
        ),
        file: path.display().to_string(),
    };
    ask_then_write(question, &fs, &path, (&text, &absent), changed, cx).await?;
    Ok(ToolAnswer {
        structured: json!({
            "result": "applied",
            "keystrokes": binding.typed,
            "action": binding.action,
            "context": binding.context,
            "file": path.display().to_string(),
        }),
        text: None,
        image: None,
    })
}
