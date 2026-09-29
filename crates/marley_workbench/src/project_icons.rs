//! A project's own icon on its rail header (#564): the favicon or logo its repository holds, read
//! from the disk and nothing else.
//!
//! The candidates are Orca's fifteen names, each as PNG, SVG, WebP, then ICO, then the icon an
//! `index.html` declares with `<link rel="icon">`. The first that reads, is at most 256 KiB and
//! decodes wins; one that fails is logged by name and the search goes on. The image is made 32
//! px on its long side before it becomes a `RenderImage`, so the atlas never holds a large file.

use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use anyhow::Context as _;
use gpui::RenderImage;
use regex::Regex;

/// The names searched for at a project's root, in order.
const NAMES: &[&str] = &[
    "favicon",
    "public/favicon",
    "app/favicon",
    "app/icon",
    "src/favicon",
    "src/app/icon",
    "assets/favicon",
    "assets/icon",
    "static/favicon",
    "logo",
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
pub(crate) fn icon_in(root: &Path) -> Option<(PathBuf, Arc<RenderImage>)> {
    candidates_in(root).into_iter().find_map(|path| {
        let size = std::fs::metadata(&path).map(|metadata| metadata.len());
        match size {
            Ok(size) if size > MOST_BYTES => {
                log::info!(
                    "project icons: skipped {}: {size} bytes, over {MOST_BYTES}",
                    path.display()
                );
                None
            }
            Ok(_) => match std::fs::read(&path)
                .context("reading it")
                .and_then(|bytes| decode(&path, &bytes))
            {
                Ok(image) => Some((path, Arc::new(image))),
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
    })
}

/// Whether a changed file, relative to the project's root, can change its icon: a candidate name
/// or one of the pages. The rail also watches the file a page declared, which may be named
/// anything.
pub(crate) fn is_candidate(relative: &str) -> bool {
    let relative = relative.trim_start_matches('/');
    PAGES.contains(&relative)
        || relative.rsplit_once('.').is_some_and(|(stem, extension)| {
            NAMES.contains(&stem) && EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}
