//! A page's icon, for its row in the rail (#504).
//!
//! Headless Chromium sends no favicon event, so the icon is read after each load: the page's
//! `link[rel~="icon"]` hrefs, read in an isolated world where the page's scripts cannot see the
//! read (`FAVICON_HREF`). The first http, https or `data:image/` one is taken (`data:,` says
//! the page has none), else `/favicon.ico` at the page's origin. The bytes are loaded through the
//! page's frame, [`MAX_BYTES`] at most, and their format is read from their first bytes
//! ([`format()`]), since a server's content type is often wrong for icons.

use base64::Engine as _;
use gpui::ImageFormat;

/// The largest icon read, in bytes.
pub const MAX_BYTES: usize = 256 * 1024;

/// The script that names the page's icon: the first declared icon a browser would draw, else the
/// origin's `/favicon.ico` for a web page, else nothing.
pub(crate) const FAVICON_HREF: &str = r#"(() => {
  for (const link of document.querySelectorAll('link[rel~="icon" i]')) {
    const href = link.href;
    if (href && href !== 'data:,' && /^(https?:|data:image\/)/i.test(href)) {
      return href;
    }
  }
  if (location.protocol === 'http:' || location.protocol === 'https:') {
    return new URL('/favicon.ico', location.origin).href;
  }
  return null;
})()"#;

/// The image format `bytes` hold, from their first bytes: PNG, ICO, GIF, JPEG, WebP or SVG.
#[must_use]
pub fn format(bytes: &[u8]) -> Option<ImageFormat> {
    let starts = |prefix: &[u8]| bytes.starts_with(prefix);
    if starts(b"\x89PNG\r\n\x1a\n") {
        Some(ImageFormat::Png)
    } else if starts(b"\0\0\x01\0") {
        Some(ImageFormat::Ico)
    } else if starts(b"GIF87a") || starts(b"GIF89a") {
        Some(ImageFormat::Gif)
    } else if starts(b"\xff\xd8\xff") {
        Some(ImageFormat::Jpeg)
    } else if starts(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some(ImageFormat::Webp)
    } else if is_svg(bytes) {
        Some(ImageFormat::Svg)
    } else {
        None
    }
}

/// Whether `bytes` read as an SVG document: `<svg` or `<?xml` after any leading blanks.
fn is_svg(bytes: &[u8]) -> bool {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let head = bytes.get(start..).unwrap_or_default();
    head.starts_with(b"<svg") || head.starts_with(b"<?xml")
}

/// The bytes of a `data:image/…` URL, base64 or percent-encoded, at most [`MAX_BYTES`].
#[must_use]
pub fn data_url_bytes(url: &str) -> Option<Vec<u8>> {
    let rest = url.strip_prefix("data:")?;
    let (header, payload) = rest.split_once(',')?;
    let bytes = if header.ends_with(";base64") {
        base64::engine::general_purpose::STANDARD
            .decode(payload.trim())
            .ok()?
    } else {
        percent_encoding::percent_decode_str(payload).collect()
    };
    (bytes.len() <= MAX_BYTES).then_some(bytes)
}
