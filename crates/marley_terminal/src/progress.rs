//! A running build's own progress, read from the line it redraws (#918).
//!
//! Two shapes: cargo's bar (`Building [==>  ] 1781/1813: zed`, which `cargo build`, `check`,
//! `test` and `doc` print) and a leading `[N/M]` (ninja, meson). Anything else gives none.

/// How far a build has come: `done` of `total` steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// The steps done.
    pub done: u32,
    /// The steps in all, never 0.
    pub total: u32,
}

impl Progress {
    /// The share done, from 0 to 1.
    #[must_use]
    pub fn share(self) -> f32 {
        // Scaled into a u16 so the division stays exact in an f32.
        let scale = self.total / u32::from(u16::MAX) + 1;
        let part = |count: u32| f32::from(u16::try_from(count / scale).unwrap_or(u16::MAX));
        (part(self.done) / part(self.total).max(1.0)).min(1.0)
    }
}

/// The progress `line` shows, if it is cargo's bar or starts with `[N/M]`.
#[must_use]
pub fn parse_progress(line: &str) -> Option<Progress> {
    let line = line.trim();
    cargo_bar(line).or_else(|| leading_count(line))
}

/// Cargo's `Building [==>  ] N/M: crates`.
fn cargo_bar(line: &str) -> Option<Progress> {
    let rest = line
        .strip_prefix("Building")?
        .trim_start()
        .strip_prefix('[')?;
    let (bar, rest) = rest.split_once(']')?;
    if !bar
        .chars()
        .all(|character| matches!(character, '=' | '>' | ' '))
    {
        return None;
    }
    count(rest.trim_start())
}

/// A leading `[N/M]`.
fn leading_count(line: &str) -> Option<Progress> {
    let (inside, _) = line.strip_prefix('[')?.split_once(']')?;
    count(inside)
}

/// `N/M` at the start of `text`, up to a colon or a space.
fn count(text: &str) -> Option<Progress> {
    let end = text.find([':', ' ']).unwrap_or(text.len());
    let (done, total) = text.get(..end)?.split_once('/')?;
    let done: u32 = done.parse().ok()?;
    let total: u32 = total.parse().ok()?;
    (total > 0 && done <= total).then_some(Progress { done, total })
}
