---
pipeline_id: 8ae5d149-44c6-487d-8237-4bda9de2aafd
ticket: forge#241 (71ad57c2-5b9f-4d86-a26c-5f1d4d5841ef) · local docs/planning/tickets/open/TICKET-241-disambiguate-tab-labels.md
aar_id: c3d175c1-4439-45ba-91d1-117a9b292d69
status: Phase 5 — Complete PASS
title: Disambiguate identical idle-terminal tab labels in the rail
type: feature
milestone: M14
references: [forge#201, forge#157, forge#177, forge#112]
---

## Title
Disambiguate identical idle-terminal tab labels in the left rail (chad observed several indistinguishable
"Marley" tabs).

An idle terminal's rail label falls back to its cwd basename (`live_tab_title` → `display_title(None, None, cwd,
fallback)`, app.rs:4345), so N terminals idling in the same directory all read "Marley". When ⌘T opens more
terminals (all rooted in the project), the rail becomes a wall of identical "Marley" rows.

## Scope
### In
- **Pure `disambiguate_labels(labels: &[String]) -> Vec<String>`** (titlebar.rs, cov/MSI 100) — for a project's
  tab labels IN ORDER, append a `" {n}"` suffix (2, 3, …) to the 2nd-and-later occurrence of each duplicate; the
  FIRST occurrence stays bare. `["Marley","Details","Marley","Marley"]` → `["Marley","Details","Marley 2","Marley
  3"]`. Uniques (Details/Agents/Forge/Editor) are untouched.
- **Shim pre-pass** (app.rs): BEFORE the rail render loop, compute per-project disambiguated labels over the
  **FULL** `rail_row_list` (NOT the `rail_skip`-truncated view — scroll-safe) into a `(project, tab) → label`
  map; the `RailLevel::Tab` arm looks the label up (instead of the inline `live_tab_title`); the #112 "Search
  tabs" filter matches the disambiguated label.

### Out
- Cross-project disambiguation — different projects are separate rail groups; a "Marley" tab under project A and
  one under project B is fine (per-project scope).
- Changing the idle-terminal label SOURCE (still the cwd basename via #201) — only the collision suffix is new.
- A #177 custom rename is a distinct explicit label; if a user renames two tabs to the SAME name, the generic
  helper disambiguates those too (acceptable; not a special case).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — disambiguate the DISPLAYED (live) labels, in the shim**, not rail_rows' static `tab.title` (the static
  "terminal N" is already unique; the collision is on the live cwd-basename labels which only the shim computes).
- **D2 — a pure helper does the suffix logic** (`disambiguate_labels`), so it's exact-value + mutation tested;
  the shim only wires it (batch the labels → map → lookup).
- **D3 — the pre-pass runs over the FULL rail list** (all tabs of each project), so the suffix is correct even
  when the rail is scrolled (a per-visible-row counter would miscount skipped siblings). DESIGN confirms the
  batching shape.
- **D4 — suffix format `" {n}"`** (space + 1-based ordinal, first bare) — the browser/VSCode idiom. DESIGN
  confirms the exact format + where the helper lives (titlebar.rs, beside `display_title`).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `disambiguate_labels` shall return all labels unchanged when unique, and append `" {n}"` (n = 2,3,… the 1-based occurrence) to the 2nd-and-later duplicates while the first stays bare. | unit tests: no-dup, one dup pair, triple, all-same, empty, interleaved |
| REQ-002 | The rail's Tab-row labels shall be disambiguated per project over the full tab list, independent of the scroll offset (a scrolled rail shows correct suffixes). | shim review (pre-pass over the FULL list) + driven capture |
| REQ-003 | The left rail shall display distinct labels for same-cwd idle terminals (e.g., "Marley", "Marley 2", "Marley 3"). | driven capture — relaunch with the persisted multi-terminal layout |

## Phase Plan
- **P2 Design** — the helper signature + suffix format + location; the shim pre-pass shape (build the map over
  the full list; the Tab arm + filter lookup); `cargo mutants --list -f titlebar.rs`; the test matrix.
- **P3 Implement** — pure `disambiguate_labels` (+ tests) → the shim pre-pass + the Tab-arm lookup.
- **P3.5 Inspect** — critic: the helper's ordinal correctness (off-by-one on n), the scroll-safety (full list),
  the filter still matches, no perf blowup, the #240 "Editor"/cockpit uniques untouched, clean-room. AWAIT the
  critic before Inspect-PASS (the #240 lesson).
- **P4 Validate** — RUN the helper unit tests; gate green; DRIVEN capture (multiple "Marley" → "Marley"/"Marley
  2"/…).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #241; archive.
