---
pipeline_id: a8f7f9d7-698c-48f7-b8ae-cf480baf7bc0
ticket: docs/planning/tickets/open/TICKET-756-the-browser-tools-agents-lack.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The browser tools agents lack
type: feature
slice: prong 3 (the Browser tab), the first of three slices; T3 Code survey item 10, Orca report 03 item 11
references:
  - docs/t3code_architecture/05-terminal-browser-and-capture.md
  - docs/orca_architecture/03-browser-and-design-mode.md
  - docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md
  - docs/planning/pipeline/completed/571-pause-before-a-consequential-click.spec.md
---

## Title
The acting tools an agent needs to test a web app in a Browser tab: hover, pick an option, drag,
upload a file, answer a dialog, resize to a device and switch the colour scheme, plus a wait that
acts on nothing. Chad, 2026-10-11: "18.) build now".

## Scope
### In
- **Seven write tools** (`Tier::Write`, grant `browser.write`, in `browser_tools::WRITES`, so each
  brings its tab forward, shows on the Agent chip and is refused while a click is paused there):
  - `browser_hover {tab?, ref? | x, y}`: the pointer to the element's centre or the point; the
    answer names what is under it.
  - `browser_select {tab?, ref, values? | labels?}`: chooses options of the `<select>` the ref
    names, by value or by visible label, several for a `multiple` list; the page gets `input` and
    `change` as from a person's choice, and Marley's own list stays shut. An option not there is
    refused `no_option` with the options there (50 at most).
  - `browser_drag {tab?, from_ref | from_x, from_y, to_ref | to_x, to_y}`: press at the start,
    move in steps, release at the end. A drag the page starts with HTML's drag and drop is
    intercepted and finished with drag events at the end point.
  - `browser_upload {tab?, ref, paths}`: sets the files of the `<input type=file>` the ref names.
    Each path is absolute, a file, inside an open project's folders, and not matching
    `marley.agent_control.secret_globs`; else `outside_projects`, `no_file` or `secret_file`. A
    ref that is not a file input is refused `not_a_file_input`.
  - `browser_dialog {tab?, accept, text?}`: answers the tab's open alert, confirm, prompt or
    leave-page dialog as the card's OK or Cancel would, `text` for a prompt; the card closes. No
    dialog open: `no_dialog`.
  - `browser_resize {tab?, preset? | width, height | fill}`: the page's viewport at a device
    preset (a fixed list after Chrome DevTools', with its scale and the mobile flag), at a size
    (320 to 3840 by 240 to 2160), or back to the tab's own size. The user agent stays.
  - `browser_color_scheme {tab?, scheme}`: `prefers-color-scheme` forced to `dark` or `light`,
    or `system` to stop forcing it.
- **One read tool**, `browser_wait {tab?, text?, text_gone?, role?, name?, url?, timeout_ms?}`
  (`Tier::Read`): waits until every condition given holds, looking every 250 ms, for up to
  `timeout_ms` (10,000 by default, 20,000 at most). Text is read through a fixed function in the
  page's isolated world, never an agent's script; an element by role and name through
  `Accessibility.queryAXTree`; the URL from the tab. It answers what held and how long it took,
  or refuses `not_met` with what did not hold and the next step (a snapshot).
- **What the tab shows:**
  - A page at a preset or a size is drawn at that size, scaled down to fit when larger than the
    tab and centred. The tab's own resizing no longer resets it until Fill.
  - While a page runs at a preset, a size or a forced scheme, a chip in the toolbar names it
    ("iPhone 15 Pro · 393×852 · dark") with Reset, which puts the tab back to its size and the
    system's scheme.
  - `browser_tabs` and `browser_look` report an open dialog (its kind, message redacted, the
    site), the viewport's preset and the forced scheme.
- **Docs:** the guide's browser tool tables, "What agents can do in the tab", "Select lists and
  dialogs" (an agent may now answer a dialog), "Limits of the Browser tab"; the HTML guide's tool
  list; `docs/marley_architecture/marley_mcp.md`.

### Out (explicitly deferred)
- **The second slice, MCP annotations** on every Marley tool and on the IDE server's five, and
  the guide's missing tool rows. Drafted in the notes; it needs a number.
- **The third slice, file choosers, downloads and Take Over** for the Browser tab, and
  `browser_upload` answering an open file chooser. Drafted in the notes; it needs a number.
- Tools from Orca's list this ticket does not take: check and uncheck (a click does it), forward
  and reload, full-page screenshots, PDF, cookies and storage, request interception, reduced
  motion, geolocation.
- `browser_evaluate`: Marley runs no script an agent supplies (T3 survey 05 open question 2,
  default kept).
- Outside clients (#524): their tool lists stay as `524-…` and `584-…` pin them.
- A device preset that survives a restart: the tab restores at its own size.

## Reference (§20)
N/A — Marley-specific: the Browser tab is Marley's own headless Chromium, drawn in a Zed pane,
with no Warp or Zed analog. The tool shapes follow T3 Code's preview tools as the survey read
them (report 05 §2.5: `preview_hover` 184, `preview_select` 195, `preview_drag` 209,
`preview_upload` 220, `preview_dialog` 86, `preview_resize` 108, `preview_set_appearance` 121,
`preview_wait_for` 276 in `apps/server/src/mcp/toolkits/preview/tools.ts`) and Orca's list
(report 03 §2.11, item 11). The presets follow Chrome DevTools' device list.

### Prior art
- **The code we ship:**
  - `crates/marley_workbench/src/browser_tools.rs` (1,796 lines): `answer` 91-114, `run` 259-318,
    `WRITES` 67-74, `not_paused` 116-123, `page_of` 412-453, `click` 1333-1374, `type` 1376-1426,
    `ref_target` 1529-1537, `place` 1597-1610, `ref_origin` 1644-1679, `click_at` 1761-1786,
    `acting` 1230-1250 (the Agent chip).
  - `crates/marley_browser`: `cdp.rs` `Connection::call` 256-295 (15 s timeout); `page.rs`
    `set_viewport` 536-552 (`Emulation.setDeviceMetricsOverride`, `mobile: false` fixed),
    `answer_dialog` 718-730 (`Page.handleJavaScriptDialog`), `node_facts` 1069-1162
    (`DOM.resolveNode` into an isolated world, then a fixed `Runtime.callFunctionOn`, the pattern
    for select and wait); `select.rs` 17-199 (Marley's select list, which an agent's click keeps
    shut); `input.rs` `mouse_event` 306.
  - `crates/marley_workbench/src/browser.rs`: dialogs (`dialog_opened` 1702-1719,
    `answer_dialog` 1667-1686, `render_dialog` 6229-6305), the viewport set on every paint
    (`PageElement::prepaint` 7211-7226 → `lay_out` 2853-2875), `PageMapping` 7271-7319 (clicks
    mapped through the frame's metadata), the toolbar 5914-6020 and the Agent chip 5987-6007.
  - `click_pause.rs` (#571): the pause the new write tools respect through `not_paused`.
  - No crate we build drives a browser beyond our CDP client: Chromium's own DevTools protocol is
    the seam.
- **Behavior maps:** T3 survey 05 §2.4 to §2.6 and §3 items 2 to 4; Orca report 03 §2.11, §2.12
  and §3 item 11.
- **Published material:** the Chrome DevTools Protocol: `Input.dispatchMouseEvent`,
  `Input.setInterceptDrags`, `Input.dragIntercepted`, `Input.dispatchDragEvent`,
  `DOM.setFileInputFiles`, `Page.handleJavaScriptDialog`, `Emulation.setDeviceMetricsOverride`,
  `Emulation.setEmulatedMedia`, `Accessibility.queryAXTree`; MCP 2025-06-18.

## UI proof
`script/e2e/756-the-browser-tools-agents-lack.sh`, under `compositor sway`, with the browser
fixture (`offline_chromium`, `serve_site`, `mcp_agent` through the plugin's bridge, its generic
`tool <name> <json>`). The fixture page `lab/index.html` holds a button that reads "hovered" while
the pointer is on it, a colour `<select>` whose choice shows beside it, two lists of draggable
cards whose order shows below, a file input that lists the chosen names, Alert, Confirm and Prompt
buttons whose outcome shows, a Start button after which "Ready" appears two seconds later, the
viewport's size, and a block that reads "dark" or "light" from `prefers-color-scheme`. The
project folder holds `notes.txt` to upload.

Shots:
- `756-01-hover`: the button reads "hovered"; the Agent chip names the hover.
- `756-02-select`: "blue" chosen and shown; no Marley list open.
- `756-03-drag`: the card moved to the second list, the order line changed.
- `756-04-upload`: the input lists `notes.txt`.
- `756-05-dialog`: Confirm pressed by the agent; the card shown, then answered by
  `browser_dialog`: the outcome reads "confirmed", the card gone.
- `756-06-wait`: "Ready" on the page after `browser_wait` returned.
- `756-07-resize`: the page at iPhone 15 Pro, centred in the tab, its size line 393×852, the chip
  in the toolbar.
- `756-08-scheme`: the scheme block reads "dark", the chip names it.
- `756-09-reset`: after Reset, the page at the tab's size and the block "light".

## Locked-In Decisions
- **D1:** this ticket is the tools; annotations and choosers, downloads and Take Over follow as
  two more tickets (the notes' drafts). Rejected: one ticket for all of item 10 (about three
  times this one, touching every tool's registry row and two CDP domains the tab does not use).
- **D2:** `browser_wait` is a read tool, needing no grant, since it changes nothing on the page;
  the seven that act take `browser.write`. Rejected: putting the wait under `browser.write` too.
- **D3:** waits name what an agent can know before the element exists: text, role and name, a
  URL. Rejected: a ref (it names a node already in a snapshot); CSS selectors (Marley's tools
  target by refs and accessibility, not selectors).
- **D4:** an agent may answer a dialog. The click that opened a confirm has passed the click
  consequence already (#571), so the answer adds no second pause. This ends "Marley's dialog card
  is answered by you only".
- **D5:** an upload takes files from the open projects' folders only and never one a secret glob
  matches. Rejected: any path (a page could take `~/.ssh` through an agent).
- **D6:** `browser_select` sets the options through a fixed function on the node the ref names
  and fires `input` and `change`, rather than opening Marley's list and clicking in it, so the
  list keeps its rule that an agent's click never opens it (#495).
- **D7:** a preset or size is kept in the page's state, beside the viewport the tab sets, so the
  tab's next paint does not undo it; Fill or Reset clears it. The user agent never changes
  (T3's rule).
- **D8:** waits stay inside the server's 30 seconds (L-claude-682): 20 seconds at most.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `browser_hover` on a ref, the system shall move the page's pointer onto that element and name the action on the Agent chip. | Shot 756-01 |
| REQ-002 | WHEN an agent calls `browser_select` with a value or label, the system shall choose that option, fire the page's change, and open no Marley list. | Shot 756-02 |
| REQ-003 | WHEN an agent calls `browser_drag` from one element to another, the system shall complete the page's drag and drop there. | Shot 756-03 |
| REQ-004 | WHEN an agent calls `browser_upload` with a file inside an open project, the system shall set it on the file input. | Shot 756-04 |
| REQ-005 | IF an upload path lies outside every open project or matches a secret glob, THEN the system shall refuse with `outside_projects` or `secret_file` and set nothing. | The client's reply (checked) |
| REQ-006 | WHEN an agent calls `browser_dialog` while its tab shows a dialog, the system shall answer it and close the card. | Shot 756-05 |
| REQ-007 | WHEN an agent calls `browser_wait` for text that appears within its timeout, the system shall answer once the text is on the page. | Shot 756-06 |
| REQ-008 | IF a wait's conditions do not hold within its timeout, THEN the system shall refuse with `not_met` and what did not hold. | The client's reply (checked) |
| REQ-009 | WHEN an agent calls `browser_resize` with a preset, the system shall draw the page at that size in the tab and show the chip. | Shot 756-07 |
| REQ-010 | WHEN an agent calls `browser_color_scheme` with `dark`, the system shall make the page's `prefers-color-scheme` dark and name it on the chip. | Shot 756-08 |
| REQ-011 | WHEN the user presses the chip's Reset, the system shall put the page back to the tab's size and the system's scheme. | Shot 756-09 |
| REQ-012 | `tools/list` shall list the eight tools with input and output schemas. | The client's `tools` reply (checked) |
| REQ-013 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam checked
  again.
- **P2 Code** — `registry.rs` (eight rows, schemas, the pinned list), `browser_tools.rs` (the
  tools, `WRITES`), `marley_browser` (`page.rs`: emulation, hover, drag, file input, the select
  and wait functions), `browser.rs` (the kept viewport, the scaled frame, the chip, dialog and
  emulation in `browser_tabs`/`look`), the guide and the HTML guide; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
