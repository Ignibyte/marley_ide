---
pipeline_id: 6aaa0ac3-df09-424b-8095-0bc7f3e6a94b
ticket: docs/planning/tickets/closed/TICKET-571-pause-before-a-consequential-click.md
status: Phase 4 — Complete PASS
title: "A pause before a consequential click in the Browser tab"
type: feature
slice: prong 3 (the Browser tab's agent tools, after #492 and #501) with prong 2's C0; use 6 of the System One layer, on #565; narrowed by #520's terminal identity (shipped) and by #532 when it lands
references: [docs/planning/design-notes/jev-system-one-2026-09-25.md, docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/planning/pipeline/completed/565-system-one-layer.spec.md, docs/planning/pipeline/completed/520-terminal-identity.spec.md, docs/planning/pipeline/completed/569-stalled-or-looping-agents.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md, docs/planning/pipeline/completed/501-zeds-agents-drive-the-browser.spec.md, docs/planning/pipeline/queued/525-agent-drives-a-running-program.spec.md, docs/planning/pipeline/queued/532-agent-permission-modes.spec.md, docs/marley/three-prong-plan.md, docs/orca_architecture/03-browser-and-design-mode.md]
---

## Title
When an agent's `browser_click` (or a `browser_type` with a ref, which clicks first) would press
something that pays, deletes, sends in the user's name or changes an account, Marley holds the
click and asks in the Browser tab: a card naming the element and the page with Allow (Enter) and
Refuse, a workspace toast with Show, and after 25 seconds a refusal the agent can read.
Rules computed in code class the click from the element's role and name, its form and the page's
URL; #565's layer is asked, through the `click_consequence` set, only for what the rules leave
open, and may add a pause, never remove one. By default the pause applies only to agents running
without their own permission prompts.

## Scope
### In
- **`marley_browser::consequence`** (new, pure): the facts of a click target (`role`, `name`,
  `tag`, `input_type`, `in_form`, `form_action`, `form_method`, `nearby_text`, `url_path`,
  `loopback`) and `classify(facts) -> Class` (`Pays`, `Deletes`, `Sends`, `ChangesAccount`,
  `Open`, `Plain`) by the word lists and the rule table in the notes.
- **The facts at the click site** (`browser_tools::click`, and `type_text` with a ref, before
  `click_at`): for a ref, the AX role and name the ref carries, plus one CDP read of the node in
  the ref's session: `DOM.describeNode {backendNodeId}` for the tag and its attributes (`type`,
  `name`, `aria-label`, `formaction`, `formmethod`), and `DOM.resolveNode` into the page's
  isolated world, then a fixed function through `Runtime.callFunctionOn` (as `DESCRIBE_ELEMENT`
  is, never an agent's script) for the enclosing form's `action` and `method` and the text of the
  nearest form, dialog or section cut to 300 characters; for a point click,
  `DOM.getNodeForLocation` first, then the same.
- **The pause**: a `PendingClick` on the tab's page state (the call, the target's identity, the
  facts, the class and its source, a deadline) shown as a card under the toolbar in the Browser
  tab, with Allow and Refuse buttons, and as a workspace toast with Show; the card never takes the
  focus, and Enter and Esc answer it only while it holds the focus (`key_context
  ("MarleyBrowserPause")`, after a click on it or the toast's Show). At Allow the target is
  resolved again and, when it still has the same role and name on the same URL, clicked, else
  refused with "the page changed"; Refuse, Esc on the focused card or 25 seconds answer the call
  `Err("Marley paused this click: <sentence>; <why it did not happen>")`, which the server returns
  as `isError` with `result: refused`. While a pause waits on a tab, the tab's other write tools
  (the `WRITES` set, `browser_navigate` and `browser_check_pick` on that tab) are refused with "a
  click is paused in this tab; wait for the user". The flight recorder gets an `Agent` entry for
  the pause and its end.
- **Who is paused** (`marley.browser_click_pause_agents`: `agents_without_prompts`, the default,
  or `all_agents`): the workbench decides prompt-less by D2's rule from the call's calling terminal
  (#520's `Caller`) and its Claude Code seat, or, with no terminal, from the session's client name,
  which `marley_mcp` now keeps from `initialize`'s `clientInfo` and carries on `AppCall`
  (`client: Option<String>`); unknown counts as prompt-less.
- **The model**: for `Open` only, `system_one::ask` with the set `click_consequence/1` (four
  nouls: pays, deletes, sends in the user's name, changes an account), the state of D6, and a
  1.5 s deadline (the layer's one retry inside it); in `act` a noul that holds pauses as the rule
  would; in `suggest` the click goes and a toast names the element and the reading; in `shadow`
  the reading is logged. A rule's pause is logged as a `rules` row. Outcomes: `allowed`,
  `refused`, `expired`, `changed`, and for an unpaused click whether the user took the tab's
  focus or navigated back within a minute.
- **Settings**: the use (`UseSpec { name: "click_consequence", deadline: 1.5 s }` beside
  `STALL_KIND`), its mode in `marley.system_one.uses` (`off` by default), its dropdown on the
  Marley page's System One section; and `marley.browser_click_pause_agents`
  (`MarleyClickPauseAgents`) in `MarleySettingsContent`, `default.json`, the Marley page's Agents
  section (a dropdown) and `settings_ui.rs`'s dropdown renderers.
- `script/e2e/571-pause-before-a-consequential-click.sh`.

### Out (explicitly deferred)
- Pausing typed text, key presses, scrolls, navigation to an address (a logout URL) and picks;
  the Agent Panel's own confirmations (Zed's `authorize_third_party_tool` stays the first check).
- A host exemption (loopback dev servers) or a per-site allow list; a "for this page" or "for this
  agent" memory of an Allow: every consequential click asks.
- #532's flag for Codex's full access: until it lands, a Codex terminal is unknown and paused.
- #524's outside clients: they arrive through the same tools with a name of their own, so the
  rule covers them by construction; nothing more here.
- Clicks by the user's hand, and Zed Agent threads' clicks under Zed's own prompt.

## Reference (§20)
- **Warp:** its agent permissions run each command past two regex lists; the denylist "always
  asks and beats both the allowlist and Agent Decides", and each capability can be set to Always
  ask (docs.warp.dev/agents/capabilities/agent-profiles-permissions/,
  docs.warp.dev/agents/cli/permissions-and-profiles/, as `warp-blocks-and-natural-language-2026-09-25.md`
  records them). Marley's rules are that denylist for clicks, with the model as a second signal
  "while the lists stay the gate" (the same note, "Where a System One model fits"). Nothing of
  Warp's was read.
- **Marley's own D15** (`docs/marley/three-prong-plan.md`): two checks an agent cannot skip, "the
  client's approval of each write call (Claude Code asks by default) and the Browser tab, where
  every agent action happens in front of Chad". This ticket adds the third, for clients with no
  first check, in the same tab.
- **Claude Code's auto mode** (anthropic.com/engineering/claude-code-auto-mode) is the approver
  Chad keeps for Marley's terminals (his answer to the Jev note's question 5), which is why the
  default pauses only agents without a prompt of their own.
- **Orca** (report 03 §2.13): computer use's skill "forbids sending, submitting, buying or
  deleting unless asked. There is no per-action approval." Marley adds the per-action pause.
- **Upstream Zed:** the Agent Panel's third-party tool authorization
  (`crates/agent/src/tools/context_server_registry.rs:349`, the options in
  `crates/agent/src/thread.rs:5767-5814`), kept as a Zed thread's first check.

### Prior art
- **Behavior maps and reports.** Report 03 §2.11 (Orca's agent browser: eighty commands, error
  codes an agent can branch on such as `browser_stale_ref`; the stale-ref recovery by role, name
  and occurrence, which D3's re-resolution echoes) and §2.13 (above). The Jev note's safety rules
  1 (code decides the dangerous classes first; the model "can only add caution"), 2 (an acting
  decision needs code's classification and a probability above threshold; "Jev alone never
  authorizes"), 3 (irreversible actions go to a person: "spending, messages to people"), 5 (a
  verdict binds to the exact call: "classify last, hash the exact `tool_input`") and 6 (hostile
  text stays out of acting decisions; "after three automatic refusals in a row the session goes
  back to Chad"), and its budget row for reviewer-less approvals (a 1.5 s hard deadline, one
  retry, up to 2k tokens, no tool output). #565's layer (`UseSpec`, `ask`, `StateBuilder`,
  `Reading`, the providers, the day's file).
- **Published material.** CDP: `DOM.describeNode`, `DOM.getNodeForLocation`,
  `Runtime.callFunctionOn`, `Input.dispatchMouseEvent`. MCP 2025-06-18 tool results with `isError`
  and `structuredContent`; the `initialize` request's `clientInfo`. TypeSafe's docs as the note
  cites them (a noul "phrased so high means yes"; the 0.35 to 0.65 band as no signal;
  docs.typesafe.ai/api, the note's "Abstain bands and thresholds"). The action-gate study the
  note cites (111 cases: Jev 100 right, one unsafe allow, as Claude Opus 5 had) for why the rules
  stay the gate. langchain-ai/langchain#40694 (a call replaced after it was classified) for D3.
- **The code we already ship.** `browser_tools.rs`: `run` (`:63-111`), `WRITES` and
  `show_for_agent` (`:42-49`, `:85-87`), `click` (`:494-532`), `target_point` (`:690-709`),
  `ref_target` (`:678-686`), `ref_origin` (`:760-795`), `click_at` (`:876-901`: three
  `Input.dispatchMouseEvent`), `type_text` clicking first (`:550-555`), `done` (`:394-406`),
  `acting` (`:409-428`); `marley_browser::snapshot::RefTarget` (`snapshot.rs:113-128`:
  `backend_node_id`, `role`, `name`) and `describe` (`:130-140`); `page.rs`: `DESCRIBE_ELEMENT`
  (`:29-31`) and `focused_element` (`:758-819`, the fixed-function template), `DOM.describeNode`
  (`:783-785`), `scroll_into_view` (`:651-663`), `box_center` (`:670-679`); `browser.rs`:
  `AgentAction` (`:130-136`), `agent_started` and `agent_ended` (`:1609-1667`, the recorder's
  `Agent` entry at `:1633-1638`), the chip in `render_toolbar` (`:4130-4150`, display only),
  `render_dialog` (`:4357-4434`: a modal with Enter and Esc through `MarleyBrowserDialog` in
  `crates/marley_workbench/keymap.json:84-96`, the card's template), `render_tray` (`:4438-4483`),
  `record_this`'s toast (`:3779-3780`); `marley_mcp`: `ToolSpec.grant_class` and `browser_write`
  (`registry.rs:46-57`, `:233-241`), `decide` (`permission.rs:54-64`), `tool_error` and
  `tool_result` (`tools.rs:62-64`, `:18-25`), `APP_CALL_TIMEOUT_SECONDS` 30 (`marley_mcp.rs:81`),
  `PendingCall` and `AppCall` (`:120-137`, tool and arguments only), `SessionEntry`
  (`session.rs:35-38`, id and last-seen only), `initialize` dropping its params
  (`dispatch.rs:27`) while the bridge forwards the client's own (`marley-mcp-bridge`, `connect`);
  #525's queued design (the card in the target's footer, a toast with Show, a 25 s wait under the
  transport's 30, the refusal's shape); Zed's `agent_settings` `tool_permissions` (`ToolPermissions`,
  `crates/agent_settings/src/agent_settings.rs:474-478`; `always_allow_tool_actions` is gone); #520's
  `Caller` and `mcp::caller_terminal` (`mcp.rs:749-758`); #519's `permission_mode` label
  (`claude_events.rs:263-274`); `redact_url` (`observe.rs:287`) and `Redactor`. Does a crate we
  build own the seam? `marley_browser` owns the CDP facts, `marley_mcp` the refusal's shape, #565
  the ask; nothing owns the classes, the pause or the caller's name.

## UI proof
UI-AFFECTING: a card and a toast in the Browser tab, its buttons and keys, the tool's answers.
The shot list and checks below are the draft's; the notes' "Changed at promotion" and E2E plan
override them where they differ (the refusal's printed shape, the buttons clicked, the terminal
and Zed callers, the settings shot).
`script/e2e/571-pause-before-a-consequential-click.sh` (`compositor sway`: the card, the toast and
the tab are clicked; the offline Chromium). Fixtures: `serve_site shop`, a page with a cart form
(`POST /checkout`, a submit button "Place order"), a "Remove item" button, a "Next" link, a
"Continue" button under the text "You will be charged $12.00", a second "Continue" with no such
text, a settings section with "Delete account", and a "Swap" button that renames "Place order" to
"Cancel order" in place; every click appends to a `<p id="log">` and the page never leaves; the
stand-in `mcp_agent` (its `clientInfo` is `e2e`, unknown to Marley) clicking in the background;
`marley.browser_click_pause_agents` at `all_agents`; #565's layer enabled on `replay` with the
scratch repository listed and `uses.click_consequence` rewritten per step by
`system_one_setting`; the replay file at `$E2E_PROFILE/system_one/replay.jsonl` (the first
Continue: `pays` 0.88; the second: 0.5, no signal). Shots:
- `571-01-paused`: the mode `shadow`; `click-on button "Place order"`: the card ("The agent wants
  to click Place order in a form that posts to /checkout on 127.0.0.1"), the toast, the page's
  log empty.
- `571-02-allowed`: Enter: the page's log reads `Place order`; the stand-in's output: `clicked`.
- `571-03-refused`: `click-on button "Delete account"`, Esc: the log unchanged; the stand-in's
  output: `refused: Marley paused this click: … the user refused`.
- `571-04-open-case`: the mode `act`; `click-on button "Continue"`: paused, the card naming the
  reading (`pays, 0.88`).
- `571-05-plain`: `click-on link "Next"`: no card, the log reads `Next` at once.
- `571-06-expired`: `click-on button "Remove item"`, 26 s: the card gone, the stand-in's output
  `refused: … did not answer within 25 seconds`, the log unchanged.
- `571-07-no-signal`: the second Continue: no card, clicked at once; the day's file has the row.
- `571-08-changed`: `click-on button "Place order"`, then the scenario clicks "Swap" by the
  pointer, then Enter: nothing clicked, the stand-in's output `refused: … the page changed`.
- `571-09-blocked-writes`: during a pause, `mcp_agent scroll 100`: `refused: a click is paused`.
- `571-10-unknown-caller`: `agents_without_prompts`: the stand-in's Place order still pauses.
- `571-11-off`: the mode `off`: clicked at once, no card.
Checks: the stand-in's outputs as above; the day's file's rows name `click_consequence/1` and the
outcome of each pause; `mcp_agent recording` after a `record_this` shows the pause's entry; in
`571-01` the Decisions view lists the rule's row.

## Locked-In Decisions
- D1 — "local first and then jev second" (Chad, 2026-09-26). The classes come from the element,
  its form and the URL; only an open target is asked, and a reading may add a pause and never
  remove one (the note's rules 1 and 2: code classifies first, the model "can only add caution").
  With the `rules` provider, the project unlisted (`Refused`), the provider unreachable
  (`Unavailable`) or the reading `NoSignal`, the rules' pauses stand and an open click goes as
  today.
- D2 — Off by default, with its own switch and its own mode, and no provider named: "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system" (Chad). The switch is the use's mode
  in `marley.system_one.uses` (#565's shape). On, it pauses "by default only for agents running
  without their own prompts" (the Jev note's use 6, approved), decided per call (changed at
  promotion, #520 having shipped): a call from a Marley terminal whose Claude Code seat's
  `permission_mode` is `bypassPermissions` or `dontAsk` is prompt-less, and `default`,
  `acceptEdits`, `plan` or `auto` prompt (`auto` keeps its classifier, Chad's default); a call from
  a terminal with no Claude Code seat (Codex until #532, a script) is unknown; a call with no
  terminal from a session whose client named itself `Zed` is prompt-less while Zed's
  `agent.tool_permissions` let `mcp:marley:browser_click` run without asking (that tool's own
  default, else the global one, is `allow`); any other call, #524's outside clients and harness
  actors included, is unknown, and unknown pauses. `all_agents` pauses everyone.
- D3 — The verdict binds to the exact call (the note's rule 5): the pause holds the ref's node,
  role, name and URL; Allow resolves the target again and clicks only the same element on the same
  page; anything else is refused with the reason. A pause is per tab and refuses the tab's other
  writes, so a click by coordinates cannot go around it.
- D4 — 25 seconds, under the transport's 30 (#525's D7); an unanswered pause refuses. The agent
  reads a refusal it can act on: `isError`, `result: refused`, and a reason that says the user
  did not allow it, so its next move is to ask, not to retry.
- D5 — The card sits in the Browser tab, where every agent action happens in front of Chad (D15),
  and a workspace toast with Show points at it; Marley moves no focus by itself (L-claude-493,
  #525's D6). The card never takes the focus, since an agent's action must not move the user's
  (F-claude-495) and an Enter meant for the page must never allow a click: Allow and Refuse are
  its buttons, and Enter and Esc answer only while the card itself holds the focus, after a click
  on it or the toast's Show (changed at promotion; the draft bound them to the whole tab).
- D6 — The state is the URL (through `redact_url`), the page's title, the element's role, name
  and tag, the form's action and method, the nearby text (through #516's redactor as the layer's
  `Mask`, 300 characters), and the facts; only for a listed project (the tab's workspace's);
  `Detail::Facts` (role, tag and the URL's host and path) for a metadata-only project. The
  element's name and the page's text are the page's own words, so their reading may only add a
  pause (the note's rule 6); a hostile page can at worst talk the model out of a pause the rules
  never made.
- D7 — Modes as #565 defines them: `off` makes no pause and no call; `shadow` pauses on the rules
  and logs the model's reading of an open target; `suggest` does as `shadow`, and after an open
  target's click that the model reads consequential, a toast names the element and the reading;
  `act` pauses an open target on a noul that holds (above 0.65, the layer's band; refitted from
  the outcomes). A noul in the band, no signal, `Refused` or `Unavailable` is today's click. Only
  open targets are asked (D1); a rule's pause is a `rules` row.
- D8 — No host exemption in this slice: a local app can charge a real card through a payment
  provider's frame or send real mail. If the pause proves noisy on dev servers, a later ticket adds
  the exemption with its own switch.
- D9 — `clientInfo.name` and #520's terminal id are courtesy identities with the bearer's trust,
  not a security boundary (`Caller` is "never an authority"): a process with the bearer can claim
  any name or terminal, as it can call any tool. They sort callers into "asks first" and "does
  not", which is what the default needs; `all_agents` is the setting for anyone who wants no
  sorting.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the use is not `off` and the setting applies to the caller, WHEN an agent's click names an element the rules class as pays, deletes, sends or changes an account, the system shall click nothing and show the card naming the element and the page, with Allow and Refuse, and a toast with Show. | Shot `571-01-paused`; the page's log |
| REQ-002 | WHEN the user allows, the system shall make the click and answer the tool as a click. | Shot `571-02-allowed`; the stand-in's output |
| REQ-003 | WHEN the user refuses, the system shall answer the tool with `isError`, a refused result and the reason, and click nothing. | Shot `571-03-refused`; the stand-in's output |
| REQ-004 | WHEN 25 seconds pass unanswered, the system shall refuse the same way and remove the card and the toast. | Shot `571-06-expired`; the stand-in's output |
| REQ-005 | WHERE the mode is `act` and the project listed, WHEN the rules leave a target open and a noul reads at or above its threshold, the system shall pause as for a rule. | Shot `571-04-open-case`; the day's file |
| REQ-006 | WHEN the rules class a target as plain, the system shall click at once and ask nothing. | Shot `571-05-plain`; the day's file has no row |
| REQ-007 | WHEN the reading is in the band, no signal, refused or unavailable, the system shall click at once and log the call. | Shot `571-07-no-signal`; the day's file |
| REQ-008 | WHEN the element or the page changed under the pause, Allow shall click nothing and refuse with the reason. | Shot `571-08-changed`; the stand-in's output |
| REQ-009 | WHILE a pause waits on a tab, the system shall refuse the tab's other write tools with a reason. | Shot `571-09-blocked-writes`; the stand-in's output |
| REQ-010 | WHERE the setting is `agents_without_prompts` and the caller is unknown, the system shall pause. | Shot `571-10-unknown-caller` |
| REQ-011 | WHERE the mode is `off`, the system shall click at once with no card and no call. | Shot `571-11-off`; the day's file |
| REQ-012 | WHEN a pause starts and ends, the flight recorder shall carry an entry for each. | The run log: `mcp_agent recording` |
| REQ-013 | WHEN a session initializes, the server shall keep its client's name and carry it on each of its calls. | Review of the diff; REQ-017's run: a caller named `Zed` is sorted by Zed's permissions |
| REQ-014 | WHEN the Marley settings page opens, its System One section shall show the use's mode and its Agents section `Browser Click Pause Agents` with its two values. | Shot `571-15-settings` |
| REQ-015 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` (492, 504, 524 and 567 click through the agent path) |
| REQ-016 | WHERE the setting is `agents_without_prompts`, WHEN a Claude Code in a Marley terminal whose seat's permission mode prompts clicks a consequential element, the system shall click at once, and WHEN the mode is `bypassPermissions` it shall pause. | Shots `571-12-claude-prompts`, `571-13-claude-bypass`; the stand-in's output |
| REQ-017 | WHERE the setting is `agents_without_prompts`, WHEN a caller with no terminal that named itself `Zed` clicks a consequential element, the system shall pause only while Zed's tool permissions let the tool run without asking. | Shot `571-14-zed`; the stand-in's outputs |

## Phase Plan
- **P1 Plan:** this spec; the design and the e2e plan in the notes. At promotion: #565 and #520
  have shipped, #532 has not; D2 narrows to the calling terminal; `brain_ask`.
- **P2 Code:** the ledger rows first (`crates/settings_content/src/marley.rs`,
  `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs`,
  `assets/settings/default.json`); the client's name in
  `marley_mcp`; `marley_browser::consequence` and the facts read; the pause, the card, the toast,
  the blocked writes and the recorder entry in `browser.rs` and `browser_tools.rs`; the
  `click_consequence/1` set in `marley_system_one`'s root; the caller rule and the settings.
  fmt and clippy clean; a review of the diff with the security lens: nothing clicked without
  Allow while a pause holds, and no reading ever removes a rule's pause.
- **P3 Test:** write and run the scenario and read every shot; rerun 492, 504, 524 and 567 (their
  agents' clicks must go at once with the use off, and 524's outside client's too); `just regress`;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_browser.md`, `marley_mcp.md`,
  `marley_workbench.md` and `marley_system_one.md`; the plan's D15 paragraph names the third
  check; ledger capture; close the ticket, archive, commit.
