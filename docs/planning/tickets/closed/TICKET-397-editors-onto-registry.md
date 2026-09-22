# TICKET-397 — Editors onto the ContentId registry: one Buffer, many views (the #388 slice-3 / #259 collapse)

- **Forge:** #397 `19d1d1cd-9326-4595-b757-e9912029cf65` (sprint "M28 — The Registry Payoff" `6558258f-a0d2-45fb-b601-cd5329a1cd77`)
- **Type:** feature
- **Milestone:** M28
- **Status:** closed (shipped M28, 2026-08-04)
- **Depends:** #396 (terminals onto the registry) — **HARD**; #396 ships the concrete `Content` enum, the `(ContentId, ContentKind)` view-tag pattern, the accessor-resolution template, and the `release_view` close routing this slice follows verbatim
- **React-first (policy 2026-08-04):** APPLICABLE — build & visually verify the one-buffer-two-views demo in marley-web FIRST (shared edit propagation across two editor views of the same `PaneItem`, one dirty flag; `pnpm --filter @workspace/marley-ide run dev` → localhost:5173), then port 1:1. Vocabulary tie: **ContentId ↔ PaneItem** stays aligned (`marley-web/docs/MARLEY-PARITY.md` § Shared vocabulary).
- **Pipeline:** ../../pipeline/completed/397-editors-onto-registry.spec.md (ran to Phase 5 — Complete PASS)

## Summary
The **#388 train slice-3 — the first VISIBLE registry payoff.** Today the same file open in an editor
tab AND a split cell is **two divergent `Buffer`s** (the shipped #259 stance: `TabContent::CodeView
(EditorSurface)` tabs.rs:31 and `PaneContent::CodeView(EditorSurface)` workspace.rs:329 each own their
own buffer; the four births are `Tab::code` tabs.rs:163, split-create app.rs:5307, split-restore
app.rs:1972, `from_files` session-restore app.rs:2093 — the split even seeds from DISK, so a dirty
tab's split opens stale). The #275 external-change machinery is the net: saving either copy trips the
other's disk-stat conflict banner — the **self-conflict arm**. This slice moves the `OpenFile`
instance-half (the `Buffer` + undo history + `saved_version`/dirty + nonce + #275 disk/conflict state
+ LSP doc identity — editor_surface.rs:32-75) into **`Content::Editor`, owned ONCE per open file** in
the #394 `ContentRegistry`; tabs and split cells become **views** holding a `(ContentId, ContentKind)`
and keeping only per-view state (caret/selection, scroll, focus, composition — per
`AD-claude-one-instance-many-views-001`). Undo is **per-instance**: one history, either view's ⌘Z
rewinds the shared buffer (the #388 open-Q3 recommendation; observed Zed behavior, behavior-level
only). The #275 self-conflict arm dies **by construction** (one snapshot per file); external detection
survives unchanged. LSP doc-sync derives from the registry, so one file is exactly ONE `didOpen`
regardless of view count. Persistence: **ids never serialize** (the #396 discipline) — the `V=`/`c=`
codecs are untouched and restore **rebuilds** registry entries from restored paths, resolving the
second mount of a path to `acquire_view` instead of a second buffer; the #163/#205/#177 round-trips
stay byte-identical. **Visible payoff: the same file in a tab and a split shares edits live, with one
dirty ● and one undo history.** Size **L**.

## Out (later train slices / explicit non-goals)
Add-any-thing-to-a-pane + the rail's `ContentId` cross-link labels (slice-5); nameable arrangements
(slice-6); cockpit/browser onto the registry (slice-8); any CRDT/collab multi-replica buffer
(architecturally excluded — single process, one owner); multi-language LSP (#315 — doc-sync stays
rust-scoped exactly as shipped); an unsaved-changes-on-close prompt (today's silent-discard posture
is unchanged); full editor-feature parity inside split panes beyond what the shared instance gives
for free (the #259 follow-up stands).

## Headline acceptance
Open a file in the editor tab, split it into a cell: typing in EITHER view appears in the other
(one `Buffer`), one dirty ● governs both, ⌘Z from either view rewinds the one shared history, and
saving from one view no longer trips the other's #275 conflict banner — while a genuinely external
write still flags. The LSP host holds exactly one open doc per file (`open_doc_count` = 1 for a
tab+split pair; `didClose` only on the LAST view's close). The #163/#205/#177 persistence
round-trips are byte-identical and no `ContentId` ever serializes. Inspect critics: acquire/release
leak-path over every editor close path, LSP double-didOpen/didChange ping-pong, undo divergence,
restore-rebuild dedupe, per-view state bleed on focus switch. Driven headless capture of the
shared-edit payoff + the React↔Marley parity pair at Validate.
