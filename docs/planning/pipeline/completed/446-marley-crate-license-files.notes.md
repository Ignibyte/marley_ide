# License files for the Marley crates — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-446-marley-crate-license-files.md
- **Pipeline spec:** 446-marley-crate-license-files.spec.md

## Phase 1 — Plan
- **Request:** TICKET-446, waiting since #438 for the copyright line of the MIT text; Chad gave
  the holder on 2026-09-28: "446 - ignibyte".
- **Classification / tier:** chore, S. Marley crates' folders (ten symlinks, ten files), and two
  Zed paths: `script/check-licenses` (a new ledger row) and `README.md` (its row extended).
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo
  busy with #587's release install, so no cargo ran and no crate was touched); recall ✓; mint ✓;
  discovery (the files named below, read directly: five known paths, no search) ✓; the prior-art
  sweep ✓; spec and design ✓.
- **Recall (§18.3):**
  - AD-claude-438-marley-crates-may-link-zeds-gpl-crates-001: the Marley crates keep `MIT OR
    Apache-2.0` though `marley_workbench` links Zed's GPL crates, and ship only as part of the
    fork; "The crates' license files wait on TICKET-446."
  - #438's inspect ledger, U5: "No license files; `script/check-licenses` fails on every Marley
    crate."
  - CONSTITUTION §20: IP counsel's sign-off stays pending before commercializing
    (`docs/decisions/licensing-ownership-strategy.md`).
  - The brain (consultation 6a8b91e08a184237880824efc066c193): nothing on this seam.
- **Discovery (2026-09-28):**
  - `script/check-licenses`: `check_no_agpl_license_file`; per `Cargo.toml` from `git ls-files`,
    `check_manifest_for_agpl` (no AGPL, no `license-file`) and, for every one but the root's,
    `check_license <dir>`: a `LICENSE-GPL` or `LICENSE-APACHE` symlink whose target is
    `../`×depth plus the name, and an error for a regular file of either name. It exits at the
    first error.
  - Run today: `Error: crates/marley_agent does not contain a LICENSE-GPL or LICENSE-APACHE
    symlink`. Every folder without a symlink: the ten `crates/marley_*` and
    `vendor/alacritty_terminal`, which holds alacritty's `LICENSE-APACHE` as a file.
  - The root: `LICENSE-APACHE` (first line `Copyright 2022 - 2025 Zed Industries, Inc.`) and
    `LICENSE-GPL`; no `LICENSE-AGPL`. Zed's `crates/gpui` and `crates/util` hold
    `LICENSE-APACHE -> ../../LICENSE-APACHE`; `crates/editor` holds `LICENSE-GPL ->
    ../../LICENSE-GPL`.
  - The ten Marley manifests: `license = "MIT OR Apache-2.0"`, `publish.workspace = true`
    (`publish = false` in `[workspace.package]`).
  - `script/licenses/zed-licenses.toml` and `deny.toml`: `private = { ignore = true }`, `MIT` and
    `Apache-2.0` accepted.
  - `README.md`'s `### Licensing` (Zed's): "Zed source code is licensed primarily under
    GPL-3.0-or-later, with Apache-2.0 components where marked."
- **Decisions:** D1 to D4 in the spec.

### Design
- **Approach.** For each `crates/marley_*`: `ln -s ../../LICENSE-APACHE LICENSE-APACHE`, and
  `LICENSE-MIT` with the MIT text (the OSI and SPDX wording, verbatim) under `Copyright (c) 2026
  Ignibyte`. In `script/check-licenses`, the loop skips a manifest under `vendor/` before its
  checks, with a `# Marley:` comment saying why; the rest of the script is untouched. In
  `README.md`'s Licensing section, one paragraph after Zed's first line: the Marley crates, their
  holder, their two licenses and their two files, and that the root Apache text's first line is
  Zed's notice for Zed's code.
- **File manifest.**
  - Marley crates' folders: `crates/marley_{agent,browser,dcs,fleet,mcp,rail,remote,system_one,terminal,workbench}/LICENSE-APACHE`
    (symlinks) and `…/LICENSE-MIT` (files).
  - Zed paths: `script/check-licenses` (its ledger row, new, first); `README.md` (its row
    extended, first).
  - No `.rs`: the gate runs `--fast`, and the e2e run is `just shot`.
- **Ledger rows.** `script/check-licenses`, new; `README.md`, extended.
- **Knowledge at Complete (expected).** An AD for the licensing layout (D1 to D3); a lesson on
  Zed's check and third-party crates under `vendor/`.

### E2E plan
N/A — no UI delta. `just shot` runs the app once (§7).

| REQ | Check | Evidence |
|---|---|---|
| REQ-001 | `readlink crates/marley_*/LICENSE-APACHE`, and the check | ten `../../LICENSE-APACHE` |
| REQ-002 | the ten `LICENSE-MIT` compared with one another and searched for the line | one text; `Copyright (c) 2026 Ignibyte` in each |
| REQ-003 | `script/check-licenses` | exit 0, `check-licenses succeeded` |
| REQ-004 | review of the README paragraph | review |
| REQ-005 | `script/gates.sh --fast`; `just shot` | the gate's log; the shot |

### Risks
- The root Apache text's first line names Zed Industries: a reader of a Marley crate's
  `LICENSE-APACHE` sees Zed's notice. The README says whose it is, and each crate's `LICENSE-MIT`
  names Marley's holder. Changing the root file would misstate Zed's Apache crates.
- A later vendored crate is skipped the same way; its own license file is then its record.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker present ✓; the ledger rows first
  (`script/check-licenses` new, `README.md` extended) ✓; `script/check-licenses` ✓; `README.md` ✓;
  the ten crates' files, once #587's release build had finished compiling ✓; the review ✓. No
  `.rs`, so no cargo ran.
- **What was built.**
  - Each of the ten `crates/marley_*`: `LICENSE-APACHE -> ../../LICENSE-APACHE` and
    `LICENSE-MIT`, the MIT text as OSI and SPDX publish it under `Copyright (c) 2026 Ignibyte`,
    one text for all ten.
  - `script/check-licenses`: the manifest loop skips a manifest under `vendor/` first, with a
    `# Marley:` comment; the checks themselves are Zed's, unchanged.
  - `README.md`: after the Licensing section's first line, the Marley crates' holder, their two
    licenses at the reader's option, their two files, whose notice the root Apache text's first
    line is, and the crates under `vendor/` keeping their own licenses.
- **Deviations from the design**: none.
- **The review**:
  - REQ-001, REQ-003: `script/check-licenses` prints `check-licenses succeeded` (exit 0); before,
    it stopped on `crates/marley_agent`.
  - REQ-002: each `LICENSE-MIT` is the same text, with the notice once.
  - The hunk in Zed's script is additive, its ledger row written first. `shellcheck` on the
    script reports three SC2155 warnings on Zed's own lines 37 to 40, none on the hunk; the gate's
    shellcheck list does not include the script, and Zed's lines stay as upstream has them.
  - Provenance: the MIT text is the license's own; nothing else is copied.

## Phase 3 — Test
- **Checklist** (no task tool): the file checks ✓; `just shot` and its shot read ✓; `gates.sh
  --fast` ✓ (no `.rs`).
- **The checks** (run again in this phase): `readlink` over the ten `LICENSE-APACHE`, each
  `../../LICENSE-APACHE` (REQ-001); the ten `LICENSE-MIT` one text (a single checksum) with
  `Copyright (c) 2026 Ignibyte` once in each (REQ-002); `script/check-licenses` exits 0 with
  `check-licenses succeeded` (REQ-003).
- **`just shot`** (`446-license-files`, §7's run for a change with nothing new to see): Marley
  starts on the copied profile and draws its window, the rail and the last session's project,
  with nothing of #446 to see; the runner reported the user's window and workspace as they were.
  The shot stays in the scratchpad: the copied profile shows the user's own session.
- **REQ-004**: reviewed, the README's paragraph names the holder, the two licenses at the
  reader's option, the two files, whose notice the root Apache text's first line is, and the
  crates under `vendor/`.
- **The gate**: `script/gates.sh --fast`: `GATE GREEN [fast]`, 15 gates PASS (rustfmt, clippy on
  every target, cargo-audit, cargo-deny, cargo-shear, gitleaks, shellcheck, no-suppressions,
  source-bans, docs, zed-ledger, manifests, typos, semgrep, dylint); the log is scratchpad
  `446/gate.log`. No `.rs` changed, so the commit needs no diff receipt.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented**: `CHANGELOG.md` (Added: license files for the Marley crates); `README.md`'s
  Licensing paragraph (the change itself). The two Zed paths' ledger rows, `script/check-licenses`
  (new) and `README.md` (extended), were written before their edits and describe what shipped.
  No per-crate architecture note changes: no crate's code or surface moved.
- **Knowledge**: AD-claude-446-the-marley-crates-carry-their-own-mit-file-and-zeds-apache-link-001;
  L-claude-446-zeds-license-check-wants-a-symlink-in-every-crate-001. No F-block: nothing broke.
- **Brain**: consultation 6a8b91e08a184237880824efc066c193 closed with
  `decisions/the-marley-crates-carry-their-own-mit-file-and-zeds-apache-link`.
- **No release install**: no code changed since #587's, which is installed.
- **Closed**: TICKET-446 moved to `tickets/closed/`; its BACKLOG row went at promotion.
