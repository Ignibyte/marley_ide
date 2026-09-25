---
pipeline_id: 539e18c6-b2ed-4fe6-b42c-151901f7191f
ticket: docs/planning/tickets/open/TICKET-497-pick-source.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "B3b: Open a picked element's listener source in the editor"
type: feature
slice: prong 3 B3b (wave 2; after #496)
references: [docs/marley/browser-handoff.md, docs/planning/pipeline/queued/496-element-picker.spec.md]
---

## Title
From a picked element to its code: each listener in a pick is followed through its script's
source map to the original file and line in the workspace, which opens in Marley's editor.

## Scope
### In
- For each listener in a pick (#496), Marley reads its script's source map: the
  `sourceMapURL` `Debugger.scriptParsed` reports, resolved against the script's URL, or an
  inline `data:` map; it decodes the map's mappings (Source Map v3, base64 VLQ) and finds the
  original source, line and column for the listener's generated position.
- It finds the original source in the workspace: the map's `sources` entry joined with its
  `sourceRoot`, with a bundler's scheme and prefix (`webpack://`, `/@fs/`, a leading `./`)
  taken off, tried under each worktree of the workspace, first match wins.
- The tray shows each listener as `file:line` when found, else as the script's URL and line; a
  click on a found one (or `marley::OpenPickSource` on the tray's selection) opens the file in the
  editor at that line, in the workspace the tab belongs to.
- `browser_pick` gives each listener's original file and line beside the script location.

### Out (explicitly deferred)
- Showing a script that has no map, or whose source is not in the workspace, in a read-only
  buffer; source maps behind authentication; index maps with `sections` beyond the first level.

## Reference (§20)
Chrome DevTools' Sources panel, which shows a listener's original file through its source map
and opens it at the line; Marley does that in its own editor, through Zed's
`Workspace::open_abs_path` and `Editor::go_to_singleton_buffer_point` (the `editor` and
`workspace` crates), as Zed's debugger opens a stack frame's file. Warp: N/A.

### Prior art
- **Published material.** The Source Map format (ECMA-426, version 3): `version`, `sources`,
  `sourceRoot`, `mappings` in base64 VLQ, `sourcesContent`; the `//# sourceMappingURL=` comment.
  CDP's `Debugger.scriptParsed` (`sourceMapURL`), `Debugger.getScriptSource`,
  `Network.loadNetworkResource` for a map the page's origin serves.
- **Observed (the probe, 2026-09-25).** A listener added in `app.js` reports `app.js:0:69`, and
  the script's `sourceMapURL` is the relative `app.js.map` its comment names.
- **Code we already ship.** No source-map crate in `Cargo.lock` (checked `sourcemap`,
  `source-map`, `swc_sourcemap`, `oxc_sourcemap`); Zed's debugger (`debugger_ui`) opens a file at
  a line with `go_to_singleton_buffer_point`; the worktrees come from the workspace's project.

## UI proof
UI-AFFECTING. `script/e2e/497-pick-source.sh` (`compositor sway`, offline): a scratch repository
with `src/app.ts` and a built `dist/app.js` with its map (the fixture writes both, the map by
hand), served from `dist/`; Marley opens the repository. Steps: pick the button whose listener
`app.js` adds (`497-01-tray`: the listener as `src/app.ts:2`); click it (`497-02-opened`: the
editor on `src/app.ts`, the cursor on line 2); a pick on an element whose listener's script has
no map (`497-03-no-map`: its URL and line, nothing opened); the agent's `browser_pick` (the run
log: the original location).

## Locked-In Decisions
- D1 — Marley decodes the maps itself (a small base64 VLQ decoder in `marley_browser`) unless
  the Plan phase finds a crate already in the tree; the mappings are the only part it needs.
- D2 — The workspace's worktrees are the only place a source is looked for: a pick never opens a
  file outside the project.
- D3 — A map is fetched from the page's own origin, as the page could fetch it, or read inline.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pick's listener's script has a source map whose source is in the workspace, the tray shall show that source's path and line. | Shot `497-01-tray` |
| REQ-002 | WHEN the user clicks that listener, Marley shall open the source in the editor at that line. | Shot `497-02-opened` |
| REQ-003 | WHEN a listener's script has no source map, the tray shall show the script's URL and line and open nothing. | Shot `497-03-no-map` |
| REQ-004 | WHEN an agent calls `browser_pick`, each listener with a source map shall carry its original path and line. | The run log |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams against #496 as built, the design.
- **P2 Code** — the map fetch and decode, the lookup, the tray's link, the tool's field; fmt and
  clippy clean.
- **P3 Test** — the scenario, every shot; #496's scenario again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
