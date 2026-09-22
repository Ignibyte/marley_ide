# TICKET-308 — LSP client core: rust-analyzer lifecycle + JSON-RPC framing + initialize handshake

- **Forge ticket:** #308 9371f921-9dc4-4c9f-bceb-56c5e38ee9e9 (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** 4556dc66-8ed4-4b5c-88d2-e5224609fcaa
- **Pipeline doc:** ../../pipeline/active/308-lsp-client-core.spec.md
- **Source ticket:** forge #308 (M20 batch #308-317, created 2026-07-14)
- **Status:** closed

## Summary
Bring up the LSP wire as a new `marley_lsp` crate: Content-Length framing (incremental decoder),
JSON-RPC request/response/notification routing with timeout + cancel-on-drop, a lifecycle state
machine with capped crash-restart, and the `initialize` handshake — plus the rust-analyzer spawn
shim (settings-configured, per-language shape) and an `LSP:` status-bar segment. The transport is
deliberately DUMB (no buffer/language knowledge); every M20 feature ticket (#309-317) speaks
through it. Clean-room from the published LSP 3.17 spec + MIT crates.io `lsp-types` (roadmap B6).

## Acceptance
Open a `.rs` file → rust-analyzer spawns, handshakes, status reads ready (REQ-001); kill it →
capped restart then `failed` (REQ-002); absent binary → quiet muted note, editor fine (REQ-003);
framing/routing/cancel/settings round-trip proven pure at cov/MSI 100 (REQ-004..008). Full EARS
table in the pipeline spec.
