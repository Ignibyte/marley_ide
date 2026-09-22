---
ticket: TICKET-009
forge: forge#13 (5aebc012-a61f-41f8-8a83-bce7a1891a90)
status: closed
type: feature
milestone: M1
sprint: M1.A — The Usable Terminal (seq 2/5)
branch: ticket-009-ui-components
pipeline: docs/planning/pipeline/active/ui-components-theme.spec.md
spec: docs/specs/SPEC-ui-components.spec.md
aar: 585c0d67-ebf5-4e72-863c-33b3028c811c
---

# TICKET-009 — marley_ui_components (M1.A cut: the theme vocabulary)

Sprint M1.A seq 2/5. The M1.A cut of SPEC-ui-components: the workspace's **theming value vocabulary** —
`Appearance` (Light/Dark) + `ThemeColors` (7 `gpui::Hsla` roles + 3 metrics) + `default_for` (the
Light/Dark fallback palettes). The foundation `terminal_blocks`/`editor`/`app_shell` render from.

**The 5 widgets (button/switch/dialog/tooltip/keyboard-shortcut) DEFER to M1.B** — they serve the
command-palette/settings UI, not the terminal. See the pipeline spec for the scope call.

## Acceptance
- `Appearance` + `ThemeColors` + `default_for` (Light ≠ Dark, R17) — a PURE value-type library
  (no Render/headed surface) → **cov 100 / MSI 100 whole-crate**, gate-15 N/A.
- FULL `scripts/gates.sh` → `GATE GREEN`; §21 CHANGELOG + `docs/marley_architecture/ui_components.md`.
- At complete: create the M1.B "ui_components widgets" ticket (the deferred 5).
