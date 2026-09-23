---
pipeline_id: 8182446f-1739-4e7e-b778-2e9b1d5bdf9e
ticket: docs/planning/tickets/closed/TICKET-461-vendor-alacritty-terminal.md
status: Phase 4 — Complete PASS
title: Zed's alacritty_terminal carried in the repo
type: chore
slice: prong 1, T0a (split from T0)
references: [docs/marley/three-prong-plan.md, docs/marley/zed-touchpoints.md, CONSTITUTION.md]
---

## Title
The block terminal needs a change inside alacritty's event loop (three-prong plan, D1). Chad
chose to keep that code in this repository rather than in a fork of its own. This slice copies
the `alacritty_terminal` that Zed pins into `vendor/`, unchanged, and makes the build and the
gates use and cover it. #462 then makes the change.

## Scope
### In
- **The copy.** `vendor/alacritty_terminal/`: `Cargo.toml`, `CHANGELOG.md` and `src/` from
  `zed-industries/alacritty` at `4c129667ce56611becdc82de6e28218c80e2e88f`, directory
  `alacritty_terminal/`, plus the repository's real `LICENSE-APACHE`, where upstream keeps a
  symlink.
- **The one manifest hunk.** `edition` and `rust-version` spelled out (`2024`, `1.85.0`,
  alacritty's workspace values), since the copy has no alacritty workspace to inherit them from.
- **The build.** `[patch."https://github.com/zed-industries/alacritty"]` points
  `alacritty_terminal` at the copy, and `[workspace] exclude` keeps `vendor/` out of Zed's
  workspace.
- **The bookkeeping.**
  - `vendor/*` joins the Marley-owned set (`marley_owned_path`) and the ledger's list of
    owned paths.
  - `vendor/` joins the receipt fingerprint (`gate_state_hash`, CONSTITUTION §15).
  - `vendor/README.md` records what is carried, from where, what is left out, Marley's
    hunks, and how to re-sync when Zed moves its pin.
  - The `Cargo.toml` ledger row describes the patch and the exclude.

### Out (explicitly deferred)
- Any change to alacritty's behavior: #462.
- `tests/ref`, alacritty's 46 MB of recorded sessions. They stay upstream, and every spelling
  hit the crate has is in them.
- vte: D1 scans the bytes before the parser, so vte stays as it is.

## Reference (§20)
N/A — build and repository tooling with no behavior to match. The design is the plan's D1,
with Chad's choice of an in-repo copy over a fork of its own.

### Prior art
- **Behavior maps:** none apply.
- **Published material:** Cargo's `[patch]` for git sources (a URL-keyed table) and
  `[workspace] exclude`.
- **Code we already ship.**
  - The root `[patch.crates-io]` already carries a local path patch,
    `scratch = { path = "tooling/corgi/patches/scratch" }` (`Cargo.toml:1039`).
  - `deny.toml` license-checks non-private path crates, and Apache-2.0 is allowed.
  - `marley_owned_path` and `gate_state_hash` in `.claude/hooks/lib-hook-helpers.sh` define
    ownership and the fingerprint.

## UI proof
N/A — no UI delta: the copy is byte-identical in `src/`, and Zed's terminal tests prove it
behaves the same.

## Locked-In Decisions
- D1 — The copy lives out of Zed's workspace. As a member, `cargo fmt --all`, clippy,
  cargo-shear and dylint would hold alacritty's upstream code to Zed's and Marley's bars. It
  builds only as the patched dependency.
- D2 — `src/` is byte-identical to upstream in this slice; the only hunk is the manifest's two
  fields. Every later Marley hunk is marked `// Marley:` and listed in `vendor/README.md`.
- D3 — `vendor/` is Marley-owned for the ledger. Its provenance is recorded in its README, not
  one ledger row per file.
- D4 — Re-sync by copying the new upstream `alacritty_terminal/` over the old and re-applying
  the hunks the README lists. No script until a second re-sync shows the steps.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The build shall take `alacritty_terminal` from `vendor/alacritty_terminal` | `cargo tree -i alacritty_terminal` shows the path source |
| REQ-002 | The vendored `src/` shall match upstream's byte for byte, and the manifest shall differ only in `edition` and `rust-version` | `diff -r` against the cargo git checkout |
| REQ-003 | Zed's terminal shall behave as before | `cargo nextest run -p terminal` green |
| REQ-004 | A change under `vendor/` shall invalidate the gate receipt | `gate_state_hash` changes when a vendored file changes (negative smoke) |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the copy, the manifest hunk, the patch and exclude, the owned set, the
  fingerprint, the README, the ledger rows.
- **P3 Test** — REQ-001 to REQ-004, and the gate.
- **P4 Complete** — CHANGELOG and docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
