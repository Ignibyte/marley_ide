// Marley: the fork's own settings migrations, in a module of their own so no upstream migration's
// date collides with one.

use anyhow::Result;
use serde_json::Value;

use crate::migrations::migrate_settings;

const MARLEY_KEY: &str = "marley";
const RUSTY_TOOLS_KEY: &str = "rusty_tools";
const RUSTY_KEY: &str = "rusty";

/// Moves `marley.rusty_tools` (#633) into `marley.rusty` (#643): its value becomes
/// `agent_tools`, and `true` also turns Rusty on, since the tools are offered only while it is on.
/// A key already under `marley.rusty` wins.
pub(crate) fn move_rusty_tools_into_rusty(value: &mut Value) -> Result<()> {
    migrate_settings(value, &mut migrate_one)
}

fn migrate_one(object: &mut serde_json::Map<String, Value>) -> Result<()> {
    let Some(marley) = object.get_mut(MARLEY_KEY).and_then(Value::as_object_mut) else {
        return Ok(());
    };
    let Some(rusty_tools) = marley.get(RUSTY_TOOLS_KEY).cloned() else {
        return Ok(());
    };
    // A `rusty` that is not an object is left for the schema to flag, with the old key beside it.
    if marley
        .get(RUSTY_KEY)
        .is_some_and(|rusty| !rusty.is_object())
    {
        return Ok(());
    }
    marley.remove(RUSTY_TOOLS_KEY);
    let Some(rusty) = marley
        .entry(RUSTY_KEY)
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
    else {
        return Ok(());
    };
    if rusty_tools == Value::Bool(true) {
        rusty.entry("enabled").or_insert_with(|| Value::Bool(true));
    }
    rusty.entry("agent_tools").or_insert(rusty_tools);
    Ok(())
}
