# TICKET-448 — Zed's dylint lints on the Marley crates

- **Ticket:** LOCAL #448 (chore, quality gates)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (not specced)
- **Source ticket:** the Out list of #447 (`../../pipeline/completed/447-rustal-quality-gates.spec.md`)
- **Status:** open

## Summary
Zed keeps a dylint library in `tooling/lints` whose lints catch the gpui mistakes clippy
cannot see: `entity_update_in_render` and `notify_in_render` (state changed while a view
renders), `blocking_io_on_foreground`, `async_block_without_await`,
`shared_string_from_str_literal`, `owned_string_into_shared` and `map_lookup_then_insert`.
The Marley crates should pass them too, `marley_workbench` first, since W3 to W6 grow its
views. #447 left them out because they need `cargo-dylint`, `dylint-link` and the nightly
toolchain `tooling/lints/rust-toolchain.toml` pins (`nightly-2026-03-21`, with `rustc-dev`,
`rust-src` and `llvm-tools-preview`), none of which is installed on the dev box yet.

## Acceptance
The tools are installed and recorded in CONSTITUTION §0's tools line. A new gate (the next
free number, 21) runs Zed's dylint library over the Marley crates, and a planted
`cx.notify()` inside a `render` turns it red. Every hit in the Marley crates is fixed at the
source.
