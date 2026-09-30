//! The Marley guide (#599): one HTML page Marley carries, written under the data folder when it
//! is opened, and shown in a Browser tab of the project on screen or in the system browser.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use gpui::{App, AppContext as _, Context, TaskExt as _, Window};
use workspace::Workspace;

use crate::{OpenGuide, browser};

/// The page, as the crate carries it.
const PAGE: &str = include_str!("../guide/index.html");

/// Registers `marley: open guide` on every workspace; the title bar's `?` dispatches it by name.
pub(crate) fn init(cx: &App) {
    cx.observe_new(|workspace: &mut Workspace, _, _: &mut Context<Workspace>| {
        workspace.register_action(|_, _: &OpenGuide, window, cx| {
            open(window, cx);
        });
    })
    .detach();
}

/// Writes the page to `guide/index.html` under `data_dir`, unless the file there holds it
/// already, so an updated Marley shows its own guide; returns the file's path.
pub(crate) fn write_page_in(data_dir: &Path) -> anyhow::Result<PathBuf> {
    let dir = data_dir.join("guide");
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let path = dir.join("index.html");
    if std::fs::read(&path).ok().as_deref() != Some(PAGE.as_bytes()) {
        std::fs::write(&path, PAGE).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(path)
}

/// Opens the guide in a Browser tab of the workspace's project when it has a local folder, and
/// in the system browser otherwise: a Browser tab's Chromium belongs to a project, and a window
/// with no folder would start one keyed by no folder at all.
fn open(window: &Window, cx: &Context<Workspace>) {
    let data_dir = paths::data_dir().clone();
    let written = cx.background_spawn(futures::future::lazy(move |_| write_page_in(&data_dir)));
    cx.spawn_in(window, async move |workspace, cx| {
        let page = written.await.and_then(|path| {
            url::Url::from_file_path(&path)
                .map_err(|()| anyhow::anyhow!("{} is not an absolute path", path.display()))
        });
        workspace.update_in(cx, |workspace, window, cx| match page {
            Ok(url) => {
                let project = workspace.project().read(cx);
                let in_tab = project.is_local() && project.visible_worktrees(cx).next().is_some();
                if in_tab {
                    // The guide's own contents links add a fragment to the tab's address, which
                    // still shows the guide.
                    let on_guide = |address: &url::Url| {
                        let mut address = address.clone();
                        address.set_fragment(None);
                        address == url
                    };
                    if browser::show_tab_where(workspace, on_guide, window, cx).is_none() {
                        browser::open_url_tab(workspace, url.to_string(), window, cx);
                    }
                } else {
                    cx.open_url(url.as_str());
                }
            }
            Err(error) => workspace.show_error(error, cx),
        })
    })
    .detach_and_log_err(cx);
}
