# TICKET-472 — Rust at the latest stable

- **Ticket:** LOCAL #472 (chore, the toolchain)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-09-23: "Also we should try to upgrade rust to the latest version?"
- **Status:** open

## Summary
Marley already pins the latest stable: `rust-toolchain.toml` names 1.98.1, which `rustup check`
reports as the current stable (2026-09-01). The box's default `stable`, which other projects
build with, is still 1.98.0. The dylint lints pin Zed's `nightly-2026-03-21` in
`tooling/lints`, which their `rustc-dev` API ties them to; that pin stays Zed's. This ticket
updates the default `stable` to 1.98.1 while no cargo runs, and records the practice of moving
Marley's pin at each new stable with a full gate run: 1.99 is due in early October.

## Acceptance
`rustup check` reports every stable toolchain up to date; Marley's pin is the latest stable;
the practice is written where the gate's tools are listed.
