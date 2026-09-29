//! PURE — copy and paste while an agent CLI runs in the terminal (#536).
//!
//! [`strip_shared_indent`] takes from a copied selection the run of leading spaces every non-blank
//! line shares, the gutter an agent's reply is drawn in. [`is_raw_image_path`] says whether a
//! dropped file's path can go to the agent raw, the form in which Claude Code and Codex attach an
//! image. Both rules are Orca's.

use std::borrow::Cow;
use std::path::Path;

/// The file extensions an agent attaches as an image: the ones the Claude API takes.
const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

/// The characters that keep a path from going raw, since a shell would read them.
const SHELL_SPECIALS: &str = "\"'$;&|<>(){}[]*?!#\\`";

/// `text` without the leading spaces its non-blank lines share.
///
/// Lines split on `\n`. A line of spaces, or an empty one, does not count, and loses at most the
/// shared run. Relative indentation stays, and a shared run of zero, as when a line starts at
/// column 0, returns `text` as it is.
#[must_use]
pub fn strip_shared_indent(text: &str) -> Cow<'_, str> {
    let indent = |line: &str| line.bytes().take_while(|&byte| byte == b' ').count();
    let shared = text
        .split('\n')
        .filter(|line| line.trim_end_matches('\r').bytes().any(|byte| byte != b' '))
        .map(indent)
        .min()
        .unwrap_or(0);
    if shared == 0 {
        return Cow::Borrowed(text);
    }
    let lines: Vec<&str> = text
        .split('\n')
        .map(|line| line.get(indent(line).min(shared)..).unwrap_or(line))
        .collect();
    Cow::Owned(lines.join("\n"))
}

/// Whether `path` can go to an agent raw: a file name with an image's extension, in any case,
/// and no control character nor any character a shell would read.
#[must_use]
pub fn is_raw_image_path(path: &Path) -> bool {
    let Some(text) = path.to_str() else {
        return false;
    };
    let image = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|image| extension.eq_ignore_ascii_case(image))
        });
    image
        && !text
            .chars()
            .any(|character| character.is_control() || SHELL_SPECIALS.contains(character))
}
