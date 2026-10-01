//! A finished block's located failures (#620): the places compilers and runtimes name as failing,
//! found in the block's text with the row each report starts on.
//!
//! Four families are read: rustc and cargo (`error[E…]: …` over ` --> path:line:col`), the GNU
//! shape gcc, clang and go print (`path:line:col: error: …`), tsc's (`path(line,col): error TS…`)
//! and Python's traceback (the last `File "path", line N` frame before the exception's line).

/// How bad a failure is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// An error: the build or the run failed there.
    Error,
    /// A warning, which did not fail it alone.
    Warning,
}

/// A place a block's output names as failing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The absolute line the report starts on.
    pub row: u64,
    /// The path as printed: relative to the block's folder, or absolute.
    pub path: String,
    /// Its line, counted from 1.
    pub line: u32,
    /// Its column, counted from 1, when the report gives one.
    pub column: Option<u32>,
    /// Whether the report is an error or a warning.
    pub severity: Severity,
    /// What the report says.
    pub message: String,
}

/// The failures `text` reports, in order.
///
/// `text` is a block's output with wrapped rows joined, its first line on the absolute row
/// `start`; `wraps[i]` says whether the row `first + i` continues into the next, so each line's
/// row is known though rows wrap.
#[must_use]
pub fn failures(text: &str, start: u64, wraps: &[bool], first: u64) -> Vec<Failure> {
    let lines: Vec<&str> = text.lines().collect();
    let rows = line_rows(lines.len(), start, wraps, first);
    let mut found = Vec::new();
    // rustc's header, the line that says what is wrong, waits for its ` --> ` line.
    let mut header: Option<(u64, Severity, String)> = None;
    // Python's innermost frame so far, waiting for the exception's line.
    let mut frame: Option<(u64, String, u32)> = None;
    for (content, &row) in lines.iter().zip(&rows) {
        let trimmed = content.trim_start();
        if let Some((severity, message)) = rust_header(trimmed) {
            header = Some((row, severity, message));
            continue;
        }
        if let Some((path, line, column)) = trimmed.strip_prefix("--> ").and_then(location) {
            let (row, severity, message) =
                header
                    .take()
                    .unwrap_or((row, Severity::Error, String::new()));
            found.push(Failure {
                row,
                path,
                line,
                column,
                severity,
                message,
            });
            continue;
        }
        if let Some((path, line)) = python_frame(trimmed) {
            frame = Some((row, path, line));
            continue;
        }
        if let Some((row, path, line)) = frame.take() {
            if content.starts_with(char::is_whitespace) || content.starts_with("Traceback") {
                // The frame's source line, or the next traceback's start.
                frame = Some((row, path, line));
                continue;
            }
            found.push(Failure {
                row,
                path,
                line,
                column: None,
                severity: Severity::Error,
                message: content.trim().to_string(),
            });
            continue;
        }
        if let Some(failure) = tsc(trimmed, row).or_else(|| gnu(trimmed, row)) {
            found.push(failure);
        }
    }
    found
}

/// The first error among `failures`, else the first failure.
#[must_use]
pub fn first_failure(failures: &[Failure]) -> Option<&Failure> {
    failures
        .iter()
        .find(|failure| failure.severity == Severity::Error)
        .or_else(|| failures.first())
}

/// The absolute row each of `count` lines starts on, the first on `start`: a row whose wrap flag
/// is set continues into the next, so a long line takes more than one.
fn line_rows(count: usize, start: u64, wraps: &[bool], first: u64) -> Vec<u64> {
    let mut rows = Vec::with_capacity(count);
    let mut row = start;
    for _ in 0..count {
        rows.push(row);
        let mut index = usize::try_from(row.saturating_sub(first)).unwrap_or(usize::MAX);
        while wraps.get(index).copied().unwrap_or(false) {
            index += 1;
        }
        row = first
            .saturating_add(u64::try_from(index).unwrap_or(u64::MAX))
            .saturating_add(1);
    }
    rows
}

/// rustc's `error[E0425]: …` or `warning: …`, with what it says.
fn rust_header(line: &str) -> Option<(Severity, String)> {
    let (severity, rest) = if let Some(rest) = line.strip_prefix("error") {
        (Severity::Error, rest)
    } else {
        (Severity::Warning, line.strip_prefix("warning")?)
    };
    // `error[E0425]: message` or `error: message`.
    let rest = match rest.strip_prefix('[') {
        Some(code) => code.split_once(']')?.1,
        None => rest,
    };
    let message = rest.strip_prefix(':')?.trim();
    Some((severity, message.to_string()))
}

/// `path:line:col` or `path:line`, the path looking like one.
fn location(text: &str) -> Option<(String, u32, Option<u32>)> {
    let mut parts = text.trim().splitn(3, ':');
    let path = parts.next()?;
    let line = parts.next()?.parse().ok()?;
    let column = parts.next().and_then(|column| column.parse().ok());
    looks_like_path(path).then(|| (path.to_string(), line, column))
}

/// The GNU shape: `path:line:col: message` or `path:line: message`.
fn gnu(line: &str, row: u64) -> Option<Failure> {
    let mut parts = line.splitn(4, ':');
    let path = parts.next()?;
    if !looks_like_path(path) {
        return None;
    }
    let number = parts.next()?.parse().ok()?;
    let third = parts.next()?;
    let (column, message) = match third.parse::<u32>() {
        Ok(column) => (Some(column), parts.next()?.trim()),
        Err(_) => (None, third.trim()),
    };
    if message.is_empty() {
        return None;
    }
    let severity = if message.starts_with("warning") {
        Severity::Warning
    } else {
        Severity::Error
    };
    let message = message
        .strip_prefix("fatal error:")
        .or_else(|| message.strip_prefix("error:"))
        .or_else(|| message.strip_prefix("warning:"))
        .unwrap_or(message)
        .trim()
        .to_string();
    Some(Failure {
        row,
        path: path.to_string(),
        line: number,
        column,
        severity,
        message,
    })
}

/// tsc's `path(line,col): error TS2304: message`.
fn tsc(line: &str, row: u64) -> Option<Failure> {
    let (place, rest) = line.split_once("): ")?;
    let (severity, message) = if let Some(message) = rest.strip_prefix("error ") {
        (Severity::Error, message)
    } else {
        (Severity::Warning, rest.strip_prefix("warning ")?)
    };
    let (path, numbers) = place.rsplit_once('(')?;
    let (number, column) = numbers.split_once(',')?;
    if !looks_like_path(path) {
        return None;
    }
    Some(Failure {
        row,
        path: path.to_string(),
        line: number.trim().parse().ok()?,
        column: column.trim().parse().ok(),
        severity,
        message: message.trim().to_string(),
    })
}

/// Python's `File "path", line N, in name`.
fn python_frame(line: &str) -> Option<(String, u32)> {
    let rest = line.strip_prefix("File \"")?;
    let (path, rest) = rest.split_once('"')?;
    let number = rest.strip_prefix(", line ")?;
    let digits: String = number.chars().take_while(char::is_ascii_digit).collect();
    Some((path.to_string(), digits.parse().ok()?))
}

/// Whether `text` reads as a file's path: one word, with a folder or an extension, not a URL's
/// scheme or a Windows drive.
fn looks_like_path(text: &str) -> bool {
    !text.is_empty()
        && !text.contains(char::is_whitespace)
        && !text.contains(['"', '\'', '(', ')', '<', '>'])
        && (text.contains('/') || text.contains('.'))
        && !matches!(text, "http" | "https" | "file")
        && text.len() > 1
}
