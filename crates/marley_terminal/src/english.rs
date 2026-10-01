//! Whether a line typed at a shell's prompt reads as English or as a command (#557), by local
//! rules alone: no line leaves the machine.
//!
//! A line that starts with `#`, `!` or `\`, whose first word assigns (`=`), or that holds a shell
//! operator, a flag, a variable, a glob or a path is a command. Otherwise the first word decides:
//! a word that is no command reads as English, a single one included, as Warp reads it; a
//! command followed by three or more words among which is an English marker reads as English,
//! since `rm the old build folder` would run `rm` on four files; any other command is one.

/// Words that make a command's arguments read as a sentence.
pub const MARKERS: &[&str] = &[
    "the", "a", "an", "my", "me", "all", "this", "that", "these", "please", "what", "how", "why",
    "which", "where", "is", "are", "does", "do", "of", "for", "with", "from", "into", "in", "on",
    "to",
];

/// bash's and zsh's builtins and keywords, which the search path does not hold.
pub const BUILTINS: &[&str] = &[
    "cd", "export", "alias", "unalias", "source", ".", "echo", "printf", "read", "set", "unset",
    "exit", "type", "command", "builtin", "eval", "exec", "jobs", "fg", "bg", "kill", "wait",
    "pushd", "popd", "dirs", "history", "fc", "local", "declare", "typeset", "let", "test", "[",
    "[[", "if", "then", "else", "fi", "for", "while", "until", "do", "done", "case", "esac",
    "function", "select", "time", "sudo", "true", "false", "return", "shift", "trap", "umask",
    "ulimit", "hash", "help", "logout", "setopt", "unsetopt", "autoload", "bindkey", "zle",
    "emulate", "rehash", "which", "whence",
];

/// What a typed line reads as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// Nothing but whitespace.
    Blank,
    /// A shell command.
    Command,
    /// A request in words, for an agent.
    English,
}

/// What `line` reads as, where `is_command` says whether a word names a command.
#[must_use]
pub fn read_line(line: &str, is_command: impl Fn(&str) -> bool) -> Reading {
    let line = line.trim();
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some(first) = words.first() else {
        return Reading::Blank;
    };
    if line.starts_with(['#', '!', '\\'])
        || first.contains('=')
        || words.iter().any(|word| looks_like_shell(word))
    {
        return Reading::Command;
    }
    if !is_command(first) {
        return Reading::English;
    }
    let rest = &words[1..];
    let sentence = rest.len() >= 3
        && rest
            .iter()
            .any(|word| MARKERS.contains(&word.to_lowercase().as_str()));
    if sentence {
        Reading::English
    } else {
        Reading::Command
    }
}

/// Whether #573's model is asked about `line`: a command followed by plain words, one of them an
/// English marker.
///
/// The rules above settle that case only by their marker rule. A line with shell syntax, one
/// whose first word is no command, or a command with plain arguments such as `git status` is
/// settled.
#[must_use]
pub fn open_case(line: &str, is_command: impl Fn(&str) -> bool) -> bool {
    let line = line.trim();
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some(first) = words.first() else {
        return false;
    };
    marker_count(line) > 0
        && !line.starts_with(['#', '!', '\\'])
        && !first.contains('=')
        && !words.iter().any(|word| looks_like_shell(word))
        && is_command(first)
}

/// How many of `line`'s words after the first are English markers.
#[must_use]
pub fn marker_count(line: &str) -> usize {
    line.split_whitespace()
        .skip(1)
        .filter(|word| MARKERS.contains(&word.to_lowercase().as_str()))
        .count()
}

/// Whether `word` is shell syntax no sentence has: an operator, a redirection, a variable, a
/// flag, a glob or a path.
fn looks_like_shell(word: &str) -> bool {
    word.starts_with('-')
        || word.contains(['|', '<', '>', '&', ';', '$', '*', '/', '`'])
        || word.starts_with('~')
        || (word.contains('?') && word.len() > 1 && !word.ends_with('?'))
}
