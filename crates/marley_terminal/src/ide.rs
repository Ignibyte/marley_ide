//! The port of Claude Code's IDE link a project's terminals name (#653).
//!
//! While Marley serves Claude Code's IDE link for a local project, each new terminal of that
//! project starts with [`PORT_VARIABLE`], the port of the project's own server, which tells a
//! `claude` started there which lock file is its IDE's, and [`AUTO_CONNECT_VARIABLE`], which asks it
//! to connect at its start. The workbench serves the link and sets the port here; the project's
//! terminal builder reads it by the project's entity id.

use std::collections::HashMap;
use std::sync::RwLock;

/// The variable naming the port of the IDE whose lock file a Claude Code uses.
pub const PORT_VARIABLE: &str = "CLAUDE_CODE_SSE_PORT";

/// The variable asking Claude Code to connect to an IDE at its start.
pub const AUTO_CONNECT_VARIABLE: &str = "CLAUDE_CODE_AUTO_CONNECT_IDE";

/// Each served project's port, by the project's entity id.
static PORTS: RwLock<Option<HashMap<u64, u16>>> = RwLock::new(None);

/// Sets the port `project`'s new terminals name, or none.
pub fn set_port(project: u64, port: Option<u16>) {
    let mut guard = PORTS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let ports = guard.get_or_insert_with(HashMap::new);
    let _previous = match port {
        Some(port) => ports.insert(project, port),
        None => ports.remove(&project),
    };
    drop(guard);
}

/// The variables a new terminal of `project` starts with: none while Marley serves no IDE link
/// for it.
#[must_use]
pub fn variables(project: u64) -> Vec<(String, String)> {
    let port = PORTS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .and_then(|ports| ports.get(&project).copied());
    port.map(|port| {
        vec![
            (PORT_VARIABLE.to_string(), port.to_string()),
            (AUTO_CONNECT_VARIABLE.to_string(), "true".to_string()),
        ]
    })
    .unwrap_or_default()
}
