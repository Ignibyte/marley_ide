//! The two `git` programs of per-turn diffs (#509): a turn's commit, and the list of turn refs.
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
