---
pipeline_id: f7b6fd1d-a1c6-45d0-89f1-7faab46fd295
ticket: docs/planning/tickets/open/TICKET-732-a-feedback-template-for-review-notes.md
status: Phase 4 — Complete PASS
title: A feedback template for review notes
type: feature
slice: the review batch after the finish (docs/planning/intake/plannotator-findings.md item 7; docs/planning/design-notes/finish-line-2026-10-11.md row 11)
references:
  - docs/planning/pipeline/completed/522-review-notes-to-the-agent.spec.md
  - docs/planning/intake/plannotator-findings.md
---

## Title
Review notes reach the agent inside a message the user can set. The default asks the agent to check
each note, give a verdict on it, and change nothing the notes do not name. Chad, 2026-10-11, on the
finish-line list: "11.) yes".

## Scope
### In
- `marley.review_notes_template`, a string in `settings.json`: the message that carries the notes,
  with `{notes}` where they go. Unset or empty, Marley's default applies.
- The default, in `marley_agent` beside `review_prompt`:

  ```
  Review notes on your work follow. For each note, check it against the code before you change
  anything, then say whether you agree and why. Fix only what you agree with, and change nothing
  the notes do not name.

  {notes}
  ```

- Every send of review notes through #522's picker uses it: the paste into an agent and Copy notes.
- A template without `{notes}` gets the notes after it, separated by a blank line.
- The setting's doc comment, and the guide's "Review notes to the agent", say how to set it.

### Out (explicitly deferred)
- A row in the Settings window: its text fields are one line, and a template is several.
- Templates per agent or per project, and placeholders other than `{notes}`.
- Plan feedback (#726) and question answers (#728): they carry their own lead lines.

## Reference (§20)
N/A — Marley-specific: #522's send has no upstream counterpart (upstream Zed's handler of
`SendReviewToAgent` was removed in #48339). The behavior follows plannotator's feedback templates
(published docs, `docs.plannotator.ai/open-source/reference/custom-feedback`): a template string
with a placeholder for the annotations, and a default whose review suffix asks the agent to validate
each finding and report a verdict before it changes code. No plannotator code is taken.

### Prior art
- **The code we ship:**
  - `marley_agent::review_prompt` (`crates/marley_agent/src/marley_agent.rs:371`) builds the notes'
    text; `escape_comment` (392) keeps control characters out of it.
  - `crates/marley_workbench/src/review_notes.rs`: `prompt` (162) wraps `review_prompt`; `deliver`
    (108) sends it (Copy at 119, the paste at 151).
  - `MarleySettings` (`crates/marley_workbench/src/marley_workbench.rs:370`) and its
    `from_settings` (634) resolve the `marley` block; `MarleySettingsContent`
    (`crates/settings_content/src/marley.rs:12`) holds the keys, with `push.url` (a string) as the
    model for an optional string.
  - The Settings window's Marley page (`crates/settings_ui/src/marley_page.rs`) draws strings as
    one-line fields (`marley.push.url`, 1124).
- **Behavior maps:** Orca report 02 §2.9 (`docs/orca_architecture/02-worktrees-and-review.md`): the
  note format #522 adopted; Orca has no template.
- **Published material:** plannotator's custom feedback reference (above): `prompts.review`, the
  `{{feedback}}` placeholder, fallback to the built-in text when the key is blank.
- No crate we build offers message templating that fits; a single `str::replace` is enough.

## UI proof
`script/e2e/732-a-feedback-template-for-review-notes.sh`, under `compositor sway`, on #522's set-up:
a scratch repository with one changed line, and in its terminal the stand-in `claude` that sends
Marley's plugin's `SessionStart` (so its seat is idle) and echoes each line it reads.

Shots:
- `732-01-default`: a note sent with no template set: the agent's terminal shows the default's lines,
  then `File:`, `Line:` and `User comment:`.
- `732-02-custom`: `marley.review_notes_template` set by the scenario to
  `"Scenario template.\n\n{notes}\n\nEnd of notes."`, a second note sent: the terminal shows the
  custom lines around the note and none of the default's.

## Locked-In Decisions
- **D1:** one setting, `marley.review_notes_template`, read where the prompt is built. Rejected: a
  template file in the config folder (a second place to look, and file IO for one string); a
  Settings window row (its one-line field cannot hold a template of several lines).
- **D2:** the placeholder is `{notes}`; a template without it gets the notes appended. Rejected:
  plannotator's `{{feedback}}` (Marley's word is notes) and dropping the notes when the placeholder
  is missing (plannotator does; a missing placeholder is more likely a slip than a wish).
- **D3:** the default text lives in `marley_agent` as a constant, pure and beside `review_prompt`.
- **D4:** Copy notes uses the same message, so a paste by hand reads as Marley's send does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.review_notes_template` is unset or empty, WHEN review notes are sent to an agent, the system shall send the default's lines followed by the notes. | Shot 732-01 |
| REQ-002 | WHEN review notes are sent while `marley.review_notes_template` is set, the system shall send the template with `{notes}` replaced by the notes, and none of the default's lines. | Shot 732-02 |
| REQ-003 | WHERE the template holds no `{notes}`, the system shall send the template, a blank line and the notes. | Review of the diff |
| REQ-004 | WHEN the user chooses Copy notes, the system shall put the same message on the clipboard. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — the setting in `crates/settings_content/src/marley.rs` (its touchpoint row updated),
  `MarleySettings`, the default and the wrapper in `marley_agent`, `review_notes.rs`, the guide; a
  review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
