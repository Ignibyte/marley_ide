//! Screencast frames into images gpui draws.

use std::sync::Arc;

use anyhow::Context as _;
use base64::Engine as _;
use gpui::RenderImage;
use image::{Frame, ImageFormat};

/// Decodes a screencast frame, a base64 JPEG, into the BGRA image gpui draws.
///
/// # Errors
///
/// When the data is not base64, or not a JPEG.
pub fn decode(data: &str) -> anyhow::Result<Arc<RenderImage>> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("the frame is not base64")?;
    let mut image = image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg)
        .context("the frame is not a JPEG")?
        .into_rgba8();
    // gpui's images are BGRA.
    let (pixels, _) = image.as_chunks_mut::<4>();
    for pixel in pixels {
        pixel.swap(0, 2);
    }
    Ok(Arc::new(RenderImage::new(vec![Frame::new(image)])))
}
