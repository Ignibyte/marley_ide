# What agents may do, as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-938-agent-access-as-the-board.md
- **Pipeline spec:** 938-agent-access-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, AgentAccess).
- **Classification / tier:** feature, M. Marley crate (`marley_workbench`) plus one attribute in
  `settings_content`'s Marley file. The pairing card is split off as a follow-up (D6).
- **Recall (§18.3):**
  - L-claude-871 (a scenario's MCP client runs in Marley's terminal): an outside client is refused
    every tool that acts; the scripted client runs from the `.bashrc`'s shell function in Marley's
    terminal, with `terminal_env BRIDGE` and `MARLEY_MCP_ENDPOINT`.
  - #705's scenario: the scripted client, `src/main.rs` with `let answer = 42;`, and the question's
    button places (`ALLOW_X`, `ALLOW_Y`).
  - F-claude-865 (a hook called from a render never reads that view): the question is built
    before it is shown; `show`'s builder only reads the `Question` it was handed, so the terminal's
    title is read in `admit_asking`, not in the notification's render.
  - PR-claude-810 (a bar for an agent's action has a way out): the question keeps Deny and the
    two-minute expiry.
  - F-claude-903: no agent CLI starts in this run; the client is a Python script.
- **Discovery:** `agent_control.rs` (`Asked`, `Question`, `admit_asking`, `show`); `asks.rs`
  (`Ask::tile`, `Ask::change`); `editor_tools.rs` (`handle`'s `editor_edit` branch, `edit_words`);
  `click_pause.rs` (`Who::of` gives the words the tile is chosen from); `home_page.rs` (`cli_glyph`,
  `glyph_tile` made shared); `agent_activity.rs` (the new section); `clients.rs` (the level
  labels); `crates/settings_content/src/marley.rs` (`MarleyAgentControlMode::AskEvery`).
- **Decisions:** D1–D6 in the spec.

### Design
- `Asked` gains named fields `headline: Option<String>`, `task: Option<String>` and `change:
  Option<Change>`, where `Change` is a struct with `removed: Vec<String>`, `added: Vec<String>` and
  `every_match: bool` (no tuples, per the style guide). `Asked` gets `Default`, so the existing
  callers fill only what they use.
- `admit_asking` builds the headline from `asked.headline` when set, else "<who> wants to use
  <tool>". It reads the asking terminal's title once, through `mcp::caller_terminal`, and puts
  "<title> · <project>" (or the project alone) in `Question.task`; `show` then skips the
  "Project" row when a task line is present.
- `Ask` gains `tile(glyph: SharedString)` and `change(change: Change)`. `render` draws the tile
  before the title in an `h_flex`; the change as a `v_flex` of rows, each an `h_flex` with a
  one-character gutter ("−" or "+") and the line in the buffer font, `bg` from
  `status().deleted_background` / `created_background`; at most 12 rows, then "… N more lines".
- The tile's letters: a function `agent_glyph(words: &str) -> SharedString` beside the shared
  `glyph_tile` (in `home_page.rs` made `pub(crate)`, or a small `glyphs.rs` if both callers read
  better that way): "Claude Code" → CC, "Codex" → CX, "Zed's agent" → Z, else the first letter.
- `editor_tools.rs`: the `editor_edit` branch builds `Asked { headline: Some(format!("{who} wants
  to edit {file name}")), change, footnote: Some("One undo step, left unsaved. Saving asks
  again."), .. }`. `who` comes from `Who::of(call, cx).words`, as `admit_asking` does now.
- `agent_activity.rs`: `render_kept(cx)` after `render_state`: a `Label` heading, an
  `h_flex().flex_wrap()` of `Chip::new(glob)` from the raw glob strings (the settings' list, not
  the compiled `PathMatcher`), the note by `redact_secrets`, the muted settings line.
- `clients.rs:941-948`: the labels from `Power::words`, capitalised ("Read and Write"); the
  buttons' ids are unchanged.
- `settings_content/src/marley.rs`: `#[strum(serialize = "ask_every_time")]` on `AskEvery`.
  Checked: `MarleyAgentControlMode` derives only `VariantArray` and `VariantNames` from strum, and
  serde's `rename_all` is separate, so the stored value does not change. Confirm in Phase 2 that
  no code parses the mode from `VariantNames`.
- File manifest:
  - Marley crate: `crates/marley_workbench/src/agent_control.rs`, `asks.rs`, `editor_tools.rs`,
    `agent_activity.rs`, `clients.rs`, `home_page.rs`.
  - Zed crate path: `crates/settings_content/src/marley.rs` (Marley's file). Its row in
    `docs/marley/zed-touchpoints.md` gets "`MarleyAgentControlMode::AskEvery` labelled Ask Every
    Time through strum, the stored value unchanged (#938)".

### Visual check plan
- REQ-001 to REQ-004: the scripted client calls `editor_edit` on `src/main.rs` (42 → 43) from
  Marley's terminal with the editors' mode `ask_every`; `938-01-asked` shows the tile (the
  caller is "An agent in a terminal", so its letter is A), the headline, the task line, the −/+
  lines, the footnote and the buttons; Deny clicked after the shot. `938-06-day` repeats it in
  Marley Day.
- REQ-005: `marley: open agent activity`; `938-02-kept` shows the section with the eight default
  globs; `profile_setting marley.redact_secrets_for_agents false`, `938-03-unredacted`.
- REQ-006: open Settings, search "editors"; `938-04-modes` shows Ask Every Time.
- REQ-007: setup writes one paired client into the run's `clients.json` (the registry's own
  shape from `clients.rs:66`); `marley: outside clients`; `938-05-clients`.
- Not reachable in a run: a caller named "Claude Code" (it needs a real Claude Code's permission
  label on the terminal's seat). The tile's CC case is checked in the review of the diff.

### Risks
- #934 (Agent Activity lists Rusty's calls) also changes `agent_activity.rs`; whichever lands
  second rebases onto the other's rows.
- A long `old_text` could make the notification taller than the window; the 12-line cap holds it.
- Changing `Asked` touches every `admit_asking` caller (#871's delegation, actions); `Default`
  keeps their edits to nothing.
- If any code path turns `VariantNames` back into a value (a schema, a JSON hint), the strum
  rename would change what it writes; check `settings_ui`'s enum handling before relying on D4.
- Notifications sit at the window's bottom right; the shots must leave the pointer away from
  them so no hover colour shows.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
