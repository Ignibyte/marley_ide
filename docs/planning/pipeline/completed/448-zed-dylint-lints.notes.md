# Zed's dylint lints on the Marley crates, as gate:21 — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-448-zed-dylint-lints.md
- **Pipeline spec:** 448-zed-dylint-lints.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] feasibility run · [x] spec · [x] design · [x] present (autonomous).
- **Request:** TICKET-448, the last row of the Queue; autonomous per Chad's goal "lets mimic
  the quality gates on rustal … make sure high quality rust items are here and lets continue
  working on the remaining tickets".
- **Classification / tier:** chore, medium; the gate script, the Marley crate roots, the
  CONSTITUTION.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).**
  - #447's spec put the library Out for want of the tools; its notes queued this ticket.
  - CONSTITUTION §0: every verdict is a tool's exit code, never a grep of its output; Zed's
    crates keep Zed's bar.
  - Brain: consultation `c02d4cb109cb4c27aafee01c40c369c6`, nothing on this seam.
- **Discovery.**
  - The root toolchain is 1.98.1 and the library pins `nightly-2026-03-21` (1.96.0-nightly),
    with `clippy_utils` 0.1.96 to match. Nothing in the workspace sets `rust-version`.
  - Upstream runs the library in neither CI nor `script/clippy`.
  - The lints are declared at warn, by plain name, from a library named `lints`.
  - `blocking_io_on_foreground` looks only in functions that take a synchronous gpui context
    and in `render`.
  - Roots: seven libraries (`src/marley_*.rs`) and `marley_terminal/tests/integration.rs`.
- **Tools installed** (user-level): `rustup toolchain install nightly-2026-03-21 --profile
  minimal` with `llvm-tools-preview`, `rustc-dev` and `rust-src`; `cargo install cargo-dylint
  dylint-link --locked` (6.0.4 each), built in a scratch target directory.
- **Feasibility run.**
  - `cargo dylint --all -- -p marley_rail` built the library and its driver on the nightly,
    and checked `marley_agent` and `marley_rail` clean, exit 0. The check's target directory
    is `$CARGO_TARGET_DIR/dylint`.
  - Over every Marley crate with `--all-targets`: 691 crates checked on the nightly in 1m 52s,
    exit 0, no compile error, so D1 holds. Zed's crates carry about 600 hits: 373
    `SharedString` constructions from literals, 181 async blocks with no await, 26 map
    lookups, 20 blocking calls, 7 entity updates in render. D2 follows from that.
  - The Marley hits: twelve `shared_string_from_str_literal` in `marley_workbench`, one in
    `agents.rs:142` (the Zed Agent's name) and eleven in two test fixtures of a registry agent
    (`agents_tests.rs:37-45`, `rail_tests.rs:2029-2054`). The other six crates are clean.

### Design
- **The deny**, at the top of each root, after its `//!` docs:
  `#![cfg_attr(dylint_lib = "lints", deny(<the seven lints>))]`, with a one-line comment
  pointing at gate:21.
- **gate:21** (`dylint_g`): `need` checks for `cargo-dylint` and `dylint-link`, then
  `cargo dylint --all -- --all-targets "${MARLEY_PKG_ARGS[@]}"`. It is registered after
  gate:20 with the static gates.
- **Fixes:** each Marley hit, at the source, as the lint's own help says.
- **File manifest.**
  - Marley: the eight roots; any file with a hit.
  - Gate: `script/gates.sh`.
  - Docs: `CONSTITUTION.md` §0; at Complete, `CHANGELOG.md`.
  - No Zed path changes.

### Test plan
| REQ | Test |
|---|---|
| 001 | `cargo dylint --version`, `rustup toolchain list`, the §0 tools line |
| 002 | `script/gates.sh --fast` prints gate:21 over the Marley crates |
| 003 | negative smoke: a `cx.notify()` planted in a Marley `render` turns gate:21 red; restored by checksum |
| 004 | the gate:21 run: Zed crates' warnings, exit 0 |
| 005 | gate:21 green |
| 006 | `script/gates.sh --diff` |

### Risks
- **The nightly.** If a Zed crate the Marley crates depend on needs a feature newer than
  1.96, the check fails to compile and D1 does not hold; the fallback is a bump of the
  library's pins, a touchpoint. The feasibility run answers it.
- **Time.** The first check builds Zed's dependency tree on the nightly; later runs are
  incremental.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] the eight roots · [x] the twelve fixes ·
  [x] gate:21 · [x] CONSTITUTION §0 · [x] fmt, clippy and shellcheck · [x] review.
- **Built.**
  - The deny, in the seven library roots and `marley_terminal/tests/integration.rs`, with a
    comment pointing at gate:21.
  - The twelve hits now use `SharedString::new_static`: the Zed Agent's name in
    `agents.rs:142`, and the registry-agent fixtures in `agents_tests.rs` and `rail_tests.rs`.
  - gate:21 (`dylint_g`): the two `need` checks, then `cargo dylint --all -- --all-targets`
    over the Marley crates. It is registered after gate:20, and the header lists it.
  - CONSTITUTION §0: the table row, the known-scope bullet (what gate:21 covers, and that a
    new library lint warns until the lists name it), and the tools line.
- **Deviations.**
  - In the integration test root the deny sits above `#![cfg(unix)]`. Placed after
    `#![allow(clippy::expect_used)] // …`, it made rustfmt move that line's justification
    to a line of its own, and gate:12 reads the justification on the same line.
  - rustfmt joined the deny to `#![deny(missing_docs)]` in `marley_terminal.rs`.
- **Review.**
  - Cache: I drafted a §0 note telling the reader to clear `$CARGO_TARGET_DIR/dylint` after a
    library change, then checked dylint 6.0.4's driver. It adds each loaded library to the
    crate's dep-info and hashes the libraries' contents into rustc's tracked options
    (`dylint_driver/src/untracked_state.rs`), so a rebuilt library re-runs the check by
    itself. The note was wrong and was removed.
  - A normal build never sets `dylint_lib`, and the Marley manifests allow `unexpected_cfgs`,
    so clippy on the seven crates stays clean.
  - gate:12 reads `allow` and `expect` only; the attribute is a `deny`.
  - No Zed path changed; `tooling/lints` is used as upstream ships it.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 to REQ-005 · [x] negative smoke · [x] the fingerprint · [x] gate.
- **REQ-001.** `cargo dylint --version` prints 6.0.4, `dylint-link` is on the path, and
  `rustup toolchain list` shows `nightly-2026-03-21`; §0's tools line names all three.
- **REQ-002, REQ-005.** gate:21's command over the seven crates: each Marley crate re-checked
  (the roots changed), no Marley hit, exit 0 in 2.24s. In the first `--diff` run, gate:21
  passed within the gate.
- **REQ-003, the negative smoke.** A `cx.notify()` planted at the top of `RailSwitcher::render`:
  the check reports `error: cx.notify() called during render …` at `rail_switcher.rs:233`,
  with "the lint level is defined here" on the root's deny, and exits 1. Restored by sha256
  checksum. The smoke ran gate:21's command alone: in the full gate, a notify in render could
  loop the driven tests' renders.
- **REQ-004.** The green run replays 595 of the library's warnings from 70 of Zed's crates and
  exits 0.
- **The fingerprint.** gate:21 depends on `tooling/lints`, which the receipt did not bind. It
  is now in `gate_state_hash`'s pathspecs (24 files) and in §15's list of gate-defining files.
  That edit to `.claude/hooks` made the first run's receipt stale, so the gate ran again.
- **Gate:** the first `script/gates.sh --diff`: GATE GREEN [diff], 20 of 20, gate:21 passing,
  440 tests, coverage at 100% of lines (8762) and functions (1039).
- **Gate, again after the fingerprint:** `script/gates.sh --diff`: GATE GREEN [diff], 20 of 20,
  gate:21 passing, the same 440 tests and 100% coverage. The receipt matches the tree.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:** `CHANGELOG.md` (Added); `CONSTITUTION.md` §0 (the table row, the known-scope bullet,
  the tools line) and §15 (`tooling/lints` among the gate-defining files); the gate script's
  header. No Marley crate note changes: the deny is the same line in every root, and §0 is its
  record. No Zed path changed.
- **Knowledge:** `L-claude-448-running-zeds-dylint-library-on-the-fork-001`,
  `AD-claude-448-zeds-dylint-lints-are-errors-in-the-marley-crates-only-001`. No `F-` block:
  the drafted cache note was wrong but was caught in review, before it shipped.
- **Brain:** consultation `c02d4cb109cb4c27aafee01c40c369c6` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #448 closed. The Queue is empty; the Deliberate rows wait for Chad.
