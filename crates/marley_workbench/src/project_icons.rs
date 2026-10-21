//! A project's own icon (#564): the logo or favicon its repository holds, read from the disk and
//! nothing else.
//!
//! Since #831 it is the project's mark everywhere Marley draws one (the rail's header and strip,
//! Home and the title bar), kept in one global cache by the project's first folder, with the
//! user's pick from `marley.project_logos` first and the name's first letter last.
//!
//! The candidates are Orca's fifteen names, each as PNG, SVG, WebP, then ICO, then the icon an
//! `index.html` declares with `<link rel="icon">`. The first that reads, is at most 256 KiB and
//! decodes wins; one that fails is logged by name and the search goes on. The image is made 32
//! px on its long side before it becomes a `RenderImage`, so the atlas never holds a large file.

use std::collections::{BTreeMap, HashMap};
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use gpui::{AnyElement, App, FontWeight, Global, Pixels, RenderImage, Task, img};
use regex::Regex;
use settings::{RegisterSetting, Settings, SettingsContent, SettingsStore};
use ui::{Label, LabelSize, prelude::*};

/// The names searched for at a project's root, in order: its logo first (#831).
const NAMES: &[&str] = &[
    "logo",
    "favicon",
    "public/favicon",
    "app/favicon",
    "app/icon",
    "src/favicon",
    "src/app/icon",
    "assets/favicon",
    "assets/icon",
    "static/favicon",
    "public/logo",
    "public/icon",
    "src-tauri/icons/icon",
    "app-icon",
    "icon",
];

/// Each name's extensions, in order.
const EXTENSIONS: &[&str] = &["png", "svg", "webp", "ico"];

/// The pages whose `<link rel="icon">` names an icon.
const PAGES: &[&str] = &["index.html", "public/index.html", "src/index.html"];

/// The largest file an icon is read from.
const MOST_BYTES: u64 = 256 * 1024;

/// The long side of the image an icon becomes, in pixels.
const SIDE: u16 = 32;

static LINK: LazyLock<Option<Regex>> = LazyLock::new(|| compiled(r"(?is)<link\b[^>]*>"));
static REL_ICON: LazyLock<Option<Regex>> =
    LazyLock::new(|| compiled(r#"(?i)\brel\s*=\s*["']?\s*(shortcut\s+)?icon\b"#));
static HREF: LazyLock<Option<Regex>> =
    LazyLock::new(|| compiled(r#"(?i)\bhref\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#));

fn compiled(pattern: &str) -> Option<Regex> {
    Regex::new(pattern)
        .inspect_err(|error| log::error!("project icons: a pattern does not compile: {error}"))
        .ok()
}

/// The files that may be `root`'s icon, in the order they are tried.
fn candidates_in(root: &Path) -> Vec<PathBuf> {
    let named = NAMES.iter().flat_map(|name| {
        EXTENSIONS
            .iter()
            .map(move |extension| root.join(format!("{name}.{extension}")))
    });
    let declared = PAGES.iter().filter_map(|page| {
        let page = root.join(page);
        let html = std::fs::read_to_string(&page).ok()?;
        let href = declared_icon(&html)?;
        let folder = page.parent()?.to_path_buf();
        [folder, root.join("public"), root.to_path_buf()]
            .into_iter()
            .map(|base| base.join(&href))
            .find(|path| path.is_file())
    });
    named
        .filter(|path| path.is_file())
        .chain(declared)
        .collect()
}

/// The icon `html` declares, as a path relative to where it is resolved; `None` for none, or one
/// that is not a file of the project (a scheme, `//`, `..` or `data:`).
fn declared_icon(html: &str) -> Option<String> {
    let (link, rel_icon, href) = (LINK.as_ref()?, REL_ICON.as_ref()?, HREF.as_ref()?);
    link.find_iter(html)
        .map(|tag| tag.as_str())
        .filter(|tag| rel_icon.is_match(tag))
        .find_map(|tag| {
            let captures = href.captures(tag)?;
            let value = captures
                .get(1)
                .or_else(|| captures.get(2))
                .or_else(|| captures.get(3))?
                .as_str();
            let value = value.split(['?', '#']).next()?.trim();
            let outside = value.is_empty()
                || value.contains("://")
                || value.starts_with("//")
                || value.starts_with("data:")
                || value.split('/').any(|part| part == "..");
            (!outside).then(|| value.trim_start_matches('/').to_string())
        })
}

/// `bytes` of the file at `path` as a 32 px image: an SVG rendered at that size, a raster
/// decoded and shrunk to it.
fn decode(path: &Path, bytes: &[u8]) -> anyhow::Result<RenderImage> {
    let svg = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"));
    let rgba = if svg {
        let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())
            .context("the SVG does not parse")?;
        let size = tree.size().to_int_size();
        let target = u32::from(SIDE);
        let long = size.width().max(size.height()).max(1);
        let (width, height) = (
            (size.width() * target / long).max(1),
            (size.height() * target / long).max(1),
        );
        let long = u16::try_from(long).context("the SVG is too large to be an icon")?;
        let scale = f32::from(SIDE) / f32::from(long);
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(width, height).context("the SVG has no size to draw")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );
        let pixels: Vec<u8> = pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let color = pixel.demultiply();
                [color.red(), color.green(), color.blue(), color.alpha()]
            })
            .collect();
        image::RgbaImage::from_raw(width, height, pixels).context("the SVG's pixels")?
    } else {
        image::load_from_memory(bytes)
            .context("the image does not decode")?
            .thumbnail(u32::from(SIDE), u32::from(SIDE))
            .into_rgba8()
    };
    let mut bgra = rgba;
    let (pixels, _) = bgra.as_chunks_mut::<4>();
    for pixel in pixels {
        pixel.swap(0, 2);
    }
    Ok(RenderImage::new([image::Frame::new(bgra)]))
}

/// `root`'s icon and the file it came from: the first candidate that reads within the cap and
/// decodes. Blocking: it runs off the main thread.
fn icon_in(root: &Path) -> Option<(PathBuf, Arc<RenderImage>)> {
    candidates_in(root)
        .into_iter()
        .find_map(|path| read_icon(&path).map(|image| (path, image)))
}

/// The image at `path`, within the cap and decoded; `None`, logged, for one that is not.
/// Blocking.
fn read_icon(path: &Path) -> Option<Arc<RenderImage>> {
    let size = std::fs::metadata(path).map(|metadata| metadata.len());
    match size {
        Ok(size) if size > MOST_BYTES => {
            log::info!(
                "project icons: skipped {}: {size} bytes, over {MOST_BYTES}",
                path.display()
            );
            None
        }
        Ok(_) => match std::fs::read(path)
            .context("reading it")
            .and_then(|bytes| decode(path, &bytes))
        {
            Ok(image) => Some(Arc::new(image)),
            Err(error) => {
                log::info!("project icons: skipped {}: {error:#}", path.display());
                None
            }
        },
        Err(error) => {
            log::info!("project icons: skipped {}: {error}", path.display());
            None
        }
    }
}

/// Whether a changed file, relative to the project's root, can change its icon: a candidate name
/// or one of the pages. The rail also watches the file a page declared, which may be named
/// anything.
fn is_candidate(relative: &str) -> bool {
    let relative = relative.trim_start_matches('/');
    PAGES.contains(&relative)
        || relative.rsplit_once('.').is_some_and(|(stem, extension)| {
            NAMES.contains(&stem) && EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

/// How often a project's mark is searched for at most (#564).
const SEARCH_EVERY: Duration = Duration::from_secs(1);

/// What the user chose for a project's mark (#831).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectLogo {
    /// The name's first letter, whatever the folder holds.
    Letter,
    /// This image file.
    File(PathBuf),
}

/// `marley.project_logos`: the marks the user chose, by the project's first folder (#831).
#[derive(Debug, Clone, PartialEq, Eq, RegisterSetting)]
pub struct ProjectLogos(BTreeMap<PathBuf, ProjectLogo>);

impl Settings for ProjectLogos {
    fn from_settings(content: &SettingsContent) -> Self {
        let logos = content
            .marley
            .as_ref()
            .and_then(|marley| marley.project_logos.as_ref());
        Self(
            logos
                .into_iter()
                .flatten()
                .map(|(folder, logo)| {
                    let logo = if logo.eq_ignore_ascii_case("letter") {
                        ProjectLogo::Letter
                    } else {
                        ProjectLogo::File(PathBuf::from(logo))
                    };
                    (PathBuf::from(folder), logo)
                })
                .collect(),
        )
    }
}

/// Where a mark comes from: the folder's own files, the user's file, or the letter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Source {
    #[default]
    Search,
    Picked(PathBuf),
    Letter,
}

fn source_for(root: &Path, cx: &App) -> Source {
    match ProjectLogos::get_global(cx).0.get(root) {
        Some(ProjectLogo::Letter) => Source::Letter,
        Some(ProjectLogo::File(path)) => Source::Picked(path.clone()),
        None => Source::Search,
    }
}

/// A project's mark as last found.
#[derive(Default)]
struct Entry {
    image: Option<Arc<RenderImage>>,
    /// The file it came from, which a change to searches again.
    chosen: Option<PathBuf>,
    /// The source the last search followed.
    source: Source,
    searched_at: Option<Instant>,
    search: Option<Task<()>>,
    /// Whether another search was asked for while this one ran.
    again: bool,
}

/// Every project's mark, by its first folder (#831): the rail, its strip, Home and the title bar
/// read it, and a view that draws marks observes it.
#[derive(Default)]
pub struct ProjectMarks {
    marks: HashMap<PathBuf, Entry>,
}

impl Global for ProjectMarks {}

impl std::fmt::Debug for ProjectMarks {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProjectMarks")
            .field("projects", &self.marks.len())
            .finish()
    }
}

/// A project's mark.
pub enum ProjectMark {
    /// Its logo or icon, or the file the user picked.
    Image(Arc<RenderImage>),
    /// Its name's first letter on a tint of its own.
    Letter,
}

impl std::fmt::Debug for ProjectMark {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Image(_) => "Image",
            Self::Letter => "Letter",
        })
    }
}

/// Starts the cache, and searches again each mark whose pick in the settings changed.
pub fn init(cx: &mut App) {
    cx.set_global(ProjectMarks::default());
    cx.observe_global::<SettingsStore>(|cx| {
        let changed: Vec<PathBuf> = cx
            .global::<ProjectMarks>()
            .marks
            .iter()
            .filter(|(root, entry)| entry.source != source_for(root, cx))
            .map(|(root, _)| root.clone())
            .collect();
        for root in changed {
            search(&root, cx);
        }
    })
    .detach();
}

/// `root`'s mark as known now; it does not search (see [`want`]).
pub fn mark(root: &Path, cx: &App) -> ProjectMark {
    if source_for(root, cx) == Source::Letter {
        return ProjectMark::Letter;
    }
    cx.try_global::<ProjectMarks>()
        .and_then(|marks| marks.marks.get(root))
        .and_then(|entry| entry.image.clone())
        .map_or(ProjectMark::Letter, ProjectMark::Image)
}

/// Searches for `root`'s mark unless it was searched for already.
pub fn want(root: &Path, cx: &mut App) {
    let known = cx
        .try_global::<ProjectMarks>()
        .is_some_and(|marks| marks.marks.contains_key(root));
    if !known {
        search(root, cx);
    }
}

/// A project's files changed: its mark is searched for again when a changed file can be one, or
/// is the one it shows (#564). `changed` are relative to `root`.
pub fn files_changed<'a>(root: &Path, changed: impl IntoIterator<Item = &'a str>, cx: &mut App) {
    let Some(entry) = cx
        .try_global::<ProjectMarks>()
        .and_then(|marks| marks.marks.get(root))
    else {
        return;
    };
    let chosen = entry
        .chosen
        .as_ref()
        .and_then(|chosen| chosen.strip_prefix(root).ok())
        .map(Path::to_path_buf);
    let touched = changed.into_iter().any(|relative| {
        is_candidate(relative)
            || chosen
                .as_deref()
                .is_some_and(|chosen| chosen == Path::new(relative))
    });
    if touched {
        search(root, cx);
    }
}

/// Reads `root`'s mark off the main thread; a search asked for while one runs, or within a second
/// of the last, waits for its turn.
fn search(root: &Path, cx: &mut App) {
    let root = root.to_path_buf();
    let now = cx.background_executor().now();
    let source = source_for(&root, cx);
    let entry = cx
        .default_global::<ProjectMarks>()
        .marks
        .entry(root.clone())
        .or_default();
    if entry.search.is_some() {
        entry.again = true;
        return;
    }
    let wait = entry
        .searched_at
        .map(|at| SEARCH_EVERY.saturating_sub(now.saturating_duration_since(at)))
        .unwrap_or_default();
    let task = cx.spawn({
        let root = root.clone();
        async move |cx| {
            if !wait.is_zero() {
                cx.background_executor().timer(wait).await;
            }
            let (searched, followed) = (root.clone(), source.clone());
            let found = cx
                .background_spawn(futures::future::lazy(move |_| match followed {
                    Source::Search => icon_in(&searched),
                    Source::Picked(path) => read_icon(&path).map(|image| (path, image)),
                    Source::Letter => None,
                }))
                .await;
            cx.update(|cx| {
                let now = cx.background_executor().now();
                let current = source_for(&root, cx);
                let Some(entry) = cx.global_mut::<ProjectMarks>().marks.get_mut(&root) else {
                    return;
                };
                let (chosen, image) = found.unzip();
                entry.chosen = chosen;
                entry.image = image;
                entry.searched_at = Some(now);
                entry.search = None;
                let again = std::mem::take(&mut entry.again) || current != source;
                entry.source = source;
                if again {
                    search(&root, cx);
                }
            });
        }
    });
    if let Some(entry) = cx.global_mut::<ProjectMarks>().marks.get_mut(&root) {
        entry.search = Some(task);
    }
}

/// Whether the user chose `root`'s mark (#831).
pub fn picked(root: &Path, cx: &App) -> bool {
    ProjectLogos::get_global(cx).0.contains_key(root)
}

/// Writes the user's choice for `root`'s mark to `marley.project_logos`: a file's path, "letter",
/// or `None` to follow the folder's own files again.
pub fn set_logo(root: &Path, logo: Option<String>, cx: &App) {
    let folder = root.to_string_lossy().into_owned();
    settings::update_settings_file(<dyn fs::Fs>::global(cx), cx, move |content, _| {
        let logos = content
            .marley
            .get_or_insert_default()
            .project_logos
            .get_or_insert_default();
        match logo {
            Some(logo) => {
                logos.insert(folder, logo);
            }
            None => {
                logos.remove(&folder);
            }
        }
    });
}

/// Asks for an image file and makes it `root`'s mark.
pub fn choose_logo(root: &Path, cx: &App) {
    let picked = cx.prompt_for_paths(gpui::PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some(SharedString::new_static("Use as the Project's Logo")),
    });
    let root = root.to_path_buf();
    cx.spawn(async move |cx| {
        let Some(path) = picked
            .await
            .context("the file prompt closed")??
            .and_then(|paths| paths.into_iter().next())
        else {
            return anyhow::Ok(());
        };
        cx.update(|cx| set_logo(&root, Some(path.to_string_lossy().into_owned()), cx));
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

/// A name's first letter, uppercase, for a mark with no image.
#[must_use]
pub fn initial(name: &str) -> String {
    name.chars()
        .find(|character| character.is_alphanumeric())
        .map_or_else(
            || "·".to_string(),
            |character| character.to_uppercase().collect(),
        )
}

/// A number from a project's name, so its letter keeps one tint.
fn name_hash(name: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    u32::try_from(hasher.finish() & u64::from(u32::MAX)).unwrap_or_default()
}

/// The theme's tint for `name`'s letter.
pub fn tint(name: &str, cx: &App) -> gpui::Hsla {
    cx.theme()
        .players()
        .color_for_participant(name_hash(name))
        .cursor
}

/// `root`'s mark at `size`: its image, or `name`'s first letter on its tint.
pub fn project_mark(root: &Path, name: &str, size: Pixels, cx: &App) -> AnyElement {
    match mark(root, cx) {
        ProjectMark::Image(image) => img(image)
            .size(size)
            .flex_none()
            .rounded_sm()
            .into_any_element(),
        ProjectMark::Letter => {
            let tint = tint(name, cx);
            let label_size = if size >= px(24.) {
                LabelSize::Large
            } else {
                LabelSize::XSmall
            };
            div()
                .size(size)
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .bg(tint.opacity(0.18))
                .child(
                    Label::new(initial(name))
                        .size(label_size)
                        .weight(FontWeight::SEMIBOLD)
                        .color(Color::Custom(tint)),
                )
                .into_any_element()
        }
    }
}
