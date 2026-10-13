# Rusty's agent in the title bar — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-778-rustys-agent-in-the-title-bar.md
- **Pipeline spec:** 778-rustys-agent-in-the-title-bar.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-12: the title bar's Rusty button should open Rusty's agent, and
  Rusty's home page button goes back to the bottom. Run autonomously with the batch.
- **Classification / tier:** feature, Marley crate only (`marley_workbench`), no Zed hunk.
- **Checklist:** pick ✓ (BACKLOG row removed), pre-flight ✓, recall ✓, mint ✓, prior art ✓,
  spec ✓, design ✓.
- **Recall (§18.3):**
  - PR-claude-701: a handler that opens a tab updates the workspace, so a button's click is a
    plain closure over the weak workspace, never a `cx.listener` of a view the workspace holds
    (`HomeButton` already follows it).
  - #746's notes: `TitleButtons` observes the settings, `Rusty` and `Assistant` globals; the in-app
    page names the buttons beside the `?`.
  - The brain (consultation 727e486d…): nothing on this seam.
- **Discovery:** `status_buttons.rs` (`HomeButton` draws Home alone, with no observers;
  `TitleButtons` draws Rusty then Marley); `assistant.rs` (`talk_to`, `talk_to_marley`,
  `marley_present`); `rusty.rs` (`is_on`, `is_connected`, `open_home_here`, `RUSTY_ICON`);
  `guide/index.html` line ~360 (the buttons beside the `?`).

### Design
- `assistant.rs` (Marley crate): `rusty_present(cx)` beside `marley_present`; `talk_to_rusty`
  beside `talk_to_marley`.
- `status_buttons.rs` (Marley crate):
  - `TitleButtons::render`: the Rusty button shows while AI is on and `rusty_present`, its tooltip
    `Tooltip::for_action("Talk to Rusty", &TalkToRusty, cx)`, its click `talk_to_rusty`.
  - `HomeButton` gains the observers of the settings and the `Rusty` global (made with `cx.new(|cx|
    HomeButton::new(handle, cx))`) and draws Home, then Rusty while `rusty::is_on` and
    `rusty::is_connected`, Rusty opening `rusty::open_home_here`, tooltip "Rusty", as #739's.
  - The module doc names the new placement.
- `guide/index.html` (Marley crate): the title bar's Rusty opens Rusty's conversation; Rusty's home
  page button sits beside Home in the status bar.
- No Zed crate changes, no ledger rows.

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| 001, 003 | Rusty on through the stand-in, Marley on the scripted agent; the window as it starts | `778-01-bars` |
| 001 | Click the title bar's Rusty | `778-02-title-rusty` |
| 002 | Click the status bar's Rusty | `778-03-status-rusty` |
| 004 | Rusty off in the run's settings | `778-04-rusty-off` |

### Risks
- The title bar's Rusty now waits on Rusty's agent entry, which needs an agent (`marley.assistant`
  on, an agent found): with none, the title bar shows no Rusty while the status bar's does. That
  is the point of D1: no button that does nothing.

## Phase 2 — Code
- **Built:** `assistant.rs`: `rusty_present` and `talk_to_rusty` beside their Marley twins.
  `status_buttons.rs`: `TitleButtons`' Rusty shows while AI is on and Rusty's entry is there, its
  tooltip "Talk to Rusty" with the action's key, its click Talk to Rusty; `HomeButton::new`
  observes the settings and `Rusty`, and the view draws Home, then Rusty while Rusty is on and
  connected, opening Rusty's home page (#739's button, tooltip "Rusty"); the module doc names the
  placement and Chad's words. `guide/index.html`: the in-app page's sentence about the buttons.
- **Deviations:** none.
- **Review:** both clicks are plain closures over the weak workspace (PR-claude-701); the title
  bar's Rusty re-renders on the `Assistant` global it already observes, which carries Rusty's
  entry; the status bar's on the settings and `Rusty` globals it now observes. No Zed hunk.
- **Gate:** `script/gates.sh --diff` GATE GREEN [diff], 17 of 17 (`gate-778.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/778-rustys-agent-in-the-title-bar.sh` under `compositor sway`, #746's
  set-up (the Marley entry on a scripted agent, Rusty on through its stand-in `rusty-mcp`, the
  copy's thread list emptied). A first run with `STOP_AT_BARS=1` placed the two Rusty buttons.
- **The final run**, every check passed (`e2e exit 0`; Hyprland untouched, sway stopped):
  - **778-01-bars** (REQ-001, REQ-003; read in crops of both bars): the title bar reads `?`,
    Rusty, Marley, Sign In; the status bar's right reads Zed's edit prediction, Home, Rusty, then
    Zed's own items.
  - **778-02-title-rusty** (REQ-001): the title bar's Rusty opened "New Agent Thread" in a repo
    tab with "Message Rusty" in its box, its row "Rusty · idle"; the tooltip reads "Talk to
    Rusty"; the scripted agent logged a `session/new`.
  - **778-03-status-rusty** (REQ-002): the status bar's Rusty opened Rusty's home page in a repo
    tab (RUSTY, Recent Pages, Follow-ups due, Tasks); the tooltip reads "Rusty".
  - **778-04-rusty-off** (REQ-004, crops): the title bar reads `?`, Marley, Sign In; the status
    bar's right reads Zed's edit prediction, Home, then Zed's items: no Rusty anywhere.
- **Pre-existing — not in scope:** Rusty's home page in a scenario lists the user's own recent
  page names, read from the copied profile (seen before, in the hand-off notes).
- **Gate after the scenario:** `script/gates.sh --diff` again on the final tree.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Changed, #778); `docs/marley_architecture/marley_workbench.md` (the title
  bar section); the in-app guide page in the code change; `docs/marley/guide.md`'s button text
  goes with the documentation commit that follows. No Zed path, no ledger row.
- Ledger appends: none. No bug was found in Code or Test, and the lesson (place buttons from a
  first shot with a stop flag) is #746's.
- Brain: consultation 727e486d… closed with `brain no-decision` (Chad's instruction, carried out).
- Ticket closed; pipeline archived; committed on `marley/workbench-shell`.
