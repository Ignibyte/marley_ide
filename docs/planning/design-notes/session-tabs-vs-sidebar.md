# Design note — terminal session tabs vs the sidebar (M8 #143 SPIKE)

- **Ticket:** forge #143 `0b72dbcb-5461-41b7-bf4b-8f27fea2a58f` (spike) · **AAR:** `912f9291-5c4f-4fea-82b2-e31f0393766d`
- **Question:** chad's recurring "tabs at the top" — should Marley add a horizontal terminal-tab strip
  (Warp-style), and how would tabs interact with split panes / the sidebar?
- **Method:** compared Marley's current model against chad's **actual Warp** (3 reference screenshots,
  `~/Downloads/Screenshot 2026-07-06 at 12.09.30/12.09.49/12.10.39 PM.png` — a live Warp session on Marley).

## What Warp actually does (from the references)
1. **Left sidebar = the session list, grouped by project.** Vertical rows grouped under project headers
   (`Rusty`, `Forge`, `Dev 1`, `Dev 2`, `Dev 3`, `UCSOS Dev`, `Ignibyte IDE`, `AIC`). Each row: an icon
   (a `❯` prompt or an active dot), a **live title = the running command / open file** (e.g. "Create tickets
   for next", "clippy.toml"), and a subtitle (branch `⑂ main` or cwd `~`). The active session is highlighted.
2. **Tiled panes with per-pane title bars.** The main area tiles panes side-by-side — a file explorer, the
   terminal, a code viewer, a git panel — each with a **title bar** carrying the pane title + a `⋮` + a `×`.
3. **A centered top search** ("Search sessions, agents, files…") in the traffic-light row; cwd·branch in the
   footer; **no separate horizontal top-tab strip.**

## The key finding
**Marley already implements Warp's model.** The left dock is the session list grouped by workspace
(`sessions::session_rows` / `SessionGroup`, M5 #109/#110); panes tile via the `PaneGroup` tree with per-pane
title bars + a close `×` (M6/M7 #120/#132); the centered search (#117) and cwd·branch titlebar (#142) match.
**Warp has no horizontal top-tab strip** — so "tabs at the top" is already satisfied by (a) the vertical
sidebar sessions and (b) the per-pane title bars. Building a redundant horizontal tab strip would *diverge*
from Warp, not converge.

## Where Marley actually differs from Warp (the real gaps)
| # | Warp | Marley today | Gap |
|---|------|--------------|-----|
| G1 | Sidebar row title = the **live running command / open file** | Static `"terminal N"` | titles feel dead |
| G2 | Sidebar row subtitle = the **real branch** | Hardcoded `"main"` (app.rs ~2221) | latent bug (noted in #142) |
| G3 | Clicking a sidebar row **switches focus** to that session; the active row is highlighted | Rows render but aren't the focus control | sidebar isn't a "tab bar" |
| G4 | Sidebar groups = **multiple projects/workspaces** | One project (the launch cwd) | no multi-project |

## Options considered
- **A — Add a horizontal top-tab strip.** Diverges from Warp (which has none); competes with the sidebar +
  the per-pane title bars for the same job; costs top-bar space already holding icons/search/cwd. **Rejected.**
- **B — Replace the sidebar with top tabs.** Throws away a Warp-faithful, already-shipped surface. **Rejected.**
- **C — Keep the sidebar model (Warp-faithful) and make it a proper tab bar.** Close G1–G3: live titles,
  real branch, click-to-focus + active highlight. Defer G4 (multi-project) as a larger milestone. **Recommended.**

## Recommendation
**Option C.** Marley's sidebar + tiled-pane model is the correct, Warp-faithful shape — do **not** add a
horizontal tab strip. Invest instead in making the sidebar behave like Warp's session tabs (live titles,
real branch, click-to-switch, active highlight). Treat multi-project workspaces (G4) as a separate, larger
effort (its own milestone), since it changes `Workspace` from one `PaneGroup` to many.

## Follow-up tickets (created — backlog for chad to prioritize)
- **F1 → forge #145** — Sidebar rows: **click-to-focus + active-row highlight** (session switcher). [G3]
- **F2 → forge #146** — Sidebar row **live title** = running command / cwd (not "terminal N"). [G1]
- **F3 → forge #147** — Sidebar row **real branch** subtitle from `.git/HEAD` (reuse
  `titlebar::branch_from_git_head`), replacing the hardcoded "main". [G2]
- **F4 → forge #148** (larger, own milestone) — **multi-project workspaces**: `Workspace` holds many
  `PaneGroup`s with an active index; sidebar groups = projects; a project switcher. [G4]

## Confirm with chad
If "tabs at the top" specifically meant *literal horizontal tabs* (a deliberate preference beyond Warp),
that's Option A — revisit only on explicit request. This note assumes the goal is **Warp fidelity**, which
the sidebar model already delivers.
