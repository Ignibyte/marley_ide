//! Rusty's task groups and tasks as its tools answer them (#655), and the writes the Tasks tab
//! makes through Rusty's ten task tools (#658).
//!
//! Rusty calls a list a task group and serves both in their `sort_order`, so every answer comes in
//! the order the tab shows. A write is one [`TaskWrite`], in Rusty's parameter names: a task is
//! `id`, a list `group_id`.

use serde::Deserialize;
use serde_json::{Value, json};

/// Rusty's tool for its task groups.
pub const LIST_TASK_GROUPS: &str = "list_task_groups";

/// Rusty's tool for one group's tasks.
pub const LIST_TASKS: &str = "list_tasks";

/// Makes a list.
pub const CREATE_TASK_GROUP: &str = "create_task_group";
/// Renames a list.
pub const RENAME_TASK_GROUP: &str = "rename_task_group";
/// Deletes a list and its tasks.
pub const DELETE_TASK_GROUP: &str = "delete_task_group";
/// Adds a task at the end of a list.
pub const CREATE_TASK: &str = "create_task";
/// Checks a task done, or open again.
pub const TOGGLE_TASK: &str = "toggle_task";
/// Changes a task's title.
pub const UPDATE_TASK_TITLE: &str = "update_task_title";
/// Archives a task.
pub const ARCHIVE_TASK: &str = "archive_task";
/// Brings an archived task back.
pub const UNARCHIVE_TASK: &str = "unarchive_task";
/// Deletes a task for good.
pub const DELETE_TASK: &str = "delete_task";
/// Puts a list's tasks in a new order.
pub const REORDER_TASKS: &str = "reorder_tasks";

/// A task group: one of Rusty's lists.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TaskGroup {
    /// Rusty's id for it.
    pub id: i64,
    /// Its name.
    pub name: String,
}

/// A task.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UserTask {
    /// Rusty's id for it.
    pub id: i64,
    /// What it says.
    pub title: String,
    /// Whether it is done.
    #[serde(default)]
    pub completed: bool,
    /// Whether it is archived: hidden unless asked for, and kept.
    #[serde(default)]
    pub archived: bool,
    /// The group it is in (#658).
    #[serde(default)]
    pub header_id: i64,
}

/// `list_task_groups`' answer.
///
/// # Errors
///
/// When the answer is not a list of groups.
pub fn groups_from_answer(text: &str) -> Result<Vec<TaskGroup>, serde_json::Error> {
    serde_json::from_str(text)
}

/// `list_tasks`' answer.
///
/// # Errors
///
/// When the answer is not a list of tasks.
pub fn tasks_from_answer(text: &str) -> Result<Vec<UserTask>, serde_json::Error> {
    serde_json::from_str(text)
}

/// The id `create_task_group` or `create_task` answers.
///
/// # Errors
///
/// When the answer is not a number.
pub fn id_from_answer(text: &str) -> Result<i64, serde_json::Error> {
    serde_json::from_str(text)
}

/// One change to Rusty's lists or tasks: what the Tasks tab queues and sends, one at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskWrite {
    /// A new list.
    NewList {
        /// Its name.
        name: String,
    },
    /// A list renamed.
    RenameList {
        /// The list.
        id: i64,
        /// Its new name.
        name: String,
    },
    /// A list deleted, its tasks with it.
    DeleteList {
        /// The list.
        id: i64,
    },
    /// A task added at the end of a list.
    NewTask {
        /// The list.
        list: i64,
        /// What it says.
        title: String,
    },
    /// A task checked done or open again.
    Toggle {
        /// The task.
        id: i64,
    },
    /// A task's title changed.
    Retitle {
        /// The task.
        id: i64,
        /// Its new title.
        title: String,
    },
    /// A task archived.
    Archive {
        /// The task.
        id: i64,
    },
    /// An archived task brought back.
    Restore {
        /// The task.
        id: i64,
    },
    /// A task deleted for good.
    Delete {
        /// The task.
        id: i64,
    },
    /// A list's tasks in a new order, those named first.
    Reorder {
        /// The list.
        list: i64,
        /// The tasks, in their new order.
        ids: Vec<i64>,
    },
}

impl TaskWrite {
    /// The Rusty tool that makes the change.
    #[must_use]
    pub const fn tool(&self) -> &'static str {
        match self {
            Self::NewList { .. } => CREATE_TASK_GROUP,
            Self::RenameList { .. } => RENAME_TASK_GROUP,
            Self::DeleteList { .. } => DELETE_TASK_GROUP,
            Self::NewTask { .. } => CREATE_TASK,
            Self::Toggle { .. } => TOGGLE_TASK,
            Self::Retitle { .. } => UPDATE_TASK_TITLE,
            Self::Archive { .. } => ARCHIVE_TASK,
            Self::Restore { .. } => UNARCHIVE_TASK,
            Self::Delete { .. } => DELETE_TASK,
            Self::Reorder { .. } => REORDER_TASKS,
        }
    }

    /// The tool's arguments, in Rusty's parameter names.
    #[must_use]
    pub fn arguments(&self) -> Value {
        match self {
            Self::NewList { name } => json!({ "name": name }),
            Self::RenameList { id, name } => json!({ "group_id": id, "name": name }),
            Self::DeleteList { id } => json!({ "group_id": id }),
            Self::NewTask { list, title } => json!({ "group_id": list, "title": title }),
            Self::Retitle { id, title } => json!({ "id": id, "title": title }),
            Self::Toggle { id }
            | Self::Archive { id }
            | Self::Restore { id }
            | Self::Delete { id } => {
                json!({ "id": id })
            }
            Self::Reorder { list, ids } => json!({ "group_id": list, "task_ids": ids }),
        }
    }
}

/// A name or a title as typed, trimmed; `None` when it is empty or the same as `current`, so
/// nothing is sent. Rusty stores any string it is given, so the rule is Marley's, as it is
/// Rusty's page's.
#[must_use]
pub fn typed_name(typed: &str, current: Option<&str>) -> Option<String> {
    let trimmed = typed.trim();
    (!trimmed.is_empty() && current != Some(trimmed)).then(|| trimmed.to_string())
}

/// The order `ids` with `id` one place up or down; `None` at either end or for an id not there,
/// so nothing is sent.
#[must_use]
pub fn moved_one(ids: &[i64], id: i64, up: bool) -> Option<Vec<i64>> {
    let at = ids.iter().position(|held| *held == id)?;
    let to = if up {
        at.checked_sub(1)?
    } else {
        Some(at + 1).filter(|to| *to < ids.len())?
    };
    let mut moved = ids.to_vec();
    moved.swap(at, to);
    Some(moved)
}

/// The list chosen after a read: the one chosen before when it is still there, else the first.
#[must_use]
pub fn kept_list(lists: &[TaskGroup], chosen: Option<i64>) -> Option<i64> {
    chosen
        .filter(|chosen| lists.iter().any(|list| list.id == *chosen))
        .or_else(|| lists.first().map(|list| list.id))
}

/// The row selected after a read: the task selected before by its id, else the same place, else
/// the last row (Rusty's page's rule); none for an empty list or no selection.
#[must_use]
pub fn kept_task(tasks: &[UserTask], selected: Option<i64>, at: Option<usize>) -> Option<usize> {
    let last = tasks.len().checked_sub(1)?;
    selected
        .and_then(|id| tasks.iter().position(|task| task.id == id))
        .or_else(|| at.map(|at| at.min(last)))
}
