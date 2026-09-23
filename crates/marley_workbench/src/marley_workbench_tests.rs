//! Driven tests for the layout switch, and the harness the rail's tests share.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent_settings::AgentSettings;
use fs::FakeFs;
use gpui::{Context, Pixels, Task, TestAppContext, VisualTestContext, px};
use project::Project;
use serde_json::json;
use settings::MarleySettingsContent;
use terminal::terminal_settings::{AlternateScroll, CursorShape, TerminalSettings};
use terminal::{Terminal, TerminalBuilder};
use util::path;
use util::paths::PathStyle;
use workspace::Sidebar as _;

use super::*;

/// Where the tests' terminal factory, [`display_only_terminal_in`], was asked to start each
/// terminal.
#[derive(Default)]
pub(crate) struct RequestedDirectories(pub(crate) Vec<Option<PathBuf>>);

impl Global for RequestedDirectories {}

/// The tests' terminal factory: it notes the directory it was given and makes a display-only
/// terminal, with no shell behind it, so no test starts a real program.
pub(crate) fn display_only_terminal_in(
    _: &mut Project,
    directory: Option<PathBuf>,
    cx: &mut Context<Project>,
) -> Task<anyhow::Result<Entity<Terminal>>> {
    cx.default_global::<RequestedDirectories>()
        .0
        .push(directory);
    Task::ready(Ok(display_only_terminal(cx)))
}

pub(crate) fn display_only_terminal(cx: &mut App) -> Entity<Terminal> {
    cx.new(|cx| {
        TerminalBuilder::new_display_only(
            CursorShape::default(),
            AlternateScroll::On,
            None,
            0,
            cx.background_executor(),
            PathStyle::local(),
        )
        .subscribe(cx)
    })
}

/// Makes every terminal the rail or the New Agent picker starts a display-only one.
pub(crate) fn use_display_only_terminals(cx: &mut VisualTestContext) {
    cx.update(|_, cx| {
        let launcher = agents::Launcher {
            terminal_factory: display_only_terminal_in,
            ..agents::launcher(cx)
        };
        cx.set_global(launcher);
    });
}

/// Points the search for agent CLIs at `directory`.
pub(crate) fn search_agents_in(directory: &Path, cx: &mut VisualTestContext) {
    cx.update(|_, cx| {
        let launcher = agents::Launcher {
            search_path: Some(directory.as_os_str().to_owned()),
            ..agents::launcher(cx)
        };
        cx.set_global(launcher);
    });
}

/// A directory holding a program for each name, executable or not.
#[cfg(unix)]
pub(crate) fn programs_in(programs: &[(&str, bool)]) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().expect("a temporary directory");
    for (name, executable) in programs {
        let path = dir.path().join(name);
        std::fs::write(&path, "#!/bin/sh\n").expect("the program is written");
        let mode = if *executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
            .expect("the mode is set");
    }
    dir
}

/// Zed's sidebar state for a sidebar the user dragged to 321 px.
pub(crate) const ZED_SIDEBAR_STATE: &str =
    r#"{"width":321.0,"width_set_by_user":true,"active_view":"ThreadList"}"#;

/// The settings, database, theme and editor every window here needs.
pub(crate) fn init_test(cx: &TestAppContext) {
    cx.update(|cx| {
        let settings_store = SettingsStore::test(cx);
        cx.set_global(settings_store);
        cx.set_global(db::AppDatabase::test_new());
        theme_settings::init(theme::LoadThemes::JustBase, cx);
        editor::init(cx);
        terminal_view::init(cx);
    });
}

/// The globals the thread tests need: Zed's agent test setup, which installs its own settings
/// store and database, so it replaces `init_test` rather than following it, plus the stores the
/// Agent Panel reads and a thread database of the test's own.
pub(crate) fn init_agent_test(cx: &mut TestAppContext) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_DATABASE: AtomicUsize = AtomicUsize::new(0);
    let database = NEXT_DATABASE.fetch_add(1, Ordering::SeqCst);
    cx.update(|cx| {
        cx.set_global(agent_ui::thread_metadata_store::TestMetadataDbName(
            format!("MARLEY_RAIL_THREADS_{database}"),
        ));
        cx.set_global(
            agent_ui::terminal_thread_metadata_store::TestTerminalMetadataDbName(format!(
                "MARLEY_RAIL_TERMINAL_THREADS_{database}"
            )),
        );
    });
    agent_ui::test_support::init_test(cx);
    cx.update(|cx| {
        agent::ThreadStore::init_global(cx);
        agent_ui::thread_metadata_store::ThreadMetadataStore::init_global(cx);
        language_model::LanguageModelRegistry::test(cx);
        prompt_store::init(cx);
        terminal_view::init(cx);
    });
}

/// An Agent Panel in `workspace`, as `crates/zed` adds one.
pub(crate) fn add_agent_panel(
    workspace: &Entity<workspace::Workspace>,
    cx: &mut VisualTestContext,
) -> Entity<agent_ui::AgentPanel> {
    workspace.update_in(cx, |workspace, window, cx| {
        let panel = cx.new(|cx| agent_ui::AgentPanel::test_new(workspace, window, cx));
        workspace.add_panel(panel.clone(), window, cx);
        panel
    })
}

/// Replaces the user settings with the Marley layout and one custom agent per name.
pub(crate) fn configure_agents(names: &[&str], cx: &mut VisualTestContext) {
    let servers: serde_json::Map<String, serde_json::Value> = names
        .iter()
        .map(|name| {
            (
                (*name).to_string(),
                json!({ "type": "custom", "command": name }),
            )
        })
        .collect();
    let settings = json!({ "marley": { "layout": "marley" }, "agent_servers": servers });
    cx.update(|_, cx| {
        SettingsStore::update_global(cx, |store, cx| {
            let parsed = store.set_user_settings(&settings.to_string(), cx);
            assert!(
                !matches!(parsed.parse_status, settings::ParseStatus::Failed { .. }),
                "{:?}",
                parsed.parse_status
            );
        });
    });
    cx.run_until_parked();
}

/// What `sidebar::Sidebar::new` reads, for the tests that build Zed's sidebar.
pub(crate) fn init_zed_sidebar(cx: &TestAppContext) {
    cx.update(|cx| {
        agent::ThreadStore::init_global(cx);
        agent_ui::thread_metadata_store::ThreadMetadataStore::init_global(cx);
        agent_ui::terminal_thread_metadata_store::TerminalThreadMetadataStore::init_global(cx);
        language_model::LanguageModelRegistry::test(cx);
        prompt_store::init(cx);
    });
}

fn update_user_settings(cx: &TestAppContext, update: impl FnOnce(&mut SettingsContent)) {
    cx.update(|cx| {
        SettingsStore::update_global(cx, |store, cx| store.update_user_settings(cx, update));
    });
    cx.run_until_parked();
}

pub(crate) fn set_layout(layout: MarleyLayout, cx: &TestAppContext) {
    update_user_settings(cx, |content| {
        content.marley = Some(MarleySettingsContent {
            layout: Some(layout),
        });
    });
}

/// A window over one project per root, opened in order, so the last is displayed. Zed puts each
/// new project group on top, so the rail lists them in reverse.
pub(crate) async fn open_projects<'a>(
    roots: &[&str],
    cx: &'a mut TestAppContext,
) -> (
    Entity<MultiWorkspace>,
    Vec<Entity<workspace::Workspace>>,
    &'a mut VisualTestContext,
) {
    let fs = FakeFs::new(cx.executor());
    let mut projects = Vec::new();
    for root in roots {
        fs.insert_tree(root, json!({ "src": {} })).await;
        projects.push(Project::test(Arc::<FakeFs>::clone(&fs), [Path::new(root)], cx).await);
    }
    cx.update(|cx| <dyn Fs>::set_global(fs, cx));
    let mut projects = projects.into_iter();
    let first = projects.next().expect("at least one root");
    let (multi_workspace, cx) =
        cx.add_window_view(|window, cx| MultiWorkspace::test_new(first, window, cx));
    let mut workspaces = vec![
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone()),
    ];
    for project in projects {
        workspaces.push(
            multi_workspace.update_in(cx, |multi_workspace, window, cx| {
                multi_workspace.test_add_workspace(project, window, cx)
            }),
        );
    }
    cx.run_until_parked();
    (multi_workspace, workspaces, cx)
}

/// Asserts that the rail's blob `saved` holds every field of Zed's blob `zed`, unchanged, and
/// says the rail is open.
pub(crate) fn assert_keeps_zeds_fields(saved: &str, zed: &str) {
    let saved: serde_json::Value = serde_json::from_str(saved).expect("the rail's blob is JSON");
    let zed: serde_json::Value = serde_json::from_str(zed).expect("Zed's blob is JSON");
    for (key, value) in zed.as_object().expect("Zed's blob is an object") {
        assert_eq!(&saved[key], value, "Zed's `{key}`");
    }
    assert_eq!(saved["marley_rail_closed"], false);
}

/// Registers the layout's sidebar the way `crates/zed` does when a window opens.
pub(crate) fn register(multi_workspace: &Entity<MultiWorkspace>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| register_sidebar(multi_workspace, window, cx));
    cx.run_until_parked();
}

pub(crate) fn rail_of(multi_workspace: &Entity<MultiWorkspace>, cx: &App) -> Option<Entity<Rail>> {
    multi_workspace.read(cx).sidebar()?.to_any().downcast().ok()
}

fn zed_sidebar_of(
    multi_workspace: &Entity<MultiWorkspace>,
    cx: &App,
) -> Option<Entity<sidebar::Sidebar>> {
    multi_workspace.read(cx).sidebar()?.to_any().downcast().ok()
}

fn assert_zed_defaults(cx: &App) {
    assert!(TerminalSettings::get_global(cx).button);
    assert_eq!(AgentSettings::get_global(cx).dock, DockPosition::Left);
}

fn assert_marley_defaults(cx: &App) {
    assert!(!TerminalSettings::get_global(cx).button);
    assert_eq!(AgentSettings::get_global(cx).dock, DockPosition::Right);
}

async fn saved_layout(fs: &FakeFs) -> Option<MarleyLayout> {
    let text = fs
        .load(paths::settings_file())
        .await
        .expect("the settings file");
    let content: SettingsContent =
        settings::parse_json_with_comments(&text).expect("valid settings");
    content.marley.and_then(|marley| marley.layout)
}

#[gpui::test]
async fn a_window_in_the_zed_layout_gets_zeds_sidebar_and_zeds_defaults(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    cx.read(|cx| {
        assert!(zed_sidebar_of(&multi_workspace, cx).is_some());
        assert!(!multi_workspace.read(cx).sidebar_open());
        assert_zed_defaults(cx);
    });
}

#[gpui::test]
async fn a_window_opened_in_the_marley_layout_starts_with_its_rail_open(cx: &mut TestAppContext) {
    init_test(cx);
    set_layout(MarleyLayout::Marley, cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    cx.read(|cx| {
        assert!(rail_of(&multi_workspace, cx).is_some());
        assert!(multi_workspace.read(cx).sidebar_open());
        assert_marley_defaults(cx);
    });
}

#[gpui::test]
async fn switching_to_the_marley_layout_gives_every_window_an_open_rail(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (first, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&first, cx);
    let (second, _, cx) = open_projects(&[path!("/beta")], cx).await;
    register(&second, cx);
    set_layout(MarleyLayout::Marley, cx);
    cx.read(|cx| {
        for multi_workspace in [&first, &second] {
            assert!(rail_of(multi_workspace, cx).is_some());
            assert!(multi_workspace.read(cx).sidebar_open());
        }
        assert_marley_defaults(cx);
    });
}

#[gpui::test]
async fn switching_back_hands_each_window_its_own_zed_sidebar(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let sidebar = cx
        .read(|cx| zed_sidebar_of(&multi_workspace, cx))
        .expect("Zed's sidebar");
    multi_workspace.update(cx, MultiWorkspace::open_sidebar);
    sidebar.update(cx, |sidebar, cx| sidebar.set_width(Some(px(321.)), cx));
    let state = sidebar.read_with(cx, workspace::Sidebar::serialized_state);

    set_layout(MarleyLayout::Marley, cx);
    let rail = cx
        .read(|cx| rail_of(&multi_workspace, cx))
        .expect("the rail");
    // While the rail stands in, the window keeps saving Zed's sidebar's state, with the rail's
    // own field added.
    let saved = rail
        .read_with(cx, workspace::Sidebar::serialized_state)
        .expect("the rail's blob");
    assert_keeps_zeds_fields(&saved, &state.expect("Zed's blob"));

    set_layout(MarleyLayout::Zed, cx);
    cx.read(|cx| {
        let back = zed_sidebar_of(&multi_workspace, cx).expect("Zed's sidebar again");
        assert_eq!(back.entity_id(), sidebar.entity_id());
        assert_eq!(back.read(cx).width(cx), px(321.));
        assert!(multi_workspace.read(cx).sidebar_open());
        assert_zed_defaults(cx);
    });
}

#[gpui::test]
async fn a_closed_zed_sidebar_is_closed_again_after_a_round_trip(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    set_layout(MarleyLayout::Marley, cx);
    assert!(multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.sidebar_open()));
    set_layout(MarleyLayout::Zed, cx);
    cx.read(|cx| {
        assert!(zed_sidebar_of(&multi_workspace, cx).is_some());
        assert!(!multi_workspace.read(cx).sidebar_open());
    });
}

#[gpui::test]
async fn a_window_opened_in_the_marley_layout_gives_zeds_sidebar_its_saved_state(
    cx: &mut TestAppContext,
) {
    init_test(cx);
    init_zed_sidebar(cx);
    set_layout(MarleyLayout::Marley, cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let rail = cx
        .read(|cx| rail_of(&multi_workspace, cx))
        .expect("the rail");
    rail.update_in(cx, |rail, window, cx| {
        rail.restore_serialized_state(ZED_SIDEBAR_STATE, window, cx);
    });
    set_layout(MarleyLayout::Zed, cx);
    cx.read(|cx| {
        let sidebar = zed_sidebar_of(&multi_workspace, cx).expect("Zed's sidebar");
        assert_eq!(sidebar.read(cx).width(cx), px(321.));
    });
}

/// The blob the window's next save writes for its sidebar.
fn sidebar_blob(multi_workspace: &Entity<MultiWorkspace>, cx: &VisualTestContext) -> String {
    cx.read(|cx| {
        multi_workspace
            .read(cx)
            .sidebar()
            .and_then(|sidebar| sidebar.serialized_state(cx))
            .expect("the sidebar saves a blob")
    })
}

fn saved_closed(blob: &str) -> bool {
    let saved: serde_json::Value = serde_json::from_str(blob).expect("JSON");
    saved["marley_rail_closed"] == true
}

/// Restores the test's window from saved state the way Zed restores a window at startup.
async fn restore_window(
    sidebar_open: bool,
    sidebar_state: Option<String>,
    cx: &mut VisualTestContext,
) {
    let window = cx
        .update(|window, _| window.window_handle())
        .downcast::<MultiWorkspace>()
        .expect("a multi-workspace window");
    let state = workspace::MultiWorkspaceState {
        active_workspace_id: None,
        sidebar_open,
        project_groups: Vec::new(),
        sidebar_state,
    };
    let fs = cx.update(|_, cx| <dyn Fs>::global(cx));
    let mut async_cx = cx.to_async();
    workspace::apply_restored_multiworkspace_state(window, &state, fs, &mut async_cx).await;
    cx.run_until_parked();
}

fn rail_width(multi_workspace: &Entity<MultiWorkspace>, cx: &VisualTestContext) -> Pixels {
    cx.read(|cx| {
        rail_of(multi_workspace, cx)
            .expect("the rail")
            .read(cx)
            .width(cx)
    })
}

fn set_rail_width(
    multi_workspace: &Entity<MultiWorkspace>,
    width: Pixels,
    cx: &mut VisualTestContext,
) {
    let rail = cx
        .read(|cx| rail_of(multi_workspace, cx))
        .expect("the rail");
    rail.update(cx, |rail, cx| rail.set_width(Some(width), cx));
}

#[gpui::test]
async fn a_closed_rail_stays_closed_when_its_window_is_restored(cx: &mut TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let saved = {
        let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
        register(&multi_workspace, cx);
        multi_workspace.update_in(cx, |multi_workspace, window, cx| {
            multi_workspace.close_sidebar(window, cx);
        });
        cx.run_until_parked();
        sidebar_blob(&multi_workspace, cx)
    };
    assert!(saved_closed(&saved));

    // The next launch builds the window's rail open, then restores what was saved.
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    cx.read(|cx| assert!(multi_workspace.read(cx).sidebar_open()));
    restore_window(false, Some(saved), cx).await;
    cx.read(|cx| assert!(!multi_workspace.read(cx).sidebar_open()));
}

#[gpui::test]
async fn an_open_rail_stays_open_when_its_window_is_restored(cx: &mut TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let saved = {
        let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
        register(&multi_workspace, cx);
        sidebar_blob(&multi_workspace, cx)
    };
    assert!(!saved_closed(&saved));
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    restore_window(true, Some(saved), cx).await;
    cx.read(|cx| assert!(multi_workspace.read(cx).sidebar_open()));
}

#[gpui::test]
async fn with_ai_off_no_close_is_saved(cx: &mut TestAppContext) {
    init_test(cx);
    update_user_settings(cx, |content| {
        content.agent.get_or_insert_default().enabled = Some(false);
    });
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    multi_workspace.update_in(cx, |multi_workspace, window, cx| {
        multi_workspace.close_sidebar(window, cx);
    });
    cx.run_until_parked();
    assert!(!saved_closed(&sidebar_blob(&multi_workspace, cx)));
}

#[gpui::test]
async fn the_rails_width_holds_through_a_switch_to_zed_and_back(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    set_rail_width(&multi_workspace, px(400.), cx);

    // The window opened in the Marley layout, so Zed's sidebar is built fresh from the blob.
    set_layout(MarleyLayout::Zed, cx);
    cx.read(|cx| {
        let sidebar = zed_sidebar_of(&multi_workspace, cx).expect("Zed's sidebar");
        assert_eq!(sidebar.read(cx).width(cx), px(400.));
    });
    set_layout(MarleyLayout::Marley, cx);
    assert_eq!(rail_width(&multi_workspace, cx), px(400.));
}

#[gpui::test]
async fn a_width_set_on_the_rail_carries_to_the_zed_sidebar_it_keeps(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let sidebar = cx
        .read(|cx| zed_sidebar_of(&multi_workspace, cx))
        .expect("Zed's sidebar");
    set_layout(MarleyLayout::Marley, cx);
    set_rail_width(&multi_workspace, px(400.), cx);
    cx.read(|cx| assert_eq!(sidebar.read(cx).width(cx), px(400.)));
    set_layout(MarleyLayout::Zed, cx);
    cx.read(|cx| {
        let back = zed_sidebar_of(&multi_workspace, cx).expect("Zed's sidebar again");
        assert_eq!(back.entity_id(), sidebar.entity_id());
        assert_eq!(back.read(cx).width(cx), px(400.));
    });
}

#[gpui::test]
async fn a_restored_width_sizes_the_rail(cx: &mut TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let saved = r#"{"width":333.0,"width_set_by_user":true}"#.to_string();
    restore_window(true, Some(saved), cx).await;
    assert_eq!(rail_width(&multi_workspace, cx), px(333.));
}

#[gpui::test]
async fn a_rail_built_over_zeds_sidebar_starts_at_its_width(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let sidebar = cx
        .read(|cx| zed_sidebar_of(&multi_workspace, cx))
        .expect("Zed's sidebar");
    sidebar.update_in(cx, |sidebar, window, cx| {
        sidebar.restore_serialized_state(ZED_SIDEBAR_STATE, window, cx);
    });
    set_layout(MarleyLayout::Marley, cx);
    assert_eq!(rail_width(&multi_workspace, cx), px(321.));
}

#[gpui::test]
async fn a_swap_keeps_focus_in_the_sidebar(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let sidebar = cx
        .read(|cx| zed_sidebar_of(&multi_workspace, cx))
        .expect("Zed's sidebar");
    multi_workspace.update(cx, MultiWorkspace::open_sidebar);
    cx.run_until_parked();
    cx.update(|window, cx| sidebar.focus_handle(cx).focus(window, cx));
    set_layout(MarleyLayout::Marley, cx);
    let rail = cx
        .read(|cx| rail_of(&multi_workspace, cx))
        .expect("the rail");
    cx.update(|window, cx| assert!(rail.focus_handle(cx).contains_focused(window, cx)));
}

#[gpui::test]
async fn with_ai_off_the_rail_is_registered_but_left_closed(cx: &mut TestAppContext) {
    init_test(cx);
    update_user_settings(cx, |content| {
        content.agent.get_or_insert_default().enabled = Some(false);
    });
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    cx.read(|cx| {
        assert!(rail_of(&multi_workspace, cx).is_some());
        assert!(!multi_workspace.read(cx).sidebar_open());
    });
}

#[gpui::test]
fn the_marley_layout_moves_two_defaults_and_user_values_still_win(cx: &TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    cx.read(assert_marley_defaults);
    update_user_settings(cx, |content| {
        content.terminal.get_or_insert_default().button = Some(true);
        content.agent.get_or_insert_default().dock = Some(DockPosition::Left);
    });
    cx.read(assert_zed_defaults);
}

#[gpui::test]
fn a_second_init_keeps_zeds_own_defaults(cx: &TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    cx.update(init);
    set_layout(MarleyLayout::Zed, cx);
    cx.read(assert_zed_defaults);
}

#[gpui::test]
async fn the_layout_actions_write_the_choice_to_the_settings_file(cx: &TestAppContext) {
    init_test(cx);
    let fs = FakeFs::new(cx.executor());
    cx.update(|cx| <dyn Fs>::set_global(Arc::<FakeFs>::clone(&fs), cx));
    cx.update(init);

    cx.update(|cx| cx.dispatch_action(&UseMarleyLayout));
    cx.run_until_parked();
    assert_eq!(saved_layout(&fs).await, Some(MarleyLayout::Marley));
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Marley));

    cx.update(|cx| cx.dispatch_action(&UseZedLayout));
    cx.run_until_parked();
    assert_eq!(saved_layout(&fs).await, Some(MarleyLayout::Zed));
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Zed));

    // Asking for the layout already in use leaves the file alone.
    fs.remove_file(paths::settings_file(), fs::RemoveOptions::default())
        .await
        .expect("the settings file");
    cx.update(|cx| cx.dispatch_action(&UseZedLayout));
    cx.run_until_parked();
    assert!(!fs.is_file(paths::settings_file()).await);
}

/// The source of each binding for `marley::NewAgent`.
fn new_agent_bindings(cx: &App) -> Vec<Option<gpui::KeyBindingMetaIndex>> {
    let keymap = cx.key_bindings();
    let keymap = keymap.borrow();
    keymap
        .bindings()
        .filter(|binding| binding.action().name() == "marley::NewAgent")
        .map(gpui::KeyBinding::meta)
        .collect()
}

#[gpui::test]
fn the_marley_keymap_binds_new_agent_as_a_default(cx: &TestAppContext) {
    init_test(cx);
    cx.update(|cx| {
        load_keymap_from(KEYMAP, cx).expect("the Marley keymap loads");
        assert_eq!(
            new_agent_bindings(cx),
            [Some(KeybindSource::Default.meta())],
            "bound once, as a default source, below the user's keymap"
        );
    });
}

#[gpui::test]
fn a_marley_keymap_that_fails_to_load_binds_nothing(cx: &TestAppContext) {
    init_test(cx);
    cx.update(|cx| {
        assert!(load_keymap_from("not a keymap", cx).is_err());
        let unknown = r#"[{ "context": "Workspace", "bindings": {
            "ctrl-alt-m": "marley::NoSuchAction",
            "ctrl-alt-n": "marley::NewAgent"
        } }]"#;
        assert!(load_keymap_from(unknown, cx).is_err());
        assert!(
            new_agent_bindings(cx).is_empty(),
            "not even the binding that did load"
        );
    });
}
