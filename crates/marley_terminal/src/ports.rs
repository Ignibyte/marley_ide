//! A port of its own for each worktree agent's worktree (#590).
//!
//! Each worktree New Agent in Worktree makes holds a slot, and the terminals and tasks of its
//! project start with [`PORT_OFFSET_VARIABLE`] and [`PORT_VARIABLE`] from it, so the dev servers
//! of two worktree agents that read `PORT` ask for different ports. Where a slot is kept is the
//! workbench's to know: it registers the reader, and the terminal builders ask it as each terminal
//! starts, so a terminal restored at a launch gets its port too.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// The variable holding a worktree's offset, its slot times [`PORT_STEP`].
pub const PORT_OFFSET_VARIABLE: &str = "MARLEY_PORT_OFFSET";

/// The variable most dev servers read for the port they listen on.
pub const PORT_VARIABLE: &str = "PORT";

/// The port a worktree's offset is added to, the one most JavaScript dev servers default to.
pub const PORT_BASE: u32 = 3000;

/// The ports each slot has to itself, as Conductor gives each workspace ten.
pub const PORT_STEP: u32 = 10;

/// Reads the slot of the worktree whose folder it is given, `None` for a folder that holds none.
pub type SlotReader =
    Arc<dyn Fn(PathBuf) -> Pin<Box<dyn Future<Output = Option<u16>> + Send>> + Send + Sync>;

/// A [`SlotReader`] from an async function of the folder.
pub fn slot_reader<Read, Reading>(read: Read) -> SlotReader
where
    Read: Fn(PathBuf) -> Reading + Send + Sync + 'static,
    Reading: Future<Output = Option<u16>> + Send + 'static,
{
    Arc::new(move |folder| Box::pin(read(folder)))
}

/// The reader terminals started from now on ask, if any.
static SLOT_READER: RwLock<Option<SlotReader>> = RwLock::new(None);

/// Sets the reader each terminal started from now on asks for its project's slot, or none, so
/// terminals get no port.
pub fn set_slot_reader(reader: Option<SlotReader>) {
    *SLOT_READER
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = reader;
}

/// The variables a terminal or task of the project whose first folder is `folder` starts with:
/// none without a folder, a reader or a slot.
pub async fn variables(folder: Option<PathBuf>) -> Vec<(String, String)> {
    let reader = SLOT_READER
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let (Some(folder), Some(reader)) = (folder, reader) else {
        return Vec::new();
    };
    let Some(slot) = reader(folder).await else {
        return Vec::new();
    };
    let offset = u32::from(slot) * PORT_STEP;
    vec![
        (PORT_OFFSET_VARIABLE.to_string(), offset.to_string()),
        (PORT_VARIABLE.to_string(), (PORT_BASE + offset).to_string()),
    ]
}
