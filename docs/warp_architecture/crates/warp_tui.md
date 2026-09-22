# warp_tui

> Per-crate reference (Marley round 2) — crate dir `crates/warp_tui`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` — a headless front-end **reference template** only. Marley's front-end is **`marley_app` `[Marley-original]`** on `gpui` `[permissive: Apache-2.0]`; `warp_tui` is not carried. See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| Field | Value |
|-------|-------|
| Subsystem | [03 — Terminal & Session Core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`); no own LICENSE marker. |
| Internal deps | 6 |
| Used by | 0 (leaf — binary crate) |

## Purpose

`warp_tui` is the **headless, ratatui-based terminal-UI front-end** for Warp — a thin, login-light
shell that builds a `ChannelState` and hands off to the engine via `warp::run_tui()`. It is a *leaf*
crate (no internal dependents): proof that the terminal engine can run without the full GUI, and the
closest existing template for a stripped-down front-end. Its own library surface is tiny — just an
editor-backed input view; the heavy lifting lives in the `app` crate behind the `tui` feature.

## Key types, modules & public API

`lib.rs` exposes a single module:

- **`input`** (`src/input/mod.rs`):
  - **`TuiInputView`** (`input::view`) — a ratatui-rendered view implementing `TuiView`, backed by a
    `warp::editor::CodeEditorModel` in char-cell mode.
  - **`TuiInputViewEvent`** (e.g. `Submitted`) — events the view emits.
  - `input::kill_buffer` — TUI kill-buffer (Emacs-style) state; session state (scroll, width) lives
    on the view, not a separate model.

The real entry points are the **per-channel binaries** (`autobins = false`, `default-run =
"warp-tui-oss"`):

- `src/bin/oss.rs` → `warp-tui-oss` (the `default-run`; hand-builds a production `ChannelState` with
  `Channel::Oss`, `WarpServerConfig::production()`, `OzConfig::production()`, then calls
  **`warp::run_tui()`**).
- `src/bin/{local,dev,preview,stable}.rs` → `warp-tui`, `warp-tui-dev`, `warp-tui-preview`,
  `warp-tui-stable` — same pattern per channel (some use the `warp_channel_config` generator).
- `examples/tui_input_demo.rs` — standalone demo of `TuiInputView`.

## Depends on (internal)

- [`warp`](./warp.md) — the `app` crate (`features = ["tui"]`); provides `run_tui()` and `CodeEditorModel`.
- [`warp_core`](./warp_core.md) — `Channel`/`ChannelState`/`ChannelConfig`/`WarpServerConfig`/`OzConfig`/`AppId`.
- [`warp_channel_config`](./warp_channel_config.md) — `load_config!` macro for non-OSS channel bins.
- [`warp_editor`](./warp_editor.md) — the editor model behind the input view.
- [`string-offset`](./string-offset.md) — offset arithmetic in the input view.
- [`warpui_core`](./warpui_core.md) — UI runtime (`features = ["tui"]`).

## Used by (internal dependents)

None — `warp_tui` is a terminal **leaf/binary** crate. Total dependents: 0.

## Related crates

- [`warp`](./warp.md) — owns `run_tui()` and the actual PTY engine this front-end drives.
- [`warp_editor`](./warp_editor.md) — sibling text/editor crate.
- [`local_control`](./local_control.md) — the out-of-process way to drive a front-end like this.

## Marley relevance

**Classify: KEEP (reference) → EXTEND/RENAME later.** This is the **single best starting point for
goal (1) expand the UI surface with a custom Ignibyte panel** and a fast path to goal (2)
spawn/write/read: it already boots the engine headlessly via `warp::run_tui()` with minimal login
coupling, so a Marley-branded TUI (or a custom panel) can be prototyped here without the full GUI.
Because it has **zero dependents**, renaming `warp_tui` → `marley_tui` and rebranding its bins
(`warp-tui-*` → `marley-tui-*`, the `AppId::new("dev","warp","WarpTui")` string, `warp-tui.log`) is
**cheap and safe** — do it early. For offline boot (goal 3), this crate is also where you'd inject a
stubbed `ChannelState` (point `WarpServerConfig` at a no-auth local config) before `run_tui()`.

## Notes / gotchas

- `run_tui()` lives in the **`app` crate (`warp`)**, gated behind its `tui` feature — `warp_tui` is a
  thin launcher, not the engine.
- `build.rs` (3.3 KB) exists for bin/channel wiring; `release_bundle` feature is declared but **off
  today** — the local bin uses a runtime `warp_channel_config` generator; embedding the config via
  `build.rs` is "phase 2."
- Tests/examples need `warp_core/test-util` for `Appearance::mock()` to register the `Appearance`
  singleton the input view depends on.
- `edition = 2021` (vs 2024 for `command`/`local_control`).
