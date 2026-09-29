---
pipeline_id: 0d108a4b-f739-47c2-8fb9-5e5e107bb6db
ticket: docs/planning/tickets/closed/TICKET-528-block-filter.md
status: Phase 4 — Complete PASS
title: "Filter a block's output"
type: feature
slice: prong 1 T1 (stage-one block actions; Warp once-over item 5), the overlay first
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/planning/pipeline/completed/481-rich-input.spec.md]
---

## Title
Show only the lines of a block that match text or a regex, with case, invert and context lines,
deleting nothing. The first slice is Warp's block filter as a panel over the terminal: it reads
the block's output and lists what matches, and the terminal's own rows stay as they are.

## Scope
### In
- **Opening it.** `marley::FilterBlock` on Alt+Shift+F in `Terminal` filters the newest block with
  a row in view; a Filter button (`IconName::Filter`) beside Copy on a hovered block filters that
  block. The same key, or Escape, in the panel closes it.
- **The panel**, over the terminal's grid (the footer stays): a header with the block's command and
  "N of M lines"; a query field with the focus; toggles for case sensitivity and regex (Zed's
  `CaseSensitive` and `Regex` icons), invert, and a small field for context lines; under them the
  kept lines in a read-only Zed editor, the matches highlighted, groups of context separated by a
  `--` line as grep separates them.
- **Matching** (pure, `marley_terminal::filter`, new): each output line against the query, as text
  or as a regex (the `regex` crate), ignoring case unless the toggle is on; invert keeps the lines
  that do not match; context N keeps N lines before and after each match. An invalid regex is
  named in the panel and the last good list stays.
- **The block's lines** from `Terminal::block_output`; a block whose output has left the
  scrollback says so. A running block's list follows its output, filtered again at most four
  times a second off the main thread.
- **Keys stay in the panel**: its container stops the keys the terminal would send its program, as
  the rich input's does (F-claude-481), and binds Enter and Escape itself.
- **The query and toggles** are kept per terminal view for the session and come back when the panel
  opens again.
- **Two hooks in `terminal_view`**, as #477's footer and #484's suggestion are hooks: an overlay the
  view draws over its grid, and the Filter button's action, both set by `marley_workbench`.

### Out (explicitly deferred)
- Filtering the terminal's rows in place, with the other blocks around the filtered one: it needs
  stage two's display-row map (T5), as the once-over notes.
- The block's colors in the kept lines (the list is the block's text).
- A "Toggle Block Filter" item in #554's Block menu: the scenarios of #554 and #555 choose that
  section's items by their place from its end, and the key and the button cover the need.
- Filters kept across restarts, and a filter over several blocks.

## Reference (§20)
- **Warp, block filtering** (https://docs.warp.dev/terminal/blocks/block-filtering/; the Warp
  once-over, `docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 5): a filter icon at
  a block's top right, `Alt+Shift+F` on Linux, or "Toggle Block Filter" from the block's menu,
  opens "a large input field with two buttons on the left and a smaller input field on the
  right"; "Only lines containing text that matches the filter query will be shown"; regex and
  case-sensitive buttons, an invert button, and a number for context lines; "Filtering does not
  delete any output lines, so you can clear the filter to go back to the original output"; a
  filter toggled off comes back when toggled on. Marley keeps the key, the button on the block,
  the four controls, the kept filter and the deleting of nothing; it shows the kept lines in a
  panel over the terminal instead of in the block's place (the once-over: "Filtering in place
  needs T5's display-row map; an overlay does not"). No Warp code.
- **Upstream Zed:** the terminal's search (`buffer_search::Deploy` on Ctrl+Shift+F in `Terminal`;
  `TerminalView`'s `SearchableItem`, regex only) highlights matches in the grid and hides nothing;
  it stays as it is, and the filter is a second, separate tool.

### Prior art
- **Behavior maps and reports.** The once-over, item 5 ("S each, M to filter in place"). Orca report
  05 §2.6: Orca's terminal has search with case and regex toggles and a match counter (xterm's
  search addon) and no filter; Zed's search is the same kind of tool.
- **Published material.** grep's `-v` (invert), `-C` (context) and its `--` group separator, the
  meanings the toggles take.
- **The code we already ship.**
  - Zed's `editor`: `Editor::single_line` for the query and context fields, `Editor::multi_line`
    with `set_read_only` and `set_text` for the kept lines, `highlight_background` for the matches,
    so selecting, copying and scrolling the list are the editor's.
  - The `regex` crate (1.12 in `Cargo.lock`, `regex = "1.5"` in the workspace; `marley_mcp` takes it
    for #516), with `RegexBuilder::case_insensitive` and `regex::escape` for a plain query.
  - `Terminal::block_output` (#464), `marley_terminal::visible_spans` for the blocks in view.
  - #474's block element in `terminal_view`: the hover group, Copy, `marley_keep_from_terminal`.
  - #481's rich input: an editor inside the terminal view, its container's `on_key_down` that stops
    the keys the terminal maps, and its own `Enter` and `Escape` in `MarleyRichInput > Editor`.
  - The footer hook (AD-claude-477) as the pattern for a Zed-crate hook that Marley's crate fills.

## UI proof
UI-AFFECTING. `script/e2e/528-block-filter.sh` (`compositor sway`, for the hover and the button).
The scenario's own bash; `log.txt`, 40 lines of `INFO`, `WARN` and `ERROR` entries in mixed case.
Steps and shots: `seq 1 300`, then `cat log.txt`, then Alt+Shift+F (`528-01-open`: the panel on
`cat log.txt`, 40 of 40 lines, the field focused); `error` typed (`528-02-text`: the lines holding
it in any case, "N of 40"); the case toggle (`528-03-case`); the regex toggle and
`^WARN .* 9[0-9]%$` (`528-04-regex`), then `(` (`528-05-invalid`: named invalid, the last list
kept); invert (`528-06-invert`); context 1 (`528-07-context`); Escape (`528-08-closed`: the
terminal as before, the prompt line empty); Alt+Shift+F again (`528-09-reopened`: the same query and
toggles); the pointer on `seq 1 300`'s block and its Filter button (`528-10-button`: the panel on
that block); a loop printing a numbered line every 0.2 s, filtered on `5`, shot twice a few
seconds apart (`528-11-running-a`, `528-11-running-b`).

## Locked-In Decisions
- D1 — A panel over the terminal, not a filter in place: hiding rows needs stage two's display-row
  map (T5). The panel reads the block's output and lists what it keeps, and the grid is never
  touched, so the filter deletes nothing by construction.
- D2 — The kept lines sit in a read-only Zed editor: selection, copy and scrolling come with it.
- D3 — Per-line matching with the `regex` crate: a plain query escaped, case ignored by default,
  and invert and context in grep's meanings.
- D4 — Alt+Shift+F, Warp's key on Linux, which Zed's Linux keymap leaves unbound; it filters the
  selected block (#554) when one is selected, else the newest block in view. The hover button
  filters the block under the pointer. (At promotion, 2026-09-29: #554 shipped a selected block.)
- D5 — The query and toggles are kept per terminal for the session, as Warp keeps a filter that was
  toggled off.
- D6 — Marley's code in `marley_workbench`: Zed's terminal view gains only two hooks, the overlay
  and the button's action, as the footer (#477) and the suggestion (#484) are hooks.
- D7 — A running block's list follows its output, filtered again at most four times a second, off
  the main thread.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Alt+Shift+F in a terminal, the system shall open the filter over the terminal for the selected block, or with none selected the newest block in view, with the query field focused and the block's lines listed with their count. | Shot `528-01-open` |
| REQ-002 | WHEN the user clicks a hovered block's Filter button, the system shall open the filter for that block. | Shot `528-10-button` |
| REQ-003 | WHILE a query is typed, the filter shall list only the block's lines that contain it, ignoring case, and say how many of the block's lines it shows. | Shot `528-02-text` |
| REQ-004 | WHEN case sensitivity is on, the filter shall keep only lines that match the query's case. | Shot `528-03-case` |
| REQ-005 | WHEN regex is on, the filter shall match the query as a regular expression, and WHEN the query is not a valid one, the filter shall say so and keep the last list. | Shots `528-04-regex`, `528-05-invalid` |
| REQ-006 | WHEN invert is on, the filter shall list the lines that do not match. | Shot `528-06-invert` |
| REQ-007 | WHEN context is N, the filter shall list N lines before and after each match, with `--` between groups that do not touch. | Shot `528-07-context` |
| REQ-008 | WHEN the user presses Escape or Alt+Shift+F in the filter, the system shall close it, and the terminal shall show the block's lines as they were. | Shot `528-08-closed` |
| REQ-009 | WHILE the filter is open, no key typed in it shall reach the terminal's program. | Shot `528-08-closed`: the prompt line empty |
| REQ-010 | WHEN the filter opens again on the same terminal, it shall have the last query and toggles. | Shot `528-09-reopened` |
| REQ-011 | WHILE the filtered block runs, the list shall take in its new output lines. | Shots `528-11-running-a`, `528-11-running-b` |
| REQ-012 | The diff gate shall be green. | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design and the test plan in the notes.
- **P2 Code** — the touchpoints rows first; the two hooks and the button in `terminal_view`; the pure
  filter in `marley_terminal`; the panel, the action and the keys in `marley_workbench`; fmt and
  clippy clean; a review of the diff (no key from the panel reaches the PTY; the grid is never
  written).
- **P3 Test** — write and run the scenario, read every shot; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley/three-prong-plan.md` (T1), the Marley crates' notes
  under `docs/marley_architecture/` (§21), ledger capture (§19), close, archive, commit.
