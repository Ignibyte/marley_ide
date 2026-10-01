//! Autosuggestions (T3a): the rest of the newest command in history that starts with what was
//! typed at a prompt, and the commands a shell's history file holds.

/// The rest of the first command in `history`, taken newest first, that starts with `typed` and
/// goes on past it on the same line. Blank `typed` has none.
#[must_use]
pub fn suggestion<'a>(typed: &str, history: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    if typed.trim().is_empty() {
        return None;
    }
    history.into_iter().find_map(|command| {
        command
            .strip_prefix(typed)
            .filter(|rest| !rest.is_empty() && !rest.contains('\n'))
    })
}

/// The commands a shell's history file holds, oldest first.
///
/// Bash's file is a command a line, with a `#<seconds>` line before each when `HISTTIMEFORMAT` is
/// set; zsh's extended history is `: <seconds>:<elapsed>;<command>`, a command of several lines
/// ending each but its last with a backslash; fish's is `- cmd: <command>` with indented lines
/// under it, the command's newlines and backslashes escaped (#466).
#[must_use]
pub fn parse_history(text: &str) -> Vec<String> {
    let first = text.lines().find(|line| !line.trim().is_empty());
    if first.is_some_and(|line| line.starts_with(FISH_COMMAND)) {
        return text
            .lines()
            .filter_map(|line| line.strip_prefix(FISH_COMMAND))
            .map(fish_unescape)
            .collect();
    }
    let mut commands: Vec<String> = Vec::new();
    // Whether the last zsh command goes on to the next line.
    let mut continued = false;
    for line in text.lines() {
        if continued && let Some(last) = commands.last_mut() {
            last.push('\n');
            let rest = line.strip_suffix('\\');
            last.push_str(rest.unwrap_or(line));
            continued = rest.is_some();
        } else if let Some(command) = zsh_command(line) {
            let rest = command.strip_suffix('\\');
            commands.push(rest.unwrap_or(command).to_string());
            continued = rest.is_some();
        } else if !is_bash_time(line) && !line.trim().is_empty() {
            commands.push(line.to_string());
        }
    }
    commands
}

/// What starts each entry of fish's history.
const FISH_COMMAND: &str = "- cmd: ";

/// A command as fish's history file escapes it: `\n` for a newline and `\\` for a backslash.
fn fish_unescape(command: &str) -> String {
    let mut unescaped = String::with_capacity(command.len());
    let mut characters = command.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            unescaped.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => unescaped.push('\n'),
            // A trailing backslash stays as it was written.
            Some('\\') | None => unescaped.push('\\'),
            Some(other) => {
                unescaped.push('\\');
                unescaped.push(other);
            }
        }
    }
    unescaped
}

/// The command of a zsh extended-history line, `: <seconds>:<elapsed>;<command>`.
fn zsh_command(line: &str) -> Option<&str> {
    let (stamp, command) = line.strip_prefix(": ")?.split_once(';')?;
    let (seconds, elapsed) = stamp.split_once(':')?;
    (all_digits(seconds) && all_digits(elapsed)).then_some(command)
}

/// Whether `line` is the `#<seconds>` line bash writes before a command.
fn is_bash_time(line: &str) -> bool {
    line.strip_prefix('#').is_some_and(all_digits)
}

fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}
