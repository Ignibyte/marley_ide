//! The main checkout's gitignored files a new worktree agent's worktree gets (#585).
//!
//! `.worktreeinclude` at the main checkout's root names them in `.gitignore` syntax, as Claude
//! Code reads it for its own worktrees (code.claude.com/docs/en/worktrees, "Copy gitignored files
//! into worktrees"): a file is copied only when a line matches it and git ignores it, so a tracked
//! file never is. A line that starts with `**/`, or has no slash, reaches into a directory git
//! ignores as a whole only when that directory matches, or when the first name after the `**/` is
//! one of the directory's names; a line that names the directory reaches it. Sizes are read before
//! anything is written, and an entry that would pass the budget is left out whole. Nothing already
//! in the worktree is overwritten.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use fs::{CopyOptions, Fs};
use futures::StreamExt as _;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::worktree_git;

/// The file at the main checkout's root that names what a new worktree gets.
pub(crate) const INCLUDE_FILE: &str = ".worktreeinclude";

/// The most a new worktree gets. Claude Code sets no limit and Orca refuses oversized trees before
/// writing; these keep a create quick.
const BUDGET_BYTES: u64 = 100 * 1024 * 1024;
const BUDGET_FILES: usize = 10_000;

/// What a copy did.
#[derive(Debug, Default)]
pub(crate) struct Included {
    /// How many files it copied.
    pub(crate) copied: usize,
    /// What it left out, each with why: past the budget, or a copy that failed.
    pub(crate) skipped: Vec<String>,
}

/// An entry git lists as ignored, with the files the lines take of it.
struct Entry {
    /// Relative to the main checkout.
    path: PathBuf,
    is_dir: bool,
    /// The files, relative to the main checkout, with their sizes.
    files: Vec<(PathBuf, u64)>,
    /// Whether reading it passed the budget's count of files on its own.
    over: bool,
}

/// Copies into `worktree` the files of the main checkout `main` that git ignores and its
/// `.worktreeinclude` names; nothing when it has none.
///
/// # Errors
///
/// When the file cannot be read or parsed, or git cannot list what it ignores; a file that
/// cannot be copied is named in [`Included::skipped`] instead.
pub(crate) async fn copy_included(
    main: &Path,
    worktree: &Path,
    fs: &dyn Fs,
) -> anyhow::Result<Included> {
    let file = main.join(INCLUDE_FILE);
    if fs
        .metadata(&file)
        .await?
        .is_none_or(|metadata| metadata.is_dir)
    {
        return Ok(Included::default());
    }
    let contents = fs
        .load(&file)
        .await
        .with_context(|| format!("reading {INCLUDE_FILE}"))?;
    let lines: Vec<&str> = contents.lines().collect();
    let matcher = matcher(main, &lines)?;
    let reaches: Vec<Reach> = lines.iter().filter_map(|line| Reach::of(line)).collect();
    let mut entries = Vec::new();
    for (path, is_dir) in worktree_git::ignored_entries(main).await? {
        if let Some(entry) = entry(main, path, is_dir, &matcher, &reaches, fs).await {
            entries.push(entry);
        }
    }
    let mut included = Included::default();
    let (mut bytes, mut count) = (0_u64, 0_usize);
    let mut chosen: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let size: u64 = entry.files.iter().map(|(_, size)| size).sum();
        if entry.over || count + entry.files.len() > BUDGET_FILES || bytes + size > BUDGET_BYTES {
            let name = if entry.is_dir {
                format!("{}/", entry.path.display())
            } else {
                entry.path.display().to_string()
            };
            included.skipped.push(if entry.over {
                format!("{name} (more than {BUDGET_FILES} files)")
            } else {
                format!(
                    "{name} ({}, past the {} budget)",
                    megabytes(size),
                    megabytes(BUDGET_BYTES)
                )
            });
            continue;
        }
        bytes += size;
        count += entry.files.len();
        chosen.extend(entry.files.into_iter().map(|(path, _)| path));
    }
    for path in chosen {
        let target = worktree.join(&path);
        let copied = async {
            if let Some(parent) = target.parent() {
                fs.create_dir(parent).await?;
            }
            fs.copy_file(
                &main.join(&path),
                &target,
                CopyOptions {
                    overwrite: false,
                    ignore_if_exists: true,
                },
            )
            .await
        };
        match copied.await {
            Ok(()) => included.copied += 1,
            Err(error) => included
                .skipped
                .push(format!("{} ({error:#})", path.display())),
        }
    }
    Ok(included)
}

/// The lines as a matcher rooted at the main checkout.
fn matcher(main: &Path, lines: &[&str]) -> anyhow::Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(main);
    for line in lines {
        builder
            .add_line(None, line)
            .with_context(|| format!("{INCLUDE_FILE} has a line that is no pattern: {line}"))?;
    }
    builder
        .build()
        .with_context(|| format!("reading the patterns of {INCLUDE_FILE}"))
}

/// What the lines take of one ignored entry: a file a line matches; a directory a line matches,
/// whole; a directory a line reaches, its files a line matches; else nothing.
async fn entry(
    main: &Path,
    path: PathBuf,
    is_dir: bool,
    matcher: &Gitignore,
    reaches: &[Reach],
    fs: &dyn Fs,
) -> Option<Entry> {
    let named = matcher
        .matched_path_or_any_parents(&path, is_dir)
        .is_ignore();
    if !is_dir {
        if !named {
            return None;
        }
        let metadata = fs.metadata(&main.join(&path)).await.ok()??;
        // A link to a folder is listed as a file; it is not walked.
        if metadata.is_dir {
            return None;
        }
        let size = metadata.len;
        return Some(Entry {
            path: path.clone(),
            is_dir,
            files: vec![(path, size)],
            over: false,
        });
    }
    let start = if named {
        path.clone()
    } else {
        reaches.iter().find_map(|reach| reach.start(&path))?
    };
    let (files, over) = walk(main, &start, matcher, fs).await;
    (!files.is_empty() || over).then_some(Entry {
        path,
        is_dir,
        files,
        over,
    })
}

/// The files at or under `start` a line matches, relative to `main`, with their sizes; a line's
/// `!` keeps a file out. A folder holding a `.git`, another repository or worktree, and a link to a
/// folder are passed over; the walk stops past the budget's count of files.
async fn walk(
    main: &Path,
    start: &Path,
    matcher: &Gitignore,
    fs: &dyn Fs,
) -> (Vec<(PathBuf, u64)>, bool) {
    let mut files = Vec::new();
    let mut paths = vec![main.join(start)];
    while let Some(path) = paths.pop() {
        let Ok(Some(metadata)) = fs.metadata(&path).await else {
            continue;
        };
        if metadata.is_dir {
            let nested = fs
                .metadata(&path.join(".git"))
                .await
                .ok()
                .flatten()
                .is_some();
            if metadata.is_symlink || nested {
                continue;
            }
            let Ok(mut children) = fs.read_dir(&path).await else {
                continue;
            };
            while let Some(child) = children.next().await {
                if let Ok(child) = child {
                    paths.push(child);
                }
            }
            continue;
        }
        let Ok(relative) = path.strip_prefix(main) else {
            continue;
        };
        if !matcher
            .matched_path_or_any_parents(relative, false)
            .is_ignore()
        {
            continue;
        }
        files.push((relative.to_path_buf(), metadata.len));
        if files.len() > BUDGET_FILES {
            return (files, true);
        }
    }
    (files, false)
}

/// How a line reaches into a directory git ignores as a whole, by Claude Code's rule.
#[derive(Debug, PartialEq, Eq)]
enum Reach {
    /// A line that starts with `**/`, or has no slash: into a directory one of whose names is its
    /// first name after the `**/`.
    Named(String),
    /// A line with a slash: into a directory its leading names without a wildcard lead to or
    /// under.
    Under(PathBuf),
}

impl Reach {
    /// The reach of one line; none for a comment, a negation, an empty line, or a line whose
    /// first name is a wildcard, which names no directory.
    fn of(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            return None;
        }
        let line = line.trim_end_matches('/');
        if let Some(rest) = line.strip_prefix("**/") {
            let first = rest.split('/').next().filter(|first| !first.is_empty())?;
            return Some(Self::Named(first.to_string()));
        }
        if !line.contains('/') {
            return Some(Self::Named(line.to_string()));
        }
        let prefix: PathBuf = line
            .trim_start_matches('/')
            .split('/')
            .take_while(|part| !part.contains(['*', '?', '[', '\\']))
            .collect();
        (prefix.components().next().is_some()).then_some(Self::Under(prefix))
    }

    /// Where a walk of the ignored directory `dir` for this line starts, when the line reaches
    /// into it: the directory, or the line's deeper leading names under it.
    fn start(&self, dir: &Path) -> Option<PathBuf> {
        match self {
            Self::Named(name) => dir
                .components()
                .any(|part| part.as_os_str() == name.as_str())
                .then(|| dir.to_path_buf()),
            Self::Under(prefix) if prefix.starts_with(dir) => Some(prefix.clone()),
            Self::Under(prefix) if dir.starts_with(prefix) => Some(dir.to_path_buf()),
            Self::Under(_) => None,
        }
    }
}

/// `bytes` in whole megabytes, as a toast says it.
fn megabytes(bytes: u64) -> String {
    format!("{} MB", bytes.div_ceil(1024 * 1024))
}
