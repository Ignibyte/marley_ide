//! `settings_change` (#682): an agent proposes a value for one key of the user's settings, and
//! Marley writes it only when the user accepts.
//!
//! The change is checked before anyone is asked: the key must be in Zed's settings schema, and the
//! file with the change must parse as Zed parses it, a parse error the file already had aside. The
//! question is an app notification with Apply and Decline, in every window, since the agent may run
//! where the user is not looking. The file is edited as text, as Zed's own settings writer edits it,
//! so comments and the other keys stay, and the write is refused when the file changed while the
//! user was asked.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fs::Fs;
use futures::FutureExt as _;
use futures::channel::oneshot;
use gpui::{App, AppContext as _, AsyncApp, SharedString};
use marley_mcp::{AppCall, Refusal, ToolAnswer};
use serde_json::{Map, Value, json};
use settings::{ParseStatus, RootUserSettings as _, UserSettingsContent};
use ui::prelude::*;
use workspace::notifications::simple_message_notification::MessageNotification;
use workspace::notifications::{NotificationId, dismiss_app_notification, show_app_notification};

use crate::settings_tools::{SchemaInputs, find_setting, hidden, key_argument, value_at};

/// How long the user has to answer before the call is refused: under the 30 seconds the server
/// waits for the app's answer (`marley_mcp::APP_CALL_TIMEOUT_SECONDS`), as `terminal_type`'s 25.
const ANSWER_WAIT: Duration = Duration::from_secs(25);

/// The most characters of a value the question shows.
const SHOWN_CHARACTERS: usize = 200;

/// The indent a key Marley adds to the settings file gets, Zed's settings writer's.
const TAB_SIZE: usize = 2;

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
    let text = load(&fs, &path).await?;
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
    let id = NotificationId::composite::<SettingsChangeQuestion>(
        NEXT_QUESTION.fetch_add(1, Ordering::Relaxed),
    );
    let (sender, receiver) = oneshot::channel();
    let slot: AnswerSlot = Arc::new(Mutex::new(Some(sender)));
    let question = Question {
        agent: agent.to_string(),
        key: key.to_string(),
        before: shown(&before),
        after: shown(&value),
        file: path.display().to_string(),
    };
    cx.update(|cx| ask(id.clone(), question, slot, cx));
    let timer = cx.background_executor().timer(ANSWER_WAIT);
    let answered = futures::select_biased! {
        answered = receiver.fuse() => answered.ok(),
        () = timer.fuse() => None,
    };
    cx.update(|cx| dismiss_app_notification(&id, cx));
    match answered {
        Some(true) => {}
        Some(false) => {
            return Err(Refusal::new("declined", "the user declined the change")
                .next("ask the user what they want instead"));
        }
        None => {
            return Err(Refusal::new(
                "no_answer",
                format!(
                    "the user did not answer within {} seconds; nothing was written",
                    ANSWER_WAIT.as_secs()
                ),
            )
            .next("tell the user what you proposed, and propose it again when they are there"));
        }
    }
    if load(&fs, &path).await? != text {
        return Err(Refusal::new(
            "changed",
            "the settings file changed while the user was asked; nothing was written",
        )
        .next("propose the change again"));
    }
    fs.atomic_write(path.clone(), changed)
        .await
        .map_err(|error| {
            Refusal::new(
                Refusal::REFUSED,
                format!("Marley could not write {}: {error:#}", path.display()),
            )
        })?;
    Ok(answer("applied"))
}

/// The user's settings file's text; an absent file reads as an empty object.
async fn load(fs: &Arc<dyn Fs>, path: &Path) -> Result<String, Refusal> {
    if !fs.is_file(path).await {
        return Ok("{}\n".to_string());
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

/// What the question shows.
#[derive(Clone)]
struct Question {
    agent: String,
    key: String,
    before: String,
    after: String,
    file: String,
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
                    .child(Label::new(SharedString::from(format!(
                        "{} wants to change your settings",
                        question.agent
                    ))))
                    .child(
                        Label::new(SharedString::from(format!(
                            "{}: {} → {}",
                            question.key, question.before, question.after
                        )))
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
