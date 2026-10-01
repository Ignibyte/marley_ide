//! A command's colours at the prompt (#626): what was typed at a shell's prompt, drawn in the
//! theme's syntax colours as Zed colours a shell script, and the same language for the shell's
//! prompt editor.
//!
//! The terminal's cells keep the colours the shell gave them; the element asks
//! `MarleyPromptColoring` each frame and paints the typed range over them, so readline's own
//! redraws are never fought.

use std::sync::Arc;

use gpui::{App, Context, Entity, Global, Hsla};
use language::{Language, Rope};
use terminal::Terminal;
use terminal_view::{MarleyPromptColoring, MarleyPromptColors};
use ui::ActiveTheme as _;
use util::ResultExt as _;
use workspace::Workspace;

/// The language Zed colours shell scripts with, once a project's registry has loaded it.
#[derive(Default)]
struct ShellLanguage(Option<Arc<Language>>);

impl Global for ShellLanguage {}

/// The name Zed registers the bash grammar under.
const SHELL_SCRIPT: &str = "Shell Script";

/// Sets the element's hook and loads the shell language from the first workspace's project.
/// [`crate::init`] calls it once, before any window opens.
pub fn init(cx: &mut App) {
    cx.set_global(ShellLanguage::default());
    cx.set_global(MarleyPromptColoring(Arc::new(colors)));
    cx.observe_new(
        |workspace: &mut Workspace, _, cx: &mut Context<Workspace>| {
            if cx.global::<ShellLanguage>().0.is_some() {
                return;
            }
            let loading = workspace
                .project()
                .read(cx)
                .languages()
                .language_for_name(SHELL_SCRIPT);
            cx.spawn(async move |_, cx| {
                if let Some(language) = loading.await.log_err() {
                    cx.update(|cx| cx.global_mut::<ShellLanguage>().0 = Some(language));
                }
            })
            .detach();
        },
    )
    .detach();
}

/// The shell language, when loaded.
pub fn shell_language(cx: &App) -> Option<Arc<Language>> {
    cx.try_global::<ShellLanguage>()?.0.clone()
}

/// The colours of what was typed at `terminal`'s prompt: each highlighted run of the line, at the
/// columns from where the typing started.
fn colors(terminal: &Entity<Terminal>, cx: &App) -> Option<MarleyPromptColors> {
    let language = shell_language(cx)?;
    let terminal = terminal.read(cx);
    let typed = crate::autosuggest::typed_text(terminal)?;
    if typed.trim().is_empty() {
        return None;
    }
    let (_, start_column) = terminal.marley_anchored().input_start()?;
    let line = terminal.last_content().cursor.point.line;
    // Each byte's column: one cell a character, from the input's start.
    let columns: Vec<usize> = typed
        .char_indices()
        .enumerate()
        .flat_map(|(index, (_, character))| {
            std::iter::repeat_n(start_column + index, character.len_utf8())
        })
        .collect();
    let end = start_column + typed.chars().count();
    let column_of = |byte: usize| columns.get(byte).copied().unwrap_or(end);
    let syntax = cx.theme().syntax();
    let runs = language
        .highlight_text(&Rope::from(typed.as_str()), 0..typed.len())
        .into_iter()
        .filter_map(|(range, id)| {
            let color = syntax.get(id)?.color?;
            Some((column_of(range.start)..column_of(range.end), color))
        })
        .collect::<Vec<(std::ops::Range<usize>, Hsla)>>();
    (!runs.is_empty()).then_some(MarleyPromptColors { line, runs })
}
