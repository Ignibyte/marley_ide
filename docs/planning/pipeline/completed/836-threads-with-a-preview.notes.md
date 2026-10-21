# The Threads page with filters, day groups and a preview — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-836-threads-with-a-preview.md
- **Pipeline spec:** 836-threads-with-a-preview.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #835; Chad's overnight goal.
- **Classification / tier:** feature, M. Marley crate only (`threads_page.rs`).
- **Recall (§18.3):** #754's fixtures make CLI sessions without an agent; PR-claude-701 (weak
  workspace in the page's handlers); the brain (consultation 519dc996; the decision went on 02988c02, a fresh one, at Complete): nothing on this seam.
- **Discovery:** `threads_page.rs`: `ThreadsPage` (:203), `Row` (:223), `Target` (:237), `Chip`
  (:281), `Sections` (:287), `sections` (:329), `render_chips` (:384), `render_section` (:417),
  `render_row` (:442), `open_row` (:489), `open_thread` (:530), `render` (:592); the rail's
  `archive_thread` (`rail.rs:6341`, `ThreadMetadataStore::archive`).
- **Decisions:** D1 to D3 in the spec.

### Design
- `ThreadsPage` gains `folder: Option<String>` and `selected: Option<String>` (a row's `key()`).
- `sections`: the folder filter beside the agent filter; one `rows: Vec<Row>` (assistants and
  others together, newest first) in place of `assistants`/`others`; `folders: Vec<String>`
  collected like `chips`.
- `day_of(updated_at) -> &'static str`: Today / Yesterday / This week (within 7 days) / Earlier,
  from the local date.
- `render_filters`: WHO (All + chips, stacked, `ButtonStyle::Subtle` with `toggle_state`) and
  FOLDER (All + folders, truncated).
- `render_row`: `ListItem::toggle_state(selected == key)`; `on_click` with `click_count() == 2`
  → `open_row`, else select and notify.
- `render_preview(row)`: icon + agent name, `Headline` title, folder, `when`, snippet; buttons
  Open (`open_row`), Open in a Split (`workspace.update(|workspace| { let pane =
  workspace.split_pane(workspace.active_pane().clone(), SplitDirection::Right, window, cx); window
  focus on pane })` then `open_row`), Archive (thread only: `ThreadMetadataStore::archive`).
- `render`: header + search on top; then `h_flex` of filters (200 px), list (`flex_1`, scroll,
  day sections), preview (320 px, `border_l_1`).
- **File manifest:** `crates/marley_workbench/src/threads_page.rs` (Marley), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | #754's two Claude sessions (repo now, sub four days old); `marley: open threads` | `836-01-night` |
| REQ-003 | click the first card | `836-02-selected` |
| REQ-001 | click the `sub` folder | `836-03-folder` |
| REQ-004 | All folders; first card; Open | `836-04-opened` |
| REQ-001..003 | Marley Day | `836-05-day` |

Archive (a Zed conversation) is read in the diff: the fixture has CLI sessions only.

### Risks
- CLI sessions' `updated_at` comes from their files; the fixture sets the older one's mtime.

## Phase 2 — Code
- **Paused 2026-10-21 ~07:30 at Chad's word, mid-phase (not gated, not committed).** Built in the
  working tree: `folder`/`selected` state, `day_of`, one `rows` list with `folders`, `folder_keeps`,
  WHO and FOLDER stacked at the left (`render_chips`, `render_folders`), `render_preview` (Open,
  Open in a Split, Archive for a Zed conversation), click-selects / double-click-opens in
  `render_row`, `render_list` (day sections + archived toggle), the three-column `render`. Clippy
  clean. Scenario `script/e2e/836-threads-with-a-preview.sh` written; one probe run: the cards sit
  under TODAY / YESTERDAY / THIS WEEK, a click selects and fills the preview, and Open resumed the
  session (the stand-in logged `claude: --resume 1111…`).
- **Resumed 2026-10-21 at Chad's "lets continue on".** The FOLDER list shows each folder's own
  name (its whole path in a tooltip) with `truncate`, since full paths ran across the page; the
  filter and preview columns clip (`overflow_hidden`). The preview's first line (agent · folder)
  sits in a `flex_1 min_w_0` box with `truncate_start`, so the folder's end shows. `full_width`
  on the folder buttons was tried and dropped: it centers the label and Button has no alignment.
- **Deviation:** the probe's folder list also holds the test profile's other conversations
  (Rusty's, Marley's, earlier scenarios'), so the `sub` point is read from the probe's shot.
- **Gate:** `just gate-diff` green, all 16 gates on the scope and the receipt.
- **Review:** the preview's Split goes through a weak workspace (PR-claude-701); Archive only for
  a Zed conversation (D3); nothing reads the page while it is updated.

## Phase 3 — Test
- Scenario `script/e2e/836-threads-with-a-preview.sh` (sway), on the gated build. One rerun: the
  first recorded run's palette typed late on a cold start (shot 01 showed it half typed) and every
  later click landed early; the palette's settles grew from 1 s to 2 s and 3 s, the first settle
  from 12 s to 15 s, and shot 03 now moves the pointer away first. Gate re-run after the edit.
- `836-01-night`: WHO (All, Rusty, Marley, Claude Agent, Claude Code CLI) and FOLDER (All, then
  each folder's name, `repo` and `sub` among them) at the left, the cards under TODAY, YESTERDAY,
  THIS WEEK, "Pick a conversation to see it here." at the right (REQ-001, REQ-002). The profile's
  other conversations (Rusty's, earlier scenarios') are listed too.
- `836-02-selected`: "Fix the flaky tokenizer test" drawn selected; the preview: its icon, the
  folder's end (`…/repo`), the title, "Last changed Oct 21, 08:28", Open and Open in a Split, no
  Archive for a CLI session (REQ-003). The probe's shot showed Archive on Rusty's conversation (D3).
- `836-03-folder`: `sub` lit orange; only "Release notes for sub" under THIS WEEK (REQ-001).
- `836-04-opened`: All folders, the first card, Open: a terminal tab running
  `claude --resume 1111…` in the repository; the log: `claude: --resume 1111… @ …/repo` (REQ-004).
- `836-05-day`: Marley Day, the first card selected and previewed (REQ-001 to REQ-003).
- Not shot: Open in a Split's split and the double-click (one code path with Open, read in the
  diff); Archive's effect (the fixture has no Zed conversation of its own).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added), the guide's Threads page, `marley_workbench.md`'s Threads page
  (sections, the layout), the look plan's W2.3 done. No Zed path touched.
- **Knowledge:** L-claude-836-a-full-width-button-centers-its-label-001,
  L-claude-836-a-cold-start-types-the-first-palette-late-001. No F block: the missed folder click
  was the scenario's timing, not the code.
- **Brain:** decision on consultation 02988c02.
- Ticket closed, pipeline archived.
