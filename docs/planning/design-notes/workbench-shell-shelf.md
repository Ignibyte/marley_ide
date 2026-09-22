# The workbench-shell shelf: seven queued specs (2026-09-22)

Chad's call, 2026-09-22: Zed stays the base, Warp's layout is the target, and it arrives as
a Marley layout beside Zed's own. Every change outside Marley-owned paths is recorded. The
plan is [workbench-shell.md](../../marley/workbench-shell.md); these are its slices W0 to W6,
minted by `/spec` on his goal "lets create the tickets and make it happen".

| Slice | Ticket | One line |
|---|---|---|
| W0 | #436 | The Zed touchpoint ledger, enforced: gate:16 and a write hook |
| W1 | #437 | Marley's own app identity: `APP_NAME`, the binary, Chad's settings copied once |
| W2 | #438 | The Marley layout switch and the first rail: projects, center terminals, New Terminal |
| W3 | #439 | Zed agent threads in the rail |
| W4 | #440 | Agent CLIs in rail terminals |
| W5 | #441 | Terminal routing and keys in the Marley layout |
| W6 | #442 | Rail persistence and polish |

## Order

436 → 437 → 438 → 439 → 440 → 441 → 442. #438 needs #437's name so its live drive runs
against Marley's own config directory; #439 to #442 each build on #438's crate.

## Before #436: the baseline commit

The port of 2026-09-18 was never committed, so every `--diff` gate would mutate all five
Marley crates again. The first commit on `marley/workbench-shell` is the port plus this plan.
The gpui-era archive specs predate the §20 reference rule; each got a short note saying so,
which is what lets `enforce-warp-reference.sh` pass them unchanged.

Done as TICKET-443 (2026-09-22). The commit sits behind a `--diff` green. Every Marley file was
new to the index, so that run mutated all 603 mutants of the five crates, the same scope a FULL
run covers. The first FULL run had exposed a shared-target race in the gate itself, which #443
fixed.

## Load-bearing findings (2026-09-22 sweeps; the plan cites file:line for each)

- Zed's sidebar is a trait (`workspace::Sidebar`) registered in one deferred callback in
  `crates/zed/src/zed.rs:536-546`; any implementation inherits resizing, open state,
  persistence and the toggle actions.
- The `sidebar` crate must stay linked: its actions are bound in the default keymaps and
  `load_default_keymap` unwraps.
- `SettingsContent` is closed, so a `marley` settings key costs a field, a new file and one
  line in `vscode_import.rs`.
- Bindings added at init die at the first `reload_keymaps`; the Marley keymap needs one line
  in `load_default_keymap`.
- `test_action_namespaces` in `zed.rs` must list every action namespace.
- Defaults can be patched from a crate with `SettingsStore::update_default_settings`.
- The Terminal Panel is the workspace's `TerminalProvider`; tasks reach the center through a
  replacement provider, not a setting.

## Recall pins (knowledge ledger)

- `PR-claude-single-selection-is-a-derived-selector-not-scattered-booleans-001`: the rail's
  one selected row comes from one selector.
- `PR-claude-unmodified-terminal-chords-yield-to-the-pty-001`: Marley keys in a Terminal
  context stay modified.
- `PR-claude-new-setting-needs-nondefault-roundtrip-leg-001`: `marley.layout` gets a
  round-trip test on the non-default value.
- `PR-claude-integration-only-coverage-fails-gate4-001`: driven tests live in-crate, not in
  `tests/`.
- `PR-claude-deferred-gpui-handle-op-needs-notify-in-headless-001`: headless asserts on focus
  or scroll notify first.
