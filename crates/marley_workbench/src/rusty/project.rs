//! The project join's app side (#655): every brain project page, cached; a workspace's project and
//! the page it resolves to; the two links a user makes; and their pickers.
//!
//! The cache is one app-wide global, built when a project view or a Graph tab first asks, read
//! again on Rusty's change signal (now, and once more after Rusty's indexer has run), and written
//! only when what it holds differs. A Rusty that answers `brain_list_pages`' properties needs one
//! call; an older one is read page by page, each page again only when Rusty saw it change.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use fuzzy::{StringMatch, StringMatchCandidate};
use gpui::{
    App, AppContext as _, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    Global, SharedString, Task, WeakEntity, Window, actions,
};
use marley_rusty::project::{
    BRAIN_READ_PAGE, BRAIN_SET_PROPERTY, ListedPage, PAGE_LIMIT, PATH_KEY, PROJECT_TYPE,
    PROPERTIES, Project, ProjectPage, Resolution, TASK_GROUP_KEY, listed_from_answer,
    path_value_with, read_from_answer, resolve,
};
use marley_rusty::switcher::BRAIN_LIST_PAGES;
use marley_rusty::tasks::{LIST_TASK_GROUPS, TaskGroup, groups_from_answer};
use picker::{Picker, PickerDelegate};
use serde_json::{Value, json};
use ui::{ListItem, ListItemSpacing, prelude::*};
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{ModalView, Toast, Workspace};

actions!(
    rusty,
    [
        /// Links this project to a brain project page: picks the page and adds the project's
        /// folders to its path.
        #[derive(Eq)]
        LinkProjectPage,
        /// Links the project's brain page to a Rusty task group: picks the group and writes its
        /// name to the page's task_group.
        #[derive(Eq)]
        LinkTaskGroup,
    ]
);

/// How long after a change signal the list is read once more: Rusty's indexer syncs a disk edit
/// about 5 s after it announces it, so the first list can predate the new page.
const SECOND_LOOK: Duration = Duration::from_secs(6);

/// How many pages an older Rusty is asked for at once.
const READS_AT_ONCE: usize = 8;

/// Every brain project page: `None` until first read.
#[derive(Default)]
pub(crate) struct ProjectPages(Option<Result<Arc<[ProjectPage]>, SharedString>>);

impl Global for ProjectPages {}

/// The cache's reads, which nothing observes: one under way and one queued, the delayed second
/// look, and each page read from an older Rusty with the `updated_at` it was read at.
#[derive(Default)]
struct ProjectReads {
    reading: bool,
    again: bool,
    second_look: Option<Task<()>>,
    read: HashMap<String, (Value, ProjectPage)>,
}

impl Global for ProjectReads {}

/// Sets up the cache and registers the two links on every workspace; `rusty::init` calls it once.
pub(super) fn init(cx: &mut App) {
    cx.set_global(ProjectPages::default());
    cx.set_global(ProjectReads::default());
    cx.observe_global::<super::Announced>(|cx| {
        if cx.global::<ProjectPages>().0.is_some() {
            refresh(cx);
        }
    })
    .detach();
    cx.observe_global::<super::Rusty>(|cx| {
        let failed = matches!(cx.global::<ProjectPages>().0, Some(Err(_)));
        if failed && super::is_connected(cx) {
            read(cx);
        }
    })
    .detach();
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|workspace, _: &LinkProjectPage, window, cx| {
            link_project_page(workspace, window, cx);
        });
        workspace.register_action(|workspace, _: &LinkTaskGroup, window, cx| {
            link_task_group(workspace, window, cx);
        });
    })
    .detach();
}

/// Starts the first read of the project pages, unless one ran or runs.
pub(crate) fn ensure(cx: &mut App) {
    let unread = cx.global::<ProjectPages>().0.is_none();
    if unread && !cx.global::<ProjectReads>().reading && super::is_connected(cx) {
        read(cx);
    }
}

/// Reads the list now and once more after Rusty's indexer has run.
fn refresh(cx: &mut App) {
    read(cx);
    let second_look = cx.spawn(async move |cx| {
        cx.background_executor().timer(SECOND_LOOK).await;
        cx.update(read);
    });
    cx.global_mut::<ProjectReads>().second_look = Some(second_look);
}

/// Lists the project pages, reads those an older Rusty gave without properties, and keeps the
/// outcome, a failure included, so nothing asks again in a loop.
fn read(cx: &mut App) {
    let reads = cx.global_mut::<ProjectReads>();
    if reads.reading {
        reads.again = true;
        return;
    }
    reads.reading = true;
    let listing = super::call_tool(
        BRAIN_LIST_PAGES,
        json!({ "page_type": PROJECT_TYPE, "limit": PAGE_LIMIT, "properties": PROPERTIES }),
        cx,
    );
    cx.spawn(async move |cx| {
        let outcome = gather(listing, cx).await;
        cx.update(|cx| {
            let reads = cx.global_mut::<ProjectReads>();
            reads.reading = false;
            let again = std::mem::take(&mut reads.again);
            let held = match outcome {
                Ok((pages, read)) => {
                    reads.read = read;
                    Ok(Arc::<[ProjectPage]>::from(pages))
                }
                Err(error) => {
                    log::warn!("rusty: the project pages: {error}");
                    Err(SharedString::from(error))
                }
            };
            if cx.global::<ProjectPages>().0.as_ref() != Some(&held) {
                cx.global_mut::<ProjectPages>().0 = Some(held);
            }
            if again {
                read(cx);
            }
        });
    })
    .detach();
}

type Gathered = (Vec<ProjectPage>, HashMap<String, (Value, ProjectPage)>);

async fn gather(
    listing: Task<Result<String, String>>,
    cx: &gpui::AsyncApp,
) -> Result<Gathered, String> {
    let text = listing.await?;
    let listed: Vec<ListedPage> = cx
        .background_spawn(futures::future::lazy(move |_| listed_from_answer(&text)))
        .await
        .map_err(|error| format!("{BRAIN_LIST_PAGES}'s answer did not parse: {error}"))?;
    let held = cx.update(|cx| cx.global::<ProjectReads>().read.clone());
    let mut pages = Vec::new();
    let mut read = HashMap::new();
    let mut unread = Vec::new();
    for page in listed
        .into_iter()
        .filter(|page| !page.slug.starts_with("archive/"))
    {
        if let Some(project_page) = ProjectPage::from_listed(&page) {
            pages.push(project_page);
        } else if let Some((at, project_page)) = held
            .get(&page.slug)
            .filter(|(at, _)| *at == page.updated_at)
        {
            pages.push(project_page.clone());
            read.insert(page.slug.clone(), (at.clone(), project_page.clone()));
        } else {
            unread.push(page);
        }
    }
    for chunk in unread.chunks(READS_AT_ONCE) {
        let asking: Vec<Task<Result<String, String>>> = cx.update(|cx| {
            chunk
                .iter()
                .map(|page| super::call_tool(BRAIN_READ_PAGE, json!({ "slug": page.slug }), cx))
                .collect()
        });
        let answers = futures::future::join_all(asking).await;
        for (page, answer) in chunk.iter().zip(answers) {
            let parsed = answer.and_then(|text| {
                read_from_answer(&text)
                    .map_err(|error| format!("{BRAIN_READ_PAGE}'s answer did not parse: {error}"))
            });
            match parsed {
                Ok(Some(found)) => {
                    let mut project_page = ProjectPage::from_read(&found);
                    if project_page.aliases.is_empty() {
                        project_page.aliases.clone_from(&page.aliases);
                    }
                    read.insert(
                        page.slug.clone(),
                        (page.updated_at.clone(), project_page.clone()),
                    );
                    pages.push(project_page);
                }
                Ok(None) => {}
                Err(error) => log::warn!("rusty: the project page {}: {error}", page.slug),
            }
        }
    }
    Ok((pages, read))
}

/// The workspace's project: its project group's folders, `None` with no folder.
pub(crate) fn project_of(workspace: &Workspace, cx: &App) -> Option<Project> {
    let key = workspace.project_group_key(cx);
    let folders = key.path_list().paths().to_vec();
    (!folders.is_empty()).then(|| Project {
        folders,
        remote: key.host().is_some(),
    })
}

/// What the workspace's project joins to, as far as the cache knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Join {
    /// The workspace has no folder.
    NoProject,
    /// The project pages are not read yet.
    Reading,
    /// Reading them failed.
    Failed(SharedString),
    /// The join's outcome.
    Resolved(Resolution),
}

pub(crate) fn join(workspace: &Workspace, cx: &App) -> Join {
    let Some(project) = project_of(workspace, cx) else {
        return Join::NoProject;
    };
    match cx
        .try_global::<ProjectPages>()
        .and_then(|pages| pages.0.as_ref())
    {
        None => Join::Reading,
        Some(Err(error)) => Join::Failed(error.clone()),
        Some(Ok(pages)) => Join::Resolved(resolve(&project, pages, util::paths::home_dir())),
    }
}

/// The slug of the page the workspace's project resolves to.
pub(crate) fn project_page(workspace: &Workspace, cx: &App) -> Option<String> {
    match join(workspace, cx) {
        Join::Resolved(Resolution::Page { slug, .. }) => Some(slug),
        _ => None,
    }
}

/// The cached project page `slug`.
pub(crate) fn cached(slug: &str, cx: &App) -> Option<ProjectPage> {
    let pages = cx.try_global::<ProjectPages>()?.0.as_ref()?.as_ref().ok()?;
    pages.iter().find(|page| page.slug == slug).cloned()
}

fn toast(workspace: &WeakEntity<Workspace>, message: String, cx: &mut App) {
    workspace
        .update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(NotificationId::unique::<ProjectPages>(), message),
                cx,
            );
        })
        .log_err();
}

/// The toast for a workspace whose update this runs in.
fn show(workspace: &mut Workspace, message: impl Into<String>, cx: &mut Context<Workspace>) {
    workspace.show_toast(
        Toast::new(NotificationId::unique::<ProjectPages>(), message.into()),
        cx,
    );
}

/// Adds the workspace's project folders to page `slug`'s `path:`, keeping what it held.
pub(crate) fn link_page(workspace: &Entity<Workspace>, slug: &str, cx: &mut App) {
    let weak = workspace.downgrade();
    let Some(project) = project_of(workspace.read(cx), cx) else {
        return;
    };
    if project.remote {
        toast(
            &weak,
            "This project's folders are on another machine, so no page's path can list them."
                .to_string(),
            cx,
        );
        return;
    }
    let Some(page) = cached(slug, cx) else {
        toast(
            &weak,
            format!("{slug} is not a project page Rusty listed."),
            cx,
        );
        return;
    };
    match path_value_with(slug, &page.path, &project.folders) {
        Ok(Some(value)) => write(weak, slug.to_string(), PATH_KEY, &value, cx),
        Ok(None) => refresh(cx),
        Err(refusal) => toast(&weak, refusal, cx),
    }
}

/// Writes `key` on `slug` through `brain_set_property`, then reads the pages again; Rusty's
/// refusal goes to a toast.
fn write(
    workspace: WeakEntity<Workspace>,
    slug: String,
    key: &'static str,
    value: &Value,
    cx: &App,
) {
    let writing = super::call_tool(
        BRAIN_SET_PROPERTY,
        json!({ "slug": slug, "key": key, "value": value }),
        cx,
    );
    cx.spawn(async move |cx| match writing.await {
        Ok(_) => cx.update(refresh),
        Err(error) => cx.update(|cx| {
            toast(
                &workspace,
                format!("Could not write {key} on {slug}: {error}"),
                cx,
            );
        }),
    })
    .detach();
}

/// `rusty::LinkProjectPage`: a picker over the project pages; the pick adds the folders.
fn link_project_page(workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        show(workspace, reason.to_string(), cx);
        return;
    }
    let Some(project) = project_of(workspace, cx) else {
        show(workspace, "This window has no project folder to link.", cx);
        return;
    };
    if project.remote {
        show(
            workspace,
            "This project's folders are on another machine, so no page's path can list them.",
            cx,
        );
        return;
    }
    let pages = match cx.global::<ProjectPages>().0.clone() {
        Some(Ok(pages)) => pages,
        Some(Err(error)) => {
            show(workspace, error.to_string(), cx);
            return;
        }
        None => {
            ensure(cx);
            show(
                workspace,
                "Reading the brain's project pages; try again in a moment.",
                cx,
            );
            return;
        }
    };
    let mut pages: Vec<ProjectPage> = pages.iter().cloned().collect();
    pages.sort_by_key(|page| page.title.to_lowercase());
    let choices = pages
        .iter()
        .map(|page| Choice {
            label: page.title.clone().into(),
            detail: Some(page.slug.clone().into()),
        })
        .collect();
    let slugs: Vec<String> = pages.into_iter().map(|page| page.slug).collect();
    let workspace_entity = cx.entity();
    let on_pick: OnPick = Rc::new(move |index, cx| {
        if let Some(slug) = slugs.get(index) {
            link_page(&workspace_entity, slug, cx);
        }
    });
    workspace.toggle_modal(window, cx, |window, cx| {
        LinkPicker::new(
            choices,
            "Link this project to a brain page…",
            on_pick,
            window,
            cx,
        )
    });
}

/// `rusty::LinkTaskGroup`: a picker over Rusty's task groups; the pick writes the group's name
/// to the project page's `task_group`.
fn link_task_group(workspace: &mut Workspace, window: &Window, cx: &mut Context<Workspace>) {
    if let Some(reason) = super::unavailable(cx) {
        show(workspace, reason.to_string(), cx);
        return;
    }
    let Some(slug) = project_page(workspace, cx) else {
        show(workspace, "Link this project to a brain page first.", cx);
        return;
    };
    let reading = super::call_tool(LIST_TASK_GROUPS, json!({}), cx);
    cx.spawn_in(window, async move |workspace, cx| {
        let groups = reading.await.and_then(|text| {
            groups_from_answer(&text)
                .map_err(|error| format!("{LIST_TASK_GROUPS}'s answer did not parse: {error}"))
        });
        workspace
            .update_in(cx, |workspace, window, cx| match groups {
                Ok(groups) => open_group_picker(workspace, slug, groups, window, cx),
                Err(error) => show(workspace, error, cx),
            })
            .log_err();
    })
    .detach();
}

fn open_group_picker(
    workspace: &mut Workspace,
    slug: String,
    groups: Vec<TaskGroup>,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    let choices = groups
        .iter()
        .map(|group| Choice {
            label: group.name.clone().into(),
            detail: None,
        })
        .collect();
    let weak = workspace.weak_handle();
    let on_pick: OnPick = Rc::new(move |index, cx| {
        if let Some(group) = groups.get(index) {
            write(
                weak.clone(),
                slug.clone(),
                TASK_GROUP_KEY,
                &Value::String(group.name.clone()),
                cx,
            );
        }
    });
    workspace.toggle_modal(window, cx, |window, cx| {
        LinkPicker::new(
            choices,
            "Link the project's page to a task group…",
            on_pick,
            window,
            cx,
        )
    });
}

/// What a pick does, given the picked choice's index.
pub(super) type OnPick = Rc<dyn Fn(usize, &mut App)>;

/// What closing the picker does: its modal's dismissal, or the follow-up form's way back (#660).
pub(super) type OnClose = Rc<dyn Fn(&mut App)>;

/// One choice of a link picker: its words, and a muted detail.
pub(super) struct Choice {
    pub(super) label: SharedString,
    pub(super) detail: Option<SharedString>,
}

/// A picker over choices whose pick writes a link, in the workspace's modal layer.
struct LinkPicker {
    picker: Entity<Picker<LinkDelegate>>,
}

impl LinkPicker {
    fn new(
        choices: Vec<Choice>,
        placeholder: &'static str,
        on_pick: OnPick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let modal = cx.entity().downgrade();
        let on_close: OnClose = Rc::new(move |cx| {
            modal.update(cx, |_, cx| cx.emit(DismissEvent)).log_err();
        });
        let delegate = LinkDelegate::new(choices, placeholder, on_pick, on_close, true);
        Self {
            picker: cx.new(|cx| Picker::uniform_list(delegate, window, cx)),
        }
    }
}

impl ModalView for LinkPicker {}

impl EventEmitter<DismissEvent> for LinkPicker {}

impl Focusable for LinkPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.picker.focus_handle(cx)
    }
}

impl Render for LinkPicker {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context("RustyLinkPicker")
            .w(rems(34.))
            .child(self.picker.clone())
    }
}

/// A picker's choices, matched by their words; shared by the link pickers and the follow-up
/// form's successor (#660).
pub(super) struct LinkDelegate {
    choices: Vec<Choice>,
    matches: Vec<StringMatch>,
    selected: usize,
    placeholder: &'static str,
    on_pick: OnPick,
    on_close: OnClose,
    /// Whether a pick also closes the picker; the follow-up form closes its own.
    closes_on_pick: bool,
}

impl LinkDelegate {
    pub(super) fn new(
        choices: Vec<Choice>,
        placeholder: &'static str,
        on_pick: OnPick,
        on_close: OnClose,
        closes_on_pick: bool,
    ) -> Self {
        Self {
            choices,
            matches: Vec::new(),
            selected: 0,
            placeholder,
            on_pick,
            on_close,
            closes_on_pick,
        }
    }
}

impl PickerDelegate for LinkDelegate {
    type ListItem = ListItem;

    fn name() -> &'static str {
        "rusty link picker"
    }

    fn match_count(&self) -> usize {
        self.matches.len()
    }

    fn selected_index(&self) -> usize {
        self.selected
    }

    fn set_selected_index(&mut self, index: usize, _: &mut Window, _: &mut Context<Picker<Self>>) {
        self.selected = index;
    }

    fn placeholder_text(&self, _: &mut Window, _: &mut App) -> Arc<str> {
        self.placeholder.into()
    }

    fn no_matches_text(&self, _: &mut Window, _: &mut App) -> Option<SharedString> {
        Some(SharedString::new_static("Nothing matches"))
    }

    fn update_matches(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Picker<Self>>,
    ) -> Task<()> {
        let candidates: Vec<StringMatchCandidate> = self
            .choices
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                let text = choice.detail.as_ref().map_or_else(
                    || choice.label.to_string(),
                    |detail| format!("{} {detail}", choice.label),
                );
                StringMatchCandidate::new(index, &text)
            })
            .collect();
        let executor = cx.background_executor().clone();
        cx.spawn_in(window, async move |picker, cx| {
            // An empty query keeps every choice in its own order.
            let matches = if query.is_empty() {
                candidates
                    .into_iter()
                    .map(|candidate| StringMatch {
                        candidate_id: candidate.id,
                        score: 0.,
                        positions: Vec::new(),
                        string: candidate.string,
                    })
                    .collect()
            } else {
                let cancelled = AtomicBool::new(false);
                fuzzy::match_strings(&candidates, &query, false, true, 100, &cancelled, executor)
                    .await
            };
            picker
                .update(cx, |picker, cx| {
                    picker.delegate.matches = matches;
                    picker.delegate.selected = 0;
                    cx.notify();
                })
                .log_err();
        })
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<Picker<Self>>) {
        let picked = self
            .matches
            .get(self.selected)
            .map(|found| found.candidate_id);
        if self.closes_on_pick {
            self.dismissed(window, cx);
        }
        if let Some(index) = picked {
            let on_pick = Rc::clone(&self.on_pick);
            // The pick reads the workspace, which may be the update this one runs in.
            cx.defer(move |cx| on_pick(index, cx));
        }
    }

    fn dismissed(&mut self, _: &mut Window, cx: &mut Context<Picker<Self>>) {
        (self.on_close)(cx);
    }

    fn render_match(
        &self,
        index: usize,
        selected: bool,
        _: &mut Window,
        _: &mut Context<Picker<Self>>,
    ) -> Option<Self::ListItem> {
        let found = self.matches.get(index)?;
        let choice = self.choices.get(found.candidate_id)?;
        Some(
            ListItem::new(index)
                .inset(true)
                .spacing(ListItemSpacing::Sparse)
                .toggle_state(selected)
                .child(
                    h_flex()
                        .gap_2()
                        .child(Label::new(choice.label.clone()))
                        .children(choice.detail.clone().map(|detail| {
                            Label::new(detail)
                                .size(LabelSize::Small)
                                .color(Color::Muted)
                        })),
                ),
        )
    }
}
