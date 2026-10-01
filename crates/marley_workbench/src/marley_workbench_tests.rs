//! Driven tests for the layout switch, and the harness the rail's tests share.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_settings::AgentSettings;
use fs::FakeFs;
use gpui::{Context, Pixels, Task, TestAppContext, VisualTestContext, px};
use project::Project;
use serde_json::json;
use settings::MarleySettingsContent;
use terminal::terminal_settings::{AlternateScroll, CursorShape, TerminalSettings};
use terminal::{Terminal, TerminalBuilder};
use terminal_view::TerminalView;
use util::path;
use util::paths::PathStyle;
use workspace::Sidebar as _;
use workspace::dock::test::TestPanel;

use super::*;

// The routing tests start the system shell, and the workbench reads and writes the data
// directory: neither may touch the user's (#475).
// SAFETY: before `main` it reads the binary's path and sets `paths`' `OnceLock`, nothing else.
#[ctor::ctor(unsafe)]
fn use_test_data_dir() {
    terminal::marley_use_test_data_dir();
}

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
    _: collections::HashMap<String, String>,
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
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> Entity<AgentPanel> {
    workspace.update_in(cx, |workspace, window, cx| {
        let panel = cx.new(|cx| AgentPanel::test_new(workspace, window, cx));
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
            ..MarleySettingsContent::default()
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
    Vec<Entity<Workspace>>,
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
    set_layout(MarleyLayout::Zed, cx);
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
async fn with_no_layout_chosen_a_window_opens_in_the_marley_layout(cx: &mut TestAppContext) {
    init_test(cx);
    cx.update(init);
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Marley));
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
    set_layout(MarleyLayout::Zed, cx);
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
    set_layout(MarleyLayout::Zed, cx);
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
    set_layout(MarleyLayout::Zed, cx);
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
    set_layout(MarleyLayout::Zed, cx);
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
    set_layout(MarleyLayout::Zed, cx);
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

    cx.update(|cx| cx.dispatch_action(&UseZedLayout));
    cx.run_until_parked();
    assert_eq!(saved_layout(&fs).await, Some(MarleyLayout::Zed));
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Zed));

    cx.update(|cx| cx.dispatch_action(&UseMarleyLayout));
    cx.run_until_parked();
    assert_eq!(saved_layout(&fs).await, Some(MarleyLayout::Marley));
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Marley));

    // Asking for the layout already in use leaves the file alone.
    fs.remove_file(paths::settings_file(), fs::RemoveOptions::default())
        .await
        .expect("the settings file");
    cx.update(|cx| cx.dispatch_action(&UseMarleyLayout));
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

/// Which of Zed's preset handlers ran, as these tests' stand-ins for them record it.
#[derive(Default)]
struct PresetsRan(Vec<&'static str>);

impl Global for PresetsRan {}

/// Stand-ins for Zed's preset handlers on `workspace`, registered as `title_bar` registers its
/// own; `title_bar::init` builds a title bar per workspace, with globals these tests do not set.
fn stand_in_for_zeds_presets(workspace: &Entity<Workspace>, cx: &mut VisualTestContext) {
    workspace.update(cx, |workspace, _| {
        workspace.register_action(|_, _: &UseClassicLayout, _, cx| {
            cx.default_global::<PresetsRan>().0.push("classic");
        });
        workspace.register_action(|_, _: &UseAgenticLayout, _, cx| {
            cx.default_global::<PresetsRan>().0.push("agentic");
        });
    });
}

fn presets_ran(cx: &VisualTestContext) -> Vec<&'static str> {
    cx.read(|cx| {
        cx.try_global::<PresetsRan>()
            .map(|ran| ran.0.clone())
            .unwrap_or_default()
    })
}

fn preset_toast_shown(workspace: &Entity<Workspace>, cx: &VisualTestContext) -> bool {
    workspace.read_with(cx, |workspace, _| {
        workspace.has_notification(&NotificationId::unique::<LayoutPresets>())
    })
}

/// Dispatches `action` from the workspace's center pane, below the workspace's root.
fn dispatch_from_center(
    workspace: &Entity<Workspace>,
    action: impl gpui::Action,
    cx: &mut VisualTestContext,
) {
    workspace.update_in(cx, |workspace, window, cx| {
        workspace.active_pane().focus_handle(cx).focus(window, cx);
    });
    cx.update(|window, _| window.refresh());
    cx.dispatch_action(action);
    cx.run_until_parked();
}

#[gpui::test]
async fn in_the_marley_layout_zeds_layout_presets_only_explain(cx: &mut TestAppContext) {
    init_test(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let workspace = workspaces[0].clone();
    stand_in_for_zeds_presets(&workspace, cx);
    dispatch_from_center(&workspace, UseClassicLayout, cx);
    assert!(preset_toast_shown(&workspace, cx));
    dispatch_from_center(&workspace, UseAgenticLayout, cx);
    assert!(preset_toast_shown(&workspace, cx));
    assert!(presets_ran(cx).is_empty(), "neither reaches Zed's handler");
    cx.read(assert_marley_defaults);
}

#[gpui::test]
async fn the_toasts_button_switches_to_zeds_layout(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    cx.update(init);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    cx.update(use_zed_layout);
    cx.run_until_parked();
    cx.read(|cx| assert_eq!(MarleySettings::get_global(cx).layout, MarleyLayout::Zed));
}

#[gpui::test]
async fn in_the_zed_layout_the_presets_reach_zed(cx: &mut TestAppContext) {
    init_test(cx);
    init_zed_sidebar(cx);
    set_layout(MarleyLayout::Zed, cx);
    cx.update(init);
    let (multi_workspace, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    register(&multi_workspace, cx);
    let workspace = workspaces[0].clone();
    stand_in_for_zeds_presets(&workspace, cx);
    dispatch_from_center(&workspace, UseClassicLayout, cx);
    dispatch_from_center(&workspace, UseAgenticLayout, cx);
    assert_eq!(presets_ran(cx), ["classic", "agentic"]);
    assert!(!preset_toast_shown(&workspace, cx));
}

// ── W6g: the docks across a layout round trip ────────────────────────────────

/// A window in the Zed layout with the Agent Panel open in the left dock and a test panel open in
/// the right, as a user leaves them, and a second test panel behind the first.
async fn open_with_docks(
    cx: &mut TestAppContext,
) -> (
    Entity<Workspace>,
    Entity<AgentPanel>,
    [Entity<TestPanel>; 2],
    &mut VisualTestContext,
) {
    init_agent_test(cx);
    init_zed_sidebar(cx);
    set_layout(MarleyLayout::Zed, cx);
    cx.update(init);
    let (multi_workspace, workspaces, cx) = open_projects(&[path!("/alpha")], cx).await;
    let workspace = workspaces[0].clone();
    let agent_panel = add_agent_panel(&workspace, cx);
    let test_panels = workspace.update_in(cx, |workspace, window, cx| {
        let panels = [5, 6].map(|priority| {
            cx.new(|cx| TestPanel::new(workspace::dock::DockPosition::Right, priority, cx))
        });
        for panel in &panels {
            workspace.add_panel(panel.clone(), window, cx);
        }
        workspace.focus_panel::<TestPanel>(window, cx);
        workspace.focus_panel::<AgentPanel>(window, cx);
        panels
    });
    register(&multi_workspace, cx);
    (workspace, agent_panel, test_panels, cx)
}

/// Each dock, left, bottom and right: whether it is open, and the panel it shows.
fn docks(workspace: &Entity<Workspace>, cx: &VisualTestContext) -> [(bool, Option<EntityId>); 3] {
    workspace.read_with(cx, |workspace, cx| {
        workspace.all_docks().map(|dock| {
            let dock = dock.read(cx);
            (
                dock.is_open(),
                dock.active_panel().map(|panel| panel.panel_id()),
            )
        })
    })
}

fn right_dock(
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
    change: impl FnOnce(&mut Dock, &mut Window, &mut Context<Dock>),
) {
    workspace.update_in(cx, |workspace, window, cx| {
        workspace
            .right_dock()
            .update(cx, |dock, cx| change(dock, window, cx));
    });
    cx.run_until_parked();
}

#[gpui::test]
async fn a_round_trip_leaves_each_dock_as_it_was(cx: &mut TestAppContext) {
    let (workspace, agent_panel, [test_panel, _], cx) = open_with_docks(cx).await;
    let before = docks(&workspace, cx);
    assert_eq!(
        before,
        [
            (true, Some(agent_panel.entity_id())),
            (false, None),
            (true, Some(test_panel.entity_id())),
        ]
    );
    set_layout(MarleyLayout::Marley, cx);
    assert_eq!(
        docks(&workspace, cx)[2],
        (true, Some(agent_panel.entity_id())),
        "the Agent Panel took over the right dock"
    );
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(docks(&workspace, cx), before);
}

#[gpui::test]
async fn a_dock_closed_before_the_trip_comes_back_closed_on_its_panel(cx: &mut TestAppContext) {
    let (workspace, _, [test_panel, _], cx) = open_with_docks(cx).await;
    right_dock(&workspace, cx, |dock, window, cx| {
        dock.set_open(false, window, cx);
    });
    set_layout(MarleyLayout::Marley, cx);
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(
        docks(&workspace, cx)[2],
        (false, Some(test_panel.entity_id()))
    );
}

#[gpui::test]
async fn a_panel_chosen_during_the_trip_stays_chosen(cx: &mut TestAppContext) {
    let (workspace, _, [_, other], cx) = open_with_docks(cx).await;
    set_layout(MarleyLayout::Marley, cx);
    // The dock lists the Agent Panel, then the two test panels.
    right_dock(&workspace, cx, |dock, window, cx| {
        dock.activate_panel(2, window, cx);
    });
    assert_eq!(docks(&workspace, cx)[2], (true, Some(other.entity_id())));
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(docks(&workspace, cx)[2], (true, Some(other.entity_id())));
}

#[gpui::test]
async fn a_dock_closed_during_the_trip_stays_closed(cx: &mut TestAppContext) {
    let (workspace, _, [test_panel, _], cx) = open_with_docks(cx).await;
    set_layout(MarleyLayout::Marley, cx);
    right_dock(&workspace, cx, |dock, window, cx| {
        dock.set_open(false, window, cx);
    });
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(
        docks(&workspace, cx)[2],
        (false, Some(test_panel.entity_id()))
    );
}

#[gpui::test]
async fn a_dock_the_agent_panel_was_shown_in_later_gets_its_panel_back(cx: &mut TestAppContext) {
    let (workspace, agent_panel, [test_panel, _], cx) = open_with_docks(cx).await;
    workspace.update_in(cx, |workspace, window, cx| {
        workspace
            .left_dock()
            .update(cx, |dock, cx| dock.set_open(false, window, cx));
    });
    cx.run_until_parked();
    // The Agent Panel arrives hidden, then the user shows it in its new dock.
    set_layout(MarleyLayout::Marley, cx);
    right_dock(&workspace, cx, |dock, window, cx| {
        dock.activate_panel(0, window, cx);
    });
    assert_eq!(
        docks(&workspace, cx)[2],
        (true, Some(agent_panel.entity_id()))
    );
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(
        docks(&workspace, cx)[2],
        (true, Some(test_panel.entity_id()))
    );
}

#[gpui::test]
async fn a_hidden_agent_panel_moves_without_moving_anything_else(cx: &mut TestAppContext) {
    let (workspace, _, [test_panel, _], cx) = open_with_docks(cx).await;
    workspace.update_in(cx, |workspace, window, cx| {
        workspace
            .left_dock()
            .update(cx, |dock, cx| dock.set_open(false, window, cx));
    });
    cx.run_until_parked();
    let right = (true, Some(test_panel.entity_id()));
    set_layout(MarleyLayout::Marley, cx);
    assert_eq!(docks(&workspace, cx)[2], right);
    set_layout(MarleyLayout::Zed, cx);
    assert_eq!(docks(&workspace, cx)[2], right);
}

// PTY helpers shared by the agent bar's tests and the notifications' (#477, #478).

/// A scratch folder holding `claude`, a link to `sleep`, so a script can become an agent by
/// name, as `claude` is started from the PATH.
pub(crate) fn fake_claude_bin() -> tempfile::TempDir {
    let path = std::env::var_os("PATH").expect("a PATH");
    let sleep = std::env::split_paths(&path)
        .map(|dir| dir.join("sleep"))
        .find(|candidate| candidate.is_file())
        .expect("sleep on the PATH");
    let bin = tempfile::tempdir().expect("a scratch directory");
    std::os::unix::fs::symlink(sleep, bin.path().join("claude")).expect("the link");
    bin
}

/// A real scratch folder, by its canonical path, which a process reports as its directory.
pub(crate) fn scratch_folder() -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::tempdir().expect("a scratch folder");
    let path = folder.path().canonicalize().expect("its canonical path");
    (folder, path)
}

/// A window whose project is `folder`, which is also a fake repository on branch `main`.
pub(crate) async fn workspace_over<'a>(
    folder: &Path,
    cx: &'a mut TestAppContext,
) -> (Entity<Workspace>, &'a mut VisualTestContext) {
    let fs = FakeFs::new(cx.executor());
    fs.insert_tree(folder, json!({ ".git": {}, "src": {} }))
        .await;
    fs.set_branch_name(&folder.join(".git"), Some("main"));
    let project = Project::test(Arc::<FakeFs>::clone(&fs), [folder], cx).await;
    cx.update(|cx| <dyn Fs>::set_global(fs, cx));
    let (multi_workspace, cx) =
        cx.add_window_view(|window, cx| MultiWorkspace::test_new(project, window, cx));
    let workspace =
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone());
    cx.run_until_parked();
    (workspace, cx)
}

/// A terminal view in `workspace`'s center over a real PTY that runs `script` in `folder`, with
/// `bin` first on the PATH.
pub(crate) async fn terminal_running(
    script: &str,
    folder: &Path,
    bin: &Path,
    workspace: &Entity<Workspace>,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Entity<TerminalView>) {
    let program = "/bin/sh".to_string();
    let args = vec!["-c".to_string(), script.to_string()];
    let inherited = std::env::var_os("PATH").expect("a PATH");
    let path = std::env::join_paths(
        std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&inherited)),
    )
    .expect("a PATH");
    let mut env = collections::HashMap::default();
    env.insert(
        "PATH".to_string(),
        path.into_string().expect("a UTF-8 PATH"),
    );
    let builder = cx
        .update(|_, cx| {
            TerminalBuilder::new(
                Some(folder.to_path_buf()),
                terminal::TerminalMode::task(task::SpawnInTerminal {
                    command: Some(program.clone()),
                    args: args.clone(),
                    ..Default::default()
                }),
                task::Shell::WithArguments {
                    program,
                    args,
                    title_override: None,
                },
                env,
                CursorShape::default(),
                AlternateScroll::On,
                None,
                vec![],
                Duration::ZERO,
                false,
                0,
                cx,
                vec![],
                PathStyle::local(),
            )
        })
        .await
        .expect("the terminal starts");
    let terminal = cx.update(|_, cx| cx.new(|cx| builder.subscribe(cx)));
    let view = workspace.update_in(cx, |workspace, window, cx| {
        let view = cx.new(|cx| {
            TerminalView::new(
                terminal.clone(),
                workspace.weak_handle(),
                None,
                workspace.project().downgrade(),
                window,
                cx,
            )
        });
        workspace.add_item_to_active_pane(Box::new(view.clone()), None, true, window, cx);
        view
    });
    (terminal, view)
}

pub(crate) fn redraw(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}
