//! `marley_terminal` — the UI-agnostic PTY shell session and per-command **Block** model.
//!
//! This crate spawns a PTY-backed shell, accepts byte/command writes, reads the shell's output
//! back, and segments that output into [`Block`]s — one per command — using the shell bootstrap's
//! DCS hook metadata (cwd, git branch, exit code, originating subshell) rather than output
//! heuristics. It owns **no rendering and no GUI**; it produces a [`BlockList`] any front-end can
//! observe. The PTY byte engine and ANSI cell grid come from the reused `alacritty_terminal`.
//!
//! The session's process-unique identity is [`SessionId`] (this crate's process-unique session
//! counter). The id a shell *self-reports* in its bootstrap is the distinct, correlation-only
//! [`ShellSessionId`], mapped to the session's `SessionId` for subshell detection.
//!
//! ## The pure / shim seam
//!
//! Everything that decides *behavior* is **pure** and unit-testable without a PTY:
//!
//! - [`dcs`] — the stateless codec: [`encoding_for_dcs_terminator`] (terminator → encoding) and
//!   [`decode_hook`] (bytes → [`DcsHook`]), plus the incremental DCS byte scanner.
//! - [`block`] — the value types and the ordered [`BlockList`].
//! - [`apply`] — the stateful model: the staged-prompt buffer, the
//!   [`ShellSessionId`] → `SessionId` registry, and the `apply_hook` transition machine.
//! - [`session`] — [`TerminalSession`]: the write re-queue loop, the `pump` read/decode/apply/render
//!   orchestration, and `resize`, all driven over a small `PtyChannel` trait so they are mock-tested.
//!
//! The only **shim** is the private `pty_os` module: the raw `alacritty_terminal::tty` spawn, the
//! leader-fd read/write, the `rustix` `tcsetwinsize`, and the `next_child_event` reap. These four
//! OS calls have no deterministic unit harness; they are the crate's sole ACCEPTED-UNTESTABLE
//! surface, exercised by the real-PTY integration tests.
#![deny(missing_docs)]
// gate:21 runs Zed's dylint lints (`tooling/lints`) with these as errors in the Marley crates;
// Zed's crates keep them at warn (CONSTITUTION §0).
#![cfg_attr(
    dylint_lib = "lints",
    deny(
        async_block_without_await,
        blocking_io_on_foreground,
        entity_update_in_render,
        map_lookup_then_insert,
        notify_in_render,
        owned_string_into_shared,
        shared_string_from_str_literal
    )
)]

pub mod anchored;
pub mod apply;
pub mod block;
pub mod dcs;
pub mod keys;
pub mod mouse;
pub mod session;
pub mod shell_integration;
pub mod styled;

mod session_id;

pub use anchored::{AnchoredBlock, AnchoredBlocks};
pub use block::{
    Block, BlockCopy, BlockId, BlockIndex, BlockList, BlockState, ExitCode, PromptInfo,
    ShellSessionId,
};
pub use dcs::{
    DcsEncoding, DcsHook, DecodeError, PrecmdValue, PreexecValue, decode_frame, decode_hook,
    encoding_for_dcs_terminator,
};
pub use keys::{KeyCode, KeyInput, Route, ctrl_byte, encode_key, input_route, paste_bytes};
pub use mouse::{MouseEvent, MouseModes, MouseMods, mouse_report};
pub use session::{ApplyHookError, SessionError, SessionEvent, SessionOptions, TerminalSession};
pub use session_id::SessionId;
// The scanner's frame, which `decode_frame` reads.
pub use marley_dcs::RawDcs;
pub use styled::{StyledLine, StyledRun, coalesce_row};

// The alacritty ANSI types carried in the public styled model — re-exported so consumers
// (the render) can match on them without a direct `alacritty_terminal` dependency.
pub use alacritty_terminal::term::cell::Flags;
pub use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
