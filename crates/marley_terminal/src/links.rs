//! URLs a program wrapped itself, joined across the rows it put them on (#579).
//!
//! The terminal joins a URL that runs past the right edge when the terminal wrapped it: the row
//! carries alacritty's `WRAPLINE`, which the link search follows. A program can wrap a URL itself,
//! with a newline where the edge is, or draw it inside a box of `│` frames a row at a time; to the
//! terminal those are rows that end, and the URL would open cut in two. [`joined_url`] finds such
//! a URL from the rows around a point, by the rules Orca's terminal keeps (`edge-wrapped` and
//! `hard-wrapped-terminal-http-links`, MIT, reimplemented):
//!
//! - **At the edge**, a row that fills to the last column runs on into the next unless the next
//!   starts with a space, a new `http://` or `https://`, or a `label:` or `HTTP/2:` form, or holds
//!   a frame.
//! - **In a frame**, rows join when they have the frame at the same two columns and the same text
//!   before the URL's column, while each part ends in `/ ? & = # % + : -` or fills most of the
//!   width between.
//!
//! A URL of one row is the terminal's own search's, and so is a row the terminal wrapped.

/// The characters a box frame draws its sides with.
const FRAMES: &[char] = &['│', '┃', '║', '╎', '╏', '┆', '┇', '┊', '┋', '|'];

/// The longest URL joined, in characters.
const MAX_URL: usize = 2048;

/// The characters a framed URL's part may end in and still run on.
const FRAMED_CONTINUES: &[char] = &['/', '?', '&', '=', '#', '%', '+', ':', '-'];

/// How much of the width between its frames a part fills for the URL to run on, in percent.
const FRAMED_FILL_PERCENT: usize = 80;

/// One row of the grid as text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkRow {
    /// The row's characters, a wide one once and its spacer left out, trailing blanks trimmed.
    pub text: Vec<char>,
    /// The column of each character.
    pub columns: Vec<usize>,
    /// Whether the terminal wrapped the row into the next (alacritty's `WRAPLINE`).
    pub wrapped: bool,
}

impl LinkRow {
    /// The row from its cells' characters, a wide one's spacer given as `None`.
    #[must_use]
    pub fn from_cells(cells: impl IntoIterator<Item = Option<char>>, wrapped: bool) -> Self {
        let mut row = Self {
            wrapped,
            ..Self::default()
        };
        for (column, cell) in cells.into_iter().enumerate() {
            if let Some(character) = cell {
                row.text.push(character);
                row.columns.push(column);
            }
        }
        let used = row
            .text
            .iter()
            .rposition(|character| !character.is_whitespace())
            .map_or(0, |end| end + 1);
        row.text.truncate(used);
        row.columns.truncate(used);
        row
    }

    fn column(&self, index: usize) -> Option<usize> {
        self.columns.get(index).copied()
    }

    /// The index of the character at `column`, if one starts there.
    fn index_at(&self, column: usize) -> Option<usize> {
        self.columns.iter().position(|&start| start == column)
    }

    /// Whether the row's text reaches `last_column`, or stops one short of a wide character that
    /// starts the next row.
    fn fills_to(&self, last_column: usize, next: Option<&Self>) -> bool {
        let Some(&end) = self.columns.last() else {
            return false;
        };
        let wide_start = next.is_some_and(|next| next.columns.get(1) == Some(&2));
        end >= last_column || (wide_start && end + 1 >= last_column)
    }

    fn has_frame(&self) -> bool {
        self.text.iter().any(|character| FRAMES.contains(character))
    }
}

/// A URL that runs over rows, and the cells to mark.
///
/// A cell is an index into the rows given and a column. An edge-wrapped URL's cells run from
/// `start` to `end` in reading order; a framed one's are the clicked row's part, since the frames
/// lie between its rows' parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinedUrl {
    /// The URL.
    pub url: String,
    /// Its first cell to mark.
    pub start: (usize, usize),
    /// Its last cell to mark.
    pub end: (usize, usize),
}

/// The URL over two rows or more of `rows` that holds `column` of row `row`, if there is one.
/// `last_column` is the grid's last column.
#[must_use]
pub fn joined_url(
    rows: &[LinkRow],
    row: usize,
    column: usize,
    last_column: usize,
) -> Option<JoinedUrl> {
    let clicked = rows.get(row)?;
    if clicked.has_frame() {
        framed_url(rows, row, column)
    } else {
        edge_url(rows, row, column, last_column)
    }
}

/// The start of each `http://` or `https://` in `text` with no word character before it.
fn schemes(text: &[char]) -> Vec<usize> {
    let starts_with = |at: usize, scheme: &str| {
        scheme
            .chars()
            .enumerate()
            .all(|(offset, expected)| text.get(at + offset) == Some(&expected))
    };
    (0..text.len())
        .filter(|&at| starts_with(at, "http://") || starts_with(at, "https://"))
        .filter(|&at| {
            at == 0
                || text
                    .get(at - 1)
                    .is_none_or(|before| !(before.is_alphanumeric() || *before == '_'))
        })
        .collect()
}

/// Whether `row` starts what reads as a line of its own: a space, a new URL, a `label:` or
/// `HTTP/2: 200` form.
fn starts_afresh(row: &LinkRow, fills: bool) -> bool {
    let Some(first) = row.text.first() else {
        return true;
    };
    if first.is_whitespace() || schemes(&row.text).first() == Some(&0) {
        return true;
    }
    let text: String = row.text.iter().collect();
    let http_status = text.starts_with("HTTP/")
        && text
            .split_once(':')
            .and_then(|(version, _)| version.get(5..))
            .is_some_and(|number| number.chars().all(|c| c.is_ascii_digit() || c == '.'));
    let label = text.split_once(':').is_some_and(|(name, rest)| {
        !name.is_empty() && (rest.starts_with(char::is_whitespace) || (rest.is_empty() && !fills))
    });
    http_status || label
}

/// Whether a URL may hold `character`, as the terminal's own URL search reads one: no space, no
/// control character (`U+0000` to `U+001F`, `U+007F` to `U+009F`), no quote or bracket it ends at.
const fn in_url(character: char) -> bool {
    let code = character as u32;
    !(character.is_whitespace()
        || code <= 0x1f
        || (code >= 0x7f && code <= 0x9f)
        || matches!(
            character,
            '<' | '>' | '"' | '{' | '|' | '}' | '^' | '`' | '\'' | '⟨' | '⟩'
        ))
}

/// `text` without the punctuation a sentence puts after a URL and a closing bracket it did not
/// open.
fn trim_url(text: &[char]) -> usize {
    let mut length = text.len();
    loop {
        let Some(&last) = length.checked_sub(1).and_then(|index| text.get(index)) else {
            return length;
        };
        let unopened = |close: char, open: char| {
            last == close
                && text[..length].iter().filter(|&&c| c == close).count()
                    > text[..length].iter().filter(|&&c| c == open).count()
        };
        if matches!(last, '.' | ',' | ':' | ';' | '!' | '?')
            || unopened(')', '(')
            || unopened(']', '[')
        {
            length -= 1;
        } else {
            return length;
        }
    }
}

/// A URL a program wrapped at the right edge.
fn edge_url(rows: &[LinkRow], row: usize, column: usize, last_column: usize) -> Option<JoinedUrl> {
    // Whether the URL goes on from row `index` into the next.
    let runs_on = |index: usize| {
        let (Some(this), Some(next)) = (rows.get(index), rows.get(index + 1)) else {
            return false;
        };
        let fills = next.fills_to(last_column, rows.get(index + 2));
        this.wrapped
            || (this.fills_to(last_column, Some(next))
                && !next.has_frame()
                && !starts_afresh(next, fills))
    };
    // The URL starting at `start` of row `start_row`, run on over the rows after it.
    let url_from = |start_row: usize, start: usize| -> Option<JoinedUrl> {
        let mut characters: Vec<(char, usize, usize)> = Vec::new();
        let mut current = start_row;
        let mut from = start;
        loop {
            let this = rows.get(current)?;
            for index in from..this.text.len() {
                characters.push((this.text[index], current, this.column(index)?));
            }
            if characters.len() > MAX_URL || !runs_on(current) {
                break;
            }
            current += 1;
            from = 0;
        }
        let length = characters
            .iter()
            .position(|&(character, _, _)| !in_url(character))
            .unwrap_or(characters.len());
        let text: Vec<char> = characters[..length].iter().map(|&(c, _, _)| c).collect();
        let length = trim_url(&text);
        let first = characters.first()?;
        let last = characters.get(length.checked_sub(1)?)?;
        // One row is the terminal's own search's; the point must be in the URL.
        let joined = first.1 != last.1;
        let holds = (row, column) >= (first.1, first.2) && (row, column) <= (last.1, last.2);
        (joined && holds && length <= MAX_URL).then(|| JoinedUrl {
            url: text[..length].iter().collect(),
            start: (first.1, first.2),
            end: (last.1, last.2),
        })
    };
    // The clicked row's URLs that start before the point, nearest first, then the last one of
    // each row above that runs on into the row below it.
    let clicked = rows.get(row)?;
    let before_point = clicked
        .columns
        .iter()
        .position(|&start| start > column)
        .unwrap_or(clicked.text.len());
    for at in schemes(&clicked.text).into_iter().rev() {
        if at < before_point
            && let Some(url) = url_from(row, at)
        {
            return Some(url);
        }
    }
    let mut above = row;
    while above > 0 && runs_on(above - 1) && !rows[above - 1].has_frame() {
        above -= 1;
        if let Some(&at) = schemes(&rows[above].text).last() {
            return url_from(above, at);
        }
    }
    None
}

/// A URL a program drew inside a box frame.
fn framed_url(rows: &[LinkRow], row: usize, column: usize) -> Option<JoinedUrl> {
    // The URL's start: a scheme on the clicked row or one above in the same frame, after a frame.
    let mut start_row = row;
    let (url_column, prefix, right) = loop {
        let this = rows.get(start_row)?;
        let found = schemes(&this.text).into_iter().rev().find_map(|at| {
            let before = &this.text[..at];
            let frame_before = before.iter().any(|c| FRAMES.contains(c));
            let right = this.text[at..]
                .iter()
                .position(|c| FRAMES.contains(c))
                .map(|offset| at + offset)?;
            frame_before.then(|| (this.column(at), before.to_vec(), this.column(right)))
        });
        if let Some((Some(url_column), prefix, Some(right))) = found {
            break (url_column, prefix, right);
        }
        if start_row == 0 {
            return None;
        }
        start_row -= 1;
    };
    // A row's part of the URL, between its URL column and its right frame, when the row has the
    // same frame and prefix.
    let part = |index: usize| -> Option<(Vec<char>, usize)> {
        let this = rows.get(index)?;
        let start = this.index_at(url_column)?;
        if this.text[..start] != prefix[..] {
            return None;
        }
        let right_index = this.index_at(right)?;
        if !FRAMES.contains(this.text.get(right_index)?) {
            return None;
        }
        let between = &this.text[start..right_index];
        let used = between
            .iter()
            .rposition(|c| !c.is_whitespace())
            .map_or(0, |end| end + 1);
        let piece = &between[..used];
        if piece.is_empty() || piece.iter().any(|c| c.is_whitespace()) {
            return None;
        }
        Some((piece.to_vec(), start))
    };
    let width = right.checked_sub(url_column)?;
    let continues = |piece: &[char]| {
        piece.last().is_some_and(|c| FRAMED_CONTINUES.contains(c))
            || piece.len() * 100 >= width * FRAMED_FILL_PERCENT
    };
    let mut pieces = vec![part(start_row)?];
    let mut current = start_row;
    while pieces.last().is_some_and(|(piece, _)| continues(piece)) {
        let Some(next) = part(current + 1) else { break };
        if schemes(&next.0).first() == Some(&0) {
            break;
        }
        pieces.push(next);
        current += 1;
        if pieces.iter().map(|(piece, _)| piece.len()).sum::<usize>() > MAX_URL {
            return None;
        }
    }
    let last_row = start_row + pieces.len() - 1;
    if pieces.len() < 2 || row < start_row || row > last_row {
        return None;
    }
    let text: Vec<char> = pieces
        .iter()
        .flat_map(|(piece, _)| piece.iter().copied())
        .collect();
    let length = text.iter().position(|&c| !in_url(c)).unwrap_or(text.len());
    let length = trim_url(&text[..length]);
    let url: String = text[..length].iter().collect();
    // The clicked row's part, which the hover marks.
    let (piece, start) = &pieces[row - start_row];
    let clicked = rows.get(row)?;
    let first = clicked.column(*start)?;
    let last = clicked.column(start + piece.len() - 1)?;
    (column >= first && column <= last).then_some(JoinedUrl {
        url,
        start: (row, first),
        end: (row, last),
    })
}
