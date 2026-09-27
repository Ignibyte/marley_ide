# B1d: Marley draws the page's select lists — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-495-select-picker.md
- **Pipeline spec:** 495-select-picker.spec.md

## Phase 1 — Plan (drafted 2026-09-25, split from #493)
- **Request:** the select half of the queued #493, split so each ticket is one slice.
- **Recall (§18.3):** the probe (no popup in a headless frame); #489's press path and
  `last_press`; #492's isolated world helper.
- **Checklist (no TaskCreate in this harness):** mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #494 committed (1bb52d739a); no other active pipeline; cargo idle.
- **Recall (§18.3):** nothing in the ledgers on select popups. #489's press path in
  `PageElement` and the view's `last_press`; #492's isolated world (`Page::isolated_context`)
  and `observe`, which each cross-site iframe's session gets too; #493's `BrowserEvent` per
  page. The brain (consultation `8368040d0c9249d6b2e081d5267accd5`): nothing on this seam.
- **The probe (2026-09-25):** see the spec's prior art. It settled D3: the queued plan's hit
  test would have put `DOM.getNodeForLocation` and `DOM.describeNode` before every press in the
  page.
- **Seams re-verified:** `ui::ContextMenu` (`header`, `toggleable_entry_disabled_when`,
  `select_toggled_or_first`, `DismissEvent`, arrows, Enter and Escape through `menu::`
  actions, cancel on blur), drawn at a point with `deferred(anchored().position(…))` as the
  editor's mouse context menu is; `Page::attach` and `observe`; the hub's `follow_observed`.

### Design
- **`marley_browser::select` (new):** the listener (a script, idempotent per world), the world
  name and the binding's; `SelectRequest`, the binding's payload (the select's box in its
  frame, the press point in the frame or none for a key, the selected index, the options with
  text, disabled and group); the choice's expression. `Page::watch_selects(session)`:
  `Runtime.addBinding` for the world, `Page.addScriptToEvaluateOnNewDocument` into it, and for
  each frame of the session's frame tree `Page.createIsolatedWorld` and the listener evaluated
  there. `Page::choose_option(session, context, index)`. `Page::attach` watches the page's
  session; the hub watches each cross-site iframe's as it observes it.
- **The hub:** `Runtime.bindingCalled` with the binding's name, from a session the hub knows,
  becomes the page's `select` (the session, the context and the request) and
  `BrowserEvent::SelectOpened { target }`; `choose_option(target, index)` and
  `dismiss_select(target)`.
- **The tab:** on `SelectOpened` for its page, when its page has the focus or the user pressed in
  it within the last second, it builds a `ContextMenu` (a header per group, a checked entry for
  the selected option, disabled ones greyed) and draws it with `deferred(anchored())` under the
  select: the list's corner in the page is the select's bottom left, moved by the difference
  between the press in the page and in the frame (so it lands right in any frame), or, for a
  key, the box as the main frame's. The list takes the focus and starts on the current option;
  a choice calls `choose_option`, a dismissal `dismiss_select`, and either gives the page the
  focus back. The tab keeps the frame's mapping from its last paint for the arithmetic.
- **Manifest:** `crates/marley_browser/src/select.rs` (new) and `marley_browser.rs` (the
  module), `src/page.rs`; `crates/marley_workbench/src/browser.rs`;
  `script/e2e/495-select-picker.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot |
|---|---|---|
| REQ-001 | a click on the select | `495-01-open`: the list under the select, "Pear" checked, "Lemon" greyed, the "Sweet" header |
| REQ-002 | a click on "Plum" | `495-02-chosen`: the select reads Plum; the page's log `input plum`, `change plum` |
| REQ-003 | a click on the select, then Escape | `495-03-escaped`: no list; still Plum; no new events |
| REQ-004 | Alt+Down on the focused select, ArrowUp, Enter | `495-04-keys`: the value the keys chose; its events |
| (regression) | #489's and #493's scenarios (the press path, the tabs) | their shots |

### Risks
- The choice's events are not trusted; a page that checks `isTrusted` on `change` ignores it.
- A select in a frame whose position the press cannot give (a key opening in a frame) opens its
  list where the main frame would have the select.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (and the probes), spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built.**
  - `marley_browser::select` (new): `WORLD`, `BINDING`, `LISTENER` (the `mousedown` and
    opening-key listener, idempotent per world, with `__marleyChoose`), `SelectRequest` and
    `SelectOption` (the report), `Page::watch_selects(session)` and
    `Page::choose_option(session, context, index)`.
  - The hub: a page's `select` (`SelectState`: the session, the listener's context, the
    request), set from `Runtime.bindingCalled` with the binding's name, which also emits
    `BrowserEvent::SelectOpened`; `choose_option` and `dismiss_select`. A page's session is
    watched once the page is attached, and each cross-site iframe's once it is observed.
  - The tab: on `SelectOpened` for its page, and only when the user pressed in the page within a
    second or the page has the focus, a `ContextMenu` with a header per group (a separator
    after one), a checked entry for the selected option, disabled entries greyed; it starts on
    the current option, takes the focus, and is drawn with `deferred(anchored())` at the select's
    bottom left, found from the frame's mapping of the last paint and, for a press, the
    difference between the press in the page and in its frame. A choice calls the hub's
    `choose_option`, a dismissal its `dismiss_select`, and either gives the page the focus back.
- **Deviations from the design:** none.
- **Review of the diff.** Re-entrancy: the list opens from the hub's event, after the hub's
  update; an entry's handler updates the tab from inside the menu's update, and the tab drops
  the menu and its subscription there, which Zed's editor also does with its context menu. An
  agent's click opens nothing (D4), and a select in a background tab opens nothing. The
  owned-string lint flags literals only; the labels are the page's.
- **Checks:** `cargo check`, `cargo fmt`, `cargo clippy -p marley_browser -p marley_workbench
  --all-targets -- -D warnings` clean.
- **Checklist (no TaskCreate in this harness):** the select module ✓, the hub ✓, the tab ✓,
  review ✓.

Phase 2 PASS. Next: `/pipeline:test`.

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/495-select-picker.sh`, in the headless sway with an offline
  Chromium. A loopback page with a select in two groups ("Sweet": Apple, Pear, selected;
  "Sour": Lemon, disabled, and Lime) and Plum after them, which prints its value and each
  `input` and `change`; a cross-site iframe below it with a select of its own; the stand-in MCP
  agent for the last step.
- **Shots** (the third run):
  - `495-01-open` (REQ-001): Marley's list right under the select at its left edge: "Sweet",
    Apple, Pear checked and highlighted, "Sour", Lemon greyed, Lime, a separator, Plum.
  - `495-02-chosen` (REQ-002): a click on Plum: the select reads Plum, "The value: plum", and the
    page's log `input plum`, `change plum`.
  - `495-03-escaped` (REQ-003): the list opened again and Escape: no list, still Plum, no new
    line in the log.
  - `495-04a-keys-open` and `495-04-keys` (REQ-004): Alt+Down on the focused select opened the
    list on Plum; ArrowUp went to Lime past the separator and Enter chose it: "The value: lime",
    `input lime`, `change lime`.
  - `495-05-frame` (D2): a click on the cross-site iframe's select opened its list (Small,
    Medium checked, Large) right under that select, placed by the press's offset in its frame.
  - `495-06-agent` (D4): the stand-in agent's `browser_click` on the select ("e1: clicked
    combobox") opened no list; the select took the focus and the log has no new line.
- **Fixes found here.** The first run's click on Plum missed: the estimate was 25 pixels low;
  the scenario takes its place from `495-01-open` now. The review of D4 against the second run
  found the list would also open for an agent's click while the user's focus sat in the page;
  the tab now opens it only within a second of the user's own press or key in the page.
- **Regressions:** #489 (every shot as at #489, the cross-site frame's field included; latency
  median 22.5 ms, 95th percentile 34.3 ms), #492 (every criterion), #493 (the seven shots as at
  #493).
- **Focus report:** every run in its own headless sway, each ending "hyprland: 0 Marley windows
  before the run, 0 after; the run added no rule and did not reload it".
- **Gate:** `just gate-diff` on `marley_browser` and `marley_workbench`: all fifteen gates PASS,
  `GATE GREEN [diff]`, the receipt matching the tree. No pre-existing failure.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  D2's frame ✓, D4's agent ✓, the regressions ✓, the gate ✓.

Phase 3 PASS. Next: `/pipeline:complete`.

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` ("Select lists in the Browser tab"); `marley_browser.md` (the
  select module and its two page calls); `marley_workbench.md` (the hub's select state and
  event, the tab's list, its placement and the user-only rule); the plan's B1d row shipped, and
  the risk line on browser UI. No Zed path touched.
- **Knowledge appended:** F-claude-495-the-list-opened-for-an-agents-click-in-a-focused-tab-001,
  L-claude-495-a-listener-and-a-binding-catch-what-headless-chromium-hides-001,
  L-claude-495-a-menu-at-a-point-in-a-view-001, AD-claude-495-marley-draws-the-pages-select-lists-001.
- **Brain:** consultation `8368040d0c9249d6b2e081d5267accd5` closed with `brain decide`:
  `decisions/marley-draws-the-pages-select-lists-caught-by-a-listener-in-an-isolated-world`, a
  follow-up by 2026-10-09.
- **Closed:** the ticket in `tickets/closed/`; BACKLOG holds no #495 row. Wave 1 of prong 3
  (#487 to #495) is complete.
