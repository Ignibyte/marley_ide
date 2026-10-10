# Marley's agents get the Rustal STE skill by default — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-725-marleys-agents-get-rustal-ste.md
- **Pipeline spec:** 725-marleys-agents-get-rustal-ste.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09: "for the skill lets adapt it and take it and use it for rustal
  specific instructions across agents", then "have the agents in zed install it by default as
  well".
- **Recall:**
  - #709 wrote a digest folder read-only and put it first in `CLAUDE_CODE_PLUGIN_DIRS` for local
    interactive terminals (`marley_terminal::identity`).
  - Zed reads `builtin_skills()` on every skill load (`agent::combine_skills`), and
    `builtin_skill_content` serves a built-in's file to mentions.
  - The Marley entry builds its own env (`assistant.rs`, around line 746).
- **Design:**
  - **The files:** `crates/marley_workbench/agent_skills/rustal-ste/` (copied from the store).
  - **`agent_skills` (Zed crate):** a `RwLock<Vec<(&'static str, &'static str)>>` of registered
    built-ins. `register_builtin_skill`/`unregister_builtin_skill` change it; `builtin_skills`
    and `builtin_skill_content` read it after `create-skill`. A ledger row first.
  - **`rustal_skill.rs` (new, Marley crate):**
    - `FILES` (the path and the embedded text) and their digest.
    - `init` reconciles now and on every settings change.
    - When on: register the built-in; write the plugin folder in a background task, then set
      `identity::set_skill_plugin`; write the Codex copy with its marker.
    - When off: unregister, clear the plugin folder setting, and remove the Codex copy if its
      marker is there.
    - IO goes through `*_in(dir)` functions.
  - **`marley_terminal::identity`:** `SKILL_PLUGIN`, `set_skill_plugin`, `skill_plugin`;
    `plugin_dirs` takes several folders.
  - **`assistant.rs`:** the Claude entry's env gets `CLAUDE_CODE_PLUGIN_DIRS` with the folder.
  - **Settings:** `MarleyContent.rustal_ste_skill`, `default.json`, and the Agents section's toggle
    (Zed paths; rows first).
- **File manifest:**
  - Zed: `agent_skills.rs`, `settings_content/src/marley.rs`, `default.json` and `marley_page.rs`.
  - Marley: `marley_workbench` (`rustal_skill.rs`, `marley_workbench.rs`, `assistant.rs`, the
    skill files) and `marley_terminal` (`identity.rs`).
  - Docs: the touchpoints and the guide.
  - The scenario.

## Phase 2 — Code
- **Built:**
  - **The skill's files** in `crates/marley_workbench/agent_skills/rustal-ste/` (a copy of the
    store, without the examples), scanned for hosts, addresses and home paths: none.
  - **`agent_skills.rs` (Zed, row first):** `REGISTERED_BUILTINS`, `register_builtin_skill` and
    `unregister_builtin_skill`; `builtin_skills` and `builtin_skill_content` read them after
    `create-skill`.
  - **`rustal_skill.rs` (new):**
    - `FILES` and `digest`;
    - `reconcile` on start and on settings changes;
    - `install_plugin_in` (digest folder, read-only, staged rename);
    - `write_codex_copy_in` and `remove_codex_copy_in` (the `.marley-owned` mark, never over or
      away a folder without it);
    - `plugin_folder` for the Marley entry.
  - **`identity.rs`:** `set_skill_plugin`/`skill_plugin`; `plugin_dirs` takes Marley's folders in
    order; `plugin_dirs_for`.
  - **`assistant.rs`:** the Claude entry's env puts the folder first in `CLAUDE_CODE_PLUGIN_DIRS`.
  - **Settings:** `rustal_ste_skill` (default true), `default.json`, and the Agents toggle.
  - **`guide.md`:** a section.
- **Review:**
  - **The e2e runner** started a real Marley with the user's own `HOME`, and Codex's home wasn't
    isolated. With the setting on by default, every scenario would have written into the user's
    `~/.codex/skills`. So `script/e2e.sh`'s settings copy turns `rustal_ste_skill` off, and every
    run exports `CODEX_HOME` into its own folder (#668's rule). No scenario ran before the fix;
    `~/.codex/skills` has no `rustal-ste`.
  - **The Marley entry** names the folder by its digest path at once. The folder is written
    milliseconds after start, before any agent can start.
- **Gate:**
  - `725-gate-1.log` RED, twice over:
    - clippy (`into_iter` on a one-item array, replaced by `iter::once`);
    - typos (the copied skill's linter regex and the glossary's component name, so
      `crates/marley_workbench/agent_skills/` is excluded as copied content, with the typos row
      extended).
  - `725-gate-2.log` RED: the ledger row itself quoted the two words; reworded.
  - `725-gate-3.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/725-marleys-agents-get-rustal-ste.sh`, under `compositor sway`. The
  runner's copy turns the skill off and exports `CODEX_HOME` into the run; the scenario turns it
  on. Runs used `nice -n 19` beside the harness's gate.
- **Run a (`shots-725a`):** every file check passed, but 725-01 proved nothing. `agent: new thread`
  opened this profile's default agent, Claude Agent, not Zed's own, and the `/rustal` text never
  reached its editor.
- **Run b (`shots-725b`): every check passes, and every shot shows its criterion.** A run keymap
  binds Ctrl+Alt+Shift+Z to `agent::NewExternalAgentThread` with `"Zed Agent"`, as #633 bound a
  key.
  - **725-01-zed-skills (REQ-001):** "New Zed Agent Thread", `/rustal`, and the menu reading
    "Skills: rustal-ste built-in" with the skill's description.
  - **725-02-terminal (REQ-002):** a new terminal ran the echo; its file reads
    `dirs=…/profile.*/claude-code/rustal-ste/93f2cf6b…`, the digest folder under the run's data
    directory.
  - **REQ-003:** `$CODEX_HOME/skills/rustal-ste/SKILL.md` and `.marley-owned` exist.
  - **725-03-off (REQ-004):** after the setting was turned off from outside, a new terminal's file
    reads `dirs=` (empty), and the Codex copy is gone.
  - **The user's folder (REQ-003):** a `rustal-ste` folder with the user's own `USER.md` survives
    turning the setting on again, with no mark added.
  - **The user's real `~/.codex/skills`** holds no `rustal-ste` after both runs.
- **Not covered:** the Marley entry on Claude Code reading the variable. It needs a signed-in
  Claude Code. The env line is the same folder `plugin_folder` gives, as reviewed in Phase 2.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added);
  - `docs/marley_architecture/marley_workbench.md` (a section);
  - the guide (Phase 2).
  - The touchpoint rows describe the shipped hook, setting, default, toggle and typos exclusion.
- **Knowledge appended:** L-claude-725-a-default-on-feature-that-writes-outside-marley-must-be-off-in-e2e-001.
- **Ticket:** closed.
- **Gate:** `725-gate-4.log`, GATE GREEN [diff], on the tree committed.
