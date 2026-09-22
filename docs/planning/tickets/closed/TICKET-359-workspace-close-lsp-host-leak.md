# TICKET-359 — Closing a workspace never drops its LSP host (rust-analyzer leaks)

- **Forge ticket:** #359 `d7981725-3b80-437c-8dae-0d2183d4bbf5` (bug, M20/lsp/resource-leak, #321 follow-up, from-inspect)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `cea6a711-e834-41b4-a5ae-8a707f0ec365`
- **Pipeline doc:** ../../pipeline/active/359-workspace-close-lsp-host-leak.spec.md
- **Source ticket:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #321 follow-up, from its inspect)
- **Status:** closed

## Summary
`RootView.lsp_hosts` is insert-only — `close_project_at` (app.rs:6381) cleans up the other per-project maps
(`collapsed_projects`/`agents`/`remotes`/`last_agent`) but NOT `lsp_hosts`, so a workspace's rust-analyzer (1–4 GB
RSS) keeps running until Marley exits even after that workspace is closed. `LspHost::drop` already does the polite
`shutdown`+`exit`+reap and is correct — it is simply never called for a closed workspace because nothing removes the
host from the map. #321 widened the blast radius: host creation is now open-document-state-derived on the pump, so
switching to a restored multi-project session spawns one rust-analyzer per root; several unreclaimed is a
user-visible memory problem on a long session.

## The fix
Remove the closed project's host from `lsp_hosts` in `close_project_at` (before `removed` is moved into its drop
thread) → `Drop` fires. The map is keyed by the project root (`active_root` at the insert, app.rs:4550), so the
remove uses the closed `Project`'s raw `root` field (design confirms the key forms match). `LspHost::drop` is
untouched. Dropping the host on the last EDITOR-tab close is OUT of scope (a separate #321 decision).

## Acceptance
Close a workspace → its `lsp_hosts` entry is gone (Drop shuts down + reaps); another open workspace's host survives.
Headless via `lsp_host_exists_for_test(root)`. Full EARS in the pipeline spec.
