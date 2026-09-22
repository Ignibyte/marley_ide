---
pipeline_id: a777da2a-a83c-42b9-88c0-e2e489327a1e
ticket: forge#57 (2d7d3df4-0ff3-4fe7-8e44-07a5c568496e) · local docs/planning/tickets/open/TICKET-057-file-finder.md
aar_id: 5edda1b8-fc27-4af7-a856-ea28ec7c5f91
status: Phase 5 — Complete PASS
title: fuzzy file-open overlay (cmd-P)
type: feature
milestone: M2.A
references:
  - crates/marley_app/src/finder.rs (NEW PURE module — FinderState)
  - crates/marley_app/src/app.rs (SHIM: RootView finder state + cmd-P overlay + key handling)
  - crates/marley_app/src/keymap.rs (cmd-P binding)
---

## Title
cmd-P opens a fuzzy FILE FINDER: type → the project's files ranked by `marley_search_core::fuzzy_rank`
→ Enter inserts the chosen path at the shell prompt. The moat's signature quick-open.

## Scope
### In
- PURE (`crates/marley_app/src/finder.rs`, NEW; cov/MSI 100 — a marley_app module, not app.rs):
  `#[derive(Debug, Default, Clone, PartialEq, Eq)] pub struct FinderState { query, selected }` mirroring
  `PaletteState`: `new`, `query()`, `selected()`, `push(text)` + `backspace()` (reset selected=0),
  `move_up()` (saturating_sub), `move_down(results_len)` (`(selected+1).min(len-1)`); plus
  `results(&self, files: &[&str]) -> Vec<usize>` (= `fuzzy_rank(files, query)` → `.index`) and
  `chosen<'a>(&self, files: &'a [PathBuf], results: &[usize]) -> Option<&'a Path>`.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView { finder_open, finder, project_files }`
  (project_files stored from #56's `list_files_in` call — no re-walk); a cmd-P overlay (palette pattern)
  showing the query + ranked rows with the selected highlight; when open, keys drive the finder (type/
  backspace/up/down/Enter→write chosen path to the PTY/Esc→close).
- `keymap.rs`: a bare `cmd-p` binding → `open-file-finder` (distinct from the palette's `cmd-shift-p`).

### Out
- Opening the file in an editor pane (first cut inserts the path at the prompt). Live file-watch. Scroll
  for very long result lists (cap the rendered rows). Preview pane. Multi-root.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `FinderState` in a NEW `finder.rs` (mirrors `palette.rs`) — a marley_app module → cov/MSI 100; the
  overlay/keys are the app.rs shim.
- D2 — cmd-P is BARE (`cmd-p`); the command palette stays `cmd-shift-p` — no chord conflict.
- D3 — "Open" = write the chosen path to the active session's PTY (inserts at the shell prompt) — the
  first cut; an editor-pane open is later.
- D4 — `project_files` is stored once at startup from the SAME `list_files_in` that feeds #56's tree
  (single walk); the finder ranks over it.
- D5 — `results` takes `&[&str]` (fuzzy_rank's shape); the shim builds the lossy strings per open.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `push`/`backspace` edits the query, `selected` shall reset to 0. | unit |
| REQ-002 | WHEN `move_up`/`move_down(len)` runs, `selected` shall clamp to `0..len` (empty list stays 0). | unit |
| REQ-003 | WHEN `results(files)` runs, it shall return the fuzzy_rank-ordered matching file indices for the query (empty query → all in order). | unit |
| REQ-004 | WHEN `chosen(files, results)` runs, it shall return the `selected`-th result's path, or `None` when results are empty. | unit |
| REQ-005 (visual) | WHEN cmd-P is pressed and a fragment typed, the overlay shall list the ranked matching files with the top/selected row highlighted. | self-test (drive cmd-P + type → capture) |
| REQ-006 | `scripts/gates.sh` GREEN, cov/MSI 100 on finder.rs; app shim excluded. | gate |

## Phase Plan
- **P2** — FinderState shapes, the overlay + key-handling + keymap shim, mutation targets, unit + self-test plan.
- **P3** — finder.rs + the app.rs shim (state, cmd-P action, overlay render, key routing) + keymap binding.
- **P3.5** — critic: the clamp/reset/chosen edges, results wiring, the cmd-P vs cmd-shift-p keymap
  non-conflict, the shim seam.
- **P4** — FinderState unit tests (cov/MSI 100) + `-p marley` green + the SELF-TEST capture (cmd-P lists
  ranked matches) + gate GREEN.
- **P5** — docs, AAR, archive, close #57.
