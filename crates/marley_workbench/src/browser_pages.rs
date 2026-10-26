//! `browser_show_page` (#802): an agent's HTML page in a Browser tab of its project.
//!
//! A page the agent hands over is written to its project's scratch folder under Marley's data
//! directory; an HTML file of the project opens as it is, and nothing is written into the
//! project. A tab the tool opened loads its page again when the file changes. Beside the scratch
//! folders sits a stylesheet of the active theme's colors and Marley's fonts as CSS variables,
//! linked into a written page's head, so a page that uses them looks like Marley and follows a
//! theme change.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fs::Fs;
use futures::StreamExt as _;
use gpui::{App, AsyncApp, Entity, Global, Hsla, Rgba, TaskExt as _, Window};
use marley_browser::address;
use marley_mcp::ToolAnswer;
use serde_json::{Value, json};
use settings::{Settings as _, SettingsStore};
use theme::{ActiveTheme as _, Appearance};
use theme_settings::ThemeSettings;
use workspace::{MultiWorkspace, Workspace};

use crate::browser::{BrowserProject, reload_tabs_on};
use crate::browser_tools::{declined, open_answer, open_in_workspace};

/// The stylesheet's name, in the pages folder beside the projects' scratch folders.
const STYLESHEET: &str = "marley-theme.css";

/// How long the answer waits for a new tab's page to attach, for its id.
const ATTACH_WAIT: Duration = Duration::from_secs(5);

/// How often it looks.
const POLL: Duration = Duration::from_millis(100);

/// How long a file's changes gather before its tabs load it again: an agent's write is often
/// several events.
const LATENCY: Duration = Duration::from_millis(150);

/// The longest page name.
const NAME_LIMIT: usize = 100;

/// The page files whose tabs follow them, and the stylesheet as it was last written.
#[derive(Default)]
struct ShownPages {
    /// Each file a watch follows.
    watched: HashSet<PathBuf>,
    /// None until the tool first writes the stylesheet: no theme change writes it before.
    stylesheet: Option<String>,
}

impl Global for ShownPages {}

/// The folder of the tool's pages and their stylesheet under `data_dir`.
fn pages_dir_in(data_dir: &Path) -> PathBuf {
    data_dir.join("pages")
}

/// Rewrites the stylesheet when the theme or the fonts change, once the tool has written it.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(ShownPages::default());
    cx.observe_global::<theme::GlobalTheme>(restyle).detach();
    cx.observe_global::<SettingsStore>(restyle).detach();
}

/// `browser_show_page`: the page from `arguments` in a Browser tab of `home`, the caller's
/// workspace.
pub(crate) async fn show_page(
    arguments: &Value,
    home: Option<Entity<Workspace>>,
    cx: &AsyncApp,
) -> ToolAnswer {
    let Some(workspace) = home else {
        return declined("you run in no project of Marley's");
    };
    let text = |name: &str| arguments.get(name).and_then(Value::as_str);
    let fs = cx.update(|cx| <dyn Fs>::global(cx));
    let stylesheet = match write_stylesheet(cx).await {
        Ok(path) => path,
        Err(error) => {
            log::error!("browser_show_page: the theme stylesheet: {error:#}");
            return declined("Marley could not write the theme stylesheet");
        }
    };
    let Ok(stylesheet_url) = url::Url::from_file_path(&stylesheet) else {
        return declined("Marley could not write the theme stylesheet");
    };
    let page = match (text("html"), text("path")) {
        (Some(_), Some(_)) => return declined("send `html` and `name`, or `path`, not both"),
        (None, None) => return declined("send `html` and `name`, or `path`"),
        (Some(html), None) => {
            let Some(name) = text("name") else {
                return declined("send a `name` with `html`");
            };
            let file_name = match page_file_name(name) {
                Ok(file_name) => file_name,
                Err(reason) => return declined(reason),
            };
            let key = cx.update(|cx| BrowserProject::of(workspace.read(cx).project(), cx).key);
            let folder = pages_dir_in(paths::data_dir()).join(key.as_ref());
            let themed = arguments.get("theme").and_then(Value::as_bool) != Some(false);
            let html = if themed {
                with_stylesheet(html, stylesheet_url.as_str())
            } else {
                html.to_string()
            };
            let file = folder.join(file_name);
            let written = async {
                fs.create_dir(&folder).await?;
                fs.atomic_write(file.clone(), html).await
            };
            if let Err(error) = written.await {
                log::error!("browser_show_page: {}: {error:#}", file.display());
                return declined("Marley could not write the page");
            }
            file
        }
        (None, Some(path)) => {
            let roots = cx.update(|cx| workspace.read(cx).root_paths(cx));
            match project_page(path, &roots, fs.as_ref()).await {
                Ok(file) => file,
                Err(reason) => return declined(reason),
            }
        }
    };
    let Ok(url) = url::Url::from_file_path(&page) else {
        return declined("not a local HTML page");
    };
    let opened = cx.update(|cx| {
        let opened = open_in_workspace(url.to_string(), &workspace, cx);
        if opened.is_ok() {
            watch(page.clone(), url, cx);
        }
        opened
    });
    let (project, view) = match opened {
        Ok(opened) => opened,
        Err(reason) => return declined(reason),
    };
    let mut answer = json!({
        "opened": true,
        "project": project,
        "path": page.to_string_lossy(),
        "stylesheet": stylesheet_url.as_str(),
    });
    let mut waited = Duration::ZERO;
    loop {
        if let Some(tab) = view.read_with(cx, |view, _| view.target().map(str::to_string)) {
            answer["tab"] = json!(tab);
            break;
        }
        if waited >= ATTACH_WAIT {
            break;
        }
        cx.background_executor().timer(POLL).await;
        waited += POLL;
    }
    open_answer(answer)
}

/// Brings forward the Browser tab of the window whose page is the file `file_name`, showing its
/// project first (#927): a page card's Show. Nothing happens when no tab shows it.
pub(crate) fn show_tab_of(file_name: &str, window: &mut Window, cx: &mut App) {
    let Some(multi_workspace) = window.root::<MultiWorkspace>().flatten() else {
        return;
    };
    let workspaces: Vec<Entity<Workspace>> =
        multi_workspace.read(cx).workspaces().cloned().collect();
    for workspace in workspaces {
        let shown = workspace.update(cx, |workspace, cx| {
            crate::browser::show_tab_where(
                workspace,
                |address| {
                    address.scheme() == "file"
                        && address
                            .path_segments()
                            .and_then(|mut segments| segments.next_back())
                            == Some(file_name)
                },
                window,
                cx,
            )
            .is_some()
        });
        if shown {
            multi_workspace.update(cx, |multi_workspace, cx| {
                multi_workspace.activate(workspace, None, window, cx);
            });
            return;
        }
    }
}

/// The file name of the page `name`: `<name>.html`, `name` without its own `.html`, of
/// letters, digits, `-`, `_` and `.`, not starting with a dot.
pub(crate) fn page_file_name(name: &str) -> Result<String, &'static str> {
    let name = name.trim();
    let stem = name
        .strip_suffix(".html")
        .or_else(|| name.strip_suffix(".htm"))
        .unwrap_or(name);
    let allowed =
        |character: char| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.');
    if stem.is_empty() || stem.len() > NAME_LIMIT || stem.starts_with('.') {
        return Err("a page `name` is 1 to 100 characters and does not start with a dot");
    }
    if !stem.chars().all(allowed) {
        return Err("a page `name` holds letters, digits, `-`, `_` and `.` only");
    }
    Ok(format!("{stem}.html"))
}

/// The HTML file `path` names, an absolute path or a `file:` URL or a path from the project's
/// first folder, once it is a file inside one of `roots` after links are followed.
async fn project_page(
    path: &str,
    roots: &[Arc<Path>],
    fs: &dyn Fs,
) -> Result<PathBuf, &'static str> {
    let Some(first) = roots.first() else {
        return Err("your project has no folder");
    };
    let page = address::local_page(path, first).ok_or("not an .html or .htm file")?;
    let page = fs
        .canonicalize(&page)
        .await
        .map_err(|_| "no such file in your project")?;
    let mut inside = false;
    for root in roots {
        if fs
            .canonicalize(root)
            .await
            .is_ok_and(|root| page.starts_with(root))
        {
            inside = true;
            break;
        }
    }
    if !inside {
        return Err("the file is outside your project");
    }
    let file = fs
        .metadata(&page)
        .await
        .ok()
        .flatten()
        .is_some_and(|metadata| !metadata.is_dir && !metadata.is_fifo);
    if file {
        Ok(page)
    } else {
        Err("not an .html or .htm file")
    }
}

/// `html` with a `<link>` to the stylesheet at `href`: in its `<head>`, else after its `<html>`
/// tag, else after its doctype, else first. Text already naming the stylesheet keeps its own.
fn with_stylesheet(html: &str, href: &str) -> String {
    if html.contains(STYLESHEET) {
        return html.to_string();
    }
    let link = format!("<link rel=\"stylesheet\" href=\"{href}\">");
    // ASCII lowercasing keeps every byte where it was, so an index into it is one into `html`.
    let lower = html.to_ascii_lowercase();
    let after_tag = |tag: &str| {
        lower.match_indices(tag).find_map(|(index, _)| {
            let next = lower.as_bytes().get(index + tag.len())?;
            if !matches!(next, b'>' | b' ' | b'\t' | b'\n' | b'\r') {
                return None;
            }
            Some(index + lower.get(index..)?.find('>')? + 1)
        })
    };
    let at = after_tag("<head")
        .or_else(|| after_tag("<html"))
        .or_else(|| after_tag("<!doctype"))
        .unwrap_or(0);
    match (html.get(..at), html.get(at..)) {
        (Some(before), Some(after)) => format!("{before}{link}{after}"),
        _ => format!("{link}{html}"),
    }
}

/// Follows `file` for the rest of the session, through its folder since a write by rename
/// replaces the file: each change loads again the tabs on `url`. A watch that ended when no tab
/// showed the file would race a call that opens a new tab on it as it ends.
fn watch(file: PathBuf, url: url::Url, cx: &mut App) {
    if !cx
        .default_global::<ShownPages>()
        .watched
        .insert(file.clone())
    {
        return;
    }
    let fs = <dyn Fs>::global(cx);
    cx.spawn(async move |cx| {
        if let Some(folder) = file.parent() {
            let (events, _watcher) = fs.watch(folder, LATENCY).await;
            futures::pin_mut!(events);
            while let Some(events) = events.next().await {
                if events.iter().any(|event| event.path == file) {
                    cx.update(|cx| reload_tabs_on(&url, cx));
                }
            }
        }
    })
    .detach();
}

/// Writes the stylesheet when the theme changed it, then loads again the tabs of every page a
/// watch follows.
fn restyle(cx: &mut App) {
    let Some(written) = cx
        .try_global::<ShownPages>()
        .and_then(|pages| pages.stylesheet.clone())
    else {
        return;
    };
    if written == stylesheet_text(cx) {
        return;
    }
    cx.spawn(async move |cx| {
        write_stylesheet(cx).await?;
        cx.update(|cx| {
            let pages: Vec<PathBuf> = cx
                .default_global::<ShownPages>()
                .watched
                .iter()
                .cloned()
                .collect();
            for page in pages {
                if let Ok(url) = url::Url::from_file_path(&page) {
                    reload_tabs_on(&url, cx);
                }
            }
        });
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

/// Writes the stylesheet of the theme as it is now, when its text changed, and gives its path.
async fn write_stylesheet(cx: &AsyncApp) -> anyhow::Result<PathBuf> {
    let (fs, text, unchanged) = cx.update(|cx| {
        let text = stylesheet_text(cx);
        let unchanged = cx.default_global::<ShownPages>().stylesheet.as_ref() == Some(&text);
        (<dyn Fs>::global(cx), text, unchanged)
    });
    let folder = pages_dir_in(paths::data_dir());
    let path = folder.join(STYLESHEET);
    if !unchanged || !fs.is_file(&path).await {
        fs.create_dir(&folder).await?;
        fs.atomic_write(path.clone(), text.clone()).await?;
        cx.update(|cx| cx.default_global::<ShownPages>().stylesheet = Some(text));
    }
    Ok(path)
}

/// The stylesheet: the active theme's colors and Marley's UI and monospace fonts as CSS
/// variables, with the theme's color scheme.
fn stylesheet_text(cx: &App) -> String {
    let theme = cx.theme();
    let colors = theme.colors();
    let status = theme.status();
    let settings = ThemeSettings::get_global(cx);
    let scheme = match theme.appearance() {
        Appearance::Light => "light",
        Appearance::Dark => "dark",
    };
    let variables = [
        ("background", colors.editor_background),
        ("surface", colors.surface_background),
        ("text", colors.text),
        ("text-muted", colors.text_muted),
        ("border", colors.border),
        ("accent", colors.text_accent),
        ("success", status.success),
        ("warning", status.warning),
        ("error", status.error),
    ];
    let mut lines = vec![
        format!(
            "/* Marley's theme ({}), written by Marley for browser_show_page. */",
            theme.name
        ),
        ":root {".to_string(),
        format!("  color-scheme: {scheme};"),
    ];
    lines.extend(
        variables
            .iter()
            .map(|(name, color)| format!("  --marley-{name}: {};", css_color(*color))),
    );
    lines.push(format!(
        "  --marley-font: \"{}\", system-ui, sans-serif;",
        css_family(&settings.ui_font.family)
    ));
    lines.push(format!(
        "  --marley-mono-font: \"{}\", ui-monospace, monospace;",
        css_family(&settings.buffer_font.family)
    ));
    lines.push("}\n".to_string());
    lines.join("\n")
}

/// `color` as CSS's `rgb()`, in percentages so no channel is rounded to a byte.
fn css_color(color: Hsla) -> String {
    let color = Rgba::from(color);
    format!(
        "rgb({:.1}% {:.1}% {:.1}% / {:.3})",
        color.r * 100.0,
        color.g * 100.0,
        color.b * 100.0,
        color.a
    )
}

/// A font family name as the inside of a CSS string: Zed's own names (`.ZedSans`) as the fonts
/// they stand for, without the quotes and backslashes that would end or escape the string.
fn css_family(family: &str) -> String {
    gpui::font_name_with_fallbacks(family, "system-ui")
        .chars()
        .filter(|character| !matches!(character, '"' | '\\' | '\n'))
        .collect()
}
