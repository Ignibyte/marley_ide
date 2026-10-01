//! Rows that are no grid line (#631): the space a terminal view draws between blocks, and above a
//! header taller than its prompt, as a map over the viewport's rows.
//!
//! The grid keeps its rows and its size; only where each row is drawn moves. A row's y is its
//! place in the grid plus everything inserted above it, moved as a whole so the anchored edge keeps
//! its row. The mouse goes the other way through [`RowMap::grid_y`].

use std::collections::BTreeMap;

/// Which edge of the view keeps its row where the grid puts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// The last row stays on the bottom edge and the space pushes the rows above it up: the live
    /// screen, whose prompt is on its last row.
    Bottom,
    /// The first row stays on the top edge and the space pushes the rows below it down: a view
    /// scrolled back, where a block navigated to lands on the top row.
    Top,
}

/// The space inserted above one row.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Insert {
    row: usize,
    /// Between the block above and the one starting at the row.
    gap: f32,
    /// A header's line beyond its prompt's, drawn as part of the row's block.
    lead: f32,
}

impl Insert {
    fn size(self) -> f32 {
        self.gap + self.lead
    }
}

/// Where a viewport's rows are drawn once space is inserted above some of them, in pixels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowMap {
    /// By row, ascending, one at most per row.
    inserts: Vec<Insert>,
    /// How far every row moves so the anchored edge keeps its row.
    shift: f32,
}

impl RowMap {
    /// The map for a viewport where blocks start at the rows `starts`, each `gap` pixels below the
    /// block above, and the headers at the rows `tall` take `lead` pixels more than their prompt.
    #[must_use]
    pub fn new(starts: &[usize], tall: &[usize], gap: f32, lead: f32, anchor: Anchor) -> Self {
        let mut by_row: BTreeMap<usize, Insert> = BTreeMap::new();
        for &row in starts {
            by_row
                .entry(row)
                .or_insert(Insert {
                    row,
                    gap: 0.0,
                    lead: 0.0,
                })
                .gap = gap.max(0.0);
        }
        for &row in tall {
            by_row
                .entry(row)
                .or_insert(Insert {
                    row,
                    gap: 0.0,
                    lead: 0.0,
                })
                .lead = lead.max(0.0);
        }
        let inserts: Vec<Insert> = by_row
            .into_values()
            .filter(|insert| insert.size() > 0.0)
            .collect();
        let shift = match anchor {
            Anchor::Bottom => -inserts.iter().map(|insert| insert.size()).sum::<f32>(),
            Anchor::Top => 0.0,
        };
        Self { inserts, shift }
    }

    /// Whether every row is drawn where the grid puts it.
    #[must_use]
    pub const fn is_identity(&self) -> bool {
        self.inserts.is_empty()
    }

    /// How far below its place in the grid `row` is drawn; negative when it moves up.
    #[must_use]
    pub fn offset(&self, row: usize) -> f32 {
        self.shift
            + self
                .inserts
                .iter()
                .take_while(|insert| insert.row <= row)
                .map(|insert| insert.size())
                .sum::<f32>()
    }

    /// The header line drawn above `row` as part of its block, beyond its prompt's rows.
    #[must_use]
    pub fn lead(&self, row: usize) -> f32 {
        self.inserts
            .iter()
            .find(|insert| insert.row == row)
            .map_or(0.0, |insert| insert.lead)
    }

    /// The rows with space inserted above them, ascending.
    pub fn breaks(&self) -> impl Iterator<Item = usize> + '_ {
        self.inserts.iter().map(|insert| insert.row)
    }

    /// The y in the grid for `y` as drawn, both from the top of the viewport's first row with rows
    /// `line_height` tall. A y in an insert lands on the first pixel of the row below it, the row
    /// the gap or the header line belongs to.
    #[must_use]
    pub fn grid_y(&self, y: f32, line_height: f32) -> f32 {
        let y = y - self.shift;
        let mut inserted = 0.0;
        for insert in &self.inserts {
            let row_top = row_y(insert.row, line_height);
            let top = row_top + inserted;
            if y < top {
                break;
            }
            if y < top + insert.size() {
                return row_top;
            }
            inserted += insert.size();
        }
        y - inserted
    }
}

/// The y of `row` in the grid, rows `line_height` tall.
fn row_y(row: usize, line_height: f32) -> f32 {
    f32::from(u16::try_from(row).unwrap_or(u16::MAX)) * line_height
}
