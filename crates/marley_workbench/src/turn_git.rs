//! The `git` programs of per-turn diffs (#509) and of a turn's undo (#714).
//!
//! A turn's commit and the list of turn refs; the merge that takes a turn out, the paths it
//! changes, and the write through a temporary index.
//!
//! Zed's `Repository` makes the checkpoints and the refs, and has no way to write a commit from a
//! tree or to list refs, so these run `git` itself, never through a shell, in the repository's
//! work directory. The workbench's spawns move to one module with #541, which takes these over.

use std::path::Path;

use anyhow::Context as _;

/// Where Marley's turn refs live.
pub(crate) const TURN_REFS: &str = "refs/marley/turns/";

/// The name and address a turn commit is written under.
const IDENTITY: (&str, &str) = ("Marley", "marley@localhost");

/// Writes a commit of the tree `tree` (any tree-ish, such as `<sha>^{tree}`) whose one parent is
/// `parent`, with the message's paragraphs `message`, in the repository at `work_dir`; its sha.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn commit_tree_in(
    work_dir: &Path,
    tree: &str,
    parent: &str,
    message: &[String],
) -> anyhow::Result<String> {
    let mut args = vec!["commit-tree", tree, "-p", parent, "--no-gpg-sign"];
    for paragraph in message {
        args.extend(["-m", paragraph.as_str()]);
    }
    let output = crate::process::output(
        "git",
        args,
        Some(work_dir),
        &[
            ("GIT_AUTHOR_NAME", IDENTITY.0),
            ("GIT_AUTHOR_EMAIL", IDENTITY.1),
            ("GIT_COMMITTER_NAME", IDENTITY.0),
            ("GIT_COMMITTER_EMAIL", IDENTITY.1),
        ],
    )
    .await
    .context("running git commit-tree")?;
    if !output.status.success() {
        anyhow::bail!(
            "git commit-tree refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    anyhow::ensure!(!sha.is_empty(), "git commit-tree wrote no commit");
    Ok(sha)
}

/// A file a turn changed, by its name, with its added and removed lines (#924).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileCount {
    pub(crate) name: String,
    pub(crate) added: u32,
    pub(crate) removed: u32,
}

/// Each file changed from `base` to `head` in the repository at `work_dir`, with its lines, from
/// `git diff --numstat` (#924).
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn numstat_in(
    work_dir: &Path,
    base: &str,
    head: &str,
) -> anyhow::Result<Vec<FileCount>> {
    let output = crate::process::output(
        "git",
        ["diff", "--numstat", base, head],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git diff --numstat")?;
    anyhow::ensure!(
        output.status.success(),
        "git diff --numstat refused: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let stat = git::status::parse_numstat(&String::from_utf8_lossy(&output.stdout));
    Ok(stat
        .entries
        .iter()
        .map(|(path, counts)| FileCount {
            name: path
                .file_name()
                .map_or_else(|| path.as_unix_str().to_string(), str::to_string),
            added: counts.added,
            removed: counts.deleted,
        })
        .collect())
}

/// Whether the commits `left` and `right` in the repository at `work_dir` hold the same tree,
/// from `git diff-tree --quiet`, which exits 1 for a difference.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn same_tree_in(work_dir: &Path, left: &str, right: &str) -> anyhow::Result<bool> {
    let output = crate::process::output(
        "git",
        ["diff-tree", "--quiet", left, right],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git diff-tree")?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => anyhow::bail!(
            "git diff-tree refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

/// One turn ref: its name, its commit and when that commit was made, in seconds since the epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TurnRef {
    pub(crate) name: String,
    pub(crate) sha: String,
    pub(crate) committed: u64,
}

/// The turn refs of the repository at `work_dir`.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn turn_refs_in(work_dir: &Path) -> anyhow::Result<Vec<TurnRef>> {
    let output = crate::process::output(
        "git",
        [
            "for-each-ref",
            "--format=%(refname) %(objectname) %(committerdate:unix)",
            TURN_REFS,
        ],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git for-each-ref")?;
    if !output.status.success() {
        anyhow::bail!(
            "git for-each-ref refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(' ');
            let name = parts.next()?.to_string();
            let sha = parts.next()?.to_string();
            let committed = parts.next()?.parse().ok()?;
            Some(TurnRef {
                name,
                sha,
                committed,
            })
        })
        .collect())
}

/// What taking a turn out of the tree comes to (#714).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UndoMerge {
    /// The tree with the turn taken out, written as objects only.
    Merged(String),
    /// The files where a later change meets the turn's.
    Conflict(Vec<String>),
}

/// The tree `now` (a checkpoint) has with the turn `turn` (a turn's commit) taken out: a
/// three-way merge of trees whose base is the turn, ours `now` and theirs the turn's start. It
/// writes objects only; no file, index or ref changes.
///
/// # Errors
///
/// When `git` cannot run, refuses, or is older than 2.40, which has no `--merge-base`.
pub(crate) async fn undo_merge_in(
    work_dir: &Path,
    turn: &str,
    now: &str,
) -> anyhow::Result<UndoMerge> {
    let base = format!("--merge-base={turn}");
    let start = format!("{turn}^");
    let output = crate::process::output(
        "git",
        [
            "merge-tree",
            "--write-tree",
            "--name-only",
            "-z",
            base.as_str(),
            now,
            start.as_str(),
        ],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git merge-tree")?;
    let fields: Vec<String> = output
        .stdout
        .split(|byte| *byte == 0)
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    match output.status.code() {
        Some(0) => {
            let tree = fields.first().map(|tree| tree.trim().to_string());
            match tree {
                Some(tree) if !tree.is_empty() => Ok(UndoMerge::Merged(tree)),
                _ => anyhow::bail!("git merge-tree wrote no tree"),
            }
        }
        // The tree's id, the conflicted files, an empty field, then git's messages.
        Some(1) => Ok(UndoMerge::Conflict(
            fields
                .iter()
                .skip(1)
                .take_while(|field| !field.is_empty())
                .cloned()
                .collect(),
        )),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if stderr.contains("merge-base") {
                anyhow::bail!(
                    "{} has no `merge-tree --merge-base`, which needs git 2.40 or newer",
                    git_version_in(work_dir).await
                );
            }
            anyhow::bail!("git merge-tree refused: {stderr}")
        }
    }
}

/// How a path changes from one tree to another (#714).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathChange {
    /// Put back as it was.
    Restored(String),
    /// Removed: the turn made it, and nothing changed it since.
    Removed(String),
}

/// The paths that differ between the trees `from` and `to`, each restored or removed.
///
/// # Errors
///
/// When `git` cannot run or refuses.
pub(crate) async fn changed_paths_in(
    work_dir: &Path,
    from: &str,
    to: &str,
) -> anyhow::Result<Vec<PathChange>> {
    let output = crate::process::output(
        "git",
        [
            "diff-tree",
            "-r",
            "-z",
            "--no-renames",
            "--name-status",
            from,
            to,
        ],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git diff-tree")?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff-tree refused: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let fields: Vec<String> = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    Ok(fields
        .chunks(2)
        .filter_map(|pair| match pair {
            [status, path] if status == "D" => Some(PathChange::Removed(path.clone())),
            [_, path] => Some(PathChange::Restored(path.clone())),
            _ => None,
        })
        .collect())
}

/// What writing the merged tree came to (#714).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UndoWrite {
    /// The turn's paths were written.
    Written,
    /// A file changed after the checkpoint `now`; git wrote nothing.
    Changed(String),
}

/// Writes `merged` over the working tree of the repository at `work_dir`, which held `now` (a
/// checkpoint): only the paths that differ, through a temporary index in the git directory, so the
/// user's index, `HEAD` and refs stay. A path changed since `now` makes git refuse before it
/// writes any file.
///
/// # Errors
///
/// When `git` cannot run or refuses for another reason.
pub(crate) async fn write_merged_in(
    work_dir: &Path,
    now: &str,
    merged: &str,
) -> anyhow::Result<UndoWrite> {
    let git_dir = crate::process::output(
        "git",
        ["rev-parse", "--absolute-git-dir"],
        Some(work_dir),
        &[],
    )
    .await
    .context("running git rev-parse")?;
    anyhow::ensure!(
        git_dir.status.success(),
        "git rev-parse found no git directory"
    );
    let git_dir = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();
    let index = Path::new(&git_dir).join(format!("index-marley-undo-{}.tmp", uuid::Uuid::new_v4()));
    let index_text = index.to_string_lossy().into_owned();
    let env = [("GIT_INDEX_FILE", index_text.as_str())];
    let written = async {
        let read = crate::process::output("git", ["read-tree", now], Some(work_dir), &env)
            .await
            .context("running git read-tree")?;
        anyhow::ensure!(
            read.status.success(),
            "git read-tree refused: {}",
            String::from_utf8_lossy(&read.stderr).trim()
        );
        // The stat information, so a file that still matches counts as up to date; one that does
        // not is what the merge below refuses on.
        let refresh = crate::process::output(
            "git",
            ["update-index", "-q", "--refresh"],
            Some(work_dir),
            &env,
        )
        .await
        .context("running git update-index")?;
        if !refresh.status.success() {
            log::debug!("turn undo: files differ from the checkpoint before the write");
        }
        let merge = crate::process::output(
            "git",
            ["read-tree", "-m", "-u", now, merged],
            Some(work_dir),
            &env,
        )
        .await
        .context("running git read-tree -m")?;
        if merge.status.success() {
            return Ok(UndoWrite::Written);
        }
        let stderr = String::from_utf8_lossy(&merge.stderr).trim().to_string();
        if stderr.contains("not uptodate") || stderr.contains("would be overwritten") {
            return Ok(UndoWrite::Changed(stderr));
        }
        anyhow::bail!("git read-tree -m refused: {stderr}")
    }
    .await;
    if let Err(error) = smol::fs::remove_file(&index).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        log::warn!(
            "turn undo: the temporary index stays at {}: {error}",
            index.display()
        );
    }
    written
}

/// `git --version`'s line, or `git` when it does not run.
async fn git_version_in(work_dir: &Path) -> String {
    crate::process::output("git", ["--version"], Some(work_dir), &[])
        .await
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| "git".to_string())
}
