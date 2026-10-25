//! The pickers as the Palette board draws them (#839).
//!
//! In the Marley layout the command palette groups an empty query's commands under headings, and
//! the file finder opens with its preview at the right until the user picks another layout. Both
//! are hooks Zed's crates ask.

use command_palette::{MarleyPaletteGroups, MarleyPaletteLook, MarleyPaletteRow};
use gpui::{App, SharedString};
use picker::{MarleyPreviewDefault, PreviewLayout};

/// The file finder's delegate name, under which the picker keeps its layout.
const FILE_FINDER: &str = "file finder";

/// Sets the two hooks. [`crate::init`] calls it once.
pub(crate) fn init(cx: &mut App) {
    cx.set_global(MarleyPaletteGroups(crate::marley_layout));
    cx.set_global(MarleyPaletteLook {
        row: palette_row,
        help: &["marley: open guide", "zed: open keymap"],
    });
    cx.set_global(MarleyPreviewDefault(preview_default));
}

/// A command the Palette board draws with its own name and glyph (#920, D2, D3).
struct PaletteName {
    /// Zed's name for the command, as the palette matches it.
    zed: &'static str,
    plain: &'static str,
    glyph: &'static str,
}

/// The board's names, and those the rule reads wrong.
const PALETTE_NAMES: &[PaletteName] = &[
    PaletteName {
        zed: "marley: open browser",
        plain: "Open Browser",
        glyph: "www",
    },
    PaletteName {
        zed: "marley: open threads",
        plain: "Open Threads",
        glyph: "Th",
    },
    PaletteName {
        zed: "marley: open mcp servers",
        plain: "Open MCP Servers",
        glyph: "Mc",
    },
    PaletteName {
        zed: "marley: open component library",
        plain: "Open Component Library",
        glyph: "Cl",
    },
    PaletteName {
        zed: "workspace: open",
        plain: "Open Folder\u{2026}",
        glyph: "..",
    },
    PaletteName {
        zed: "zed: open settings",
        plain: "Open Settings",
        glyph: "St",
    },
    PaletteName {
        zed: "zed: open keymap",
        plain: "Open Keymap",
        glyph: "Km",
    },
    PaletteName {
        zed: "zed: open log",
        plain: "Open Log",
        glyph: "Lg",
    },
    PaletteName {
        zed: "marley: open guide",
        plain: "Open Guide",
        glyph: "?",
    },
];

/// The palette's row for the command Zed names `zed_name` in the Marley layout (#920): the board's
/// name and glyph where it draws one; otherwise the name past its namespace with each word
/// capitalized, and the first two letters of its second word, or of its only word. Marley's own
/// commands have their tile in the accent (D4). None in Zed's layout (D5).
fn palette_row(zed_name: &str, cx: &App) -> Option<MarleyPaletteRow> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let (namespace, rest) = zed_name.split_once(": ")?;
    let ours = namespace.starts_with("marley") || namespace == "rusty";
    if let Some(known) = PALETTE_NAMES.iter().find(|known| known.zed == zed_name) {
        return Some(MarleyPaletteRow {
            name: SharedString::new_static(known.plain),
            glyph: SharedString::new_static(known.glyph),
            ours,
        });
    }
    let words: Vec<String> = rest.split_whitespace().map(capitalized).collect();
    let source = words.get(1).or_else(|| words.first())?;
    let glyph: String = source.chars().take(2).collect();
    Some(MarleyPaletteRow {
        name: words.join(" ").into(),
        glyph: glyph.into(),
        ours,
    })
}

/// `word` with its first letter capitalized.
fn capitalized(word: &str) -> String {
    let mut characters = word.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(characters).collect()
    })
}

/// The file finder's preview at the right in the Marley layout; Zed's default elsewhere.
fn preview_default(picker: &'static str, cx: &App) -> Option<PreviewLayout> {
    (picker == FILE_FINDER && crate::marley_layout(cx)).then_some(PreviewLayout::Right)
}
