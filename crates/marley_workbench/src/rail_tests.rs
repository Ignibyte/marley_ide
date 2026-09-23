//! Driven tests for the rail: its rows, the clicks and menus on them, the bell, and the answers
//! the `MultiWorkspace` reads from it.

use std::path::Path;

use fs::FakeFs;
use gpui::{Global, Modifiers, TestAppContext, VisualTestContext};
use marley_rail::Selection;
use serde_json::json;
use settings::MarleyLayout;
use terminal::{
    TerminalBuilder,
    terminal_settings::{AlternateScroll, CursorShape},
};
use util::{path, path_list::PathList, paths::PathStyle};
use workspace::SaveIntent;

use super::*;
use crate::marley_workbench_tests::{
    ZED_SIDEBAR_STATE, init_test, open_projects, rail_of, register, set_layout,
};

/// Where the rail asked [`display_only_terminal_in`] to start each terminal.
#[derive(Default)]
struct RequestedDirectories(Vec<Option<PathBuf>>);

impl Global for RequestedDirectories {}

/// The rail's terminal factory in these tests: it notes the directory it was given and makes a
/// display-only terminal, with no shell behind it.
fn display_only_terminal_in(
    _: &mut Project,
    directory: Option<PathBuf>,
    cx: &mut Context<Project>,
) -> Task<anyhow::Result<Entity<Terminal>>> {
    cx.default_global::<RequestedDirectories>()
        .0
        .push(directory);
    Task::ready(Ok(display_only_terminal(cx)))
}

fn display_only_terminal(cx: &mut App) -> Entity<Terminal> {
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

/// A window over `alpha` and `beta` in the Marley layout, with `beta` displayed and the rail
/// open, making display-only terminals. The rail lists `beta` first.
async fn open_rail(
    cx: &mut TestAppContext,
) -> (
    Entity<MultiWorkspace>,
    Entity<Workspace>,
    Entity<Workspace>,
    Entity<Rail>,
    &mut VisualTestContext,
) {
    init_test(cx);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, workspaces, cx) =
        open_projects(&[path!("/alpha"), path!("/beta")], cx).await;
    register(&multi_workspace, cx);
    let rail = cx
        .read(|cx| rail_of(&multi_workspace, cx))
        .expect("the Marley layout registers the rail");
    rail.update(cx, |rail, _| {
        rail.terminal_factory = display_only_terminal_in;
    });
    let alpha = workspaces[0].clone();
    let beta = workspaces[1].clone();
    (multi_workspace, alpha, beta, rail, cx)
}

/// A display-only terminal added to `workspace`'s center the way the rest of Zed adds one.
fn add_terminal(
    workspace: &Entity<Workspace>,
    focus: bool,
    cx: &mut VisualTestContext,
) -> (Entity<Terminal>, Entity<TerminalView>) {
    let (pane, handle, project) = workspace.read_with(cx, |workspace, _| {
        (
            workspace.active_pane().clone(),
            workspace.weak_handle(),
            workspace.project().downgrade(),
        )
    });
    let (terminal, view) = cx.update(|window, cx| {
        let terminal = display_only_terminal(cx);
        let view =
            cx.new(|cx| TerminalView::new(terminal.clone(), handle, None, project, window, cx));
        (terminal, view)
    });
    pane.update_in(cx, |pane, window, cx| {
        pane.add_item(Box::new(view.clone()), true, focus, None, window, cx);
    });
    cx.run_until_parked();
    (terminal, view)
}

/// Each project header's name, with the ids of the terminal rows listed under it.
fn listing(rail: &Entity<Rail>, cx: &VisualTestContext) -> Vec<(String, Vec<u64>)> {
    let mut listing: Vec<(String, Vec<u64>)> = Vec::new();
    for row in rail.read_with(cx, |rail, _| marley_rail::rail_rows(&rail.snapshot.rail)) {
        match row {
            Row::Project(project) => listing.push((project.name, Vec::new())),
            Row::Terminal(terminal) => listing[terminal.project].1.push(terminal.id),
            Row::Thread(_) => {}
        }
    }
    listing
}

fn names(rail: &Entity<Rail>, cx: &VisualTestContext) -> Vec<String> {
    listing(rail, cx)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

fn selected(rail: &Entity<Rail>, cx: &VisualTestContext) -> Selection {
    rail.read_with(cx, |rail, _| marley_rail::selection(&rail.snapshot.rail))
}

fn has_notifications(rail: &Entity<Rail>, cx: &VisualTestContext) -> bool {
    rail.read_with(cx, Sidebar::has_notifications)
}

fn id(view: &Entity<TerminalView>) -> u64 {
    view.entity_id().as_u64()
}

fn terminal_selector(view: &Entity<TerminalView>) -> &'static str {
    format!("marley-rail-terminal-{}", id(view)).leak()
}

/// Draws the window afresh and clicks the middle of what is drawn under `selector`.
fn click(selector: &'static str, cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    let bounds = cx.debug_bounds(selector).expect("the element is drawn");
    cx.simulate_click(bounds.center(), Modifiers::none());
    cx.run_until_parked();
}

#[gpui::test]
async fn the_rail_lists_each_project_with_its_center_terminals(cx: &mut TestAppContext) {
    let (_, alpha, beta, rail, cx) = open_rail(cx).await;
    let (_, alpha_terminal) = add_terminal(&alpha, false, cx);
    let (_, beta_terminal) = add_terminal(&beta, true, cx);
    assert_eq!(
        listing(&rail, cx),
        [
            ("beta".to_string(), vec![id(&beta_terminal)]),
            ("alpha".to_string(), vec![id(&alpha_terminal)]),
        ]
    );
    cx.update(|window, _| window.refresh());
    for selector in [
        "marley-rail-project-0",
        "marley-rail-project-1",
        terminal_selector(&alpha_terminal),
        terminal_selector(&beta_terminal),
    ] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector}");
    }
}

#[gpui::test]
async fn the_rail_debugs_as_its_width_and_rows(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    let debug = rail.read_with(cx, |rail, _| format!("{rail:?}"));
    let rows = rail.read_with(cx, |rail, _| format!("{:?}", rail.snapshot.rail));
    let width = format!("{DEFAULT_WIDTH:?}");
    assert_eq!(
        debug,
        format!("Rail {{ width: {width}, rows: {rows}, .. }}")
    );
}

#[gpui::test]
async fn projects_with_the_same_name_are_told_apart_by_their_parents(cx: &mut TestAppContext) {
    init_test(cx);
    set_layout(MarleyLayout::Marley, cx);
    let (multi_workspace, _, cx) = open_projects(&[path!("/one/app"), path!("/two/app")], cx).await;
    register(&multi_workspace, cx);
    let rail = cx
        .read(|cx| rail_of(&multi_workspace, cx))
        .expect("the rail");
    assert_eq!(names(&rail, cx), ["two/app", "one/app"]);
}

#[gpui::test]
async fn a_group_with_no_open_workspace_is_not_listed(cx: &mut TestAppContext) {
    let (multi_workspace, _, _, rail, cx) = open_rail(cx).await;
    multi_workspace.update(cx, |multi_workspace, cx| {
        multi_workspace.test_add_project_group(ProjectGroup {
            key: ProjectGroupKey::new(None, PathList::new(&[path!("/gamma")])),
            workspaces: Vec::new(),
            expanded: true,
        });
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(names(&rail, cx), ["beta", "alpha"]);
}

#[gpui::test]
async fn a_project_added_without_being_shown_is_listed_and_opens(cx: &mut TestAppContext) {
    let (multi_workspace, _, _, rail, cx) = open_rail(cx).await;
    let fs = FakeFs::new(cx.executor());
    fs.insert_tree(path!("/gamma"), json!({ "src": {} })).await;
    let project = Project::test(fs, [Path::new(path!("/gamma"))], cx).await;
    // `add` keeps a workspace in the window without showing it, so it has never been active.
    let gamma = multi_workspace.update_in(cx, |multi_workspace, window, cx| {
        let gamma = cx.new(|cx| Workspace::test_new(project, window, cx));
        multi_workspace.add(gamma.clone(), window, cx);
        gamma
    });
    cx.run_until_parked();
    assert_eq!(names(&rail, cx), ["gamma", "beta", "alpha"]);

    click("marley-rail-project-0", cx);
    assert_eq!(
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone()),
        gamma
    );
}

#[gpui::test]
async fn the_chevron_folds_a_projects_terminals_away_and_back(cx: &mut TestAppContext) {
    let (multi_workspace, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, true, cx);
    let key = beta.read_with(cx, Workspace::project_group_key);
    let expanded = |cx: &VisualTestContext| {
        multi_workspace.read_with(cx, |multi_workspace, _| {
            multi_workspace
                .group_state_by_key(&key)
                .map(|group| group.expanded)
        })
    };

    click("marley-rail-disclosure-0", cx);
    assert_eq!(listing(&rail, cx)[0], ("beta".to_string(), Vec::new()));
    assert_eq!(expanded(cx), Some(false));

    click("marley-rail-disclosure-0", cx);
    assert_eq!(
        listing(&rail, cx)[0],
        ("beta".to_string(), vec![id(&terminal)])
    );
    assert_eq!(expanded(cx), Some(true));
}

#[gpui::test]
async fn a_terminal_row_shows_its_project_and_focuses_the_terminal(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, _, _, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&alpha, false, cx);
    click(terminal_selector(&terminal), cx);
    cx.update(|window, cx| {
        assert_eq!(multi_workspace.read(cx).workspace(), &alpha);
        let active = alpha
            .read(cx)
            .active_item(cx)
            .and_then(|item| item.downcast::<TerminalView>());
        assert_eq!(active, Some(terminal.clone()));
        assert!(terminal.focus_handle(cx).is_focused(window));
    });
}

#[gpui::test]
async fn the_selected_row_follows_what_the_window_shows(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, beta, rail, cx) = open_rail(cx).await;
    assert_eq!(selected(&rail, cx), Selection::Project(0));

    let (_, terminal) = add_terminal(&beta, true, cx);
    assert_eq!(selected(&rail, cx), Selection::Terminal(id(&terminal)));

    click("marley-rail-project-1", cx);
    assert_eq!(
        multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone()),
        alpha
    );
    assert_eq!(selected(&rail, cx), Selection::Project(1));
}

#[gpui::test]
async fn new_terminal_starts_in_its_projects_directory(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, _, rail, cx) = open_rail(cx).await;
    click("marley-rail-project-menu-1", cx);
    click("MENU_ITEM-New Terminal", cx);
    cx.read(|cx| {
        assert_eq!(
            cx.global::<RequestedDirectories>().0,
            [Some(PathBuf::from(path!("/alpha")))]
        );
        assert_eq!(multi_workspace.read(cx).workspace(), &alpha);
    });
    let terminal = alpha
        .read_with(cx, |alpha, cx| {
            alpha
                .active_item(cx)
                .and_then(|item| item.downcast::<TerminalView>())
        })
        .expect("the new terminal is the active item");
    assert_eq!(
        listing(&rail, cx)[1],
        ("alpha".to_string(), vec![id(&terminal)])
    );
}

#[gpui::test]
async fn a_bell_marks_its_row_and_the_rail_until_the_row_is_opened(cx: &mut TestAppContext) {
    let (_, alpha, _, rail, cx) = open_rail(cx).await;
    let (terminal, view) = add_terminal(&alpha, false, cx);
    let bell = format!("marley-rail-bell-{}", id(&view)).leak();
    cx.update(|window, _| window.refresh());
    assert!(!has_notifications(&rail, cx));
    assert!(cx.debug_bounds(bell).is_none());

    // The view marks the bell without a notify, so only the rail's own redraw can show the dot.
    terminal.update(cx, |_, cx| cx.emit(terminal::Event::Bell));
    cx.run_until_parked();
    assert!(has_notifications(&rail, cx));
    assert!(cx.debug_bounds(bell).is_some());

    // Folded away, the terminal's bell shows on its project's header.
    click("marley-rail-disclosure-1", cx);
    assert!(cx.debug_bounds("marley-rail-attention-1").is_some());
    click("marley-rail-disclosure-1", cx);

    click(terminal_selector(&view), cx);
    assert!(!view.read_with(cx, |view, _| view.has_bell()));
    assert!(!has_notifications(&rail, cx));
}

#[gpui::test]
async fn a_bell_while_the_rail_is_closed_lights_the_sidebar_toggle(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, _, _, cx) = open_rail(cx).await;
    let (terminal, _) = add_terminal(&alpha, false, cx);
    multi_workspace.update_in(cx, |multi_workspace, window, cx| {
        multi_workspace.close_sidebar(window, cx);
    });
    cx.run_until_parked();
    terminal.update(cx, |_, cx| cx.emit(terminal::Event::Bell));
    cx.run_until_parked();
    assert!(multi_workspace.read_with(cx, |multi_workspace, cx| {
        multi_workspace.sidebar_has_notifications(cx)
    }));
}

#[gpui::test]
async fn closing_a_terminal_takes_its_row_away(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, true, cx);
    assert_eq!(listing(&rail, cx)[0].1, [id(&terminal)]);
    let pane = beta.read_with(cx, |beta, _| beta.active_pane().clone());
    pane.update_in(cx, |pane, window, cx| {
        pane.close_item_by_id(terminal.entity_id(), SaveIntent::Skip, window, cx)
    })
    .await
    .expect("the tab closes");
    cx.run_until_parked();
    assert_eq!(listing(&rail, cx)[0].1, Vec::<u64>::new());
}

#[gpui::test]
async fn the_rail_keeps_its_width_in_bounds_and_sits_on_the_left(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    assert_eq!(rail.read_with(cx, Sidebar::width), px(260.));
    for (requested, expected) in [
        (Some(px(300.)), px(300.)),
        (Some(px(10.)), px(180.)),
        (Some(px(10_000.)), px(600.)),
        (None, px(260.)),
    ] {
        rail.update(cx, |rail, cx| rail.set_width(requested, cx));
        assert_eq!(rail.read_with(cx, Sidebar::width), expected);
    }
    rail.read_with(cx, |rail, cx| {
        assert_eq!(rail.side(cx), SidebarSide::Left);
        assert!(!rail.is_threads_list_view_active());
    });
}

#[gpui::test]
async fn zeds_saved_sidebar_state_is_kept_unread_for_zed(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    rail.update_in(cx, |rail, window, cx| {
        rail.restore_serialized_state(ZED_SIDEBAR_STATE, window, cx);
    });
    rail.read_with(cx, |rail, cx| {
        assert_eq!(rail.width(cx), px(260.));
        assert_eq!(
            rail.serialized_state(cx).as_deref(),
            Some(ZED_SIDEBAR_STATE)
        );
    });
}

#[gpui::test]
async fn add_project_opens_the_recent_projects_popover(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    click("marley-rail-add-project", cx);
    assert!(rail.read_with(cx, |rail, _| rail.add_project_menu.is_deployed()));
}

#[gpui::test]
async fn the_header_still_draws_in_fullscreen(cx: &mut TestAppContext) {
    let (_, _, _, _, cx) = open_rail(cx).await;
    cx.update(|window, _| {
        window.toggle_fullscreen();
        window.refresh();
    });
    cx.run_until_parked();
    cx.update(|window, _| assert!(window.is_fullscreen()));
    assert!(cx.debug_bounds("marley-rail-add-project").is_some());
}

// ── W3: Zed agent threads in the rail ────────────────────────────────────────

mod threads {
    use acp_thread::{PermissionOptions, StubAgentConnection};
    use agent_client_protocol::schema::v1 as acp;
    use agent_ui::test_support::{
        active_session_id, active_thread_id, open_thread_with_connection, send_message,
    };
    use gpui::UpdateGlobal as _;
    use project::WorktreePaths;
    use settings::SettingsStore;

    use super::*;
    use crate::marley_workbench_tests::init_agent_test;

    /// A window over `alpha` and `beta` in the Marley layout with an Agent Panel in each, `beta`
    /// displayed. The rail lists `beta` first.
    async fn open_rail_with_agents(
        cx: &mut TestAppContext,
    ) -> (
        Entity<MultiWorkspace>,
        [Entity<Workspace>; 2],
        [Entity<AgentPanel>; 2],
        Entity<Rail>,
        &mut VisualTestContext,
    ) {
        init_agent_test(cx);
        set_layout(MarleyLayout::Marley, cx);
        let (multi_workspace, workspaces, cx) =
            open_projects(&[path!("/alpha"), path!("/beta")], cx).await;
        let alpha = workspaces[0].clone();
        let beta = workspaces[1].clone();
        let panels = [add_agent_panel(&alpha, cx), add_agent_panel(&beta, cx)];
        register(&multi_workspace, cx);
        let rail = cx
            .read(|cx| rail_of(&multi_workspace, cx))
            .expect("the Marley layout registers the rail");
        (multi_workspace, [alpha, beta], panels, rail, cx)
    }

    fn add_agent_panel(
        workspace: &Entity<Workspace>,
        cx: &mut VisualTestContext,
    ) -> Entity<AgentPanel> {
        workspace.update_in(cx, |workspace, window, cx| {
            let panel = cx.new(|cx| AgentPanel::test_new(workspace, window, cx));
            workspace.add_panel(panel.clone(), window, cx);
            panel
        })
    }

    /// Each project's name with its thread rows, in the rail's order: title, status and
    /// attention.
    type ThreadListing = Vec<(String, Vec<(String, ThreadStatus, bool)>)>;

    fn threads(rail: &Entity<Rail>, cx: &VisualTestContext) -> ThreadListing {
        rail.read_with(cx, |rail, _| {
            rail.snapshot
                .rail
                .projects
                .iter()
                .map(|project| {
                    (
                        project.name.clone(),
                        project
                            .threads
                            .iter()
                            .map(|thread| (thread.title.clone(), thread.status, thread.attention))
                            .collect(),
                    )
                })
                .collect()
        })
    }

    fn status_of(rail: &Entity<Rail>, key: &str, cx: &VisualTestContext) -> Option<ThreadStatus> {
        rail.read_with(cx, |rail, _| {
            rail.snapshot
                .rail
                .projects
                .iter()
                .flat_map(|project| &project.threads)
                .find(|thread| thread.key == key)
                .map(|thread| thread.status)
        })
    }

    fn attention_of(rail: &Entity<Rail>, key: &str, cx: &VisualTestContext) -> bool {
        rail.read_with(cx, |rail, _| {
            rail.snapshot
                .rail
                .projects
                .iter()
                .flat_map(|project| &project.threads)
                .any(|thread| thread.key == key && thread.attention)
        })
    }

    fn key_of(panel: &Entity<AgentPanel>, cx: &VisualTestContext) -> String {
        active_thread_id(panel, cx).to_key_string()
    }

    fn thread_selector(key: &str) -> &'static str {
        format!("marley-rail-thread-{key}").leak()
    }

    /// Saves a thread row the way the Agent Panel does, `minute` minutes into a fixed hour.
    fn seed(
        title: &str,
        main: &str,
        folder: &str,
        minute: i64,
        archived: bool,
        draft: bool,
        cx: &mut VisualTestContext,
    ) {
        let at = chrono::DateTime::from_timestamp(1_800_000_000 + minute * 60, 0)
            .expect("a time in range");
        let metadata = ThreadMetadata {
            thread_id: ThreadId::new(),
            session_id: (!draft).then(|| acp::SessionId::new(format!("session {title}"))),
            agent_id: AgentId::new("stub"),
            title: Some(title.to_string().into()),
            title_override: None,
            updated_at: at,
            created_at: None,
            interacted_at: Some(at),
            worktree_paths: WorktreePaths::from_path_lists(
                PathList::new(&[main]),
                PathList::new(&[folder]),
            )
            .expect("one main path per folder path"),
            remote_connection: None,
            archived,
        };
        cx.update(|_, cx| {
            ThreadMetadataStore::global(cx).update(cx, |store, cx| store.save(metadata, cx));
        });
        cx.run_until_parked();
    }

    /// Starts a thread on `connection` in `panel` and sends it a message, so it runs until the
    /// stub ends its turn.
    fn start_thread(
        panel: &Entity<AgentPanel>,
        connection: &StubAgentConnection,
        cx: &mut VisualTestContext,
    ) -> String {
        open_thread_with_connection(panel, connection.clone(), cx);
        send_message(panel, cx);
        key_of(panel, cx)
    }

    /// Replaces the user settings with the Marley layout and one custom agent per name.
    fn configure_agents(names: &[&str], cx: &mut VisualTestContext) {
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

    /// Opens a project's `+` menu and its New Agent Thread submenu from the keyboard: the
    /// submenu's trigger carries no selector to click.
    fn open_agent_submenu(project: usize, cx: &mut VisualTestContext) {
        click(format!("marley-rail-project-menu-{project}").leak(), cx);
        // The submenu's trigger, the row under New Terminal, carries no selector of its own, and a
        // popover's menu takes focus only on a platform frame a test never delivers, so the
        // pointer opens it.
        let new_terminal = cx
            .debug_bounds("MENU_ITEM-New Terminal")
            .expect("the project's menu is open");
        let trigger = gpui::point(new_terminal.center().x, new_terminal.bottom() + px(12.));
        cx.simulate_mouse_move(trigger, None, Modifiers::none());
        cx.simulate_click(trigger, Modifiers::none());
        // The submenu anchors to its trigger's bounds from the frame before it draws.
        draw_frames(2, cx);
    }

    fn draw_frames(count: usize, cx: &mut VisualTestContext) {
        for _ in 0..count {
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
        }
    }

    #[gpui::test]
    async fn threads_list_under_their_project_newest_first(cx: &mut TestAppContext) {
        let (_, [alpha, _], _, rail, cx) = open_rail_with_agents(cx).await;
        let (_, view) = add_terminal(&alpha, false, cx);
        seed(
            "old",
            path!("/alpha"),
            path!("/alpha"),
            10,
            false,
            false,
            cx,
        );
        seed(
            "new",
            path!("/alpha"),
            path!("/alpha"),
            20,
            false,
            false,
            cx,
        );
        // Written before main paths were kept: found by its folder paths.
        seed(
            "legacy",
            path!("/elsewhere"),
            path!("/alpha"),
            15,
            false,
            false,
            cx,
        );
        seed(
            "archived",
            path!("/alpha"),
            path!("/alpha"),
            30,
            true,
            false,
            cx,
        );
        seed(
            "draft",
            path!("/alpha"),
            path!("/alpha"),
            40,
            false,
            true,
            cx,
        );
        seed("other", path!("/beta"), path!("/beta"), 5, false, false, cx);
        let titles = threads(&rail, cx)
            .into_iter()
            .map(|(project, threads)| {
                let titles: Vec<String> = threads.into_iter().map(|(title, ..)| title).collect();
                (project, titles)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            titles,
            [
                ("beta".to_string(), vec!["other".to_string()]),
                (
                    "alpha".to_string(),
                    vec!["new".to_string(), "legacy".to_string(), "old".to_string()]
                ),
            ]
        );
        // Under its project, a thread's row follows the terminals.
        let rows = rail.read_with(cx, |rail, _| marley_rail::rail_rows(&rail.snapshot.rail));
        let terminal = rows
            .iter()
            .position(|row| matches!(row, Row::Terminal(row) if row.id == id(&view)))
            .expect("the terminal's row");
        let first_thread = rows
            .iter()
            .position(|row| matches!(row, Row::Thread(row) if row.project == 1))
            .expect("alpha's first thread row");
        assert!(terminal < first_thread, "{rows:?}");
        cx.update(|window, _| window.refresh());
        let keys: Vec<String> = rail.read_with(cx, |rail, _| {
            rail.snapshot.threads.keys().cloned().collect()
        });
        for key in keys {
            assert!(cx.debug_bounds(thread_selector(&key)).is_some(), "{key}");
        }
    }

    #[gpui::test]
    async fn a_live_threads_row_follows_its_status(cx: &mut TestAppContext) {
        let (_, _, [alpha_panel, _], rail, cx) = open_rail_with_agents(cx).await;
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        assert_eq!(status_of(&rail, &key, cx), Some(ThreadStatus::Running));
        connection.end_turn(
            active_session_id(&alpha_panel, cx),
            acp::StopReason::EndTurn,
        );
        cx.run_until_parked();
        assert_eq!(status_of(&rail, &key, cx), Some(ThreadStatus::Done));
    }

    #[gpui::test]
    async fn a_thread_row_opens_its_thread_in_its_projects_agent_panel(cx: &mut TestAppContext) {
        let (multi_workspace, [alpha, beta], [alpha_panel, _], rail, cx) =
            open_rail_with_agents(cx).await;
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        connection.end_turn(
            active_session_id(&alpha_panel, cx),
            acp::StopReason::EndTurn,
        );
        cx.run_until_parked();
        multi_workspace.update_in(cx, |multi_workspace, window, cx| {
            multi_workspace.activate(beta.clone(), None, window, cx);
        });
        cx.run_until_parked();
        click(thread_selector(&key), cx);
        assert_eq!(
            multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone()),
            alpha
        );
        assert_eq!(key_of(&alpha_panel, cx), key);
        let focused =
            cx.update(|window, cx| alpha_panel.focus_handle(cx).contains_focused(window, cx));
        assert!(focused, "the Agent Panel takes focus");
        assert_eq!(selected(&rail, cx), Selection::Thread(key));
    }

    #[gpui::test]
    async fn new_agent_thread_starts_one_in_that_projects_panel(cx: &mut TestAppContext) {
        let (multi_workspace, [alpha, _], [alpha_panel, _], rail, cx) =
            open_rail_with_agents(cx).await;
        configure_agents(&["stub"], cx);
        open_agent_submenu(1, cx);
        click("MENU_ITEM-stub", cx);
        assert_eq!(
            multi_workspace.read_with(cx, |multi_workspace, _| multi_workspace.workspace().clone()),
            alpha
        );
        let key = key_of(&alpha_panel, cx);
        let focused =
            cx.update(|window, cx| alpha_panel.focus_handle(cx).contains_focused(window, cx));
        assert!(focused, "the new thread's panel takes focus");
        assert!(
            rail.read_with(cx, |rail, _| rail.snapshot.threads.contains_key(&key)),
            "the new thread is a row"
        );
    }

    #[gpui::test]
    async fn the_agent_submenu_lists_the_zed_agent_then_agents_by_name(cx: &mut TestAppContext) {
        let (_, _, _, _, cx) = open_rail_with_agents(cx).await;
        configure_agents(&["zeta", "Alpha", "beta"], cx);
        open_agent_submenu(1, cx);
        let tops: Vec<Pixels> = ["Zed Agent", "Alpha", "beta", "zeta"]
            .into_iter()
            .map(|name| {
                cx.debug_bounds(format!("MENU_ITEM-{name}").leak())
                    .unwrap_or_else(|| panic!("{name} is listed"))
                    .top()
            })
            .collect();
        assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{tops:?}");
    }

    #[gpui::test]
    async fn a_run_that_ends_unseen_lights_the_dot_until_the_thread_is_shown(
        cx: &mut TestAppContext,
    ) {
        let (multi_workspace, [_, beta], [alpha_panel, _], rail, cx) =
            open_rail_with_agents(cx).await;
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        multi_workspace.update_in(cx, |multi_workspace, window, cx| {
            multi_workspace.activate(beta.clone(), None, window, cx);
        });
        cx.run_until_parked();
        assert!(!attention_of(&rail, &key, cx));
        assert!(!has_notifications(&rail, cx));
        connection.end_turn(
            active_session_id(&alpha_panel, cx),
            acp::StopReason::EndTurn,
        );
        cx.run_until_parked();
        assert!(attention_of(&rail, &key, cx));
        assert!(has_notifications(&rail, cx));
        click(thread_selector(&key), cx);
        assert!(!attention_of(&rail, &key, cx));
        assert!(!has_notifications(&rail, cx));
    }

    #[gpui::test]
    async fn a_thread_waiting_for_a_confirmation_needs_the_user(cx: &mut TestAppContext) {
        let (_, _, [alpha_panel, _], rail, cx) = open_rail_with_agents(cx).await;
        let tool_call_id = acp::ToolCallId::new("1");
        let connection =
            StubAgentConnection::new().with_permission_requests(HashMap::from_iter([(
                tool_call_id.clone(),
                PermissionOptions::Flat(vec![acp::PermissionOption::new(
                    "1",
                    "Allow",
                    acp::PermissionOptionKind::AllowOnce,
                )]),
            )]));
        connection.set_next_prompt_updates(vec![acp::SessionUpdate::ToolCall(
            acp::ToolCall::new(tool_call_id, "Edit a file").kind(acp::ToolKind::Edit),
        )]);
        let key = start_thread(&alpha_panel, &connection, cx);
        assert_eq!(status_of(&rail, &key, cx), Some(ThreadStatus::Waiting));
        assert!(has_notifications(&rail, cx));
        // Folded away, the wait shows on alpha's header.
        click("marley-rail-disclosure-1", cx);
        cx.update(|window, _| window.refresh());
        assert!(cx.debug_bounds("marley-rail-attention-1").is_some());
    }

    #[gpui::test]
    async fn focus_picks_the_thread_row_or_the_terminal_row(cx: &mut TestAppContext) {
        let (multi_workspace, [alpha, _], [alpha_panel, _], rail, cx) =
            open_rail_with_agents(cx).await;
        multi_workspace.update_in(cx, |multi_workspace, window, cx| {
            multi_workspace.activate(alpha.clone(), None, window, cx);
        });
        let (_, view) = add_terminal(&alpha, true, cx);
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        // Opening the dock is what draws the panel, so focus can land inside it.
        alpha.update_in(cx, |workspace, window, cx| {
            workspace.focus_panel::<AgentPanel>(window, cx);
        });
        cx.run_until_parked();
        assert_eq!(selected(&rail, cx), Selection::Thread(key));
        view.update_in(cx, |view, window, cx| {
            view.focus_handle(cx).focus(window, cx);
        });
        cx.run_until_parked();
        assert_eq!(selected(&rail, cx), Selection::Terminal(id(&view)));
    }

    #[gpui::test]
    async fn agents_take_their_name_and_icon_from_the_registry(cx: &mut TestAppContext) {
        use project::agent_registry_store::{RegistryAgentMetadata, RegistryNpxAgent};
        let (_, _, _, rail, cx) = open_rail_with_agents(cx).await;
        cx.update(|_, cx| {
            AgentRegistryStore::init_test_global(
                cx,
                vec![project::RegistryAgent::Npx(RegistryNpxAgent {
                    metadata: RegistryAgentMetadata {
                        id: AgentId::new("zeta"),
                        name: "Zeta Code".into(),
                        description: "An agent from the registry".into(),
                        version: "1.0.0".into(),
                        repository: None,
                        website: None,
                        license_url: None,
                        icon_path: Some("icons/zeta.svg".into()),
                    },
                    package: "zeta".into(),
                    args: Vec::new(),
                    env: HashMap::default(),
                })],
            );
        });
        configure_agents(&["zeta"], cx);
        // A thread the registry's agent ran draws the registry's icon.
        let at = chrono::DateTime::from_timestamp(1_800_000_000, 0).expect("a time in range");
        let thread_id = ThreadId::new();
        cx.update(|_, cx| {
            ThreadMetadataStore::global(cx).update(cx, |store, cx| {
                store.save(
                    ThreadMetadata {
                        thread_id,
                        session_id: Some(acp::SessionId::new("zeta session")),
                        agent_id: AgentId::new("zeta"),
                        title: Some("zeta's thread".into()),
                        title_override: None,
                        updated_at: at,
                        created_at: None,
                        interacted_at: None,
                        worktree_paths: WorktreePaths::from_folder_paths(&PathList::new(&[path!(
                            "/alpha"
                        )])),
                        remote_connection: None,
                        archived: false,
                    },
                    cx,
                );
            });
        });
        cx.run_until_parked();
        let icon = rail.read_with(cx, |rail, _| {
            rail.snapshot
                .threads
                .get(&thread_id.to_key_string())
                .map(|thread| thread.icon.clone())
        });
        assert!(
            matches!(icon, Some(AgentIcon::Svg(ref path)) if path.as_ref() == "icons/zeta.svg"),
            "the registry's icon"
        );
        cx.update(|window, _| window.refresh());
        assert!(
            cx.debug_bounds(thread_selector(&thread_id.to_key_string()))
                .is_some()
        );
        // The menu names the agent as the registry does.
        open_agent_submenu(1, cx);
        assert!(cx.debug_bounds("MENU_ITEM-Zeta Code").is_some());
    }

    #[test]
    fn each_rail_status_draws_as_zeds_thread_status() {
        assert_eq!(ui_status(ThreadStatus::Done), AgentThreadStatus::Completed);
        assert_eq!(ui_status(ThreadStatus::Running), AgentThreadStatus::Running);
        assert_eq!(
            ui_status(ThreadStatus::Waiting),
            AgentThreadStatus::WaitingForConfirmation
        );
        assert_eq!(ui_status(ThreadStatus::Error), AgentThreadStatus::Error);
    }

    #[gpui::test]
    async fn the_rail_does_not_claim_the_threads_list(cx: &mut TestAppContext) {
        let (multi_workspace, _, [alpha_panel, _], rail, cx) = open_rail_with_agents(cx).await;
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        assert!(status_of(&rail, &key, cx).is_some());
        assert!(!multi_workspace.read_with(cx, |multi_workspace, cx| {
            multi_workspace.is_threads_list_view_active(cx)
        }));
    }
}
