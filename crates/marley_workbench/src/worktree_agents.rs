//! Worktree agents (#510): an agent CLI in a git worktree and on a branch of its own.
//!
//! A project's + in the rail offers New Agent in Worktree for a local project with a git
//! repository. Its prompt names the branch and the base; Enter makes the worktree through Zed's
//! worktree service on `agent/<name>`, from the main checkout's branch, writes the base as
//! `branch.agent/<name>.base` in the repository's config, and starts the agent in a center
//! terminal of the worktree's workspace with its first prompt and its project's permission mode
//! (#532). The routing's first-terminal seed passes over that workspace.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context as _;
use editor::Editor;
use fs::Fs;
use git_ui_core::{worktree_names, worktree_service};
use gpui::{
    App, AppContext as _, AsyncWindowContext, Context, DismissEvent, Entity, EventEmitter,
    FocusHandle, Focusable, Global, Render, WeakEntity, Window,
};
use marley_agent::AgentKind;
use project::git_store::{Repository, RepositorySnapshot};
use project::project_settings::ProjectSettings;
use settings::Settings as _;
use ui::prelude::*;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};
use zed_actions::{CreateWorktree, NewWorktreeBranchTarget};

use crate::StartWorktreeAgent;

/// Where a worktree agent's branch lives.
const BRANCH_PREFIX: &str = "agent/";

/// How many names a create tries when a folder already sits where the worktree would go.
const NAME_TRIES: usize = 3;

/// The worktrees being made, and the folders the first-terminal seed passes over.
#[derive(Debug, Default)]
struct WorktreeAgents {
    /// The projects with a worktree being made, by their main folder: one at a time for each.
    creating: HashSet<PathBuf>,
    /// The folders of worktrees being made, whose workspaces get the agent's terminal and no
    /// other.
    seed_skips: Vec<PathBuf>,
}

impl Global for WorktreeAgents {}

/// Whether the first-terminal seed passes over a workspace whose first root is `root`: a
/// worktree agent's, which gets the agent's terminal instead; the mark goes with the answer.
/// `new_local` canonicalizes a root, so the two compare by their last two parts,
/// `<name>/<project>`.
pub(crate) fn take_seed_skip(root: &Path, cx: &mut App) -> bool {
    let at = cx.try_global::<WorktreeAgents>().and_then(|agents| {
        agents
            .seed_skips
            .iter()
            .position(|marked| last_two(marked) == last_two(root))
    });
    at.is_some_and(|at| {
        cx.global_mut::<WorktreeAgents>().seed_skips.remove(at);
        true
    })
}

/// A path's last two parts, last first.
fn last_two(path: &Path) -> Vec<std::path::Component<'_>> {
    path.components().rev().take(2).collect()
}

/// What a New Agent in Worktree will make.
#[derive(Debug, Clone)]
struct Plan {
    repository: Entity<Repository>,
    /// The main checkout's folder, which names the project for the one-at-a-time guard.
    main: PathBuf,
    /// The worktree's name, which the branch and the folder take.
    name: String,
    /// The main checkout's branch, or its commit when it is detached.
    base: String,
}

impl Plan {
    fn branch(&self) -> String {
        format!("{BRANCH_PREFIX}{}", self.name)
    }
}

/// Whether `workspace`'s project can make a worktree agent: a local project whose first folder is
/// a git repository.
pub(crate) fn offered(workspace: &Workspace, cx: &App) -> bool {
    workspace.project().read(cx).is_local() && repository_of(workspace, cx).is_some()
}

/// The repository whose folder is `workspace`'s first.
fn repository_of(workspace: &Workspace, cx: &App) -> Option<Entity<Repository>> {
    let project = workspace.project().read(cx);
    let root = project.visible_worktrees(cx).next()?.read(cx).abs_path();
    project
        .git_store()
        .read(cx)
        .repositories()
        .values()
        .find(|repository| *repository.read(cx).work_directory_abs_path == *root)
        .cloned()
}

/// The plan for `workspace`'s project: its repository, the main checkout's branch, and a name no
/// worktree and no `agent/` branch has.
fn plan(workspace: &Workspace, cx: &App) -> anyhow::Result<Plan> {
    anyhow::ensure!(
        workspace.project().read(cx).is_local(),
        "a worktree agent needs a project on this machine"
    );
    let repository =
        repository_of(workspace, cx).context("the project's folder is not a git repository")?;
    let snapshot = repository.read(cx);
    let main = snapshot
        .main_worktree_abs_path()
        .context("the repository has no main checkout")?
        .to_path_buf();
    let base = main_base(snapshot).context("the main checkout has no branch or commit")?;
    let name = free_name(snapshot, &[]).context("every name Zed makes is taken")?;
    Ok(Plan {
        repository,
        main,
        name,
        base,
    })
}

/// The main checkout's branch, or its commit when it is detached.
fn main_base(snapshot: &RepositorySnapshot) -> Option<String> {
    if snapshot.is_main_worktree() {
        return snapshot
            .branch
            .as_ref()
            .map(|branch| branch.name().to_string())
            .or_else(|| {
                snapshot
                    .head_commit
                    .as_ref()
                    .map(|commit| commit.sha.to_string())
            });
    }
    let main = snapshot
        .linked_worktrees()
        .iter()
        .find(|worktree| worktree.is_main)?;
    Some(
        main.branch_name()
            .map_or_else(|| main.sha.to_string(), str::to_string),
    )
}

/// A name Zed's generator makes that no worktree of `snapshot`, no `agent/` branch and nothing in
/// `also` has.
fn free_name(snapshot: &RepositorySnapshot, also: &[String]) -> Option<String> {
    let mut taken: Vec<String> = also.to_vec();
    for worktree in snapshot.linked_worktrees() {
        // Zed's layout puts a worktree at `<name>/<project>`, other layouts at `<name>`.
        let names = [
            worktree.path.file_name(),
            worktree.path.parent().and_then(Path::file_name),
        ];
        taken.extend(
            names
                .into_iter()
                .flatten()
                .filter_map(|name| name.to_str())
                .map(str::to_string),
        );
    }
    taken.extend(
        snapshot
            .branch_list
            .iter()
            .filter_map(|branch| branch.name().strip_prefix(BRANCH_PREFIX))
            .map(str::to_string),
    );
    let taken: Vec<&str> = taken.iter().map(String::as_str).collect();
    worktree_names::generate_worktree_name(&taken, &mut rand::rng())
}

/// Opens the prompt for a worktree agent running `kind` in `workspace`'s project, or says why it
/// cannot.
pub(crate) fn open_prompt(
    workspace: &WeakEntity<Workspace>,
    kind: AgentKind,
    window: &mut Window,
    cx: &mut App,
) -> anyhow::Result<()> {
    let workspace = workspace.upgrade().context("the project was closed")?;
    workspace.update(cx, |workspace, cx| {
        let plan = match plan(workspace, cx) {
            Ok(plan) => plan,
            Err(error) => {
                workspace.show_toast(
                    Toast::new(
                        NotificationId::unique::<WorktreePrompt>(),
                        format!("No worktree agent: {error:#}"),
                    ),
                    cx,
                );
                return;
            }
        };
        let handle = cx.entity().downgrade();
        workspace.toggle_modal(window, cx, |window, cx| {
            WorktreePrompt::new(handle, kind, plan, window, cx)
        });
    });
    Ok(())
}

/// The first prompt for a worktree agent, over a line that names the branch and its base.
pub(crate) struct WorktreePrompt {
    editor: Entity<Editor>,
    workspace: WeakEntity<Workspace>,
    kind: AgentKind,
    plan: Plan,
}

impl WorktreePrompt {
    fn new(
        workspace: WeakEntity<Workspace>,
        kind: AgentKind,
        plan: Plan,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let placeholder = format!("A first prompt for {}, or none", kind.display_name());
        let editor = cx.new(|cx| {
            let mut editor = Editor::auto_height(1, 8, window, cx);
            editor.set_soft_wrap_mode(settings::SoftWrap::EditorWidth, cx);
            editor.set_placeholder_text(&placeholder, window, cx);
            editor
        });
        Self {
            editor,
            workspace,
            kind,
            plan,
        }
    }

    fn confirm(&mut self, _: &StartWorktreeAgent, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = self.editor.read(cx).text(cx);
        start(
            self.workspace.clone(),
            self.kind,
            self.plan.clone(),
            prompt,
            window,
            cx,
        );
        cx.emit(DismissEvent);
    }
}

impl ModalView for WorktreePrompt {}

impl EventEmitter<DismissEvent> for WorktreePrompt {}

impl Focusable for WorktreePrompt {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl Render for WorktreePrompt {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .key_context("MarleyWorktreePrompt")
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(|_, _: &menu::Cancel, _, cx| cx.emit(DismissEvent)))
            .w(rems(34.))
            .elevation_3(cx)
            .p_3()
            .gap_2()
            .child(
                Headline::new(format!("{} in a New Worktree", self.kind.display_name()))
                    .size(HeadlineSize::Small),
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.editor_background)
                    .child(self.editor.clone()),
            )
            .child(
                Label::new(format!("{} from {}", self.plan.branch(), self.plan.base))
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
    }
}

/// Makes the worktree `plan` names and starts `kind` in it with `prompt`, unless a worktree of
/// the project is being made already.
fn start(
    workspace: WeakEntity<Workspace>,
    kind: AgentKind,
    plan: Plan,
    prompt: String,
    window: &Window,
    cx: &mut App,
) {
    if !cx
        .default_global::<WorktreeAgents>()
        .creating
        .insert(plan.main.clone())
    {
        show_toast(
            &workspace,
            "A worktree of this project is being made; start the next agent once it is.".into(),
            cx,
        );
        return;
    }
    let fs = <dyn Fs>::global(cx);
    let setting = ProjectSettings::get_global(cx)
        .git
        .worktree_directory
        .clone();
    window
        .spawn(cx, async move |cx| {
            create(&workspace, kind, &plan, prompt, &fs, &setting, cx).await;
            cx.update(|_, cx| {
                cx.default_global::<WorktreeAgents>()
                    .creating
                    .remove(&plan.main);
            })
            .log_err();
        })
        .detach();
}

/// The create itself. Zed's service shows why its own steps fail; Marley shows why its own do.
async fn create(
    workspace: &WeakEntity<Workspace>,
    kind: AgentKind,
    plan: &Plan,
    prompt: String,
    fs: &Arc<dyn Fs>,
    setting: &str,
    cx: &mut AsyncWindowContext,
) {
    let Some((name, path)) = free_folder(plan, fs, setting, cx).await else {
        cx.update(|_, cx| {
            show_toast(
                workspace,
                "No worktree agent: a folder already sits where each worktree would go.".into(),
                cx,
            );
        })
        .log_err();
        return;
    };
    let branch = format!("{BRANCH_PREFIX}{name}");
    let action = CreateWorktree {
        worktree_name: Some(name),
        branch_target: NewWorktreeBranchTarget::ExistingBranch {
            name: plan.base.clone(),
        },
    };
    // The seed's mark goes in before the workspace is made, which is inside the create.
    cx.update(|_, cx| {
        cx.default_global::<WorktreeAgents>()
            .seed_skips
            .push(path.clone());
    })
    .log_err();
    let creating = workspace.update_in(cx, |workspace, window, cx| {
        worktree_service::create_worktree_workspace_on_branch(
            workspace,
            &action,
            branch.clone(),
            window,
            None,
            cx,
        )
    });
    let created = match creating {
        Ok(creating) => creating.await,
        Err(error) => Err(error),
    };
    // A mark the seed did not take goes, so the folder opened later gets its first terminal.
    cx.update(|_, cx| {
        cx.default_global::<WorktreeAgents>()
            .seed_skips
            .retain(|marked| *marked != path);
    })
    .log_err();
    let created = match created {
        Ok(created) => created,
        Err(error) => {
            // Zed's service has shown why.
            log::warn!("no worktree agent: {error:#}");
            return;
        }
    };
    if let Err(error) = write_base(&path, &branch, &plan.base).await {
        cx.update(|_, cx| {
            show_toast(
                workspace,
                format!("The worktree's base was not written: {error:#}"),
                cx,
            );
        })
        .log_err();
    }
    created
        .workspace
        .update_in(cx, |workspace, window, cx| {
            crate::agents::start_cli_with_prompt(workspace, kind, prompt, window, cx);
        })
        .log_err();
}

/// The name and the folder of a worktree with nothing at its folder: Zed's rollback of a refused
/// create removes the folder with force, so a folder already there is never offered.
async fn free_folder(
    plan: &Plan,
    fs: &Arc<dyn Fs>,
    setting: &str,
    cx: &mut AsyncWindowContext,
) -> Option<(String, PathBuf)> {
    let mut tried: Vec<String> = Vec::new();
    let mut name = plan.name.clone();
    for _ in 0..NAME_TRIES {
        let path = cx
            .update(|_, cx| {
                plan.repository
                    .read(cx)
                    .path_for_new_linked_worktree(&name, setting)
            })
            .ok()?
            .log_err()?;
        if matches!(fs.metadata(&path).await, Ok(None)) {
            return Some((name, path));
        }
        tried.push(name);
        name = cx
            .update(|_, cx| free_name(plan.repository.read(cx), &tried))
            .ok()??;
    }
    None
}

/// Writes `base` as `branch.<branch>.base` in the repository's config, which every worktree of
/// it reads, from the worktree at `path`.
async fn write_base(path: &Path, branch: &str, base: &str) -> anyhow::Result<()> {
    let output = util::command::new_command("git")
        .current_dir(path)
        .args(["config", &format!("branch.{branch}.base"), base])
        .output()
        .await
        .context("running git config")?;
    anyhow::ensure!(
        output.status.success(),
        "git config refused: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

fn show_toast(workspace: &WeakEntity<Workspace>, message: String, cx: &mut App) {
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<WorktreePrompt>(), message),
                cx,
            );
        })
        .log_err();
}
