# A failed block's errors as project diagnostics — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-623-a-failed-blocks-errors-as-diagnostics.md
- **Pipeline spec:** 623-a-failed-blocks-errors-as-diagnostics.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `merge_lsp_diagnostics` is the one public way in; diagnostics are keyed by a language server id, and paths outside a worktree are dropped.
  - No Marley crate publishes diagnostics yet; the Diagnostics view reacts to the project's diagnostic events.
  - Plan D6 names this; #620's locator is its reader.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** the open question settled by reading: `LspStore::merge_lsp_diagnostics(source_kind,
  updates, merge, cx)` takes any `LanguageServerId` (the clangd extension merges under its own),
  skips a path outside every worktree with a warning, and refuses a remote project; the update is
  `project::lsp_store::DocumentDiagnosticsUpdate { diagnostics: lsp::PublishDiagnosticsParams,
  server_id, result_id, registration_id, disk_based_sources }`; `merge` keeps the existing entries
  it returns true for, so `|_, _, _| false` replaces the server's entries for that path, and an
  empty list clears them. #620's `failures` already returns every failure with its severity.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation 6aa922d60c7d4d4485fb8142b8a0c1a3):
  nothing on this seam.

### Design
- **`crates/marley_workbench/src/failures.rs`:** `BLOCK_DIAGNOSTICS`, a reserved
  `LanguageServerId` far above any real server's. `read_finished` takes the view's workspace; for
  each newly finished block keyed by (command, folder): a failed one with failures replaces that
  key's published paths with its own (each failure an `lsp::Diagnostic` at its line and column,
  error or warning, source `marley`, the report's text); a successful one, or a failed one with
  none, clears the key's paths. The published paths per key live in `BlockFailures`.
- **`crates/marley_workbench/Cargo.toml`:** `lsp` and `language` as dependencies where missing.
- **Manifest:** `failures.rs`, the Cargo manifest, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | a task printing two rustc-shaped errors and a warning in `src/main.rs`, exit 101 | `status.png` (the status bar's counts), `diagnostics.png` (the Diagnostics view) |
| REQ-002 | the same task, made to pass by a flag file, run again | `cleared.png` |

### Risks
- A reserved server id has no language server status: the Diagnostics view names entries by their
  `source`, and the counts sum every server's summaries.

## Phase 2 — Code (2026-09-30)
- **Built (`failures.rs`):** `BLOCK_DIAGNOSTICS`; `Command` (text, folder); `published`;
  `read_finished` gathers every finished run with its failures and calls `publish`, which groups
  them by file, clears the files the same command's last run named and this one does not, and
  merges the rest through `merge_lsp_diagnostics` with `|_, _, _| false`; `diagnostic` maps a
  failure. `lsp` added to the manifest.
- **Clippy found:** a borrow of the file map in the filter against its move (`needless_collect`
  forbade collecting first), settled with a list of the named files.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/623-a-failed-blocks-errors-as-diagnostics.sh` (sway).
- **Run 1:** `status.png` and `diagnostics.png` passed; `cleared.png` did not: after the passing
  rerun the status bar still counted 2 errors and 1 warning. Zed's rerun had given the tab a new
  `Terminal`, which nothing watched (F-claude-623). Fixed: `watch` follows the view's terminal,
  re-armed from `observe_self`; clippy (`needless_pass_by_ref_mut`, a borrow of `cx`), the gate
  GREEN again, rebuilt.
- **Run 2, the shots read:**
  - `status.png` (REQ-001): the `build` tab's failed block (three reports, the Jump to Failure chip,
    `exit 101`); the status bar's diagnostics `2` errors and `1` warning; `src` marked in the
    project panel.
  - `diagnostics.png` (REQ-001): the Diagnostics view's tab `2 ⚠ 1`, `main.rs src/` with
    `unused variable: \`y\` (marley)` and `cannot find value \`x\` in this scope (marley)` on line
    2 and `cannot find value \`w\` in this scope (marley)` on line 3, the words underlined.
  - `cleared.png` (REQ-002): after `ok` was made and the task run again, the tab reads `compiled`
    with the check pill; the Diagnostics tab says `No problems`; the status bar shows no counts.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide; `terminal_blocks.md`; the plan's T4 row.
- **Knowledge:** F-claude-623-a-reruns-new-terminal-was-not-watched-001,
  L-claude-623-zeds-task-rerun-replaces-the-tabs-terminal-001.
- **Brain:** the consultation closed with `brain decide`.

