---
pipeline_id: d490eb5b-8e67-4c0f-9bdb-561ad70f4837
ticket: forge#392 (2157791d-4d6b-4207-9322-0c73378c3e4f) · local docs/planning/tickets/open/TICKET-392-close-last-terminal.md
aar_id: f3583ee3-a5bf-455b-80b6-dba092bb341a
status: Phase 5 — Complete PASS
title: No-terminal + zero-tab totality — the behavior-neutral substrate for closing the last terminal
type: chore
milestone: M27
references:
  - docs/planning/pipeline/completed/391-no-terminal-totality.spec.md (the always-run half; this is the handler+zero-tab half)
  - docs/planning/tickets/open/TICKET-395-empty-workspace.md (the capstone — dissolve+UI, depends on this)
  - docs/planning/pipeline/completed/387-section-actions.spec.md (the close_tab_refusal seam #395 shrinks)
---

## Title
The **behavior-neutral substrate** for chad's "terminate the last terminal" — the D4-split's second-and-
third totality half (chad chose "split it" 2026-07-23). #391 made the ALWAYS-RUN render/pump/persist total
over a no-terminal project; #392 completes the totality so a user could reach a no-terminal AND a **zero-tab**
project WITHOUT a panic: (a) the ~15 no-terminal USER-HANDLER helpers (via #391's `try_workspace`), and (b)
the NEW **zero-tab `active_tab()` totality** — dissolving `LastTab` (in #395) makes a zero-tab project
reachable, and `active_tab()`/`active_tab_mut()` are bare indexes (`&self.tabs[self.active]`, 15+3 sites)
that panic on it. **Guards stay ON here** (behavior-neutral, test-only reachable — the #391 discipline);
**#395 flips the guards + ships the empty-workspace UI** on top of this total substrate.

## Scope
### In
This ticket is **behavior-neutral** (guards ON → the empty states are TEST-ONLY reachable; the full suite
stays byte-identical — the #391 discipline). It makes the app TOTAL over the two empty states so #395's
guard-flip is crash-safe.
- **The zero-tab `active_tab()` totality (the NEW second audit):** `active_tab()`/`active_tab_mut()` (tabs.rs,
  bare `&self.tabs[self.active]`) panic on a zero-tab project — reachable once #395 dissolves `LastTab`.
  Add a total twin (`try_active_tab()`/`try_active_tab_mut() -> Option<&(mut) Tab<S>>` via
  `self.tabs.get(self.active)`) and route the **15 `active_tab()` + 3 `active_tab_mut()` app.rs sites** that
  are reachable on a zero-tab project through it (render center dispatch, status bar, titlebar, handlers) —
  or guard them on `tab_count()>0`. Harden `terminal_grid_index`'s bare `self.tabs[self.active]`
  (tabs.rs:375) → `.get(self.active)` (the #391-forward note).
- **The no-terminal USER-HANDLER totality (~15 helpers, via #391's `try_workspace`):** the ~30 call sites
  in `391-no-terminal-totality.notes.md`'s ledger collapse to the shared helpers — splits (`split_focused_pane`)
  → **no-op**; terminal-create (`new_terminal_pane`) → **create-anyway** (`spawn_terminal_tab_in`, grid-free);
  focus-nav (`focus_neighbor`) → no-op; agent/rerun/clear/block-jump/prompt-insert/comment/history/finder
  overlays → no-op; **the terminal RAW-KEY router (app.rs:15822) → no-op** (the CRITICAL one — a keystroke
  on a cockpit tab must not route to a non-existent terminal); the deferred-input closures (app.rs
  15196/16460/16937/17011). The **6 product questions** default to the simple total (no-op/flash) — the
  behavior becomes user-reachable in #395, where chad reviews it.
- **Codec:** confirm a zero-tab project line (a project with NO tab entries) parses in `restore_shell`
  (extend #391's `restore_shell_parses_a_no_terminal_project` — reachable only in tests until #395).
- **Tests:** zero-tab / no-terminal `Project` constructors driving the totalled seams; the FULL suite
  byte-identical (guards on); the pure `try_active_tab`/codec seams at cov/MSI 100.

### Out (explicitly deferred → #395, the capstone)
- **The guard-dissolve itself** (`close_tab_refusal` shrink, the `TabError` variant + app.rs:7322 removal,
  the #387 test rewrites) → **#395**.
- **The empty-workspace UI** (the empty-center placeholder, the empty Terminal header) + the driven capture
  → **#395** (where the empty state becomes user-reachable + visible).
- Relaxing the zero-WORKSPACE model (the launcher stays the app's empty-of-workspaces state — always).
- Auto-reopening/seeding on empty (no magic respawn).

## Reference (§20)
**Convention, not app-matched.** The empty-workspace-with-hints state follows the universal IDE
convention (VS Code / JetBrains: a workspace with nothing open shows a keybinding hint panel) —
published-material leg, no source read. The rail/section behavior is Marley's own sectioned shell
(chad 2026-07-22); docs/zed_architecture/subsystems/07-workspace-panes-palette.md (research map)
notes Zed workspaces tolerate zero editors — behavior observation only. No Warp analog (Warp always
holds ≥1 session block surface — docs/warp_architecture/subsystems/03-terminal-session-core.md,
research map).

### Prior art
1. **In-repo owners:** #387's `close_tab_refusal` (the single seam to shrink — the refactor that
   makes this flip surgical); #391's totality ledger (the substrate); the #385 empty-header render
   (Editor/Browser already empty gracefully); the #247 launcher (the zero-workspace precedent + the
   boundary this must NOT cross); the #234 zero-project persist guard.
2. **Behavior maps** — the two research maps above.
3. **Published material** — IDE empty-state hint panels (convention).
4. **Permissive deps** — "none: checked gpui — no owner; this is pure Marley policy."

## Locked-In Decisions
- **D1 — Both guards dissolve** (chad, AskUserQuestion 2026-07-22): LastTerminal AND LastTab. A
  workspace may be fully empty.
- **D2 — Empty ≠ launcher:** the empty-center placeholder keeps the workspace open; the launcher
  remains zero-workspace only (#247 boundary).
- **D3 — The test rewrite is the deliverable, not collateral:** the old pins encoded the old policy;
  rewriting them IS the spec change, done loudly (changelog + notes).
- **D4 — Default-seed unchanged:** a NEW workspace still boots with one terminal — empty states are
  REACHED by closing, never by creation.
- **D5 — Variants removed, not deprecated:** no dead `TabError` arms survive (§0 no-suppressions).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the sole terminal of a multi-tab project is closed, the close shall succeed and the Terminal section shall render its empty muted header. | Rewritten truth-table unit + integration; capture. |
| REQ-002 | WHEN the last remaining tab of a project is closed, the close shall succeed and the center shall render the empty-workspace placeholder (rail keeps the workspace + empty headers); no panic anywhere. | Unit + headless drive; capture. |
| REQ-003 | An empty project shall persist and restore empty (round-trip), while a NEW workspace still default-seeds a terminal. | Codec round-trip unit + boot unit. |
| REQ-004 | `close_tab_refusal` shall refuse ONLY IndexOutOfRange; the LastTab/LastTerminal variants shall no longer exist in the codebase. | The shrunk truth-table unit (cov/MSI 100) + grep-clean at inspect. |
| REQ-005 | Closing a terminal shall still reap its PTY session (lifecycle unchanged). | Existing reaper tests remain green. |

## Phase Plan
- **P2 Design** — the empty-center placeholder's exact content/geometry; the focus/no-op audit on
  empty; the variant-removal ripple (every `TabError` consumer); confirm #391's ledger covers
  reachability.
- **P3 Implement** — tabs.rs (shrink + variant removal), app.rs (empty-center render + close-path
  reach), the test rewrite.
- **P3.5 Inspect** — critics: a missed LastTab/LastTerminal consumer (grep sweep), the empty-state's
  keyboard ladder (every chord a no-op or sensible), restore-empty vs default-seed precedence, ⌘W
  chains (close last tab → close project? — must NOT cascade).
- **P4 Validate** — the rewritten unit matrix (cov/MSI 100 on the shrunk seam) + empty-state drives +
  round-trip + `--diff` gate + captures (empty Terminal header; the empty-center hints).
- **P5 Complete** — CHANGELOG (a surfaced behavior change), app_shell.md invariant record updated
  (the M10 never-empties era formally ends), archive, close #392.
