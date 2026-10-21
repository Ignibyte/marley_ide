//! The pickers as the Palette board draws them (#839).
//!
//! In the Marley layout the command palette groups an empty query's commands under headings, and
//! the file finder opens with its preview at the right until the user picks another layout. Both
//! are hooks Zed's crates ask.

use command_palette::MarleyPaletteGroups;
use gpui::App;
use picker::{MarleyPreviewDefault, PreviewLayout};

/// The file finder's delegate name, under which the picker keeps its layout.
const FILE_FINDER: &str = "file finder";

/// Sets the two hooks. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyPaletteGroups(crate::marley_layout));
    cx.set_global(MarleyPreviewDefault(preview_default));
}

/// The file finder's preview at the right in the Marley layout; Zed's default elsewhere.
fn preview_default(picker: &'static str, cx: &App) -> Option<PreviewLayout> {
    (picker == FILE_FINDER && crate::marley_layout(cx)).then_some(PreviewLayout::Right)
}
