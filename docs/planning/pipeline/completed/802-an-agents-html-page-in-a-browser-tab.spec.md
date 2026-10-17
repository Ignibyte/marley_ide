---
pipeline_id: be71c2e1-c737-4a6f-a31a-ccedde1f3d45
ticket: docs/planning/tickets/closed/TICKET-802-an-agents-html-page-in-a-browser-tab.md
status: Phase 4 — Complete PASS
title: An agent's HTML page in a Browser tab
type: feature
slice: Browser service (agent components)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
  - docs/planning/pipeline/completed/586-file-pages-in-the-browser-tab.spec.md
---

## Title
`browser_show_page` shows an agent's HTML page in a Browser tab of its project: a page it hands
over, written to a scratch folder Marley keeps, or an HTML file already in the project. The tab
reloads when the file changes, and a stylesheet of Marley's theme lets the page look like Marley.

## Scope
### In
- **The tool,** in the browser family (`marley_mcp/src/registry.rs`, write tier as
  `browser_open_url` is): `html` and a `name`, or `path` (an `.html` or `.htm` file inside the
  caller's project); never both.
  - With `html`: Marley writes `<name>.html` into the project's scratch folder under Marley's
    data directory (`pages/<project>/`), adds the theme stylesheet's `<link>` to its `<head>`
    unless `theme: false`, and opens it.
  - With `path`: Marley opens the file as it is and writes nothing.
  - Either way the page opens in a Browser tab of the caller's project, bringing forward the
    tab already on that file, as `browser_open_url` does for a local file (#586). The answer
    names the tab, the file's path and the stylesheet's `file:` URL.
- **Reload:** while a tab the tool opened still shows its file, a change to the file reloads the
  tab.
- **The stylesheet** (`pages/marley-theme.css` in Marley's data directory): the active theme's
  colors and Marley's UI and monospace fonts as CSS variables (`--marley-background`,
  `--marley-surface`, `--marley-text`, `--marley-text-muted`, `--marley-border`,
  `--marley-accent`, `--marley-success`, `--marley-warning`, `--marley-error`, `--marley-font`,
  `--marley-mono-font`) and `color-scheme`, rewritten when the theme changes; pages the tool
  opened reload then.
- The tool's description tells an agent when to choose a page over `component_show` (#800):
  for what the catalog cannot draw.

### Out (explicitly deferred)
- MCP Apps (`ui://` resources, the sandboxed iframe and its host bridge): the intake doc.
- Writing into the project: an agent writes project files with its own tools and passes `path`.
- Pruning the scratch folder.

## Reference (§20)
Upstream Zed: none (Zed has no browser). Marley's own #586 (a local HTML page in a Browser tab)
and #561 (the `BROWSER` opener) are what this extends. Published material: MCP Apps
(SEP-1865) shows an agent's HTML in the host; Marley shows it in its own Browser tab, with the
user's theme offered as CSS variables.

### Prior art
- The code we ship: `browser_open_url`'s local-file path (#586) opens a file in the project's
  Browser tab and reuses one already on it; `Fs::watch` for the reload; the Browser tab's CDP
  session (`Page.reload`); `theme::ActiveTheme` and `ThemeSettings` for the colors and fonts;
  `paths::data_dir()` for the scratch folder.
- Behavior maps: none.
- Published material: MCP Apps' theming (a host passes theme tokens to the app), the design
  note.

## UI proof
Scenario `script/e2e/802-an-agents-html-page-in-a-browser-tab.sh`, under `compositor sway` (the
Browser tab), with a terminal stand-in that calls the tool through the bridge:
- `802-01-scratch`: `html` with a page whose body uses `var(--marley-surface)` and
  `var(--marley-text)`: a Browser tab shows it in the theme's colors (REQ-001, REQ-003).
- `802-02-reload`: the stand-in calls again with a changed page under the same `name`: the same
  tab shows the change, and no second tab opened (REQ-002).
- `802-03-path`: `path` to a project file; the stand-in then rewrites the file with `sed`: the
  tab reloads to the new text (REQ-002, REQ-004).
- `802-04-theme`: the theme switched to a light one: the page reloads in the light colors
  (REQ-003).
- `802-05-outside`: `path` outside the project: `opened: false` and why, no tab (REQ-005).

## Locked-In Decisions
- **D1 — Marley writes only its scratch folder.** A project file is the agent's to write.
- **D2 — Theme by CSS variables in a linked stylesheet**, so a page that ignores them still
  works and one that uses them follows the theme.
- **D3 — A Browser tab, not inline**: the Browser is a screencast tab and cannot sit in a thread
  row.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `browser_show_page` is called with `html` and a `name`, Marley shall write the page to the project's scratch folder and show it in a Browser tab of the caller's project | shot `802-01-scratch` |
| REQ-002 | WHEN the file of a page the tool opened changes, its tab shall reload, and a call for a page already open shall bring forward its tab | shots `802-02-reload`, `802-03-path` |
| REQ-003 | The stylesheet shall carry the active theme's colors and fonts, and WHEN the theme changes, the tool's pages shall reload in the new colors | shots `802-01-scratch`, `802-04-theme` |
| REQ-004 | WHEN called with `path` to an HTML file in the project, Marley shall show that file and write nothing | shot `802-03-path`; review of the diff |
| REQ-005 | IF `path` is outside the caller's project or not an HTML file, THEN the tool shall answer `opened: false` with the reason | shot `802-05-outside` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, the scratch folder is `pages/<project key>/`, the key
  that names the project's browser folder, and the stylesheet is kept current by an observer of
  the active theme. Stylesheet: `pages/marley-theme.css`, beside the project folders.
- **P2 Code:** the tool, the writer, the stylesheet, the reload; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md`, ledger, close, archive, commit.
