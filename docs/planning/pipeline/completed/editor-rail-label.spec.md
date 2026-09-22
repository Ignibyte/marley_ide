---
pipeline_id: bea123ab-4a42-4705-af74-836f5b8b5ab3
ticket: forge#240 (c5033176-df15-48ed-adbb-b3eaa25a8848) · local docs/planning/tickets/open/TICKET-240-editor-rail-label.md
aar_id: 934cef3b-a029-45ce-99a2-4114c2a24b70
status: Phase 5 — Complete PASS
title: Editor surface rail row shows a stable "Editor" label
type: chore
milestone: M14
references: [forge#237, forge#201, forge#177, forge#239]
---

## Title
Give the editor surface's LEFT-RAIL row a stable "Editor" label instead of the frozen first-file name
(chad live issue #2, his pick: stable label over track-active).

Today `Project::open_or_switch_code` (tabs.rs:274-281) creates the editor tab with `title = <first file's
file_name, else "code">`, and that title never changes as more files open or the active file switches — so the
rail row is stuck on file #1 ("inspect.md") while the file-tab STRIP correctly shows each open file. chad noticed
the mismatch ("retains the same name"). Decision: the rail row gets a calm, stable **"Editor"** identity; the
strip stays the source of truth for *which* file.

## Scope
### In
- **`tabs.rs`:** `Project::open_or_switch_code` creates the editor tab with a stable title `"Editor"` (a
  `const EDITOR_TAB_TITLE: &str = "Editor";`) instead of the first file's name. Re-assert the two title checks
  in `open_or_switch_code_cases` ("a.rs"/"code" → "Editor").

### Out
- The file-tab STRIP (the code-view render reading `surface.files()`) — unchanged; it keeps showing each file's
  own name. The active-file content render — unchanged.
- Track-the-active-file naming (the alternative chad weighed and rejected) — not this ticket; a trivial future
  flip if he changes his mind.
- Any per-tab icon in the rail (the rail rows are text labels today).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — set the title at creation.** `open_or_switch_code` → `Tab::code(EDITOR_TAB_TITLE, state)`. The rail's
  displayed label flows `rail_rows` (`row.label = tab.title`) → the shim's `live_tab_title(p, t, row.label)`
  (app.rs:4346). For an editor tab `tab.grid()` is None → no terminal → command/cwd None → `display_title`
  returns the **fallback** (= `tab.title` = "Editor"). So "Editor" reaches the rail with NO shim change. Chosen
  over deriving "Editor" in the shim (pure + unit-testable; the title field IS the rail-label source).
- **D2 — the strip is the source of truth** for which file (`surface.files()`, per-file names) — untouched.
- **D3 — a #177 rename still wins.** `live_tab_title` returns `custom_title` when the user renamed the tab, so
  "Editor" is only the DEFAULT; renaming still overrides it. No regression.
- **D4 — persistence is path-based, unaffected.** The editor tab persists as `TabLayout::Code(path)`
  (app.rs:915/2156), NOT its title; restore rebuilds via `open_or_switch_code` → title "Editor". Nothing keys
  off the editor tab's title.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | When the editor tab is created and as further files open, its `title` shall be the stable `"Editor"` — never the file name. | `open_or_switch_code_cases` unit: `active_tab().title == "Editor"` after 1 file, after N files, and after re-opening/switching |
| REQ-002 | The file-tab strip shall continue to show each open file's own name (the surface is unchanged). | `editor_surface` tests (`files()` names) — unchanged; strip reads `surface.files()` |
| REQ-003 | The left rail shall display "Editor" for the editor tab regardless of which/how many files are open. | driven capture — open 2+ files → the rail row reads "Editor"; the strip shows the distinct names; switching files keeps the rail "Editor" |

## Phase Plan
- **P2 Design** — confirm D1 (the `live_tab_title` fallback path); the file manifest (tabs.rs const +
  open_or_switch_code + the 2 test re-asserts); `cargo mutants --list -f tabs.rs` for the seam.
- **P3 Implement** — the const + the one-line title change + re-assert the 2 test title checks.
- **P3.5 Inspect** — critic: does anything else read the editor tab's title? does the live_tab_title fallback
  truly yield "Editor" (no term)? persistence unaffected? clean-room.
- **P4 Validate** — RUN open_or_switch_code_cases + editor_surface tests; gate green; DRIVEN capture (rail
  "Editor" across files).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #240; archive.
