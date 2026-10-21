# Home: one prompt, then what needs you — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-835-home-one-prompt.md
- **Pipeline spec:** 835-home-one-prompt.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #834; Chad's overnight goal.
- **Classification / tier:** feature, M. Marley crate only (`home_page.rs`).
- **Split:** a project chip for the prompt → TICKET-867, queued before #794.
- **Recall (§18.3):** #826's `start_cli_with_prompt` path for a CLI with a prompt; the global
  `enter` → `menu::Confirm` (`default-linux.json:17`) reaches a single-line editor; the brain
  (consultation 042c1401): nothing on this seam.
- **Discovery:** `home_page.rs`: `MarleyHome` (struct ~:159, `new` ~:186), `render_start` (:318),
  `render_agents` (:365), `render_configure` (:512), `render` (:636, cards in order), `card()`.
- **Decisions:** D1 to D3 in the spec.

### Design
- `MarleyHome` gains `prompt: Entity<Editor>` (single line, placeholder "What should an agent
  do?", made in `new`) and `picked: Option<AgentKind>`.
- `render_prompt(&self, window, cx)`: a `v_flex` card body: the editor in a rounded bordered box
  (`key_context("MarleyHomePrompt")`, `on_action(menu::Confirm)` → `start_prompt`); a row of
  agent chips (`ToggleButton`-like `Button`s with `toggle_state(picked == kind)`,
  `ButtonStyle::Filled`, `selected_style(Tinted(Accent))`), the start folder muted, and a
  `Start` button (Filled, Accent tint).
- `start_prompt(&mut self, window, cx)`: kind = picked or the first installed CLI (none → a toast
  through the workspace); text = the editor's text trimmed; `workspace.update(|workspace, cx|
  agents::start_cli_with_prompt(workspace, kind, &text, None, window, cx))` (empty text →
  `agents::start_cli`), detached with a log on failure; clear the editor.
- `render_links`: the START buttons and New Agent…/Threads as `ButtonStyle::Subtle` buttons with
  small icons, in a wrapping row.
- `render_configure`: its buttons' style to `Subtle` with `Color::Accent` labels.
- `render`: `card("PROMPT", "marley-home-prompt", prompt + links)` first; START and NEW AGENT
  removed.
- **File manifest:** `crates/marley_workbench/src/home_page.rs` (Marley), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002/004 | `repo`; stand-ins `claude`, `codex` on PATH; Home from the status bar | `835-01-night` |
| REQ-003 | click Codex; click the box; type "write the README"; Enter | `835-02-started` |
| REQ-001/004 | theme Marley Day; Home | `835-03-day` |

### Risks
- `installed_clis` reads the launcher's search path, not the scenario's terminal PATH: the
  stand-ins must be on Marley's own PATH (exported in `setup`).

## Phase 2 — Code
- **Built:** `home_page.rs`: `prompt: Option<Entity<Editor>>` (made at the first `render`, since
  `new` gets a read-only window), `picked: Option<AgentKind>`; `render_prompt` (the box in a
  bordered rounded frame with `key_context("MarleyHomePrompt")` and `menu::Confirm` →
  `start_prompt`; the installed CLIs as chips, the picked lit with the accent tint; "in <folder>"
  from `default_working_directory`; Start; then `render_start` and `render_agent_links` as subtle
  links); `start_prompt` (the picked or first CLI; empty → `start_cli`, else
  `start_cli_with_prompt`; clears the box); `render_configure`'s buttons subtle with accent
  labels; START and NEW AGENT cards replaced by PROMPT.
- **Deviation:** the prompt editor is made lazily in `render` (`new_page`'s window is read-only,
  and changing it would touch its callers).
- **Gate:** clippy at source (`needless_pass_by_ref_mut` on `start_prompt`); `script/gates.sh
  --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/835-home-one-prompt.sh` (sway, the gated tree), exit 0, three shots;
  one probe before the gate placed the chip and the box.
- `835-01-night`: Home: PROMPT first, "What should an agent do?", the chips Claude Code (lit),
  Codex, Gemini CLI, OpenCode (the box's own CLIs too), "in <repo>", Start; under it New Terminal,
  Open Folder…, Clone Repository…, Command Palette, New Agent…, Threads; then RECENT PROJECTS,
  AGENTS AT WORK, AGENT ACTIVITY; CONFIGURE's seven entries as accent links (REQ-001, REQ-002,
  REQ-004 met).
- `835-02-started`: Codex clicked, "write the README" typed, Enter: a terminal of `repo` runs
  `codex 'write the README'` and the stand-in printed the task (REQ-003 met).
- `835-03-day`: Marley Day, Codex still picked (REQ-001, REQ-004 met).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Changed, #835); the guide's Home section; the architecture note's Home;
  look-plan W2.2 done with #867.
- **Knowledge:** none new (the probe-run practice and #826's prompt path held).
- **Brain:** decision on consultation 042c1401.
- **Ticket** closed; #867 filed; pipeline archived; committed on `marley/workbench-shell`.
