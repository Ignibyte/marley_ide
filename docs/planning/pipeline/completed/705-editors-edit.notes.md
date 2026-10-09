# Editors over MCP: edit and save — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-705-editors-edit.md
- **Pipeline spec:** 705-editors-edit.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: full control of Zed over MCP; the intake's third ticket.
- **Classification:** feature, Marley crates only.
- **Recall (§18.3):**
  - AD-704 sets the modes and the once-per-session question.
  - The intake puts a save at Sensitive (asking every time).
  - The Explore pass: Zed's agent edits as one `Agent`-sourced transaction, and its review UI is
    not exported.
- **Discovery:**
  - `language::BufferEditSource::Agent`; `Buffer::{start_transaction, edit,
    end_transaction_with_source}`; `Project::save_buffer`;
  - `editor_tools.rs` (#704): `OpenEditor` gains its workspace for the save.

### Design
- **`registry.rs`:** `editor_edit` and `editor_save`, both Write and `editor.write`, with their
  schemas. The pinned list and the count (46) are updated.
- **`agent_control.rs`:** `Level { Read, Act, Sensitive }` replaces `acts: bool`. Under `ask_first`
  a Sensitive call always asks; under `allow` nothing asks.
- **`editor_tools.rs`:**
  - `OpenEditor.workspace`.
  - `edit`: `editor_named`, the secret globs, then the matches of `old_text` in `buffer.text()`.
    None refuses `no_match`; more than one without `replace_all` refuses `ambiguous` with the
    count. Otherwise one transaction edits every match in place (offsets, last first), and the
    answer gives the replacements, the first changed line and `dirty`.
  - `save`: `editor_named`, admitted as Sensitive ("save <path>"), then
    `project.save_buffer(buffer)`; it answers `saved: true` or the error.
- **No settings change** (the levels ride on the editors mode).
- **File manifest:** `marley_mcp/src/registry.rs`; `marley_workbench/src/agent_control.rs` and
  `editor_tools.rs` (Marley crates); the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `editor_open` main.rs (question → Allow for This Session), `editor_edit` 42→43 | 705-01-edited; the disk still says 42 |
| 002 | `editor_edit` with `old_text` "nowhere" | the reply `no_match` |
| 003 | `editor_save` in the background | 705-02-save-asked |
| 003 | Allow | 705-03-saved; the disk says 43 |
| 001 | `editor_edit` 43→44, a click in the editor, Ctrl+Z, `editor_read` | the read says 43 |

### Risks
- **A buffer that changed between the agent's read and its edit:** the exact match fails,
  `no_match`, rather than editing the wrong place.
- **A save with a conflict on disk:** `save_buffer`'s error is answered as is.

## Phase 2 — Code
- **Built:**
  - `registry.rs`: `editor_edit` and `editor_save` (Write, `editor.write`). Their schemas are in a
    new `editor_change_schemas`; `editor_schemas` passes `edit` and `save` to it, which keeps it
    under clippy's 100 lines. The pinned list and the count (46) are updated.
  - `agent_control.rs`: `Level { Read, Act, Sensitive }` replaces `acts: bool`. A Sensitive call
    asks every time unless the area is `allow`, and `off` refuses all.
  - `editor_tools.rs`:
    - `OpenEditor.workspace`, and `not_secret`, now shared by read and edit.
    - `edit`: exact matches of `old_text`. None refuses `no_match`; several without `replace_all`
      refuse `ambiguous`. The rest are edited in one `Agent` transaction, and the answer gives
      `replaced`, `first_line` and `dirty`.
    - `save`: `Project::save_buffer` on the editor's workspace's project.
    - `edit_words`, `save_words` and `editor_words` give the question its text.
  - `guide.md`: the two tools' lines.
- **Deviation:** none.
- **Review of the diff:**
  - The edit runs after the admit, inside `cx.update`, so the buffer isn't being updated when it
    is.
  - Matches don't overlap (`match_indices`) and are edited in one call.
  - A save's error reaches the agent.
  - Both tools are write-tier in a listed family, so #703 logs them and the kill switch stops them.
- **Gate:** `705-gate-1.log` RED (clippy: `editor_schemas` at 133 lines); `705-gate-2.log` RED
  (`map_or_else`); `705-gate-3.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/705-editors-edit.sh`, under `compositor sway`, #704's scripted client
  with `edit` and `save`. Questions are answered by a click on Allow for This Session.
- **First run (`shots-705a`): every check passes, every shot shows its criterion.**
  - **Checks:**
    - `editor_open` opened `main.rs` after Allow;
    - `editor_edit` → `edited replaced=1 line=2 dirty=True`, with the file on disk still at 42;
    - an `old_text` found nowhere → `refused no_match`;
    - `editor_save` → `saved=True` after Allow, and the disk says 43;
    - a second edit (43→44), a click in the editor and one Ctrl+Z, then `editor_read` says 43.
  - **705-01-edited (REQ-001):** `main.rs` reads `let answer = 43;`, and its tab has the unsaved
    dot.
  - **705-02-save-asked (REQ-003):** the question "e2e-agent wants to use editor_save", "save
    …/repo/src/main.rs", with the three buttons, although the session was allowed at the open.
  - **705-03-saved (REQ-003):** the same text, the tab without its dot.
  - REQ-002's `ambiguous` branch shares `no_match`'s path in `edit`; the review covers it.
  - Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (#705's
  lines in the editors section) and `marley_mcp.md`; the guide (Phase 2). No Zed path changed.
- **Knowledge appended:** AD-claude-705-agent-edits-are-exact-replacements-and-saves-always-ask-001.
  No bug was found.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decision is in the ledger.
- **Ticket:** closed; the BACKLOG row left at promotion.
- **Gate:** `705-gate-4.log`, GATE GREEN [diff], on the tree committed (the scenario included).
