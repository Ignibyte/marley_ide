# Forge's sprint board — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-842-forge-sprint-board.md
- **Pipeline spec:** 842-forge-board.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #841.
- **Classification / tier:** feature, L. Marley crate only.
- **Recall (§18.3):** #841's Forge client, token and stand-in (L-claude-841); #836's Threads page
  as a Marley item with columns.
- **Discovery:** Forge's ticket tools and arguments (`ticket-claim {id, owner_id?, release?}`,
  `ticket-update {id, status?, phase?, …}`, `ticket-comment {ticket_id, body}`, `ticket-get {id |
  number, include_comments?}`); statuses `open`, `in-progress`, `in-review`, `qa-review`,
  `qa-failure`, `qa-pass`, `done`, `won-t-fix`, `abandoned`, `closed`. #510's `open_prompt` opens
  the worktree prompt with no text.
- **Decisions:** D1–D3 in the spec.

### Design
- **`forge.rs`:** `pub(crate) async fn connect(url, bearer_env, name, cx) -> Result<ContextServer>`
  and `call<T>(server, tool, arguments)`, moved from #841's Forge branch of
  `fleet_providers::connect` so the fleet and the board share them; the DTOs the board reads
  (`ForgeTicket`, `ForgeSprint`, `ForgeComment`).
- **`forge_board.rs` (new):** `ForgeBoard` (an `Item`): state `{ sprint, tickets, agents,
  selected, detail, filter, comment field, error }`; `refresh` spawns the reads; `render` draws the
  header, five columns (`COLUMNS` per D1), cards, and the detail pane; `claim`, `move_to(status)`
  (a `ContextMenu` of the statuses), `comment` (Enter in the field), `start_in_worktree`
  (`worktree_agents::open_prompt_seeded`). A 30 s timer re-reads while the tab is visible.
- **`worktree_agents.rs`:** `open_prompt_seeded(workspace, kind, text, window, cx)` that opens
  the prompt with its editor holding `text`; `open_prompt` unchanged.
- **Action:** `marley: open forge board` (`OpenForgeBoard`).
- **File manifest:** `forge.rs`, `forge_board.rs` (new), `fleet_providers.rs`,
  `worktree_agents.rs`, `marley_workbench.rs`; the scenario. All Marley crates.
- **Visual check plan:** the stand-in grows the sprint and ticket tools; shots per the spec's UI
  proof. Not reached: Start here in a worktree's agent start (the prompt shown, not sent), a
  failed write's toast.
- **Risks:** the item's size (keep `render` split under 100 lines each); reads racing an action
  (the latest read wins by a generation counter).

## Phase 2 — Code
- **Built:** `forge_board.rs`: `ForgeBoard` (an `Item`, tab "Forge"), opened by `marley: open
  forge board` (`OpenForgeBoard`); `refresh` connects through `fleet_providers::connect` with the
  first `forge` entry and reads `sprint-current`, `ticket-list` (the sprint's, `status: all`,
  limit 200), `agent-list` and, for a selected ticket, `ticket-get` with comments, a generation
  counter keeping the newest read; `act` calls a tool, toasts a failure, and reads again;
  `claim`, `move_to` (a `PopoverMenu` of `MOVES`), `send_comment` (Enter in the comment field),
  `start_in_worktree` (`worktree_agents::open_prompt_seeded` with the ticket's number, title and
  description); `render_header`, `render_columns` (`COLUMNS` per D1), `render_card`,
  `render_detail`, `render_comments`, `render_actions`, `render_standing`. A 30 s timer reads
  again while the tab lives. `fleet_providers`: `Client` and `connect` made `pub(crate)`,
  `call_tool` (a timeout or another contract as an error). `worktree_agents::open_prompt_seeded`.
- **Deviations:**
  - The Forge client is shared by exposing `fleet_providers::connect` and adding `call_tool`,
    not by moving the code into `forge.rs`: smaller, and the fleet and the board still reach
    Forge one way.
  - The timer reads every 30 s while the tab exists, not only while it shows.
- **Gate:** red once on clippy (two redundant closures, two `option_if_let_else`, `unused_self` on `render_actions`, `needless_pass_by_ref_mut` on `act`), fixed at source; the scenario ran again on the fixed code.
- **Review:** reads and actions run off the entity's update; an older read never overwrites a
  newer (`generation`); every failure reaches the user (the standing line or a toast).


## Phase 3 — Test
- **Scenario:** `script/e2e/842-forge-board.sh` (sway): #841's stand-in grown a sprint, eleven
  tickets, comments, and state-changing claim/update/comment; probe runs (`STOP_AT=1..3`) placed
  the card, Move to…, qa-pass and the comment field.
- `842-01-board`: "Forge · Sprint 14 · Rail and theme · release/2026-10-24 · 4 days left · 3 of 11
  done", the filter field; Open 3, In progress 2, In review 1, QA 2, Done 3; cards with number,
  kind (accent), phase, title, agent (`code-1 · forge`, `code-2 · agent1`, `qa-1 · agent2`, else
  "not claimed") and age. REQ-001 met.
- `842-02-detail`: #828 selected (accent border); the pane: `#828 feature qa-failure`, the title,
  the description, Agent code-2, Sprint 14, two comments, the comment field, Claim, Move to…,
  Start here in a worktree. REQ-002 met.
- `842-03-moved`: after Move to… → qa-pass the card and the pane read `qa-pass`, age "now"; the
  stand-in logged `ticket-update {"id": …828, "status": "qa-pass"}`. REQ-003 met.
- `842-04-comment`: "QA pass on the second run." heads Comments, the field empty; the log shows
  one `ticket-comment`. REQ-003 met. (The first full run showed the comment twice: the stand-in
  called its tool twice per request; fixed in the scenario, Marley sent one call.)
- `842-05-day`: Marley Day, the board and pane. REQ-001 met.
- Not reached: Claim's write (the button is shown; the stand-in answers it but nothing on the
  board changes), Start here in a worktree (it opens #510's prompt), a failed write's toast.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (Forge's board), look-plan W4.2 done.
- Knowledge: none new beyond L-claude-841's stand-in. Brain: `brain decide` on consultation
  3885085e.
- Ticket closed; pipeline archived.
