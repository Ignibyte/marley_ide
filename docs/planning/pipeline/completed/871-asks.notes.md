# The agents' questions as the boards draw them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-871-trust-and-asking-notifications.md
- **Pipeline spec:** 871-asks.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #870; split again while planning (Zed's trust modal →
  TICKET-872: a Zed hunk of its own, where the asks are Marley's code only).
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #870's roles and `render_button`; #491's bridge client for a scenario that
  calls Marley's tools.
- **Discovery:** #870's Explore inventory: `settings_change.rs:307` (`ask`, also the seats'),
  `agent_control.rs:282` (`show`), `clients.rs:653` (`show_pairing`), `agent_trust.rs:265`. Each
  builds a `MessageNotification::new_from_builder` with a headline, small lines and buttons
  (`primary_message`/`secondary_message` or its own row).
- **Decisions:** D1–D3 in the spec.

### Design
- **`asks.rs` (new):** `Ask { title, rows: Vec<(SharedString, SharedString)>, footnote:
  Option<SharedString>, buttons: Vec<AskButton> }` with `AskButton { label, role, on_click }`;
  `render(ask, cx) -> AnyElement`: the title semibold, the rows as a two-column grid (label muted,
  small), the footnote muted, the buttons right-aligned via `prompts::render_button` (made
  `pub(crate)`, `Role` with it).
- **The four builders** build an `Ask` in place of their `v_flex`: `settings_change` (rows Change
  and File; Decline quiet, Apply primary), `agent_control` (the `what` split into rows where it
  names an agent and a place; Deny quiet, the session grant primary, the project grant plain),
  `clients` (rows Code and Asked from; Refuse quiet, Read primary, the wider grants plain),
  `agent_trust` (Show Terminal plain, Trust Folder primary). Their `primary_message` /
  `secondary_message` move into the card's own buttons.
- **File manifest:** `asks.rs` (new), `prompts.rs`, `settings_change.rs`, `agent_control.rs`,
  `clients.rs`, `agent_trust.rs`, `marley_workbench.rs`; the scenario. All Marley crates.
- **Visual check plan:** REQ-001 by the settings ask and the `agent_start` ask, both raised by
  the bridge client; Day. Not reached: pairing (`client_pair` from another machine's client) and
  Claude Code's trust question (its folder-trust prompt in a worktree agent): the same `Ask`.
- **Risks:** the answer slots stay as they are; only the drawing moves.

## Phase 2 — Code
- **Built:** `asks.rs` (`Ask`: a title, lines, label and value rows, a footnote, buttons by #870's
  `Role`, `render` inside the notification's builder); `prompts::Role` and `render_button` made
  `pub(crate)`; `settings_change::ask` (the change as a line, the file as the footnote, Decline
  quiet, Apply primary; the seats reuse it), `agent_control::show` (the `what` line, the rows, a
  Project row, Deny quiet, Always for This Project plain, Allow for This Session primary),
  `clients::show_pairing` (Refuse quiet, the two wider grants plain, Read primary), and
  `agent_control::admit_asking` with `Asked { what, rows, footnote }`, which `agent_start` uses for
  Agent / Where / Prompt and the board's footnote; `admit` passes no rows.
- **Deviations:**
  - The settings ask draws its change as a line, not rows: the seats reuse its `Question` with
    other meanings for `change` and `file`, and the board draws it as a line too.
  - `agent_trust.rs` stays as it is: its buttons are the frame's `primary_message` and
    `secondary_message`, which dismiss the notification on click, and its "Show Terminal" is the
    way to its terminal; no board draws it. Noted for its screen's ticket.
- **Gate:** red once (`doc_markdown` on `MoreForms`, a private intra-doc link to `Ask`), fixed in the header, then green, 17 of 17.
- **Review:** each answer still goes through the call's slot once (`give`); the buttons keep
  their answers; nothing in Zed's crates changed.


## Phase 3 — Test
- **Scenario:** `script/e2e/871-asks.sh` (sway): two shell functions in Marley's terminal run a
  stand-in MCP client through the plugin's bridge, `settings_change` (`terminal.font_size` 15)
  and `action_run` (`workspace::NewFile`); both asked and ended `no_answer` after 25 s (the log).
- `871-01-asks`: two cards at the bottom right. "e2e wants to change your settings", the change
  `terminal.font_size: not set → 15`, the settings file muted, Decline quiet, Apply solid orange;
  "An agent in a terminal wants to use action_run", "run workspace::NewFile", a Project row, Deny
  quiet, Always for This Project filled, Allow for This Session solid orange. REQ-001 met.
- `871-02-day`: the same in Marley Day. REQ-001 met.
- **Fixes in the scenario:** the first run called from the harness, outside Marley's terminals:
  both tools refused `not_paired` (an outside program pairs first, #811) and nothing showed; its
  bare `wait` then waited on Marley itself, and the run was stopped. The client now runs in
  Marley's terminal, and the steps settle instead of waiting.
- Not reached: `agent_start`'s rows (it needs a caller agent's terminal and a reachable harness)
  and the pairing card (`marley-mcp-bridge` pairing from an outside program); both are the same
  `Ask`.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md` (The agents' questions), look-plan W3.4 status.
- Knowledge: L-claude-871-a-scenarios-mcp-client-runs-in-marleys-terminal-001,
  L-claude-871-a-bare-wait-in-steps-waits-on-marley-001. Brain: `brain decide` on consultation
  99b28d7a.
- Ticket closed; pipeline archived.
