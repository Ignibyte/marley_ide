//! Driven tests for the rail: its rows, the clicks and menus on them, the bell, and the answers
//! the `MultiWorkspace` reads from it.

use std::path::Path;

use fs::FakeFs;
use gpui::{
    Global, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, VisualTestContext,
};
use marley_rail::Selection;
use project::{AgentRegistryStore, Project};
use serde_json::json;
use settings::MarleyLayout;
use util::{path, path_list::PathList};
use workspace::SaveIntent;

use super::*;
use crate::marley_workbench_tests::{
    RequestedDirectories, ZED_SIDEBAR_STATE, assert_keeps_zeds_fields, display_only_terminal,
    init_test, open_projects, rail_of, register, set_layout, use_display_only_terminals,
};

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
    use_display_only_terminals(cx);
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
async fn zeds_saved_sidebar_state_sizes_the_rail_and_is_kept_for_zed(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    rail.update_in(cx, |rail, window, cx| {
        rail.restore_serialized_state(ZED_SIDEBAR_STATE, window, cx);
    });
    rail.read_with(cx, |rail, cx| {
        assert_eq!(rail.width(cx), px(321.), "one width for both layouts");
        let saved = rail.serialized_state(cx).expect("the rail's blob");
        assert_keeps_zeds_fields(&saved, ZED_SIDEBAR_STATE);
    });
}

#[test]
fn the_rail_writes_its_fields_into_zeds_blob_and_keeps_the_rest() {
    let zed = r#"{"width":321.0,"width_set_by_user":true,"active_view":"History","later":1}"#;
    let state = RailState {
        width: Some(400.),
        closed: true,
    };
    let blob = write_rail_state(Some(zed), state);
    let saved: serde_json::Value = serde_json::from_str(&blob).expect("JSON");
    assert_eq!(
        saved,
        json!({
            "width": 400.0,
            "width_set_by_user": true,
            "active_view": "History",
            "later": 1,
            "marley_rail_closed": true,
        })
    );
    assert_eq!(read_rail_state(&blob), state);
}

#[test]
fn a_width_the_user_did_not_set_is_not_saved() {
    let blob = write_rail_state(None, RailState::default());
    let saved: serde_json::Value = serde_json::from_str(&blob).expect("JSON");
    assert_eq!(
        saved,
        json!({ "width": null, "width_set_by_user": false, "marley_rail_closed": false })
    );
    // A width without Zed's flag is not the user's.
    assert_eq!(read_rail_state(r#"{"width":300.0}"#), RailState::default());
}

#[test]
fn an_unreadable_blob_holds_nothing_for_the_rail() {
    assert_eq!(read_rail_state("not json"), RailState::default());
    assert_eq!(read_rail_state("[1, 2]"), RailState::default());
    // A base that is not an object is started afresh.
    let closed = RailState {
        width: None,
        closed: true,
    };
    assert_eq!(
        read_rail_state(&write_rail_state(Some("[1, 2]"), closed)),
        closed
    );
}

/// Draws the window afresh and right-clicks the middle of what is drawn under `selector`.
fn right_click(selector: &'static str, cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    let bounds = cx.debug_bounds(selector).expect("the element is drawn");
    cx.simulate_mouse_down(bounds.center(), MouseButton::Right, Modifiers::none());
    cx.simulate_mouse_up(bounds.center(), MouseButton::Right, Modifiers::none());
    cx.run_until_parked();
    // The menu is drawn on the frame after it opens.
    cx.update(|window, _| window.refresh());
}

/// Draws the window afresh and double-clicks the middle of what is drawn under `selector`.
fn double_click(selector: &'static str, cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    let position = cx
        .debug_bounds(selector)
        .expect("the element is drawn")
        .center();
    for click_count in [1, 2] {
        cx.simulate_event(MouseDownEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
            first_mouse: false,
        });
        cx.simulate_event(MouseUpEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
        });
    }
    cx.run_until_parked();
}

fn title_of(
    rail: &Entity<Rail>,
    view: &Entity<TerminalView>,
    cx: &VisualTestContext,
) -> Option<String> {
    rail.read_with(cx, |rail, _| {
        rail.snapshot
            .rail
            .projects
            .iter()
            .flat_map(|project| &project.terminals)
            .find(|terminal| terminal.id == id(view))
            .map(|terminal| terminal.title.clone())
    })
}

/// Types `name` into a terminal's rename and presses Enter.
fn finish_rename(name: &str, cx: &mut VisualTestContext) {
    cx.simulate_input(name);
    cx.update(|window, _| window.refresh());
    cx.dispatch_action(Confirm);
    cx.run_until_parked();
}

fn is_open(
    workspace: &Entity<Workspace>,
    view: &Entity<TerminalView>,
    cx: &VisualTestContext,
) -> bool {
    workspace.read_with(cx, |workspace, cx| {
        workspace
            .items_of_type::<TerminalView>(cx)
            .any(|open| open == *view)
    })
}

#[gpui::test]
async fn a_terminal_rows_menu_renames_it_through_its_tab(cx: &mut TestAppContext) {
    let (_, alpha, beta, rail, cx) = open_rail(cx).await;
    let (_, alpha_terminal) = add_terminal(&alpha, false, cx);
    let (_, beta_terminal) = add_terminal(&beta, true, cx);
    right_click(terminal_selector(&alpha_terminal), cx);
    assert!(cx.debug_bounds("MENU_ITEM-Rename").is_some());
    assert!(cx.debug_bounds("MENU_ITEM-Close").is_some());
    click("MENU_ITEM-Rename", cx);
    // The terminal is shown first, so the tab it renames is on screen.
    cx.read(|cx| {
        assert_eq!(
            alpha.read(cx).active_item(cx).map(|item| item.item_id()),
            Some(alpha_terminal.entity_id())
        );
    });
    assert!(alpha_terminal.read_with(cx, |view, _| view.is_renaming()));
    finish_rename("Build", cx);
    assert_eq!(
        alpha_terminal.read_with(cx, |view, _| view.custom_title().map(str::to_string)),
        Some("Build".to_string())
    );
    assert_eq!(
        title_of(&rail, &alpha_terminal, cx).as_deref(),
        Some("Build")
    );
    assert!(!beta_terminal.read_with(cx, |view, _| view.is_renaming()));
}

#[gpui::test]
async fn a_double_click_on_a_row_renames_its_terminal(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, false, cx);
    double_click(terminal_selector(&terminal), cx);
    assert!(terminal.read_with(cx, |view, _| view.is_renaming()));
    finish_rename("Tests", cx);
    assert_eq!(title_of(&rail, &terminal, cx).as_deref(), Some("Tests"));
}

#[gpui::test]
async fn a_rows_close_button_closes_its_terminal(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, kept) = add_terminal(&beta, false, cx);
    let (_, closed) = add_terminal(&beta, true, cx);
    cx.update(|window, _| window.refresh());
    let row = cx
        .debug_bounds(terminal_selector(&closed))
        .expect("the row is drawn");
    // The button shows while the pointer is over the row.
    cx.simulate_mouse_move(row.center(), None, Modifiers::none());
    click(
        format!("marley-rail-terminal-close-{}", id(&closed)).leak(),
        cx,
    );
    assert!(!is_open(&beta, &closed, cx));
    assert!(is_open(&beta, &kept, cx));
    assert_eq!(
        title_of(&rail, &closed, cx),
        None,
        "its row leaves the rail"
    );
}

#[gpui::test]
async fn a_rows_menu_closes_its_terminal(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, false, cx);
    right_click(terminal_selector(&terminal), cx);
    click("MENU_ITEM-Close", cx);
    assert!(!is_open(&beta, &terminal, cx));
    assert_eq!(title_of(&rail, &terminal, cx), None);
}

/// Focuses the rail and draws it, as the sidebar's focus action does.
fn focus_rail(rail: &Entity<Rail>, cx: &mut VisualTestContext) {
    rail.update_in(cx, |rail, window, cx| {
        rail.focus_handle(cx).focus(window, cx);
    });
    cx.run_until_parked();
    cx.update(|window, _| window.refresh());
}

/// Dispatches `action` from whatever has focus.
fn dispatch(action: impl gpui::Action, cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.dispatch_action(action);
    cx.run_until_parked();
}

#[gpui::test]
async fn the_arrow_keys_walk_the_rails_rows(cx: &mut TestAppContext) {
    let (_, alpha, beta, rail, cx) = open_rail(cx).await;
    let (_, alpha_terminal) = add_terminal(&alpha, false, cx);
    let (_, beta_terminal) = add_terminal(&beta, true, cx);
    // The rail lists beta first: its header, its terminal, then alpha's header and terminal.
    focus_rail(&rail, cx);
    assert_eq!(selected(&rail, cx), Selection::Terminal(id(&beta_terminal)));
    dispatch(SelectNext, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(1));
    dispatch(SelectNext, cx);
    assert_eq!(
        selected(&rail, cx),
        Selection::Terminal(id(&alpha_terminal))
    );
    dispatch(SelectNext, cx);
    assert_eq!(
        selected(&rail, cx),
        Selection::Terminal(id(&alpha_terminal)),
        "the last stays"
    );
    dispatch(SelectPrevious, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(1));
    dispatch(SelectFirst, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(0));
    dispatch(SelectLast, cx);
    assert_eq!(
        selected(&rail, cx),
        Selection::Terminal(id(&alpha_terminal))
    );
}

#[gpui::test]
async fn enter_opens_the_highlighted_terminal(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, beta, rail, cx) = open_rail(cx).await;
    let (_, alpha_terminal) = add_terminal(&alpha, false, cx);
    add_terminal(&beta, true, cx);
    focus_rail(&rail, cx);
    dispatch(SelectLast, cx);
    dispatch(Confirm, cx);
    cx.read(|cx| assert_eq!(multi_workspace.read(cx).workspace(), &alpha));
    let focused =
        cx.update(|window, cx| alpha_terminal.focus_handle(cx).contains_focused(window, cx));
    assert!(focused, "the terminal takes focus");
    // With focus out of the rail, the highlight is the window's row again.
    assert_eq!(
        selected(&rail, cx),
        Selection::Terminal(id(&alpha_terminal))
    );
}

#[gpui::test]
async fn the_keyboards_row_goes_when_focus_leaves_the_rail(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, true, cx);
    focus_rail(&rail, cx);
    dispatch(SelectLast, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(1));
    cx.update(|window, cx| terminal.focus_handle(cx).focus(window, cx));
    cx.run_until_parked();
    assert_eq!(selected(&rail, cx), Selection::Terminal(id(&terminal)));
    // Back in the rail, the keyboard starts from the highlighted row, not the old one.
    focus_rail(&rail, cx);
    dispatch(SelectPrevious, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(0));
}

#[gpui::test]
async fn enter_shows_the_highlighted_project(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, _, rail, cx) = open_rail(cx).await;
    focus_rail(&rail, cx);
    dispatch(SelectNext, cx);
    assert_eq!(selected(&rail, cx), Selection::Project(1));
    dispatch(Confirm, cx);
    cx.read(|cx| assert_eq!(multi_workspace.read(cx).workspace(), &alpha));
    assert_eq!(selected(&rail, cx), Selection::Project(1));
}

#[gpui::test]
async fn enter_with_no_row_highlighted_does_nothing(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, beta, rail, cx) = open_rail(cx).await;
    // With its last folder gone, the displayed workspace leaves its project's row, so no row is
    // the window's.
    beta.update(cx, |beta, cx| {
        let project = beta.project().clone();
        let worktree = project
            .read(cx)
            .worktrees(cx)
            .next()
            .map(|worktree| worktree.read(cx).id());
        if let Some(worktree) = worktree {
            project.update(cx, |project, cx| project.remove_worktree(worktree, cx));
        }
    });
    cx.run_until_parked();
    // The removal alone does not rebuild the rail (TICKET-458); the next change in the window
    // does.
    add_terminal(&alpha, false, cx);
    assert_eq!(names(&rail, cx), ["alpha"]);
    focus_rail(&rail, cx);
    assert_eq!(selected(&rail, cx), Selection::None);
    dispatch(Confirm, cx);
    cx.read(|cx| assert_eq!(multi_workspace.read(cx).workspace(), &beta));
}

#[gpui::test]
async fn left_and_right_fold_unfold_and_climb(cx: &mut TestAppContext) {
    let (_, _, beta, rail, cx) = open_rail(cx).await;
    let (_, terminal) = add_terminal(&beta, true, cx);
    focus_rail(&rail, cx);
    assert_eq!(selected(&rail, cx), Selection::Terminal(id(&terminal)));
    dispatch(SelectParent, cx);
    assert_eq!(
        selected(&rail, cx),
        Selection::Project(0),
        "left climbs to the header"
    );
    dispatch(SelectParent, cx);
    assert!(
        !listing(&rail, cx)[0].1.contains(&id(&terminal)),
        "left folds the project"
    );
    dispatch(SelectParent, cx);
    assert!(
        listing(&rail, cx)[0].1.is_empty(),
        "a folded project stays folded"
    );
    dispatch(SelectChild, cx);
    assert_eq!(
        listing(&rail, cx)[0].1,
        vec![id(&terminal)],
        "right unfolds it"
    );
    dispatch(SelectChild, cx);
    assert_eq!(
        listing(&rail, cx)[0].1,
        vec![id(&terminal)],
        "an open project stays open"
    );
}

#[gpui::test]
async fn a_project_headers_menu_moves_it_up_and_down(cx: &mut TestAppContext) {
    let (_, _, _, rail, cx) = open_rail(cx).await;
    assert_eq!(names(&rail, cx), ["beta", "alpha"]);
    right_click("marley-rail-project-0", cx);
    click("MENU_ITEM-Move Project Down", cx);
    assert_eq!(names(&rail, cx), ["alpha", "beta"]);
    right_click("marley-rail-project-1", cx);
    click("MENU_ITEM-Move Project Up", cx);
    assert_eq!(names(&rail, cx), ["beta", "alpha"]);
    // At the ends the moves that would leave the list are disabled.
    right_click("marley-rail-project-0", cx);
    click("MENU_ITEM-Move Project Up", cx);
    assert_eq!(names(&rail, cx), ["beta", "alpha"]);
}

// Zed's default keys differ by platform; these are Linux's.
#[cfg(target_os = "linux")]
#[gpui::test]
async fn zeds_default_keys_walk_and_open_the_rail(cx: &mut TestAppContext) {
    let (multi_workspace, alpha, beta, rail, cx) = open_rail(cx).await;
    let (_, alpha_terminal) = add_terminal(&alpha, false, cx);
    add_terminal(&beta, true, cx);
    cx.update(|_, cx| {
        let bindings = settings::KeymapFile::load_asset_allow_partial_failure(
            settings::DEFAULT_KEYMAP_PATH,
            cx,
        )
        .expect("Zed's default keymap loads");
        cx.bind_keys(bindings);
    });
    // Zed's Focus Workspace Sidebar key reaches the rail from the focused terminal.
    let before = cx.update(|window, cx| rail.focus_handle(cx).contains_focused(window, cx));
    cx.update(|window, _| window.refresh());
    cx.simulate_keystrokes("ctrl-alt-;");
    cx.run_until_parked();
    let after = cx.update(|window, cx| rail.focus_handle(cx).contains_focused(window, cx));
    assert!(!before && after, "the key focuses the rail");
    for key in ["down", "down", "up", "down"] {
        cx.update(|window, _| window.refresh());
        cx.simulate_keystrokes(key);
        cx.run_until_parked();
    }
    assert_eq!(
        selected(&rail, cx),
        Selection::Terminal(id(&alpha_terminal))
    );
    // Left and right are bound only in the `menu` key context.
    cx.update(|window, _| window.refresh());
    cx.simulate_keystrokes("left");
    cx.run_until_parked();
    assert_eq!(selected(&rail, cx), Selection::Project(1));
    for key in ["left", "right", "down", "enter"] {
        cx.update(|window, _| window.refresh());
        cx.simulate_keystrokes(key);
        cx.run_until_parked();
    }
    cx.read(|cx| assert_eq!(multi_workspace.read(cx).workspace(), &alpha));
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
    use project::WorktreePaths;

    use super::*;
    use crate::marley_workbench_tests::{add_agent_panel, configure_agents, init_agent_test};

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
    async fn enter_opens_the_highlighted_thread(cx: &mut TestAppContext) {
        let (multi_workspace, [alpha, _], [alpha_panel, _], rail, cx) =
            open_rail_with_agents(cx).await;
        let connection = StubAgentConnection::new();
        let key = start_thread(&alpha_panel, &connection, cx);
        connection.end_turn(
            active_session_id(&alpha_panel, cx),
            acp::StopReason::EndTurn,
        );
        cx.run_until_parked();
        // The rail lists beta's header, then alpha's header and its thread.
        focus_rail(&rail, cx);
        dispatch(SelectFirst, cx);
        dispatch(SelectLast, cx);
        assert_eq!(selected(&rail, cx), Selection::Thread(key.clone()));
        dispatch(Confirm, cx);
        cx.read(|cx| assert_eq!(multi_workspace.read(cx).workspace(), &alpha));
        assert_eq!(key_of(&alpha_panel, cx), key);
        let focused =
            cx.update(|window, cx| alpha_panel.focus_handle(cx).contains_focused(window, cx));
        assert!(focused, "the Agent Panel takes focus");
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

// ── W4: agent CLIs in rail terminals ─────────────────────────────────────────

#[cfg(unix)]
mod agents {
    use marley_agent::{AgentKind, AgentStatus, WAITING_AFTER};

    use super::*;
    use crate::marley_workbench_tests::{programs_in, search_agents_in};

    /// The command each test terminal's foreground process runs, by terminal: a display-only
    /// terminal has no process of its own.
    #[derive(Default)]
    struct ForegroundCommands(HashMap<EntityId, String>);

    impl Global for ForegroundCommands {}

    fn fake_foreground(terminal: &Entity<Terminal>, cx: &App) -> Option<String> {
        cx.try_global::<ForegroundCommands>()?
            .0
            .get(&terminal.entity_id())
            .cloned()
    }

    /// Puts `command` in the terminal's foreground. A new foreground process retitles its
    /// terminal, which is what makes the rail read it again.
    fn set_foreground(terminal: &Entity<Terminal>, command: &str, cx: &mut VisualTestContext) {
        cx.update(|_, cx| {
            cx.default_global::<ForegroundCommands>()
                .0
                .insert(terminal.entity_id(), command.to_string());
        });
        terminal.update(cx, |_, cx| cx.emit(terminal::Event::TitleChanged));
        cx.run_until_parked();
    }

    fn agent_of(
        rail: &Entity<Rail>,
        view: &Entity<TerminalView>,
        cx: &VisualTestContext,
    ) -> Option<TerminalAgent> {
        rail.read_with(cx, |rail, _| {
            rail.snapshot
                .rail
                .projects
                .iter()
                .flat_map(|project| &project.terminals)
                .find(|terminal| terminal.id == id(view))
                .and_then(|terminal| terminal.agent)
        })
    }

    fn row_of(
        rail: &Entity<Rail>,
        view: &Entity<TerminalView>,
        cx: &VisualTestContext,
    ) -> (String, Option<String>) {
        rail.read_with(cx, |rail, _| {
            rail.snapshot
                .rail
                .projects
                .iter()
                .flat_map(|project| &project.terminals)
                .find(|terminal| terminal.id == id(view))
                .map(|terminal| (terminal.title.clone(), terminal.subtitle.clone()))
                .expect("the terminal's row")
        })
    }

    #[gpui::test]
    async fn the_plus_menu_lists_the_agent_clis_on_the_search_path(cx: &mut TestAppContext) {
        let (_, _, _, rail, cx) = open_rail(cx).await;
        let installed = programs_in(&[("claude", true), ("codex", true), ("gemini", false)]);
        search_agents_in(installed.path(), cx);
        click("marley-rail-project-menu-1", cx);
        cx.update(|window, _| window.refresh());
        assert!(cx.debug_bounds("MENU_ITEM-Claude Code").is_some());
        assert!(cx.debug_bounds("MENU_ITEM-Codex").is_some());
        assert!(
            cx.debug_bounds("MENU_ITEM-Gemini CLI").is_none(),
            "a file that is not executable is no program"
        );
        assert!(cx.debug_bounds("MENU_ITEM-OpenCode").is_none());

        // With nothing installed the section is absent.
        let empty = programs_in(&[]);
        search_agents_in(empty.path(), cx);
        // The menu reads the search path when the rail draws.
        rail.update(cx, |_, cx| cx.notify());
        // Opening another project's menu closes the first with an outside click.
        click("marley-rail-project-menu-0", cx);
        cx.update(|window, _| window.refresh());
        assert!(cx.debug_bounds("MENU_ITEM-New Terminal").is_some());
        assert!(cx.debug_bounds("MENU_ITEM-Claude Code").is_none());
    }

    #[gpui::test]
    async fn an_agent_cli_starts_in_a_new_terminal_in_its_project(cx: &mut TestAppContext) {
        let (multi_workspace, alpha, _, _, cx) = open_rail(cx).await;
        let installed = programs_in(&[("claude", true)]);
        search_agents_in(installed.path(), cx);
        click("marley-rail-project-menu-1", cx);
        click("MENU_ITEM-Claude Code", cx);
        cx.read(|cx| {
            assert_eq!(multi_workspace.read(cx).workspace(), &alpha);
            assert_eq!(
                cx.global::<RequestedDirectories>().0,
                [Some(PathBuf::from(path!("/alpha")))]
            );
        });
        let view = alpha
            .read_with(cx, |alpha, cx| {
                alpha
                    .active_item(cx)
                    .and_then(|item| item.downcast::<TerminalView>())
            })
            .expect("the new terminal is the active item");
        let written = view.update(cx, |view, cx| {
            view.terminal()
                .update(cx, |terminal, _| terminal.take_pty_write_log())
        });
        assert_eq!(
            written,
            [b"claude\r".to_vec()],
            "the program's name and Enter, only"
        );
    }

    #[gpui::test]
    async fn an_agent_row_shows_a_name_the_user_gave_its_terminal(cx: &mut TestAppContext) {
        let (_, alpha, _, rail, cx) = open_rail(cx).await;
        rail.update(cx, |rail, _| rail.foreground_command = fake_foreground);
        let (terminal, view) = add_terminal(&alpha, false, cx);
        set_foreground(&terminal, "claude", cx);
        view.update(cx, |view, cx| {
            view.set_custom_title(Some("Reviewer".into()), cx);
        });
        cx.run_until_parked();
        assert_eq!(row_of(&rail, &view, cx).0, "Reviewer");
    }

    #[gpui::test]
    async fn a_terminal_running_an_agent_cli_is_an_agent_row(cx: &mut TestAppContext) {
        let (_, alpha, _, rail, cx) = open_rail(cx).await;
        rail.update(cx, |rail, _| rail.foreground_command = fake_foreground);
        let (terminal, view) = add_terminal(&alpha, false, cx);
        assert_eq!(agent_of(&rail, &view, cx), None);

        set_foreground(&terminal, "/home/me/.local/bin/claude --resume", cx);
        let agent = agent_of(&rail, &view, cx).expect("an agent row");
        assert_eq!(agent.kind, AgentKind::Claude);
        // Without a title of its own, the row carries the agent's name.
        assert_eq!(
            row_of(&rail, &view, cx),
            (
                "Claude Code".to_string(),
                Some(marley_agent::status_line(AgentKind::Claude, agent.status))
            )
        );
        cx.update(|window, _| window.refresh());
        let selector = format!("marley-rail-agent-{}", id(&view)).leak();
        assert!(cx.debug_bounds(selector).is_some(), "the agent's icon");

        // The title the CLI sets becomes the row's.
        terminal.update(cx, |terminal, cx| {
            terminal.breadcrumb_text = "✳ Fix the flaky test".to_string();
            cx.emit(terminal::Event::BreadcrumbsChanged);
        });
        cx.run_until_parked();
        assert_eq!(row_of(&rail, &view, cx).0, "✳ Fix the flaky test");

        // Back at the shell, it is a plain terminal row again.
        set_foreground(&terminal, "zsh", cx);
        assert_eq!(agent_of(&rail, &view, cx), None);
        cx.update(|window, _| window.refresh());
        assert!(cx.debug_bounds(selector).is_none());
    }

    #[gpui::test]
    async fn an_agent_works_while_output_flows_and_waits_when_it_stops(cx: &mut TestAppContext) {
        let (_, alpha, _, rail, cx) = open_rail(cx).await;
        rail.update(cx, |rail, _| rail.foreground_command = fake_foreground);
        let (terminal, view) = add_terminal(&alpha, false, cx);
        set_foreground(&terminal, "codex", cx);
        let status = |cx: &VisualTestContext| agent_of(&rail, &view, cx).map(|agent| agent.status);
        // Nothing written yet: quiet, so waiting.
        assert_eq!(status(cx), Some(AgentStatus::Waiting));

        terminal.update(cx, |_, cx| cx.emit(terminal::Event::Wakeup));
        cx.run_until_parked();
        assert_eq!(status(cx), Some(AgentStatus::Working));

        cx.executor().advance_clock(WAITING_AFTER);
        cx.run_until_parked();
        assert_eq!(status(cx), Some(AgentStatus::Waiting));

        // A terminal event that is not output rereads the row without counting as output.
        view.update(cx, |_, cx| cx.emit(terminal::Event::SelectionsChanged));
        cx.run_until_parked();
        assert_eq!(status(cx), Some(AgentStatus::Waiting));

        // Fresh output works again, and a bell reads as waiting at once.
        terminal.update(cx, |_, cx| cx.emit(terminal::Event::Wakeup));
        cx.run_until_parked();
        assert_eq!(status(cx), Some(AgentStatus::Working));
        terminal.update(cx, |_, cx| cx.emit(terminal::Event::Bell));
        cx.run_until_parked();
        assert_eq!(status(cx), Some(AgentStatus::Waiting));
    }
}
