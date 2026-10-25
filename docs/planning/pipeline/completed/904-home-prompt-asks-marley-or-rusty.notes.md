# Home's prompt asks Marley or Rusty — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-904-home-prompt-asks-marley-or-rusty.md
- **Pipeline spec:** 904-home-prompt-asks-marley-or-rusty.spec.md

## Phase 1 — Plan
- **Request:** split from #903 (2026-10-23); Chad, 2026-10-25: "lets continue with the goal of
  completing each ticket and finishing them". Top of the queue.
- **Classification / tier:** feature, S. Marley crate only (`marley_workbench`); no Zed hunk.
- **Recall (§18.3):**
  - F-claude-867: Home must not open a workspace inside its own update; the Marley and Rusty
    starts run in Home's workspace through `start_thread`'s spawn, so nothing reads Home there.
  - PR-claude-735: `talk_to`'s non-fresh path defers `open_thread`; the fresh path used here
    spawns, so the workspace update ends first.
  - F-claude-903: stand-in agent CLIs first on the terminals' PATH through `.bashrc`; the
    entries run `MARLEY_ASSISTANT_ADAPTER`'s stand-in, never a real Claude Code.
  - The brain: nothing on this seam (consultation 861c24bbba224836bc7f9e5ff3cdcf1e).
- **Discovery:** `home_page.rs` (`render_prompt`, `render_project_chip`, `start_prompt`);
  `assistant.rs` (`Entry`, `talk_to`, `ask_beside`, `marley_present`, `rusty_present`);
  `thread_tab.rs` (`start_thread`, `ThreadSpec.initial`).

### Design
- `home_page.rs`: `enum Pick { Marley, Cli(AgentKind), Rusty }`; `picked: Option<Pick>`, the
  default `Cli(first installed)`. `render_prompt` builds the chips Marley, the CLIs, Rusty; a
  Marley or Rusty chip draws its icon in the tile in place of letters. `render_project_chip` takes
  the picked entry's name and draws the chip disabled, with a tooltip, while it is set. `start_prompt` matches the pick: `Cli` as
  today; `Marley` and `Rusty` call `assistant::start_with_task` in Home's workspace.
- `assistant.rs`: `start_with_task(name: &str, task: &str, …)` looks the entry up by name, as
  `talk_by_name` does, so `Entry` stays private. It shares `talk_to`'s fresh path: `talk_to`
  gains an `initial: Option<AgentInitialContent>`.
- `thread_tab.rs`: `start_thread` gains `initial: Option<AgentInitialContent>`, set on its
  `ThreadSpec`; its callers pass `None`.
- File manifest: `home_page.rs`, `assistant.rs`, `thread_tab.rs` (Marley crate
  `marley_workbench`), plus each other caller of `start_thread`. No Zed path.

### Visual check plan
- REQ-001: Rusty on (stand-in server), the Marley entry on Claude Code with the scripted
  adapter; launch on Home; shot the prompt.
- REQ-002: click the Marley chip; shot.
- REQ-003: type a task, press Enter; wait; shot the new tab with the sent message and the
  stand-in's echo. Back on Home, click Rusty, type, Start; shot.
- Marley Day: write the theme into the profile's settings; shot Home with Rusty picked.
- Not reachable: Zed's own agent as the entries' agent (its profile goes on once the thread
  exists, and the scenario has no model provider); reviewed in the diff only.

### Risks
- On Zed's own agent the entry's profile goes on the thread when the view connects, and the
  auto-sent message is sent from the thread view's constructor; the model request itself runs
  later, so the profile is in place by then. Checked by reading the code, not run.

## Phase 2 — Code
- **Built:** `Pick` and the Marley and Rusty chips (`home_page.rs`); `assistant::start_with_task`
  and `talk_to`'s `initial`; `thread_tab::start_thread`'s `initial`; `agents::start_directory`,
  used by every Marley caller of `default_working_directory`.
- **Deviations:** D2 changed after the first visual run: the project chip greys out in place
  instead of hiding (F-claude-904-a-chip-hidden-on-pick-moved-the-row-001). REQ-004 and
  `start_directory` were added after the Day shot read "config"
  (F-claude-904-a-hidden-worktree-took-homes-terminal-folder-001).
- **Review:** no entity is read in its own update (the conversation opens from
  `start_thread`'s spawn); no Zed hunk; Google style: one enum for the pick, no tuple in a
  signature, `if let` over `entry_name`, a type annotation on `picks`.
- **Gate:** `just gate-diff` 17/17 green. cargo-audit went red on RUSTSEC-2026-0387
  (crypto-bigint, published 2026-10-23), which came in from Zed's AWS SDK with no fixed version
  in range; listed in `.cargo/audit.toml` under its inherited-advisory policy, with the reason.

## Phase 3 — Test
- **Scenario:** `script/e2e/904-home-prompt-asks-marley-or-rusty.sh` (sway), stand-in ACP agent
  behind both entries, Rusty's stand-in server, stand-in CLIs on the terminals' PATH.
- `904-01-chips`: Marley first, the CLIs, Rusty last; Claude Code lit as the default (REQ-001).
- `904-02-marley-picked`: Marley lit; the project chip greyed in place, the chips unmoved
  (REQ-002).
- `904-03-marley-thread`: a "which theme am I on" tab, Marley's cat in the rail, the task sent
  and "Noted: which theme am I on" back; Details names Marley (REQ-003).
- `904-04-rusty-thread`: a "what is on my list today" tab, the butler mark and "Rusty · idle",
  the task sent and answered; Details names Rusty (REQ-003).
- `904-05-day`: Marley Day, Rusty lit, the chip greyed and reading Home with both conversations
  open (REQ-001, REQ-002, REQ-004).
- `904-06-cli-home`: Claude Code from Home: "stand-in claude in /home/cpeppers was asked: tidy
  the notes", the home folder, not Marley's settings folder (REQ-004).
- Earlier runs: the Rusty click missed when the chip hid (fixed, above); the Day shot read
  "config" (fixed, above).
- Not reached: Zed's own agent behind the entries (no model provider in a run); read in the
  diff only.

## Phase 4 — Complete
- CHANGELOG (Added and Fixed), `docs/marley_architecture/marley_workbench.md`, the look-plan row
  (already names #904). No Zed path, so no ledger row.
- Knowledge: F-claude-904-a-hidden-worktree-took-homes-terminal-folder-001,
  F-claude-904-a-chip-hidden-on-pick-moved-the-row-001.
