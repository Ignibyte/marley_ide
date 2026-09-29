---
pipeline_id: aab0119c-78d4-4ff9-aa7f-1d1000d728ca
ticket: docs/planning/tickets/closed/TICKET-446-marley-crate-license-files.md
status: Phase 4 — Complete PASS
title: "License files for the Marley crates"
type: chore
slice: licensing; after #438's inspect ledger (U5)
references: [docs/planning/pipeline/completed/438-marley-layout-and-rail.notes.md, docs/decisions/licensing-ownership-strategy.md]
---

## Title
Every Marley crate says `license = "MIT OR Apache-2.0"` and carries no license file, so Zed's
`script/check-licenses`, which its CI runs, stops on the first of them, and nothing in the tree
names who holds the Marley crates' copyright. Each Marley crate gets its two license files, the
MIT text with Chad's holder, Ignibyte, and the Apache-2.0 text the way Zed's Apache crates carry
it; the check skips the vendored crate, which keeps alacritty's own; the README says who holds
what.

## Scope
### In
- **Each of the ten `crates/marley_*`**: `LICENSE-APACHE`, a symlink to the root
  `LICENSE-APACHE` at the relative path the check expects (`../../LICENSE-APACHE`), as Zed's
  Apache crates (`gpui`, `util`) have it; and `LICENSE-MIT`, a file of its own with the MIT text
  and `Copyright (c) 2026 Ignibyte`.
- **`script/check-licenses`** skips the crates under `vendor/`: third-party crates Marley patches
  and carries (`alacritty_terminal`, Apache-2.0), which keep their own license files, where the
  check wants a symlink to Zed's.
- **The README's Licensing section**: one paragraph on Marley's crates, their holder and their
  two licenses, and what the root Apache text's first line is.

### Out (explicitly deferred)
- A root `LICENSE-MIT`: it would read as covering the tree, which is Zed's under GPL-3.0-or-later
  and Apache-2.0 (D2).
- `check-licenses` in Marley's gate (`script/gates.sh`): the gate runs `cargo deny`'s license
  check already; a later chore, if Zed's CI is not run on the fork.
- IP counsel's sign-off, which `docs/decisions/licensing-ownership-strategy.md` keeps pending
  before commercializing.

## Reference (§20)
Upstream Zed: `script/check-licenses` wants each crate's folder to hold a `LICENSE-GPL` or
`LICENSE-APACHE` symlink to the root file, and no AGPL and no `license-file`; Zed's
Apache-licensed crates (`crates/gpui`, `crates/util`) carry `LICENSE-APACHE ->
../../LICENSE-APACHE`, and its README's Licensing section states the tree's licenses. Marley's
crates follow the same form, add the MIT file the Rust convention for `MIT OR Apache-2.0` pairs
with it, and state their own holder. Warp: N/A, licensing is outside the terminal.

### Prior art
- **The code we ship.** `script/check-licenses` (walks `git ls-files` for `Cargo.toml`, checks
  each crate folder's symlink with `readlink` against `../` per level, fails on a regular file
  named `LICENSE-GPL` or `LICENSE-APACHE`, on `license-file` and on AGPL; it never reads a
  `LICENSE-MIT`); Zed's CI runs it (`.github/workflows/run_tests.yml:813`). Today it stops on
  `crates/marley_agent`, and the ten Marley crates and `vendor/alacritty_terminal` are the only
  folders without a symlink (checked 2026-09-28). The root `LICENSE-APACHE` opens with
  `Copyright 2022 - 2025 Zed Industries, Inc.`; `LICENSE-GPL` holds the GPL text.
  `script/licenses/zed-licenses.toml` (cargo-about) and `deny.toml` both ignore private crates,
  and every Marley crate is `publish = false` through the workspace, so neither reads them.
  `vendor/alacritty_terminal` is alacritty's crate (Apache-2.0) with its own `LICENSE-APACHE`
  file.
- **Published material.** The Rust API Guidelines' C-PERMISSIVE (a crate under `MIT OR
  Apache-2.0` carries `LICENSE-MIT` and `LICENSE-APACHE`); the MIT license's text as the Open
  Source Initiative and SPDX (`MIT`) publish it; Apache-2.0's own appendix, which puts a
  holder's notice beside the license rather than inside its terms.
- **Behavior maps.** None apply: no Warp, Zed or Orca map covers licensing.
- Does a crate we build own this seam? No: this is files and a script.

## UI proof
N/A — no UI delta: license files, one check script and a README paragraph change nothing the
user sees or types into; the e2e run is `just shot`.

## Locked-In Decisions
- D1: The Apache side is a symlink to the root `LICENSE-APACHE`, the form Zed's check demands
  and Zed's own Apache crates use. The root file's first line is Zed's notice for Zed's code;
  Marley's holder is stated in each crate's `LICENSE-MIT` and in the README.
- D2: The MIT text lives in each Marley crate, a file of its own, as a crate under `MIT OR
  Apache-2.0` carries it, so each crate is whole on its own. A root `LICENSE-MIT` would read as
  the tree's license.
- D3: The check skips `vendor/`, whose crates are third-party and keep their own license files;
  every other crate is checked as before (a `# Marley:` hunk, its ledger row first).
- D4: The holder is `Ignibyte`, as Chad gave it on 2026-09-28: `Copyright (c) 2026 Ignibyte`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each `crates/marley_*` shall hold `LICENSE-APACHE`, a symlink to the repository's `LICENSE-APACHE` at the relative path the check expects. | `script/check-licenses`; `readlink` over the ten |
| REQ-002 | Each `crates/marley_*` shall hold `LICENSE-MIT`, the MIT text with `Copyright (c) 2026 Ignibyte`. | A check over the ten: one text, the line present |
| REQ-003 | `script/check-licenses` shall pass over the tree, skipping the crates under `vendor/`. | Its exit 0 and `check-licenses succeeded` |
| REQ-004 | The README's Licensing section shall say who holds the Marley crates' copyright and under which licenses. | Review |
| REQ-005 | The gate shall be green, and `just shot` shall run. | `script/gates.sh --fast` (no `.rs`); `just shot` |

## Phase Plan
- **P1 Plan:** this spec; the design in the notes.
- **P2 Code:** the ledger rows first; the twenty files; the script's skip; the README paragraph.
- **P3 Test:** the checks; `just shot`; the gate.
- **P4 Complete:** CHANGELOG; the ledger capture; close the ticket, archive, commit.
