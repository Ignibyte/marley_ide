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

## Phase 1 — Plan (promoted 2026-10-12)
- **Promotion:** the queued pair moved to `active/`; the ticket is in progress; its Backlog row is
  gone. The brain (consultation 13bdcfa42655409d91432cd83f9dd84b) had nothing on the seam.
- **Seams re-verified** directly at `4759d8d839` (the `review` family and `review_tools.rs` are
  #730's, shipped today): `Family::Review` and `tool_schemas` (registry.rs 910-940), which matches
  on the family and needs `review_schemas(verb)` now that the family has two tools;
  `review_tools::answer` routes by name; `agent_control::admit(Area::Editors, Level::Act)`;
  `editor_tools::project_holding`; `ProjectDiff::deploy_at_project_path` (git_ui 188), which moves
  to the file when the diff holds it; `Markdown::new` (markdown.rs 661, HTML not parsed by default)
  and `MarkdownElement::new` with `MarkdownStyle::themed` as `rusty/page.rs` draws them; a
  `Checkbox` with `cx.stop_propagation()` as `rusty/tasks_tab.rs` draws one.
- **Design:** `review_guide_open` in the `review` family (Write, `review.write`); `review_tools`
  validates (1 to 30 sections, absolute paths, lines from 1), admits as #730 does, and opens or
  replaces the tab; a new `guide_tab.rs`: `GuideTab` (an `Item`) with the title, the summary, "K of
  N reviewed" and each section's number, title, Markdown summary, file chips and Reviewed box; a
  ticked section folds to its title; a chip runs `deploy_at_project_path` when the project diff
  holds the file (its git status shows a change), else opens the file at its first line. A guide
  with the same title in the same workspace replaces the open tab's sections.

## Phase 2 — Code

- **Built:** `review_guide_open` in the registry (`guide_schemas`; `review_schemas` takes the
  verb; 69 tools); `review_tools::{guide_open, guide_of}`; `guide_tab.rs` (new: `Guide`,
  `GuideSection`, `GuideFile`, `GuideTab`, `open`, `in_project_diff`, `chip_label`); the in-app
  guide's row, written before the gate (L-claude-730-the-receipt-hashes-the-in-app-guide-001).
- **Deviations:** the chip opens through `window.defer` (the fix below); otherwise as designed.
- **The review found** nothing before the run; the Test phase found the panic below.
- **Clippy:** a `Button::icon` that this version names `start_icon`; then
  `semicolon_if_nothing_returned` and `needless_pass_by_ref_mut` after the fix. Fixed at the source.
- **Gate:** `script/gates.sh --diff` green, 17 of 17, on the fixed tree the third run passed on.

## Phase 3 — Test

- **Scenario:** `script/e2e/733-guided-review.sh` under `compositor sway`; `mcp_agent` calls
  `review_guide_open` twice. The first run stopped at 733-01 to place the chip (333,278), section
  1's box (298,214) and the tab (590,51).
- **The second run found a crash:** the chip's click panicked ("cannot read GuideTab while it is
  already being updated") and ended Marley, so the later shots were black and the second call
  found no Marley
  (F-claude-733-a-guides-file-chip-opened-the-diff-inside-the-tabs-own-update-001). Fixed by
  deferring the open; clippy and the build again; the third run passed every check with no panic
  in the log.
- **733-01-guide (REQ-001):** "Guide · Shout the second lines"; the title, "0 of 2 reviewed", the
  summary, section 1 "Alpha first" with its Markdown summary (bold kept) and `alpha.txt:2`, section
  2 "Then beta" with `beta.txt:2`, each with its Reviewed box. The answer: `sections: 2`,
  `replaced: false`.
- **733-02-chip (REQ-002):** Uncommitted Changes in front at `alpha.txt`.
- **733-03-ticked (REQ-003):** section 1 ticked, folded to its muted title; "1 of 2 reviewed".
- **733-04-replaced (REQ-005):** the same title with three sections: all three shown, no tick,
  "0 of 3 reviewed"; the answer `replaced: true`.
- **REQ-004, REQ-006:** by the review: a file with no git status opens through `open_abs_path` and
  `go_to_singleton_buffer_point`; `guide_of` refuses no sections, more than 30, a missing title or a
  relative path, naming the section and file.

## Phase 4 — Complete

- **Documented:** CHANGELOG (Added); the guide (Guided review; the tool in Agent Control's list)
  and the in-app guide's Review notes table; `marley_mcp.md`; `marley_workbench.md` ("Guided
  review"). No Zed path changed.
- **Knowledge:** F-claude-733-a-guides-file-chip-opened-the-diff-inside-the-tabs-own-update-001
  (under PR-claude-defer-a-pane-change-out-of-an-items-own-event-001, no new rule),
  AD-claude-733-a-guided-review-is-a-tab-of-sections-over-the-project-diff-001.
- **Brain:** `brain decide` on the consultation recorded at promotion.
