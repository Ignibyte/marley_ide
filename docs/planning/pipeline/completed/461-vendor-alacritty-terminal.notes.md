# Zed's alacritty_terminal carried in the repo — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-461-vendor-alacritty-terminal.md
- **Pipeline spec:** 461-vendor-alacritty-terminal.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad: "ok lets go for it").
- **Request:** Chad, 2026-09-23: start T0, with the alacritty change kept in this repo: "i
  would keep it all in the same so we dont have to manage another library".
- **Split.** T0 became three tickets. #461 is the copy. #462 covers the hooks through the event
  loop, `Event::ShellHook` and the anchored `BlockList` on Zed's `Terminal`. #463 covers the
  shell scripts and their injection.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).**
  - `PR-claude-ordered-events-coalesced-stream-hooks-001`: hooks and passthrough bytes stay in
    stream order (#462's concern).
  - `PR-claude-real-pty-flake-under-mutants-baseline-not-code-001` and
    `BF-claude-gate-wedge-real-pty-test-under-llvm-cov-253`: real-PTY tests under
    instrumentation can wedge; nextest isolates them.
  - Brain: consultation `f83171b55e5b42c299db6b4f1aa0a879`, nothing on this seam.
- **Discovery** (two Explore sweeps).
  - Zed pins `alacritty_terminal` by git rev (`Cargo.toml:536`), and only `crates/terminal`
    and `crates/marley_terminal` use it.
  - Upstream's crate (`alacritty_terminal/` in the fork) is 23 source files and 12,001 lines,
    Apache-2.0, with `LICENSE-APACHE` a symlink to the repository root's.
  - `edition` and `rust-version` are inherited from alacritty's workspace (2024, 1.85.0), and
    Zed's `[workspace.package]` has no `rust-version`.
  - `tests/ref` is 46 MB of recordings. All the crate's `typos` hits are in them; `src/` is
    436 KB and clean.
  - `serde_json` is also used by a unit test in `src/term/mod.rs`, so the dev-dependency stays.
  - `vendor/` is not in the owned set, which matches exact rows, and the fork point has no
    `vendor/`.

### Design
- Copy from the cargo git checkout
  (`~/.cargo/git/checkouts/alacritty-20195d12a03fa0c5/4c12966`): `alacritty_terminal/Cargo.toml`,
  `CHANGELOG.md`, `src/`, and the root `LICENSE-APACHE`.
- In the manifest, `edition = "2024"` and `rust-version = "1.85.0"` replace the two
  `.workspace = true` lines.
- The root `Cargo.toml` gets `exclude = ["vendor"]` in `[workspace]` and a new
  `[patch."https://github.com/zed-industries/alacritty"]` table, each with a `# Marley:`
  comment.
- `lib-hook-helpers.sh`: `vendor/*` in `marley_owned_path`; `vendor` in both of
  `gate_state_hash`'s pathspecs and its comment.
- `vendor/README.md`: the base, the license, what is left out, Marley's hunks, and the
  re-sync steps.
- Ledger: the owned-set prose in `zed-touchpoints.md` and the `Cargo.toml` row.
- **File manifest.** New: `vendor/alacritty_terminal/**`, `vendor/README.md`. Changed: root
  `Cargo.toml`, `Cargo.lock` (regenerated), `.claude/hooks/lib-hook-helpers.sh`,
  `docs/marley/zed-touchpoints.md`, and `CONSTITUTION.md` §15 at Complete.

### Test plan
| REQ | Test |
|---|---|
| 001 | `cargo tree -i alacritty_terminal -p terminal` shows `(/srv/stacks/marley_ide/vendor/alacritty_terminal)` |
| 002 | `diff -r` of `src/` against the checkout is empty; the manifest diff is the two fields |
| 003 | `cargo nextest run -p terminal`; also the copy's own unit tests, `cargo test --manifest-path vendor/alacritty_terminal/Cargo.toml` |
| 004 | touch a vendored file: `gate_state_hash` changes; restore by checksum |
| 005 | `script/gates.sh --diff` |

### Risks
- **An exclude and a patch together.** If Cargo refuses a patch into an excluded directory,
  the fallback is membership with the fmt, shear and clippy consequences D1 avoids. The first
  `cargo metadata` answers it.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] the ledger first · [x] the copy · [x] the manifest hunk · [x] the patch and
  exclude · [x] the owned set and fingerprint · [x] `vendor/README.md`.
- **Built.**
  - The ledger's owned-set prose names `vendor/`. The `Cargo.toml` row describes the exclude
    and the patch, with the re-vendor step for upstream merges.
  - `vendor/alacritty_terminal/`: `Cargo.toml`, `CHANGELOG.md` and `src/` from the cargo git
    checkout of `4c12966`, plus the root's real `LICENSE-APACHE`.
  - The manifest spells out `edition = "2024"` and `rust-version = "1.85.0"`, with a
    `# Marley:` comment.
  - Root `Cargo.toml`: `exclude = ["vendor"]` and the
    `[patch."https://github.com/zed-industries/alacritty"]` table, each with a `# Marley:`
    comment.
  - `lib-hook-helpers.sh`: `vendor/*` in `marley_owned_path`, `vendor` in both fingerprint
    pathspecs, and the comment names it.
  - `vendor/README.md`, with concrete re-sync steps.
- **The design's risk, answered.** Cargo takes a patch into an excluded directory. `cargo
  tree` resolves `alacritty_terminal v0.26.1-dev (/srv/stacks/marley_ide/vendor/alacritty_terminal)`,
  and `Cargo.lock` loses only the entry's `source = "git+…"` line.
- **Deviations:** none.

## Phase 3 — Test (2026-09-23)
- **REQ-001:** `cargo tree -i alacritty_terminal -p terminal` shows the vendored path.
  `cargo metadata` lists no `alacritty_terminal` among the workspace members.
- **REQ-002:** `diff -r` of `src/` against the checkout is empty. The manifest differs only in
  the two fields and their comment, and `LICENSE-APACHE` compares equal.
- **REQ-003:** `cargo nextest run -p terminal -p marley_terminal`: 262 passed. The copy's own
  tests, `cargo test --offline --manifest-path vendor/alacritty_terminal/Cargo.toml`: 138 unit
  tests and 1 doctest passed. That run wrote a `Cargo.lock` into the copy, which was removed. A
  note on #462 says a gate step running those tests must commit that lockfile.
- **REQ-004, the negative smoke:** a byte appended to `vendor/alacritty_terminal/src/sync.rs`
  changed `gate_state_hash`; restoring it by checksum restored the hash.
- **The gates `vendor/` meets.** `typos` over `vendor/` is clean, since `tests/ref` stayed
  upstream. fmt, clippy, cargo-shear and dylint see workspace members only. gates 12, 13 and 17
  scan `crates/` only.
- **Gate:** `script/gates.sh --diff`: GATE GREEN [diff], 20 of 20, with 441 tests over the
  Marley crates. Coverage was skip-clean, since no Marley crate changed. The receipt matches.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:**
  - `CHANGELOG.md` (Changed).
  - `CONSTITUTION.md`: §15's fingerprint list, and §0's known-scope bullet for `vendor/`.
  - `docs/marley/three-prong-plan.md`: D1's copy, the T0 row's split, the risk, and open
    decision 1 marked decided.
  - `docs/marley/README.md` (the docs map), `vendor/README.md`, and the ledger rows.
- **Knowledge:** `AD-claude-461-upstream-crates-marley-changes-live-in-vendor-001`,
  `L-claude-461-carrying-an-upstream-crate-in-vendor-001`.
- **Brain:** consultation `f83171b55e5b42c299db6b4f1aa0a879` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #461 closed. #462 and #463 are queued, with rows in `BACKLOG.md`.
