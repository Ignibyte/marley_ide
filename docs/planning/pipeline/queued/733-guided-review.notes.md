# Guided review — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-733-guided-review.md
- **Pipeline spec:** 733-guided-review.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** plannotator finding 8 (`docs/planning/intake/plannotator-findings.md`): "An agent
  splits a turn, a branch or a pull request into ordered sections, each with a reviewed tick, which
  Marley draws; any agent can submit one over MCP." Chad, 2026-10-11, on the finish-line list
  (`docs/planning/design-notes/finish-line-2026-10-11.md`): "11.) yes". Everything opens in a
  center tab (`docs/planning/design-notes/tabs-everywhere-2026-10-11.md`). It waits under
  Deliberate with the rest of the review batch until after the finish.
- **Classification:** feature, medium. Marley crates only.
- **Recall (§18.3):**
  - #730's spec: the `review` family, `review.write`, admission through `Area::Editors` at `Act`.
  - AD-claude-704-areas-ask-once-per-session-and-reads-stay-free-001: one question per session per
    caller and project.
  - AD-claude-674-every-center-tab-has-a-rail-row-and-files-fold-under-files-001: the Guide tab
    gets its rail row from its `Item`.
  - PR-claude-derive-wire-list-from-single-registry-001: one `REGISTRY` row, its schema in the
    match.
  - F-claude-600-a-registry-lookup-read-the-workspace-its-caller-was-updating-001: the project is
    found before any workspace update.
  - PR-claude-703-an-unactivated-insert-keeps-the-active-tab-001: the tab is shown (the call is the
    user's request through an agent); no unactivated insert.
- **Discovery:**
  - `crates/marley_mcp/src/registry.rs`: `Family` (15), `ToolSpec` (93), `REGISTRY` (125),
    `tool_schemas` (715).
  - `crates/marley_workbench/src/editor_tools.rs`: `after_admit` (64), `open` (470-562),
    `project_holding` (565-586).
  - `crates/marley_workbench/src/rusty/tasks_tab.rs`: `open` (86-110), `render_task` (956-1042),
    `Item` (1188-1229).
  - `crates/marley_workbench/src/threads_page.rs`: `open` (47-60), `Item` (437-457).
  - `crates/marley_workbench/src/rusty/page.rs`: the `Markdown` entity (338-351).
  - `crates/git_ui/src/project_diff.rs`: `deploy_at_project_path` (188), `move_to_project_path`
    (309); `DiffMultibuffer::move_to_path` is crate-private (`diff_multibuffer.rs:261-283`), so a
    line inside the diff is not reachable without a Zed touch, which this ticket does not make.
- **Decisions:** D1 to D4 in the spec.

### Design
- **`crates/marley_mcp/src/registry.rs`** (Marley crate): the `review_guide_open` row
  (`Tier::Write`, `review.write`) and its schema.
- **`crates/marley_workbench/src/review_tools.rs`** (#730's, Marley crate): `guide_open`: validate,
  admit (`Area::Editors`, `Level::Act`, the words "open a review guide: <title>"), find the project,
  then `guide_tab::show`; the answer names the tab.
- **New `crates/marley_workbench/src/guide_tab.rs`** (Marley crate): `GuideTab` (`Item`, `Render`,
  `Focusable`): the guide, a `Markdown` entity per summary, the ticks; `show` finds a tab with the
  same title in the workspace and replaces it, else adds one to the center; a chip's click checks
  the project diff's files (`ProjectDiff::multibuffer` paths) and calls `deploy_at_project_path`,
  else `open_path` and moves the cursor to the line.
- **`marley_workbench.rs`:** the module.
- **The guide:** "Guided review", and the tool in the MCP tools list.
- **File manifest:** `marley_mcp/src/registry.rs`, `marley_workbench/src/review_tools.rs`,
  `marley_workbench/src/guide_tab.rs` (new), `marley_workbench/src/marley_workbench.rs` (Marley
  crates); the guide; the scenario. No Zed path, no ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | The client calls `review_guide_open`; Allow for This Session | 733-01-guide |
| 002 | Clicks the `alpha.txt:3` chip | 733-02-chip: the project diff at `alpha.txt` |
| 003 | Clicks the Guide tab, ticks section 1 | 733-03-ticked |
| 004 to 006 | (review of the diff) | — |

### Risks
- The project diff loads files asynchronously; `deploy_at_project_path` keeps a pending scroll for a
  file not loaded yet, so a chip clicked early still lands.
- A guide can name files outside every open project; their chips open nothing and say so, and the
  tab still draws.
