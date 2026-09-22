# 00 — Marley Architecture Overview

> The top-level "how Marley is built" map. Read this first; then drill into the crate doc or subsystem
> reference cited in each section. This is the **Marley** doc — for the reference apps Marley reimplements,
> see the [Warp deconstruction](../warp_architecture/subsystems/00-overview.md) (terminal + cockpit) and the
> [Zed deconstruction](../zed_architecture/subsystems/00-overview.md) (editor).

**Status:** Marley @ **M15** — a runnable terminal-first workbench: a Warp-parity block-terminal, a
Zed-inspired editable editor, and an IDE-style workspace cockpit, all clean-room on `gpui`.

**Cross-links:**
[crate-map](crate-map.md) ·
[app_shell (the deep shell doc)](app_shell.md) ·
[editor](editor.md) ·
[terminal_blocks](terminal_blocks.md) ·
[ui_components](ui_components.md) ·
[settings](settings.md) ·
[CONSTITUTION](../../CONSTITUTION.md) ·
[CHANGELOG](../../CHANGELOG.md) ·
[Warp reference](../warp_architecture/subsystems/00-overview.md) ·
[Zed reference](../zed_architecture/subsystems/00-overview.md)

---

## 1. What Marley is, in one paragraph

Marley is a GPU-rendered, native desktop **terminal-first workbench** written almost entirely in Rust on
`gpui` `[permissive: gpui Apache-2.0]`. It is two products fused in one window: a **block-terminal** — a
Warp-style per-command Block model over a real PTY, with shell-integration hooks, clickable links, ghost
text, and a scrollback that reads like Warp — and an **editable editor** — a Zed-inspired, rope-backed file
editor with a live caret, typing, save, undo, selection, and clipboard. Around them sits an **IDE cockpit**:
docks, tileable panes, a command palette, a fuzzy launcher, and a Workspace → Project → Tab rail. Marley is
**not** a Warp fork of source; it is a **clean-room reimplementation** built from observed behavior on its
own **15 crates** (see [crate-map](crate-map.md)), reusing only permissive dependencies. The sellable
**brain** — the agent/AI layer that launches, controls, and observes sessions — is the intended proprietary
product (M3+, not yet built); the terminal + editor + cockpit are the intended **open-core** tier. See §7.

## 2. The layer stack

Bottom-up: permissive substrate → Marley value crates → the UI-agnostic subsystems → the one gpui shell.
The **brain** attaches at the top across an arm's-length boundary (§7), never linked into the copyleft tiers.

```
  ═══ THE BRAIN  (M3+, [Marley-original], PROPRIETARY — not built) ══════════════════════
      direct model-provider streaming; launches / controls / OBSERVES sessions.
      Talks to the tiers below ONLY across an arm's-length seam — the EditOrigin::Agent
      write, session.read/write, MCP, IPC — never by linking their code. (§7)
  ─────────────────────────── arm's-length protocol boundary ──────────────────────────────
┌─ marley_app  — the cockpit shell (RootView)                            [Marley-original] ─┐
│   workspace→project→tab · docks · PaneGroup split/close/resize · command palette ·         │
│   keymap chords · launcher · editor surface · Blocks render · prompt input · viewport      │
│   ~30 PURE modules (cov/MSI 100)  +  ONE gpui shim: app.rs Render (mutants::skip, gate-15) │
├───────────────────┬───────────────────┬──────────────────┬────────────────────────────────┤
│ terminal_blocks   │ editor            │ ui_components     │ marley_settings · _project ·    │
│ [Warp-derived]    │ (marley_editor)   │ [Marley-original] │ _search_core · _agent ·         │
│ Block model over  │ [Marley-orig; Zed │ Theme/ThemeColors │ _remote                         │
│ the alac grid +   │  reference] Buffer │ + 5 cockpit       │ typed TOML · repo/file tree ·   │
│ shell DCS/OSC     │ caret · undo ·    │ widgets · WCAG    │ fuzzy · agent obs ·             │
│ hooks · links     │ EditOrigin{H,A}   │ contrast          │ ssh-as-a-pane                   │
├───────────────────┴───────────────────┴──────────────────┴────────────────────────────────┤
│ marley_command      · marley_core        · marley_util       · marley_text_offsets           │
│ argv process adapter  non-cloud substrate   value types         CharOffset/ByteOffset vocab   │
├───────────────────────────────────────────────────────────────────────────────────────────┤
│ REUSE (permissive, never rewritten):  gpui (Apache-2.0) · alacritty_terminal + vte (Apache)  │
│   · ropey (MIT) · nucleo (MIT) · serde/toml · rust-embed   — attribution kept, used freely    │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

Reading the tiers from the task's stated spine — **gpui render → alacritty PTY + Blocks → the editor → the
cockpit shell → agents**:

- **gpui render** `[permissive: gpui Apache-2.0]` — elements, windows, focus, text shaping, the retained
  scene → Metal. Marley's only GPU dependency; adopted directly (Marley does **not** use Warp's `warpui`).
- **PTY + Blocks** — `terminal_blocks` spawns a shell over `alacritty_terminal`, reads its grid, and
  segments output into per-command **Blocks** from shell **DCS/OSC hook** metadata (cwd, git branch, exit
  code) rather than output heuristics. The `BlockList` is the unit any front-end — and the brain — observes.
  See [terminal_blocks](terminal_blocks.md).
- **the editor** — `marley_editor` is a UI-agnostic, `ropey`-backed `Buffer` with a caret, coalesced undo,
  selection, and movement. Every range edit carries an **`EditOrigin { Human, Agent }`** — the write-
  provenance seam the brain later uses to tell human typing from streamed agent writes. See [editor](editor.md).
- **the cockpit shell** — `marley_app` is the top-of-graph window: it composes the terminal, the editor, and
  the panels into the Workspace → Project → Tab model (§5), and holds nearly all UI **decision logic** in pure
  modules behind one gpui Render shim (§4). See [app_shell](app_shell.md).
- **agents** — `marley_agent` (agent-cockpit observability) and `marley_remote` (ssh-as-a-terminal-pane)
  are the cockpit-side slices of the future brain; the brain itself is out of tree (§7). A third slice,
  `marley_forge_client` (the read-only app→forge sidecar seam), was **deleted** at #411 — the
  2026-08-09 scrap-forge product rip.

## 3. The crate structure

14 workspace crates (`crates/*`, workspace license `MIT OR Apache-2.0`; a 15th, `marley_forge_client`, was deleted at #411). The knowledge graph — each crate's
Warp lineage, its triage bucket, and the permissive crate underneath — is [crate-map](crate-map.md).

| Crate | Role | Provenance | Doc |
|---|---|---|---|
| `marley_app` | the runnable window — RootView + all cockpit logic | `[Marley-original]` on gpui | [app_shell](app_shell.md) |
| `terminal_blocks` (`marley_terminal`) | PTY session + per-command Block model + DCS/OSC hooks | `[Warp-derived]` · `[permissive: alacritty_terminal]` | [terminal_blocks](terminal_blocks.md) |
| `editor` (`marley_editor`) | rope Buffer, caret, undo, selection, `EditOrigin` | `[Marley-original]`; Zed reference · `[permissive: ropey]` | [editor](editor.md) |
| `ui_components` | Theme/ThemeColors vocabulary + 5 cockpit widgets + WCAG | `[Marley-original]` on gpui | [ui_components](ui_components.md) |
| `marley_settings` | typed declarative TOML settings framework | `[Marley-original]` · `[permissive: serde/toml]` | [settings](settings.md) |
| `marley_project` | project discovery (git-root walk-up) + file tree | `[Marley-original]` | [marley_project](marley_project.md) |
| `marley_search_core` | fuzzy-search seam (`fuzzy_score`/`fuzzy_rank`) | `[Marley-original]` · `[permissive: nucleo]` | [marley_search_core](marley_search_core.md) |
| `marley_agent` | agent-cockpit observability slice (status/run model) | `[Marley-original]` | [marley_agent](marley_agent.md) |
| `marley_remote` | ssh-as-a-terminal-pane seam (handles no secrets) | `[Marley-original]` | [marley_remote](marley_remote.md) |
| `marley_command` | argv (never shell) process-spawn adapter | `[Warp-derived]` · `[permissive: std/async-process]` | [marley_command](marley_command.md) |
| `marley_core` | non-cloud substrate (config dir, channel config) | `[Marley-original]` | [marley_core](marley_core.md) |
| `marley_util` | value types (paths, dirs) | `[Marley-original]` · `[permissive: serde]` | [marley_util](marley_util.md) |
| `marley_text_offsets` | the `CharOffset`/`ByteOffset` type vocabulary | `[Marley-original]` | (see [crate-map](crate-map.md)) |
| `marley_visual_harness` | gate-15 headed capture / AX test tooling | `[Marley-original]` tooling | [marley_visual_harness](marley_visual_harness.md) |

## 4. The pure / shim discipline + the gate & pipeline

Marley's signature architecture pattern is the **pure / shim seam**, present in every crate:

- **PURE** — all logic (parsing, layout math, state algebra, the Block/Buffer models, theme calibration) is
  written **gpui-free and OS-free**, unit-tested to **100% line coverage and 100% mutation MSI** (`cargo
  llvm-cov` + `cargo-mutants`). No baselines, no suppressions, source-fix only.
- **SHIM (ACCEPTED-UNTESTABLE)** — the thin, genuinely untestable edge (the gpui `Render`, the PTY spawn, an
  `fs::write`, a `git diff` adapter) is marked `#[mutants::skip]`, coverage-excluded on an **explicit,
  documented** list, and asserted by the **headed visual/AX harness** (gate-15) — pixels captured off the
  live window by `CGWindowID`, driven with synthetic input. Markup rendering alone never proves a pane works.

So a typical feature is a small pure seam (e.g. `viewport::scrollbar_thumb`, `links::scan_links`,
`code_view::line_layout`, `tabs::rail_rows`) carrying the whole mutation burden, wired into the app by a
byte-thin masked shim. This is what keeps a GPU app at a 100/100 bar.

Every change ships through the enforced **phase-gate pipeline** — `/work → plan → design → implement →
inspect → validate → complete → commit` — where `inspect` runs adversarial critics that verify by running
commands, and `complete` (§21) updates the CHANGELOG + these architecture docs. The canonical truth gate is
`scripts/gates.sh` (15 gates: fmt, clippy, tests, coverage, mutation, miri, audit, deny, secrets, visual/AX,
…), which writes the receipt the commit hook requires. The bar and the process are law in the
[CONSTITUTION](../../CONSTITUTION.md) (§0 gates, §3 phases, §7 testing, §20 clean-room, §21 docs).

## 5. The workspace → project → tab model

The shell is a three-level hierarchy of **full-screen tabs** (M9), re-cast toward a multi-workspace IDE in
M13:

- **Workspace** — the top-level container (one per window today), holding one or more projects. A
  **launcher** (#234) makes a zero-project state reachable → a PhpStorm-style "Open a workspace" landing.
- **Project** — a repo root (discovered by git-root walk-up) + its tabs. Switching projects re-targets Files,
  the finder, git, and the titlebar from one place (`sync_active_project`). The left-dock **rail** renders the
  Workspace → Project → Tab → Pane tree with collapse state + a focused-project highlight.
- **Tab** — full-screen, one of: a **terminal** (owning a `PaneGrid` — split / close / resize / directional
  focus via the pure `PaneGroup` algebra + `pane_rects` tiling), a **cockpit** section (Details / Agents —
  the Forge section retired at #411), or the **editor surface** (an N-file tab strip; files open here, not as rail clutter). A terminal
  can also split-right into a read-only file pane.

**Terminology note (M13, pinned, rename deferred):** in chad's IDE framing, a "workspace" ≈ today's
**Project** promoted to the top-level focusable unit, and the singleton container becomes the "Session." The
code still uses the M9 names (`Workspace → Project → Tab`) because a `tabs::Project → Workspace` rename
collides with `marley_project::Project` across ~75 sites; it is a dedicated later pass. See
[app_shell §M9/§M13](app_shell.md).

## 6. The data-flow spine

Two directions, framework-wide: **state flows down** (a gpui `Render` reads model state each frame); **events
flow up** (an element handler mutates state → `cx.notify()` → re-render). The terminal path:

```
key → on_key_down (app.rs)                                          [marley_app shim]
  ├─ ⌘-chord?      → keymap::action_for → dispatch_action (palette/split/save/…)
  ├─ editor tab?   → input::apply_editor_key(&mut Buffer, &mut caret)   [marley_editor]
  ├─ alt-screen?   → encode_key → TerminalSession::write_bytes → PTY    [terminal_blocks]
  └─ cooked?       → input::apply_key over the prompt Buffer, submit on Enter
submit → TerminalSession::write_command → shell (zsh) over the PTY leader fd
shell stdout → alac grid → 16ms pump reads it → shell DCS/OSC hooks attach
  cwd/exit/git → the BlockList  →  cx.notify()  →  RootView::render paints Blocks
```

The **16ms pump** (a gpui timer) drives every pane's session, auto-closes exited shells, and clocks the
action-confirmation flash. The editor path is the same router, minus the PTY: keys edit the file `Buffer`,
`⌘S` writes it to disk, the dirty ● tracks `version != saved_version`. Both paths keep every layout/state
**decision** in a pure module; only the `Render` and the byte I/O are shim.

## 7. Open-core licensing + the brain boundary

Marley is owned outright — its own code, permissive deps only (gate-8 allows MIT / Apache / BSD / ISC /
Unicode / Zlib; copyleft rejected at the dependency layer). The **release posture** layers copyleft on top of
that by design, mirroring the two reference apps:

- **Terminal + cockpit = `[Warp-derived]`.** Built clean-room from observed Warp behavior, but a close
  behavioral reimplementation of an **AGPL-3.0** work carries conservative AGPL exposure. Intended posture:
  **AGPL the terminal + cockpit** (open-core, exactly as Warp itself is), keep the Warp attribution. See the
  [Warp overview](../warp_architecture/subsystems/00-overview.md).
- **Editor = `[Marley-original]`, Zed-referenced.** The M15 editable editor is Marley's structural win (Warp
  has no file editor). Its behavioral reference is **Zed (GPL-3.0)**; the editor tier's intended posture is
  GPL. See the [Zed overview](../zed_architecture/subsystems/00-overview.md).
- **The brain stays sellable via one boundary.** The sold brain must be a **separate program** —
  `[Marley-original]`, containing **no** Warp- or Zed-derived code — talking to the copyleft tiers only across
  an **arm's-length protocol boundary** (separate process / IPC / the `EditOrigin::Agent` write seam /
  `session.read`-`write` / MCP), never by linking their code in. **Mere aggregation over a wire is not a
  derivative work; linking is.** The hazard the editor's GPL does not have is **AGPL §13** (network-use
  copyleft): if the brain *incorporates* AGPL-derived terminal/cockpit code and is reached over a network, §13
  attaches copyleft even with no binary shipped — and the open-core model collapses. The **`EditOrigin
  { Human, Agent }`** seam already in `marley_editor` is exactly that boundary; keep the brain on its own side.

**Clean-room (§20) is the enforcement.** Observe behavior and use gpui + permissive deps freely; **never**
read or translate the AGPL/GPL source. Every pipeline spec fills a `## Reference (§20)` section (Warp for
terminal/cockpit/UX, Zed for the editor) that a commit hook checks, and the inspect phase runs a code-layer
provenance review. Docs tag capabilities `[Warp-derived]` / `[Zed-derived]` / `[permissive: <dep>]` /
`[Marley-original]` so the eventual legal review is mechanical. IP-counsel sign-off is a pending release gate.

## 8. Milestone spine (what shipped)

| Milestone | Delivered |
|---|---|
| **M0** | foundation crates (offsets, util, core, command, project, search, agent, forge-client *(deleted at #411)*, harness) |
| **M1** | the terminal MVP — app shell, block terminal, editor input buffer, 5 widgets, settings, keymap, palette, docks, PaneGroup |
| **M4–M8** | code/diff viewer · the Warp cockpit layout · top bar · real SVG icons, unified titlebar, cwd+branch |
| **M9** | the Workspace → Project → Tab full-screen-tab model + the rail |
| **M10–M12** | Warp polish + **Warp parity** (density, block hover, block cursor, sidebar wash, keycaps) · clickable `file:line:col` links (#196) + OSC 8 hyperlinks (#214) · live theme picker · ghost text · scrollbar/jump-to-bottom |
| **M13** | the **Workspace Cockpit** — multi-workspace direction, PhpStorm-style **launcher**, collapsible rail + focused highlight, the multi-file **editor surface** |
| **M14** | cockpit polish (dark default, disambiguated tabs, editor label) + session/layout/file/collapse persistence |
| **M15** | the **editable editor** (#249–258) — doc model, faithful renderer, input intercept, ⌘S + dirty ●, coalesced undo, click-to-caret, selection, copy/cut/paste, caret-motion parity, split-file persist |

Full detail per change is in the [CHANGELOG](../../CHANGELOG.md); full shell detail is in [app_shell](app_shell.md).

## 9. Where to read next

| Doc | Covers |
|---|---|
| [crate-map](crate-map.md) | the crate knowledge graph — lineage · bucket · REUSE crate |
| [app_shell](app_shell.md) | the deep, authoritative shell doc — every pure module + the one shim, M1→M15 |
| [terminal_blocks](terminal_blocks.md) · [editor](editor.md) · [ui_components](ui_components.md) · [settings](settings.md) | the core subsystem crates |
| [CONSTITUTION](../../CONSTITUTION.md) | the binding quality bar, phase gates, clean-room, and doc discipline |
| [Warp reference](../warp_architecture/subsystems/00-overview.md) · [Zed reference](../zed_architecture/subsystems/00-overview.md) | the deconstructions Marley reimplements + the provenance posture |
| [crate-triage](crate-triage.md) · [byo-agent-protocol](byo-agent-protocol.md) · [icon-audit](icon-audit.md) | the Warp-crate verdicts · the future agent protocol · the emoji→icon audit |
