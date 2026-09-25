# B2: The agent sees and drives the browser — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-492-browser-tools-for-agents.md
- **Pipeline spec:** 492-browser-tools-for-agents.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; Chad's "first class access to what the user sees … cursor
  like experience".
- **Classification:** feature; `marley_browser` (the rings, the snapshot, the redaction),
  `marley_mcp` (the browser family), `marley_workbench` (answering calls, the chip). No Zed
  path.
- **Recall (§18.3):**
  - orchestration-shell.md §8 and §10: "don't scrape what you can query"; write tools behind
    grants (D2 here changes the default for this family, with its reasons).
  - The handoff's token measurements (agent-browser's interactive snapshot about 3,400 tokens;
    Playwright MCP about 11,900 plus a 4,600-token schema).
  - The probe: the main frame's AX tree leaves cross-site iframes out.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #491 committed (3181a9efd6); no other active pipeline; cargo idle.
- **Recall:** the #491 registry and deferred calls are the seam (the prevention rule: the list
  derives from `REGISTRY`); D15's redaction (no headers or bodies, secret-looking query values);
  #489's key and mouse mappings for the write tools. Brain consultation
  `50d22159f78b403a8a715ab9531ca7f9`: nothing on this seam.
- **The probe (2026-09-25, a scratch headless Chromium 152):**
  - `Target.setAutoAttach {autoAttach, waitForDebuggerOnStart: false, flatten}` on the page
    session attaches a cross-site iframe as `type: iframe` with its own session; its target id
    is its frame id. The page's `Accessibility.getFullAXTree` shows it as an empty `Iframe`
    node; the iframe session's tree has its field.
  - `DOM.getBoxModel {backendNodeId}` works without `DOM.enable`. In an iframe session the quad
    is in the iframe's viewport; `DOM.getFrameOwner {frameId}` then `DOM.getBoxModel` on the
    page session gives the owner's content box, whose top left is the offset.
  - AX names carry trailing spaces; a password field's AX value is Chromium's bullets, one per
    character.
  - `Page.captureScreenshot {format: jpeg}` and `Page.getLayoutMetrics` (`cssLayoutViewport`,
    `cssVisualViewport`) answer on the page session.

### Design
- **`marley_browser`:**
  - `snapshot.rs` (new, pure): AX nodes into text lines and refs. Interactive roles (button,
    link, textbox, searchbox, checkbox, radio, combobox, listbox, option, menuitem and its
    checkbox and radio kinds, slider, spinbutton, switch, tab, treeitem), names trimmed, states
    (disabled, checked, expanded, selected, focused), a link's URL redacted; `full` indents the
    whole tree and folds unnamed generic nodes; no values; capped at 30,000 characters.
  - `observe.rs` (new, pure): `ConsoleEntry` from `Runtime.consoleAPICalled`,
    `Runtime.exceptionThrown` and `Log.entryAdded`; `NetworkEntry` built up from
    `Network.requestWillBeSent`, `responseReceived`, `loadingFinished` and `loadingFailed`;
    rings of 200; `redact_url` (query and fragment values whose names hold `token`, `key`,
    `secret`, `password`, `auth`, `code`, `sig` or `session` become `…`; user info dropped; a
    `data:` URL cut to 100 characters).
  - `page.rs`: `call_in(session, …)`; `observe()` (Runtime, Network, Log and auto-attach on a
    session); `screenshot()`; `layout_metrics()`; `frame_tree()`; `ax_tree(session, frame)`;
    `scroll_into_view`, `box_center`, `frame_owner_origin`; `focused_element()` (in the isolated
    world: tag, type, value unless a password field; then role and name from the element's AX
    node).
  - `input.rs`: `char_press(char)`, and `chord(text)` for `Ctrl+A`, `Enter`, `Shift+Tab`,
    `ArrowLeft`, through gpui's `Keystroke::parse` and `key_press`.
- **`marley_mcp`:** `Family::Browser` (served), the read rows (`Tier::Read`) and the write rows
  (`Tier::Write`, grant class `browser.write`), their schemas; `ToolAnswer::image` and an image
  content block in `tool_answer_result`.
- **`marley_workbench`:**
  - `browser.rs`: the hub enables the observers on the page and on each iframe session, keeps
    the iframe sessions, the rings, the snapshot's refs and the agent's last action; methods the
    tools use (`page`, `navigate_task`, `go_task`, waiting for the load); the toolbar's Agent
    chip, hidden five seconds after the last action ends.
  - `browser_tools.rs` (new): the ten tools, each answered in a task that awaits CDP and then
    `AppCall::answer`s; a write tool first brings a Browser tab to the front (opening one in the
    active workspace, without focus) and waits for the hub to show the page.
  - `mcp.rs`: `browser_*` calls go to `browser_tools`; the server starts with `browser.write`
    granted.
- **Manifest:** `crates/marley_browser/src/{marley_browser.rs, snapshot.rs, observe.rs,
  page.rs, input.rs}`; `crates/marley_mcp/src/{marley_mcp.rs, registry.rs, tools.rs}`;
  `crates/marley_workbench/src/{browser.rs, browser_tools.rs, mcp.rs, marley_workbench.rs}`;
  `script/e2e/492-browser-tools.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline Chromium) | Shot or log |
|---|---|---|
| REQ-007, 004 | no Browser tab; the agent navigates to the fixture | `492-01-opened-and-navigated` |
| REQ-001 | `browser_look`, its image saved | run log, `492-look.jpg` |
| REQ-002 | `browser_snapshot` | run log: the iframe's field with a ref |
| REQ-003 | `browser_console`, `browser_network` | run log: the page's message and error, `token=…`, `page=2` |
| REQ-005, 006 | type into the email field and the iframe's field by ref, click Sign in | `492-02-typed-and-clicked`: the page's report of trusted events, the chip |
| REQ-006 | six seconds later | `492-03-chip-gone` |
| REQ-004 | `file:` and `javascript:` navigations | run log: refused |
| REQ-008 | `tools/list` | run log: no evaluate tool |

### Risks
- An agent's write lands in the page Chad is looking at; each call goes through Claude Code's
  approval unless allowed, and the chip says what happened.
- The rings hold what the page logs; redaction is by name, so a secret in an oddly named
  parameter passes. Headers and bodies never enter.
- Refs go stale when the page changes; a stale ref answers with an error that says to take a
  new snapshot.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (and the probe), spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `marley_browser`: `snapshot.rs` (`AxNode`, `FrameTree`, `RefTarget` with `describe`,
    `render`); `observe.rs` (`ConsoleLog`, `NetworkLog`, `redact_url`); `page.rs` (`call_in`,
    `observe`, `screenshot`, `viewport`, `frames`, `accessibility_tree`, `scroll_into_view`,
    `box_center`, `frame_origin`, `focused_element`, and `isolated_context`, which
    `selected_text` now shares); `input.rs` (`char_press`, `chord`); `address::agent_url`.
  - `marley_mcp`: `Family::Browser`, served; four read rows and six write rows (grant class
    `browser.write`), their schemas one function per kind; `ToolAnswer::image` and
    `ToolImage`, an image block after the text in `tool_answer_result`.
  - `marley_workbench`: the hub observes the page and each cross-site iframe, keeps the iframe
    sessions, the two rings, the newest snapshot's refs, the agent's last action and the calls
    waiting for a load; `navigate_task` and `go_task` answer once the page has loaded, or after
    15 seconds (`navigate` and `go` detach them); the toolbar's Agent chip; `show_for_agent`,
    which brings a Browser tab to the front or opens one without the focus;
    `browser_tools.rs`, the ten tools; `mcp.rs` routes `browser_*` there and starts the server
    with `browser.write` granted.
- **Deviations from the design, and why:**
  - A load waiter is taken before a navigation is sent, and fires when the main frame stops
    loading or moves within its document: waiting after the call returns could miss a
    `frameStartedLoading` that the event loop had not handled yet.
  - `browser_look` reads no selection while a password field has the focus: the selection script
    of #489 reads the field's value.
  - The chip counts typed characters and never shows them: an agent may type a password.
  - A ref inside a cross-site iframe nested in another answers with an error: its owner is in the
    outer iframe's process, which the page's session cannot place.
- **Review of the diff:**
  - Re-entrancy: tools read and update the hub from their tasks, never inside another update;
    `show_for_agent` runs in the call loop's top-level update.
  - Security: every URL an agent reads goes through `redact_url` (the page's in `look` and the
    write tools' answers, links in the snapshot, requests, console sources); no header or body is
    kept; no tool takes script; `agent_url` refuses every scheme but `http` and `https`.
  - Input: clicks and keys go through #489's `mouse_event` and `press`, so the page gets what
    the user's hand sends.
- **Checks:** `cargo check`, `cargo fmt`, and `cargo clippy -p marley_browser -p marley_mcp -p
  marley_workbench --all-targets -- -D warnings` clean. Fixed at the source: two schema
  functions too long (split per kind), `format_push_string` (a line's parts joined instead),
  `option_if_let_else`, a redundant clone, `needless_pass_by_value` and `needless_pass_by_ref_mut`
  (the helpers that only read take `&AsyncApp`), `needless_collect`, and the visibility pair
  (`redundant_pub_crate` against `unreachable_pub`, settled by a public module as the crate's
  others are).
- **Checklist (no TaskCreate in this harness):** snapshot ✓, observe ✓, page ✓, input ✓,
  address ✓, marley_mcp ✓, hub ✓, chip ✓, browser_tools ✓, mcp ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/492-browser-tools.sh` (`compositor sway`, `offline_chromium`). A
  loopback site serves a sign-in page that reports each key and click it gets with
  `isTrusted`, logs to the console, throws an uncaught error and fetches
  `/api?token=abc123&page=2`, with a cross-site iframe (`localhost`) holding a field. No Browser
  tab is open at the start. A stand-in agent (`agent-mcp.py`, written by the scenario) runs the
  plugin's bridge from the tree and calls the browser tools, finding refs in the snapshot by
  role and name. The harness gains `shot_file <name>`, the path of a file kept beside the
  shots, for the frame the agent saves.
- **First run: red on REQ-003.** `browser_network` held only the iframe's document and the
  favicon. A probe on a scratch Chromium showed every request reported on the page's session,
  so the fault was Marley's: `attached` set the hub to Showing and then enabled the observers in
  a task of its own, and the agent's `Page.navigate` reached Chromium between `Runtime.enable`
  and `Network.enable`, so the small page loaded and fetched before the network was watched
  (the console looked whole because Runtime and Log replay what they buffered). Fixed: the
  observers come on before the hub shows the page. Everything else in that run matched the
  criteria (below).
- **Second run: every criterion.**
  - `492-01-opened-and-navigated` (REQ-007, REQ-004, REQ-006): the agent's first
    `browser_navigate`, with no Browser tab open, opened one and loaded "Sign in"; the toolbar's
    Agent chip says "went to http://127.0.0.1:44161/index.html"; the answer:
    `{"did": "went to …/index.html", "url": "…/index.html", "title": "Sign in"}`.
  - `492-look.jpg` against `492-01` (REQ-001): the frame is the page the tab shows. The answer
    held the URL, the title, `loading: false`, the viewport (1085 by 860 at scale 1, scrolled 0),
    `focused: null` and an empty selection.
  - The snapshot (REQ-002): `textbox "Email" [ref=e1]`, `textbox "Password" [ref=e2]`,
    `button "Sign in" [ref=e3]`, then `iframe (http://localhost:…/frame.html):` and its
    `textbox "the frame's field" [ref=e4]`; no values.
  - The console (REQ-003): the page's log and warning with their lines, the uncaught error with
    its stack, Chromium's autocomplete hint, and the 404s, the fetch's source shown as
    `…/api?token=…&page=2`.
  - The network (REQ-003, the second run): `GET 200 Document …/index.html`, `GET 200 Document
    http://localhost:…/frame.html`, `GET 404 Fetch …/api?token=…&page=2`, `GET 404 Other
    …/favicon.ico`; no headers.
  - `492-02-typed-and-clicked` (REQ-005, REQ-006): "agent@example.com" in Email, "typed by the
    agent" in the iframe's field (its page shows "the frame has: typed by the agent"), "Signed in
    as agent@example.com."; the page's report: "keys: 17, trusted: 17", "click on email,
    trusted true", "click on submit, trusted true" (a click in the iframe stays in the iframe's
    document); the chip: "clicked button “Sign in”". The answers: "e1: typed 17 characters into
    textbox “Email”", "e4: typed 18 characters into textbox “the frame's field”", "e3: clicked
    button “Sign in”".
  - `492-03-chip-gone` (REQ-006): six seconds later the chip is gone.
  - Refused (REQ-004): `file:///etc/passwd` and `javascript:alert(1)` answered "an agent may
    open http and https URLs only, not file:" (and `javascript:`); the page stayed.
  - `browser_scroll` with 300 answered "scrolled 300 pixels down", and the next look had
    `scroll_y: 300`, with the Sign in button focused (`role: button`, `name: Sign in`).
  - `tools/list` (REQ-008): 13 tools, the three terminal and ten browser ones; none evaluates
    script.
- **Focus report:** a headless sway; "hyprland: 0 Marley windows before the run, 0 after; the
  run added no rule and did not reload it".
- **Gate:** the first `just gate-diff` was red on gate:14 only: three public docs linked the
  private `LOAD_TIMEOUT` and `AGENT_CHIP` (#488's class again). The docs now say 15 seconds and
  five seconds. The second run: 16 passed, 0 failed, `GATE GREEN [diff]`, touched
  `marley_browser`, `marley_mcp` and `marley_workbench`; the receipt matches the tree. The fix
  changed doc comments only, after the second e2e run.
- **Verdict:** every criterion has its shot or its line in the run log, read; the gate is green.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓ (second run),
  REQ-004 ✓, REQ-005 ✓, REQ-006 ✓, REQ-007 ✓, REQ-008 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (Agents see and drive the Browser tab); `marley_browser.md`
  (for agents: the snapshot, the rings and redaction, the page's new calls, the input helpers);
  `marley_mcp.md` (the browser family, images in answers); `marley_workbench.md` (the browser's
  agent tools, the hub's observers, rings, refs and chip); `three-prong-plan.md` (B2 shipped).
  No Zed path was touched.
- **Knowledge appended:** `F-claude-492-the-observers-came-on-after-the-page-showed-001`,
  `PR-claude-watch-before-you-announce-ready-001`,
  `L-claude-492-runtime-and-log-replay-network-does-not-001`,
  `L-claude-492-chromium-shows-a-password-by-length-001`,
  `AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001`. Two traps this ticket
  hit were in the ledger already, and recall missed them: a public doc linking a private item
  (the prevention rule at line 1098 of `prevention-rules.md`), and `unreachable_pub` against
  `redundant_pub_crate`, settled by a public module (the decision at line 1281 of
  `architecture-decisions.md`).
- **Brain:** consultation `50d22159f78b403a8a715ab9531ca7f9` closed by
  `decisions/agents-see-and-drive-marleys-browser-tab-through-ten-mcp-tools`, follow-up by
  2026-10-24.
- **Checklist (no TaskCreate in this harness):** document ✓, capture knowledge ✓, close the
  ticket ✓, archive ✓, commit (below).
