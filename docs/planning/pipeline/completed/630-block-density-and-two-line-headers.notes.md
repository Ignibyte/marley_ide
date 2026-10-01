# Block density: two-line headers and gaps — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-630-block-density-and-two-line-headers.md
- **Pipeline spec:** 630-block-density-and-two-line-headers.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, third batch (#628 to #630, and #466): T5 stage two, and fish.
- **Recall (§18.3):**
  - Rows beyond the grid need pixel scroll as Zed's inline assist uses (`scroll_top`); #628 and #629 avoid them.
  - The tree defines no Warp density for blocks; #216's notes are about chrome and fonts.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):**
  - AD-claude-628: one for one; no row moves. The queued draft's "rows that are no grid line"
    needs the map #628 avoided.
  - F-claude-565: a reader of a workspace inside that workspace's update panics; the branch is
    recorded from a terminal observer, at effect flush, where no entity is being updated.
  - The agent bar (#477) already finds a folder's branch in Zed's git store.
  - Brain (consultation 889c06b1fb1c40e48b4854c7c5052dfd): nothing on this seam.
- **Split:** gaps, a taller header and the density setting became #631 (Deliberate), with the
  reason in its row.

### Design
- **`agent_bar.rs`:** the repository lookup moves into `pub(crate) fn branch_of(project, folder,
  cx) -> Option<SharedString>`, which `contents` calls.
- **`block_headers.rs`:** a global `Branches(HashMap<(EntityId, usize), Option<SharedString>>)`;
  `init` observes each new `TerminalView`'s terminal and records, for each running or finished
  block without an entry, the branch of its `pwd` (`branch_of` with the view's workspace's
  project); a released view's entries go. `header(view, terminal, index, line_height, rows, cx)`:
  the folder (`pwd`, home as `~`), the recorded branch, and the command: rows ≥ 2 → a muted first
  row `folder · branch` and the command on the second; one row → the command, then
  `  folder · branch` muted.
- **`terminal_view.rs` / `terminal_element.rs`:** `MarleyBlockHeader` gains `rows: usize`, which
  the element passes (`rows.len()`).
- **Default:** `default.json` `"block_headers": true`, the docstring `Default: true`,
  `BlockHeaders::from_setting` `Native` unless `Some(false)`.

### File manifest
- Marley: `crates/marley_workbench/src/block_headers.rs`, `agent_bar.rs`, `marley_workbench.rs`;
  `script/e2e/630-block-density-and-two-line-headers.sh`.
- Zed: `crates/terminal_view/src/terminal_view.rs`, `crates/terminal_view/src/terminal_element.rs`,
  `crates/settings_content/src/marley.rs`, `assets/settings/default.json`; rows widened first.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-003, REQ-004 | git repo on `main`; bash, two-line PS1; `echo hi`, `ls` | `two-rows.png` |
| REQ-001 | a second terminal whose shell reads a one-line PS1 (`PS1_ROWS=1` in `.bashrc`'s choice); `echo hi` | `one-row.png` |
| REQ-002 | `git checkout -q -b other`, `echo after` | `checkout.png` |

## Phase 2 — Code (2026-10-01)
- **Built:** `agent_bar::branch_of`, the agent bar's repository lookup as a function it now calls;
  in `block_headers.rs`, the `Branches` record (each started block's branch, from a terminal
  observer, dropped with the view) and the header's folder (`~` for home) and branch, on a first
  row over the command when the prompt took two rows or more, after it on one; the hook's `rows`
  argument (`terminal_view.rs`, `terminal_element.rs`); the setting on by default
  (`default.json`, the docstring, `BlockHeaders::from_setting`).
- **Review:** the record is written from the terminal's observer, at effect flush, so reading the
  workspace's project there reads no entity in its update (F-claude-565); a block with no `pwd`
  gets no folder and no branch; a folder in no repository records `None` once.
- **Clippy found:** the module doc's first paragraph, a missing semicolon.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenario:** `script/e2e/630-block-density-and-two-line-headers.sh` (sway): a repository on
  `main` under the shell's HOME, a two-line PS1, the setting left unset. Two runs.
- **First run, red:** the folder was the whole path (the scenario's HOME is not Marley's, so no
  `~`), which pushed the branch off the row's right end, and over one row the folder squeezed
  the command to `echo…`. A long folder does the same for any user.
- **Fix:** past two folders the label keeps the last two after `…` (`~` still for home), the
  command keeps its width (`flex_none`) and the folder and branch give way (`flex_1`, truncated).
  Gate GREEN, 17 PASS.
- **Second run:**
  - `two-rows` (REQ-001, REQ-003, REQ-004): `…/home/repo · main` muted over `echo hi`, then `hi`;
    the same over `ls`; the live `[the prompt]` and `$ ` as bash drew them; no setting written.
  - `one-row` (REQ-001): after `PS1='$ '`, `echo one …/home/repo · main` on one row, then `one`.
  - `checkout` (REQ-002): `git checkout -q -b other …/home/repo · main` (the branch when it
    started), then `echo after …/home/repo · other`; the earlier headers still read `main`.
- **Seen, not a fault:** the folder reads `…/home/repo`, not `~/repo`, since Marley's own home is
  not the scenario's HOME; under one HOME it reads `~/repo`.

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `marley_workbench.md` (Block headers); the
  plan's T5 row; the ledger rows of `terminal_view.rs`, `terminal_element.rs`, `settings_content`
  and `default.json`.
- **Knowledge:** F-claude-630-a-long-folder-pushed-the-branch-off-the-header-001,
  AD-claude-630-the-header-shows-where-and-on-what-branch-in-its-prompts-rows-001.
- **Split:** TICKET-631 (gaps and density), Deliberate, in BACKLOG.
- **Brain:** the consultation closed with `brain decide`.
