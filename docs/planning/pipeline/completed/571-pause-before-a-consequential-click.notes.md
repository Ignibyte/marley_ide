# A pause before a consequential click in the Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-571-pause-before-a-consequential-click.md
- **Pipeline spec:** 571-pause-before-a-consequential-click.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-26, approving the seven ranked uses of the Jev note
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md`). Use 6: "nouls for pays, deletes,
  sends in the user's name, changes an account; asks in the Browser tab's Agent chip, by default
  only when the agent runs without its own prompts; waits on #532; S–M". Drafted by the second
  spec drafter beside #565 and #566 to #568, and aligned to #565's pair once it was on disk.
- **Classification / tier:** feature, prong 3 with prong 2's C0. Marley crates, plus the three
  additive Zed paths a Marley setting takes (`browser_click_pause_agents`) and the page's
  dropdown for the use's mode. Size M.
- **Recall (§18.3):**
  - AD-claude-492 (agents drive the Browser tab through the MCP server; the write tools send the
    same CDP input events the user's hand makes; the checks are the client's approval and the tab):
    the pause is a third check in the same tab, and the click after Allow is the same
    `click_at`.
  - F-claude-495 (an agent's click read as the user's): the pause's card is drawn by Marley and
    takes Enter and Esc only while the tab holds the focus, so a key meant for the page never
    answers it, and the agent's own click never moves the focus.
  - L-claude-493 (a tab brought forward in the focused pane takes the focus) and #525's D6: the
    toast with Show, no focus moved by Marley.
  - L-claude-498 (a new toolbar button moves older scenarios' clicks): the card sits under the
    toolbar and appears only while a pause waits, so #492's and #500's coordinates hold.
  - AD-claude-516 and PR-claude-redact-the-whole-text-before-cutting-it-001: the nearby text is
    redacted whole, then cut.
  - AD-claude-490 (the Browser tab draws its own chrome and dialogs): the card follows
    `render_dialog`'s pattern.
  - #565's D1, D2 and D5 (the verdict handed with each ask; `rules` answers it; a compiled-in
    set) and its budget row for reviewer-less approvals (a 1.5 s deadline, one retry).
  - Brain: not consulted in this drafting session; promotion asks.
- **Discovery** (an Explore sweep at `ca70b6488d`; `browser.rs`, `browser_tools.rs`,
  `marley_browser` and `marley_mcp` are at HEAD; promotion re-verifies):
  - `crates/marley_workbench/src/browser_tools.rs`: `run` (63-111), `WRITES` (42-49),
    `show_for_agent` (85-87), `click` (494-532), `target_point` (690-709), `ref_target` (678-686),
    `place` (713-726), `ref_origin` (760-795), `click_at` (876-901), `type_text` (534-580, clicks
    at 550-555), `done` (394-406), `acting` (409-428), `answer` (53-61).
  - `crates/marley_browser/src/snapshot.rs`: `RefTarget` (113-128), `describe` (130-140); the
    snapshot keeps role and name on a ref (274-281) and no tag or form.
  - `crates/marley_browser/src/page.rs`: `DESCRIBE_ELEMENT` (29-31), `focused_element`
    (758-819), `DOM.describeNode` (783-785), `scroll_into_view` (651-663), `box_center`
    (670-679), `answer_dialog` (507).
  - `crates/marley_workbench/src/browser.rs`: `AgentAction` (130-136), `AGENT_CHIP` (97-98),
    `PageState.agent` (313-314), `agent_started` (1609-1623), `agent_ended` (1625-1667),
    `agent_chip` (1669-1675), `render_toolbar` (4064-4163; the chip 4130-4150), `render_dialog`
    (4357-4434), `answer_dialog` and `dismiss_dialog` (3457-3475), `dialog_focus` (3014),
    `render_tray` (4438-4483), `record_this` (3760-3790), `send_pick` (3645-3689),
    `show_for_agent` (5678-5688); `crates/marley_workbench/keymap.json` (60-63, 84-96).
  - `crates/marley_mcp`: `registry.rs` (`ToolSpec` 46-57, `browser_write` 233-241, the click
    schema 761-769), `permission.rs` (`Tier` 31-36, `decide` 54-64), `dispatch.rs`
    (`initialize` 27, `tools_call` 133-174), `transport.rs` (`HttpRequest` 381-388, the session
    at `initialize` 212-223, `ask_app` 280-292), `session.rs` (`SessionEntry` 35-38),
    `tools.rs` (`tool_result` 18-25, `tool_error` 62-64), `marley_mcp.rs`
    (`APP_CALL_TIMEOUT_SECONDS` 81, `PendingCall` 120-127, `AppCall` 131-137).
  - The bridge, `claude_plugin/marley/bin/marley-mcp-bridge`: `connect` forwards the client's own
    `initialize` params, so Marley's server sees Claude Code's, Zed's (`Zed`,
    `crates/context_server/src/context_server.rs:150-155`) and the stand-in's (`e2e`,
    `browser-fixture.sh:234`) `clientInfo`; `REQUEST_TIMEOUT` 40.
  - Zed: `crates/agent/src/tools/context_server_registry.rs:349-350`
    (`authorize_third_party_tool`), `crates/agent/src/thread.rs:993-1056` (the options),
    `crates/agent_settings/src/agent_settings.rs:1680` (`always_allow_tool_actions`).
  - `script/e2e/browser-fixture.sh`: `mcp_agent` (76-81), `write_mcp_agent` (205-479: `click-on
    <role> <name>` 336-345 through `browser_snapshot` and `find_ref`; `tool` prints `refused:
    <text>` for an `isError` result 263-273; `recording`), `serve_site` (43-55),
    `offline_chromium` (57-69); `script/e2e/492-browser-tools.sh` (the sign-in page, the chip shot).
  - #565 (queued): `UseSpec`, `system_one::ask`, `StateBuilder`, `Detail`, `Reading`, the day's
    file, the replay file and `system_one_setting`; `marley.system_one.uses`.
  - `crates/settings_content/src/marley.rs` (10-29; `MarleyLayout`'s `strum` derives for a
    dropdown, 33-54), `crates/settings_ui/src/marley_page.rs` (42-91),
    `assets/settings/default.json` (1664-1674).
- **Decisions:** D1 to D9 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #569's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, at `3da83e59c5`) ✓; spec and design updated ✓; the ledger rows widened ✓.
- **Recall, added:** PR-claude-a-state-fact-holds-only-what-code-computed-001 (the page's words go
  in as text, code's facts as facts); L-claude-569 and AD-claude-569 (#569's landing and banner
  patterns; `notifications::post`); L-claude-567-a-tool-a-setting-lists-must-reach-the-servers-data-001
  (anything the server must know per call, such as the client's name, is server data kept under
  its lock); F-claude-495 (an agent's action read as the user's), which now decides D5. The brain
  (consultation 16c9dadae2ab4f289a22c967fb762b19): nothing on this seam.
- **What the code says now** (the Explore report, 2026-09-28; the drafted line numbers are stale):
  - `browser_tools::answer` (`:85-110`) builds a scope from `call.caller()` (#520, #574) and `by`
    from `call.principal().client_name()` (#524's outside clients), and spawns `run(tool,
    arguments, scope, by, hub, cx)` (`:246-304`). `run` returns early for `browser_navigate`,
    `browser_check_pick` and `browser_open_url` before the `WRITES` gate (`:277`). `click`
    (`:1280-1320`) takes `by`; `type_text` with a ref clicks through `click_at` before `acting`
    (`:1339-1344`). Refs live on `PageState.refs` (`browser.rs:387`), and `browser_find` replaces
    them.
  - #520 has shipped: the bridge sends `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` on
    every request; `AppCall::caller()` is `Caller {terminal, project, cwd}` ("never an
    authority"), and `mcp::caller_terminal(caller, cx)` (`mcp.rs:749-758`) finds the view; the seat
    is `AgentEvents::seat(view.entity_id())`, its mode the `permission_mode` label (no constant).
    Zed's context server `marley` runs the bridge with a blank terminal id.
  - `initialize` still drops its params (`dispatch.rs:42`); `SessionEntry` is `{id, owner,
    last_seen_ms}`; the bridge forwards the client's own `clientInfo` (`e2e` for the fixture, `Zed`
    for Zed). `tool_error` gives `{"result":"refused","reason":…}` with `isError`, and an app
    `Err(reason)` reads `"{tool}: {reason}"`; the fixture prints `browser_click refused: {…}`.
  - Zed's `always_allow_tool_actions` is gone: `AgentSettings.tool_permissions` is
    `ToolPermissions { default: ToolPermissionMode (Allow, Deny, Confirm), tools }`
    (`agent_settings.rs:474-478`), per tool by id (`mcp:marley:browser_click`). `agent_settings` is
    a dev-dependency of the workbench only.
  - `BrowserHub` holds a Chromium per project (#507); `PageState` has `agent`, `driven_by`,
    `refs` and `viewers`, and `record_entry` records only while the tab has viewers.
    `RecordedEntry::Agent { did, by }` (`recorder.rs:323-330`). `agent_chip` gives (who, action).
  - The view renders the toolbar, then the trays (scripts, picks), then the page area with the
    dialog overlay (`render_dialog`, an `AlertModal` with `key_context("MarleyBrowserDialog")`,
    Enter and Esc in `keymap.json:108-116`); `focus_dialog` moves the focus to a dialog when the
    tab holds it.
  - `page.rs` has `call_in(session, method, params)` and `focused_element`'s isolated world;
    `DOM.resolveNode {backendNodeId}` is used only in `pick.rs`; `DOM.getNodeForLocation` nowhere.
  - #565's sets and uses live in `marley_system_one`'s root (`STALL_KIND_SET`, `STALL_KIND`);
    there is no `question.rs` and no `NoSignal`; a noul holds above `NOUL_HIGH` (0.65). A use passes
    an `Asking`; `state_for` builds the masked state.
  - `Toast::new(id, message).on_click(label, …)` has one button; `show_toast` and `dismiss_toast`
    by `NotificationId`. `record_this`'s toast has none.
  - An enum dropdown on the Marley page needs `settings_ui.rs`'s `add_basic_renderer` (`:559-564`);
    the Agents section has six items and System One twelve.
  - The golden set's agent clicks: 492 (`click-on button "Sign in"`), 504 (`click-on button
    Save`), 524 (an outside driver's `click-on button Save`) and 567 (`click-ref`).

### Design
- **Changed at promotion** (the seams re-read on 2026-09-28; each item overrides the drafted
  design after it):
  - **The caller is the calling terminal.** `prompt_less(call, cx)` reads `call.caller()`: with a
    terminal, `caller_terminal` gives the view and `AgentEvents::seat` its Claude Code seat, whose
    `permission_mode` of `bypassPermissions` or `dontAsk` is prompt-less and any other mode
    prompts; no seat is unknown. With no terminal, the session's client name `Zed` is prompt-less
    while Zed's `tool_permissions` give `mcp:marley:browser_click` (or, unset, the default)
    `Allow`; any other name, and #524's outside clients, are unknown. Unknown pauses.
  - **The client's name.** `SessionEntry` gains `client: Option<String>` from `initialize`'s
    `params.clientInfo.name`, kept at the session's assignment in `transport.rs` (printable ASCII,
    at most 64 characters), and `AppCall` gains `client()`. `PendingCall` is unchanged.
  - **Zed's setting.** `AgentSettings::get_global(cx).tool_permissions`, with `agent_settings`
    moved from the workbench's dev-dependencies to its dependencies.
  - **The pause's home.** `PageState.pause: Option<PendingClick>`, beside `refs`. `run` refuses a
    write on a paused tab at the `WRITES` gate and in `browser_navigate`'s and
    `browser_check_pick`'s own paths.
  - **The keys.** The card has its own focus handle and never takes the focus (F-claude-495; an
    Enter meant for the page must never allow a click, so `focus_dialog`'s pattern is not
    followed). Allow and Refuse are buttons; the toast's Show brings the tab forward and focuses
    the card; `MarleyBrowserPause` binds Enter to `marley::AllowPausedClick` and Esc to
    `marley::RefusePausedClick` for the focused card only.
  - **The facts read.** In the ref's session: `DOM.describeNode {backendNodeId}`, then
    `DOM.resolveNode {backendNodeId, executionContextId}` in the isolated world and
    `Runtime.callFunctionOn` with `FORM_FACTS`, released after; a point first goes through
    `DOM.getNodeForLocation`. `page.rs` gains `node_facts` and `node_at` over `call_in`.
  - **The set.** `CLICK_CONSEQUENCE_SET` (`click_consequence/1`: nouls `pays`, `deletes`, `sends`,
    `changes_account`) and `CLICK_CONSEQUENCE` (1.5 s) in `marley_system_one`'s root. The state:
    facts code computed (role, tag, input type, in form, form method, the URL's host and path, the
    class the rules considered); texts the page's words (the name, the form's action, the title,
    the nearby text), per PR-claude-a-state-fact-holds-only-what-code-computed-001. A rule's pause
    is a `rules` row with its noul held (`noul_verdict`).
  - **The modes.** As D7 now reads: `suggest` adds a toast after an open click the model reads
    consequential; nothing asks the model about a rule-classed target.
  - **The toast.** `NotificationId::composite::<PausedClick>(tab)`, `on_click("Show", …)`,
    dismissed when the pause ends.
  - **The fixtures.** The fixture's client name gains an override (`MCP_CLIENT_NAME`, default
    `e2e`), so a scenario can call as `Zed`; #566's stand-in `claude` gains a `run` action that runs
    the fixture's client inside Marley's terminal, so its calls carry the terminal's id; the checks
    read the refusal's reason in the fixture's JSON; the card's buttons are clicked (sway).
- **The word lists** (`marley_browser::consequence`, matched on the lowercased name, the
  `aria-label` and the button's text; whole words):
  - Pays: `pay`, `pay now`, `buy`, `buy now`, `purchase`, `checkout`, `check out`, `place order`,
    `order now`, `subscribe`, `upgrade`, `renew`, `confirm payment`, `charge`, `donate`, `tip`;
    a form whose action or the URL's path matches `checkout|billing|payment|pay|subscribe|cart`.
  - Deletes: `delete`, `remove`, `destroy`, `erase`, `discard`, `drop`, `purge`, `deactivate`,
    `close account`, `cancel subscription`, `unsubscribe`, `revoke`, `wipe`; a form with
    `formmethod=delete`, or a path matching `delete|remove|destroy`.
  - Sends: `send`, `post`, `publish`, `reply`, `tweet`, `share`, `email`, `submit` when the form
    holds a `textarea` or a field named `message|comment|body|subject|to`; `invite`; a path
    matching `compose|send|publish|post`.
  - Changes an account: `change password`, `change email`, `update email`, `enable two-factor`,
    `disable two-factor`, `add key`, `create token`, `generate token`, `api key`, `transfer
    ownership`, `make admin`, `sign out everywhere`, `log out all`, `link account`,
    `connect account`, `grant`, `authorize`; a path matching `settings/(account|security|
    password|tokens|keys|members)|oauth|authorize`.
  - Plain: `cancel`, `close`, `back`, `next`, `previous`, `skip`, `learn more`, `menu`, tabs,
    `menuitem`s that only open menus, links whose URL is the same origin and matches none of the
    paths above, checkboxes, radios, and any `combobox`, `textbox` or `searchbox`.
  - Open: a `button` or `link` named `continue`, `confirm`, `ok`, `yes`, `submit`, `done`,
    `proceed`, `apply`, `save` (a save is plain unless the form or the path says otherwise), or a
    name in no list, on a page whose path or nearby text names a consequential context (`charge`,
    `$`, `€`, `£`, `payment`, `permanently`, `cannot be undone`, `will be sent`, `recipients`,
    `password`, `two-factor`).
- **The rule table** (`classify`, first match wins): 1 a name in a consequential list → that
  class; 2 a form or path fact → that class; 3 an open name with a consequential context →
  `Open`; 4 else `Plain`. `Open` is asked only where the project is listed and the mode is not
  `off`; otherwise it clicks.
- **The facts.** `browser_tools::click` (and `type_text` with a ref) calls
  `target_facts(page, target)` before `click_at`: `RefTarget`'s role and name; `DOM.describeNode`
  on the backend node for the tag and attributes; `Runtime.callFunctionOn` with a fixed function
  (`FORM_FACTS`, beside `DESCRIBE_ELEMENT`) returning `{form: {action, method, has_textarea,
  field_names}, context: innerText of the closest form, dialog, section or main, cut to 300}`;
  for a point, `DOM.getNodeForLocation` first. A failed read gives `Open` with what is known (the
  URL alone can still make it consequential through rule 2).
- **The pause.** `BrowserHub` gains `pauses: HashMap<target, PendingClick>`; `click` builds the
  facts, classifies, applies the caller rule, and for a consequential class (or an `act`-mode
  reading) parks the call: it registers the `PendingClick` (the call's `AppCall`, the target
  identity, the sentence, the deadline), shows the card and the toast, and awaits a oneshot the
  card answers. Allow: `ref_target` again, `describe` compared to the parked role and name, the
  URL compared, then `click_at` and `done`; else `Err`. The timer (`cx.background_executor()
  .timer(25 s)`) answers `expired`. While `pauses` holds the tab, `run` refuses the tab's
  other `WRITES` before dispatch. The recorder gets `Agent { did: "paused: …" }` and
  `Agent { did: "allowed: …" | "refused: …" | "expired: …" | "changed: …" }`.
- **The card.** In `BrowserView::render`, under the toolbar and above the tray, while
  `hub.pause(target)` is some: a bordered row with the sentence, `Button "Allow"` (filled) and
  `Button "Refuse"`, `key_context("MarleyBrowserPause")` with `marley::AllowPausedClick` (enter)
  and `marley::RefusePausedClick` (escape) in `keymap.json`, tracked focus as the dialog's; the
  toast: `workspace.show_toast(Toast::new(id, "An agent wants to click Place order in
  shop.example").on_click("Show", …))`, dismissed with the pause.
- **The caller.** `marley_mcp::session::SessionEntry.client: Option<String>` from
  `initialize`'s `params.clientInfo.name` (cut to 64); `PendingCall.client` and `AppCall.client`
  carry it. `marley_workbench::browser_tools::prompt_less(client, cx) -> bool` by D2, reading
  `AgentSettings` for `Zed` and `AgentEvents` for Claude Code's seats.
- **The ask.** `click_consequence/1`: four nouls over the state built with `StateBuilder::new(
  project, detail)`: facts `role`, `tag`, `input type`, `in form`, `form method`, `url host`,
  `url path`, `rules found` (the class the rules considered, or `nothing`); texts `name`,
  `form action`, `page title`, `context` (each through the layer's `Mask`, dropped under
  `Detail::Facts`); the use's verdict (`Open`) with it; `UseSpec { name: "click_consequence",
  deadline: 1.5 s }`. The answer is folded into the class when any noul is at or above its
  threshold (0.65 to start), in `act` only.
- **File manifest** (as promoted).
  - Marley: `crates/marley_browser/src/consequence.rs` (new: the facts, the word lists,
    `classify`), `marley_browser.rs` (the module), `page.rs` (`FORM_FACTS`, `node_facts`,
    `node_at`); `crates/marley_workbench/src/browser_tools.rs` (the facts read, the class, the
    caller rule, the pause at `click` and at `type_text`'s click, the refused writes in `run`),
    `browser.rs` (`PageState.pause`, `PendingClick`, the card, the toast, Allow and Refuse, the
    recorder entries), `marley_workbench.rs` (the two actions, `MarleySettings`),
    `keymap.json` (`MarleyBrowserPause`), `crates/marley_workbench/Cargo.toml` (`agent_settings`
    to the dependencies); `crates/marley_mcp/src/session.rs` (`SessionEntry.client`),
    `transport.rs` (the name kept at `initialize`, carried to the `AppCall`), `marley_mcp.rs`
    (`AppCall::client`); `crates/marley_system_one/src/marley_system_one.rs` (the set and the use).
  - Zed paths with rows: `crates/settings_content/src/marley.rs` (`browser_click_pause_agents`,
    `MarleyClickPauseAgents`, the `uses` docstring), `crates/settings_ui/src/marley_page.rs` (the
    two items), `crates/settings_ui/src/settings_ui.rs` (the dropdown renderer),
    `assets/settings/default.json`.
  - Scenarios: `script/e2e/571-pause-before-a-consequential-click.sh` (new),
    `script/e2e/browser-fixture.sh` (`MCP_CLIENT_NAME`), `script/e2e/golden`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md` rows 55, 58, 60 and 61 widen for
  `browser_click_pause_agents`, `MarleyClickPauseAgents` and `uses.click_consequence` before those
  files are written (done at promotion).

### E2E plan
As promoted, over the draft below: the stand-in `mcp_agent` runs in the background for each click
with its output kept, and the checks read the refusal's reason inside the fixture's JSON
(`browser_click refused: {"result":"refused","reason":"browser_click: Marley paused this click:
…"}`); the card's Allow and Refuse are clicked by the pointer, and REQ-002's Enter and REQ-003's
Esc are pressed after a click has focused the card; `marley_setting` (569's) writes
`browser_click_pause_agents` and the layer's block. Three rows join the table:

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-016 | `agents_without_prompts`, mode `shadow`; #566's stand-in `claude` in Marley's terminal sends a session with `permission_mode` `default`, then runs the fixture's `click-on button "Place order"` from inside the terminal; then the same with `bypassPermissions` | `571-12-claude-prompts`: no card, the log reads `Place order`; `571-13-claude-bypass`: the card |
| REQ-017 | `MCP_CLIENT_NAME=Zed` from the scenario's shell (no terminal id); the profile's `agent.tool_permissions.default` `confirm`, then `allow` | `571-14-zed`: no card at `confirm`, the card at `allow` |
| REQ-014 | the Marley settings page | `571-15-settings`: Browser Click Pause Agents in Agents, Click Consequence in System One |

The draft's plan:

Fixtures: the offline Chromium and `serve_site shop`; a scratch repository as the project; the
stand-in `mcp_agent` in the background for each click (`wait "$pid"` after the answer);
`marley.browser_click_pause_agents` rewritten per step; #565's block enabled on `replay` with
the repository listed and `uses.click_consequence` rewritten per step by `system_one_setting`;
`$E2E_PROFILE/system_one/replay.jsonl` written by setup (matched on the context text: `charged`
→ `pays` 0.88; the second Continue's context → 0.5); the card's and the toast's coordinates
measured from the first run, as #547's chip was.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `all_agents`, mode `shadow`; `mcp_agent click-on button "Place order" &`; settle 2 | `571-01-paused`; `mcp_agent look` shows the log empty |
| REQ-002 | click the tab; `press "" Return`; settle 2; `wait "$pid"` | `571-02-allowed`; the stand-in's output holds `clicked` |
| REQ-003 | `click-on button "Delete account" &`; settle 2; `press "" Escape`; `wait` | `571-03-refused`; the output holds `refused: Marley paused this click` |
| REQ-005 | mode `act`; `click-on button "Continue" &`; settle 2 | `571-04-open-case`; the day's file's row; then Escape |
| REQ-006 | `click-on link "Next"`; settle 1 | `571-05-plain`; the log reads `Next`; no row |
| REQ-004 | `click-on button "Remove item" &`; settle 26; `wait` | `571-06-expired`; the output holds `did not answer within 25 seconds` |
| REQ-007 | the page's second Continue (no charge text); `click-on` it; settle 2 | `571-07-no-signal`; the day's file's row |
| REQ-008 | `click-on button "Place order" &`; settle 2; `click` on Swap by the pointer; `press "" Return`; `wait` | `571-08-changed`; the output holds `the page changed` |
| REQ-009 | `click-on button "Place order" &`; settle 1; `mcp_agent scroll 100`; Escape | `571-09-blocked-writes`; the output holds `a click is paused` |
| REQ-010 | `agents_without_prompts`; `click-on button "Place order" &`; settle 2; Escape | `571-10-unknown-caller` |
| REQ-011 | mode `off`; `click-on button "Place order"`; settle 1 | `571-11-off`; the log reads `Place order` |
| REQ-012 | `record_this` after 01; `mcp_agent recording` | the run log: `paused:` and `allowed:` entries |
| REQ-013 | the card's tooltip (`pointer_to` the card) | the run log: `e2e` in the tooltip's text |
| REQ-014 | see REQ-014 above | `571-15-settings` |

Not reachable by a scenario: a Zed Agent thread with a model behind it (the stand-in calls as
`Zed` from outside any terminal, which is what the rule reads), a real Claude Code seat in
`bypassPermissions` (#566's stand-in sends the label from inside Marley's terminal), and Codex's
full access before #532.

### Risks
- One more CDP round trip before each click (`describeNode` and a `callFunctionOn`): about
  10 ms on the box; `browser_click` already scrolls the node into view and reads its box.
- The word lists miss a page's wording (a button named "Go" that pays): that is the open case,
  and the model's job; what both miss clicks as today, in front of Chad.
- A pause on a `type_text` with a ref (it clicks first) surprises an agent typing into a
  checkout field; the pause names the element, and a field is `Plain` by rule 4, so only a
  button-shaped ref pauses.
- Claude Code's own MCP prompt plus this pause makes two questions for one click under
  `all_agents`; the default `agents_without_prompts` avoids it, and D9 explains the coarse rule
  until #520.
- The `rules` and `shadow` cases pause on the rules alone, so a user who wants the pause with no
  model at all sets the provider to `rules` (#565's D1); the layer must be enabled for any pause,
  which is #565's REQ-001 and the price of one switch per use.
- If the slice runs long, the client's name (REQ-013) and the recorder entries (REQ-012) split
  off, with every caller treated as unknown meanwhile; the classes, the pause and the model are
  the first slice.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger rows first (55, 58, 60 and 61, at promotion) ✓; the
  README marker ✓; every manifest file written ✓; `cargo check` and `just clippy` on the touched
  crates, one at a time after #569's release install ended ✓; the review below ✓.
- **Built.**
  - Settings: `browser_click_pause_agents` and `MarleyClickPauseAgents { AgentsWithoutPrompts,
    AllAgents }` in the content, `MarleySettings`, `default.json` (with `uses.click_consequence:
    "off"`), the Marley page's Agents section (Browser Click Pause Agents, seven items) and System
    One section (Click Consequence after Stall Kind, thirteen items), and `settings_ui.rs`'s
    dropdown renderer. The actions `AllowPausedClick` and `RefusePausedClick`.
  - `marley_system_one`: `CLICK_CONSEQUENCE_SET` (`click_consequence/1`: `pays`, `deletes`,
    `sends`, `changes_account`) and `CLICK_CONSEQUENCE` (1.5 s).
  - `marley_mcp`: `SessionEntry.client` from `initialize`'s `clientInfo.name`
    (`client_name_of`, printable ASCII, 64 characters), kept at the session's assignment
    (`name_client`) and read under the server's lock for each request (`client_of`); #520's
    `Caller` gains `client`, so `AppCall::caller()` carries it with no new accessor.
  - `marley_browser::consequence` (new, pure): `ClickFacts`, `Class`, `Verdict` and `classify`,
    the rule table first match wins: a role that only chooses; a consequential name; a plain
    name; a form that deletes, or a form's or a link's target that says what it does; an open
    name (a link) or any name (anything else) among consequential text; else plain.
    `page.rs`: `CLICK_FACTS`, `NodeFacts`, `node_facts` and `node_at`, and
    `isolated_context_in(session)` under `isolated_context`; `pick::whole_pixels` is
    `pub(crate)` for `node_at`'s integers.
  - `marley_workbench::click_pause` (new): `Who::of` (D2), `Click`, `before_click`, `hold`,
    `same_element`, the facts and the state. `browser_tools`: `answer` sorts the caller;
    `run` refuses a write on a paused tab (`not_paused`), as do `navigate` and `check_pick`;
    `click` and `type_text`'s first click go through `paused_point`. `browser.rs`:
    `PageState.pause` and `PendingClick`; the hub's `pause_click`, `is_paused`, `answer_pause`
    and `end_pause`, with the recorder's `paused:` and `allowed:`/`refused:`/`expired:`/
    `changed:`/`gone:` entries; the view's `pause_focus`, `render_pause` (the bar under the
    toolbar), `allow_paused_click`, `refuse_paused_click` and `answer_pause`; `tab_workspace`,
    `show_pause_toast` (Show), `dismiss_pause_toast`, `show_click_notice` and `show_paused`.
    `keymap.json`: `MarleyBrowserPause`. `agent_settings` moved to the workbench's
    dependencies. `claude_events` names the permission mode's label (`PERMISSION_MODE_LABEL`),
    which #569's watch now uses too.
- **Deviations from the design, and why.**
  - **A module of its own.** The pause is `click_pause.rs`, not more of `browser_tools.rs`
    (2,000 lines): it is its own component, and `browser_tools` calls it at two points.
  - **The client's name rides on `Caller`.** `AppCall::new` keeps its signature; the name is
    one more self-reported field beside #520's.
  - **The plain names come before the targets.** A page's or a form's path would otherwise
    pause "Back" on a checkout page; a consequential name still wins over a plain one ("Cancel
    subscription" deletes, "Cancel" is plain).
  - **Point clicks climb to what they commit.** `DOM.getNodeForLocation` gives the text or the
    span inside a button, and a point click has no snapshot's role or name, so the facts
    function climbs to the nearest clickable element and gives its role and name.
  - **The page's words are text, code's are facts** (PR-claude-a-state-fact-holds-only-what-code-computed-001):
    the name, what the rules found, the form's target, the page's path and title, the text
    around it; the host is a fact, as D6 says.
  - **A page gone during a pause** refuses as `gone`, not as the user's refusal.
  - **An unpaused open click's outcome** (whether the user took the tab or went back within a
    minute) waits for a later slice; the pauses' outcomes are logged.
- **Checks.** `cargo check` on the seven crates: clean. `just clippy` (all targets) found, and
  the code now answers: `redundant_pub_crate` and `unreachable_pub` pulling opposite ways on the
  session module's helpers (re-exported from the crate root, as the module's other items are);
  `significant_drop_tightening` and `too_many_lines` in `serve_connection` (the session gate is
  now `gate`, which drops its guard on every path); `needless_pass_by_value` in `pause_click`
  (its arguments are moved in); `too_many_lines` in `BrowserView::render` (the bar is
  `pause_bar`, one line there); `option_if_let_else`, `needless_pass_by_ref_mut` and two long
  first doc paragraphs. The last run is green.
- **Review** (REQ-001 to REQ-017, the security lens):
  - Nothing is clicked while a pause holds: the click waits on the answer, a second write in the
    tab is refused, and a second pause in the tab is refused.
  - No reading removes a rule's pause: a rule's class pauses before the layer is asked, and only
    `Open` is asked; `act` adds a pause on a noul that holds.
  - Allow clicks only the same element: the page's URL and the element's own role, name and tag
    under the same node, read again.
  - The card never takes the focus by itself; the buttons stop their clicks' propagation, so the
    card does not take the focus as it leaves.
  - The page's named elements and scripts cannot shadow what the facts function reads: it runs in
    an isolated world where one resolves, through the DOM's own prototypes.
  - Known limits: a click by a point into a cross-site iframe reads the iframe element; a submit
    by Enter from `browser_type` is not a click (deferred by the spec).

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; the fixture's
  `MCP_CLIENT_NAME` ✓; 571 in the golden set ✓; `just build` ✓; the scenario run and every shot
  read ✓; the reruns ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/571-pause-before-a-consequential-click.sh` (compositor sway): the
  offline Chromium and a served shop page (a cart form posting to `/checkout` with Place order,
  Remove item and Swap, which renames Place order to Cancel order; a Next link; a Continue under
  "You will be charged $12.00." and a second under "Your password stays as it is."; Delete
  account), whose script writes each click into its log and its title, which `browser_tabs`
  reads; the fixture's client through the plugin's bridge, in the background for a click that
  waits; the layer on `replay` with the repository listed, the mode and
  `browser_click_pause_agents` set per step, and Zed's `agent.tool_permissions.default` set
  by a merge into the profile's settings; `replay.jsonl` rows keyed `click_consequence/1`
  matched on the text around the element; #566's stand-in `claude` in Marley's terminal sending
  a prompt in `default`, then in `bypassPermissions`, and running the fixture's click from inside
  the terminal, so the call carries the terminal's id. The real `claude` never ran. Run 1 stopped
  where the guessed Allow point missed and the pause expired, which its shot showed; the card's,
  the page's and the toast's points came from that shot. Run 2 passed all 25 checks; run 3 moved
  the System One shot down to its last item and passed all 25 again.
- **The shots** (read one by one):
  - `571-01-paused` (REQ-001): under the toolbar, the card "e2e wants to click button “Place
    order”, which pays (its name), on 127.0.0.1:…" with Refuse and Allow; the toast with the same
    words and Show; the page's log empty. The rules' row is logged (`rules found: its name`).
  - `571-02-allowed` (REQ-002): Allow clicked; the card and the toast gone; the Agent chip
    "clicked button “Place order”"; the log and the title "Log: Place order". The recording
    saved next holds `paused: …` and `allowed: …` entries (REQ-012).
  - `571-03-refused` (REQ-003): a click on the card focused it and Escape refused; no card; the
    log without Delete account; the agent read "The user refused it; ask them before you try
    again".
  - `571-04-open-case` (REQ-005): in `act`, the first Continue waits: "which pays as the model
    reads it (0.88)"; the row names the charge; Refuse answered it.
  - `571-05-plain` (REQ-006): the Next link clicked at once (the chip, the log); no row.
  - `571-06-expired` (REQ-004): 25 seconds unanswered; no card, no toast; the agent read "did not
    answer within 25 seconds"; the log without Remove item.
  - `571-07-no-signal` (REQ-007): the second Continue, read at 0.5 by every noul, clicked at once;
    its row logged.
  - `571-08-changed` (REQ-008): the user's Swap renamed the button under the pause, and Enter on
    the focused card clicked nothing: the log holds Swap, not Cancel order; the agent read "The
    page changed while it waited".
  - `571-09-blocked-writes` (REQ-009): the card up while the agent's scroll was refused with "a
    click is paused in this tab; wait for the user".
  - `571-10-unknown-caller` (REQ-010): `agents_without_prompts`; the fixture's client, `e2e`, with
    no terminal, waits.
  - `571-11-off` (REQ-011): `off`; the click at once, no card, no new row.
  - `571-12-claude-prompts` (REQ-016): the stand-in Claude Code in the terminal, its seat in
    `default` ("working · Place the order"), clicked at once: no card, the log "Place order".
  - `571-13-claude-bypass` (REQ-016): its seat in `bypassPermissions`: "Claude Code wants to click
    button “Place order”…", the card and the toast; the toast's Show brought the card forward and
    Escape refused.
  - `571-14a-zed-asks`, `571-14b-zed-allows` (REQ-013, REQ-017): a caller that named itself `Zed`,
    with no terminal, clicked at once while Zed's permissions were `confirm`, and waited as "Zed's
    agent wants to click…" once they were `allow`: the server kept the client's name.
  - `571-15a-agents`, `571-15b-system-one` (REQ-014): Browser Click Pause Agents (All Agents, the
    profile's) in the Agents section; Click Consequence after Stall Kind in System One.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: a Zed Agent thread with a model behind it (a caller named
  `Zed` stands in, which is all the rule reads), a real Claude Code in `bypassPermissions` (the
  stand-in's label stands in), and Codex's full access before #532.
- **The reruns** (`just regress`, the debug build): 492, 504, 524, 567, 515 and 569 all passed
  (30, 85, 80, 49, 28 and 150 seconds): the agents' clicks in 492, 504 and 567 and the outside
  client's in 524 go at once with the use off.
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed; its
  full log is kept in the scratchpad.
- **The golden set** with 571 added: `regress: all 42 passed`, 571 in 153 seconds.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented** (§21): `CHANGELOG.md` (Added: the click consequence);
  `docs/marley/three-prong-plan.md` (S1: the click consequence shipped, the fourth use; D15 names
  the third check); `docs/marley_architecture/marley_browser.md` (consequential clicks: `classify`,
  `node_facts`, `node_at`); `marley_mcp.md` (the session's client name); `marley_system_one.md`
  (the set, its consumer, its scenario); `marley_agent.md` (`PERMISSION_MODE_LABEL`);
  `marley_workbench.md` (the pause's section); `docs/marley/guide.md` ("The click consequence",
  the card's key row, the checks' bullet). The Zed paths' rows in `docs/marley/zed-touchpoints.md`
  (55 `settings_content/src/marley.rs`, 58 `settings_ui/src/marley_page.rs`, 60
  `settings_ui/src/settings_ui.rs`, 61 `assets/settings/default.json`) were widened at promotion
  and describe what shipped.
- **Knowledge appended** (§19): F-claude-571-a-click-by-a-point-would-read-as-plain-001,
  F-claude-571-the-cards-buttons-would-hand-it-the-focus-as-it-left-001,
  F-claude-571-a-page-gone-in-a-pause-read-as-a-refusal-001;
  PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001;
  L-claude-571-a-forms-named-controls-shadow-it-even-in-an-isolated-world-001,
  L-claude-571-unreachable-pub-and-redundant-pub-crate-meet-at-a-re-export-001,
  L-claude-571-a-scenario-reads-a-pages-state-from-its-title-001;
  AD-claude-571-a-consequential-click-waits-in-the-tab-for-callers-with-no-prompt-001.
- **The brain**: consultation 16c9dadae2ab4f289a22c967fb762b19 closed with `brain_decide`
  (`decisions/marley-571-a-consequential-click-waits-in-the-browser-tab-for-callers-with-no-prompt`),
  a follow-up due 2026-10-28: the word lists and the 0.65 threshold against a month of pauses and
  their outcomes.
- **Closed**: TICKET-571 moved to `tickets/closed/`, its pipeline doc pointed at `completed/`; no
  `BACKLOG.md` row was left, and no queued spec named the queued pair.
- **The commit**: the Test phase's receipt matches the tree (docs only since the green).
