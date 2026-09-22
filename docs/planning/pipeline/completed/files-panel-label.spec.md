---
pipeline_id: f9399f07-4685-4339-bf0f-21245d511b69
ticket: forge#190 (7f2d0ce2-60eb-4a97-a83d-a1f922c6660f) · local docs/planning/tickets/open/TICKET-190-files-panel-label.md
aar_id: fe8cb735-d10e-46d7-bcb7-52c3191549c4
status: Phase 1 — Plan PASS · Phase 2 — Design PASS · Phase 3 — Implement PASS · Phase 3.5 — Inspect PASS · Phase 4 — Validate PASS · Phase 5 — Complete PASS
title: The "Files" panel is mislabeled — the real file browser has no name and the tab dock wears "Files"
type: bug
milestone: M12.1
references: []
---

## Title
Clicking a file "in the Files panel" doesn't show its code view. **Two causes found (Phase 4 live repro):**
(1) **the real bug — a silent file-open failure:** the file tree / ⌘P finder hand `open_file_in_viewer` a path
RELATIVE to the project root, but `std::fs::read` resolves it against the process CWD; a bundled app launched via
LaunchServices runs with CWD `/`, so the read silently fails (`Err(_) => {}`) and the click does nothing.
(2) **a discoverability aggravator:** the panel labeled **Files** is actually the Workspace→Tab navigator; the
real file browser (`files_panel`) was unlabeled. (chad live-app feedback #3.)

## Scope
### In
- **[PRIMARY] Fix the file-open path resolution.** Add a pure `resolve_under_root(root, path)` (marley_project):
  a relative path is joined onto the project root (CWD-independent), an absolute path passes through. Route
  `open_file_in_viewer` through it. This makes a file click (and the ⌘P-finder ⌘↵) actually open the code view
  from any CWD. PURE + unit-tested + mutation-covered.
- Rename `dock_title(DockSide::Left)` **"Files" → "Workspace"** (layout.rs) — the left dock renders the
  Workspace→Project→Tab navigator (`rail_rows`, a "Search tabs" box), not files. PURE; update its unit test.
- Give `files_panel` (app.rs) a **"Files" title header** (shared `caption_header`, matching `dock_panel`) so the
  actual file browser is identifiable. Shim (render).

### Out (explicitly deferred)
- **Defaulting the file browser open** (`FilesOpen: bool = false → true`). Note as a possible follow-up.
- The ⌘P-finder's plain-↵ *paste* branch (inserts a relative path into the terminal) — left as-is; only the ⌘↵
  *open* branch benefits from the resolution fix, and it does so automatically via `open_file_in_viewer`.
- Re-architecting the docks (merging tabs+files into one panel).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Direction chosen autonomously.** chad was away; the AskUserQuestion (relabel vs relabel+show vs
  a specific-file failure) timed out. Proceeding with the relabel fix (the essential one for chad, who has the
  browser open). **FLAG at delivery for chad's review** — including the exact word for the left dock.
- **D2 — Left dock title = "Workspace".** Most accurate for the Workspace→Project→Tab tree and matches the
  code's own name ("the left rail is the Workspace → Project → Tab tree"). Alternatives chad may prefer: "Tabs"
  (matches the "Search tabs" box), "Sessions". The panel has an internal "WORKSPACE" section header; if the
  title↔subsection redundancy reads poorly, design may adjust the title or drop the subsection header.
- **D3 — Root cause is the stale label**, not a broken handler. #152/#154 repurposed the left dock from the
  file tree (#56) to the tab navigator and moved files into `files_panel`, but `dock_title(Left)` kept "Files".

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the left dock header renders, the shell shall title it "Workspace" (never "Files"). | unit `dock_title(DockSide::Left) == "Workspace"`; driven capture shows "Workspace" atop the tab tree |
| REQ-002 | WHEN the file-browser panel (`files_panel`) renders, the shell shall display a "Files" title header above the file tree. | driven capture shows a "Files" header atop the file (directory) tree |
| REQ-003 | WHEN a user clicks a file row in the file browser, the shell shall open and switch to a code tab rendering that file's contents. | driven capture — clicking a repo file (`audit.toml`) shows its code view + a tab |
| REQ-004 | WHEN resolving a project-relative file path for reading, the system shall join it onto the project root (absolute paths unchanged) so the read is independent of the process CWD. | unit `resolve_under_root` (relative→joined, absolute→unchanged, root-sensitive) + mutation |

## Phase Plan
- **P2 Design** — name the exact edits: `dock_title` arm + its test (layout.rs); the `files_panel` header row
  (app.rs, matching the left-dock header idiom / `dock_panel` header). Test plan: the pure `dock_title` unit +
  the two driven captures. Confirm no other consumer hardcodes the "Files" dock label.
- **P3 Implement** — make the two edits; `cargo check`.
- **P3.5 Inspect** — critics vs the diff (any other "Files" label consumer? the palette "Files" command that
  toggles `files_open` — keep it; it refers to the browser, which is correct). Fix real findings.
- **P4 Validate** — the `dock_title` unit test; driven captures for REQ-001/002/003; the staged `--diff` gate.
- **P5 Complete** — CHANGELOG + app_shell.md (the dock-title/label correction), AAR capture, close the ticket.
