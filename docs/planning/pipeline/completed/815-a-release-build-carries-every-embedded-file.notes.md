# A release build carries every file its embedded folders hold — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-815-a-release-build-carries-every-embedded-file.md
- **Pipeline spec:** 815-a-release-build-carries-every-embedded-file.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-18, after rebooting onto the `ba5aacce78` install: "the icons for
  home, marley, and rusty did not appear after reboot".
- **Classification / tier:** bug; four Zed crates and one Marley crate each gain a build script;
  no application code changes.
- **Diagnosis (verified):**
  - The installed `~/.local/lib/marley/marley` (2026-10-18 19:28) holds `icons/list_tree.svg` and
    `icons/sparkle.svg` but none of the three new paths; `marley_home` and `rusty_butler` appear
    once each (the `IconName` strings), so the code that asks for them is in.
  - `/mnt/fast/target/release/deps/libassets-d245c948a6f9c7a2.rlib` dates from 2026-10-17 19:07;
    `assets/icons/marley_cat.svg` from 21:19.
  - `rust-embed-impl` 8.11.0 under `compression` emits `flate!(... from "<path>")`;
    `include-flate-codegen` 0.3.4 turns it into a byte literal. No `include_bytes!`, so the
    crate's dep-info names none of the files.
- **Recall (§18.3):**
  - `F`/`L` #261 (failures.md, lessons.md): an icon wired through `icon_path` but missing from the
    asset source draws as a silent blank, with every gate green. Same symptom, different cause (a
    static match then; a stale embed now).
  - #681 notes: "a release build embeds them" (the docs bundle); nobody checked that a release
    build sees a new or edited doc.
  - The completed archive and the prevention rules hold nothing on build scripts or rust-embed
    tracking. The brain (`rusty-cli brain ask`, consultation 8a72f55d8cf347c38ac1bd4ced6f61ce)
    returned nothing on this seam.
- **Discovery:** `util::fs_embed!` callers in Marley's binary: `crates/assets/src/assets.rs`,
  `crates/settings/src/settings.rs:138`, `crates/agent/src/templates.rs:9`,
  `crates/grammars/src/grammars.rs:8`, `crates/marley_workbench/src/docs_tools.rs:23,31`. None
  of the five crates has a `build.rs`; none sets `build =` in its manifest.
- **Decisions:** D1 to D4 in the spec.

### Design
- Approach: each crate gets a `build.rs` of this shape, its comment saying why, its paths taken
  from its own `fs_embed!`:
  ```rust
  fn main() {
      println!("cargo:rerun-if-changed=build.rs");
      if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
          println!("cargo:rerun-if-changed=../../assets");
      }
  }
  ```
  Without any `rerun-if-changed`, cargo would rerun the script when any file of the package
  changes. Naming `build.rs` keeps a debug build's script from rerunning at all.
- File manifest:
  - `crates/assets/build.rs` (Zed crate, new): `../../assets`.
  - `crates/settings/build.rs` (Zed crate, new): `../../assets/settings`, `../../assets/keymaps`.
  - `crates/agent/build.rs` (Zed crate, new): `src/templates`.
  - `crates/grammars/build.rs` (Zed crate, new): `src`.
  - `crates/marley_workbench/build.rs` (Marley crate, new): `../../docs/src`,
    `../../docs/marley/guide.md`, `../../docs/marley/walkthrough.md`, `../../CHANGELOG.md`.
  - `script/e2e/815-release-build-carries-embedded-files.sh` (new): #807's set-up and first two
    steps, run on the installed binary.
  - Phase 4: `docs/marley/zed-touchpoints.md` (four rows), `CHANGELOG.md`, the ledgers.

### Visual check plan
- REQ-001: after `just install`, `grep -ao` the three icon paths in the installed binary; each
  should appear once.
- REQ-002: the scenario under `compositor sway`, `binary "$HOME/.local/lib/marley/marley"`:
  `815-01-title-bar` (title bar's butler and cat, status bar's house) and `815-02-home` (Home's
  tab and rail row with the house).
- REQ-003: `cargo build --release -p assets` (a baseline for that feature set), `touch
  assets/icons/marley_cat.svg`, `cargo build --release -p assets -v` (expect the script to run
  and `Compiling assets`), then again (expect `Fresh assets`).
- REQ-004: `cargo build -p assets`, `touch` the same file, `cargo build -p assets -v` (expect
  `Fresh assets`).
- REQ-005: the review of the diff, script by script against each `fs_embed!`.
- Not reached by a scenario: `settings`, `agent`, `grammars` and `marley_workbench` pick up an
  edit in release. Their scripts are the same mechanism REQ-003 proves on `assets`.

### Risks
- The first release build after this change recompiles all five crates and everything after
  them: one full release rebuild, about the cost of an upstream merge install.
- From then on, an edit to the default settings or a keymap rebuilds `settings` and its
  dependents in release, which nearly every Marley ticket's code change rebuilds anyway.
- The install replaces the binary by a rename; Chad's running Marley keeps its file until he
  restarts it.

## Phase 2 — Code
- Built to the manifest: `crates/{assets,settings,agent,grammars}/build.rs` (Zed crates, each
  opening with a `// Marley:` comment; the last three point to `assets`'s for the reason) and
  `crates/marley_workbench/build.rs` (Marley crate). Each prints `rerun-if-changed=build.rs`,
  then, when `CARGO_CFG_DEBUG_ASSERTIONS` is unset, its folders. Four `zed-touchpoints.md` rows
  went in first. No manifest changes.
- Deviation: `marley_workbench` denies `clippy::print_stdout` and `missing_docs`, and a crate's
  lint table covers its build script, so that script writes through `writeln!` on a locked
  stdout, returns `std::io::Result<()>`, and carries a `//!` doc. The Zed scripts use `println!`
  as Zed's own build scripts do.
- `script/e2e/815-release-build-carries-embedded-files.sh` sources #807's scenario for its
  set-up, names the installed binary, and takes three shots (title bar, Home, the New Agent
  picker); the spec's UI proof gained the picker shot. REQ-005 was reworded to allow D3's wider
  `assets` watch.
- Review against the criteria: each watched path checked against its `fs_embed!` (`assets`:
  `../../assets`, a superset per D3; `settings`: `assets/settings`, `assets/keymaps`; `agent`:
  `src/templates`; `grammars`: `src`; `marley_workbench`: `docs/src`, `docs/marley/guide.md`,
  `docs/marley/walkthrough.md`, `CHANGELOG.md`). Nothing panics; no entity code; nothing from
  Warp or from a Zed function body.
- The gate: `just gate-diff` green, 17 of 17 (scope: the five crates, every Marley crate). The
  first run's 16 gates passed and its receipt failed because the spec and the scenario were
  edited mid-run; the second run, on the settled tree, passed whole. Clippy compiled all five
  build scripts. cargo-audit printed registry timeouts on its yank checks and passed
  (pre-existing, network).

## Phase 3 — Test
- `just install` (2026-10-19 09:13 to 09:27): the release build 8m 24s, the remote server 5m 19s,
  installed at `ba5aacce78` with this change uncommitted. Chad's Marley (pid 7745) kept its file.
- REQ-001: `grep -ao` in the installed `~/.local/lib/marley/marley` finds `icons/marley_cat.svg`,
  `icons/marley_home.svg` and `icons/rusty_butler.svg` once each (none before).
- REQ-002: `just e2e script/e2e/815-release-build-carries-embedded-files.sh` on the installed
  binary, headless sway, `SHOT_DIR` in the scratchpad. Focus report: "1 Marley windows before the
  run, 1 after; the run added no rule". Each shot read:
  - `815-01-title-bar`: the title bar's right end reads `?`, the butler (head, hair, bow-tied
    body), the cat (ears, whiskers), `Sign In`; the status bar draws the house beside `Z»`
    (cropped at 4× to read them).
  - `815-02-home`: the Home tab and the Home row at the top of the rail's `repo` group each draw
    the house.
  - `815-03-picker`: the New Agent picker lists Marley with the cat and Rusty with the butler, both
    `Thread`; Zed Agent, Claude Agent and the terminal agents keep their own icons.
- REQ-003: `cargo build --release -p zed --bin marley -v` (the install's own command, so the
  install's units) with nothing changed: `Fresh` for `settings`, `assets`, `agent`, `grammars`,
  `marley_workbench` and `zed`, 0.65s. After `touch assets/icons/marley_cat.svg`: `Dirty assets
  … the file crates/assets/../../assets has changed`, its build script ran, `assets` compiled, and
  `prompt_store`, `agent` and on to `zed` followed (7m 11s).
- REQ-004: the dev profile's stored watch lists
  (`debug/.fingerprint/{assets,marley_workbench}-*/run-build-script-build-script-build.json`,
  written by the gate's clippy) hold only `build.rs`, so an asset edit cannot rerun a debug script.
- REQ-005: the release watch lists cargo stored after the install: `settings` `build.rs`,
  `../../assets/settings`, `../../assets/keymaps`; `agent` `build.rs`, `src/templates`; `assets`
  `build.rs`, `../../assets`; `grammars` `build.rs`, `src`; `marley_workbench` `build.rs`,
  `../../docs/src`, `../../docs/marley/guide.md`, `../../docs/marley/walkthrough.md`,
  `../../CHANGELOG.md`.
- Deviation from the plan: REQ-003 ran the install's `-p zed --bin marley` and REQ-004 read the
  stored watch list, not `cargo build [--release] -p assets`. Building `assets` alone unifies a
  smaller feature set, which can compile a second `gpui` and its tree in each profile on a
  disk at 81%.
- No fix was needed; no gate rerun.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #815); `docs/marley_architecture/marley_workbench.md` (the docs
  bundle's `build.rs`, under #681/#723's note); `docs/marley/zed-touchpoints.md`'s four rows read
  against what shipped. No slice of `three-prong-plan.md` owns the install tooling, so no status
  line moved.
- Ledger appends: `F-claude-815-a-release-build-reused-a-stale-asset-embed-001`,
  `PR-claude-815-every-fs-embed-folder-is-watched-001`,
  `L-claude-815-a-debug-check-cannot-see-what-a-release-build-embeds-001`.
- Brain: consultation `8a72f55d8cf347c38ac1bd4ced6f61ce` closed with `brain decide`,
  `decisions/marleys-embedded-folders-reach-a-release-build-through-build-scripts-that-watch-them`
  (follow-up 2026-11-19).
- Ticket closed and moved to `tickets/closed/`; it never had a `BACKLOG.md` row (minted at Plan).
- Chad's open question at close: which queued fixes (#812, #814, #809, #806, #794, #795, or two
  new ones from 2026-10-18: a full-width agent column by default, the width button on a lone
  thread) join the next install. He was told `agent.limit_content_width: false` widens the thread;
  two `settings_change` proposals timed out unanswered.
