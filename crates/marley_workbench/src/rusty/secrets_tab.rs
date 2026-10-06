//! Rusty's secrets vault in a center tab (#667): the screen Rusty's app draws as
//! `SecretsPage.qml`.
//!
//! The names show to anyone; a value shows, changes or goes only behind the PIN. Unlocking gives
//! a token that lives in this tab alone, for as long as Rusty's `pin_timeout_minutes`; the tab
//! locks when it expires, on Lock, when Marley's window loses the focus, and when the tab closes.
//! Every PIN and value field is masked and emptied once sent, and nothing here logs a PIN, a token
//! or a value (Rusty-in-Marley R-D7).

use std::time::Duration;

use editor::Editor;
use gpui::{
    App, ClipboardItem, Context, Entity, EventEmitter, FocusHandle, Focusable, PromptLevel, Render,
    SharedString, Subscription, Task, WeakEntity, Window, actions,
};
use marley_rusty::secrets::{
    PinStatus, Revealed, SECRET_LIST, SECRET_LOCK, SECRET_PIN_STATUS, SECRET_REVEAL, SECRET_UNLOCK,
    SecretWrite, names_from_answer, revealed_from_answer, status_from_answer, unlock_from_answer,
};
use serde_json::json;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::item::{Item, ItemEvent};
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

use super::decisions_tab::{Link, ReadDue};

actions!(
    rusty,
    [
        /// Opens Secrets: Rusty's vault of keys and tokens, the values behind its PIN.
        #[derive(Eq)]
        OpenSecrets,
    ]
);

/// Rusty's own words on the vault, from its app.
const ABOUT: &str = "Keys for providers and services. A value is written once and shown only \
     behind the PIN; set it again to replace it. Once a PIN is set, setting and deleting need the \
     unlock too. The PIN protects this screen: the file stays readable for Rusty and its agents. \
     Never type the PIN to an agent.";

/// Registers `rusty: open secrets` on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &OpenSecrets, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Opens the Secrets tab once the update in progress ends; the rail's Brain view calls it.
pub(super) fn open_later(workspace: WeakEntity<Workspace>, window: &Window, cx: &mut App) {
    window.defer(cx, move |window, cx| {
        workspace
            .update(cx, |workspace, cx| open(workspace, window, cx))
            .log_err();
    });
}

/// Brings the workspace's Secrets tab forward with the focus, or adds one to the active pane; with
/// Rusty off, says so and opens nothing.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        workspace.show_toast(
            Toast::new(
                NotificationId::unique::<BrainSecretsView>(),
                reason.to_string(),
            ),
            cx,
        );
        return;
    }
    let open = workspace.items_of_type::<BrainSecretsView>(cx).next();
    if let Some(view) = open {
        workspace.activate_item(&view, true, true, window, cx);
        return;
    }
    let weak = cx.entity().downgrade();
    let view = cx.new(|cx| BrainSecretsView::new(weak, window, cx));
    workspace.add_item_to_active_pane(Box::new(view), None, true, window, cx);
}

/// A masked one-line field.
fn masked_field(placeholder: &str, window: &mut Window, cx: &mut App) -> Entity<Editor> {
    cx.new(|cx| {
        let mut editor = Editor::single_line(window, cx);
        editor.set_masked(true, cx);
        editor.set_placeholder_text(placeholder, window, cx);
        editor
    })
}

/// Empties `field`, so a PIN or a value sent leaves nothing on screen.
fn empty(field: &Entity<Editor>, window: &mut Window, cx: &mut App) {
    field.update(cx, |editor, cx| editor.set_text("", window, cx));
}

/// The tab's word on the last thing it did: Rusty's refusal, or what went well.
#[derive(Clone, Debug)]
struct Notice {
    text: SharedString,
    color: Color,
}

impl Notice {
    fn refused(error: &str) -> Self {
        Self {
            text: error.lines().next().unwrap_or_default().to_string().into(),
            color: Color::Error,
        }
    }

    const fn done(text: &'static str) -> Self {
        Self {
            text: SharedString::new_static(text),
            color: Color::Muted,
        }
    }
}

/// The tab's fields, every PIN and value masked.
struct Fields {
    pin: Entity<Editor>,
    pin_again: Entity<Editor>,
    unlock_pin: Entity<Editor>,
    key: Entity<Editor>,
    value: Entity<Editor>,
    replace_value: Entity<Editor>,
}

/// The Secrets tab.
pub(super) struct BrainSecretsView {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    /// The names and the lock's state as last read; none before the first.
    listed: Option<(Vec<String>, PinStatus)>,
    failure: Option<SharedString>,
    reading: Option<Task<()>>,
    due: ReadDue,
    link: Link,
    /// The live token from this tab's unlock; none while locked.
    token: Option<String>,
    /// Locks the tab when the unlock runs out.
    expiry: Option<Task<()>>,
    /// The one value shown, until Hide, another Reveal or the lock.
    revealed: Option<Revealed>,
    /// The secret whose replace field shows.
    replacing: Option<String>,
    changing_pin: bool,
    fields: Fields,
    notice: Option<Notice>,
    writing: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl BrainSecretsView {
    fn new(workspace: WeakEntity<Workspace>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe_global::<super::Announced>(|this, cx| {
                if this.showing(cx) {
                    this.read(cx);
                } else if this.due == ReadDue::No {
                    this.due = ReadDue::WhenShown;
                }
            }),
            cx.observe_global::<super::Rusty>(Self::rusty_changed),
            // Losing the window relocks, as Rusty's app does.
            cx.observe_window_activation(window, |this, window, cx| {
                if !window.is_window_active() {
                    this.lock(cx);
                }
            }),
            // A closed tab leaves no live token behind.
            cx.on_release(|this, cx| {
                if this.token.is_some() {
                    Self::send_lock(cx);
                }
            }),
        ];
        let key = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("key, such as openai_api_key", window, cx);
            editor
        });
        let fields = Fields {
            pin: masked_field("PIN, six characters or more", window, cx),
            pin_again: masked_field("again", window, cx),
            unlock_pin: masked_field("PIN", window, cx),
            key,
            value: masked_field("value", window, cx),
            replace_value: masked_field("new value, Enter saves", window, cx),
        };
        let mut view = Self {
            workspace,
            focus_handle: cx.focus_handle(),
            listed: None,
            failure: None,
            reading: None,
            due: ReadDue::No,
            link: Link::now(cx),
            token: None,
            expiry: None,
            revealed: None,
            replacing: None,
            changing_pin: false,
            fields,
            notice: None,
            writing: None,
            _subscriptions: subscriptions,
        };
        view.read(cx);
        view
    }

    /// Whether the tab is its pane's active item, asked each time (PR-607).
    fn showing(&self, cx: &Context<Self>) -> bool {
        let Some(workspace) = self.workspace.upgrade() else {
            return false;
        };
        let id = cx.entity_id();
        workspace.read(cx).panes().iter().any(|pane| {
            pane.read(cx)
                .active_item()
                .is_some_and(|item| item.item_id() == id)
        })
    }

    /// Off drops everything, the token first; the link coming up reads (L-658).
    fn rusty_changed(&mut self, cx: &mut Context<Self>) {
        let link = Link::now(cx);
        if link == self.link {
            return;
        }
        self.link = link;
        match link {
            Link::Off => {
                self.forget();
                self.listed = None;
                self.failure = None;
                self.reading = None;
                self.due = ReadDue::No;
            }
            Link::Up => self.read(cx),
            Link::Down => self.forget(),
        }
        cx.notify();
    }

    /// Reads the names and the lock's state together; with one read running, one more after it.
    fn read(&mut self, cx: &Context<Self>) {
        if !super::is_on(cx) || !super::is_connected(cx) {
            return;
        }
        if self.reading.is_some() {
            self.due = ReadDue::AfterThis;
            return;
        }
        self.due = ReadDue::No;
        let names = super::call_tool(SECRET_LIST, json!({}), cx);
        let status = super::call_tool(SECRET_PIN_STATUS, json!({}), cx);
        self.reading = Some(cx.spawn(async move |this, cx| {
            let names = names.await.and_then(|text| names_from_answer(&text));
            let status = status.await.and_then(|text| status_from_answer(&text));
            let read = names.and_then(|names| status.map(|status| (names, status)));
            this.update(cx, |this, cx| this.take_read(read, cx))
                .log_err();
        }));
    }

    fn read_again(&mut self, cx: &mut Context<Self>) {
        self.read(cx);
        cx.notify();
    }

    fn take_read(
        &mut self,
        read: Result<(Vec<String>, PinStatus), String>,
        cx: &mut Context<Self>,
    ) {
        self.reading = None;
        match read {
            Ok(listed) => {
                if self
                    .revealed
                    .as_ref()
                    .is_some_and(|revealed| !listed.0.contains(&revealed.key))
                {
                    self.revealed = None;
                }
                self.listed = Some(listed);
                self.failure = None;
            }
            Err(error) => {
                let line = error.lines().next().unwrap_or_default().to_string();
                self.failure = Some(line.into());
            }
        }
        if self.due == ReadDue::AfterThis {
            self.read(cx);
        }
        cx.notify();
    }

    fn pin_set(&self) -> bool {
        self.listed.as_ref().is_some_and(|(_, status)| status.set)
    }

    /// Whether a secret can be set or deleted now: with no PIN, or unlocked.
    fn can_write(&self) -> bool {
        !self.pin_set() || self.token.is_some()
    }

    /// Drops the token, the shown value and the timer, without telling Rusty.
    fn forget(&mut self) {
        self.token = None;
        self.expiry = None;
        self.revealed = None;
        self.replacing = None;
        self.changing_pin = false;
    }

    /// Locks the tab and Rusty's vault.
    fn lock(&mut self, cx: &mut Context<Self>) {
        if self.token.is_none() {
            return;
        }
        self.forget();
        Self::send_lock(cx);
        self.notice = Some(Notice::done("Locked."));
        self.read_again(cx);
    }

    fn send_lock(cx: &App) {
        let asking = super::call_tool(SECRET_LOCK, json!({}), cx);
        cx.spawn(async move |_| {
            asking.await.log_err();
        })
        .detach();
    }

    /// Unlocks with the PIN typed; the token stays in this tab until the lock.
    fn unlock(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pin = self.fields.unlock_pin.read(cx).text(cx);
        empty(&self.fields.unlock_pin, window, cx);
        if pin.is_empty() || self.writing.is_some() {
            return;
        }
        self.notice = None;
        let asking = super::call_tool(SECRET_UNLOCK, json!({ "pin": pin }), cx);
        self.writing = Some(cx.spawn(async move |this, cx| {
            let unlocked = asking.await.and_then(|text| unlock_from_answer(&text));
            this.update(cx, |this, cx| {
                this.writing = None;
                match unlocked {
                    Ok(unlock) => {
                        this.token = Some(unlock.token);
                        let lifetime = Duration::from_secs(unlock.expires_in_seconds.max(1));
                        this.expiry = Some(cx.spawn(async move |this, cx| {
                            cx.background_executor().timer(lifetime).await;
                            this.update(cx, Self::lock).log_err();
                        }));
                        this.notice = None;
                    }
                    Err(error) => this.notice = Some(Notice::refused(&error)),
                }
                this.read_again(cx);
            })
            .log_err();
        }));
        cx.notify();
    }

    /// Sets the PIN, or changes it while unlocked, when it was typed twice alike.
    fn set_pin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pin = self.fields.pin.read(cx).text(cx);
        let again = self.fields.pin_again.read(cx).text(cx);
        empty(&self.fields.pin, window, cx);
        empty(&self.fields.pin_again, window, cx);
        if pin != again {
            self.notice = Some(Notice::done("The two PINs differ; type it twice alike."));
            cx.notify();
            return;
        }
        let write = SecretWrite::PinSet {
            pin,
            token: self.token.clone(),
        };
        self.send(&write, Notice::done("PIN set."), cx);
    }

    /// Sets the secret the key and value fields hold.
    fn set_secret(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.fields.key.read(cx).text(cx).trim().to_string();
        let value = self.fields.value.read(cx).text(cx);
        empty(&self.fields.value, window, cx);
        if key.is_empty() || value.is_empty() || !self.can_write() {
            return;
        }
        empty(&self.fields.key, window, cx);
        let write = SecretWrite::Set {
            key,
            value,
            token: self.token.clone(),
        };
        self.send(&write, Notice::done("Set."), cx);
    }

    /// Replaces the value of the secret whose replace field shows.
    fn replace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.fields.replace_value.read(cx).text(cx);
        empty(&self.fields.replace_value, window, cx);
        let (Some(key), Some(token)) = (self.replacing.clone(), self.token.clone()) else {
            return;
        };
        if value.is_empty() {
            return;
        }
        self.replacing = None;
        if self
            .revealed
            .as_ref()
            .is_some_and(|revealed| revealed.key == key)
        {
            self.revealed = None;
        }
        self.send(
            &SecretWrite::Update { key, value, token },
            Notice::done("Replaced."),
            cx,
        );
    }

    /// Asks first, then deletes the secret `key`.
    fn delete(key: String, window: &mut Window, cx: &mut Context<Self>) {
        let answer = window.prompt(
            PromptLevel::Warning,
            &format!("Remove {key} from the vault?"),
            Some("Rusty deletes the value; agents that read it will not find it."),
            &["Delete", "Cancel"],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if answer.await.ok() != Some(0) {
                return;
            }
            this.update(cx, |this, cx| {
                if this
                    .revealed
                    .as_ref()
                    .is_some_and(|revealed| revealed.key == key)
                {
                    this.revealed = None;
                }
                let write = SecretWrite::Delete {
                    key,
                    token: this.token.clone(),
                };
                this.send(&write, Notice::done("Deleted."), cx);
            })
            .log_err();
        })
        .detach();
    }

    /// Sends `write`; a success says `done`, a refusal Rusty's words, and the names read again.
    fn send(&mut self, write: &SecretWrite, done: Notice, cx: &mut Context<Self>) {
        if self.writing.is_some() {
            return;
        }
        self.notice = None;
        let pin_set = matches!(write, SecretWrite::PinSet { .. });
        let asking = super::call_tool(write.tool(), write.arguments(), cx);
        self.writing = Some(cx.spawn(async move |this, cx| {
            let answer = asking.await;
            this.update(cx, |this, cx| {
                this.writing = None;
                match answer {
                    Ok(_) => {
                        if pin_set {
                            this.changing_pin = false;
                        }
                        this.notice = Some(done);
                    }
                    Err(error) => this.notice = Some(Notice::refused(&error)),
                }
                this.read_again(cx);
            })
            .log_err();
        }));
        cx.notify();
    }

    /// Shows the value of `key`, or hides it when shown.
    fn reveal(&mut self, key: &str, cx: &mut Context<Self>) {
        if self
            .revealed
            .as_ref()
            .is_some_and(|revealed| revealed.key == key)
        {
            self.revealed = None;
            cx.notify();
            return;
        }
        let Some(token) = self.token.clone() else {
            return;
        };
        let asking = super::call_tool(SECRET_REVEAL, json!({ "key": key, "token": token }), cx);
        self.writing = Some(cx.spawn(async move |this, cx| {
            let revealed = asking.await.and_then(|text| revealed_from_answer(&text));
            this.update(cx, |this, cx| {
                this.writing = None;
                match revealed {
                    Ok(revealed) if this.token.is_some() => this.revealed = Some(revealed),
                    Ok(_) => {}
                    Err(error) => this.notice = Some(Notice::refused(&error)),
                }
                cx.notify();
            })
            .log_err();
        }));
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if let Some(revealed) = &self.revealed {
            cx.write_to_clipboard(ClipboardItem::new_string(revealed.value.clone()));
            self.notice = Some(Notice::done("Copied."));
            cx.notify();
        }
    }

    fn start_replace(&mut self, key: String, window: &mut Window, cx: &mut Context<Self>) {
        self.replacing = Some(key);
        self.fields.replace_value.focus_handle(cx).focus(window, cx);
        cx.notify();
    }
}

fn line(text: impl Into<SharedString>, color: Color) -> AnyElement {
    Label::new(text)
        .size(LabelSize::Small)
        .color(color)
        .into_any_element()
}

/// An editor in a box with a border.
fn boxed(editor: &Entity<Editor>, cx: &App) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(cx.theme().colors().border)
        .child(editor.clone())
}

impl BrainSecretsView {
    fn render_state(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if !super::is_on(cx) {
            return Some(line(
                "Rusty is off. Turn it on in the Rusty section of the Marley settings.",
                Color::Muted,
            ));
        }
        if !super::is_connected(cx) {
            let reason = super::unavailable(cx).unwrap_or_default();
            return Some(line(
                format!("Marley is not connected to Rusty: {reason}"),
                Color::Muted,
            ));
        }
        if let Some(failure) = &self.failure {
            return Some(
                h_flex()
                    .gap_2()
                    .child(
                        Label::new(failure.clone())
                            .size(LabelSize::Small)
                            .color(Color::Error),
                    )
                    .child(
                        Button::new("rusty-secrets-read-again", "Read again")
                            .label_size(LabelSize::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.read_again(cx))),
                    )
                    .into_any_element(),
            );
        }
        self.listed
            .is_none()
            .then(|| line("Reading Rusty's vault…", Color::Muted))
    }

    /// The PIN twice and Set PIN: a first PIN, or a new one while unlocked.
    fn render_pin_set(&self, cx: &Context<Self>) -> AnyElement {
        h_flex()
            .key_context("RustySecret menu")
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.set_pin(window, cx)))
            .gap_2()
            .child(boxed(&self.fields.pin, cx).w(rems(16.)))
            .child(boxed(&self.fields.pin_again, cx).w(rems(16.)))
            .child(
                Button::new("rusty-secrets-set-pin", "Set PIN")
                    .style(ButtonStyle::Filled)
                    .on_click(cx.listener(|this, _, window, cx| this.set_pin(window, cx))),
            )
            .into_any_element()
    }

    /// The lock: set a PIN, unlock with it, or lock and change it.
    fn render_lock(&self, status: PinStatus, cx: &Context<Self>) -> AnyElement {
        if !status.set {
            return v_flex()
                .gap_1()
                .child(Label::new("Set a PIN"))
                .child(self.render_pin_set(cx))
                .child(line(
                    "Digits or a passphrase; it protects this screen only.",
                    Color::Muted,
                ))
                .into_any_element();
        }
        if self.token.is_none() {
            return v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .key_context("RustySecret menu")
                        .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| {
                            this.unlock(window, cx);
                        }))
                        .gap_2()
                        .child(boxed(&self.fields.unlock_pin, cx).w(rems(16.)))
                        .child(
                            Button::new("rusty-secrets-unlock", "Unlock")
                                .style(ButtonStyle::Filled)
                                .disabled(status.locked_out_seconds > 0)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.unlock(window, cx);
                                })),
                        ),
                )
                .child(line(
                    if status.locked_out_seconds > 0 {
                        format!(
                            "Locked for {} seconds after wrong PINs.",
                            status.locked_out_seconds
                        )
                    } else {
                        "Unlocking shows, edits, copies and deletes values for a few minutes."
                            .to_string()
                    },
                    Color::Muted,
                ))
                .into_any_element();
        }
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new("Unlocked"))
                    .child(
                        Button::new("rusty-secrets-lock", "Lock")
                            .on_click(cx.listener(|this, _, _, cx| this.lock(cx))),
                    )
                    .child(
                        Button::new("rusty-secrets-change-pin", "Change PIN").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.changing_pin = !this.changing_pin;
                                cx.notify();
                            }),
                        ),
                    ),
            )
            .when(self.changing_pin, |lock| {
                lock.child(self.render_pin_set(cx))
            })
            .into_any_element()
    }

    /// A key, a masked value and Set.
    fn render_set_row(&self, cx: &Context<Self>) -> AnyElement {
        let can_write = self.can_write();
        h_flex()
            .key_context("RustySecret menu")
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| {
                this.set_secret(window, cx);
            }))
            .gap_2()
            .child(boxed(&self.fields.key, cx).w(rems(16.)))
            .child(boxed(&self.fields.value, cx).flex_1().min_w_0())
            .child(
                Button::new("rusty-secrets-set", "Set")
                    .disabled(!can_write)
                    .on_click(cx.listener(|this, _, window, cx| this.set_secret(window, cx))),
            )
            .when(!can_write, |row| {
                row.child(line("Unlock to set a value.", Color::Muted))
            })
            .into_any_element()
    }

    fn render_name(&self, index: usize, key: &str, cx: &Context<Self>) -> AnyElement {
        let unlocked = self.token.is_some();
        let can_write = self.can_write();
        let shown = self
            .revealed
            .as_ref()
            .filter(|revealed| revealed.key == key)
            .map(|revealed| revealed.value.clone());
        let replacing = self.replacing.as_deref() == Some(key);
        let reveal_key = key.to_string();
        let replace_key = key.to_string();
        let delete_key = key.to_string();
        v_flex()
            .gap_1()
            .py_1()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .flex_none()
                            .w(rems(16.))
                            .child(Label::new(key.to_string()).buffer_font(cx).truncate()),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            Label::new(shown.clone().unwrap_or_else(|| "••••••••".to_string()))
                                .buffer_font(cx)
                                .color(if shown.is_some() {
                                    Color::Accent
                                } else {
                                    Color::Muted
                                })
                                .truncate(),
                        ),
                    )
                    .when(unlocked, |row| {
                        row.child(
                            Button::new(
                                ("rusty-secret-reveal", index),
                                if shown.is_some() { "Hide" } else { "Reveal" },
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    this.reveal(&reveal_key, cx);
                                },
                            )),
                        )
                    })
                    .when(shown.is_some(), |row| {
                        row.child(
                            Button::new(("rusty-secret-copy", index), "Copy")
                                .on_click(cx.listener(|this, _, _, cx| this.copy(cx))),
                        )
                    })
                    .when(unlocked, |row| {
                        row.child(
                            Button::new(("rusty-secret-replace", index), "Replace").on_click(
                                cx.listener(move |this, _, window, cx| {
                                    this.start_replace(replace_key.clone(), window, cx);
                                }),
                            ),
                        )
                    })
                    .when(can_write, |row| {
                        row.child(
                            Button::new(("rusty-secret-delete", index), "Delete").on_click(
                                cx.listener(move |_, _, window, cx| {
                                    Self::delete(delete_key.clone(), window, cx);
                                }),
                            ),
                        )
                    }),
            )
            .when(replacing, |name| {
                name.child(
                    h_flex()
                        .key_context("RustySecret menu")
                        .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| {
                            this.replace(window, cx);
                        }))
                        .pl(rems(16.5))
                        .child(boxed(&self.fields.replace_value, cx).w(rems(20.))),
                )
            })
            .into_any_element()
    }
}

impl Focusable for BrainSecretsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for BrainSecretsView {}

impl Render for BrainSecretsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // A change announced while the tab was hidden is read now that it draws (AD-609).
        if self.due == ReadDue::WhenShown && self.reading.is_none() {
            self.read(cx);
        }
        let state = self.render_state(cx);
        let listed = self
            .listed
            .clone()
            .filter(|_| super::is_on(cx) && super::is_connected(cx));
        v_flex()
            .key_context("RustySecrets")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                v_flex()
                    .px_4()
                    .pt_3()
                    .pb_2()
                    .gap_1()
                    .child(Label::new("Secrets").size(LabelSize::Large))
                    .child(line(ABOUT, Color::Muted))
                    .children(state),
            )
            .children(listed.map(|(names, status)| {
                v_flex()
                    .id("rusty-secrets")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_4()
                    .gap_3()
                    .child(self.render_lock(status, cx))
                    .child(self.render_set_row(cx))
                    .children(
                        self.notice
                            .as_ref()
                            .map(|notice| line(notice.text.clone(), notice.color)),
                    )
                    .child(Label::new(if names.len() == 1 {
                        "1 secret".to_string()
                    } else {
                        format!("{} secrets", names.len())
                    }))
                    .children(
                        names
                            .iter()
                            .enumerate()
                            .map(|(index, key)| self.render_name(index, key, cx)),
                    )
            }))
    }
}

impl Item for BrainSecretsView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("Secrets")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Lock))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static("Rusty's secrets vault"))
    }

    fn to_item_events(event: &ItemEvent, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    /// With the service connection no change is announced, so each showing reads (#647's D9).
    fn deactivated(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(
            cx.global::<super::Rusty>().source,
            super::Source::Service(_)
        ) {
            self.due = ReadDue::WhenShown;
        }
    }

    fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        // L-613: an item moved to another workspace follows it.
        self.workspace = workspace.weak_handle();
    }
}
