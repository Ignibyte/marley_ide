---
pipeline_id: b910b1ff-6a5a-468f-8d6d-3c0c130ab3a0
ticket: docs/planning/tickets/closed/TICKET-460-marley-layout-by-default.md
status: Phase 4 — Complete PASS
title: The Marley layout by default
type: feature
slice: workbench shell (the Default paragraph of the plan)
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.spec.md]
---

## Title
A fresh install opens in Zed's layout, and the user has to run "marley: use marley layout" to
reach the rail. Chad wants the fork to start in its own layout. `marley.layout` now defaults
to `marley`, and `zed` stays a choice.

## Scope
### In
- **The default** (`crates/settings_content/src/marley.rs`, Marley's file in a Zed crate): the
  `#[default]` variant of `MarleyLayout` is `Marley`, and the field's doc says so.
- **The reader** (`marley_workbench`): `MarleySettings::from_settings` falls back to that
  default; its comment names it.
- **Zed's tests** (`crates/zed/src/zed.rs`, test module): `init_test_with_state` writes
  `marley.layout = zed` before `initialize_workspace`, since those tests test Zed's own
  layout.
- **Marley's tests**: each test that leant on the Zed default sets the Zed layout, and a test
  proves the new default.
- **Docs**: the shell plan's Default paragraph, the workbench note, the ledger rows,
  CHANGELOG.

### Out (explicitly deferred)
- A user who wrote `zed`: the choice stands.
- Zed's onboarding and welcome page: unchanged; they open in the center.
- A Marley entry in Zed's Panel Layout menu (the plan defers it).

## Reference (§20)
N/A — Marley-specific: which of the fork's two layouts it starts in. The design is the shell
plan's (`docs/marley/workbench-shell.md:133-136`): the same switch with the default flipped.

### Prior art
- **Behavior maps:** none apply.
- **Published material:** none.
- **Code we already ship.**
  - The setting and its reader: `MarleyLayout` (`crates/settings_content/src/marley.rs`) and
    `MarleySettings::from_settings`. `default.json` carries no `marley` block, so the enum's
    default decides.
  - The Marley layout already opens from the start for anyone whose settings choose it:
    `init` applies its defaults, and a window restored in it builds the rail open (#438,
    #442).
  - `zed`'s test init reaches `marley_workbench::init` through `initialize_workspace`
    (`crates/zed/src/zed.rs:428-430`, `:6154-6221`), so those tests take the default too.

## UI proof
UI-AFFECTING.
- **Driven tests:** with no layout in the settings, a new window gets the rail and the
  Marley defaults; with `zed` in the settings, it gets Zed's sidebar and Zed's defaults.
- **Live drive:** start Marley with a settings file that has no `marley` key and see the rail.
  Chad's own settings choose `marley` since 07:47 today, so the drive needs a scratch config
  directory; it needs no input.

## Locked-In Decisions
- D1 — The default is `MarleyLayout::Marley`, set on the enum, the same place the Zed default
  was. `default.json` stays free of a `marley` block, which would be a second touchpoint.
- D2 — Zed's own tests pin the Zed layout once, in `init_test_with_state`, before
  `initialize_workspace`, so they keep testing upstream's behavior and an upstream test
  merged later needs no change.
- D3 — A user who never chose a layout moves to the Marley layout on update. That is the point
  of the default.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the settings hold no `marley.layout`, Marley shall open its windows in the Marley layout | unit test; driven test |
| REQ-002 | WHERE the settings choose `zed`, Marley shall keep Zed's layout | driven tests |
| REQ-003 | Zed's own `zed` crate tests shall run in Zed's layout and stay green | `cargo nextest run -p zed` (gate:3's scope) |
| REQ-004 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the ledger rows first, then the default, the reader's comment and the Zed
  test pin; fmt and clippy clean.
- **P3 Test** — the Marley tests made explicit, the new default test, negative checks, the
  `zed` crate's tests, the live drive, `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
