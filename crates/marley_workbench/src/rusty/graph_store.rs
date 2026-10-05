//! What the Graph tab keeps across a restart (#657), in two stores, as Rusty splits them.
//!
//! The graph settings (`marley_rusty::graph_settings`) are one record for every window in Zed's
//! key-value store, as Rusty keeps one `graph` entry for both its graph tabs: read once at
//! [`init`], before any window restores, and written 300 ms after the last change and again at
//! quit, so a slider's drag writes once. Every Graph tab observes [`GraphSettingsStore`], so a
//! change in one shows in all.
//!
//! Each tab's own state (its scope, its page, its filter, the hidden types, the panel and its
//! sections) is its row in `marley_rusty_graph_tabs`, keyed by workspace and item: an item id
//! repeats across launches (F-576). The rows are read into memory at [`init`] and each save
//! updates that copy, so a restore reads memory and never a row a cleanup already took (#575).

use std::collections::HashMap;
use std::time::Duration;

use db::kvp::KeyValueStore;
use gpui::{App, AppContext as _, Global, Task};
use marley_rusty::graph_settings::GraphSettings;
use serde::{Deserialize, Serialize};
use util::ResultExt as _;
use workspace::{ItemId, WorkspaceId};

use persistence::MarleyRustyGraphTabsDb;

/// The key-value store's scope and key for the record.
const SCOPE: &str = "marley-rusty-graph";
const KEY: &str = "settings";

/// How long the record waits after a change before it is written.
const WRITE_AFTER: Duration = Duration::from_millis(300);

/// The graph settings every Graph tab draws with, and the write waiting to keep them.
pub(super) struct GraphSettingsStore {
    settings: GraphSettings,
    _write: Option<Task<()>>,
}

impl Global for GraphSettingsStore {}

/// Which of the panel's sections are open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Sections {
    pub(super) groups: bool,
    pub(super) display: bool,
    pub(super) forces: bool,
}

/// A Graph tab's own state, kept with its workspace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct SavedGraphTab {
    /// A local graph, rather than the vault's.
    pub(super) local: bool,
    /// The page last in front, the local graph's centre; none while the project's page is.
    pub(super) page: Option<String>,
    /// The filter field's text.
    pub(super) filter: String,
    /// The page types turned off in the legend.
    pub(super) hidden_types: Vec<String>,
    /// Whether the panel shows.
    pub(super) panel_open: bool,
    /// The panel's sections open.
    pub(super) sections: Sections,
}

impl Default for SavedGraphTab {
    fn default() -> Self {
        Self {
            local: false,
            page: None,
            filter: String::new(),
            hidden_types: Vec::new(),
            panel_open: true,
            sections: Sections::default(),
        }
    }
}

/// Each saved Graph tab by workspace and item: the table's rows at the start, and each saved
/// since.
#[derive(Default)]
struct SavedGraphTabs(HashMap<(WorkspaceId, ItemId), SavedGraphTab>);

impl Global for SavedGraphTabs {}

/// Reads the record and the tabs' rows, and writes the record at quit; `graph_tab::init` calls it
/// once, before any window restores.
pub(super) fn init(cx: &mut App) {
    let store = KeyValueStore::global(cx);
    let settings = store
        .scoped(SCOPE)
        .read(KEY)
        .log_err()
        .flatten()
        .map_or_else(GraphSettings::default, |text| {
            let (settings, fell_back) = GraphSettings::from_stored(&text);
            for reason in fell_back {
                log::warn!("graph settings: {reason}; Rusty's default is used");
            }
            settings
        });
    cx.set_global(GraphSettingsStore {
        settings,
        _write: None,
    });
    let saved = MarleyRustyGraphTabsDb::global(cx)
        .all_tabs()
        .log_err()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(workspace_id, item_id, state)| {
            let tab = serde_json::from_str::<SavedGraphTab>(&state)
                .map_err(|error| anyhow::anyhow!("a saved Graph tab did not parse: {error}"))
                .log_err()?;
            Some(((workspace_id, item_id), tab))
        })
        .collect();
    cx.set_global(SavedGraphTabs(saved));
    cx.on_app_quit(move |cx| {
        let text = cx.global::<GraphSettingsStore>().settings.to_stored();
        let store = store.clone();
        async move {
            store
                .scoped(SCOPE)
                .write(KEY.to_string(), text)
                .await
                .log_err();
        }
    })
    .detach();
}

/// The graph settings in force.
pub(super) fn settings(cx: &App) -> &GraphSettings {
    &cx.global::<GraphSettingsStore>().settings
}

/// Changes the graph settings for every Graph tab, and writes them once they rest. A change that
/// leaves them as they were tells no tab.
pub(super) fn update(cx: &mut App, change: impl FnOnce(&mut GraphSettings)) {
    let mut settings = settings(cx).clone();
    change(&mut settings);
    if settings == *self::settings(cx) {
        return;
    }
    let text = settings.to_stored();
    let store = KeyValueStore::global(cx);
    let write = cx.spawn(async move |cx| {
        cx.background_executor().timer(WRITE_AFTER).await;
        store
            .scoped(SCOPE)
            .write(KEY.to_string(), text)
            .await
            .log_err();
    });
    cx.set_global(GraphSettingsStore {
        settings,
        _write: Some(write),
    });
}

/// The tab saved for the item, from memory.
pub(super) fn saved_tab(
    workspace_id: WorkspaceId,
    item_id: ItemId,
    cx: &App,
) -> Option<SavedGraphTab> {
    cx.try_global::<SavedGraphTabs>()?
        .0
        .get(&(workspace_id, item_id))
        .cloned()
}

/// Keeps the tab's state for the item, in memory and in the table.
pub(super) fn save_tab(
    workspace_id: WorkspaceId,
    item_id: ItemId,
    tab: SavedGraphTab,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    let state = match serde_json::to_string(&tab) {
        Ok(state) => state,
        Err(error) => return Task::ready(Err(error.into())),
    };
    let _previous = cx
        .default_global::<SavedGraphTabs>()
        .0
        .insert((workspace_id, item_id), tab);
    let db = MarleyRustyGraphTabsDb::global(cx);
    cx.background_spawn(async move { db.save_tab(item_id, workspace_id, state).await })
}

/// Drops the workspace's rows for the items it did not load, in memory and in the table.
pub(super) fn cleanup(
    workspace_id: WorkspaceId,
    alive_items: Vec<ItemId>,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    cx.default_global::<SavedGraphTabs>()
        .0
        .retain(|(workspace, item), _| *workspace != workspace_id || alive_items.contains(item));
    workspace::delete_unloaded_items(
        alive_items,
        workspace_id,
        "marley_rusty_graph_tabs",
        &MarleyRustyGraphTabsDb::global(cx),
        cx,
    )
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;
    use workspace::{ItemId, WorkspaceDb, WorkspaceId};

    /// The Graph tabs' own table: each tab's state as a JSON object, so a later field needs no
    /// migration. Keyed by workspace and item, with no `UNIQUE(item_id)`: ids repeat across
    /// launches (#576).
    pub(super) struct MarleyRustyGraphTabsDb(ThreadSafeConnection);

    impl Domain for MarleyRustyGraphTabsDb {
        const NAME: &str = stringify!(MarleyRustyGraphTabsDb);

        const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE marley_rusty_graph_tabs (
                workspace_id INTEGER,
                item_id INTEGER,
                state TEXT NOT NULL,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
        )];
    }

    db::static_connection!(MarleyRustyGraphTabsDb, [WorkspaceDb]);

    impl MarleyRustyGraphTabsDb {
        query! {
            pub(super) async fn save_tab(
                item_id: ItemId,
                workspace_id: WorkspaceId,
                state: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_rusty_graph_tabs(item_id, workspace_id, state)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub(super) fn all_tabs() -> Result<Vec<(WorkspaceId, ItemId, String)>> {
                SELECT workspace_id, item_id, state
                FROM marley_rusty_graph_tabs
            }
        }
    }
}
