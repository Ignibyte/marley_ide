# A pause before a consequential click in the Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-571-pause-before-a-consequential-click.md
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

### Design
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
- **File manifest.** Marley: `crates/marley_browser/src/consequence.rs` (new), `marley_browser.rs`
  (the module), `page.rs` (`FORM_FACTS`, `node_at`), `snapshot.rs` (`RefTarget` describes with
  its backend node; unchanged otherwise); `crates/marley_workbench/src/browser_tools.rs`,
  `browser.rs`, `marley_workbench.rs` (the actions, the setting), `keymap.json`;
  `crates/marley_mcp/src/session.rs`, `transport.rs`, `dispatch.rs`, `marley_mcp.rs`;
  `crates/marley_system_one/src/question.rs` (the `click_consequence/1` set) and the workbench's
  `system_one.rs` (the `UseSpec`); `script/e2e/browser-fixture.sh` (nothing new; `click-on`
  already prints refusals); `script/e2e/571-pause-before-a-consequential-click.sh`. Zed:
  `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
  `assets/settings/default.json`.
- **Ledger rows.** The three Zed paths' rows gain `browser_click_pause_agents` and the use's
  dropdown (#571) before their edits.

### E2E plan
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
| REQ-014 | the golden set's 515 run | its page shot |

Not reachable by a scenario: a Zed Agent thread with a model behind it (D2's Zed rule is proved
by review), a real Claude Code seat in `bypassPermissions` (#519's stand-in can send the label,
which Test uses for the Claude Code half of D2), and #520's terminal identity before it lands.

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
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
