# Marley's own app identity — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-437-marley-app-identity.md
- **Pipeline spec:** 437-marley-app-identity.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad answered "Rename to Marley now" (2026-09-22) after the plan found the fork
  sharing Zed's config and database with the stock Zed at `~/.local/zed.app` (updated
  2026-09-22 14:06; `~/.config/zed/settings.json` touched 14:07).
- **Classification / tier:** chore; two tiny Zed touchpoints, no Marley crate.
- **Recall (§18.3):** nothing on this seam in the ledgers (the gpui era had its own name).
  `PR-claude-rename-sweeps-intradoc-links-001` applies in spirit: sweep for other readers of
  the old name (only `paths.rs` and `main.rs` read `APP_NAME`).
- **Discovery:** `crates/paths/src/paths.rs:14-40` (the constants), `:120-270` (derived dirs);
  `crates/zed/Cargo.toml:9` (`default-run`), `:56-58` (`[[bin]]`); `crates/zed/src/main.rs:10-16`
  (the assert); Chad's current settings: `agent_servers.claude-acp`, `base_keymap: Zed`, font
  sizes, the One Light / One Dark theme pair.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.

## Phase 2 — Design (drafted while #443's first FULL run finished)
- **Hunks.**
  - `crates/paths/src/paths.rs:18`: `APP_NAME` becomes `"Marley"` under a `// Marley:` line;
    a Linux-gated unit test at the foot of the file asserts `config_dir()`, `data_dir()` and
    `state_dir()` end in `marley`. The crate has no tests today, so this is its first.
  - `crates/zed/Cargo.toml:9` `default-run` and `:57` the main `[[bin]]` name become
    `marley`, each under a `# Marley:` line. `main.rs:10-16` checks the pair at compile time.
  - Ledger rows for both files, added before the edits (the #436 hook enforces the order).
- **Left alone on purpose:** the crash handler's `binary: "zed"` label (`main.rs:401`), the
  clap command name (`main.rs:1693`), the bundle metadata (`crates/zed/Cargo.toml:282-309`),
  the release channel's app id, and `crates/cli`'s `zed-editor` lookup. None of them move
  state; each can be its own touchpoint when Marley ships a package.
- **Settings copy (the one environment action):** `install -Dm600 ~/.config/zed/settings.json
  ~/.config/marley/settings.json` only when the target is absent; the same for `keymap.json`
  if it exists (it does not today) and `themes/` (empty today). Nothing under `~/.config/zed`
  is written.
- **Test plan:** the `paths` unit test (REQ-002); a build showing `target/debug/marley`
  (REQ-001, REQ-003); the live drive that launches it and lists the new directories and the
  unchanged mtimes of the stock ones (REQ-002 to REQ-004); gate:16 and `rg "Marley:"`
  (REQ-005); `script/gates.sh --diff` (REQ-006).
