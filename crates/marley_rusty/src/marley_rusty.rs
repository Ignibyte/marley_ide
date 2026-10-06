//! `marley_rusty`: the pure core of Rusty in Marley (`docs/marley/rusty-in-marley.md`, R-D1).
//!
//! Typed views of what `rusty-mcp`, Rusty's MCP server, answers, with no gpui and no IO; the
//! adapter in `marley_workbench::rusty` connects to it and draws. Rusty stays the store and the
//! only writer of its data: Marley reads and writes through Rusty's tools.
//!
//! `fixtures/` holds answers in `rusty-mcp`'s shape, and `stand_in/rusty-mcp` serves them to the
//! e2e scenarios, which never reach the user's own Rusty (R-D8).
//!
//! - [`settings`]: Rusty's own server settings and the embedding provider (#643).
//! - [`vault`]: the vault's tree, brain search's hits, and the Brain view's rows (#644).
//! - [`page`]: a rendered page, the wikilink pass for Zed's renderer, and a tab's history (#645).
//! - [`knowledge`]: a page's tags, backlinks and links, and search snippets' marks (#646).
//! - [`graph`]: the vault as a graph and the part the Graph tab shows; [`graph_layout`]: where its
//!   nodes sit and how its view looks at them (#647); [`graph_settings`]: the panel's groups,
//!   display and forces, kept for every window (#657).
//! - [`switcher`]: the page picker's list, its recently opened pages and its order (#654).
//! - [`project`]: a workspace's folders joined to a brain project page; [`decisions`] and
//!   [`tasks`]: the follow-ups due and the task groups the project view reads (#655).
//! - [`bookmarks`]: Rusty's bookmarks and favourites, and the writes to them (#662).
//! - [`capture`]: a line or a URL captured into the brain, and a vault imported (#663).
//! - [`memories`]: Rusty's long-term memories and the writes to them (#664).
//! - [`skills`]: Rusty's skills and scripts, and the writes to them (#665).
//! - [`secrets`]: Rusty's secrets vault behind its PIN, and the writes to it (#667).

// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

pub mod bookmarks;
pub mod capture;
pub mod decisions;
pub mod graph;
pub mod graph_layout;
pub mod graph_settings;
pub mod knowledge;
pub mod memories;
pub mod page;
pub mod project;
pub mod secrets;
pub mod settings;
pub mod skills;
pub mod switcher;
pub mod tasks;
pub mod vault;

pub use settings::{EmbeddingProvider, ServerSettings};
