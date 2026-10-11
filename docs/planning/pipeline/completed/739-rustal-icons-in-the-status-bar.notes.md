# Rustal icons in the status bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-739-rustal-icons-in-the-status-bar.md
- **Pipeline spec:** 739-rustal-icons-in-the-status-bar.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - No Marley status item exists; the Fleet icon is a dock button. Right items render reversed (`status_bar.rs:221`).
  - #679 replaced #672's row of screen buttons with the single Rusty button this removes.
  - #701: Home's page is Home's first tab and comes back when it closes; opened elsewhere it is an ordinary tab.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `workspace::StatusItemView` and `StatusBar::add_right_item`; no Marley status item yet.
  - `rail.rs` `render_rusty_button` (4444), drawn in the header while `brain.connected` (4419).
  - `home_page.rs` `ensure` (40) and `MarleyHome::new`; `rusty/home_tab.rs` `ensure` (202),
    `open_later` (183, into the Rusty group), `RustyHome::new`; `rusty::{is_on, is_connected,
    RUSTY_ICON}`.
  - `threads_page::open` (#737); `assistant::talk_to` (#738); `Assistant` is a private global.
  - Icons: Home's tab is `ListTree`, Rusty's `Blocks`, the Threads page's `Thread`, the Marley entry's
    threads `Sparkle`.

### Design
- **New `crates/marley_workbench/src/status_buttons.rs`** (Marley crate): `RustalButtons`, a
  `Render + StatusItemView` holding its workspace's weak handle, added to every workspace's status
  bar from `init` (`observe_new`, `add_right_item`). It redraws when the settings, Rusty or the
  assistant change. Buttons, left to right: Home (always), Rusty (on and connected), Threads (AI
  on), Marley (the entry there). Each click is a plain closure over the weak workspace
  (PR-claude-701): `home_page::open_here`, `rusty::open_home_here`, `threads_page::open`,
  `assistant::talk_to_marley`.
- **`home_page.rs`, `rusty/home_tab.rs`:** `open_here(workspace, …)`: the workspace's page forward,
  else a new one added to its center and shown.
- **`assistant.rs`:** `Assistant` `pub(crate)` so the buttons can observe it; `marley_present(cx)`
  and `talk_to_marley(workspace, …)`.
- **`rail.rs`:** the header's Rusty button and `render_rusty_button` go.
- **`marley_workbench.rs`:** the module and its `init`.
- **The guide:** the buttons; the rail header's Rusty button gone.
- **File manifest:** `status_buttons.rs` (new), `home_page.rs`, `rusty/home_tab.rs`, `rusty.rs`,
  `assistant.rs`, `rail.rs`, `marley_workbench.rs` (Marley crate); the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 005 | Starts on the repo with Rusty's stand-in and the Marley entry | 739-01-bar: the four buttons; the rail header without Rusty's |
| 002 | Clicks Home, Rusty, Threads | 739-02-home, 739-03-rusty, 739-04-threads: each page in a repo tab |
| 003 | Clicks Marley | 739-05-marley: a Marley tab in the repo; the log's new session |
| 004 | Clicks Threads again | 739-06-again: the same Threads tab in front, one Threads tab |

## Phase 2 — Code
- **Checklist** (no TaskCreate here): `status_buttons.rs` ✓, `home_page.rs` ✓, `rusty/home_tab.rs` ✓,
  `rusty.rs` ✓, `assistant.rs` ✓, `rail.rs` ✓, `marley_workbench.rs` ✓, guide ✓, scenario
  (before the gate) ✓, gate ✓.
- **Built:**
  - `status_buttons.rs` (new): `RustalButtons` (`Render + StatusItemView`, a weak workspace),
    added to every workspace's status bar from `init`; it redraws on the settings, Rusty's global
    and the assistant's. Home always; Rusty while on and connected; Threads while AI is on;
    Marley while the entry is there. `hide_setting` is `None`: each button follows its feature's
    own switch.
  - `home_page::open_here` and `rusty::home_tab::open_here` (re-exported as
    `rusty::open_home_here`): the workspace's page forward, else a new one in its center.
  - `assistant.rs`: `Assistant` is `pub(crate)` (observed); `marley_present`; `talk_to_marley`.
  - `rail.rs`: the header's Rusty button and `render_rusty_button` removed; the re-export of
    `OpenHome` and `open_later` from `rusty.rs` went with them (unused).
  - The guide: "Buttons at the right of the status bar"; the Rusty group's line.
- **Deviations:** none from the design.
- **Found before the gate:** a missing `Debug` on the public struct, the unused re-export, and
  clippy's `too_long_first_doc_paragraph` on the module doc; fixed.
- **Review of the diff:** REQ-001 `render`'s four conditions; REQ-002/003 the four clicks, plain
  closures over the weak workspace (PR-claude-701); REQ-004 the `open_here`s bring an open page
  forward; REQ-005 the header. Re-entrancy: each click updates one workspace.
- **Gate:** `739-gate-1.log`: GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/739-rustal-icons-in-the-status-bar.sh`, under `compositor sway`, on
  #738's set-up. A first pass (`shots-739a`, `STOP_AT_BAR=1`) found the buttons at x 1309, 1329,
  1349 and 1369 on the status bar's row (y 985).
- **The Test phase's run (`shots-739-test`), after `just build` and `739-gate-1.log` green: every
  check passes.**
  - **739-01-bar (REQ-001, REQ-005):** the status bar's right end holds the four new icons (Home's
    list tree, Rusty's blocks, Threads' bubble, Marley's sparkle) left of Zed's dock buttons; the
    rail's header shows PROJECTS with no Rusty button.
  - **739-02-home (REQ-002):** Home's page in a repo tab (Start, New Agent with New Agent…, Recent
    Projects, Agents at Work, Configure, Agent Activity), its row under repo; the tooltip "Home".
  - **739-03-rusty (REQ-002):** Rusty's home page in a repo tab, beside Home's; the tooltip
    "Rusty". Its recent pages are the copied profile's.
  - **739-04-threads (REQ-002):** the Threads page in a repo tab.
  - **739-05-marley (REQ-003):** a Marley tab "Message Marley" in the repo, its row "Marley ·
    idle"; the tooltip "Talk to Marley"; the check: one new session.
  - **739-06-again (REQ-004):** Threads clicked again: the Threads tab in front, still one Threads
    tab.
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: Home, Rusty, Threads and Marley at the right of the
  status bar); the architecture note (`marley_workbench.md`, "Buttons at the right of the status
  bar"); the slice line in `workbench-shell.md`; the guide came with Phase 2. No Zed path is
  touched.
- **Knowledge appended:** none: no bug reached a run, and the pattern (a status item from
  `observe_new`) is recorded in the architecture note.
- **Brain:** no `rusty` MCP server in this repository's sessions; no brain loop ran.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `739-gate-1.log`, GATE GREEN [diff], 17 passed, on the tree committed.
