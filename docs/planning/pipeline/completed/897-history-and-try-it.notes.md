# History in the Changes inspector, and the guide's Try it — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-897-history-in-the-inspector-and-the-guides-try-it.md
- **Pipeline spec:** 897-history-and-try-it.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #891 (#890 waits on Chad).
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** #891's survey: no history list in Marley; Zed's Git Graph reads
  `graph_data` then `fetch_commit_data` per sha; the Browser runs no command from a page; the
  guide has no Try it. The brain had nothing.
- **Decisions:** D1–D3 in the spec (D1 and the History view later dropped, see Phase 2).

### Design
- `history_view.rs`: `HistoryView { repository, rows, _observed }`; `load` asks
  `graph_data(LogSource::Branch(head), LogOrder::DateOrder, 0..30)` and `fetch_commit_data` for
  each sha inside `repository.update`, keeps the loaded ones, and observes the repository so a
  finished load redraws; `render_rows(cx)` draws a row per commit (dot, subject, short sha,
  relative time). `OpenHistory` opens it in the center.
- `inspector.rs`: a `history: Option<Entity<HistoryView>>` after the panels; the switcher's last
  entry shows it.
- `guide.rs`: `try_it(url) -> Option<&'static str>`; `browser.rs`'s `navigatedWithinDocument`
  arm dispatches the action in the window of the tab showing the page.
- **File manifest:** `history_view.rs` (new), `inspector.rs`, `guide.rs`, `browser.rs`,
  `marley_workbench.rs`, `guide/index.html`, the scenario.

## Phase 2 — Code
- Built first as planned: `history_view.rs` and History as the Inspector's third entry. The first
  run's shots showed Zed's git panel already has a History tab under Changes, in the Inspector
  too, with each commit's subject, author, when and sha. The Marley view was a duplicate and was
  dropped (`inspector.rs` back as it was); the spec now names Zed's tab as the history, and a Try
  it link opens it.
- Kept: `guide::try_it` (the guide's own file page, a `#try=` word on a fixed list of six
  commands), `browser::run_try_it` on the hub's `navigatedWithinDocument` (it dispatches in the
  window of the tab showing the page), the links and their style in `guide/index.html`, and the
  page's script that clears the fragment after a click.
- Review: no page gets a binding; another page, or another word, asks for nothing; the command
  is built by name and dispatched in the guide tab's own window. `follow_navigation` went over
  100 lines with the hook; the check moved into `run_try_it`.
- Removed `script/e2e/891-branch-picker.sh`, the probe `891-search-branches-extensions.sh`
  replaced.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/897-history-and-try-it.sh` (sway, the browser fixture).
- `897-01-guide`: the guide at the rail's section: `marley: toggle rail strip` and the pill "Try
  it: fold the rail to a strip" (REQ-001).
- `897-02-tried`: after the click, the rail folded to its strip, the address without the
  fragment; the same link clicked again unfolded it (`897-03` shows the rail back) (REQ-001, D2).
- `897-03-changes`: "Try it: open Changes, with its History tab" opened the Git Panel tab; its
  History tab lists the three commits with author, when and sha (REQ-002).
- `897-04-day`: Marley Day, the guide's "This guide" section with its four Try it links
  (REQ-001).

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (the guide), look-plan W5.
- Knowledge: L-claude-897-look-for-zeds-own-view-before-building-one-001.
- Brain decision recorded on consultation 81ca7df7.
