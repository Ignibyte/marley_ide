# B3a: Pick an element in the Browser tab and send it to the agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-496-element-picker.md
- **Pipeline spec:** 496-element-picker.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar A (`browser-handoff.md`), after wave 1 (#487 to #495).
- **Classification:** feature, size L; `marley_browser` (the inspect calls, the bundle's
  reads), `marley_workbench` (pick mode, the tray, Send, the tools), `marley_mcp` (two rows).
  No Zed path expected.
- **Recall (§18.3):**
  - #492's snapshot refs and isolated world; #493's `tab` argument; #495's rule that a list
    opens only for the user, which pick mode shares (only the user turns it on).
  - `browser-handoff.md` open decision 3 ("Picks sent to the agent at once, or staged for Chad
    to confirm and caption first. The intake leans toward staged"): D1 takes the lean, under
    Chad's "do your best for decisions".
  - The measured cost of snapshots (about 3,400 tokens for an interactive list): a pick's
    bundle stays small and names one element.
- **The probe (2026-09-25):** see the spec's prior art.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** wave 2 queued (9b8f459a6e); no other active pipeline; cargo idle.
- **Recall:** as the draft's, and the brain (consultation `bffa9a7903b24036832132e8746faaaa`):
  nothing on this seam.
- **Seams re-verified:** #480's attach reaches a terminal through `blocks::focused_terminal`,
  which only finds a terminal that has the focus now, and `TerminalView::add_paths_to_terminal`
  pastes with `Terminal::paste` (bracketed when the program asks); nothing in Marley tracks the
  terminal used last, and Zed's pane activation history counts tab switches, not focus. The
  icon `Crosshair` exists. The CDP calls are the probe's.

### Design
- **`marley_browser::pick` (new):** `Page::set_inspect(on)` (`DOM.enable`, `Overlay.enable`,
  `Overlay.setInspectMode` with a highlight and its tooltip, or `none`); `Page::watch_scripts`
  (`Debugger.enable`, then `Debugger.setSkipAllPauses`); `Page::capture_pick(backend_node_id,
  scripts)` builds `PickBundle`: the node resolved (`DOM.resolveNode`) and walked up to its
  nearest interactive ancestor, through open shadow roots, by a read-only function
  (`Runtime.callFunctionOn`); that function's report (tag, the locators ranked with whether each
  is unique, the text, the blockers: `pointer-events`, `visibility`, `display`, `opacity`,
  `disabled`, and the element on top at its middle; the box in the page, through same-site
  frames); role and name (`Accessibility.getPartialAXTree` on the object); the listeners of the
  element and its ancestors (`DOMDebugger.getEventListeners` on each, their objects from one
  `Runtime.getProperties`), each with its type, script URL, line and column from the scripts
  map, capped; and a crop (`Page.captureScreenshot` with a clip around the box). A summary
  (`button "Go"`) for the tray and the terminal line.
- **The hub:** a page's `picking` flag and its scripts map (`Debugger.scriptParsed`: id to URL
  and source map URL); `set_picking(target, on)`; `Overlay.inspectNodeRequested` ends pick mode
  and captures the pick in a task; `Overlay.inspectModeCanceled` ends it. Picks are the app's,
  numbered from 1: `Pick { id, target, url, title, summary, caption, sent, bundle, crop }`;
  `BrowserEvent::PickStaged`, `picks`, `pick(id)`, `send_pick`, `discard_pick`.
- **The last terminal:** a global the workbench keeps, set when a `TerminalView` gains the focus
  (`observe_new` then `on_focus_in`, detached), with the view and its window.
- **The tab:** the toolbar's pick button (`Crosshair`, toggled while picking) and
  `marley::PickElement` (Ctrl+Shift+C in `MarleyBrowser`); Escape while picking ends pick mode
  before any key reaches the page. The tray, a bar under the toolbar while the page has picks,
  newest first: `Pick N · summary`, a caption field, Send and Discard, or "Sent". Send pastes
  `[browser pick N: summary on host/path; browser_pick id N] caption` into the last terminal,
  activates its tab and focuses it.
- **For agents:** `browser_picks` and `browser_pick {id}`, read tools; the latter answers with
  the bundle and the crop as an image.
- **Manifest:** `crates/marley_browser/src/pick.rs` (new), `marley_browser.rs`;
  `crates/marley_mcp/src/registry.rs`; `crates/marley_workbench/src/browser.rs`,
  `src/browser_tools.rs`, `src/marley_workbench.rs` (the action), `keymap.json`;
  `script/e2e/496-element-picker.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | the pick button, the pointer over the button | `496-01-hover` |
| REQ-002 | a click on the button's label (the text walks up to the button) | `496-02-staged` |
| REQ-003 | a caption, Send | `496-03-sent`: the terminal with the line |
| REQ-004 | the stand-in agent's `browser_picks`, `browser_pick` | the run log |
| REQ-005 | pick mode, Escape | `496-04-cancelled` |
| (regression) | #492's and #493's scenarios | their shots |

### Risks
- A page that swaps its DOM at a click can leave the pick's element gone before the bundle is
  read; the capture reads everything in one pass right after the pick.
- Inspect mode in a cross-site iframe needs the iframe's session; out of scope.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (the probe), spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `marley_browser::pick` (new): `Page::set_inspect`, `Page::watch_scripts`,
    `Page::capture_pick` (the node walked up to its nearest interactive ancestor; `DESCRIBE`'s
    tag, text, ranked locators with uniqueness, box in the page and blockers; role and name from
    `Accessibility.getPartialAXTree`; the listeners of the element, its ancestors, the document
    and the window, capped at 24, their scripts from the page's map), `Page::crop`,
    `PickBundle::summary`.
  - The hub: a page's `picking` and `scripts` (`Option`, filled by `Debugger.scriptParsed` once
    the first pick mode turned the Debugger domain on); `set_picking`; the
    `Overlay.inspectNodeRequested`, `Overlay.inspectModeCanceled` and `Debugger.scriptParsed`
    arms; `Pick` and the session's picks, numbered from 1; `BrowserEvent::PickStaged`;
    `picks`, `pick`, `pick_sent`, `discard_pick`; a page's `pick_error`.
  - The tab: the toolbar's Crosshair button (lit while picking), `marley::PickElement`
    (Ctrl+Shift+C), Escape in pick mode, the tray (newest first: `Pick N`, the summary, a
    caption field focused at the pick, Send and Discard, or what was sent), `marley::SendPick`
    on Enter in a caption.
  - Send: `LastTerminal`, kept by `observe_new::<TerminalView>` and `on_focus_in`;
    `reveal_terminal` (a center pane or the Terminal Panel, of any workspace in the window);
    `pick_line`; the paste with `Terminal::paste`.
  - `browser_picks` and `browser_pick {id}` in the registry and the tools, answered before the
    browser needs to show; the crop is the answer's image.
- **Deviations from the design:**
  - `"escape": null` in `MarleyBrowser`: Zed binds Escape to `workspace::Unfollow` in
    `Workspace`, and the binding ran before the tab's key listener, so Escape never reached a
    page (a gap since #489) and could not end pick mode. The null binding lets it through; the
    address bar's, the dialog's and a select list's own Escape bindings sit deeper and still
    win.
  - The scripts map is an `Option` rather than a map beside a bool (clippy's
    `struct_excessive_bools`); it also marks that the Debugger domain is on.
  - A sent pick's Discard takes it out of the tray and keeps it for the agent, whose line names
    it; an unsent pick's Discard drops it.
  - The tray lists the picks of its own page; `browser_picks` lists the session's.
  - Send with no terminal used yet says so in the tray ("No terminal to send to: click in one,
    then Send.") rather than guessing one.
- **Review against the criteria:**
  - REQ-001, REQ-005: Chromium draws the highlight; Escape and the button both end the mode.
  - REQ-002: the walk to the interactive ancestor runs in the page on the node Chromium names.
  - REQ-003: Send's workspace work runs in `cx.defer`: the terminal's pane may hold this very
    tab, and activating the terminal would update the tab inside its own update.
  - REQ-004: the tool answers from the hub alone, so a pick outlives its page and a restart.
  - Found and fixed: `DESCRIBE` read `element.value` for any element with no text, which for a
    password field is the password; it now reads a value only as an `<input>` button's label.
    Scripts with no URL are no longer kept. Errors reach the tray (a pick that did not read, a
    Send with no terminal), with Dismiss.
- **Checks:** `cargo clippy -p marley_browser -p marley_mcp -p marley_workbench --all-targets
  -- -D warnings` clean; `cargo fmt` clean.
- **Checklist (no TaskCreate in this harness):** pick.rs ✓, hub ✓, tab ✓, Send ✓, actions and
  keys ✓, registry ✓, tools ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/496-element-picker.sh` (`compositor sway`, offline Chromium): a
  loopback `card.html` with a card holding a Save button (`data-testid`, its label in a span, a
  click listener in `app.js`), and a Covered button whose middle sits under a transparent
  `div#veil`; the terminal's HOME has a `.bashrc` of its own (`PS1='$ '`). The terminal is
  clicked first; then the Browser tab, the pick button, the pointer over the label, a click, a
  caption and Enter, the stand-in agent's `browser_picks` and `browser_pick`, the Card tab
  again, Ctrl+Shift+C and a click on the Covered button's uncovered edge, Ctrl+Shift+C from the
  new pick's caption field, Escape, a click on Save. `browser-fixture.sh`'s stand-in agent gained
  `picks` and `pick <id> [<image file>]`.
- **Shots (in the scratchpad, `e2e-496/`), each read:**
  - `496-01-hover` (REQ-001): the toolbar's crosshair lit; Chromium's inspect highlight on
    `span.label` inside the Save button, with its tooltip (`span.label 114.09 × 20`, its
    accessibility name and role).
  - `496-02-staged` (REQ-002): the tray under the toolbar, `Pick 1 button “Save changes”`, the
    caption field, Send and Discard; the crosshair unlit; the page moved down by the row; the
    click did not reach the page (its log is empty).
  - `496-03-sent` (REQ-003): the terminal's tab in front, and at its prompt
    `[browser pick 1: button “Save changes” on 127.0.0.1:41653/card.html; browser_pick id 1] Make
    this button green` (bash marks the pasted region).
  - `496-04a-picking`: Ctrl+Shift+C pressed in Pick 2's caption field lit the crosshair; the
    tray lists Pick 2 staged over Pick 1 `Sent: Make this button green`, rows of one height.
  - `496-04-cancelled` (REQ-005): after Escape the crosshair is unlit, the click on Save reached
    the page (`saved` in its log), and the tray still lists two picks.
  - The crops `496-pick-1.jpg` and `496-pick-2.jpg`: each button with its margin, no inspect
    highlight in them.
- **The run log (REQ-004):** `browser_pick 1`: `tag button, role button, name 'Save changes'`;
  locators `[data-testid="save-button"]` (unique), `#save` (unique), the text, `#save`; the
  listener `click on button#save: http://127.0.0.1:41653/app.js:1:58` (line 1 from 0 is the
  file's second, the handler at column 58); `blockers: none`; the box `61,111 200x44`; the crop
  saved. `browser_pick 2`: `button “Covered”`, `listener click on button#covered: …app.js:2:61`,
  `blockers: covered by div#veil`, the box `40,240 200x44`. `browser_picks` lists both, the
  first `sent with 'Make this button green'`, the second `not sent`. `tools` lists 16, the pick
  tools among them, none that evaluates script.
- **What the first run found, fixed at the source:**
  - Ctrl+Shift+C pressed in a pick's caption field opened Zed's collab panel: its binding's
    context, `!Terminal`, matches at the deepest context, the field's, so it outranked
    `MarleyBrowser`'s binding. The key is now bound in `MarleyBrowser > Editor` too, as deep as
    the field, where Marley's keymap wins the tie.
  - A sent row was shorter than a staged one, so the page moved by uneven amounts; every row is
    now one height (`h_9`).
  - The scenario clicked above the Save button; its points now come from the first shot.
- **Regressions:** `492-browser-tools.sh` (its three shots as before; the pick button sits
  between the address bar and the Agent chip) and `493-browser-tabs.sh` (its seven shots as
  before) both pass.
- **Focus report:** every run was in the headless sway; the runner printed "hyprland: 0 Marley
  windows before the run, 0 after; the run added no rule and did not reload it".
- **Marley's log:** no panic or pick error; the one warning, "The browser closed its
  connection", comes at teardown, as in #495's run.
- **Gate:** `just gate-diff` — 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches
  the tree.
- **Pre-existing, not in scope:** none.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  REQ-005 ✓, 492 ✓, 493 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#496 under Added); `docs/marley_architecture/marley_browser.md`
  (Picking), `marley_workbench.md` (the pick tools, the Browser tab's Picks, two known limits),
  `marley_mcp.md` (the seven read tools, `browser_pick`'s schema); the plan's B3a row shipped and
  open decision 6. No path outside the Marley-owned set changed, so no ledger row.
- **Knowledge appended:** `F-claude-496-ctrl-shift-c-in-a-tab-field-opened-the-collab-panel-001`,
  `F-claude-496-escape-never-reached-a-browser-page-001`,
  `F-claude-496-the-picks-text-read-a-fields-value-001`,
  `PR-claude-a-views-key-needs-its-fields-context-too-001`,
  `L-claude-496-chromium-picks-an-element-over-cdp-001`,
  `AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001`.
- **Brain:** consultation `bffa9a7903b24036832132e8746faaaa` closed with the decision
  `decisions/marleys-element-picks-are-staged-and-sent-to-the-last-terminal`.
- **Ticket:** closed; the BACKLOG row went at promotion.
