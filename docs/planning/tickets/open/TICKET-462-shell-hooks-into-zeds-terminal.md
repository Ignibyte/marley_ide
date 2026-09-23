# TICKET-462 — Shell hooks into Zed's terminal, and its blocks

- **Ticket:** LOCAL #462 (feature, prong 1: T0b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from T0 at #461's plan)
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D1, D2)
- **Status:** open

## Summary
Marley's DCS scanner moves out of `marley_terminal` into a leaf crate, since `marley_terminal`
depends on `alacritty_terminal`. The vendored event loop (#461) scans each read with it,
parses the passthrough bytes in stream order, and at each complete hook snapshots the grid
position under the lock and emits `Event::ShellHook`. A monotonic evicted-lines counter on
the grid needs `#[serde(default)]`. Zed's `TerminalBackendEvent` mirrors the event, and
Zed's `Terminal` keeps a `BlockList` anchored by absolute line (D2), with a `blocks()`
accessor.

Running the copy's own tests standalone (`cargo test --manifest-path
vendor/alacritty_terminal/Cargo.toml`) writes a `Cargo.lock` into the copy. A gate step that
runs them must commit that lockfile and use `--locked`, or the untracked lockfile changes
the receipt fingerprint mid-run (#461).

## Acceptance
In a PTY-backed test, a shell that prints Marley's hook frames around `echo hi` leaves a
Finished block with exit 0 whose output is "hi", also when one read carries the whole command.
