# computer_use

> Per-crate reference (Marley round 2). Crate dir: `crates/computer_use`. Marley is forked from Warp (warpdotdev/warp).
> Provenance: [Warp-derived/AGPL] over [permissive/public: OS input/capture APIs] · Marley status: gap / clean-room-target — generic OS-automation capability, NOT the brain; reimplement from the public concept or defer (noop) for early builds.

| | |
|---|---|
| **Subsystem** | [agent-ai-mcp](../subsystems/04-agent-ai-mcp.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`; no per-crate LICENSE marker) |
| **Internal deps** | 2 |
| **Used by** | 2 |

## Purpose

`computer_use` gives the agent **OS-level GUI control**: synthetic mouse/keyboard input and
screen capture, abstracted across macOS, Windows, Linux/X11, and Linux/Wayland. It is the
backend that fulfils the agent's `UseComputer` action (defined in [`ai`](./ai.md)) —
"move the mouse here, click, type this, take a screenshot." It also ships a small CLI
(`use_computer`) for manual testing.

## Key types, modules & public API

The public surface is in `src/lib.rs`; per-OS implementations live behind a `cfg_attr`
path swap (`mac/`, `windows/`, `linux/{x11,wayland}/`) all exposed as the internal `imp`
module, with a `noop` fallback.

- **`trait Actor: Send + Sync + 'static`** — the core abstraction. `platform() -> Option<Platform>` and `async perform_actions(&mut self, actions: &[Action], options: Options) -> Result<ActionResult, String>`.
- **`create_actor() -> Box<dyn Actor>`** — factory returning the platform impl (or `noop::Actor` under feature `test-util`).
- **`is_supported_on_current_platform() -> bool`**.
- **`enum Platform`** — `Mac | Windows | LinuxX11 | LinuxWayland`.
- **`enum Action`** — `Wait(Duration)`, `MouseDown{button, at}`, `MouseUp`, `MouseMove{to}`, `MouseWheel{at, direction, distance}`, `TypeText{text}`, `KeyDown{key}`, `KeyUp{key}`. Coordinates use re-exported **`Vector2I`** (from `pathfinder_geometry`).
- **`enum Key`** — `Keycode(i32)` (platform virtual keycode / X11 keysym) or `Char(char)` (BMP only on Windows).
- **`enum MouseButton`**, **`enum ScrollDirection`**, **`enum ScrollDistance`** (`Pixels(i32)` / line-based).
- **`struct ScreenshotParams`**, **`struct ScreenshotRegion`** (`validate()`), **`struct Screenshot`**, **`struct Options`**, **`struct ActionResult`**.
- **Binary `use_computer`** (`src/bin/use_computer.rs`, `default-run`) — clap CLI with `Click`, `Text`, `Screenshot`, … subcommands for manual hardware testing.

## Depends on (internal)

- [`command`](./command.md) — process/command execution (macOS path, e.g. shelling out for permissions/capture helpers).
- [`warpui_core`](./warpui_core.md) — shared runtime types (pulled in on every real-OS target).

> Heavy *external* per-OS deps: `objc2`/`objc2-app-kit`/`objc2-core-graphics` + `dispatch2` (macOS), `windows` (Win32 input/GDI/HiDPI), `x11rb` + `ashpd`/`zbus` (Linux X11 + Wayland portals), plus `image` for screenshots.

## Used by (internal dependents)

- [`ai`](./ai.md) — embeds this crate's action/result types into the agent action model (`UseComputer`).
- [`warp`](./warp.md) — the app instantiates an `Actor` and executes approved computer-use actions.

## Related crates

- [`ai`](./ai.md) — owns the `UseComputer` action that this crate executes.
- [`mcp`](./mcp.md) — sibling agent "capability" backend (external tools vs. local GUI control).

## Marley relevance

**Classify: KEEP (optionally STUB for headless/early builds).** This is a clean,
provider-neutral capability crate with no Warp-cloud or auth coupling — pure OS automation.

1. **Custom panel** — our panel could surface/preview computer-use screenshots and gate
   actions behind approval, but no change to this crate is required.
2. **Session spawn/write/read** — orthogonal; it's a tool the session *invokes*, reusable as-is.
3. **De-auth + login stub** — not involved; no rebrand-blocking auth here.
4. **De-Warp rebrand** — minimal: only `warpui_core` in the dep list and a `warp-`prefixed
   thread name elsewhere. The package name `computer_use` is already generic.

For an early offline Marley boot we can **force the `noop`/`test-util` actor** (feature flag)
to skip OS permission prompts (macOS Accessibility/Screen-Recording) and ship without the
GUI-automation risk surface, then re-enable real `create_actor()` later. So: KEEP the crate,
STUB the actor by config until we want it.

## Notes / gotchas

- **Build uses `cfg_aliases`** (`build.rs` + `cfg_aliases = "0.2.1"`) to define the `macos`/`linux`/`windows`/`noop` cfgs that drive the `cfg_attr(path=…)` module swap — these are *custom* cfgs, not the standard `target_os`.
- macOS requires **Accessibility** (input synthesis) and **Screen Recording** (capture) TCC permissions at runtime; first use triggers OS prompts.
- Linux splits X11 (`x11rb` with `xtest`) and Wayland (XDG desktop portals via `ashpd`/`zbus`) — Wayland capture/input goes through portals and may show its own consent dialogs.
- `Key::Char` on Windows is **BMP-only**; supplementary-plane chars must use `Action::TypeText`.
- Test builds depend on `computer_use` with `features = ["test-util"]` (a self dev-dependency) to get the deterministic `noop` actor.
