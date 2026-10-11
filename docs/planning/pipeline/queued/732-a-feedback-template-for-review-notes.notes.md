# A feedback template for review notes — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-732-a-feedback-template-for-review-notes.md
- **Pipeline spec:** 732-a-feedback-template-for-review-notes.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 7 (`docs/planning/intake/plannotator-findings.md`): "#522's
  message to the agent from a template the user can set, by default asking the agent to verify each
  note, give a verdict, and not widen the review." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "11.) yes". It waits under Deliberate
  with the rest of the review batch until after the finish.
- **Classification:** feature, small. No Zed crate is touched beyond the `marley` settings file,
  which already has its row.
- **Recall (§18.3):**
  - AD-claude-522-review-notes-go-to-an-idle-terminal-agent-and-stay-marked-sent-001: the send path
    this wraps; the notes stay marked Sent; Copy marks nothing.
  - PR-claude-keys-after-a-paste-go-in-a-later-write-001: the Enter after the paste goes through
    `terminal_drive::paste_then`. The template adds lines to the paste and nothing after it.
  - L-claude-587-a-sed-template-needs-g-for-a-placeholder-twice-on-a-line-001: the scenario's fakes
    take `g` on every substitution.
  - No ledger entry on message templates.
- **Discovery:**
  - `crates/marley_agent/src/marley_agent.rs`: `ReviewLine` (350), `review_prompt` (371),
    `escape_comment` (392).
  - `crates/marley_workbench/src/review_notes.rs`: `deliver` (108; Copy at 119, the paste at 151),
    `prompt` (162).
  - `crates/marley_workbench/src/marley_workbench.rs`: `MarleySettings` (370), `from_settings` (634).
  - `crates/settings_content/src/marley.rs`: `MarleySettingsContent` (12); its touchpoint row in
    `docs/marley/zed-touchpoints.md` (row for that file) lists every key by ticket.
  - `docs/marley/guide.md`: "Review notes to the agent" (1078).
- **Decisions:** D1 to D4 in the spec.

### Design
- **`crates/marley_agent/src/marley_agent.rs`** (Marley crate): `pub const DEFAULT_REVIEW_TEMPLATE`
  and `pub fn review_message(template: Option<&str>, notes: &str) -> String`: an empty or missing
  template means the default; `{notes}` replaced everywhere it stands; with none, the template, a
  blank line and the notes.
- **`crates/settings_content/src/marley.rs`** (Zed crate path, Marley's file):
  `review_notes_template: Option<String>` with its doc comment; the existing row gains "(#732)".
- **`crates/marley_workbench/src/marley_workbench.rs`** (Marley crate):
  `MarleySettings.review_notes_template: Option<String>`, read in `from_settings`.
- **`crates/marley_workbench/src/review_notes.rs`** (Marley crate): `prompt` returns
  `review_message(settings.review_notes_template, review_prompt(..))`, so the paste and Copy both
  carry it.
- **`docs/marley/guide.md`:** the setting under "Review notes to the agent".
- **Ledger rows:** `crates/settings_content/src/marley.rs`'s row names `review_notes_template`
  (#732). No other Zed path.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | The diff of the changed line; Add Review in the gutter; a note; Send Review to Agent; Enter on the ready row | 732-01-default: the stand-in's echo of the default's lines, then `File:`, `Line:`, `User comment:` |
| 002 | Writes `marley.review_notes_template` into the run's settings; back to the diff; a second note; send | 732-02-custom: the custom lines around the note, none of the default's |
| 003, 004 | (review of the diff: the append without `{notes}`, and Copy) | — |

### Risks
- A long template makes a long paste; Claude Code shows a pasted block as one collapsed item, so the
  user sees less of what was sent. The default stays three lines.
- A user's template may hold `{notes}` twice: both are replaced, which is what the user wrote.
