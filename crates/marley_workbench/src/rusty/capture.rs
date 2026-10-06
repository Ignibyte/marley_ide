//! Capturing into Rusty's brain from the palette (#663): a line into today's daily note or the
//! inbox, a URL kept as a source page, and today's note opened.
//!
//! A capture is one call to Rusty from a small form over the workspace. A line's capture closes
//! the form and says in a toast where it went; a URL's opens the page Rusty made, which records a
//! failed fetch itself. Rusty's refusal stays in the form with what was typed.

use editor::Editor;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render, SharedString,
    Task, WeakEntity, Window, actions,
};
use marley_rusty::capture::{
    BRAIN_CAPTURE, CAPTURE_URL_DEADLINE, CaptureTarget, SOURCE_CAPTURE, receipt_from_answer,
    source_arguments, source_page_from_answer,
};
use marley_rusty::vault::{self, BRAIN_DAILY_NOTE};
use serde_json::json;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

actions!(
    rusty,
    [
        /// Appends a line to today's daily note in Rusty's brain.
        #[derive(Eq)]
        CaptureToToday,
        /// Appends a line to the inbox page in Rusty's brain.
        #[derive(Eq)]
        CaptureToInbox,
        /// Has Rusty fetch a web page or a file by its URL and keep it as a source page.
        #[derive(Eq)]
        CaptureUrl,
        /// Opens today's daily note from Rusty's brain, made when missing.
        #[derive(Eq)]
        OpenToday,
    ]
);

/// Registers the capture commands on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &CaptureToToday, window, cx| {
            open_form(workspace, Kind::Line(CaptureTarget::Daily), window, cx);
        });
        workspace.register_action(|workspace, _: &CaptureToInbox, window, cx| {
            open_form(workspace, Kind::Line(CaptureTarget::Inbox), window, cx);
        });
        workspace.register_action(|workspace, _: &CaptureUrl, window, cx| {
            open_form(workspace, Kind::Url, window, cx);
        });
        workspace.register_action(|workspace, _: &OpenToday, window, cx| {
            if ready(workspace, cx) {
                open_today(window, cx);
            }
        });
    })
    .detach();
}

/// Whether Rusty can take a call now; when it can't, a toast says why.
pub(super) fn ready(workspace: &mut Workspace, cx: &mut Context<Workspace>) -> bool {
    let Some(reason) = super::unavailable(cx) else {
        return true;
    };
    workspace.show_toast(
        Toast::new(NotificationId::unique::<CaptureForm>(), reason.to_string()),
        cx,
    );
    false
}

/// Shows `message` in `workspace`'s toast, with Open for the page `slug` when one is given: a
/// confirmation, which hides itself.
fn toast(workspace: &WeakEntity<Workspace>, message: String, open: Option<String>, cx: &mut App) {
    let mut toast = Toast::new(NotificationId::unique::<CaptureForm>(), message);
    if let Some(slug) = open {
        let opener = workspace.clone();
        // A confirmation goes by itself, as Zed's own do; a failure stays until closed.
        toast = toast
            .on_click("Open", move |window, cx| {
                super::page::open_later(opener.clone(), slug.clone(), false, true, window, cx);
            })
            .autohide();
    }
    workspace
        .update(cx, |workspace, cx| workspace.show_toast(toast, cx))
        .log_err();
}

/// Opens Rusty's daily note for today in a kept tab with the focus: what the Brain view's Today
/// does, without the tree.
fn open_today(window: &Window, cx: &Context<Workspace>) {
    let asking = super::call_tool(BRAIN_DAILY_NOTE, json!({}), cx);
    cx.spawn_in(window, async move |workspace, cx| {
        let slug = asking.await.and_then(|text| {
            vault::page_slug_from_answer(&text)
                .map_err(|error| format!("{BRAIN_DAILY_NOTE}'s answer did not parse: {error}"))
        });
        workspace
            .update_in(cx, |workspace, window, cx| match slug {
                Ok(slug) => {
                    super::reread_vault(cx);
                    super::page::open_later(cx.weak_entity(), slug, false, true, window, cx);
                }
                Err(error) => workspace.show_toast(
                    Toast::new(
                        NotificationId::unique::<CaptureForm>(),
                        format!("Could not open today's note: {error}"),
                    ),
                    cx,
                ),
            })
            .log_err();
    })
    .detach();
}

fn open_form(
    workspace: &mut Workspace,
    kind: Kind,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    if !ready(workspace, cx) {
        return;
    }
    let weak = cx.weak_entity();
    workspace.toggle_modal(window, cx, |window, cx| {
        CaptureForm::new(kind, weak, window, cx)
    });
}

/// What the form captures.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Line(CaptureTarget),
    Url,
}

impl Kind {
    const fn title(self) -> &'static str {
        match self {
            Self::Line(target) => target.title(),
            Self::Url => "Capture a URL as a Source",
        }
    }

    const fn placeholder(self) -> &'static str {
        match self {
            Self::Line(_) => "A line to capture",
            Self::Url => "https://…",
        }
    }
}

/// The form's width in rems, #660's form's.
const FORM_WIDTH: f32 = 34.;

pub(super) struct CaptureForm {
    kind: Kind,
    workspace: WeakEntity<Workspace>,
    field: Entity<Editor>,
    sending: Option<Task<()>>,
    /// Rusty's words when it refused the last capture.
    refusal: Option<SharedString>,
}

impl CaptureForm {
    fn new(
        kind: Kind,
        workspace: WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let field = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text(kind.placeholder(), window, cx);
            editor
        });
        Self {
            kind,
            workspace,
            field,
            sending: None,
            refusal: None,
        }
    }

    /// Sends what the field holds, as typed: Rusty trims it and refuses what it can't take.
    fn send(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.sending.is_some() {
            return;
        }
        let text = self.field.read(cx).text(cx);
        self.refusal = None;
        self.sending = Some(match self.kind {
            Kind::Line(target) => Self::send_line(target, &text, window, cx),
            Kind::Url => Self::send_url(&text, window, cx),
        });
        cx.notify();
    }

    fn send_line(
        target: CaptureTarget,
        text: &str,
        window: &Window,
        cx: &Context<Self>,
    ) -> Task<()> {
        let asking = super::call_tool(BRAIN_CAPTURE, target.arguments(text), cx);
        cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await.and_then(|text| {
                receipt_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_CAPTURE}'s answer did not parse: {error}"))
            });
            this.update(cx, |this, cx| {
                this.sending = None;
                match answer {
                    Ok(receipt) => {
                        super::reread_vault(cx);
                        toast(
                            &this.workspace,
                            format!("Captured to {}", receipt.slug),
                            Some(receipt.slug),
                            cx,
                        );
                        cx.emit(DismissEvent);
                    }
                    Err(error) => this.refused(&error),
                }
                cx.notify();
            })
            .log_err();
        })
    }

    fn send_url(url: &str, window: &Window, cx: &Context<Self>) -> Task<()> {
        let asking = super::call_tool_within(
            SOURCE_CAPTURE,
            source_arguments(url),
            CAPTURE_URL_DEADLINE,
            cx,
        );
        let url = url.trim().to_string();
        cx.spawn_in(window, async move |this, cx| {
            let answer = asking.await.and_then(|text| {
                source_page_from_answer(&text)
                    .map_err(|error| format!("{SOURCE_CAPTURE}'s answer did not parse: {error}"))
            });
            this.update_in(cx, |this, window, cx| {
                this.sending = None;
                match answer {
                    Ok(page) => {
                        super::reread_vault(cx);
                        if let Some(error) = page.failed {
                            toast(
                                &this.workspace,
                                format!("Could not fetch {url}: {error}"),
                                None,
                                cx,
                            );
                        }
                        super::page::open_later(
                            this.workspace.clone(),
                            page.slug,
                            false,
                            true,
                            window,
                            cx,
                        );
                        cx.emit(DismissEvent);
                    }
                    Err(error) => this.refused(&error),
                }
                cx.notify();
            })
            .log_err();
        })
    }

    fn refused(&mut self, error: &str) {
        let line = error.lines().next().unwrap_or_default().to_string();
        self.refusal = Some(line.into());
    }
}

impl ModalView for CaptureForm {}

impl EventEmitter<DismissEvent> for CaptureForm {}

impl Focusable for CaptureForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.field.focus_handle(cx)
    }
}

impl Render for CaptureForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sending = self.sending.is_some();
        v_flex()
            .key_context("RustyCapture menu")
            .w(rems(FORM_WIDTH))
            .p_3()
            .gap_2()
            .elevation_3(cx)
            .on_action(cx.listener(|this, _: &menu::Confirm, window, cx| this.send(window, cx)))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .child(Label::new(self.kind.title()))
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(cx.theme().colors().border)
                    .child(self.field.clone()),
            )
            .children(self.refusal.clone().map(|refusal| {
                Label::new(refusal)
                    .size(LabelSize::Small)
                    .color(Color::Error)
            }))
            .child(
                h_flex()
                    .gap_2()
                    .justify_between()
                    .child(
                        Label::new("Enter captures, Escape closes")
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Button::new(
                            "rusty-capture-send",
                            if sending { "Capturing…" } else { "Capture" },
                        )
                        .style(ButtonStyle::Filled)
                        .disabled(sending)
                        .on_click(cx.listener(|this, _, window, cx| this.send(window, cx))),
                    ),
            )
    }
}
