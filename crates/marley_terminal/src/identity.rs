//! A terminal's identity, as its programs see it (#520).
//!
//! Each local interactive terminal starts with its own [`TERMINAL_ID_VARIABLE`], a UUID Zed's
//! terminal builder mints, and [`PROJECT_VARIABLE`], its project's folder. The Claude Code
//! plugin's MCP bridge sends both on each request, so Marley's tools know which terminal called
//! them. The id scopes what a tool does by default; it is no authority, since the MCP bearer
//! gates every call. A terminal Zed restores at a launch keeps the id it had (#575): the restore
//! hands the saved id to the builder under [`RESTORED_ID_VARIABLE`].

/// The variable holding a terminal's id.
pub const TERMINAL_ID_VARIABLE: &str = "MARLEY_TERMINAL_ID";

/// The variable holding the folder of the terminal's project.
pub const PROJECT_VARIABLE: &str = "MARLEY_PROJECT";

/// The key under which a restore hands the builder a terminal's saved id (#575).
///
/// The builder takes the value and leaves the key empty before anything else reads the
/// environment, so no program sees an id under it.
pub const RESTORED_ID_VARIABLE: &str = "MARLEY_RESTORED_TERMINAL_ID";

/// A new terminal id: a version 4 UUID.
#[must_use]
pub fn new_terminal_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Whether `id` has a terminal id's shape, so a malformed header names no terminal.
#[must_use]
pub const fn is_terminal_id(id: &str) -> bool {
    uuid::Uuid::try_parse(id).is_ok()
}
