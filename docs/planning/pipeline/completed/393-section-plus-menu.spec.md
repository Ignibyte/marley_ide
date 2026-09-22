---
pipeline_id: 80bfed3f-3cfe-4694-ba85-8a335bc442fc
ticket: forge#393 (dd98405c-ec53-4aba-8376-8f4e434520c5) · local docs/planning/tickets/open/TICKET-393-section-plus-menu.md
aar_id: c91eb602-fa59-4414-9bb0-82ac683758ff
status: Phase 5 — Complete PASS
title: The section ＋ becomes a menu — section-scoped create/open verbs (the extension point)
type: feature
milestone: M27
references:
  - docs/planning/pipeline/completed/387-section-actions.spec.md (the ＋ this generalizes)
  - docs/marley_architecture/app_shell.md (#166/#175 context-menu machinery record)
---

## Title
chad (2026-07-22): "We will expand that + button to include various things (i think)." The #387 ＋
dispatches ONE hardcoded verb per section; it becomes a small anchored **section-scoped menu** — the
named extension point for the future "various things" — while staying **reuse-only** (every v1 item
is an existing verb; the #166/#175 context-menu machinery renders and dismisses it).

## Scope
### In
- **The menu items (v1, reuse-only):**
  - **Terminal＋** → *New Terminal* (`new_terminal_pane`) · *New Terminal Here*
    (`spawn_terminal_tab_in` at the focused pane's live cwd — the "Open Terminal Here" verb).
  - **Editor＋** → *Open File…* (`start_file_finder`) · *Open in Split* (the #246
    `finder_split = true` path).
  - **Browser＋** → *Forge* · *Agents* · *Details* (`open_or_switch_cockpit` per `RightSection`).
  - **Panes＋** (present only if #390 has landed) → *Split Right* · *Split Down*
    (`split_focused_pane` per axis).
- **Mechanics:** the ＋'s `on_mouse_down` opens `MenuKind::Section { p, section }` (new arm) anchored
  at the header row, instead of direct-dispatching; item click routes through the existing
  `run_context_menu_action`; esc/click-away dismiss = the existing machinery; one modal at a time
  (the #177 convention). `cx.stop_propagation()` still guards the #386 collapse on the parent header.
- **The pure seam generalizes:** `section_action(section) -> SectionAction` grows to
  `section_menu(section) -> Vec<(label, SectionAction)>` (first item = the #387 default, preserving
  muscle memory in the list order); `SectionAction` gains the needed variants (`NewTerminalHere`,
  `OpenFileSplit`, per-cockpit, per-axis). Routing stays pure — cov/MSI 100.
- **Dispatch:** `dispatch_section_action` handles the widened enum; the cross-project activate-first
  (#174) + the #387-F1 persist-the-switch rule
  (PR-claude-cross-project-affordance-must-persist-the-switch-001) apply to EVERY item.

### Out (explicitly deferred — the documented future extension point)
- SSH / remote-host targets (#87 remotes), workflow invocations (#204), a URL/browser target
  (Phase E), *New File* (no verb exists yet — inventing one violates reuse-only).
- Submenus, icons, keybinding hints in the menu rows.
- Any change to the ＋'s hover-reveal or hit-target behavior (#387 ships that; unchanged).

## Reference (§20)
**Convention, not app-matched.** Per-section header "+" menus follow the IDE section-toolbar
convention (VS Code panel toolbars open small anchored menus) — published-material leg. The section
model itself is chad's own; no Warp analog (Warp's "+" is a flat new-session button —
docs/warp_architecture/subsystems/03-terminal-session-core.md, research map; Zed's panel affordances
are feature-scoped — docs/zed_architecture/subsystems/07-workspace-panes-palette.md, research map).
No source read.

### Prior art
1. **In-repo owners (decisive):** the #166/#175 context-menu machinery (`MenuKind`, `context_menu`,
   `run_context_menu_action` — anchored render, esc/click-away, one-modal); #387's `section_action`/
   `dispatch_section_action`/`start_file_finder` (the seam that widens); the #140 "+" lineage; the
   verbs themselves (`new_terminal_pane`, `spawn_terminal_tab_in` #294/#281, `finder_split` #246,
   `open_or_switch_cockpit` #153, `split_focused_pane` #155/#166).
2. **Behavior maps** — the two research maps above (negative).
3. **Published material** — IDE section-toolbar menu convention.
4. **Permissive deps** — "none: gpui provides click/anchor primitives already wrapped by #166; no
   external owner."

## Locked-In Decisions
- **D1 — Reuse-only v1:** every item dispatches an EXISTING verb; the future items are documented,
  not built.
- **D2 — The click opens the menu** (replacing #387's direct dispatch): one extra click buys the
  extension point; the #387 default verb is the FIRST item of each menu.
- **D3 — The #166 machinery, not a new popover:** one menu system in the app.
- **D4 — Every item honors activate-first + persist-the-switch** (the #387-F1 rule, uniformly).
- **D5 — Pure `section_menu` table cov/MSI 100;** the menu render/dispatch is the masked shim.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a section's ＋ is clicked, an anchored menu shall open listing that section's verbs (Terminal 2, Editor 2, Browser 3, Panes 2 when present), first item = the #387 default. | Pure `section_menu` units; capture. |
| REQ-002 | WHEN a menu item is clicked, its verb shall dispatch against the ＋'s row project (activate-first + persist-the-switch), and the menu shall dismiss. | Routing units; driven/mechanism. |
| REQ-003 | Esc or a click-away shall dismiss the menu without dispatching (the #166 machinery). | Existing-machinery test + drive. |
| REQ-004 | The ＋ click shall still never trigger the #386 section collapse. | The #387 stop_propagation mechanism (unchanged) + drive. |
| REQ-005 | The deferred future items (SSH, workflows, URL, New File) shall be documented as the extension point, not implemented. | Doc review. |

## Phase Plan
- **P2 Design** — `MenuKind::Section` shape + anchor geometry; the `SectionAction` variant widening
  (+ its `dispatch` arms); the `section_menu` table; menu-under-collapse edge (opening on a
  collapsed section's header).
- **P3 Implement** — tabs.rs (`section_menu` + variants) + app.rs (MenuKind arm, render, dispatch).
- **P3.5 Inspect** — critics: modal-exclusivity (menu vs palette vs rename), anchor drift on scroll,
  the persist rule on every arm, Panes-arm gating when #390 absent.
- **P4 Validate** — pure-table unit matrix (cov/MSI 100) + `--diff` gate + a menu-open capture and
  one item drive per section (env-permitting; else units + the #166 mechanism).
- **P5 Complete** — CHANGELOG, app_shell.md, archive, close #393.
