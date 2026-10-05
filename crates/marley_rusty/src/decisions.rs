//! Rusty's decisions as `brain_due` answers them (#655): the follow-ups due.

use serde::Deserialize;

/// Rusty's tool for the decisions whose follow-up is due.
pub const BRAIN_DUE: &str = "brain_due";

/// One decision as Rusty sums it up.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DecisionSummary {
    /// The decision page's slug.
    pub slug: String,
    /// Its title.
    #[serde(default)]
    pub title: String,
    /// The day its follow-up is due, `YYYY-MM-DD`.
    #[serde(default)]
    pub follow_up_by: Option<String>,
    /// Whether that day has passed.
    #[serde(default)]
    pub overdue: bool,
}

/// `brain_due`'s answer: the follow-ups due within its horizon.
#[derive(Debug, Default, Deserialize)]
struct Due {
    #[serde(default)]
    due: Vec<DecisionSummary>,
}

/// The follow-ups due, in Rusty's order, from `brain_due`'s answer.
///
/// # Errors
///
/// When the answer is not `brain_due`'s.
pub fn due_from_answer(text: &str) -> Result<Vec<DecisionSummary>, serde_json::Error> {
    serde_json::from_str::<Due>(text).map(|due| due.due)
}
