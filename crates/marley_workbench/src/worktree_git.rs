//! The `git` programs of the worktree rows (#560): what a worktree's branch would meet merging its
//! base.
//!
//! Zed's `Repository` runs no `merge-tree`, `rev-list` or `merge-base`, and its runner takes
//! `merge-tree`'s exit 1, which is its answer, for an error, so these run `git` here, never
//! through a shell, in the main checkout, with the flags Zed's own git runs with. Their arguments
//! are commits and branch names git reported. #511 adds its review and merge here, and #541's
//! process adapter takes the spawns over.

use std::path::Path;
use std::process::Output;

use anyhow::Context as _;

/// The variable that names the git these run, for scenarios, as `MARLEY_CLAUDE` names Claude
/// Code.
const GIT_OVERRIDE: &str = "MARLEY_GIT";

/// What a worktree's branch would meet merging its base now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Drift {
    /// The base: a branch, or a commit.
    pub(crate) base: String,
    /// The base's tip, abbreviated.
    pub(crate) base_commit: String,
    /// The commits on the base the branch does not have.
    pub(crate) behind: u32,
    /// The files a merge would stop on, none for a clean merge; `None` when this git cannot say.
    pub(crate) conflicts: Option<Vec<String>>,
}

/// Runs `git` in `main` with `args`, with Zed's flags and no prompt, and gives its output
/// unjudged.
async fn git(main: &Path, args: &[&str]) -> anyhow::Result<Output> {
    let program = std::env::var(GIT_OVERRIDE).unwrap_or_else(|_| "git".to_string());
    util::command::new_command(program)
        .current_dir(main)
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "log.showSignature=false",
            "--no-optional-locks",
            "--no-pager",
        ])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .await
        .with_context(|| format!("running git {}", args.first().copied().unwrap_or_default()))
}

/// The base recorded for `branch` as `branch.<branch>.base`, which #510 writes, if one is.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn recorded_base(main: &Path, branch: &str) -> anyhow::Result<Option<String>> {
    let key = format!("branch.{branch}.base");
    let output = git(main, &["config", "--get", &key]).await?;
    match output.status.code() {
        Some(0) => Ok(Some(text(&output.stdout)).filter(|base| !base.is_empty())),
        // `git config --get` says a key is unset with exit 1.
        Some(1) => Ok(None),
        _ => anyhow::bail!("git config refused: {}", text(&output.stderr)),
    }
}

/// What `branch_tip` would meet merging `base_tip`, the tip of `base`: the commits it is behind,
/// and the files a merge would stop on.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn summary(
    main: &Path,
    branch_tip: &str,
    base_tip: &str,
    base: &str,
) -> anyhow::Result<Drift> {
    let merge_base = git(main, &["merge-base", branch_tip, base_tip]).await?;
    anyhow::ensure!(
        merge_base.status.success(),
        "git merge-base refused: {}",
        text(&merge_base.stderr)
    );
    let merge_base = text(&merge_base.stdout);
    let range = format!("{branch_tip}..{base_tip}");
    let behind = git(main, &["rev-list", "--count", &range]).await?;
    anyhow::ensure!(
        behind.status.success(),
        "git rev-list refused: {}",
        text(&behind.stderr)
    );
    let behind = text(&behind.stdout)
        .parse()
        .context("git rev-list gave no count")?;
    let merged = git(
        main,
        &[
            "merge-tree",
            "--write-tree",
            "--name-only",
            "-z",
            "--no-messages",
            "--merge-base",
            &merge_base,
            branch_tip,
            base_tip,
        ],
    )
    .await?;
    let conflicts = match merged.status.code() {
        Some(0) => Some(Vec::new()),
        // The merged tree's id, then each file the merge stopped on, each ended by a NUL.
        Some(1) => Some(
            merged
                .stdout
                .split(|byte| *byte == 0)
                .skip(1)
                .filter(|path| !path.is_empty())
                .map(|path| String::from_utf8_lossy(path).into_owned())
                .collect(),
        ),
        _ if refused_write_tree(&merged) => None,
        _ => anyhow::bail!("git merge-tree refused: {}", text(&merged.stderr)),
    };
    Ok(Drift {
        base: base.to_string(),
        base_commit: base_tip.chars().take(7).collect(),
        behind,
        conflicts,
    })
}

/// Whether git refused `merge-tree --write-tree` as an option it does not know: before 2.38, or
/// before 2.40 for `--merge-base`. The words are git's own, as Orca matches them.
fn refused_write_tree(output: &Output) -> bool {
    let stderr = text(&output.stderr);
    output.status.code() == Some(129)
        && (stderr.contains("unknown option")
            || stderr.contains("unrecognized option")
            || stderr.contains("usage: git merge-tree"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}
