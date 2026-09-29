//! Claude Code's question whether to trust a folder (#587), read from a terminal's screen.
//!
//! Claude Code asks it in an interactive session, before it loads a folder's configuration, for
//! a repository it has not trusted yet; its trust is keyed on the repository's main checkout, so
//! a worktree of a trusted repository never asks. Its hooks wait for the answer, so nothing but
//! the screen shows the question. Its words have changed across releases, and the reader takes
//! each published form: a question line, a trust option, an exit option and the confirm footer,
//! at the bottom of the screen. Since 2.1.263 the focus starts on the exit option, where Enter
//! declines and quits, so an answer moves the focus to the trust option by name before Enter.

/// The lines that begin the question, as Claude Code's releases have worded them.
const QUESTIONS: [&str; 3] = [
    "Do you trust the files in this folder?",
    "Is this a project you created or one you trust?",
    "Accessing workspace:",
];

/// The trust option, as its releases have worded it.
const TRUST_OPTIONS: [&str; 2] = ["Yes, I trust this folder", "Yes, proceed"];

/// The exit option.
const EXIT_OPTION: &str = "No, exit";

/// The footer every form ends with.
const FOOTER: &str = "Enter to confirm";

/// How many of the screen's last lines the footer is among while the question shows: nothing is
/// drawn under it.
const FOOTER_WITHIN: usize = 3;

/// How far above the footer the question's first line can be.
const QUESTION_WITHIN: usize = 24;

/// The mark on the option with the focus.
const FOCUS: char = '❯';

/// Up and Down as a terminal sends them in its normal cursor mode.
const UP: &[u8] = b"\x1b[A";
const DOWN: &[u8] = b"\x1b[B";

/// One of the question's answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustOption {
    /// Trust the folder.
    Trust,
    /// Decline, and quit.
    Exit,
}

/// Claude Code's trust question, as its screen shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustQuestion {
    /// The folder it names, when the screen shows it on a line of its own.
    pub folder: Option<String>,
    /// Its warnings, such as a folder that pre-approves tool permissions, one per line.
    pub warnings: Vec<String>,
    /// Its options, top to bottom.
    options: Vec<TrustOption>,
    /// The option with the focus.
    focus: Option<usize>,
}

/// The question the last lines of a terminal's screen show, oldest first, when they show it
/// whole: a question line, then a trust option and an exit option, then the footer, among the
/// last lines.
#[must_use]
pub fn read(lines: &[String]) -> Option<TrustQuestion> {
    let footer = lines
        .iter()
        .rev()
        .take(FOOTER_WITHIN)
        .position(|line| line.contains(FOOTER))
        .map(|from_end| lines.len() - 1 - from_end)?;
    let top = footer.saturating_sub(QUESTION_WITHIN);
    let start = top
        + lines
            .get(top..footer)?
            .iter()
            .position(|line| QUESTIONS.iter().any(|question| line.contains(question)))?;
    let body = lines.get(start..footer)?;
    let mut options = Vec::new();
    let mut focus = None;
    for line in body {
        let (focused, option) = option_of(line);
        if let Some(option) = option {
            if focused {
                focus = Some(options.len());
            }
            options.push(option);
        }
    }
    let answers = options.contains(&TrustOption::Trust) && options.contains(&TrustOption::Exit);
    answers.then(|| TrustQuestion {
        folder: folder_of(body),
        warnings: body
            .iter()
            .map(|line| line.trim())
            .filter(|line| line.starts_with('⚠'))
            .map(str::to_string)
            .collect(),
        options,
        focus,
    })
}

/// Whether the footer of a question is among `rows`, the rows the terminal's screen shows.
///
/// The lines [`read`] takes reach into the scrollback on a sparse screen, where an answered
/// question stays after a clear.
#[must_use]
pub fn footer_on_screen(rows: &[String]) -> bool {
    rows.iter().any(|row| row.contains(FOOTER))
}

impl TrustQuestion {
    /// The keys that answer the question with the trust option: Up or Down from the option with
    /// the focus to it, then Enter; none unless the screen shows both options and the focus.
    #[must_use]
    pub fn answer_keys(&self) -> Option<Vec<u8>> {
        let trust = self
            .options
            .iter()
            .position(|option| *option == TrustOption::Trust)?;
        let focus = self.focus?;
        let mut keys = if trust >= focus {
            DOWN.repeat(trust - focus)
        } else {
            UP.repeat(focus - trust)
        };
        keys.push(b'\r');
        Some(keys)
    }
}

/// Whether `line` carries the focus mark, and the option it names, past the mark and a `1.`
/// numbering.
fn option_of(line: &str) -> (bool, Option<TrustOption>) {
    let line = line.trim_start();
    let (focused, rest) = line
        .strip_prefix(FOCUS)
        .map_or((false, line), |rest| (true, rest.trim_start()));
    let rest = rest
        .trim_start_matches(|character: char| character.is_ascii_digit())
        .trim_start_matches(['.', ')'])
        .trim_start();
    let option = if TRUST_OPTIONS.iter().any(|option| rest.starts_with(option)) {
        Some(TrustOption::Trust)
    } else if rest.starts_with(EXIT_OPTION) {
        Some(TrustOption::Exit)
    } else {
        None
    };
    (focused, option)
}

/// The folder the question names: the first line after its first line that is a path.
fn folder_of(body: &[String]) -> Option<String> {
    body.iter()
        .skip(1)
        .map(|line| line.trim())
        .find(|line| line.starts_with('/') || line.starts_with('~'))
        .map(str::to_string)
}
