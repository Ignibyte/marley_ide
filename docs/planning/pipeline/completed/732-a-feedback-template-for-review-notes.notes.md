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

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation 123be6414c82406d85bc07097eddc6e0) had nothing on the seam.
- **Seams re-verified** at `6a1ae634c5`, directly (four known files): `ReviewLine` (389),
  `review_prompt` (408), `escape_comment` (429) in `marley_agent.rs`; `review_notes::prompt` (162)
  and `deliver` (108); `MarleySettings` (393) and `from_settings` (743); `MarleySettingsContent`
  (12) with `push` as the model; the touchpoint row for `crates/settings_content/src/marley.rs`.
  The design stands as written.

## Phase 2 — Code

- **Built:** `marley_agent::{DEFAULT_REVIEW_TEMPLATE, NOTES_PLACEHOLDER, review_message}`;
  `MarleySettingsContent.review_notes_template` (its touchpoint row first);
  `MarleySettings.review_notes_template` (blank taken as unset); `review_notes::prompt` wraps the
  notes, so the paste and Copy carry the same message.
- **Deviations:** the template's control characters other than LF and tab are dropped, which the
  queued design did not say: a template could otherwise carry an escape sequence, a bracketed
  paste's end mark among them, into the agent's terminal.
- **The review found** nothing else.
- **Clippy:** `too_long_first_doc_paragraph` and `literal_string_with_formatting_args` (the
  placeholder is now the `NOTES_PLACEHOLDER` constant). Fixed at the source.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the tree the scenario ran.

## Phase 3 — Test

- **Scenario:** `script/e2e/732-a-feedback-template-for-review-notes.sh` under `compositor sway`,
  #522's set-up, sending through the palette. The first run stopped at 732-00 to place the changed
  line's gutter on the diff's right side (960,220); the second stopped at 732-01-default; the third
  passed every check.
- **732-01a-note:** the note "Say three in lower case" under line 3, and Send Review to Agent (1).
- **732-01-default (REQ-001):** the stand-in read Marley's message: "Review notes on your work
  follow. For each note, check it against the code before you change anything, then say whether
  you agree and why. Fix only what you agree with, and change nothing the notes do not name.", a
  blank line, `File: notes.txt`, `Line: 3`, `User comment: "Say three in lower case"`.
- **732-02-custom (REQ-002):** with the template set by the scenario, a second note: "Scenario
  template.", the note, "End of notes."; none of the default's lines; only the new note was sent.
- **REQ-003, REQ-004:** by the review: `review_message` appends after a blank line without the
  placeholder; Copy calls the same `prompt`.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide's "Review notes to the agent"; `marley_agent.md`
  (`review_message`); `marley_workbench.md` (Review notes); the touchpoint row of
  `crates/settings_content/src/marley.rs` names `review_notes_template` (#732).
- **Knowledge:** L-claude-732-a-stand-in-reads-a-bracketed-pastes-marks-001,
  AD-claude-732-review-notes-ride-in-a-template-the-user-can-set-001. No failure found.
- **Brain:** `brain decide` on 123be6414c82406d85bc07097eddc6e0.
