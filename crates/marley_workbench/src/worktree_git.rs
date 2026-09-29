//! The `git` programs of the worktree rows (#560) and of a new worktree agent's worktree (#585).
//!
//! What a worktree's branch would meet merging its base, since #511 who merges it and the merge
//! itself, since #585 what a new worktree copies and the setup command a repository keeps, and
//! since #590 a worktree's port slot, and since #589 what a removal would lose and the end of a
//! removed worktree's branch. Zed's `Repository` runs no `merge-tree`, `rev-list`,
//! `merge-base` or `merge`, and its runner takes `merge-tree`'s exit 1, which is its answer, for
//! an error, so these run `git` here, never through a shell, in the main checkout or a worktree
//! of it, with the flags Zed's own git runs with. Their arguments are commits and branch names git
//! reported. #541's process adapter takes the spawns over.

use std::path::{Path, PathBuf};
use std::process::Output;

use anyhow::Context as _;
use fs::Fs;

/// The variable that names the git these run, for scenarios, as `MARLEY_CLAUDE` names Claude
/// Code.
const GIT_OVERRIDE: &str = "MARLEY_GIT";

/// The file the Rustal workflow writes at a repository's root when it manages it (`rw init`).
const WORKFLOW_FILE: &str = "workflow.toml";

/// What a worktree's branch would meet merging its base now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Drift {
    /// The base: a branch, or a commit.
    pub(crate) base: String,
    /// The base's tip, abbreviated.
    pub(crate) base_commit: String,
    /// The commits on the branch the base does not have (#511).
    pub(crate) ahead: u32,
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

/// What `branch_tip` would meet merging `base_tip`, the tip of `base`: the commits it is ahead and
/// behind, and the files a merge would stop on.
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
    // The commits only the branch has, then those only the base has.
    let range = format!("{branch_tip}...{base_tip}");
    let counts = git(main, &["rev-list", "--left-right", "--count", &range]).await?;
    anyhow::ensure!(
        counts.status.success(),
        "git rev-list refused: {}",
        text(&counts.stderr)
    );
    let counts = text(&counts.stdout);
    let (ahead, behind) = counts
        .split_once(char::is_whitespace)
        .context("git rev-list gave no counts")?;
    let ahead = ahead.trim().parse().context("git rev-list gave no count")?;
    let behind = behind
        .trim()
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
        ahead,
        behind,
        conflicts,
    })
}

/// Who merges a repository's worktree branches (#511).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MergeOwner {
    /// Marley, from a worktree row's menu.
    #[default]
    Marley,
    /// The Rustal workflow, which manages the repository.
    Workflow,
}

/// Who merges the worktree branches of the repository whose main checkout is `main`: `git config
/// marley.merge` when it is `workflow` or `marley`, else the workflow when the root holds a
/// `workflow.toml` with a `[project]` table, the file `rw init` writes.
///
/// # Errors
///
/// When `git` cannot run or refuses, or the file cannot be read.
pub(crate) async fn merge_owner(main: &Path, fs: &dyn Fs) -> anyhow::Result<MergeOwner> {
    let output = git(main, &["config", "--get", "marley.merge"]).await?;
    match output.status.code() {
        Some(0) => match text(&output.stdout).as_str() {
            "workflow" => return Ok(MergeOwner::Workflow),
            "marley" => return Ok(MergeOwner::Marley),
            other => log::warn!("marley.merge is {other:?}, neither workflow nor marley"),
        },
        // `git config --get` says a key is unset with exit 1.
        Some(1) => {}
        _ => anyhow::bail!("git config refused: {}", text(&output.stderr)),
    }
    let file = main.join(WORKFLOW_FILE);
    if fs
        .metadata(&file)
        .await?
        .is_none_or(|metadata| metadata.is_dir)
    {
        return Ok(MergeOwner::Marley);
    }
    let contents = fs.load(&file).await?;
    let managed = contents
        .parse::<toml::Table>()
        .is_ok_and(|table| table.get("project").is_some_and(toml::Value::is_table));
    Ok(if managed {
        MergeOwner::Workflow
    } else {
        MergeOwner::Marley
    })
}

/// Whether Marley merges `branch` of the repository whose main checkout is `main`, and into what
/// (#511): the workflow does not merge the repository, the branch records its base as #510 writes
/// it, a branch rather than a commit, and [`merge_checks`] pass. Gives the base and the count of
/// commits to merge.
///
/// # Errors
///
/// With why Marley does not merge the branch, or when `git` cannot run.
pub(crate) async fn ready_to_merge(
    main: &Path,
    worktree: &Path,
    branch: &str,
    fs: &dyn Fs,
) -> anyhow::Result<(String, u32)> {
    anyhow::ensure!(
        merge_owner(main, fs).await? == MergeOwner::Marley,
        "the Rustal workflow merges this repository's branches, so Marley leaves {branch} to it"
    );
    let base = recorded_base(main, branch).await?.with_context(|| {
        format!("{branch} records no base, so Marley did not start it and does not merge it")
    })?;
    anyhow::ensure!(
        !is_commit(&base),
        "{branch} was started from commit {}, not from a branch, so there is no branch to merge it \
         into",
        base.chars().take(7).collect::<String>()
    );
    let ahead = merge_checks(main, worktree, branch, &base).await?;
    Ok((base, ahead))
}

/// Whether `name` is a full commit id, SHA-1 or SHA-256.
pub(crate) fn is_commit(name: &str) -> bool {
    matches!(name.len(), 40 | 64) && name.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Checks, in order, that `branch` can be merged into `base` in the main checkout `main`: the main
/// checkout is on `base`, neither it nor `worktree` has a tracked change not committed, and the
/// branch has commits the base lacks, whose count it gives. Each check changes nothing.
///
/// # Errors
///
/// With the check that failed, or when `git` cannot run.
pub(crate) async fn merge_checks(
    main: &Path,
    worktree: &Path,
    branch: &str,
    base: &str,
) -> anyhow::Result<u32> {
    let head = git(main, &["symbolic-ref", "--quiet", "--short", "HEAD"]).await?;
    anyhow::ensure!(
        head.status.success(),
        "the main checkout is on no branch; Marley merges into {base} only"
    );
    let on = text(&head.stdout);
    anyhow::ensure!(
        on == base,
        "the main checkout is on {on}, not on {base}; check out {base} there first"
    );
    for (checkout, what) in [(main, "the main checkout"), (worktree, "the worktree")] {
        let status = git(checkout, &["status", "--porcelain", "--untracked-files=no"]).await?;
        anyhow::ensure!(
            status.status.success(),
            "git status refused in {what}: {}",
            text(&status.stderr)
        );
        // Each line is two status letters, either of which may be a space, a space, then the
        // path, so the output is read untrimmed.
        let changed: Vec<String> = String::from_utf8_lossy(&status.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.get(3..).unwrap_or(line).to_string())
            .collect();
        anyhow::ensure!(
            changed.is_empty(),
            "{what} has changes not committed: {}",
            changed.join(", ")
        );
    }
    let range = format!("refs/heads/{base}..refs/heads/{branch}");
    let ahead = git(main, &["rev-list", "--count", &range]).await?;
    anyhow::ensure!(
        ahead.status.success(),
        "git rev-list refused: {}",
        text(&ahead.stderr)
    );
    let ahead: u32 = text(&ahead.stdout)
        .parse()
        .context("git rev-list gave no count")?;
    anyhow::ensure!(ahead > 0, "{branch} has nothing to merge into {base}");
    Ok(ahead)
}

/// Merges `branch` into `base`, the branch the main checkout `main` is on, with a merge commit,
/// never a push, and gives the merge commit, abbreviated. A merge that stops is aborted, so the
/// main checkout is as it was.
///
/// # Errors
///
/// With the files a conflict stopped on, or git's refusal, once the merge is undone.
pub(crate) async fn merge(main: &Path, branch: &str, base: &str) -> anyhow::Result<String> {
    // The full ref, so a tag of the same name is never the one merged; the title names the
    // branch as git's own would.
    let reference = format!("refs/heads/{branch}");
    let title = format!("Merge branch '{branch}' into {base}");
    let merged = git(
        main,
        &["merge", "--no-ff", "--no-edit", "-m", &title, &reference],
    )
    .await?;
    if merged.status.success() {
        let head = git(main, &["rev-parse", "--short", "HEAD"]).await?;
        anyhow::ensure!(
            head.status.success(),
            "git rev-parse refused: {}",
            text(&head.stderr)
        );
        return Ok(text(&head.stdout));
    }
    let conflicted = git(main, &["diff", "--name-only", "--diff-filter=U"]).await?;
    let files: Vec<String> = text(&conflicted.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    // A merge that stopped leaves `MERGE_HEAD`; one git refused before it began leaves none.
    let stopped = git(main, &["rev-parse", "--verify", "--quiet", "MERGE_HEAD"])
        .await?
        .status
        .success();
    if stopped {
        let aborted = git(main, &["merge", "--abort"]).await?;
        anyhow::ensure!(
            aborted.status.success(),
            "the merge stopped and git merge --abort refused: {}",
            text(&aborted.stderr)
        );
    }
    if files.is_empty() {
        // A merge git refused says why on stderr; one a hook stopped may say it on stdout.
        let said = if merged.stderr.is_empty() {
            text(&merged.stdout)
        } else {
            text(&merged.stderr)
        };
        anyhow::bail!("git merge refused, and nothing changed: {said}");
    }
    anyhow::bail!(
        "the merge stopped on conflicts in {}; it was aborted, and the main checkout is as it was",
        files.join(", ")
    )
}

/// What the main checkout `main` has that git ignores, relative to it (#585): each ignored file,
/// and each directory git ignores as a whole, marked `true`.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn ignored_entries(main: &Path) -> anyhow::Result<Vec<(PathBuf, bool)>> {
    let output = git(
        main,
        &[
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--directory",
            "-z",
        ],
    )
    .await?;
    anyhow::ensure!(
        output.status.success(),
        "git ls-files refused: {}",
        text(&output.stderr)
    );
    // Separated by NULs and never trimmed: a path may start or end with a space.
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let entry = String::from_utf8_lossy(entry);
            entry.strip_suffix('/').map_or_else(
                || (PathBuf::from(entry.as_ref()), false),
                |directory| (PathBuf::from(directory), true),
            )
        })
        .collect())
}

/// The file at `path` in the commit `revision` of the repository whose main checkout is `main`
/// (#585): what a worktree made from it gets; none when the commit has no such file.
///
/// # Errors
///
/// When `git` cannot run.
pub(crate) async fn committed_file(
    main: &Path,
    revision: &str,
    path: &str,
) -> anyhow::Result<Option<String>> {
    let object = format!("{revision}:{path}");
    let output = git(main, &["cat-file", "blob", &object]).await?;
    Ok(output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned()))
}

/// The key that keeps a repository's choice of setup command for new worktree agents (#585).
const SETUP_KEY: &str = "marley.worktreeSetup";

/// The setup command a worktree agent last started with in the repository whose main checkout is
/// `main`, `none` when the box was clear; `None` while nothing is kept.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn setup_choice(main: &Path) -> anyhow::Result<Option<String>> {
    let output = git(main, &["config", "--get", SETUP_KEY]).await?;
    match output.status.code() {
        Some(0) => Ok(Some(text(&output.stdout))),
        // `git config --get` says a key is unset with exit 1.
        Some(1) => Ok(None),
        _ => anyhow::bail!("git config refused: {}", text(&output.stderr)),
    }
}

/// Keeps `choice`, a setup command or `none`, for the repository whose main checkout is `main`.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn remember_setup(main: &Path, choice: &str) -> anyhow::Result<()> {
    let output = git(main, &["config", SETUP_KEY, choice]).await?;
    anyhow::ensure!(
        output.status.success(),
        "git config refused: {}",
        text(&output.stderr)
    );
    Ok(())
}

/// The name, under a branch's section, of the port slot of the worktree Marley made for it
/// (#590), as `base` is its base. Git keeps a variable's name in lower case.
const SLOT_KEY: &str = "marleyslot";

/// The port slot of the worktree at `folder`: `None` unless it is a linked worktree, on a branch
/// that holds one.
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn slot_of(folder: &Path, fs: &dyn Fs) -> anyhow::Result<Option<u16>> {
    // A linked worktree's `.git` is a file and the main checkout's a folder, so the main
    // checkout's terminals run no git here.
    if !fs.is_file(&folder.join(".git")).await {
        return Ok(None);
    }
    let output = git(folder, &["symbolic-ref", "--quiet", "--short", "HEAD"]).await?;
    // A detached HEAD has no branch to keep a slot under.
    if !output.status.success() {
        return Ok(None);
    }
    let key = format!("branch.{}.{SLOT_KEY}", text(&output.stdout));
    let output = git(folder, &["config", "--get", &key]).await?;
    match output.status.code() {
        Some(0) => Ok(text(&output.stdout).parse().ok()),
        // `git config --get` says a key is unset with exit 1.
        Some(1) => Ok(None),
        _ => anyhow::bail!("git config refused: {}", text(&output.stderr)),
    }
}

/// Gives `branch`, whose worktree was just made at `folder`, the lowest port slot from 1 that no
/// other worktree of the repository holds, and returns it. A removed worktree holds none, so its
/// slot is taken again.
///
/// # Errors
///
/// When `git` cannot run, or refuses, or every slot is taken.
pub(crate) async fn assign_slot(folder: &Path, branch: &str) -> anyhow::Result<u16> {
    let output = git(folder, &["worktree", "list", "--porcelain"]).await?;
    anyhow::ensure!(
        output.status.success(),
        "git worktree list refused: {}",
        text(&output.stderr)
    );
    let listed =
        git::repository::parse_worktrees_from_str(String::from_utf8_lossy(&output.stdout), None);
    let others: collections::HashSet<&str> = listed
        .iter()
        .filter_map(git::repository::Worktree::branch_name)
        .filter(|other| *other != branch)
        .collect();
    let pattern = format!(r"^branch\..*\.{SLOT_KEY}$");
    let output = git(folder, &["config", "--get-regexp", &pattern]).await?;
    let taken: collections::HashSet<u16> = match output.status.code() {
        Some(0) => text(&output.stdout)
            .lines()
            .filter_map(|line| {
                let (key, slot) = line.split_once(' ')?;
                let holder = key
                    .strip_prefix("branch.")?
                    .strip_suffix(SLOT_KEY)?
                    .strip_suffix('.')?;
                others.contains(holder).then(|| slot.parse().ok()).flatten()
            })
            .collect(),
        // `git config --get-regexp` says no key matched with exit 1.
        Some(1) => collections::HashSet::default(),
        _ => anyhow::bail!("git config refused: {}", text(&output.stderr)),
    };
    let slot = (1..=u16::MAX)
        .find(|slot| !taken.contains(slot))
        .context("every port slot is taken")?;
    let key = format!("branch.{branch}.{SLOT_KEY}");
    let output = git(folder, &["config", &key, &slot.to_string()]).await?;
    anyhow::ensure!(
        output.status.success(),
        "git config refused: {}",
        text(&output.stderr)
    );
    Ok(slot)
}

/// How many changes git has not committed in the checkout at `worktree`, untracked files
/// included, which a removal would lose (#589).
///
/// # Errors
///
/// When `git` cannot run, or refuses.
pub(crate) async fn uncommitted(worktree: &Path) -> anyhow::Result<usize> {
    let output = git(
        worktree,
        &["status", "--porcelain", "--untracked-files=normal"],
    )
    .await?;
    anyhow::ensure!(
        output.status.success(),
        "git status refused: {}",
        text(&output.stderr)
    );
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .count())
}

/// What became of a removed worktree's branch (#589).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BranchEnd {
    /// Deleted: git or the proof showed its commits in a target.
    Deleted,
    /// Kept, and why.
    Kept(String),
}

/// Deletes `branch` of the repository whose main checkout is `main` when its commits are in a
/// target: `git branch -d`, and when git refuses, a proof against `base`, `origin/HEAD` and the
/// main checkout's `HEAD`, after which the branch goes only if it still points at the commit the
/// proof read. Its `branch.<branch>` config, the base and the port slot, goes with it.
///
/// # Errors
///
/// When `git` cannot run, or refuses to delete a branch the proof showed merged.
pub(crate) async fn end_branch(
    main: &Path,
    branch: &str,
    base: Option<&str>,
) -> anyhow::Result<BranchEnd> {
    let deleted = git(main, &["branch", "-d", branch]).await?;
    if !deleted.status.success() {
        let reference = format!("refs/heads/{branch}");
        let commit = git(main, &["rev-parse", "--verify", "--quiet", &reference]).await?;
        if !commit.status.success() {
            return Ok(BranchEnd::Kept(format!("git could not find {branch}")));
        }
        let commit = text(&commit.stdout);
        let mut targets: Vec<&str> = base.into_iter().collect();
        targets.extend(["origin/HEAD", "HEAD"]);
        let mut merged = None;
        for target in targets {
            if merged_into(main, &commit, target).await? {
                merged = Some(target);
                break;
            }
        }
        let Some(target) = merged else {
            return Ok(BranchEnd::Kept(format!(
                "{branch} has commits that are not in {}",
                base.unwrap_or("HEAD")
            )));
        };
        log::info!("{branch} is merged into {target}; deleting it");
        // Deletes the branch only while it still points at the commit the proof read.
        let output = git(main, &["update-ref", "-d", &reference, &commit]).await?;
        anyhow::ensure!(
            output.status.success(),
            "git update-ref refused: {}",
            text(&output.stderr)
        );
    }
    let section = format!("branch.{branch}");
    let output = git(main, &["config", "--remove-section", &section]).await?;
    // Exit 128 when there is no such section, as after `branch -d`, which takes it.
    if !output.status.success() && output.status.code() != Some(128) {
        log::warn!("git config kept {section}: {}", text(&output.stderr));
    }
    Ok(BranchEnd::Deleted)
}

/// Whether the commits of `commit` are all in `target`: it is an ancestor of `target`; merging it
/// into `target` leaves `target`'s tree as it is, as after a squash merge; or `git cherry` finds
/// each of its commits' changes in `target`. A target that does not resolve holds nothing.
async fn merged_into(main: &Path, commit: &str, target: &str) -> anyhow::Result<bool> {
    let resolved = git(main, &["rev-parse", "--verify", "--quiet", target]).await?;
    if !resolved.status.success() {
        return Ok(false);
    }
    let ancestor = git(main, &["merge-base", "--is-ancestor", commit, target]).await?;
    if ancestor.status.success() {
        return Ok(true);
    }
    let tree = format!("{target}^{{tree}}");
    let target_tree = git(main, &["rev-parse", &tree]).await?;
    let merged_tree = git(main, &["merge-tree", "--write-tree", target, commit]).await?;
    if target_tree.status.success()
        && merged_tree.status.success()
        && text(&merged_tree.stdout) == text(&target_tree.stdout)
    {
        return Ok(true);
    }
    let cherry = git(main, &["cherry", target, commit]).await?;
    let marks = text(&cherry.stdout);
    Ok(cherry.status.success()
        && !marks.is_empty()
        && marks.lines().all(|line| line.starts_with('-')))
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
