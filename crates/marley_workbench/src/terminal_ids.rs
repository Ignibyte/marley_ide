//! Each terminal's `MARLEY_TERMINAL_ID` kept across a restore (#575).
//!
//! Zed restores a terminal at the next launch with a new shell, which #520's builder would give a
//! new id. [`init`] sets `terminal_view`'s `MarleyTerminalIdentity` hook over a table of Marley's
//! own, whose rows follow Zed's `terminals` rows: a terminal's id is saved when Zed saves the
//! terminal, read when Zed restores it, moved with it to a new workspace id, and deleted with the
//! items Zed did not load. The restore hands the id to the builder, which gives it to the new
//! shell.
//!
//! The restore reads the ids from memory, which holds the table as it was at the start and each
//! id saved since: the terminal panel cleans up the terminal rows with the panel's items alone,
//! and when that runs between two terminals' restores it deletes a row the second one still
//! needs.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use gpui::{App, AppContext as _, Global, Task};
use terminal_view::MarleyTerminalIdentity;
use util::ResultExt as _;
use workspace::{ItemId, WorkspaceId};

use persistence::MarleyTerminalIdsDb;

/// Each terminal's id by workspace and item: the table's rows at the start, and each one saved
/// since.
#[derive(Default)]
struct KnownIds(HashMap<(WorkspaceId, ItemId), String>);

impl Global for KnownIds {}

/// Reads the table and sets the hook; [`crate::init`] calls it once, before any window restores
/// its terminals.
pub fn init(cx: &mut App) {
    let known = MarleyTerminalIdsDb::global(cx)
        .all_ids()
        .log_err()
        .unwrap_or_default()
        .into_iter()
        .map(|(workspace_id, item_id, terminal_id)| ((workspace_id, item_id), terminal_id))
        .collect();
    cx.set_global(KnownIds(known));
    cx.set_global(MarleyTerminalIdentity {
        saved: Arc::new(saved),
        save: Arc::new(save),
        moved: Arc::new(moved),
        cleanup: Arc::new(cleanup),
    });
}

/// The ids of the terminals saved to be restored: the table's at the start, and each saved since
/// (#540).
pub(crate) fn known_ids(cx: &App) -> HashSet<String> {
    cx.try_global::<KnownIds>()
        .map(|known| known.0.values().cloned().collect())
        .unwrap_or_default()
}

/// The id kept for the item, else the one in the table, when it is well formed.
fn saved(workspace_id: WorkspaceId, item_id: ItemId, cx: &App) -> Option<String> {
    cx.try_global::<KnownIds>()
        .and_then(|known| known.0.get(&(workspace_id, item_id)).cloned())
        .or_else(|| {
            MarleyTerminalIdsDb::global(cx)
                .saved_id(item_id, workspace_id)
                .log_err()
                .flatten()
        })
        .filter(|id| marley_terminal::identity::is_terminal_id(id))
}

fn save(
    workspace_id: WorkspaceId,
    item_id: ItemId,
    terminal_id: String,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    cx.default_global::<KnownIds>()
        .0
        .insert((workspace_id, item_id), terminal_id.clone());
    let db = MarleyTerminalIdsDb::global(cx);
    cx.background_spawn(async move { db.save_id(item_id, workspace_id, terminal_id).await })
}

fn moved(
    new_id: WorkspaceId,
    old_id: WorkspaceId,
    item_id: ItemId,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    let known = &mut cx.default_global::<KnownIds>().0;
    if let Some(terminal_id) = known.remove(&(old_id, item_id)) {
        known.insert((new_id, item_id), terminal_id);
    }
    let db = MarleyTerminalIdsDb::global(cx);
    cx.background_spawn(async move { db.move_id(new_id, old_id, item_id).await })
}

fn cleanup(
    workspace_id: WorkspaceId,
    alive_items: Vec<ItemId>,
    cx: &mut App,
) -> Task<anyhow::Result<()>> {
    let db = MarleyTerminalIdsDb::global(cx);
    workspace::delete_unloaded_items(alive_items, workspace_id, "marley_terminal_ids", &db, cx)
}

mod persistence {
    use db::query;
    use db::sqlez::domain::Domain;
    use db::sqlez::thread_safe_connection::ThreadSafeConnection;
    use db::sqlez_macros::sql;
    use workspace::{ItemId, WorkspaceDb, WorkspaceId};

    /// Each terminal item's id, by workspace and item. An item id is no key alone: ids repeat
    /// across launches, which is why Zed's own `terminals` table keys its rows by the pair too.
    pub(super) struct MarleyTerminalIdsDb(ThreadSafeConnection);

    impl Domain for MarleyTerminalIdsDb {
        const NAME: &str = stringify!(MarleyTerminalIdsDb);

        const MIGRATIONS: &[&str] = &[sql!(
            CREATE TABLE marley_terminal_ids (
                workspace_id INTEGER,
                item_id INTEGER,
                terminal_id TEXT NOT NULL,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
        )];
    }

    db::static_connection!(MarleyTerminalIdsDb, [WorkspaceDb]);

    impl MarleyTerminalIdsDb {
        query! {
            pub(super) async fn save_id(
                item_id: ItemId,
                workspace_id: WorkspaceId,
                terminal_id: String
            ) -> Result<()> {
                INSERT OR REPLACE INTO marley_terminal_ids(item_id, workspace_id, terminal_id)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub(super) fn saved_id(
                item_id: ItemId,
                workspace_id: WorkspaceId
            ) -> Result<Option<String>> {
                SELECT terminal_id
                FROM marley_terminal_ids
                WHERE item_id = ? AND workspace_id = ?
            }
        }

        query! {
            pub(super) fn all_ids() -> Result<Vec<(WorkspaceId, ItemId, String)>> {
                SELECT workspace_id, item_id, terminal_id
                FROM marley_terminal_ids
            }
        }

        query! {
            pub(super) async fn move_id(
                new_id: WorkspaceId,
                old_id: WorkspaceId,
                item_id: ItemId
            ) -> Result<()> {
                UPDATE marley_terminal_ids
                SET workspace_id = ?
                WHERE workspace_id = ? AND item_id = ?
            }
        }
    }
}
