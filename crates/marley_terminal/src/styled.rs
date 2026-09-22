//! PURE, gpui-free — the styled-output model (SPEC-terminal-blocks R20b).
//!
//! A command's output is held as [`StyledLine`]s: runs of adjacent grid cells that share the same
//! foreground/background/flags, carrying alacritty's own `Color`/`Flags` (NOT gpui `Hsla` — this
//! crate stays gpui-free; the `Color → Hsla` mapping + paint live up in `marley_app`). The plain
//! [`Block::output_text`](crate::block::Block::output_text) is a flattened view of these runs.

use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::Color;
#[cfg(test)]
use alacritty_terminal::vte::ansi::NamedColor;

/// One run of same-styled cells: the run's chars plus the shared fg/bg/flags and the OSC 8 hyperlink
/// URI carried by the run's cells (#214).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledRun {
    /// The run's characters, in column order.
    pub text: String,
    /// The shared foreground color.
    pub fg: Color,
    /// The shared background color.
    pub bg: Color,
    /// The shared cell flags (bold / inverse / …).
    pub flags: Flags,
    /// The shared OSC 8 explicit-hyperlink target URI (#214), or `None` when the cells are not part
    /// of a hyperlink. A run breaks on a hyperlink change, so every cell in a run shares this URI.
    pub hyperlink: Option<String>,
}

/// One screen line: its coalesced runs, left to right (empty when the line is blank).
pub type StyledLine = Vec<StyledRun>;

/// Coalesce a row's cells into maximal runs sharing the same `(fg, bg, flags, hyperlink)`, then
/// trailing-trim so the joined run text equals the row's `trim_end()` — keeping
/// [`Block::output_text`](crate::block::Block::output_text) byte-identical to the pre-color model
/// (R20b): a hyperlink change splits one run into two but never changes the joined text. A fully
/// blank line coalesces to an empty `Vec`. Each cell carries its OSC 8 hyperlink URI (`None` for a
/// non-hyperlinked cell) so a run breaks on a hyperlink change too, not just a style change (#214).
pub fn coalesce_row(
    cells: impl IntoIterator<Item = (char, Color, Color, Flags, Option<String>)>,
) -> StyledLine {
    let mut runs: Vec<StyledRun> = Vec::new();
    for (c, fg, bg, flags, hyperlink) in cells {
        match runs.last_mut() {
            Some(run)
                if run.fg == fg
                    && run.bg == bg
                    && run.flags == flags
                    && run.hyperlink == hyperlink =>
            {
                run.text.push(c)
            }
            _ => runs.push(StyledRun {
                text: c.to_string(),
                fg,
                bg,
                flags,
                hyperlink,
            }),
        }
    }
    // Trailing-trim equivalent to `str::trim_end` across run boundaries: drop trailing all-blank
    // runs, then trim the trailing whitespace off the new last run.
    while let Some(last) = runs.last_mut() {
        let trimmed = last.text.trim_end();
        if trimmed.is_empty() {
            runs.pop();
        } else if trimmed.len() != last.text.len() {
            last.text.truncate(trimmed.len());
            break;
        } else {
            break;
        }
    }
    runs
}

/// Drop the TRAILING all-blank rows from a block's captured output (R19). The grid snapshot is
/// screen-height, so a short command leaves many empty rows below its output; trimming them keeps a
/// block only as tall as its real output, so command blocks STACK as scrollback instead of one
/// full-screen block per command. Interior blank rows are KEPT (only the trailing empties are
/// dropped); an all-blank or empty input yields `[]`. NOT applied to the alt-screen grid
/// (`grid_styled_rows`), where a full-screen program owns its whole screen.
pub fn trim_trailing_blank_rows(mut rows: Vec<StyledLine>) -> Vec<StyledLine> {
    while rows
        .last()
        .is_some_and(|row| row.iter().all(|run| run.text.trim().is_empty()))
    {
        rows.pop();
    }
    rows
}

/// Test helper: build default-styled (foreground-on-background, no flags) lines from plain strings
/// — a non-empty string becomes one run, an empty string an empty line. Models what the old
/// `Vec<String>` output held, so existing `output_text` tests keep asserting the same plain text.
#[cfg(test)]
pub(crate) fn plain_lines(lines: &[&str]) -> Vec<StyledLine> {
    lines
        .iter()
        .map(|s| {
            if s.is_empty() {
                Vec::new()
            } else {
                vec![StyledRun {
                    text: (*s).to_string(),
                    fg: Color::Named(NamedColor::Foreground),
                    bg: Color::Named(NamedColor::Background),
                    flags: Flags::empty(),
                    hyperlink: None,
                }]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Color {
        Color::Named(NamedColor::Red)
    }
    fn green() -> Color {
        Color::Named(NamedColor::Green)
    }
    fn fg() -> Color {
        Color::Named(NamedColor::Foreground)
    }
    fn bg() -> Color {
        Color::Named(NamedColor::Background)
    }

    // ── R19 — trim_trailing_blank_rows: drop TRAILING blanks, KEEP interior (blocks stack) ──
    #[test]
    fn trim_trailing_blank_rows_drops_trailing_keeps_interior() {
        // The interior blank between "a" and "b" survives; the two trailing blanks drop.
        assert_eq!(
            trim_trailing_blank_rows(plain_lines(&["a", "", "b", "", ""])),
            plain_lines(&["a", "", "b"])
        );
        // Whitespace-only trailing rows are blank → dropped (a run whose text trims to empty).
        assert_eq!(
            trim_trailing_blank_rows(plain_lines(&["a", "  ", "\t"])),
            plain_lines(&["a"])
        );
    }

    #[test]
    fn trim_trailing_blank_rows_edges() {
        // All-blank → empty; empty → empty; no trailing blank → unchanged.
        assert_eq!(
            trim_trailing_blank_rows(plain_lines(&["", ""])),
            Vec::<StyledLine>::new()
        );
        assert_eq!(
            trim_trailing_blank_rows(Vec::<StyledLine>::new()),
            Vec::<StyledLine>::new()
        );
        assert_eq!(
            trim_trailing_blank_rows(plain_lines(&["a", "b"])),
            plain_lines(&["a", "b"])
        );
    }

    // ── R20b — run partitioning: merge same-style, split on ANY of fg/bg/flags ──────
    #[test]
    fn coalesce_row_merges_same_style_and_splits_on_change() {
        // Adjacent same-(fg,bg,flags) cells MERGE into one run.
        let same = coalesce_row([
            ('a', red(), bg(), Flags::empty(), None),
            ('b', red(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(same.len(), 1);
        assert_eq!(same[0].text, "ab");
        assert_eq!(same[0].fg, red());

        // A FG change splits — assert each run's fg (kills the `run.fg == fg` guard + `&&`→`||`).
        let fg_split = coalesce_row([
            ('a', red(), bg(), Flags::empty(), None),
            ('b', green(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(fg_split.len(), 2);
        assert_eq!((fg_split[0].text.as_str(), fg_split[0].fg), ("a", red()));
        assert_eq!((fg_split[1].text.as_str(), fg_split[1].fg), ("b", green()));

        // A BG-only change splits (kills the `run.bg == bg` guard).
        let bg_split = coalesce_row([
            ('a', fg(), red(), Flags::empty(), None),
            ('b', fg(), green(), Flags::empty(), None),
        ]);
        assert_eq!(bg_split.len(), 2);
        assert_eq!((bg_split[0].bg, bg_split[1].bg), (red(), green()));

        // A FLAGS-only change splits (kills the `run.flags == flags` guard).
        let flags_split = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), None),
            ('b', fg(), bg(), Flags::BOLD, None),
        ]);
        assert_eq!(flags_split.len(), 2);
        assert_eq!(flags_split[0].flags, Flags::empty());
        assert_eq!(flags_split[1].flags, Flags::BOLD);
    }

    // ── R20b — trailing-trim == str::trim_end (keeps output_text byte-identical) ─────
    #[test]
    fn coalesce_row_trailing_trims_across_runs() {
        // Trailing spaces in the SAME run as content are truncated.
        let same_run = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(same_run.len(), 1);
        assert_eq!(same_run[0].text, "a");

        // Trailing spaces in a SEPARATE style run are dropped, then the prior run trims —
        // cross-run-boundary trim: "x "(red) + "  "(default) → "x".
        let cross = coalesce_row([
            ('x', red(), bg(), Flags::empty(), None),
            (' ', red(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(cross.len(), 1);
        assert_eq!((cross[0].text.as_str(), cross[0].fg), ("x", red()));

        // Interior spaces are preserved; a mid space in one style run stays.
        let interior = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
            ('b', fg(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(interior.len(), 1);
        assert_eq!(interior[0].text, "a b");

        // A fully-blank line coalesces to an empty Vec.
        let blank = coalesce_row([
            (' ', fg(), bg(), Flags::empty(), None),
            (' ', fg(), bg(), Flags::empty(), None),
        ]);
        assert!(blank.is_empty());

        // Empty input → empty.
        assert!(coalesce_row([]).is_empty());
    }

    // ── R20b — output_text (via the flattening) equals raw.trim_end across a fixture ──
    #[test]
    fn coalesced_runs_flatten_to_trim_end() {
        // The load-bearing byte-identity: concat(runs) == raw.trim_end().
        let raw = "ab cd  ";
        let cells = raw
            .chars()
            .map(|c| (c, fg(), bg(), Flags::empty(), None))
            .collect::<Vec<_>>();
        let runs = coalesce_row(cells);
        let flat: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(flat, raw.trim_end());
        assert_eq!(flat, "ab cd");
    }

    // ── #214 — coalesce_row carries the OSC 8 hyperlink + breaks a run on a hyperlink change; the
    // flattened text stays byte-identical (R20b/D4). Asserts the hyperlink VALUE (inspect F2 — the
    // field-carry has no mutant, so pin it by value), not just the run count. ──
    #[test]
    fn coalesce_row_splits_on_hyperlink_change() {
        let u = || Some("u".to_string());
        // Same style + same URI → ONE run, carrying the URI (kills `== hyperlink`→`!=` via a merge).
        let same = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), u()),
            ('b', fg(), bg(), Flags::empty(), u()),
        ]);
        assert_eq!(same.len(), 1);
        assert_eq!(same[0].text, "ab");
        assert_eq!(same[0].hyperlink, u()); // carry (F2)
        // Same style, DIFFERENT URI → TWO runs; joined text byte-identical (D4/#50).
        let diff = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), Some("u".to_string())),
            ('b', fg(), bg(), Flags::empty(), Some("v".to_string())),
        ]);
        assert_eq!(diff.len(), 2);
        assert_eq!(
            (diff[0].hyperlink.as_deref(), diff[1].hyperlink.as_deref()),
            (Some("u"), Some("v"))
        );
        assert_eq!(
            diff.iter().map(|r| r.text.as_str()).collect::<String>(),
            "ab"
        );
        // Some → None boundary → TWO runs; joined text unchanged (kills the `!=` on the None side).
        let boundary = coalesce_row([
            ('a', fg(), bg(), Flags::empty(), u()),
            ('b', fg(), bg(), Flags::empty(), None),
        ]);
        assert_eq!(boundary.len(), 2);
        assert_eq!(
            (
                boundary[0].hyperlink.as_deref(),
                boundary[1].hyperlink.clone()
            ),
            (Some("u"), None)
        );
        assert_eq!(
            boundary.iter().map(|r| r.text.as_str()).collect::<String>(),
            "ab"
        );
    }
}
