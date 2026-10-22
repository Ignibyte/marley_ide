//! The System tab (#855): pages about Marley itself.
//!
//! Each page is a card as the `SystemPages` board draws one: a title, what it covers, a lead line,
//! rows and buttons. This build names the build and its data folder; Where data lives lists
//! Marley's folders with their sizes, counted off the main thread when the page opens and on Count
//! again.

use std::path::{Path, PathBuf};

use gpui::{
    App, ClipboardItem, Context, EventEmitter, FocusHandle, Focusable, Hsla, SharedString,
    Stateful, Task, Window, actions,
};
use release_channel::{AppCommitSha, AppVersion, ReleaseChannel};
use ui::prelude::*;
use util::ResultExt as _;
use workspace::Workspace;
use workspace::item::{Item, ItemEvent};

actions!(
    marley,
    [
        /// Opens the System tab: which build runs, and where Marley keeps its data.
        #[derive(Eq)]
        OpenSystemPages
    ]
);

/// A folder past this many bytes gets the warning colour (D1).
const LARGE_FOLDER_BYTES: u64 = 500 * 1024 * 1024;

/// Registers the open action on every workspace.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenSystemPages, window, cx| {
            open(workspace, window, cx);
        });
    })
    .detach();
}

/// Brings the workspace's System tab forward, or opens one in the center.
fn open(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    let open = workspace.items_of_type::<SystemPages>(cx).next();
    if let Some(tab) = open {
        workspace.activate_item(&tab, true, true, window, cx);
        return;
    }
    let tab = cx.new(SystemPages::new);
    workspace.add_item_to_center(Box::new(tab), window, cx);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Build,
    Data,
}

impl Page {
    const ALL: [Self; 2] = [Self::Build, Self::Data];

    const fn title(self) -> &'static str {
        match self {
            Self::Build => "This build",
            Self::Data => "Where data lives",
        }
    }

    const fn covers(self) -> &'static str {
        match self {
            Self::Build => "Which build · Its data folder",
            Self::Data => "Where data lives",
        }
    }

    const fn lead(self) -> &'static str {
        match self {
            Self::Build => "Which build runs, on which data folder.",
            Self::Data => {
                "Marley's folders, their size, and what each holds. Nothing here leaves the machine."
            }
        }
    }
}

/// One of Marley's folders on Where data lives, and its size once counted.
struct Folder {
    name: &'static str,
    path: PathBuf,
    /// None while counting; zero bytes and `present` false for a folder not made yet.
    bytes: Option<u64>,
    present: bool,
}

/// The folders Where data lives lists, in the order it lists them.
fn folders() -> Vec<(&'static str, PathBuf)> {
    let data = paths::data_dir();
    vec![
        ("Settings", paths::config_dir().clone()),
        ("Database", paths::database_dir().clone()),
        ("Logs", paths::logs_dir().clone()),
        ("Terminal history", data.join("terminal_history")),
        ("Browser profiles", data.join("browser")),
        ("Handoffs", data.join("handoffs")),
        ("Extensions", paths::extensions_dir().clone()),
        ("Language servers", paths::languages_dir().clone()),
    ]
}

/// The bytes of the files under `path`, symlinks not followed; zero for a missing folder.
/// Blocking: it runs on the background executor.
fn folder_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => folder_size(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map_or(0, |metadata| metadata.len()),
            _ => 0,
        })
        .sum()
}

/// `bytes` as the board writes a size: "84 KB", "42 MB", "1.2 GB".
fn size_text(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        0 => "empty".to_string(),
        bytes if bytes < MB => format!("{} KB", bytes.div_ceil(KB)),
        bytes if bytes < GB => format!("{} MB", bytes / MB),
        bytes => format!("{}.{} GB", bytes / GB, (bytes % GB) * 10 / GB),
    }
}

/// The System tab.
pub(crate) struct SystemPages {
    page: Page,
    folders: Vec<Folder>,
    focus_handle: FocusHandle,
    /// The count running or last run; a count asked for while one runs waits for it.
    counting: Option<Task<()>>,
}

impl SystemPages {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut pages = Self {
            page: Page::Build,
            folders: Vec::new(),
            focus_handle: cx.focus_handle(),
            counting: None,
        };
        pages.count(cx);
        pages
    }

    /// Counts every folder's size again (D2).
    fn count(&mut self, cx: &mut Context<Self>) {
        let running = self.folders.iter().any(|folder| folder.bytes.is_none());
        if self.counting.is_some() && running {
            return;
        }
        self.folders = folders()
            .into_iter()
            .map(|(name, path)| Folder {
                name,
                path,
                bytes: None,
                present: true,
            })
            .collect();
        let paths: Vec<PathBuf> = self
            .folders
            .iter()
            .map(|folder| folder.path.clone())
            .collect();
        self.counting = Some(cx.spawn(async move |pages, cx| {
            let sizes = cx
                .background_spawn(futures::future::lazy(move |_| {
                    paths
                        .iter()
                        .map(|path| (path.exists(), folder_size(path)))
                        .collect::<Vec<_>>()
                }))
                .await;
            pages
                .update(cx, |pages, cx| {
                    for (folder, (present, bytes)) in pages.folders.iter_mut().zip(sizes) {
                        folder.present = present;
                        folder.bytes = Some(bytes);
                    }
                    cx.notify();
                })
                .log_err();
        }));
        cx.notify();
    }

    /// This build's rows: name, detail and value.
    fn build_rows(cx: &App) -> Vec<(SharedString, SharedString, SharedString)> {
        let channel = ReleaseChannel::global(cx).display_name();
        let commit = AppCommitSha::try_global(cx).map_or_else(
            || SharedString::new_static("not recorded"),
            |sha| sha.short().into(),
        );
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let platform = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
        vec![
            (
                SharedString::new_static("Channel"),
                channel.into(),
                AppVersion::global(cx).to_string().into(),
            ),
            (
                SharedString::new_static("Commit"),
                commit,
                SharedString::default(),
            ),
            (
                SharedString::new_static("Data folder"),
                paths::data_dir().display().to_string().into(),
                SharedString::default(),
            ),
            (
                SharedString::new_static("Platform"),
                platform.into(),
                desktop.into(),
            ),
        ]
    }

    fn render_list(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .flex_none()
            .w(px(220.))
            .h_full()
            .p_2()
            .gap_0p5()
            .border_r_1()
            .border_color(colors.border_variant)
            .bg(colors.panel_background)
            .child(
                div().px_2().pb_1().child(
                    Label::new("System")
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                ),
            )
            .children(Page::ALL.into_iter().map(|page| {
                let chosen = page == self.page;
                h_flex()
                    .id(SharedString::from(format!(
                        "marley-system-page-{}",
                        page.title()
                    )))
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .when(chosen, |row| row.bg(colors.element_selected))
                    .hover(|style| style.bg(colors.ghost_element_hover))
                    .on_click(cx.listener(move |pages, _, _, cx| {
                        pages.page = page;
                        cx.notify();
                    }))
                    .child(
                        Label::new(page.title())
                            .size(LabelSize::Small)
                            .color(if chosen { Color::Default } else { Color::Muted }),
                    )
            }))
    }

    /// The page's buttons: Copy build info; Open the folder and Count again.
    fn render_buttons(page: Page, cx: &Context<Self>) -> Vec<Stateful<Div>> {
        match page {
            Page::Build => vec![
                crate::prompts::render_button(
                    0,
                    "Copy build info",
                    crate::prompts::Role::Plain,
                    false,
                    cx,
                )
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    let text = Self::build_rows(cx)
                        .into_iter()
                        .map(|(name, detail, value)| {
                            format!("{name}: {detail} {value}").trim_end().to_string()
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                })),
            ],
            Page::Data => vec![
                crate::prompts::render_button(
                    0,
                    "Open the folder",
                    crate::prompts::Role::Plain,
                    false,
                    cx,
                )
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.reveal_path(paths::data_dir());
                })),
                crate::prompts::render_button(
                    1,
                    "Count again",
                    crate::prompts::Role::Quiet,
                    false,
                    cx,
                )
                .cursor_pointer()
                .on_click(cx.listener(|pages, _, _, cx| pages.count(cx))),
            ],
        }
    }

    fn render_page(&self, cx: &Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let status = cx.theme().status();
        let page = self.page;
        let rows: Vec<AnyElement> = match page {
            Page::Build => Self::build_rows(cx)
                .into_iter()
                .map(|(name, detail, value)| {
                    render_row(colors.text_muted, name, detail, value, Color::Muted, cx)
                })
                .collect(),
            Page::Data => self
                .folders
                .iter()
                .map(|folder| {
                    let large = folder.bytes.is_some_and(|bytes| bytes > LARGE_FOLDER_BYTES);
                    let value: SharedString = match folder.bytes {
                        None => SharedString::new_static("counting…"),
                        Some(_) if !folder.present => SharedString::new_static("none yet"),
                        Some(bytes) => size_text(bytes).into(),
                    };
                    let (dot, ink) = if large {
                        (status.warning, Color::Warning)
                    } else {
                        (colors.text_muted, Color::Muted)
                    };
                    render_row(
                        dot,
                        folder.name.into(),
                        folder.path.display().to_string().into(),
                        value,
                        ink,
                        cx,
                    )
                })
                .collect(),
        };
        let buttons = Self::render_buttons(page, cx);
        v_flex()
            .id("marley-system-page")
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .p_6()
            .child(
                v_flex()
                    .max_w(px(760.))
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.elevated_surface_background)
                    .child(
                        h_flex()
                            .gap_3()
                            .items_baseline()
                            .child(Headline::new(page.title()).size(HeadlineSize::Small))
                            .child(
                                Label::new(page.covers())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                    .child(
                        Label::new(page.lead())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    )
                    .child(v_flex().gap_0p5().children(rows))
                    .child(h_flex().gap_2().children(buttons)),
            )
    }
}

/// A page's row as the board draws it: a dot, the name over its detail, and the value at the
/// right in `ink`.
fn render_row(
    dot: Hsla,
    name: SharedString,
    detail: SharedString,
    value: SharedString,
    ink: Color,
    cx: &App,
) -> AnyElement {
    h_flex()
        .gap_3()
        .px_2()
        .py_1p5()
        .rounded_md()
        .bg(cx.theme().colors().surface_background)
        .child(div().flex_none().size(px(7.)).rounded_full().bg(dot))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(Label::new(name).size(LabelSize::Small))
                .child(
                    Label::new(detail)
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .buffer_font(cx)
                        .truncate(),
                ),
        )
        .child(Label::new(value).size(LabelSize::Small).color(ink))
        .into_any_element()
}

impl Focusable for SystemPages {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for SystemPages {}

impl Render for SystemPages {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .key_context("MarleySystemPages")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(self.render_list(cx))
            .child(self.render_page(cx))
    }
}

impl Item for SystemPages {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        SharedString::new_static("System")
    }

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::Info))
    }

    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        Some(SharedString::new_static(
            "Which build runs, and where Marley keeps its data",
        ))
    }

    fn to_item_events(event: &Self::Event, f: &mut dyn FnMut(ItemEvent)) {
        f(*event);
    }

    fn show_toolbar(&self) -> bool {
        false
    }
}
