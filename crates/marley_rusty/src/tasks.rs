//! Rusty's task groups and tasks as its tools answer them (#655).

use serde::Deserialize;

/// Rusty's tool for its task groups.
pub const LIST_TASK_GROUPS: &str = "list_task_groups";

/// Rusty's tool for one group's tasks.
pub const LIST_TASKS: &str = "list_tasks";

/// A task group.
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
