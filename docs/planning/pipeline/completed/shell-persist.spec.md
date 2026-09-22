---
pipeline_id: 6edddc50-e007-4e3c-90ba-a061c584b8cf
ticket: forge#163 (657b8a5d-eee0-4c0e-a04b-c43147c33727) · local docs/planning/tickets/open/TICKET-163-shell-persist.md
aar_id: f2a6ffe7-388f-4632-b484-39b429f4a96d
status: Phase 5 — Complete PASS
title: M10 — persist the workspace shell (projects + tabs)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/grid_layout.rs (PURE: ShellLayout/ProjectLayout/TabLayout + serialize/restore_shell)
  - crates/marley_app/src/settings.rs (PURE: the workspace.shell setting + persist_shell)
  - crates/marley_app/src/app.rs (SHIM: build_shell_layout + the boot restore + the persist piggyback)
---

## Title
The whole shell survives a relaunch — projects (roots), their tabs (terminal grids by kind, cockpit sections,
code files), and the active indices. Previously only the boot tab's pane kinds persisted.

## Scope
### In
- PURE codec (`grid_layout.rs`): `ShellLayout`/`ProjectLayout`/`TabLayout`; `serialize_shell`/`restore_shell`
  (line 1 = active project; a `root \t active_tab \t entries…` line per project; entries `T=<grid-blob>` /
  `C=<section>` / `V=<path>`; malformed → skipped, empty → default).
- Settings: `workspace.shell` (String, "") + `AppliedSettings.shell` + `persist_shell`.
- SHIM: `build_shell_layout` + `persist_shell_state` (piggybacked on every `persist_grid` call site); the
  boot restores the full shape when the blob is non-empty (missing roots skipped; terminals respawn per the
  embedded grid-kind blobs with fresh #167 id blocks; code tabs re-read through the #154 guards, missing →
  dropped; an emptied project gets one terminal; actives clamped) — else the legacy single-grid path.

### Out
- Restoring scrollback/command history (PTYs respawn fresh). The Files-panel open state. Window geometry.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the codec EMBEDS the existing per-grid kinds blob (no \t/\n in it) — one codec per concern.
- D2 — paths containing \t/\n are not representable; such a record is skipped on restore (graceful).
- D3 — legacy back-compat: an empty shell blob boots the old single-grid path unchanged.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `serialize_shell` → `restore_shell` shall round-trip a 2-project, mixed-tab layout including actives; empty/malformed input shall yield the default / skip bad lines. | unit |
| REQ-002 | `workspace.shell` shall round-trip through the settings store (default ""). | unit |
| REQ-003 (visual) | WHEN the shell holds extra tabs (a split terminal, a cockpit tab) and is relaunched, the rail shall show the same shape (tabs, nested panes, the cockpit tab, actives). | driven capture |
| REQ-004 | A missing project root / code file shall be skipped without panic; an emptied project shall boot one terminal. | unit (codec) + critic (shim guards) |
| REQ-005 | gate GREEN; the codec + setting at cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (restore-loop guards: missing root/file, empty project, active clamp,
fresh id blocks, legacy fallback). P4 tests + the relaunch capture + gate. P5 docs + close the sprint's goal.
