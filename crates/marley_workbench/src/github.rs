//! The GitHub CLI, asked about a branch's pull request (#531).
//!
//! `gh pr list --state all --limit 1` with the branch as `--head`, so a branch with no pull
//! request is an empty list, and a merged or closed one still answers. The branch and the
//! repository are single arguments in their `--flag=value` form and no shell runs, since a branch
//! name may hold `$`, `;` and parentheses. #541's process adapter takes the spawn over.

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
}

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
            "--json=number,state,isDraft,title,url,baseRefName",
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
    }))
}
