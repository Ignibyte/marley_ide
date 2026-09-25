# B3b: Open a picked element's listener source in the editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-497-pick-source.md
- **Pipeline spec:** 497-pick-source.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar A's "signature move" (`browser-handoff.md`: "picked
  element → listener source → source map → open the file in the editor at that line").
- **Order:** after #496, whose bundle carries the listeners' script locations.
- **Classification:** feature; `marley_browser` (the map fetch and decode), `marley_workbench`
  (the lookup in the worktrees, the tray's link, opening the editor). No Zed path expected.
- **Recall (§18.3):** no source-map code in the tree or the lockfile; Zed's debugger opens a
  stack frame's file at a line, the nearest model; the probe's `sourceMapURL` is relative.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #496 committed (5304066ee0); no other active pipeline; cargo idle; the README
  marker present.
- **Recall:**
  - The brain (consultation `bd70c7a6897a452f949a8b31d4a9cc15`; `5ecd86c4…`, asked twice by
    mistake, closed with no decision): nothing on this seam.
  - The ledger: `L-claude-496-chromium-picks-an-element-over-cdp-001` (listener positions count
    from 0; `Debugger.enable` replays `scriptParsed` with `sourceMapURL`); nothing on source maps.
  - #496 as built: `Listener { event, on, script, line, column, source_map }`, the pick read in
    the hub's task, the tray's rows of one height, `browser_pick` serializing the `Pick`.
- **Seams re-verified:** no source-map crate in `Cargo.lock` (`sourcemap`, `source-map`,
  `swc_sourcemap`, `oxc_sourcemap`, `vlq`); `marley_browser` already depends on `url` and
  `base64`; `Workspace::open_path(ProjectPath, None, true, window, cx)` and
  `Editor::go_to_singleton_buffer_point` (Zed's debugger opens a frame's file the same way);
  `Worktree::entry_for_path(&RelPath)` and `util::rel_path::RelPath::from_unix_str`. The probe:
  see the spec's prior art.

### Design
- **`marley_browser::source_map` (new, pure but for one call):**
  - `map_location(script_url, source_map_url) -> Option<MapLocation>`: a `data:` URL decoded
    (base64 or percent-encoded) as `Inline(text)`, else the URL joined to the script's
    (`url::Url::join`) as `Remote(url)`.
  - `Page::load_resource(url)`: `Network.loadNetworkResource` in the page's main frame (its
    frame id is the target's), then `IO.read` to `eof` (base64 chunks decoded), `IO.close`,
    capped at 32 MiB; a failed load names its status.
  - `SourceMap::parse(text, map_url)`: version 3, `sources` joined to `sourceRoot` and resolved
    against the map's URL, `mappings` kept as text; an index map's first-level `sections`
    parsed each with its offset.
  - `SourceMap::original(line, column) -> Option<OriginalPosition { source, line, column }>`:
    base64 VLQ decoded while scanning to the position, the last segment at or before it that
    names a source, as DevTools' `findEntry`; 0-based in, 0-based out.
  - `source_path(source) -> Vec<String>`: the path components a source names, its scheme and
    host taken off (`webpack://app/`, `http://host/`), Vite's `/@fs` kept as an absolute path,
    `.` dropped and `..` applied.
- **The pick (`marley_browser::pick`):** `Listener` gains `original: Option<SourcePosition {
  source, file, line, column }>`, filled by the workbench; line and column count from 1 (D4),
  the listener's own too.
- **The hub:** after staging a pick (the tray shows it at once), a task reads each listener's
  map, once per map in the pick, off the main thread for the parse and the scan, and sets the
  listener's `original`; `file` comes from the project of the pick's tab: an absolute source
  inside a worktree, else the longest suffix (D5) that `entry_for_path` finds as a file.
- **The tray:** each row shows its pick's first listener that has a place: `click → src/app.ts:2`
  as a link when the file is in the project, the original source and line muted when it is not,
  `click · plain.js:2` muted with no map, `click · …` while reading; its tooltip lists every
  listener. A click on the link opens the file in the tab's workspace (`open_path`, then
  `go_to_singleton_buffer_point` at the line and column), from a task, since opening an item in
  the tab's own pane updates the tab.
- **`browser_pick`:** each listener's `original` (`source`, `file`, `line`, `column`), in the
  output schema; `browser_picks` is unchanged.
- **Manifest:** `crates/marley_browser/src/source_map.rs` (new), `marley_browser.rs`,
  `pick.rs`; `crates/marley_workbench/src/browser.rs`; `crates/marley_mcp/src/registry.rs`;
  `script/e2e/497-pick-source.sh`, `script/e2e/browser-fixture.sh` (the agent's `pick` prints
  `original`). No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | a repository with `src/app.ts`, and `dist/` served with `app.js` and a hand-written `app.js.map` (line for line, `../src/app.ts`); pick the Save button | `497-01-tray`: `click → src/app.ts:2` |
| REQ-002 | click the link | `497-02-opened`: `app.ts` in the editor, the cursor on line 2 |
| REQ-003 | pick the Plain button, whose listener `plain.js` (no map) adds; click its place | `497-03-no-map`: `click · plain.js:2`, the Browser tab still in front |
| REQ-004 | the stand-in agent's `browser_pick` for both | the run log: `original` with `src/app.ts`, line 2, and none for `plain.js` |
| (regression) | #496's scenario | its shots |

### Risks
- A framework that delegates events (React's root listener) puts the listener on the root or
  the document, so its place is the framework's code, not the app's handler; DevTools'
  "framework listeners" are out of scope.
- A big bundle's map in the debug build: the parse and the scan run off the main thread, and the
  tray shows `…` meanwhile.
- The tray row grows by one chip; the rows stay one height.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (the probe), spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `marley_browser::source_map` (new): `map_location` (a `data:` map decoded, base64 or
    percent-encoded; any other URL joined to the script's), `source_path` (scheme and host off,
    `file:` and Vite's `/@fs/` as absolute, components decoded, `.` and `..` applied),
    `SourceMap::parse` (version 3; `sources` joined to `sourceRoot` and resolved against the
    map's URL, or the script's for an inline map; one level of an index map's `sections`),
    `SourceMap::original` (the section the place is in, then a base64 VLQ scan to the place: the
    last mapping at or before it, a one-value segment mapping to nothing), `Page::load_resource`
    (`Network.loadNetworkResource` in the main frame, `IO.read` to the end, `IO.close`, 32 MiB
    at most). `percent-encoding` joins its dependencies (already in the lockfile).
  - `pick.rs`: `Listener` counts its line and column from 1 and carries `original:
    Option<SourcePosition { source, file, line, column }>`.
  - The hub: `pick_captured` returns the staged pick's id; the capture task then reads each
    distinct map of the pick (`original_positions`, `load_map`, the parse and the scans on the
    background executor) and `pick_sources` sets each listener's `original`, `file` from
    `find_source` in the tab's project (an absolute path in a worktree, else the longest suffix
    down to two components); `Pick::sources_read`.
  - The tray: each row's listener place after the summary (`listener_place`,
    `render_listener_place`): the event muted, then the file and line as a link when the file
    is in the project, the original source's or the script's name and line muted otherwise,
    `…` while the maps are read; every listener in the tooltip. `open_pick_source` opens the
    file with `Workspace::open_path` and `go_to_singleton_buffer_point` at the line, from a
    task; a file gone since, or a failed open, says so in the tray (`tray_error`, which
    `send_error` became). `language` joins the workbench's dependencies for its `Point`.
  - `browser_pick`'s schema: `original` on each listener, and every line and column "from 1".
  - The stand-in agent's `pick` prints each listener's `original`.
- **Deviations from the design:** none beyond the plan's own (D4, D5); no `marley::OpenPickSource`
  action, since the tray has no selection for one to act on (the spec's scope says so).
- **Review against the criteria:**
  - REQ-001: the link shows `file:line` from `original.file`, found at staging; REQ-003: a
    listener with no map shows its script's name and line, not a link.
  - REQ-002: opening runs after the view's update: the new editor goes into the tab's own pane,
    whose activation deactivates the tab.
  - REQ-004: the tool serializes the listeners with `original`.
  - D2 holds: `find_source` only returns entries of the project's worktrees, and a source's
    `..` is applied before any lookup, so no suffix can climb out.
  - dylint's `map_lookup_then_insert`: the maps of a pick are a short list, looked up by pair.
- **Checks:** `cargo clippy -p marley_browser -p marley_mcp -p marley_workbench --all-targets
  -- -D warnings` clean; `cargo fmt` clean.
- **Checklist (no TaskCreate in this harness):** source_map.rs ✓, pick.rs ✓, hub ✓, tray ✓,
  open ✓, registry ✓, fixture ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/497-pick-source.sh` (`compositor sway`, offline Chromium): the
  repository holds `src/app.ts`; a loopback site serves `index.html` with a Save and a Plain
  button, `app.js` (line for line with the source, its click listener at `app.js:2:32`),
  `app.js.map` written by hand (`sources: ["../src/app.ts"]`, `mappings: AAAA;AACA;AACA;AACA`) and
  `plain.js`, which has no map. Steps: Ctrl+Shift+C and a click on Save; a click on the tray's
  link; the Sources tab again, Ctrl+Shift+C and a click on Plain, then a click on its place; the
  stand-in agent's `browser_pick` for both.
- **Shots (in the scratchpad, `e2e-497/`), each read:**
  - `497-01-tray` (REQ-001): `Pick 1 button “Save” click src/app.ts:2`, the place an accent
    link.
  - `497-02-opened` (REQ-002): an `app.ts` tab beside the Sources tab, the breadcrumb
    `src/app.ts`, the cursor at the start of line 2 (`save.addEventListener(…)`), the status bar
    `2:1 TypeScript`.
  - `497-03-no-map` (REQ-003): `Pick 2 button “Plain” click plain.js:2` muted, its tooltip
    `click on button#plain: plain.js:2` under the pointer after the click, and the Sources tab
    still in front: nothing opened.
- **The run log (REQ-004):** `listener click on button#save: …/app.js:2:32 -> src/app.ts:2:1
  (source http://127.0.0.1:…/src/app.ts)`; `listener click on button#plain: …/plain.js:2:60`
  with no `original`.
- **What the gate found, fixed at the source:** dylint's `async_block_without_await` (deny in the
  Marley crates) flagged the two `background_spawn(async move { … })` calls that parse and scan
  a map; they are `futures::future::lazy`, the Marley crates' idiom for work with no await. The
  scenario ran again on the rebuilt Marley: the same three shots and the same log.
- **Regressions:** `496-element-picker.sh` passes: its shots as before, its rows now showing
  `click app.js:2` (that fixture's `app.js` has no map), its log's positions counted from 1
  (`app.js:2:59`, `app.js:3:62`).
- **Focus report:** every run in the headless sway; "hyprland: 0 Marley windows before the run,
  0 after; the run added no rule and did not reload it".
- **Noticed, not in scope:** opening `app.ts` in the e2e profile had Zed start downloading its
  eslint server (the status bar says "Downloading eslint…"); Chromium is offline, Zed's language
  servers are not.
- **Gate:** `just gate-diff` — 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches
  the tree.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  496 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#497 under Added); `docs/marley_architecture/marley_browser.md`
  (Source maps), `marley_workbench.md` (Listener sources, the framework-delegation limit),
  `marley_mcp.md` (`original`, positions from 1); the plan's B3b row shipped. No path outside the
  Marley-owned set changed.
- **Knowledge appended:** `L-claude-497-chromium-loads-a-source-map-for-its-client-001`,
  `AD-claude-497-marley-reads-source-maps-and-finds-sources-in-the-worktrees-001`. The gate's
  dylint finding was a trap the ledger already names
  (`L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001`), not a bug in the
  behavior, so it has no `F-` block.
- **Brain:** consultation `bd70c7a6897a452f949a8b31d4a9cc15` closed with
  `decisions/marley-reads-source-maps-itself-and-finds-a-picks-source-in-the-worktrees`.
- **Ticket:** closed; the BACKLOG row went at promotion.
