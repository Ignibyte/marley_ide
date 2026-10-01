//! Completions in the shell's prompt editor (#625): paths under the prompt's folder, commands from
//! the shell's history, and the project's tasks' commands, through Zed's completion menu.
//!
//! The word before the cursor completes as a path; the whole line, as the start of a command the
//! history or a task has. Tab opens the menu, Enter or Tab takes an entry, and the line does not
//! run until Enter after it.

use std::path::{Path, PathBuf};

use anyhow::Result;
use editor::{CompletionProvider, Editor};
use gpui::{App, AppContext as _, Context, Entity, Task, WeakEntity, Window};
use language::{Buffer, CodeLabel, ToOffset as _};
use project::{Completion, CompletionDisplayOptions, CompletionResponse, CompletionSource};
use terminal::Terminal;
use workspace::Workspace;

/// How many history and task commands the menu lists at most.
const COMMANDS_LISTED: usize = 12;

/// How many entries of a folder the menu lists at most.
const ENTRIES_LISTED: usize = 50;

/// The completions of the shell's editor of one terminal.
pub struct ShellCompletions {
    /// The terminal whose shell the command goes to.
    pub terminal: WeakEntity<Terminal>,
    /// The terminal's workspace, whose project's tasks complete too.
    pub workspace: WeakEntity<Workspace>,
}

impl std::fmt::Debug for ShellCompletions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ShellCompletions")
            .finish_non_exhaustive()
    }
}

impl CompletionProvider for ShellCompletions {
    fn completions(
        &self,
        buffer: &Entity<Buffer>,
        buffer_position: language::Anchor,
        _trigger: editor::CompletionContext,
        _window: &mut Window,
        cx: &mut Context<Editor>,
    ) -> Task<Result<Vec<CompletionResponse>>> {
        let buffer = buffer.read(cx);
        let cursor = buffer_position.to_offset(buffer);
        let line_start = buffer
            .reversed_chars_at(buffer_position)
            .take_while(|character| *character != '\n')
            .map(char::len_utf8)
            .sum::<usize>();
        let line_start = cursor.saturating_sub(line_start);
        let word_length = buffer
            .reversed_chars_at(buffer_position)
            .take_while(|character| !character.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        let word_start = cursor.saturating_sub(word_length);
        let line: String = buffer.text_for_range(line_start..cursor).collect();
        let word: String = buffer.text_for_range(word_start..cursor).collect();
        let line_range = buffer.anchor_before(line_start)..buffer_position;
        let word_range = buffer.anchor_before(word_start)..buffer_position;

        let terminal = self.terminal.upgrade();
        let folder = terminal
            .as_ref()
            .and_then(|terminal| prompt_folder(terminal.read(cx)));
        let history = terminal
            .as_ref()
            .map(|terminal| crate::autosuggest::history(terminal, cx))
            .unwrap_or_default();
        let tasks = self.workspace.upgrade().map_or_else(
            || Task::ready(Vec::new()),
            |workspace| task_commands(&workspace, cx),
        );
        cx.background_spawn(async move {
            let tasks = tasks.await;
            let mut completions = Vec::new();
            // The command so far: history first, newest first, then the tasks'.
            if !line.trim().is_empty() {
                let mut seen = Vec::new();
                for command in history.iter().chain(tasks.iter()) {
                    if seen.len() >= COMMANDS_LISTED {
                        break;
                    }
                    if command.starts_with(line.as_str())
                        && command.as_str() != line
                        && !seen.contains(command)
                    {
                        seen.push(command.clone());
                        completions.push(completion(line_range.clone(), command.clone()));
                    }
                }
            }
            // This runs off the main thread, so the folder is read here.
            if let Some(folder) = folder {
                completions.extend(
                    entries_for(&folder, &word)
                        .into_iter()
                        .map(|entry| completion(word_range.clone(), entry)),
                );
            }
            Ok(vec![CompletionResponse {
                completions,
                display_options: CompletionDisplayOptions {
                    dynamic_width: true,
                },
                // Each key asks again: the provider filters by what was typed itself.
                is_incomplete: true,
            }])
        })
    }

    fn is_completion_trigger(
        &self,
        _buffer: &Entity<Buffer>,
        _position: language::Anchor,
        _text: &str,
        _trigger_in_words: bool,
        _cx: &mut Context<Editor>,
    ) -> bool {
        false
    }

    fn sort_completions(&self) -> bool {
        false
    }

    fn filter_completions(&self) -> bool {
        false
    }
}

/// The folder `terminal`'s shell waits at its prompt in.
fn prompt_folder(terminal: &Terminal) -> Option<PathBuf> {
    terminal
        .marley_anchored()
        .prompt_folder()
        .map(PathBuf::from)
        .or_else(|| terminal.working_directory())
}

/// The commands of the project's tasks, each with its arguments.
fn task_commands(workspace: &Entity<Workspace>, cx: &App) -> Task<Vec<String>> {
    let project = workspace.read(cx).project().read(cx);
    let Some(inventory) = project.task_store().read(cx).task_inventory().cloned() else {
        return Task::ready(Vec::new());
    };
    let worktree = project.worktrees(cx).next().map(|tree| tree.read(cx).id());
    let listing = inventory.read(cx).list_tasks(None, None, worktree, cx);
    cx.background_spawn(async move {
        listing
            .await
            .into_iter()
            .map(|(_, template)| {
                std::iter::once(template.command)
                    .chain(template.args)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    })
}

/// The entries of the folder `word` names under `folder` whose names start with the rest of it,
/// as `word` would read with each: a folder's ending in `/`. Hidden entries only for a name
/// starting with `.`.
fn entries_for(folder: &Path, word: &str) -> Vec<String> {
    let (part, prefix) = word
        .rfind('/')
        .map_or(("", word), |at| word.split_at(at + 1));
    let listed = if Path::new(part).is_absolute() {
        PathBuf::from(part)
    } else {
        folder.join(part)
    };
    let Ok(entries) = std::fs::read_dir(&listed) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let shown =
                name.starts_with(prefix) && (prefix.starts_with('.') || !name.starts_with('.'));
            shown.then(|| {
                let folder = entry.file_type().is_ok_and(|kind| kind.is_dir());
                format!("{part}{name}{}", if folder { "/" } else { "" })
            })
        })
        .collect();
    found.sort();
    found.truncate(ENTRIES_LISTED);
    found
}

/// A menu entry that puts `text` over `range`.
fn completion(range: std::ops::Range<language::Anchor>, text: String) -> Completion {
    Completion {
        replace_range: range,
        label: CodeLabel::plain(text.clone(), None),
        new_text: text,
        documentation: None,
        source: CompletionSource::Custom,
        icon_path: None,
        icon_color: None,
        match_start: None,
        snippet_deduplication_key: None,
        insert_text_mode: None,
        confirm: None,
        group: None,
    }
}
