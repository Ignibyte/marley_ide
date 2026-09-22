# warp_terminal

> Per-crate reference (Marley round 2) — crate dir `crates/warp_terminal`. Marley is forked from Warp ([warpdotdev/warp](https://github.com/warpdotdev/warp)).
>
> **Provenance:** `[Warp-derived: AGPL-3.0]` UI-agnostic terminal **model** + `BlockId`/`BlockIndex`, embedding `[permissive: Alacritty Apache-2.0]` grid/ANSI. Marley does **not** fork this — it reimplements the Block model as **`terminal_blocks` `[Marley-original]`** directly on the upstream `alacritty_terminal` + `vte` crates (`[permissive]`). See subsystem [Provenance & licensing](../subsystems/03-terminal-session-core.md#provenance--licensing).

| Field | Value |
|-------|-------|
| Subsystem | [03 — Terminal & Session Core](../subsystems/03-terminal-session-core.md) |
| License | AGPL v3 (`license.workspace = true` → `AGPL-3.0-only`). Carries `src/model/LICENSE-ALACRITTY` (Apache-2.0) covering the Alacritty-derived grid/ANSI code embedded in `model/`. |
| Internal deps | 6 |
| Used by | 2 |

## Purpose

`warp_terminal` is the **UI-agnostic terminal model layer** — the shared, lower-level data
primitives for a terminal grid and its escape/ANSI vocabulary, adapted from Alacritty. It owns
*no PTY and no rendering*; it is pure data + parsing primitives that the richer app-level model
(`app/src/terminal/model/*`) builds on. It exists so terminal-cell storage, block identity, and
control-sequence handling live in one place that lower crates can depend on without pulling in the
`app` crate or any GUI.

## Key types, modules & public API

`lib.rs` exposes three modules: `model`, `shell`, and a private `shared_session` shim.

- **`model`** (`src/model/mod.rs`) re-exports the core identity and mode types:
  - **`BlockId`** (`model::block_id`) and **`BlockIndex`** (`model::block_index`) — the stable
    per-command block identity / ordering used throughout the block model.
  - **`TermMode`**, **`KeyboardModes`**, **`KeyboardModesApplyBehavior`** (`model::mode`) — terminal
    mode flags incl. the kitty keyboard protocol state.
  - Submodules: `model::grid` (flat cell storage — rows, dimensions, graphemes, attribute maps),
    `model::ansi` (control-sequence parameter types), `model::escape_sequences` (the `C0` constants
    `CR`/`LF`/`EOT`/`ESC` consumed by the app's `PtyController`; kitty keyboard encoding — see
    `model/ESCAPE_SEQUENCES.md`), `model::mouse`, `model::char_or_str`, `model::indexing`.
- **`shell`** (`src/shell/mod.rs`) — shell-detection / path-conversion helpers (MSYS2/WSL↔Windows
  path translation, version comparison) keyed off `warp_core::platform::SessionPlatform`.
- **`shared_session`** (private) — converts `model` `Point` to/from
  `session_sharing_protocol::common::Point`.

A consumer reads/writes cells via `model::grid`, references blocks by `BlockId`/`BlockIndex`, and
uses `model::escape_sequences` constants when emitting control bytes.

## Depends on (internal)

- [`channel_versions`](./channel_versions.md) — version/override targets (`overrides::TargetOS`) used by `shell`.
- [`string-offset`](./string-offset.md) — byte/char offset arithmetic over terminal text.
- [`warp_completer`](./warp_completer.md) — `completer::{CommandExitStatus, CommandOutput}` used by `shell`.
- [`warp_core`](./warp_core.md) — `SessionId`, `platform::SessionPlatform`, `paths` for shell logic.
- [`warp_util`](./warp_util.md) — path conversion helpers (`convert_wsl_to_windows_host_path`, etc.).
- [`warpui_core`](./warpui_core.md) — base UI/runtime primitives (geometry, get-size).

## Used by (internal dependents)

- [`ai`](./ai.md) — consumes terminal model types for agent context.
- [`warp_terminal`'s primary consumer, `warp`](./warp.md) — the `app` crate's `app/src/terminal/model/*` builds its `Block`/`BlockGrid`/`BlockList` on these primitives.

Total dependents: 2.

## Related crates

- [`warp_core`](./warp_core.md) — supplies `SessionId` that ties blocks to subshells.
- [`warp_editor`](./warp_editor.md) and [`warpui_core`](./warpui_core.md) — text/rendering siblings.
- The non-crate engine in `app/src/terminal/model/` (`block.rs`, `blocks.rs`, `blockgrid.rs`,
  `ansi/`) that wraps these primitives — see architecture doc §6.

## Marley relevance

**Classify: KEEP (rebrand surface).** This is foundational, UI-neutral terminal data and is
needed by any front-end that renders or inspects output — directly serving goal (1) *expand the UI
surface*: a custom Ignibyte panel reads cells/blocks through `model::grid` + `BlockId`/`BlockIndex`
rather than re-implementing a grid. It is largely Warp-neutral, so the **de-Warp rebrand (goal 4)**
touch here is minimal — confined to the embedded Alacritty attribution (keep the
`LICENSE-ALACRITTY` notice) and any `WarpTerminal`-ish strings, of which there are essentially
none in this crate. Do **not** rename the package early: it has the `app` crate (`warp`) as a hard
dependent and renaming ripples. Leave PTY spawn/write/read (goal 2) to `app/src/terminal/*`; this
crate is the read-model substrate behind it.

## Notes / gotchas

- The `model/` subtree is **adapted from Alacritty** — keep `LICENSE-ALACRITTY` (Apache-2.0) and its
  attribution if redistributing; the rest of the crate is AGPL.
- `test-util` feature gates test-only helpers; `dev-dependencies` pulls `unicode-segmentation`.
- Heavy use of `bitflags` + `bitflags-serde-legacy` and `static_assertions` for mode flags — changing
  flag layouts can break serialized state and `static_assertions` checks.
- No PTY, no async, no I/O here — purely synchronous data. The actual byte engine is in `app`.
