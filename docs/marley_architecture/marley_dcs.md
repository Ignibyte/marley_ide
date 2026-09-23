# `marley_dcs`

Marley's shell-hook frames found in a terminal's byte stream, split out of `marley_terminal`'s
`dcs.rs` in #462 so that the terminal emulator Marley carries (`vendor/alacritty_terminal`) can
use the scanner: `marley_terminal` depends on `alacritty_terminal`, so the scanner could not
stay there without a cycle. Pure, with no dependencies, MIT OR Apache-2.0.

## What it decides

- **The frame.** A hook is `ESC P <selector> <payload> ESC \`, and Marley's selectors are `h`,
  `p` and `q` (`HOOK_SELECTORS`), which name the payload's encoding. Decoding the payload is
  `marley_terminal`'s (`decode_frame`, `decode_hook`).
- **`DcsScanner::feed`** takes one read and returns `DcsEvent`s in stream order:
  `Passthrough(bytes)` runs and `Hook(RawDcs)` frames. State persists across reads, so a frame
  split over several reads is reassembled, and the bytes of an unfinished frame wait for the
  next read.
- **Bytes that are not Marley's pass through as they came.** The parser behind the scanner
  then sees them exactly as it would have without it:
  - another program's DCS, forwarded until the parser would end it (ESC, CAN or SUB);
  - a Marley frame cancelled by CAN or SUB, its bytes so far and the cancel byte;
  - a payload past `MAX_PAYLOAD` (64 KiB), after which the scanner still finds the next
    frame.
- An escape inside a Marley payload that is not `ESC \` stays in the payload, as the gpui era's
  scanner had it.

## Consumers

- The vendored `alacritty_terminal`'s event loop, through `marley_hooks::advance_with_hooks`:
  each passthrough run is parsed, and each hook is reported as `Event::ShellHook` with the grid
  position where it fell (#462).
- `marley_terminal`'s `TerminalSession::ingest`, the gpui era's own PTY engine.

## Tests

`src/marley_dcs.rs`, twelve unit tests, among them every split point of a read that carries
output, a frame and more output.
