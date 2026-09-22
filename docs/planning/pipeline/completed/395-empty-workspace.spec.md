---
pipeline_id: 7d3a6fae-6e21-4a72-bb8c-aae3abfa833c
ticket: forge#395 (51abebc2-7d0e-43b9-8161-272fdbce5401) · local docs/planning/tickets/open/TICKET-395-empty-workspace.md
aar_id: 6ede7abc-8fc3-44b6-837b-1bf5744703df
status: Phase 5 — Complete PASS
title: The empty workspace — dissolve the never-empties guards + the empty-center UI (the #392 capstone)
type: feature
milestone: M27
references:
  - docs/planning/pipeline/completed/392-close-last-terminal.spec.md (the totality substrate this flips onto — HARD DEP, shipped 3f18f01)
  - docs/planning/pipeline/completed/387-section-actions.spec.md (close_tab_refusal — the pure seam to shrink)
  - docs/planning/pipeline/completed/391-no-terminal-totality.spec.md (the try_workspace contract + the codec)
---

## Title
The small, VISIBLE capstone of chad's "terminate the last terminal" (the D4-split of #392; chad chose
"split it" 2026-07-23). #392 made the app TOTAL over the two empty states (zero-tab AND no-terminal) with
the guards STILL ON — a crash-safe substrate. **#395 flips the guards and ships the empty-workspace UI on
top of it:** a workspace may now be fully empty, and the empty state renders muted hints instead of a
panic. This is where the M10 "never-empties" era (a project always holds ≥1 terminal tab) formally ends,
and where **chad eyeballs the empty-workspace UX.**

## Scope
### In
1. **Dissolve BOTH never-empties guards.** `close_tab_refusal` (tabs.rs, the #387 pure truth table)
   shrinks to **IndexOutOfRange-only**; the `TabError::LastTab` and `TabError::LastTerminal` variants and
   their SOLE production consumer (app.rs's "can't close the last terminal" status flash,
   `Err(LastTerminal | LastTab) => …`) are **REMOVED** (§0 — no dead variants; the remaining
   `IndexOutOfRange` arm / `Err(_) => {}` catches the rest). An in-range close now always succeeds.
2. **Deliberately REWRITE the #387-pinned guard tests** (D3 — the rewrite IS the spec change, done
   loudly + changelog'd): `close_tab_refusal_truth_table` (its R4 `LastTab` + R6 `LastTerminal` rows →
   the new table: in-range close always `Ok`; zero-terminal + zero-tab reachable), `close_last_terminal_guard`
   (the M10-era guard), and any `close_tab_section_vocab` row that encoded the old refusal.
3. **The empty TERMINAL section header** — parity with the #385 empty Editor/Browser muted header (likely
   free once the guard is gone; the section renders its header with no rows).
4. **The empty-workspace CENTER placeholder** — FILL #392's `tab_count() == 0 → blank center` branch with
   muted hints (**new terminal ⌘T · open file ⌘P · the section ＋s**). NOT the launcher (D2 — #247 is the
   zero-WORKSPACE state; an open-but-empty workspace STAYS open with its rail + empty section headers).
5. **Persistence:** an empty project round-trips — a project line with zero tab entries restores empty (on
   #391's codec + #392's `restore_shell_parses_a_zero_tab_project`); the NEW-project default-seed is
   UNCHANGED (D4 — a new workspace still boots with one terminal; empty is REACHED by closing, never by
   creation).
6. **The PTY reaper on terminal close is UNCHANGED** — the session dies with the tab, off-thread (the
   ~600ms deferred `thread::spawn(drop)`), exactly as today.

### Out (explicitly deferred)
- Relaxing the zero-WORKSPACE model — the launcher (#247) remains the app's empty-of-workspaces state.
- Auto-reopening / seeding on empty (no magic respawn — an empty workspace stays empty until the user acts).
- The ContentId-registry terminal migration (#396) — orthogonal.

## Reference (§20)
**Convention, not app-matched.** The empty-workspace-with-hints state follows the universal IDE
convention (VS Code / JetBrains: a workspace with nothing open shows a keybinding hint panel) —
published-material leg, no source read. The rail/section behavior is Marley's own sectioned shell
(chad 2026-07-22); docs/zed_architecture/subsystems/07-workspace-panes-palette.md (research map)
notes Zed workspaces tolerate zero editors — behavior observation only. No Warp analog (Warp always
holds ≥1 session block surface — docs/warp_architecture/subsystems/03-terminal-session-core.md,
research map).

### Prior art
1. **In-repo owners (decisive):** #387's `close_tab_refusal` (the single seam to shrink — the refactor
   that makes this flip surgical); **#392's totality substrate** (the `try_active_tab`/`try_workspace`
   twins that make a zero-tab / no-terminal project crash-safe + the `tab_count()==0 → blank center`
   branch this ticket FILLS); the #385 empty-header render (Editor/Browser already empty gracefully);
   the #247 launcher (the zero-workspace precedent + the boundary this must NOT cross); the #234
   zero-project persist guard.
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
| REQ-001 | WHEN the sole terminal of a multi-tab project is closed, the close shall succeed and the Terminal section shall render its empty muted header. | Rewritten truth-table unit + integration; driven capture. |
| REQ-002 | WHEN the last remaining tab of a project is closed, the close shall succeed and the center shall render the empty-workspace placeholder (rail keeps the workspace + empty headers) with no panic anywhere. | Unit + headless drive; driven capture. |
| REQ-003 | An empty project shall persist and restore empty (round-trip), while a NEW workspace still default-seeds a terminal. | Codec round-trip unit + boot unit. |
| REQ-004 | `close_tab_refusal` shall refuse ONLY IndexOutOfRange; the `LastTab`/`LastTerminal` variants shall no longer exist anywhere in the codebase. | The shrunk truth-table unit (cov/MSI 100) + a grep-clean check at inspect. |
| REQ-005 | Closing a terminal shall still reap its PTY off-thread (lifecycle unchanged). | The existing reaper tests remain green. |

## Phase Plan
- **P2 Design** — the empty-center placeholder's exact content/geometry (the hint rows in #392's blank
  branch); the `TabError` variant-removal ripple (EVERY consumer — grep `LastTab`/`LastTerminal`); confirm
  #392's substrate covers reachability (a zero-tab close can't panic — the twins); the empty Terminal
  header's render path; the ⌘W-chain edge (close last tab → does it cascade to close the project? must NOT
  — D2).
- **P3 Implement** — tabs.rs (`close_tab_refusal` shrink + the variant removal + the rewritten guard
  tests), app.rs (the empty-center hint render + the app.rs:7322 consumer removal + the empty Terminal
  header).
- **P3.5 Inspect** — critics: a missed `LastTab`/`LastTerminal` consumer (grep sweep); the empty-state
  keyboard ladder (every chord a no-op or sensible on an empty workspace — #392 made it total, re-confirm);
  restore-empty vs default-seed precedence; the ⌘W chain (close-last-tab must NOT cascade to close-project).
- **P4 Validate** — the rewritten unit matrix (cov/MSI 100 on the shrunk `close_tab_refusal`) + the
  empty-state drives + the round-trip + `--diff` gate + the **DRIVEN CAPTURE** (the #390 pre-seed
  technique: pre-seed an isolated HOME's settings.toml `workspace.shell` with a cockpit-only / zero-tab
  project → launch → `screencapture` → READ the empty-center hints + the empty Terminal header). chad
  eyeballs the empty-workspace UX.
- **P5 Complete** — CHANGELOG (a surfaced behavior change — the M10 never-empties era ends);
  app_shell.md invariant record updated; archive; close #395.
