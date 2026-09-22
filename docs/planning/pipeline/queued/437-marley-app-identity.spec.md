---
pipeline_id: c3bbb51e-b78c-4bce-be39-e820813882f6
ticket: docs/planning/tickets/open/TICKET-437-marley-app-identity.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active for Phase 2 Design
title: Marley's own app identity (APP_NAME, the binary, Chad's settings copied once)
type: chore
slice: workbench shell W1
references: [docs/marley/workbench-shell.md, docs/marley/zed-touchpoints.md]
---

## Title
Rename the fork's app identity from Zed to Marley so it stops sharing `~/.config/zed`,
`~/.local/share/zed` and `~/.local/state/zed` with the stock Zed installed at
`~/.local/zed.app`, and bring Chad's current Zed settings across once.

## Scope
### In
- `crates/paths/src/paths.rs:18`: `APP_NAME` becomes `"Marley"`. Every config, data, state,
  log and database path derives from it (`config_dir`, `data_dir`, `state_dir`, `logs_dir`,
  `database_dir`).
- `crates/zed/Cargo.toml`: `default-run` and the main `[[bin]]` name become `marley`, which
  the compile-time assert in `crates/zed/src/main.rs:10-16` requires.
- Both hunks carry a `Marley:` comment and get ledger rows in `docs/marley/zed-touchpoints.md`.
- One-time settings copy: `~/.config/zed/settings.json` (and `keymap.json` and `themes/` if
  present) into `~/.config/marley/` when the target does not exist yet. The stock Zed config
  is only read.

### Out (explicitly deferred)
- The Wayland/Windows app id (`ReleaseChannel::app_id`, `crates/release_channel/src/lib.rs:228-233`,
  still `dev.zed.Zed-Dev`), the `cli` launcher (`crates/cli` looks for `zed-editor`),
  bundling scripts, desktop entries, icons and the About text. None of them affect where
  state is stored; each is its own touchpoint when Marley ships a package.
- `script/zed-local` and `script/debug-cli`, which hard-code `target/*/zed`; Marley does not
  use them yet.

## Reference (§20)
Upstream Zed: `crates/paths` names `APP_NAME` as the fork switch ("Forks should change this
to avoid colliding with Zed's user data", `paths.rs:14-18`), and `crates/zed/src/main.rs:8-16`
asserts at compile time that the binary name matches it ("Forks: update APP_NAME in
crates/paths/src/paths.rs when renaming the binary"). The behavior kept is Zed's own
directory layout under the new name.

### Prior art
- **Behavior maps:** none needed; this is Zed's documented fork path.
- **Published material:** the XDG base-directory spec, which `paths.rs` follows on Linux
  (`dirs::config_dir()` / `data_local_dir()` / `state_dir()` joined with the lowercased name).
- **Code we already ship:** `APP_NAME` and `APP_NAME_LOWERCASE` (`paths.rs:14-40`) and every
  derived path function (`:120-270`); `CUSTOM_DATA_DIR` (`--user-data-dir`) as the escape
  hatch tests can use; the main.rs assert. Nothing else in the tree reads `APP_NAME`
  (checked: only `paths.rs` and `main.rs`).

## UI proof
UI-AFFECTING (light): the running app's config and theme come from the copied settings.
- Driven/unit: a `paths` unit test that the derived config, data and state directories end
  in `marley` (lowercase on Linux).
- Live drive: `cargo run`, confirm the process is `target/debug/marley`, that
  `~/.config/marley/settings.json` is read (Chad's theme and font sizes show) and that
  `~/.local/share/marley/db` is created; screenshot through `dev-box-desktop`.

## Locked-In Decisions
- D1 — The display name is `Marley` and the lowercase directory and binary name is `marley`.
- D2 — Only the two touchpoints that move state and the binary are in this ticket.
- D3 — The settings copy never overwrites an existing `~/.config/marley` file and never
  writes under `~/.config/zed`.
- D4 — A fresh `~/.local/share/marley` counts as a new install, so `agent_ui`'s one-time
  editor-layout backfill does not write into Chad's settings (workbench-shell D6).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the workspace is built, the app binary shall be `target/debug/marley` and `cargo run` shall start it | build + `ls`, live drive |
| REQ-002 | WHILE Marley runs on Linux, its config, data and state directories shall be `~/.config/marley`, `~/.local/share/marley` and `~/.local/state/marley` | unit test on the derived paths; live check of the created dirs |
| REQ-003 | WHILE Marley runs, it shall not create or modify files under `~/.config/zed` or `~/.local/share/zed` | live drive: mtimes before and after |
| REQ-004 | WHEN W1 completes, `~/.config/marley/settings.json` shall hold Chad's current Zed settings, with `~/.config/zed/settings.json` unchanged | `cmp` the two files; stock mtime unchanged |
| REQ-005 | The two hunks shall carry `Marley:` comments and rows in the ledger | gate:16 green; `rg "Marley:"` |
| REQ-006 | The workspace shall still build and the touched crates' tests shall pass | `script/gates.sh --diff` |

## Phase Plan
- **P2 Design** — the exact hunks, the unit test's platform guard, the copy commands.
- **P3 Implement** — the two hunks, the ledger rows.
- **P3.5 Inspect** — critics: anything else keyed on the old name (sockets, keychain
  service names, `--user-data-dir`), accidental writes to the stock config.
- **P4 Validate** — unit test, `script/gates.sh --diff`, the live drive with the copied
  settings.
- **P5 Complete** — CHANGELOG, ledger, close, archive.
