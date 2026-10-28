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
- **Promoted 2026-10-28, seams re-checked at HEAD f205bef7ef.** Most hold. Two moved since the
  draft:
  - The tile: #905 and #933 added `item_kind::AgentMark` (`glyph`, `ink`) and `item_kind::square_tile`,
    and `click_pause::Who` now carries `mark: Option<AgentMark>`. The question's tile is therefore
    `square_tile` from `who.mark`, falling back to the words' first letter. D5 is met by the
    shared `item_kind` functions, and Home's private `glyph_tile` is left alone.
  - `agent_activity.rs`: `render_state` is now a free `pub(crate) fn render_state(id, cx)` that
    Home's card shares (`:698`). The tab's body is at `:916-918`, and "Kept from agents" goes
    after `render_state` there, not inside it.
  - `clients.rs`'s level buttons are at `:945-947`.
  - The brain (consultation 7d2964ea9a264c658d03479806fd17f3) had nothing on this seam.

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
- **Built:**
  - `asks.rs`: `Change { removed, added, occurrences }` with `Occurrences { One, Every }`, an
    enum in place of a `replace_all` bool. `Ask` gains `tile(AgentMark)` (an
    `item_kind::square_tile` before the title), `subtitle` (a muted line) and `change`.
    `render_change` draws each removed line as "−" on `status().deleted_background` and each added
    line as "+" on `created_background`, in the buffer font, at most `CHANGE_LINES` (12) and then
    "… N more lines". With `Every`, "Every match in the file." follows.
  - `agent_control.rs`: `Asked` derives `Default` and gains `edit: Option<EditAsked { file_name,
    change }>` (D1). `admit_asking` builds the edit's question: "<who> wants to edit <file>", the
    tile from `Who.mark` or the words' capital (`initial`), and the task line from `task_line`
    (the caller terminal's `tab_content_text` · project). `show` draws the task in place of the
    Project row.
  - `editor_tools.rs`: `edit_asked` replaces `edit_words`, and `after_admit_asking` sits beside
    `after_admit`. The footnote reads "One undo step, left unsaved. Saving asks again."
    `secret_glob_list` is split out of `secret_globs` for Agent Activity.
  - `agent_activity.rs`: `render_kept` after `render_state`: "Kept from agents", the globs as
    `Chip`s, the redaction note by `MarleySettings.redact_secrets`, and the settings line. The tab
    already redraws on `SettingsStore`.
  - `clients.rs`: the level buttons read Read, Read and Write, and Read, Write and Run.
  - `delegation.rs`: `..Asked::default()`.
  - `settings_content/src/marley.rs` (Zed crate path): `#[strum(serialize = "AskEveryTime")]` on
    `AskEvery`, with its ledger row written first. Only `settings_ui`'s `render_dropdown` reads
    `VariantNames`, so the stored `ask_every` is unchanged (D4).
- **Deviations:**
  - The tile is `item_kind`'s, not Home's `glyph_tile` (see Phase 1's promotion note).
  - The headline is built in `admit_asking` from `EditAsked`, not by `editor_tools`.
  - The line "change <file>" is gone from the edit's question, as the spec says.
- **Gate:** the first run failed on `doc_markdown` (`AgentAccess` in backticks). The second is
  GREEN, with the scenario in place.

## Phase 3 — Test
- **Scenario:** `script/e2e/938-agent-access-as-the-board.sh` (sway).
  - The stand-in client (#871's `ask.py`) runs from `.bashrc` functions in Marley's terminal.
  - The editors' mode is `allow` for `editor_open`, then `ask_every` for `editor_edit`.
  - One paired client, `laptop` with power `write`, goes in `$E2E_PROFILE/mcp/clients.json`.
  - The first run (`STOP_AT=1`) measured Deny at (1199, 929).
- **Red, fixed:** that first shot's task line showed the project as a full path
  (F-claude-938-a-projects-key-is-its-path-not-its-name-001). `task_line` now takes the folder's
  name, and the scenario ran again.
- **`938-01-asked` (REQ-001 to REQ-004):** the card reads "A" on a tile, then "An agent in a
  terminal wants to edit main.rs". Under it, muted, is "repo — bash · repo". Then "− let answer =
  42;" on dark red and "+ let answer = 43;" on dark green in the buffer font, "One undo step, left
  unsaved. Saving asks again.", and Deny / Always for This Project / Allow for This Session.
- **`938-02-day`:** the same card in Marley Day, on light red and light green.
- **Deny clicked:** the checks "the edit was denied" (`agent_control_declined` in the log) and "the
  file still says 42" passed.
- **`938-03-kept` (REQ-005):** "Kept from agents" sits under the Stop row, with the eight chips
  (.env*, *.pem, *.key, id_rsa*, id_ed25519*, *credentials*, .netrc, *.p12), "Keys and tokens in
  terminal output become [redacted: token] before an agent reads them. Your terminal still shows
  them." and the muted settings line.
- **`938-04-unredacted` (REQ-005):** "Redaction is off: agents read terminal output as it is."
- **`938-05-modes` (REQ-006):** the Settings window's Editors row reads Off / Ask Every Time / Ask
  First / Allow, with Ask Every Time filled (the run's `ask_every`), so the stored value still
  reads back.
- **`938-06-clients` (REQ-007):** laptop's levels read Read / Read and Write / Read, Write and Run,
  with Read and Write picked, and Cut Off.
- **Seen, not in scope:** Agent Activity's rows show a file's whole path. That has been so since
  #933.

## Phase 4 — Complete
- **Documented (§21):**
  - `CHANGELOG.md`: an Added entry for #938.
  - `docs/marley/look-plan.md`: the Tools row names #938, with the pairing card to come.
  - `docs/marley_architecture/marley_workbench.md`: "An edit's question" and "Kept from agents"
    under the agents' questions.
  - `docs/marley/zed-touchpoints.md`: the `settings_content/src/marley.rs` row (written in Code).
- **Knowledge (§19):** F-claude-938-a-projects-key-is-its-path-not-its-name-001. No `PR-` block:
  it was one label, fixed where it is built. No `AD-` block.
- **Brain:** consultation 7d2964ea9a264c658d03479806fd17f3 closed with `brain no-decision`.
- **Follow-up:** TICKET-947, the pairing card (D6), minted and queued after #946.
- **Ticket:** closed. No row was left in BACKLOG.md.
