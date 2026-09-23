# vendor/

Upstream crates Marley carries in this repository instead of taking them from their own
source, because Marley changes them. The root `Cargo.toml` points the build at each copy
with `[patch]` and keeps `vendor/` out of Zed's workspace, so Zed's formatting and lints,
and Marley's gates, never judge upstream code. The copies build only as patched
dependencies.

`vendor/` is Marley-owned for the Zed ledger (`docs/marley/zed-touchpoints.md`). This file is
the record of what each copy is and how it differs from upstream.

## alacritty_terminal

- **Why:** the block terminal catches Marley's shell hooks in alacritty's event loop (the
  three-prong plan's D1, `docs/marley/three-prong-plan.md`). Chad chose an in-repo copy over a
  fork of its own on 2026-09-23 (#461).
- **Source:** `https://github.com/zed-industries/alacritty`, rev
  `4c129667ce56611becdc82de6e28218c80e2e88f` (the rev Zed pins in the root `Cargo.toml`),
  directory `alacritty_terminal/`.
- **Carried:** `Cargo.toml`, `CHANGELOG.md` and `src/`, plus Marley's `Cargo.lock` for the
  standalone tests. `LICENSE-APACHE` is the repository root's file; upstream's crate directory
  holds a symlink to it.
- **Left out:** `tests/`, whose `ref` directory holds 46 MB of recorded sessions. To run those
  tests against a Marley hunk, copy the hunk into the cargo git checkout of the same rev and
  run `cargo test` there.
- **License:** Apache-2.0.
- **Marley's hunks** (each marked `Marley:` in the file):
  - `Cargo.toml`: `edition = "2024"` and `rust-version = "1.85.0"` in place of
    `.workspace = true`, alacritty's workspace values, since the copy has no alacritty
    workspace to inherit from (#461); the `marley_dcs` path dependency (#462).
  - `src/marley_hooks.rs`, all Marley's: `ShellHook`, `HookPosition` and
    `advance_with_hooks`, which takes Marley's shell hook frames out of each read and reports
    each one with the grid position where it fell (#462).
  - `src/lib.rs`: `pub mod marley_hooks` (#462).
  - `src/event.rs`: the `Event::ShellHook` variant and its `Debug` arm (#462).
  - `src/event_loop.rs`: the scanner in `State`, and `advance_with_hooks` in place of
    `parser.advance` in `pty_read` (#462).
  - `src/grid/mod.rs`: `evicted_lines`, with `serde(default)`, its accessor, and the three
    places history lines are dropped (`update_history`, `scroll_up`, `clear_history`) (#462).
  - `src/grid/tests.rs`: the two `evicted_lines` tests (#462).
- **Standalone tests:** `Cargo.lock` is the copy's own, and gate:3 runs `cargo test --locked
  --manifest-path vendor/alacritty_terminal/Cargo.toml`. Regenerate the lockfile when the
  copy's dependencies change.

### Re-sync when Zed moves its pin

1. Find the new rev in the root `Cargo.toml` (`alacritty_terminal = { git = ..., rev = ... }`)
   and check it out: `git clone https://github.com/zed-industries/alacritty <dir>` and
   `git -C <dir> checkout <rev>`.
2. Replace `Cargo.toml`, `CHANGELOG.md` and `src/` with `<dir>/alacritty_terminal`'s, and
   `diff -r` the new `src/` against the checkout.
3. Re-apply every hunk listed above, regenerate `Cargo.lock` with `cargo test --manifest-path
   vendor/alacritty_terminal/Cargo.toml`, and update the rev in this file.
4. Run `script/gates.sh --diff`.
