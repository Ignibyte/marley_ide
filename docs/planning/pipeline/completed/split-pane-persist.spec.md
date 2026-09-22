---
pipeline_id: 01dd3c40-bbe5-4322-ac9d-4d5beaae1ef8
ticket: forge#258 (a293b7ac-9714-4fb7-8805-c9f9c2c10ccd) · local docs/planning/tickets/open/TICKET-258-split-pane-persist.md
aar_id: 2fdf7580-0f75-45d7-9989-bebb6d09eea8
status: Phase 5 — Complete PASS
title: Persist split-file panes across restart (c=<path> grid codec) — #246 follow-on
type: feature
milestone: M15
references: []
---

## Title
Persist a split-right file (`CodeView`) pane across restart by carrying its file path through the grid codec:
`serialize_leaf` today emits a **bare `c`** for a CodeView leaf (path LOST → the pane is dropped on restore, per
#154), so a #246 split-file pane vanishes after a relaunch. Extend the codec to `c=<path>` (mirroring the #205
`t=<cwd>` terminal-cwd codec) + a restore arm that re-reads the file into a CodeView pane (mirroring the #243
editor-tab paths restore). The LAST ticket of the M15 `/work 249-258` train — the (b) half of #258.

## Scope
### In
- **The `c=<path>` codec (PURE seam):** `serialize_leaf((CodeView, Some(path))) → "c={path}"`; `parse_leaf("c=<path>")
  → (CodeView, Some(path))`; the framing guard (`breaks_grid_framing`) gates a CodeView path exactly as it gates a
  terminal cwd (a path holding `, : =` / shell framing `\t\n\r` / the #177 `\x1f` → drop to a bare `c`). Back-compat:
  a pre-#258 bare `c` → `(CodeView, None)` → dropped on restore (no path to re-read).
- **`flatten` path-capture:** a CodeView leaf sources its path from the pane registry
  (`PaneContent::CodeView(cv).path`) into the leaf's `Option<String>` (today `None` for every non-terminal leaf).
- **The restore arm (SHIM):** the `restore_panes` CodeView arm (app.rs, today a `{}` drop) re-reads a `c=<path>`
  leaf into a CodeView pane via the shared file-load ladder; unreadable path or legacy bare `c` → dropped, no
  panic. Applied on **BOTH** restore paths (the whole-shell `restore_panes` + the legacy single-grid boot).
- **The persist trigger (SHIM):** creating a split-file pane triggers a layout persist so the `c=<path>` reaches
  `settings.toml` before the next boot (the #243 persist-TRIGGER discipline — verify the save FIRES, not just that
  the codec round-trips).

### Out (explicitly deferred)
- **(a) THE EDITABLE SPLIT PANE → fast-follow #259.** Making the split-file pane editable (typing/save/caret in the
  pane, like the editor tab) is a genuine model change, NOT a codec change: it requires
  `PaneContent::CodeView(CodeViewState)` → an `OpenFile` (Buffer+caret+saved_version+anchor), plus rewiring the
  **16+** `active_tab().editor()`/`.editor_mut()` sites (the #251 typing, #254 click-caret, #255 selection, #256
  clipboard, #257 motion, save + render forks) to also serve a **focused CodeView pane** — too large + risky to
  bolt onto the tight codec seam. Split per §3 (one shippable slice). The pane stays **read-only** in #258;
  persistence lands here so the editable pane (#259) inherits a persisted surface. (b)-alone completes the ticket
  headline "so split file panes … survive restart".
- Multi-cursor in a pane; per-pane undo isolation beyond what #253 gives; split-pane-specific find/replace.

## Reference (§20)
N/A — Marley/IDE-specific pane container + grid-persistence codec (no Warp analog). The persistence blob format is
Marley's own (#122 pane-kind codes → #205 `t=<cwd>` → #243 editor-tab paths lineage); this ticket extends it with
`c=<path>` by the same rules. The in-pane **editing** feel (deferred to #259) will inherit the #249-257 editor
stack. Clean-room §20 holds trivially — no reference-app source is consulted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — SCOPE SPLIT (the plan's primary call).** #258 ships ONLY (b) the split-pane persistence codec
  (REQ-001..004). (a) the editable split pane is DEFERRED to a fast-follow **#259**. Evidence: 16+ on_key_down /
  render / save sites key off `active_tab().editor()` (a TAB concept — measured this session at app.rs
  2696/2734/3199/3525/4439/4470/4566/4576/5268/5313/5355/…); a pane has no `editor()`. Making a pane editable is a
  `PaneContent` model change + a new focused-pane key route rewiring all 16 — a separate large slice.
- **D2 — the `c=<path>` codec MIRRORS the #205 `t=<cwd>` codec EXACTLY.** Same tuple `(PaneKind, Option<String>)`,
  same `breaks_grid_framing` guard, same drop-to-bare-kind-on-framing-break stance (the #205 D2). The 2nd tuple
  element generalizes from "the terminal's cwd" to "the leaf's associated path" (cwd for Terminal, file path for
  CodeView). No new blob grammar — just a second `kind=value` producer/consumer.
- **D3 — the restore CodeView arm re-reads via the SAME load ladder as `split_file_pane`/`open_file_in_viewer`**
  (`load_code_view_state`, or a load-from-path core the design extracts if the closure can't reach `&mut self`);
  unreadable → drop, no crash (matching #243's editor-tab restore). BOTH restore paths get the arm (#234
  audit-ALL-sites).
- **D4 — the split-file-pane creation TRIGGERS `persist_grid`** (the #243 persist-TRIGGER lesson —
  `PR-claude-persist-verify-trigger-not-just-codec-001`: verify the save fires + `c=<path>` lands in settings.toml,
  not merely a codec round-trip).
- **D5 — `flatten`/`serialize_grid` gain a CodeView-path source** (the pane registry's `PaneContent::CodeView(cv)`
  → `cv.path`). The exact signature (a second `paths` map alongside `cwds`, or a unified per-leaf "detail" map) is
  the design's call; keep the `(PaneKind, Option<String>)` leaf tuple unchanged.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the grid is serialized, a `CodeView` pane leaf shall emit `c=<path>` carrying its file path; a path containing a grid-framing delimiter (`, : =` / `\t\n\r` / `\x1f`) shall instead emit a bare `c` (no blob corruption). | `#[cfg(test)]` round-trip + framing-drop unit; gate:4/5 |
| REQ-002 | WHEN a saved grid is restored, a `c=<path>` leaf shall re-read `<path>` into a `CodeView` pane; an unreadable path or a legacy bare `c` shall be dropped without a panic, on BOTH the whole-shell and legacy restore paths. | parse unit + no-panic unit + driven quit→relaunch |
| REQ-003 | WHEN a split-file pane is created, the system shall trigger a layout persist so its `c=<path>` is written before the next boot (not merely held in memory). | grep the `persist_grid` trigger on `split_file_pane` + driven settings.toml inspection |
| REQ-004 | The pure codec fns (`serialize_leaf`/`parse_leaf` + the `flatten` path-capture) shall be coverage 100% and MSI 100%. | gate:4 (cov) + gate:5 (mutation) |

## Phase Plan
- **P2 Design** — resolve D5's `flatten`/`serialize_grid` signature (paths source) + the D3 restore-load
  structure (extract a load-from-path core vs inline the fs ladder); the exact restore-arm code for BOTH paths;
  the manifest; the Regression Test Plan (round-trip, framing-drop, back-compat bare `c`, unreadable-drop,
  both-restore-paths). Confirm §20 N/A.
- **P3 Implement** — the pure codec extension (grid_layout.rs) + the two restore arms + the persist trigger
  (app.rs shim). `cargo mutants --list -f grid_layout.rs` after (the #246/#252 skip-detach re-check if a shim is
  refactored).
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — write + RUN the codec units (cov/MSI 100) + the DRIVEN quit→relaunch proof (split a file pane
  → relaunch → it restores; the #243 persist-TRIGGER check; data-safe — NEVER ⌘S a real repo file); gate green.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR capture (2fdf7580); file the #259 editable-pane fast-follow;
  close the ticket; archive.
