//! Marley's browser (prong 3): the Chromium Marley starts for itself and the CDP client that
//! talks to it.
//!
//! Chromium runs headless as a transient user unit with its own profile under Marley's data
//! directory ([`service`]). [`cdp`] speaks CDP, Chromium's debugging protocol, to it over a
//! WebSocket on loopback, [`page`] attaches to a page and streams it as screencast frames, and
//! [`frame`] decodes them into images gpui draws; [`input`] maps keys and the mouse to CDP's,
//! and [`address`] turns what the address bar holds into a URL. The Browser tab that shows the
//! page is `marley_workbench`'s (`docs/marley/three-prong-plan.md`, prong 3).

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

pub mod address;
pub mod cdp;
pub mod frame;
pub mod input;
pub mod page;
pub mod service;
