---
pipeline_id: ee7fb027-e6b8-4975-97dd-c2a7411c5465
ticket: docs/planning/tickets/closed/TICKET-472-rust-toolchain-current.md
status: Phase 4 — Complete PASS
title: Rust at the latest stable
type: chore
slice: the toolchain
references: [rust-toolchain.toml, CONSTITUTION.md]
---

## Title
Chad asked whether Rust should move to the latest version. Marley's pin already is the latest
stable; the box's default `stable` was one patch behind. It is updated, and the practice for
the next stable is written down.

## Scope
### In
- `rustup update stable` on the dev box, while no cargo runs: 1.98.0 to 1.98.1.
- CONSTITUTION: the toolchain paragraph, beside the gate's tools.

### Out (explicitly deferred)
- `tooling/lints`' nightly: Zed's pin, tied to that nightly's `rustc-dev`.
- The rustup binary: the distro manages it (self-update is disabled).

## Reference (§20)
- N/A — Marley-specific: the toolchain policy of this fork; upstream Zed pins its own, and the
  fork follows the latest stable at or after it.

### Prior art
- **Behavior maps:** none.
- **Published material:** rustup's `check` and `update`; Rust's six-week stable cadence.
- **Code we already ship:** `rust-toolchain.toml` (1.98.1, the minimal profile with rustfmt,
  clippy, rust-analyzer and rust-src, and three targets).

## UI proof
N/A — no UI delta: a toolchain on the box and a paragraph of policy.

## Locked-In Decisions
- D1 — The pin follows the latest stable, moved by a chore with a full gate run.
- D2 — The default `stable` moves with it, and only while no cargo runs.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `rustup check` shall report every stable toolchain on the box up to date | `rustup check` |
| REQ-002 | Marley's pin shall be the latest stable | `rust-toolchain.toml` against `rustup check` |
| REQ-003 | The practice for the next stable shall be written beside the gate's tools | CONSTITUTION |

## Phase Plan
- **P1 Plan** — this spec. **P2 Code** — the update and the paragraph. **P3 Test** — the checks
  above. **P4 Complete** — close, archive, commit.
