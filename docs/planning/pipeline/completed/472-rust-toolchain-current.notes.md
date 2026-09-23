# Rust at the latest stable — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-472-rust-toolchain-current.md
- **Pipeline spec:** 472-rust-toolchain-current.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request (Chad, verbatim):** "Also we should try to upgrade rust to the latest version?"
- **Found:** `rust-toolchain.toml` pins 1.98.1; `rustup check` reported the default `stable` as
  1.98.0 with 1.98.1 (2026-09-01) available, so the pin is the latest stable. Installed beside
  them: 1.94.0, 1.96.0, 1.98.0 (other projects' pins, left alone) and `nightly-2026-03-21`
  (`tooling/lints`).
- Brain: not consulted; a patch update and a written practice, no design choice.

## Phase 2 — Code (2026-09-23)
- `rustup update stable` with no cargo or rustc running: `stable-x86_64-unknown-linux-gnu
  updated - rustc 1.98.1 (48a229cea 2026-09-01) (from rustc 1.98.0 (88d9e12ae 2026-08-18))`.
- CONSTITUTION: the toolchain paragraph after the gate's tools.

## Phase 3 — Test (2026-09-23)
- `rustup check`: `stable-x86_64-unknown-linux-gnu - up to date: 1.98.1`.
- `rustup run stable rustc --version` and the pinned `rustc --version` both 1.98.1.
- No Rust source changed, so no gate receipt is needed for the commit.

## Phase 4 — Complete (2026-09-23)
- Docs: CONSTITUTION. No ledger entry: the practice lives in the constitution.
- Ticket closed; archived; one commit.
