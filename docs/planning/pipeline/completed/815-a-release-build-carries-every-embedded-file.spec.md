---
pipeline_id: 8c144f95-2ef2-4460-be6d-30d1d694b4fd
ticket: docs/planning/tickets/closed/TICKET-815-a-release-build-carries-every-embedded-file.md
status: Phase 4 — Complete PASS
title: A release build carries every file its embedded folders hold
type: bug
slice: the release install (#502), with #807's icons
references: []
---

## Title
A release build carries every file under the folders `util::fs_embed!` embeds. Today cargo is
never told about those files, so a new or edited asset, such as #807's three icons, stays out of
an install until its crate compiles for some other reason.

## Scope
### In
- A build script in each crate of Marley's binary that embeds a folder with `util::fs_embed!`:
  `assets`, `settings`, `agent`, `grammars` (Zed crates) and `marley_workbench` (Marley crate).
  In a build without `debug_assertions` it tells cargo to rerun when any path that crate's
  `fs_embed!` reads changes; in a debug build it names only itself.
- A release install, so Chad's Marley carries the icons.

### Out (explicitly deferred)
- `edit_prediction_cli`'s two `fs_embed!` folders: a separate command, not in Marley's binary.
- A debug build with `util/debug-embed` (only `remote_server`'s optional feature): such a build
  embeds the files but keeps a debug build's watch list.
- An upstream Zed pull request.

## Reference (§20)
Upstream Zed: the `livekit_api` and `extension_host` build scripts already watch a folder with
`cargo:rerun-if-changed=<dir>` (`vendored/protocol`, `../extension_api/wit`); this change uses
the same mechanism for the embedded folders. Upstream ships from clean CI builds, where a stale
crate never arises, so it has no fix of its own.

### Prior art
- **The code we ship.** `rust-embed-impl` 8.11.0 (`src/lib.rs`, `embed_file`): under
  `compression` each file becomes `flate!(static BYTES: [u8] from "<path>")`, and
  `include-flate-codegen` 0.3.4 expands that to a byte literal with no `include_bytes!`, so cargo
  tracks none of the files. `util::fs_embed!` (`crates/util/src/util.rs`) picks the derive
  only without `debug_assertions`. Zed's build scripts above show the folder watch.
- **Published material.** Cargo's build-script reference: `rerun-if-changed` on a directory
  scans everything under it for a newer modification time; when a build script reruns, its
  package compiles again and so does what depends on it.
- **Measured.** A scratch workspace (a crate with the build script below, and a binary that
  depends on it), 2026-10-18: in release, a new file and an edit in the watched folder each reran
  the script and compiled both crates; in debug both stayed `Fresh`.
- None of `gpui`, `util` or `rust-embed` offers a stable-Rust way for a derive to watch a folder.

## UI proof
`script/e2e/815-release-build-carries-embedded-files.sh` runs the **installed release build**
(`binary ~/.local/lib/marley/marley`), since a debug build reads `assets/` from disk and would
show the icons without the fix. Under sway, with #807's set-up (the Marley entry on a scripted
agent, Rusty on through its stand-in):
- `815-01-title-bar`: the title bar's Rusty and Marley buttons draw the butler and the cat, and
  the status bar's Home button the house (REQ-002).
- `815-02-home`: after `rusty: open home`, Home's tab and rail row draw the house (REQ-002).
- `815-03-picker`: the New Agent picker's Marley and Rusty entries draw the cat and the butler
  (REQ-002).

## Locked-In Decisions
- D1 — Build scripts with cargo's folder watch, not a `touch` in `script/install-marley`: every
  release build is covered (the install, the remote server, a hand-run `cargo build --release`),
  and a crate rebuilds only when one of its files changed.
- D2 — The watch is printed only when `CARGO_CFG_DEBUG_ASSERTIONS` is unset, the same condition
  `fs_embed!` embeds under. A debug build reads the checkout at runtime, and a watch there would
  recompile `assets`, then `editor` and everything after it, on every icon or theme edit.
- D3 — Each script watches exactly what its `fs_embed!` reads: `assets` the whole `assets/`
  folder (its `*.md` glob reaches any depth; the folders it leaves out, `settings/` and
  `keymaps/`, already rebuild `settings`, which nearly every crate follows); `settings`
  `assets/settings` and `assets/keymaps`; `marley_workbench` `docs/src`, `docs/marley/guide.md`,
  `docs/marley/walkthrough.md` and `CHANGELOG.md`, not all of `docs/`, whose planning files
  change with every ticket; `agent` `src/templates`; `grammars` `src`.
- D4 — No `Cargo.toml` changes: cargo finds a `build.rs` at a package's root.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley is installed from this change, the installed binary shall embed `icons/marley_cat.svg`, `icons/marley_home.svg` and `icons/rusty_butler.svg`. | `grep -ao` of the three paths in `~/.local/lib/marley/marley` after `just install` |
| REQ-002 | WHEN the installed release Marley starts, the title bar's Rusty and Marley buttons, the status bar's Home button, and Home's tab and rail row shall draw Chad's butler, cat and house. | Shots `815-01-title-bar`, `815-02-home`, `815-03-picker` |
| REQ-003 | WHEN a file under `assets/icons` changes, the next release build of `assets` shall rerun its build script and compile the crate, and a build with nothing changed shall leave it `Fresh`. | `cargo build --release -p zed --bin marley -v` (the install's command) before and after a `touch`, output read |
| REQ-004 | WHEN a file under `assets/icons` changes, the next debug build of `assets` shall leave the crate `Fresh`. | The dev profile's stored build-script watch list for `assets` (only `build.rs`) |
| REQ-005 | Each build script shall watch every path its crate's `fs_embed!` reads, and nothing beyond them except as D3 allows for `assets`. | The review of the diff against each `fs_embed!` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — five `build.rs` files; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — `just install`, the binary grep (REQ-001), the scenario on the installed build
  with every shot read (REQ-002), a release build before and after a `touch` (REQ-003), the
  stored debug watch list (REQ-004).
- **P4 Complete** — CHANGELOG, `zed-touchpoints.md` rows for the four Zed build scripts, the
  ledger (a failure and a prevention rule), close the ticket, archive, commit.
