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

pub mod settings;
pub mod vault;

pub use settings::{EmbeddingProvider, ServerSettings};
