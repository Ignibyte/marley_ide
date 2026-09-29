//! Worktree agents (#510): an agent CLI in a git worktree and on a branch of its own.
//!
//! A project's + in the rail offers New Agent in Worktree for a local project with a git
//! repository. Its prompt names the branch and the base; Enter makes the worktree through Zed's
//! worktree service on `agent/<name>`, from the main checkout's branch, writes the base as
//! `branch.agent/<name>.base` in the repository's config, and starts the agent in a center
//! terminal of the worktree's workspace with its first prompt and its project's permission mode
//! (#532). The routing's first-terminal seed passes over that workspace. Since #585 the worktree
//! first gets the main checkout's gitignored files its `.worktreeinclude` names, and the prompt
//! offers the install command of the repository's one JavaScript package manager, run before the
//! agent when its box is checked.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context as _;
use editor::Editor;
use fs::Fs;
use git_ui_core::{worktree_names, worktree_service};
use gpui::{
    App, AppContext as _, AsyncWindowContext, Context, DismissEvent, Entity, EventEmitter,
    FocusHandle, Focusable, Global, Render, Task, WeakEntity, Window,
};
use marley_agent::AgentKind;
use project::TaskSourceKind;
use project::git_store::{Repository, RepositorySnapshot};
use project::project_settings::ProjectSettings;
use settings::Settings as _;
use task::{TaskHook, TaskTemplates};
use ui::{Checkbox, ToggleState, prelude::*};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};
use zed_actions::{CreateWorktree, NewWorktreeBranchTarget};

use crate::{StartWorktreeAgent, agent_trust, worktree_git, worktree_include};

/// Where a worktree agent's branch lives.
const BRANCH_PREFIX: &str = "agent/";

/// How many names a create tries when a folder already sits where the worktree would go.
const NAME_TRIES: usize = 3;

/// A lockfile at a repository's root beside its `package.json`, and its manager's install command
/// (Orca's table, #585). Lockfiles of two managers suggest nothing.
const LOCKFILES: [(&str, &str); 5] = [
    ("pnpm-lock.yaml", "pnpm install"),
    ("bun.lock", "bun install"),
    ("bun.lockb", "bun install"),
    ("yarn.lock", "yarn install"),
    ("package-lock.json", "npm install"),
];

/// What `marley.worktreeSetup` keeps for a clear box.
const NO_SETUP: &str = "none";

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

/// Marks `folder` for the first-terminal seed to pass over, as a worktree agent's is (#511): a
/// worktree Review opens shows its diff alone.
pub(crate) fn skip_seed(folder: PathBuf, cx: &mut App) {
    cx.default_global::<WorktreeAgents>()
        .seed_skips
        .push(folder);
}

/// Drops `folder`'s mark when the seed did not take it (#511), so the folder opened later gets
/// its first terminal.
pub(crate) fn drop_seed_skip(folder: &Path, cx: &mut App) {
    if let Some(agents) = cx.try_global::<WorktreeAgents>()
        && agents.seed_skips.iter().any(|marked| marked == folder)
    {
        cx.global_mut::<WorktreeAgents>()
            .seed_skips
            .retain(|marked| marked != folder);
    }
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
        let hooks = global_create_worktree_tasks(workspace, cx);
        let handle = cx.entity().downgrade();
        workspace.toggle_modal(window, cx, |window, cx| {
            WorktreePrompt::new(handle, kind, plan, hooks, window, cx)
        });
    });
    Ok(())
}

/// Whether the user's global tasks have one Zed runs when it makes a worktree (#585): then every
/// worktree is set up by it.
fn global_create_worktree_tasks(workspace: &Workspace, cx: &App) -> bool {
    let project = workspace.project().read(cx);
    let (Some(worktree), Some(inventory)) = (
        project.visible_worktrees(cx).next(),
        project.task_store().read(cx).task_inventory(),
    ) else {
        return false;
    };
    let hooks: collections::HashSet<TaskHook> = std::iter::once(TaskHook::CreateWorktree).collect();
    inventory
        .read(cx)
        .templates_with_hooks(&hooks, worktree.read(cx).id())
        .iter()
        .any(|(kind, _)| matches!(kind, TaskSourceKind::AbsPath { .. }))
}

/// Whether the `.zed/tasks.json` committed at `base`, the one a worktree made from it gets, has a
/// task Zed runs when it makes a worktree (#585).
async fn base_create_worktree_tasks(main: &Path, base: &str) -> bool {
    let file = worktree_git::committed_file(main, base, ".zed/tasks.json")
        .await
        .log_err()
        .flatten();
    file.and_then(|file| settings::parse_json_with_comments::<TaskTemplates>(&file).log_err())
        .is_some_and(|templates| {
            templates
                .0
                .iter()
                .any(|template| template.hooks.contains(&TaskHook::CreateWorktree))
        })
}

/// The install command a new worktree of a repository is offered (#585).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SetupOffer {
    command: String,
    /// The lockfile that suggests it.
    lockfile: String,
    /// Whether its box is checked: the repository keeps this command.
    checked: bool,
}

/// The offer for a worktree made from `base` of the repository whose main checkout is `main`: a
/// `package.json` and one manager's lockfiles at its root and no `create_worktree` task at
/// `base`, the box checked when `marley.worktreeSetup` keeps that command.
async fn setup_offer(main: &Path, base: &str, fs: &dyn Fs) -> Option<SetupOffer> {
    if !fs.is_file(&main.join("package.json")).await || base_create_worktree_tasks(main, base).await
    {
        return None;
    }
    let mut found: Vec<(&str, &str)> = Vec::new();
    for (lockfile, command) in LOCKFILES {
        if fs.is_file(&main.join(lockfile)).await {
            found.push((lockfile, command));
        }
    }
    let (lockfile, command) = *found.first()?;
    if found.iter().any(|(_, other)| *other != command) {
        return None;
    }
    let kept = worktree_git::setup_choice(main).await.log_err().flatten();
    Some(SetupOffer {
        command: command.to_string(),
        lockfile: lockfile.to_string(),
        checked: kept.as_deref() == Some(command),
    })
}

/// What a worktree agent starts with: its first prompt, and the setup line as the prompt left it
/// (#585).
struct Launch {
    prompt: String,
    setup: Option<SetupOffer>,
}

/// The first prompt for a worktree agent, over a line that names the branch and its base, and
/// since #585 a setup command's line.
pub(crate) struct WorktreePrompt {
    editor: Entity<Editor>,
    workspace: WeakEntity<Workspace>,
    kind: AgentKind,
    plan: Plan,
    /// The setup command offered, once read.
    setup: Option<SetupOffer>,
    _reading: Task<()>,
}

impl WorktreePrompt {
    fn new(
        workspace: WeakEntity<Workspace>,
        kind: AgentKind,
        plan: Plan,
        hooks: bool,
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
        // Global create_worktree tasks set every worktree up themselves.
        let reading = if hooks {
            Task::ready(())
        } else {
            let (main, base, fs) = (plan.main.clone(), plan.base.clone(), <dyn Fs>::global(cx));
            cx.spawn(async move |prompt, cx| {
                let offer = cx
                    .background_spawn(async move { setup_offer(&main, &base, fs.as_ref()).await })
                    .await;
                prompt
                    .update(cx, |prompt, cx| {
                        prompt.setup = offer;
                        cx.notify();
                    })
                    .log_err();
            })
        };
        Self {
            editor,
            workspace,
            kind,
            plan,
            setup: None,
            _reading: reading,
        }
    }

    fn confirm(&mut self, _: &StartWorktreeAgent, window: &mut Window, cx: &mut Context<Self>) {
        let launch = Launch {
            prompt: self.editor.read(cx).text(cx),
            setup: self.setup.clone(),
        };
        start(
            self.workspace.clone(),
            self.kind,
            self.plan.clone(),
            launch,
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
            .children(self.setup.as_ref().map(|setup| {
                Checkbox::new("marley-worktree-setup", ToggleState::from(setup.checked))
                    .label(format!(
                        "Setup: {} ({} found)",
                        setup.command, setup.lockfile
                    ))
                    .label_size(LabelSize::Small)
                    .on_click(cx.listener(|prompt, state: &ToggleState, _, cx| {
                        if let Some(setup) = &mut prompt.setup {
                            setup.checked = state.selected();
                            cx.notify();
                        }
                    }))
            }))
    }
}

/// Makes the worktree `plan` names and starts `kind` in it with `launch`'s prompt, after its setup
/// command when its box is checked, unless a worktree of the project is being made already.
fn start(
    workspace: WeakEntity<Workspace>,
    kind: AgentKind,
    plan: Plan,
    launch: Launch,
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
            NotificationId::unique::<WorktreePrompt>(),
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
            create(&workspace, kind, &plan, launch, &fs, &setting, cx).await;
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
    launch: Launch,
    fs: &Arc<dyn Fs>,
    setting: &str,
    cx: &mut AsyncWindowContext,
) {
    let Launch { prompt, setup } = launch;
    let Some((name, path)) = free_folder(plan, fs, setting, cx).await else {
        cx.update(|_, cx| {
            show_toast(
                workspace,
                NotificationId::unique::<WorktreePrompt>(),
                "No worktree agent: a folder already sits where each worktree would go.".into(),
                cx,
            );
        })
        .log_err();
        return;
    };
    let branch = format!("{BRANCH_PREFIX}{name}");
    let action = CreateWorktree {
        worktree_name: Some(name.clone()),
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
    copy_included(workspace, plan, &name, &path, fs, cx).await;
    if let Err(error) = write_base(&path, &branch, &plan.base).await {
        cx.update(|_, cx| {
            show_toast(
                workspace,
                NotificationId::unique::<WorktreePrompt>(),
                format!("The worktree's base was not written: {error:#}"),
                cx,
            );
        })
        .log_err();
    }
    let command = remember_setup(workspace, plan, setup.as_ref(), cx).await;
    let wait = if command.is_some() {
        agent_trust::WATCH_AFTER_SETUP
    } else {
        agent_trust::WATCH_FOR
    };
    let Some(launch) = created
        .workspace
        .update_in(cx, |workspace, window, cx| {
            crate::agents::start_cli_with_prompt(workspace, kind, prompt, command, window, cx)
        })
        .log_err()
    else {
        return;
    };
    let Some(terminal) = launch.await else {
        return;
    };
    // Claude Code asks whether to trust a repository it has not trusted yet (#587).
    if kind == AgentKind::Claude {
        let worktree = created.workspace.downgrade();
        let source = workspace.clone();
        cx.update(|_, cx| agent_trust::watch(&terminal, worktree, source, name, wait, cx))
            .log_err();
    }
}

/// Copies into the new worktree at `path` the main checkout's gitignored files its
/// `.worktreeinclude` names (#585), and names what it left out in a toast. Zed's own
/// `create_worktree` tasks start with the worktree; this lands before the agent does.
async fn copy_included(
    workspace: &WeakEntity<Workspace>,
    plan: &Plan,
    name: &str,
    path: &Path,
    fs: &Arc<dyn Fs>,
    cx: &mut AsyncWindowContext,
) {
    let copying = {
        let (main, path, fs) = (plan.main.clone(), path.to_path_buf(), Arc::clone(fs));
        cx.background_spawn(async move {
            worktree_include::copy_included(&main, &path, fs.as_ref()).await
        })
    };
    let left_out = match copying.await {
        Ok(included) => included.skipped,
        Err(error) => vec![format!("{error:#}")],
    };
    if left_out.is_empty() {
        return;
    }
    let message = format!(
        "Not copied into {name} from {}: {}",
        worktree_include::INCLUDE_FILE,
        left_out.join("; ")
    );
    cx.update(|_, cx| {
        show_toast(
            workspace,
            NotificationId::unique::<worktree_include::Included>(),
            message,
            cx,
        );
    })
    .log_err();
}

/// Keeps the setup line's choice for the repository, the command or `none`, when the prompt
/// offered one (#585), and gives the command to run; a choice not kept shows in a toast.
async fn remember_setup(
    workspace: &WeakEntity<Workspace>,
    plan: &Plan,
    setup: Option<&SetupOffer>,
    cx: &mut AsyncWindowContext,
) -> Option<String> {
    let setup = setup?;
    let command = setup.checked.then(|| setup.command.clone());
    let choice = command.as_deref().unwrap_or(NO_SETUP);
    if let Err(error) = worktree_git::remember_setup(&plan.main, choice).await {
        cx.update(|_, cx| {
            show_toast(
                workspace,
                NotificationId::unique::<SetupOffer>(),
                format!("The setup choice was not kept: {error:#}"),
                cx,
            );
        })
        .log_err();
    }
    command
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

fn show_toast(
    workspace: &WeakEntity<Workspace>,
    id: NotificationId,
    message: String,
    cx: &mut App,
) {
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(Toast::new(id, message), cx);
        })
        .log_err();
}
