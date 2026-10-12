# The browser tools agents lack — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-756-the-browser-tools-agents-lack.md
- **Pipeline spec:** 756-the-browser-tools-agents-lack.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** the T3 Code survey's item 10, "The browser tools Marley's agents still lack", which
  is report 05's items 2 to 4 and Orca report 03's item 11. Chad, 2026-10-11: "18.) build now".
  The item covers the acting tools, MCP annotations on every tool, file choosers and downloads,
  and a Take Over key; this ticket is its first slice (D1), and the other two are drafted below.
- **Classification:** feature, prong 3. Marley crates only.
- **Recall (§18.3):**
  - #492 (completed): the browser family, refs from snapshots, input as a person's events.
  - #490: the dialog card (`AlertModal`), and navigation answering an open dialog with Cancel.
  - #495: Marley's select list; an agent's click keeps it shut (`USER_PRESS`, 1 s). Its scenario
    checks that an agent's click opens no list (`495-select-picker.sh:123-126`).
  - #571: the click consequence, `WRITES` and `not_paused`.
  - #574: tools act in the caller's project; #507: one Chromium profile per project.
  - L-claude-682: tools answer inside the server's 30 seconds.
  - L-claude-649: a refusal's reason does not open with the tool's name.
  - #680 deferred codes for the browser family's refusals; this ticket's new tools carry theirs
    (`no_option`, `not_a_file_input`, `no_dialog`, `not_met`, `outside_projects`, `no_file`,
    `secret_file`), and the old tools keep `refused`.
- **Discovery (Explore, 2026-10-11):**
  - `crates/marley_mcp/src/registry.rs`: `Family` 15-45, `ToolSpec` 93-104, `REGISTRY` 125-626
    (browser rows 234-382), `browser_read`/`browser_write` 629-649, `tools_list_for` 691-709,
    `browser_schemas` 774-791, `browser_arguments` 829-842, `browser_write_schemas` 1452-1532,
    the pinned list and count (54) 2565-2629.
  - `crates/marley_workbench/src/mcp.rs:106-120` grants `browser.write`; 516-536 routes
    `browser_` names.
  - `crates/marley_workbench/src/clients.rs:12-35`: outside clients' tool lists, unchanged here.
  - `crates/marley_browser/src/relay.rs:712-716, 800-832`: root events without a session are
    dropped and browser-session events lose their `sessionId` (matters for slice 3's downloads,
    not this one).
  - `crates/marley_workbench/keymap.json:140-155` (`MarleyBrowser`), 234-258 (dialog and pause).
  - The guide: tool tables 1858-1884, "Select lists and dialogs" 2042-2054, "What agents can do"
    2056-2062, "Limits" 2121-2131. The HTML guide's tool list
    (`crates/marley_workbench/guide/index.html:1669-1681`).

### Design
- **`crates/marley_mcp/src/registry.rs`:** eight `Family::Browser` rows (`hover`, `select`,
  `drag`, `upload`, `dialog`, `resize`, `color_scheme` through `browser_write()`; `wait` through
  `browser_read()`), their schemas in `browser_write_schemas` and `browser_schemas`, the pinned
  list and count (62) updated so the tree builds.
- **`crates/marley_browser/src/page.rs`** (Marley crate), new page methods:
  - `hover_at(point)`: one `mouseMoved`.
  - `drag(from, to)`: `Input.setInterceptDrags {enabled: true}`, press at `from`, moves in ten
    steps; on `Input.dragIntercepted` within the gesture, `Input.dispatchDragEvent` `dragEnter`,
    `dragOver` and `drop` at `to` with the intercepted data; release; interception off.
  - `set_file_input(backend_node_id, paths)`: `DOM.setFileInputFiles`.
  - `select_options(node, values | labels)`: a fixed function through `Runtime.callFunctionOn`
    in an isolated world, as `node_facts` does, that checks the node is a `<select>`, sets each
    option's `selected`, and dispatches bubbling `input` and `change`; it answers the chosen
    options or the options there.
  - `emulate(viewport: Option<Emulated>, scheme: Option<Scheme>)`: `setDeviceMetricsOverride`
    with the preset's size, scale and mobile flag; `Emulation.setEmulatedMedia` with
    `prefers-color-scheme`.
  - `page_text_has(text)`: a fixed function reading `document.body.innerText`;
    `ax_has(role, name)`: `Accessibility.queryAXTree` on the document.
- **`crates/marley_browser`, new `devices.rs`:** the preset table (name, width, height, scale,
  mobile), about a dozen phones, tablets and desktops after Chrome DevTools' list.
- **`crates/marley_workbench/src/browser_tools.rs`:** the eight tools in `run`; the seven writes in
  `WRITES`; `browser_upload`'s path checks (absolute, a file, inside a visible worktree of an
  open project as `editor_tools::project_holding` checks, `not_secret`); `browser_dialog` through
  `hub.answer_dialog`; `browser_wait`'s loop on the background executor's timer.
- **`crates/marley_workbench/src/browser.rs`:**
  - `PageState` gains `emulated: Option<Emulated>` and `scheme: Option<Scheme>`; `lay_out` sends
    the tab's own size only while `emulated` is `None`.
  - The page element draws an emulated frame at its size, scaled to fit and centred, and
    `PageMapping` maps clicks through the same scale.
  - The toolbar chip with Reset (the `MarleyBrowser` key context gains no key).
  - `browser_tabs`/`browser_look` answers gain `dialog`, `viewport.preset` and `color_scheme`.
- **Docs:** the guide's tables and sections named in the spec; the HTML guide; the architecture
  note `marley_mcp.md` (the tool list) and `marley_browser.md` (emulation, drag interception).
- **File manifest:** `marley_mcp` (`registry.rs`); `marley_browser` (`page.rs`, `devices.rs` new,
  the crate root's module line); `marley_workbench` (`browser_tools.rs`, `browser.rs`); the
  guide, the HTML guide, two architecture notes; the scenario and its fixture page. No Zed path.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/` at `a010517658`; the BACKLOG row removed; the ticket in
  progress. Brain: consultation 3264c06b6b2d4128aea3e20f9ffbac23, nothing on this seam. The two
  drafted slices are tickets now: #766 (annotations) and #767 (choosers, downloads, Take Over).
- **Seams checked again** (an Explore pass). What moved or differs:
  - `registry.rs`: `REGISTRY` 125-666, browser rows 236-384 (a new row goes before the `Ports`
    rows at 385); `browser_read` 669-677, `browser_write` 681-689; `tools_list_for` 731-749;
    `browser_schemas` 814-831; `browser_arguments` 869-882; `browser_write_schemas` 1492-1572; the
    pinning test `registry_is_exactly_the_l1_set_with_correct_tiers` 2748-2826 with **58** tools
    (2817), so 66 after this ticket. Outside clients' counts (`524-…:113,117`, `584-…:241`) must
    not move.
  - `browser_tools.rs`: `run` 265-324, `page_of` 418-459, `click` 1339-1380, `type_text` (was
    `type`) 1382-1432, `ref_target` 1535-1543, `place` 1603-1616, `ref_origin` 1650-1685,
    `click_at` 1767-1792, `acting` 1236-1256, `tabs` 472-518, `look` 520-564 (`Look` 990-998).
  - **Refusals:** the browser tools return `Result<ToolAnswer, String>`, so every one is coded
    `refused`. The new codes need `run` to carry `marley_mcp::Refusal` (`marley_mcp.rs:277-319`,
    `From<String>` gives `refused`), the old arms mapped through it.
  - `page.rs`: `set_viewport` 536-552 (`mobile: false` fixed), `answer_dialog` 718-730,
    `node_facts` 1069-1162 (`isolated_context_in`, `DOM.resolveNode`, `Runtime.callFunctionOn`,
    `Runtime.releaseObject`), `call` 524-528; `input.rs` `mouse_event` 306-324; `cdp.rs` `call`
    256-295 (15 s).
  - `browser.rs`: `PageState` 360-422 (`viewport: Option<Viewport>`, `dialog`), the hub's
    `dialog` 1021, `answer_dialog` 1667-1686, `dialog_opened` 1702-1719, `resize` 2843-2850,
    `lay_out` 2853-2875, the toolbar 5914-6020 (Agent chip 5987-6007), `PageElement::prepaint`
    7211-7227, `PageMapping` 7271-7319 (scale from the frame's metadata).
  - `editor_tools.rs`: `project_holding` 571-592, `not_secret` 197-210 (code `secret_file`).
  - `browser-fixture.sh`: `mcp_agent tool <name> <json>` and `refusal <tool> <json>`; `tab` goes in
    the JSON.
  - The guide's anchors moved by about 250 lines (`### Select lists and dialogs` 2308, `### What
    agents can do in the tab` 2322, `### Limits of the Browser tab` 2387; the acting table
    2138-2148); the HTML guide's browser table 1720-1733.
- **The dialog risk is open:** nothing ties `Page.javascriptDialogOpening` to a pending
  `Input.dispatchMouseEvent`, and no scenario has an agent click open a dialog. Phase 2 checks it
  first.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `mcp_agent tool browser_hover` on the hover button's ref | 756-01-hover |
| 002 | `browser_select` with `labels: ["Blue"]` on the select's ref | 756-02-select |
| 003 | `browser_drag` from card A's ref to the second list's ref | 756-03-drag |
| 004 | `browser_upload` with `$E2E_WORK/repo/notes.txt` (the repo is the open project) | 756-04-upload |
| 005 | `browser_upload` with `/etc/hostname` and with `repo/.env`; the replies checked | — |
| 006 | `browser_click` on Confirm; shot of the card; `browser_dialog {accept: true}` | 756-05-dialog |
| 007 | `browser_click` on Start, then `browser_wait {text: "Ready"}` | 756-06-wait |
| 008 | `browser_wait {text: "Never", timeout_ms: 1000}`; the reply checked | — |
| 009 | `browser_resize {preset: "iPhone 15 Pro"}` | 756-07-resize |
| 010 | `browser_color_scheme {scheme: "dark"}` | 756-08-scheme |
| 011 | Clicks the chip's Reset | 756-09-reset |
| 012 | `mcp_agent tools`; the reply checked for the eight names | — |
| 013 | — | The gate |

### Risks
- **A click that opens a dialog:** a page blocked in `confirm()` may hold the answer to
  `Input.dispatchMouseEvent` until the dialog closes, so the agent's `browser_click` would wait
  out CDP's 15 seconds before it could call `browser_dialog`. The click must answer once the
  dialog opens (race the CDP answer against `DialogOpened`). Check this first in Phase 2.
- **HTML drag and drop in headless Chromium:** synthetic mouse events start a native drag that
  headless Chromium never completes; the interception path is what Playwright's Chromium driver
  uses. A page that drags with pointer events instead (a slider, a sortable list) works through
  the mouse path alone. The fixture uses HTML's `draggable`.
- **The scaled frame:** a phone preset at scale 3 sends large frames; the screencast's size
  follows the viewport, so the frame is scaled down once in the tab.
- **The pinned counts:** the registry test's count changes; the outside clients' counts, pinned
  by `524-…` and `584-…`, do not.

## The other two slices (drafted; each needs a number)

### Slice 2 — MCP annotations on every tool (S)
- **Scope:** `ToolSpec` gains `annotations` (`title`, `read_only`, `destructive`, `idempotent`,
  `open_world`), written into each `tools/list` entry (`tools_list_for`, `registry.rs:700-705`)
  as MCP's `annotations` object. Read-tier tools: `readOnlyHint: true`. Write tools judged one by
  one: destructive for `terminal_type`, `terminal_run`, `editor_edit`, `editor_save`,
  `settings_change`, `keymap_change`, `seat_*`, `thread_post`, `thread_answer`, `action_run`,
  `browser_click`, `browser_type`, `browser_press`, `browser_navigate`, `browser_upload`,
  `browser_dialog`; not for `browser_scroll`, `browser_hover`, `browser_resize`,
  `browser_color_scheme`, `browser_annotate`, `editor_open`; `openWorldHint` for the browser's
  page-changing tools. The IDE server's five tools (`ide.rs:268-311`) read-only. The guide gets
  the rows it lacks (`browser_draft_test`, `browser_check_pick`, `browser_open_url`,
  `terminal_run`) and a key row for the terminal's Ctrl+I.
- **Proof:** N/A — no UI delta: `tools/list` is read by the client and checked; `just shot`.
- **Acceptance:** every listed tool carries annotations; the guide lists every listed tool.

### Slice 3 — File choosers, downloads and Take Over (M)
- **File choosers:** `Page.setInterceptFileChooserDialog {enabled: true}` per page;
  `Page.fileChooserOpened` (its `backendNodeId` and `mode`) opens Zed's path prompt for the person
  (`cx.prompt_for_paths`, multiple when the mode says so) and answers with
  `DOM.setFileInputFiles`. A newer chooser replaces an unanswered one. `browser_tabs` and
  `browser_look` report an open chooser; `browser_upload` without `ref` answers it, with an empty
  list to cancel. T3's rules (`ServerBrowser.ts:849-946`).
- **Downloads:** `Browser.setDownloadBehavior {behavior: allowAndName, downloadPath, eventsEnabled}`
  per project context, into the XDG download folder (T3 survey 05 open question 3, its default);
  `Browser.downloadWillBegin` and `downloadProgress`, which reach Marley without a session
  (`relay.rs:800-832`), handled in `browser.rs`'s `follow`; a toast with Open and Show in Folder
  when one finishes, the file renamed to its suggested name; agents read the tab's downloads from
  `browser_tabs`.
- **Take Over:** a Take Over button beside the Agent chip, and Ctrl-I in the `MarleyBrowser`
  context, which takes the tab only after an agent has acted in it, else reaches the page (a
  page's Ctrl-I is italics in an editor), as the terminal's does (#525 D4, D5). While taken, the
  browser write tools refuse that tab with `taken_over` and "wait until the user hands it back";
  a write queued before a change of control fails with "control changed; take a new snapshot"
  (a generation, as `terminal_drive` keeps). Hand Back gives it back.
- **Proof:** a scenario: a file chooser answered from the path prompt; an agent's
  `browser_upload` answering one; a download's toast and file; Take Over, an agent's refused
  click, Hand Back.
- **Acceptance:** a page's upload control works for a person and an agent; a download lands with
  a toast; a taken tab refuses agents until handed back.

## Phase 2 — Code (2026-10-11)
- **First, the dialog risk:** a probe scenario (an agent's `browser_click` on a button calling
  `confirm()`) on the build before any change: the click failed after 15 s, "Input.dispatchMouseEvent
  got no answer within 15 seconds" (F-claude-756-…). `acting` now races every action against the
  tab showing a dialog.
- **Built:**
  - `marley_mcp::registry`: eight rows after `browser_scroll`, `acting_arguments` (split out of
    `browser_write_schemas` for length), `wait_schemas`, the write output's `dialog`, the pinned
    list at 66.
  - `marley_browser`: `devices.rs` (eleven presets); `page.rs`: `call_on_node` (shared with
    `node_facts`), `select_options` (`SELECT_OPTIONS`), `set_file_input`, `emulate_device`,
    `set_color_scheme`, `page_text` (`PAGE_TEXT`), `has_element`, and `start_screencast(within)`.
  - `marley_workbench`: `browser_actions.rs` (new, the eight tools); `browser_tools.rs` (`run` on
    `Refusal`, `acting` generic with the dialog race, `dialog_json`, `WRITES`, `tabs` and `look`
    extras, helpers made `pub(crate)`); `browser.rs` (`Emulated`, `Scheme`, the page state,
    `emulate`, `fill`, `set_scheme`, `reset_emulation`, `restream`, the drag one-shot and the
    `Input.dragIntercepted` arm, the chip with Reset, the Agent chip moved into
    `render_agent_chip` for length).
- **Deviations from the design:**
  - The frame is fitted by the screencast's `maxWidth` and `maxHeight` rather than scaled in
    paint, and drawn from the tab's top left, not centred: the click mapping and the annotation
    placement scale by the frame's own size, so they need no second mapping (AD-claude-756-…).
  - `browser_dialog` and `browser_resize {fill}` end the chip without `acting`, as their work is
    one hub call with no await.
  - Refusal codes beyond the spec's: `not_a_select`, `not_multiple`, `no_preset`,
    `bad_argument`.
- **The review:** the upload's paths are canonicalised in the background before the project and
  secret checks (a link inside a project can point outside); `acting`'s watcher is a clone of the
  `AsyncApp`, so the race borrows nothing twice; a dialog already open before an action skips the
  race; the drag turns interception off whatever happened.
- **Clippy found and fixed:** `needless_pass_by_value` (`Emulated` made `Copy`),
  `too_many_lines` (twice), `too_long_first_doc_paragraph`, `option_if_let_else` (twice), missing
  docs, `doc_markdown`. `cargo fmt`, then clippy again, clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/756-the-browser-tools-agents-lack.sh`, under `compositor sway`,
  exit 0, 25 checks passing. Shots read:
  - `756-01-hover`: "hovered" beside the button; chip "hovered over button “Hover me”".
  - `756-02-select`: the select on Blue, "Chose: blue", no list open.
  - `756-03-drag`: Card A under Done, "Done: Card A"; chip "dragged listitem “Card A” to list
    “Done”".
  - `756-04-upload`: "notes.txt" in the input and "Files: notes.txt".
  - `756-05a-dialog-card`: the card "127.0.0.1:… says / Delete the draft? / Cancel OK" over the
    page; chip "clicked button “Confirm”; the page opened a confirm dialog"; the click answered
    in 0 s with `"dialog": {"kind": "confirm", …}`.
  - `756-05-dialog`: "Outcome: confirmed", the card gone.
  - `756-06-wait`: "Ready" beside Start.
  - `756-07-resize`: the zoomed pane, the page phone-shaped at the top left, "Size: 393x852", the
    chip "iPhone 15 Pro · 393×852" and Reset.
  - `756-08-scheme`: the page light on a dark desktop, "Scheme: light", the chip "… · light".
  - `756-09-reset`: "Size: 1322x842 · Scheme: dark", no chip.
- **Fixes to the scenario:** the cards named with `aria-label` (unnamed list items have no name
  in the snapshot); a charset for `·`; the viewport meta (L-claude-756-a-phone-…); the forced
  scheme chosen against the system's (L-claude-756-headless-…); the pane zoomed so the toolbar
  has room for the chip and Reset.
- **The last run** was on the build of the gated code (only docs changed after it): exit 0, 25
  checks.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, and Fixed for the dialog wait); the guide (the browser
  tables, "Select lists and dialogs", "What agents can do in the tab", "Limits of the Browser
  tab") and the in-app guide's browser table; `marley_mcp.md`, `marley_browser.md` and
  `marley_workbench.md`. No Zed path.
- **Knowledge appended:**
  - AD-claude-756-agents-answer-dialogs-and-lay-pages-out-as-devices-001;
  - F-claude-756-an-agents-click-that-opened-a-dialog-waited-out-cdps-15-seconds-001 with
    PR-claude-756-race-page-input-against-the-pages-dialog-001;
  - L-claude-756-a-phone-lays-a-page-without-a-viewport-meta-out-980-wide-001;
  - L-claude-756-headless-chromium-follows-the-systems-dark-scheme-001.
- **Brain:** consultation 3264c06b6b2d4128aea3e20f9ffbac23 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed; #766's spec queued.

