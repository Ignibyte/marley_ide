---
pipeline_id: b570d190-c1e1-49d5-9361-c1e823cc2fa4
ticket: docs/planning/tickets/open/TICKET-831-a-project-mark-from-logo-png.md
status: Phase 4 — Complete PASS
title: A project's mark from its logo.png
type: feature
slice: The Marley look, W1.2 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
A project brings its own mark. Marley looks for `logo.png`, then `logo.svg`, in a project's top
folder ahead of #564's other icon names. A file the user picks wins, and with nothing found the
mark is the name's first letter on a tint of its own. The mark shows in the rail's group header,
the shrunk strip, Home's recent projects and the title bar, and follows a change to the file. Today
#564's icon lives inside the rail entity, favicons come first, a header with no icon draws nothing,
and Home and the title bar have no mark.

## Scope
### In
- The search order: `logo` first in #564's names, so `logo.png`, then `logo.svg` (then `.webp`,
  `.ico`), then #564's other candidates as today.
- A shared cache: the per-project icon state moves from `Rail.project_icons` into a gpui
  `Global` (`ProjectMarks`, in `project_icons.rs`), keyed by the project's first folder. A
  search is asked for by any view that draws a mark, runs once in the background, and is asked
  again by the rail's worktree events when a candidate file changes (#564's trigger). Views
  observe the global.
- `ProjectMark { Image(Arc<RenderImage>), Letter(char) }` and one renderer, `project_mark(root,
  name, size, cx)`: the image, or the uppercase first letter on the theme's player tint for a hash
  of the name (#830's tile face, moved here and shared).
- The override: `marley.project_logos: Option<BTreeMap<String, String>>` in the user settings,
  from a project's first folder to a file path or `"letter"`. The rail header's right-click menu
  gains "Choose Logo…" (a file prompt), "Use a Letter", and, while an override is set, "Use the
  Project's Logo" (which removes the entry).
- The rail header draws the mark always, its letter included, at 16 px; the strip's tile uses the
  shared renderer.
- Home's recent projects: the mark in place of the folder icon (a recent project not open is
  searched through the same cache).
- The title bar: a project mark before the project's name when the project has an image (logo or
  pick), through a new `MarleyProjectMark` global hook in `title_bar`, as `MarleyTitleBarButtons`
  hands over Rusty's and Marley's buttons.

### Out (explicitly deferred)
- The mark on desktop notifications and in the switcher: TICKET-859 (notifications need an image
  field in gpui on each platform).
- A remote (SSH) project's logo: TICKET-860. Remote projects show their letter.
- A Settings-window editor for `project_logos` (a map of folders; the header menu sets it, and
  the file takes hand edits).
- The board's "has a logo" toast on first open.

## Reference (§20)
N/A — Marley-specific. Zed has no per-project mark (its title bar shows the name; its recent
projects picker lists names and paths). The behavior is the design canvas's ProjectLogo board
(the search order, where the mark shows, the per-project pick), built on Marley's own #564 icon
search.

### Prior art
- The code we ship: `project_icons.rs` (#564) already searches, size-limits (256 KiB), decodes
  and shrinks to 32 px, off the main thread through `Rail::search_icon`, and re-searches on
  `WorktreeUpdatedEntries` for a candidate path (`note_icon_files`). `launch.rs`'s
  `LaunchConfigs` is the shape for a path-keyed `Global` that several views read.
  `MarleyTitleBarButtons` (`title_bar.rs`, #746) is the shape for handing the title bar a view
  from a Marley crate. `PromptLevel`/`cx.prompt_for_paths` is Zed's file prompt.
  `players().color_for_participant` gives the letter tints (#830).
- The Orca behavior map (`docs/orca_architecture/`) is #564's source for the candidate names.
- Ledger: BF-claude-svg-asset-not-registered-renders-blank (a missing image draws a blank: check
  the shots show the mark), F-claude-438-a (state read by another surface comes from `refresh`,
  not `render`), PR-claude-830 (frames where every form renders, not needed here). The brain had
  nothing on this seam.

## UI proof
Scenario `script/e2e/831-a-project-mark-from-logo-png.sh` (sway): projects `logo` (with a
`logo.png`, a 64 px orange square with a white "L" drawn by `magick`) and `plain` (no logo) in one
window; both in the profile's recent projects.
- `831-01-rail-night`: the rail in Marley Night: `logo`'s header with its image, `plain`'s with a
  "P" on a tint (REQ-001, REQ-002, REQ-003).
- `831-02-title`: the title bar on `logo`: the image before its name (REQ-005).
- `831-03-strip`: the rail shrunk: the image tile and the letter tile (REQ-003).
- `831-04-home`: Home's recent projects with both marks (REQ-004).
- `831-05-replaced`: `logo.png` replaced by a green square while Marley runs: the header and the
  title bar follow (REQ-006).
- `831-06-letter`: "Use a Letter" from `logo`'s header menu: its header shows "L" on a tint, and
  `marley.project_logos` is printed from the profile (REQ-007).
- `831-07-day`: Marley Day: the headers, both marks (REQ-002).

## Locked-In Decisions
- D1 — `logo` moves to the head of #564's names; the other fourteen keep their order behind it,
  so a project that showed a favicon and has no logo keeps its favicon.
- D2 — One `Global` cache, `ProjectMarks`, keyed by the first folder, replaces the rail's private
  map. The rail's triggers (refresh, worktree events) call into it, and every view that draws a
  mark asks it.
- D3 — The override lives in the user settings as a folder → file-or-`"letter"` map, set from the
  header menu. A picked file goes through the same size limit and decode, and is watched only by
  its path: a change to it is seen at the next search the rail asks for.
- D4 — The title bar shows the project mark only for an image; a letter there would repeat the
  name beside it.
- D5 — A remote project's mark is its letter until TICKET-860.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE a project's top folder holds `logo.png` (or `logo.svg`), the rail header shall show that image as the project's mark, ahead of any other icon. | `831-01-rail-night` |
| REQ-002 | WHERE a project has no logo, no other icon and no pick, the rail header shall show its first letter on a tint of its own, in Marley Night and Marley Day. | `831-01-rail-night`, `831-07-day` |
| REQ-003 | The shrunk strip shall show the same mark as the header for each project. | `831-03-strip` |
| REQ-004 | Home's recent projects shall show each project's mark in place of the folder icon. | `831-04-home` |
| REQ-005 | WHERE the shown project has an image mark, the title bar shall show it before the project's name. | `831-02-title` |
| REQ-006 | WHEN the logo file changes, every mark of that project shall show the new image. | `831-05-replaced` |
| REQ-007 | WHEN the user picks "Use a Letter" (or "Choose Logo…") in a header's menu, the project shall show that mark and `marley.project_logos` shall record it. | `831-06-letter`; the printed setting |
| REQ-008 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | The review of the diff; gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the global cache and renderer, the search order, the override and the menu
  entries, the header, strip, Home and title bar marks, the touchpoint rows; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check above; read every shot.
- **P4 Complete** — CHANGELOG, the guide and the architecture note (§21), ledger capture (§19),
  close the ticket, archive, commit.
