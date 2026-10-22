# New Agent in Worktree as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-851-worktrees-and-review.md
- **Pipeline spec:** 851-worktree-prompt.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #850; split while planning: the prompt here, the
  review screens TICKET-881.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #510's scenario, stale since the rail moved (its prompt no longer opens);
  probing it found TICKET-880, done first.
- **Discovery:** the rail already draws drift (`drift_snapshot`), and merge and remove ask through
  `window.prompt`, styled by #870. The prompt is a headline, the editor, Also start, a branch line
  and the setup box.
- **Decisions:** D1, D2 in the spec.

### Design
- `worktree_agents.rs`: `WorktreePrompt` gains `included: Option<Vec<String>>` (read with the
  setup offer) and `mode: LaunchMode`; `render` draws the badge (`badge_of(kind)`), the title, the
  label, the editor (three lines tall), chips, Also start, a two-column facts grid
  (`render_fact`), and the footer with a primary `Button` whose click confirms.
- **File manifest:** `worktree_agents.rs`; the scenario.

## Phase 2 — Code
- **Built:** `worktree_agents.rs` `WorktreePrompt`: `hooks`, `bypass` (set after `new`, from
  `agents::launch_mode` read before the modal opens, since `new` runs inside the workspace's
  update) and `included` (the `.worktreeinclude` lines, read by `include_lines` when the prompt
  opens); `render_header` (the badge from `badge_of`, the title), the First prompt label over a
  three-line editor, `render_branch_chip` per branch, `render_also_start`, `render_facts` (the
  copied lines, the permissions, the setup: the `create_worktree` task or the offered command's
  box), and the footer with #870's primary `prompts::render_button`, whose click confirms (D2).
- **Deviation:** the port slot fact is left out (spec, Out): a slot is given when the worktree is
  made.
- **Gate:** the first run failed clippy (eight arguments to `new`, `render` at 125 lines, an `if
  let` in `include_lines`, a `SharedString` from a literal); `bypass` moved out of `new`, the
  header and Also start split off, `map_or_else`, `new_static`; the second run green, 17 of 17,
  and the scenario re-run on it.
- **Review:** Enter, Shift-Enter and Escape are the keymap's, unchanged; the setup box keeps its
  click; nothing reads the workspace inside its own update.


## Phase 3 — Test
- **Scenario:** `script/e2e/851-worktree-prompt.sh` (sway): the project's + (236, 136), New Agent
  in Worktree clicked (its submenu opens on a click, not a hover), Claude Code; Escape at the end,
  so no worktree is made.
- `851-01-prompt`: the CC badge and "Claude Code in a new worktree", First prompt with the typed
  line, `agent/<name> from main`, Also start (Codex, Gemini CLI, OpenCode), the facts (".env,
  .zed/settings.json"; "The project's: bypass"), the footer and "Start the agent · Enter" in the
  accent. REQ-001 met.
- `851-02-two`: Codex ticked: `agent/<name>-claude` and `-codex` chips, "Start 2 agents". REQ-001
  met.
- `851-03-day`: Marley Day, the same. REQ-001 met.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new (the stale #510 probe became TICKET-880). Brain: `brain decide` on
  consultation 6b1372ec.
- Ticket closed; pipeline archived; TICKET-881 holds the review screens.
