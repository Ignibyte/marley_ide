//! The pickers as the Palette board draws them (#839).
//!
//! In the Marley layout the command palette groups an empty query's commands under headings, and
//! the file finder opens with its preview at the right until the user picks another layout. Both
//! are hooks Zed's crates ask.

use command_palette::{MarleyPaletteGroups, MarleyPaletteLook, MarleyPaletteRow};
use file_finder::{MarleyFinderLook, MarleyFinderMarks};
use gpui::{AnyElement, Entity, FocusHandle, Window};
use gpui::{App, SharedString};
use picker::{MarleyPreviewDefault, PreviewLayout};
use picker_preview::MarleyPreviewHeader;
use project::{Project, ProjectPath};
use ui::prelude::*;

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
    cx.set_global(MarleyFinderLook {
        row: finder_marks,
        hint: finder_hint,
    });
    cx.set_global(MarleyPreviewHeader(preview_header));
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

/// A file row's marks in the Marley layout (#921): the extension in mono on the accent's tint (a
/// file with none keeps its icon, D2), and the git status's letter in its colour, as the project
/// panel orders them (D3). None in Zed's layout (D1).
fn finder_marks(
    project: &Entity<Project>,
    path: &ProjectPath,
    extension: Option<&str>,
    cx: &App,
) -> Option<MarleyFinderMarks> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let colors = cx.theme().colors();
    let tile = extension.map(|extension| {
        let shown: String = extension.chars().take(4).collect();
        div()
            .flex_none()
            .min_w(px(26.))
            .h(px(18.))
            .px_1()
            .rounded(px(4.))
            .bg(colors.text_accent.opacity(0.12))
            .flex()
            .items_center()
            .justify_center()
            .child(
                Label::new(shown)
                    .size(LabelSize::XSmall)
                    .buffer_font(cx)
                    .color(Color::Accent),
            )
            .into_any_element()
    });
    let badge = project
        .read(cx)
        .project_path_git_status(path, cx)
        .and_then(|status| {
            let (letter, color) = if status.is_conflicted() {
                ("!", Color::Conflict)
            } else if status.is_untracked() {
                ("U", Color::Created)
            } else if status.is_deleted() {
                ("D", Color::Deleted)
            } else if status.is_modified() {
                ("M", Color::Modified)
            } else if status.is_created() {
                ("A", Color::Created)
            } else {
                return None;
            };
            Some(
                Label::new(letter)
                    .size(LabelSize::Small)
                    .buffer_font(cx)
                    .color(color)
                    .into_any_element(),
            )
        });
    Some(MarleyFinderMarks { tile, badge })
}

/// The file finder's own key at the field's end in the Marley layout, as the keymap binds it (D5).
fn finder_hint(focus: &FocusHandle, window: &Window, cx: &App) -> Option<AnyElement> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let binding = ui::KeyBinding::for_action_in(&workspace::ToggleFileFinder::default(), focus, cx);
    binding
        .has_binding(window)
        .then(|| binding.size(rems_from_px(11_f32)).into_any_element())
}

/// The preview's header in the Marley layout: the file's path and its line count, grouped by
/// thousands with a space, as the board writes it (#921, D6).
fn preview_header(path: &str, lines: u32, cx: &App) -> Option<AnyElement> {
    if !crate::marley_layout(cx) {
        return None;
    }
    let count = grouped(lines);
    let noun = if lines == 1 { "line" } else { "lines" };
    Some(
        h_flex()
            .w_full()
            .px_3()
            .py_1()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                div().min_w_0().child(
                    Label::new(path.to_string())
                        .size(LabelSize::XSmall)
                        .buffer_font(cx)
                        .color(Color::Muted)
                        .truncate_start(),
                ),
            )
            .child(
                Label::new(format!("\u{b7} {count} {noun}"))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .into_any_element(),
    )
}

/// `count` with a space between each group of three digits: `3 412`.
fn grouped(count: u32) -> String {
    let digits = count.to_string();
    let mut shown = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            shown.push(' ');
        }
        shown.push(digit);
    }
    shown
}

/// The file finder's preview at the right in the Marley layout; Zed's default elsewhere.
fn preview_default(picker: &'static str, cx: &App) -> Option<PreviewLayout> {
    (picker == FILE_FINDER && crate::marley_layout(cx)).then_some(PreviewLayout::Right)
}
