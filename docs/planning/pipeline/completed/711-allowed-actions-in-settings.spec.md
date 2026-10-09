---
pipeline_id: e0815130-d857-4f16-ae48-0762c9cdc79e
ticket: docs/planning/tickets/open/TICKET-711-allowed-actions-in-settings.md
status: Phase 4 — Complete PASS
title: Allowed actions in Settings
type: feature
slice: #707's follow-up
references:
  - docs/planning/pipeline/completed/707-palette-actions.spec.md
---

## Title
The Marley page edits `marley.agent_control.actions_allowed` in place.

## Scope
### In
- **`MarleyActionNames`** in settings_content: a transparent newtype over `Vec<String>`, so a
  renderer of its own applies. The JSON is unchanged, and a later file still replaces an earlier
  one's list, as a `Vec` does.
- **The renderer** in `marley_page.rs`, registered in `settings_ui.rs`:
  - a row per name, with a remove button;
  - a "not an action" mark on a name `cx.all_action_names()` lacks;
  - a `SettingsInputField` (placeholder `editor::SelectAll`, confirm button) that appends a
    trimmed name once.
- **The Agent Control section's Allowed Actions item.**
- **`action_tools::user_allowed`** reads the newtype.

### Out (explicitly deferred)
- Choosing names from a picker of every action.
- Marking a name the hard refusal covers: settings_ui can't depend on `marley_workbench`, so
  `action_list` and `action_run` keep reporting those.

## Reference (§20)
Upstream Zed, settings_ui:
- `SettingsInputField`, `update_settings_file` and `add_basic_renderer`, as
  `render_text_field` uses them;
- `PathHyperlinkRegexes` in settings_content: a transparent newtype with its own `MergeFrom`.

### Prior art
- **Zed's own string lists** (`file_scan_exclusions` and the like) render
  `.unimplemented()`, which is an "Edit in settings.json" button. No list editor exists to
  reuse.
- **The parts reused:** Marley's `MarleyAgentControlMode` renderer (#704) and Zed's input
  field.

## UI proof
`script/e2e/711-allowed-actions-in-settings.sh`, under `compositor sway`. The profile names
`editor::SelectAll` and `nothing::Here`; `marley: open settings` opens the window, and its
search finds "Allowed Actions".

Shots:
- `711-01-listed`: both names, with `nothing::Here` marked "not an action";
- `711-02-added`: `pane::SplitRight` typed and confirmed, now a row;
- `711-03-removed`: `nothing::Here`'s remove clicked, the row gone.

After each shot, the run's settings.json is checked for the list.

## Locked-In Decisions
- **D1:** a newtype, not a renderer for every `Vec<String>`, so only this field gets the editor.
- **D2:** an add appends a name once; a blank or duplicate name is ignored.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Allowed Actions item shall list each name in `actions_allowed` with a remove button, marking a name no action is registered under. | Shot 711-01 |
| REQ-002 | WHEN the user confirms a name in its field, the system shall add it to `actions_allowed` once. | Shot 711-02 and settings.json |
| REQ-003 | WHEN the user clicks a name's remove button, the system shall take it out of `actions_allowed`. | Shot 711-03 and settings.json |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan.**
- **P2 Code:** the newtype, the renderer, the item, `user_allowed`; the gate.
- **P3 Test:** the scenario.
- **P4 Complete.**
