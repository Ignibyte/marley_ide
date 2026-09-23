---
pipeline_id: d101c090-6e10-4cc4-b51c-ab8f1ac5f977
ticket: docs/planning/tickets/closed/TICKET-448-zed-dylint-lints.md
status: Phase 4 — Complete PASS
title: Zed's dylint lints on the Marley crates, as gate:21
type: chore
slice: quality gates (the Out list of #447)
references: [docs/planning/pipeline/completed/447-rustal-quality-gates.spec.md, tooling/lints/README.md, CONSTITUTION.md]
---

## Title
Zed keeps a dylint library in `tooling/lints` whose lints catch gpui mistakes clippy cannot
see: state changed while a view renders, blocking IO on the foreground, an async block with no
await, and three string and map misuses. #447 left them out because the tools were not
installed. Marley now runs them over its own crates as gate:21, and a hit in a Marley crate
fails the gate.

## Scope
### In
- **The tools.** `cargo-dylint` and `dylint-link` 6.0.4, and the nightly the library pins
  (`nightly-2026-03-21`, with `rustc-dev`, `rust-src` and `llvm-tools-preview`), installed and
  named in CONSTITUTION §0's tools line.
- **The deny.** Each Marley crate root, and `marley_terminal`'s integration test root, turns
  the library's seven lints into errors while the dylint driver compiles it
  (`#![cfg_attr(dylint_lib = "lints", deny(...))]`). A normal build never sets the cfg, so it
  sees nothing.
- **The gate.** gate:21 in `script/gates.sh`, a static gate: `cargo dylint --all --
  --all-targets` over every Marley crate, in every mode. Its verdict is cargo's exit code.
- **The hits.** Every hit in the Marley crates is fixed at the source.
- **The docs.** CONSTITUTION §0 (the gate table, the tools line, the known-scope bullet), the
  gate script's header.

### Out (explicitly deferred)
- Zed's own crates: they keep the library's warn level, Zed's bar (§0).
- The library itself: its lints, its nightly and its `clippy_utils` pin stay as upstream keeps
  them.
- Zed's `single-lint` helper and CI.

## Reference (§20)
N/A — gate tooling, with no Warp or Zed behavior to match. The reference is Zed's own
`tooling/lints/README.md`: how the library is installed, discovered through the root
`[workspace.metadata.dylint]` (`Cargo.toml:1179-1184`) and run with `cargo dylint --all`.

### Prior art
- **Behavior maps:** none apply; this is tooling.
- **Published material:** dylint's documentation: `cargo dylint --all -- <cargo check
  args>`, and the `dylint_lib` cfg the driver sets for each library it loads, meant for
  `cfg_attr` lint levels in source.
- **Code we already ship.**
  - The library (`tooling/lints/src/lib.rs` registers seven lints, all at warn).
  - The root `[workspace.metadata.dylint]` entry.
  - The lints package's own `check-cfg` for `dylint_lib` (`tooling/lints/Cargo.toml:27-29`).
  - The Marley manifests already allow `unexpected_cfgs`, so the cfg costs nothing in a normal
    build.
  - Upstream runs the library in neither CI nor `script/clippy`, so Zed's crates may carry
    hits.

## UI proof
N/A — no UI delta: a gate and whatever source fixes its hits need, which keep behavior (the
crate suites and gate:4 prove it).

## Locked-In Decisions
- D1 — The gate runs Zed's library as upstream ships it, at its pinned nightly, through
  `cargo dylint --all`, the README's command. No touchpoint in `tooling/lints`.
- D2 — The Marley crates deny the lints in source, under the driver's `dylint_lib` cfg, and
  Zed's crates keep them at warn. The gate never reads the tool's output (§0). A
  `DYLINT_RUSTFLAGS="-D …"` would fail on hits in Zed's crates, which are held to Zed's bar.
- D3 — gate:21 is static, over every Marley crate with all targets, in every mode: a lint
  pass, and incremental after the first build.
- D4 — The deny lists name the library's seven lints. A lint the library adds warns until it
  is added to the lists; the gate's comment says so.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The dylint tools and the pinned nightly shall be installed and named in CONSTITUTION §0 | `cargo dylint --version`; `rustup toolchain list`; the doc |
| REQ-002 | `script/gates.sh` shall run gate:21 over every Marley crate in each mode, with cargo's exit code as its verdict | the gate's output |
| REQ-003 | WHEN Marley code has a hit of one of the library's lints, gate:21 shall fail | negative smoke: a `cx.notify()` planted in a Marley `render` |
| REQ-004 | WHEN only a Zed crate has hits, gate:21 shall pass | the run's warnings in Zed crates, with exit 0 |
| REQ-005 | The Marley crates shall have no hit | gate:21 green |
| REQ-006 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, the design and a feasibility run in the notes.
- **P2 Code** — the deny attributes, the fixes for any hit, gate:21, the docs; fmt and clippy
  clean.
- **P3 Test** — gate:21 green, the negative smoke red, `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG (§21), ledger capture (§19), close the ticket, archive, commit.
