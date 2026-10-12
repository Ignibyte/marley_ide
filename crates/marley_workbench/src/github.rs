//! The GitHub CLI, asked about a branch's pull request (#531).
//!
//! `gh pr list --state all --limit 1` with the branch as `--head`, so a branch with no pull
//! request is an empty list, and a merged or closed one still answers. The branch and the
//! repository are single arguments in their `--flag=value` form and no shell runs, since a branch
//! name may hold `$`, `;` and parentheses. #541's process adapter takes the spawn over.
//!
//! Since #718 the same call reads the pull request's checks (`statusCheckRollup`) and its head
//! commit, and `failed_log` reads a GitHub Actions job's failed steps' log.

use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;

/// The variable that names the `gh` this runs, for scenarios, as `MARLEY_GIT` names git: Marley
/// takes its PATH from the user's login shell, so a stand-in first on the PATH may not be first.
const GH_OVERRIDE: &str = "MARLEY_GH";

/// A branch's pull request, as the rail's chip shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PullRequest {
    pub(crate) number: u64,
    pub(crate) state: PullRequestState,
    pub(crate) title: String,
    pub(crate) url: String,
    /// The branch it merges into.
    pub(crate) base: String,
    /// The head commit's SHA (#718).
    pub(crate) head: String,
    /// Its checks, as GitHub rolls them up for the head commit (#718).
    pub(crate) checks: Vec<Check>,
}

/// How a check stands (#718).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckOutcome {
    Failed,
    Pending,
    Passed,
    /// Shown, never sent to an agent.
    Cancelled,
}

/// One check of a pull request: a GitHub check run or a commit status (#718).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Check {
    /// The check run's name, or the status's context.
    pub(crate) name: String,
    /// The workflow a check run belongs to; none for a status.
    pub(crate) workflow: Option<String>,
    pub(crate) outcome: CheckOutcome,
    /// Where GitHub shows it.
    pub(crate) link: Option<String>,
}

impl Check {
    /// The GitHub Actions job its link names (`…/actions/runs/<run>/job/<job>`), whose log `gh`
    /// can read; none for another CI's check.
    pub(crate) fn job(&self) -> Option<&str> {
        let link = self.link.as_deref()?;
        let (before, job) = link.trim_end_matches('/').rsplit_once("/job/")?;
        (before.contains("/actions/runs/")
            && !job.is_empty()
            && job.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(job)
    }
}

/// Where a pull request stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PullRequestState {
    Open,
    Draft,
    Merged,
    Closed,
}

impl PullRequestState {
    /// The state in the chip's tooltip.
    pub(crate) const fn words(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Draft => "Draft",
            Self::Merged => "Merged",
            Self::Closed => "Closed",
        }
    }
}

/// One pull request as `gh pr list --json` prints it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Listed {
    number: u64,
    state: String,
    is_draft: bool,
    title: String,
    url: String,
    base_ref_name: String,
    #[serde(default)]
    head_ref_oid: String,
    #[serde(default)]
    status_check_rollup: Vec<Rollup>,
}

/// One entry of `statusCheckRollup`, in the two shapes GitHub gives (`gh` 2.102.0).
#[derive(Deserialize)]
#[serde(tag = "__typename")]
enum Rollup {
    #[serde(rename_all = "camelCase")]
    CheckRun {
        name: String,
        #[serde(default)]
        workflow_name: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        conclusion: Option<String>,
        #[serde(default)]
        details_url: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    StatusContext {
        context: String,
        #[serde(default)]
        state: Option<String>,
        #[serde(default)]
        target_url: Option<String>,
    },
    #[serde(other)]
    Other,
}

impl Rollup {
    fn check(self) -> Option<Check> {
        match self {
            Self::CheckRun {
                name,
                workflow_name,
                status,
                conclusion,
                details_url,
            } => {
                let outcome = if status.as_deref() == Some("COMPLETED") {
                    match conclusion.as_deref() {
                        Some("FAILURE" | "TIMED_OUT" | "STARTUP_FAILURE" | "ACTION_REQUIRED") => {
                            CheckOutcome::Failed
                        }
                        Some("CANCELLED" | "STALE") => CheckOutcome::Cancelled,
                        _ => CheckOutcome::Passed,
                    }
                } else {
                    CheckOutcome::Pending
                };
                Some(Check {
                    name,
                    workflow: workflow_name.filter(|workflow| !workflow.is_empty()),
                    outcome,
                    link: details_url.filter(|link| !link.is_empty()),
                })
            }
            Self::StatusContext {
                context,
                state,
                target_url,
            } => Some(Check {
                name: context,
                workflow: None,
                outcome: match state.as_deref() {
                    Some("FAILURE" | "ERROR") => CheckOutcome::Failed,
                    Some("SUCCESS") => CheckOutcome::Passed,
                    _ => CheckOutcome::Pending,
                },
                link: target_url.filter(|link| !link.is_empty()),
            }),
            Self::Other => None,
        }
    }
}

/// How much of a job's failed steps' log is kept: its end, where the error is.
const LOG_KEEP: usize = 64 * 1024;

/// The newest pull request of `branch` in the GitHub repository `owner_repo` (`owner/repo`),
/// open or not, asked from `folder`; `None` when the branch has none.
///
/// # Errors
///
/// When `gh` cannot run (not installed, or not on the PATH Marley started with), refuses (not
/// logged in, no network), or answers something it cannot read.
pub(crate) async fn pull_request(
    folder: &Path,
    owner_repo: &str,
    branch: &str,
) -> anyhow::Result<Option<PullRequest>> {
    let program = std::env::var(GH_OVERRIDE).unwrap_or_else(|_| "gh".to_string());
    let output = crate::process::output(
        program,
        [
            "pr",
            "list",
            &format!("--repo={owner_repo}"),
            &format!("--head={branch}"),
            "--state=all",
            "--limit=1",
            "--json=number,state,isDraft,title,url,baseRefName,headRefOid,statusCheckRollup",
        ],
        Some(folder),
        &[("GH_PROMPT_DISABLED", "1"), ("NO_COLOR", "1")],
    )
    .await
    .context("running gh")?;
    anyhow::ensure!(
        output.status.success(),
        "gh pr list refused: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let listed: Vec<Listed> =
        serde_json::from_slice(&output.stdout).context("reading gh pr list's answer")?;
    Ok(listed.into_iter().next().map(|listed| PullRequest {
        number: listed.number,
        state: match listed.state.as_str() {
            "MERGED" => PullRequestState::Merged,
            "CLOSED" => PullRequestState::Closed,
            _ if listed.is_draft => PullRequestState::Draft,
            _ => PullRequestState::Open,
        },
        title: listed.title,
        url: listed.url,
        base: listed.base_ref_name,
        head: listed.head_ref_oid,
        checks: listed
            .status_check_rollup
            .into_iter()
            .filter_map(Rollup::check)
            .collect(),
    }))
}

/// The failed steps' log of the GitHub Actions job `job` in `owner_repo`, asked from `folder`:
/// its last 64 KiB, cut at a line (#718).
///
/// # Errors
///
/// When `gh` cannot run, or refuses: the log expired, the job is unknown, or no network.
pub(crate) async fn failed_log(
    folder: &Path,
    owner_repo: &str,
    job: &str,
) -> anyhow::Result<String> {
    let program = std::env::var(GH_OVERRIDE).unwrap_or_else(|_| "gh".to_string());
    let output = crate::process::output(
        program,
        [
            "run",
            "view",
            &format!("--repo={owner_repo}"),
            &format!("--job={job}"),
            "--log-failed",
        ],
        Some(folder),
        &[("GH_PROMPT_DISABLED", "1"), ("NO_COLOR", "1")],
    )
    .await
    .context("running gh")?;
    anyhow::ensure!(
        output.status.success(),
        "gh run view refused: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let log = String::from_utf8_lossy(&output.stdout);
    Ok(last_bytes(&log, LOG_KEEP).to_string())
}

/// The end of `text` within `limit` bytes, from the start of a line where one falls inside it.
pub(crate) fn last_bytes(text: &str, limit: usize) -> &str {
    if text.len() <= limit {
        return text;
    }
    let mut start = text.len() - limit;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    let tail = &text[start..];
    // A tail whose only newline ends it stays whole.
    tail.find('\n')
        .filter(|newline| newline + 1 < tail.len())
        .map_or(tail, |newline| &tail[newline + 1..])
}
