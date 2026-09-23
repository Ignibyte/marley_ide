// Marley: the `marley` settings block. The content types live here because the settings derive
// macros only resolve inside this crate; `crates/marley_workbench` reads them.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

/// Settings for Marley, the fork of Zed this build is.
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct MarleySettingsContent {
    /// Which layout the windows use.
    ///
    /// Default: "marley"
    pub layout: Option<MarleyLayout>,
}

/// A window layout.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum MarleyLayout {
    /// Zed's own layout: the Threads Sidebar and Zed's panel defaults.
    Zed,
    /// The Marley layout: a rail of projects with their terminals, and terminals in the center.
    #[default]
    Marley,
}
