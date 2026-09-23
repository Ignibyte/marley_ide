//! Driven tests for starting agents: the New Agent picker, its entries and what each starts. No
//! test starts a real agent: every terminal is display-only, and the agent CLIs are empty
//! programs in a temporary directory.

use agent_ui::test_support::active_thread_id;
use gpui::{KeyBinding, TestAppContext, UpdateGlobal as _, VisualTestContext, actions};
use project::agent_registry_store::{RegistryAgentMetadata, RegistryNpxAgent};
use settings::{SaturatingBool, SettingsStore};
use terminal_view::TerminalView;
use util::path;

use super::*;
use crate::marley_workbench_tests::{
    add_agent_panel, configure_agents, init_agent_test, open_projects, programs_in,
    search_agents_in, use_display_only_terminals,
};

/// A window over one project with an Agent Panel and the custom agents `stub` and `zeta`, where
/// the registry names `zeta` "Zeta Code" and gives it an icon; `claude` and `codex` are on the
/// search path, and every terminal is display-only. The directory must outlive the test.
async fn open_project(
    cx: &mut TestAppContext,
) -> (
    Entity<Workspace>,
    Entity<AgentPanel>,
    tempfile::TempDir,
    &mut VisualTestContext,
) {
    init_agent_test(cx);
    cx.update(crate::init);
    cx.update(|cx| {
        AgentRegistryStore::init_test_global(
            cx,
            vec![project::RegistryAgent::Npx(RegistryNpxAgent {
                metadata: RegistryAgentMetadata {
                    id: AgentId::new("zeta"),
                    name: SharedString::new_static("Zeta Code"),
                    description: SharedString::new_static("An agent from the registry"),
                    version: SharedString::new_static("1.0.0"),
                    repository: None,
                    website: None,
                    license_url: None,
                    icon_path: Some(SharedString::new_static("icons/zeta.svg")),
                },
                package: SharedString::new_static("zeta"),
                args: Vec::new(),
                env: std::collections::HashMap::default(),
            })],
        );
    });
    let (_, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    let workspace = workspaces[0].clone();
    let panel = add_agent_panel(&workspace, cx);
    configure_agents(&["stub", "zeta"], cx);
    let programs = programs_in(&[("claude", true), ("codex", true)]);
    search_agents_in(programs.path(), cx);
    use_display_only_terminals(cx);
    (workspace, panel, programs, cx)
}

/// Dispatches `action` from the workspace's center pane, as a key press there would.
fn dispatch_from_center(
    workspace: &Entity<Workspace>,
    action: impl gpui::Action,
    cx: &mut VisualTestContext,
) {
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.active_pane().focus_handle(cx).focus(window, cx);
    });
    dispatch(action, cx);
}

/// Dispatches `action` from whatever has focus.
fn dispatch(action: impl gpui::Action, cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.dispatch_action(action);
    cx.run_until_parked();
}

fn picker_of(
    workspace: &Entity<Workspace>,
    cx: &VisualTestContext,
) -> Option<Entity<NewAgentPicker>> {
    workspace.read_with(cx, |workspace, cx| {
        workspace.active_modal::<NewAgentPicker>(cx)
    })
}

fn open_picker(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Entity<NewAgentPicker> {
    dispatch_from_center(workspace, NewAgent, cx);
    picker_of(workspace, cx).expect("the picker opens")
}

/// What the picker lists, in order: each entry's name and where it starts.
fn entries(picker: &Entity<NewAgentPicker>, cx: &VisualTestContext) -> Vec<(String, &'static str)> {
    picker.read_with(cx, |picker, cx| {
        let delegate = &picker.picker.read(cx).delegate;
        delegate
            .matches
            .iter()
            .filter_map(|found| delegate.choices.get(found.candidate_id))
            .map(|choice| (choice.name.to_string(), choice.start.place()))
            .collect()
    })
}

fn type_query(picker: &Entity<NewAgentPicker>, query: &str, cx: &mut VisualTestContext) {
    let inner = picker.read_with(cx, |picker, _| picker.picker.clone());
    inner.update_in(cx, |picker, window, cx| picker.set_query(query, window, cx));
    cx.run_until_parked();
}

fn center_terminals(
    workspace: &Entity<Workspace>,
    cx: &VisualTestContext,
) -> Vec<Entity<TerminalView>> {
    workspace.read_with(cx, |workspace, cx| {
        workspace.items_of_type::<TerminalView>(cx).collect()
    })
}

#[gpui::test]
async fn the_picker_lists_zeds_agents_then_the_installed_clis(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    let picker = open_picker(&workspace, cx);
    assert_eq!(
        entries(&picker, cx),
        [
            ("Zed Agent".to_string(), "Thread"),
            ("stub".to_string(), "Thread"),
            ("Zeta Code".to_string(), "Thread"),
            ("Claude Code".to_string(), "Terminal"),
            ("Codex".to_string(), "Terminal"),
        ]
    );
    let focused = cx.update(|window, cx| picker.focus_handle(cx).contains_focused(window, cx));
    assert!(focused, "the picker takes the keyboard");
}

#[gpui::test]
async fn a_query_leaves_only_the_agents_it_matches(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    let picker = open_picker(&workspace, cx);
    type_query(&picker, "codex", cx);
    assert_eq!(entries(&picker, cx), [("Codex".to_string(), "Terminal")]);
    type_query(&picker, "", cx);
    assert_eq!(
        entries(&picker, cx).len(),
        5,
        "an empty query lists them all again"
    );
}

#[gpui::test]
async fn choosing_a_zed_agent_starts_a_thread_in_the_projects_panel(cx: &mut TestAppContext) {
    let (workspace, panel, _programs, cx) = open_project(cx).await;
    let picker = open_picker(&workspace, cx);
    type_query(&picker, "stub", cx);
    dispatch(menu::Confirm, cx);
    assert!(picker_of(&workspace, cx).is_none(), "the picker closes");
    let thread = active_thread_id(&panel, cx);
    assert!(!thread.to_key_string().is_empty());
    let focused = cx.update(|window, cx| panel.focus_handle(cx).contains_focused(window, cx));
    assert!(focused, "the new thread's panel takes focus");
    assert!(center_terminals(&workspace, cx).is_empty());
}

#[gpui::test]
async fn choosing_an_agent_cli_starts_it_in_a_new_center_terminal(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    let picker = open_picker(&workspace, cx);
    type_query(&picker, "claude", cx);
    assert_eq!(
        entries(&picker, cx),
        [("Claude Code".to_string(), "Terminal")]
    );
    dispatch(menu::Confirm, cx);
    assert!(picker_of(&workspace, cx).is_none(), "the picker closes");
    let terminals = center_terminals(&workspace, cx);
    assert_eq!(terminals.len(), 1);
    let terminal = terminals[0].read_with(cx, |view, _| view.terminal().clone());
    let written = terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
    assert_eq!(
        written,
        [b"claude\r".to_vec()],
        "the program's name and Enter, nothing else"
    );
}

#[gpui::test]
async fn the_arrow_keys_pick_another_agent(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    open_picker(&workspace, cx);
    for _ in 0..4 {
        dispatch(menu::SelectNext, cx);
    }
    dispatch(menu::Confirm, cx);
    let terminals = center_terminals(&workspace, cx);
    assert_eq!(terminals.len(), 1);
    let terminal = terminals[0].read_with(cx, |view, _| view.terminal().clone());
    let written = terminal.update(cx, |terminal, _| terminal.take_pty_write_log());
    assert_eq!(written, [b"codex\r".to_vec()], "the fifth entry, Codex");
}

#[gpui::test]
async fn a_query_that_matches_nothing_starts_nothing(cx: &mut TestAppContext) {
    let (workspace, panel, _programs, cx) = open_project(cx).await;
    let picker = open_picker(&workspace, cx);
    type_query(&picker, "nothing like this", cx);
    assert!(entries(&picker, cx).is_empty());
    dispatch(menu::Confirm, cx);
    assert!(picker_of(&workspace, cx).is_none(), "the picker closes");
    assert!(center_terminals(&workspace, cx).is_empty());
    let focused = cx.update(|window, cx| panel.focus_handle(cx).contains_focused(window, cx));
    assert!(!focused, "no thread started");
}

#[gpui::test]
async fn with_ai_disabled_new_agent_opens_nothing(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    cx.update(|_, cx| {
        SettingsStore::update_global(cx, |store, cx| {
            store.update_user_settings(cx, |settings| {
                settings.project.disable_ai = Some(SaturatingBool(true));
            });
        });
    });
    cx.run_until_parked();
    dispatch_from_center(&workspace, NewAgent, cx);
    assert!(picker_of(&workspace, cx).is_none());
}

actions!(
    marley_test,
    [
        /// A binding of the user's own.
        #[derive(Eq)]
        UsersOwn,
    ]
);

#[derive(Default)]
struct UsersOwnRan(bool);

impl Global for UsersOwnRan {}

// `secondary` is `ctrl` off macOS; these are the Linux keys.
#[cfg(target_os = "linux")]
#[gpui::test]
async fn the_marley_keymap_opens_the_picker_and_a_users_binding_wins(cx: &mut TestAppContext) {
    let (workspace, _, _programs, cx) = open_project(cx).await;
    cx.update(|_, cx| crate::load_keymap(cx));
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.active_pane().focus_handle(cx).focus(window, cx);
    });
    cx.update(|window, _| window.refresh());
    cx.simulate_keystrokes("ctrl-alt-n");
    cx.run_until_parked();
    let picker = picker_of(&workspace, cx).expect("the chord opens the picker");
    picker.update(cx, |_, cx| cx.emit(DismissEvent));
    cx.run_until_parked();
    assert!(picker_of(&workspace, cx).is_none());

    // A binding of the user's own, bound after the Marley keymap as Zed binds a user's keymap.
    cx.update(|_, cx| {
        cx.on_action(|_: &UsersOwn, cx| cx.set_global(UsersOwnRan(true)));
        cx.bind_keys([KeyBinding::new("ctrl-alt-n", UsersOwn, Some("Workspace"))]);
    });
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.active_pane().focus_handle(cx).focus(window, cx);
    });
    cx.update(|window, _| window.refresh());
    cx.simulate_keystrokes("ctrl-alt-n");
    cx.run_until_parked();
    assert!(
        picker_of(&workspace, cx).is_none(),
        "the user's binding wins"
    );
    assert!(cx.read(|cx| cx.try_global::<UsersOwnRan>().is_some_and(|ran| ran.0)));
}

#[test]
fn each_agent_cli_draws_its_own_icon() {
    assert_eq!(cli_icon(AgentKind::Claude), IconName::AiClaude);
    assert_eq!(cli_icon(AgentKind::Codex), IconName::AiOpenAi);
    assert_eq!(cli_icon(AgentKind::Gemini), IconName::AiGemini);
    assert_eq!(cli_icon(AgentKind::OpenCode), IconName::AiOpenCode);
}
