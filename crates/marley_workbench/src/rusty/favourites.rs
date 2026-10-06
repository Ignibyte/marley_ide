//! Rusty's bookmarks in Marley (#662).
//!
//! The list as Rusty last answered it: read when the Brain view or a Page tab first needs it,
//! again on Rusty's announcement, when the connection comes up and after each write, and dropped
//! when Rusty turns off. The Brain view draws it as Favourites above its tree, a Page tab's star
//! and `rusty: toggle bookmark` add and remove a page, and the page picker lists favourite pages
//! first. Every change is one of Rusty's bookmark tools, whose answer is the whole list.

use std::sync::Arc;

use gpui::{App, Global, WeakEntity};
use marley_rusty::bookmarks::{
    BOOKMARK_LIST, Bookmark, BookmarkWrite, bookmarks_from_answer, retitled,
};
use serde_json::json;
use util::ResultExt as _;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};

/// Rusty's bookmarks in its order: none until first read, and while Rusty is off.
#[derive(Default)]
pub(super) struct Bookmarks(Option<Arc<[Bookmark]>>);

impl Global for Bookmarks {}

/// Where the list's read stands.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Reading {
    #[default]
    Idle,
    Running,
    /// One more read follows the one running.
    RunningAgain,
}

/// The list's reads, which nothing observes.
#[derive(Default)]
struct BookmarkReads {
    /// Whether a view has asked for the list: until then nothing reads it.
    wanted: bool,
    reading: Reading,
    /// Whether Marley was connected to Rusty when last looked at, so a read follows the
    /// connection coming up and not every notification of the `Rusty` global (L-658).
    connected: bool,
}

impl Global for BookmarkReads {}

/// Sets up the list and follows Rusty; `rusty::init` calls it once.
pub(super) fn init(cx: &mut App) {
    cx.set_global(Bookmarks::default());
    cx.set_global(BookmarkReads::default());
    cx.observe_global::<super::Announced>(|cx| {
        if cx.global::<BookmarkReads>().wanted {
            read(cx);
        }
    })
    .detach();
    cx.observe_global::<super::Rusty>(|cx| {
        if !super::is_on(cx) && cx.global::<Bookmarks>().0.is_some() {
            cx.set_global(Bookmarks::default());
        }
        let connected = super::is_connected(cx);
        let reads = cx.global_mut::<BookmarkReads>();
        let came_up = connected && !reads.connected;
        reads.connected = connected;
        if came_up && reads.wanted {
            read(cx);
        }
    })
    .detach();
}

/// Starts the first read of the list, unless one ran or runs; a view that shows bookmarks calls
/// it when it is made.
pub(super) fn ensure(cx: &mut App) {
    cx.global_mut::<BookmarkReads>().wanted = true;
    let unread = cx.global::<Bookmarks>().0.is_none();
    if unread && cx.global::<BookmarkReads>().reading == Reading::Idle && super::is_connected(cx) {
        read(cx);
    }
}

/// The bookmarks in Rusty's order; none while unread or Rusty is off.
pub(super) fn list(cx: &App) -> Arc<[Bookmark]> {
    cx.try_global::<Bookmarks>()
        .and_then(|bookmarks| bookmarks.0.clone())
        .unwrap_or_default()
}

/// Whether the page `slug` is a bookmark.
pub(super) fn is_page(slug: &str, cx: &App) -> bool {
    list(cx).iter().any(|bookmark| bookmark.is_page(slug))
}

/// Reads the list; a second read asked for while one runs follows it.
pub(super) fn read(cx: &mut App) {
    let reads = cx.global_mut::<BookmarkReads>();
    if reads.reading != Reading::Idle {
        reads.reading = Reading::RunningAgain;
        return;
    }
    reads.reading = Reading::Running;
    let asking = super::call_tool(BOOKMARK_LIST, json!({}), cx);
    cx.spawn(async move |cx| {
        let answer = asking.await;
        cx.update(|cx| {
            let reads = cx.global_mut::<BookmarkReads>();
            let again = reads.reading == Reading::RunningAgain;
            reads.reading = Reading::Idle;
            match answer.and_then(|text| parsed(&text)) {
                Ok(list) => keep(list, cx),
                Err(error) => log::warn!("rusty: the bookmarks: {error}"),
            }
            if again {
                read(cx);
            }
        });
    })
    .detach();
}

fn parsed(text: &str) -> Result<Vec<Bookmark>, String> {
    bookmarks_from_answer(text)
        .map_err(|error| format!("the bookmarks' answer did not parse: {error}"))
}

/// Keeps `list` when it differs, so the views that observe the list hear of a change only.
fn keep(list: Vec<Bookmark>, cx: &mut App) {
    if !super::is_on(cx) {
        return;
    }
    let list: Arc<[Bookmark]> = list.into();
    if cx.global::<Bookmarks>().0.as_ref() != Some(&list) {
        cx.set_global(Bookmarks(Some(list)));
    }
}

/// Sends `change` to Rusty and keeps the list it answers; a refusal says so in `workspace`'s
/// toast and reads the list again.
pub(super) fn write(change: &BookmarkWrite, workspace: WeakEntity<Workspace>, cx: &App) {
    let asking = super::call_tool(change.tool(), change.arguments(), cx);
    cx.spawn(async move |cx| {
        let answer = asking.await;
        cx.update(|cx| match answer.and_then(|text| parsed(&text)) {
            Ok(list) => keep(list, cx),
            Err(error) => {
                let line = error.lines().next().unwrap_or_default().to_string();
                workspace
                    .update(cx, |workspace, cx| {
                        workspace.show_toast(
                            Toast::new(NotificationId::unique::<Bookmarks>(), line),
                            cx,
                        );
                    })
                    .log_err();
                read(cx);
            }
        });
    })
    .detach();
}

/// Adds the page `slug` as a bookmark, or removes it when it is one: the star's and
/// `rusty: toggle bookmark`'s change.
pub(super) fn toggle_page(slug: &str, workspace: WeakEntity<Workspace>, cx: &App) {
    let change = list(cx)
        .iter()
        .find(|bookmark| bookmark.is_page(slug))
        .map_or_else(
            || BookmarkWrite::Add(Bookmark::page(slug)),
            |bookmark| BookmarkWrite::Remove(bookmark.clone()),
        );
    write(&change, workspace, cx);
}

/// Sends the list with the bookmark whose key is `key` titled `title`, in Rusty's order.
pub(super) fn retitle(key: &str, title: &str, workspace: WeakEntity<Workspace>, cx: &App) {
    let list = list(cx);
    write(
        &BookmarkWrite::Set(retitled(&list, key, title)),
        workspace,
        cx,
    );
}
