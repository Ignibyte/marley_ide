---
pipeline_id: d9f2681a-082f-410d-9e79-15f66ed46b48
ticket: forge#382 (d145a213-f2e9-43c2-b6e0-79836bfc87e0) · local docs/planning/tickets/open/TICKET-382-status-bar-focus-label.md
aar_id: 3813a196-461c-49b3-9792-69ca9c9b6ccf
status: Phase 5 — Complete PASS
title: Status bar focus label — derive from the focused pane's kind, not a hardcoded "terminal"
type: bug
milestone: M25
references:
  - crates/marley_app/src/status_bar.rs
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/workspace.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/titlebar.rs
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
---

## Title
Make the footer's focus segment tell the truth. The 2026-07-21 QA run (capture 14-editor.png)
caught it: click into a `workspace.rs` editor split inside a terminal tab's grid — the #191 accent
focus border moves to the editor pane and the rail flips to the pane-2 row, but the footer keeps
reading `focus: terminal`. The verified mechanism is NOT quite the ticket's "derives from the TAB
kind" hypothesis — it is tab-kind-INVARIANT: the render shim's `footer_focus` (app.rs:18408-18414)
resolves the focused `PaneId` (`workspace_mut().focused()`, app.rs:14775) against `self.agents`
(→ the run's label), then `self.remotes` (→ the host), then falls to a **hardcoded `"terminal"`
string literal**. It never consults the focused pane's kind (`PaneState::kind`,
workspace.rs:361-368) or the active tab's content (`TabContent`, tabs.rs:23-31). Consequences,
all real today: a focused editable code pane (`PaneContent::CodeView(EditorSurface)` — the M15
#259 recast behind the #355/#357 editable-split lineage, workspace.rs:325-329) reads "terminal";
so do focused files/git panels; and with an editor or cockpit TAB active the shim reads a
BACKGROUND grid entirely (`workspace()` falls back to the first terminal tab's grid,
app.rs:5288-5294) — it can even leak a background agent pane's label while an editor tab holds
every keystroke. Fix: a pure `focus_label` derivation in the already-pure `status_bar.rs` — active
tab's kind first, then the focused pane's kind, agent/remote arms preserved byte-identical —
feeding the UNCHANGED `cockpit_status` composer (`format!("focus: {focused}")`,
status_bar.rs:82). The one-glance answer to "where will my keys go", matching what the border
shows. Cosmetic, low priority, textbook pure-seam-plus-shim shape.

## Scope
### In
- **The focus-label derivation.** Replace the app.rs:18408-18414 `footer_focus` chain with a call
  into a new pure fn in `status_bar.rs` (the module is PURE, gpui-free, and already owns every
  footer string — status_bar.rs:1-2). Inputs (exact signature = Phase 2): the active tab's content
  kind (terminal / editor / cockpit — tabs.rs:23-31), the focused pane's `PaneKind`
  (workspace.rs:210-220, via `PaneState::kind`, :361-368), and the focused pane's agent-label /
  remote-host lookups (today's app.rs:18408-18411 arms, strings unchanged).
- **The tab-kind gate.** A non-terminal active tab no longer reads the background grid: an editor
  tab → `editor`, a cockpit tab → `cockpit`; ONLY a terminal tab resolves per-pane. This closes
  the background-agent-label leak the sweep uncovered (a stricter fix than the reported repro, but
  it is the same one derivation — splitting it off would ship the lie in two of three tab kinds).
- **Exact-string tests** on the pure seam for every arm (the #201 `titlebar::display_title` idiom,
  titlebar.rs:84-102): editor split, plain terminal, agent, remote, files, git, editor tab,
  cockpit tab, and the leak case (background agent + editor tab active).
- **`cockpit_status` untouched.** Its `focused: &str` signature and segment order stay as-is
  (status_bar.rs:64-84) — zero churn on its existing tests; the ONE callsite (app.rs:18431-18438)
  keeps passing a `&str`.
- **A driven capture of the QA repro** (editor split focused in a terminal tab: border + label
  agree in one frame), with the established env-blocked fallback.

### Out (explicitly deferred)
- **The #201 tab-title fallback ("Marley" → "terminal 1") — VERIFIED intentional.**
  `live_tab_title` (app.rs:7169) reads the focused TERMINAL only (`g.terminal(g.focused())`,
  app.rs:7178) — a focused CodeView pane yields `None`, so command/cwd tiers skip and
  `display_title` falls to the static fallback (titlebar.rs:84-102, the locked #201 tiering).
  Whether a focused editor split should surface its FILENAME in the tab title is a separate
  product question — not this bug.
- Any broader status-bar redesign: segment order, new segments, click affordances (the index-1
  agents click, app.rs:18442-18453) all unchanged.
- Per-section cockpit labels ("agents" / "forge" / "details") — plain `cockpit` for now; refine
  only if QA ever asks.
- The launcher state — the footer never renders there (`should_show_launcher` returns the launcher
  before the shell paints, app.rs:14726-14727), so no launcher arm exists in the label domain.
- Agent/remote label CONTENT (`run.label` / `remote.host`) — preserved byte-identical, not
  redesigned.

## Reference (§20)
**N/A — Marley-specific cockpit affordance, with an observed-convention anchor.** The #94 footer
is Marley-original chrome (the Warp map lists Marley's status bar as `[Marley-original]` on gpui —
docs/warp_architecture/subsystems/00-overview.md:45); there is no Warp behavior to match for a
focus-kind label. The convention the bug violates is the published/observed IDE norm: a status bar
describes the FOCUSED item, not its container — cursor position / language / encoding segments in
VS Code and JetBrains IDEs all swap with the focused editor (observed product behavior). Our OWN
Zed behavior map documents the same shape at the architecture level: status-bar cells are
`StatusItemView`s (docs/zed_architecture/subsystems/07-workspace-panes-palette.md:72) whose
canonical instances — "diagnostics summary, cursor position, language selector" (:248-250) — are
by nature focused-item-reactive. Research from our maps + observed behavior only; clean-room §20
untouched — no Warp (AGPL) / Zed (GPL) source read.

### Prior art
1. **Behavior maps — checked.** The Zed map's workspace chapter documents the status-bar-as-
   focused-item-mirror shape (07-workspace-panes-palette.md:72, :248-250 — see Reference); the
   Warp overview confirms the footer is Marley-original with no Warp analog (00-overview.md:45).
   No copyleft source consulted.
2. **Published material.** IDE status bars context-switch with focus (VS Code / JetBrains — the
   segments describe the focused editor). General observed behavior; no spec governs it.
3. **Permissive deps — none: checked, no owner.** gpui (Apache-2.0) renders divs and owns no
   status-bar widget or label vocabulary; no other shipped crate plausibly owns app-chrome text.
   **The highest-yield leg is OUR OWN shipped seams — the fix is a recomposition, not an
   invention:** `PaneState::kind()` already derives a `PaneKind` from the content variant
   (workspace.rs:361-368); `PaneKind::label()` already owns the "terminal"/"files"/"code"/"git"
   vocabulary (workspace.rs:224-231); `TabContent` is the closed tab-kind enum (tabs.rs:23-31);
   and `active_editor()` (app.rs:9256-9267) is the PROVEN #259 resolution shape this label must
   mirror — editor tab → its surface, else the terminal tab's focused pane when it is an editable
   surface. The label fix is that same resolution, returned as a word.

## Locked-In Decisions
- **D1 — Tab-kind gate first, then per-pane.** Active tab `Terminal(grid)` → resolve the focused
  pane (agent → remote → pane kind); active tab `CodeView` → `editor`; active tab `Cockpit` →
  `cockpit`. This is the honest generalization of "label the focused pane": in a non-terminal tab
  the "focused grid pane" is a BACKGROUND pane (`workspace()`'s first-terminal-grid fallback,
  app.rs:5288-5294), and today it can leak a background agent's label into the footer.
  **Rejected:** pane-only derivation (keeps the lie in two of three tab kinds); leaving the
  non-terminal-tab behavior for a follow-up (same one fn, same test table — splitting it is
  process overhead, not risk reduction).
- **D2 — A focused editable code pane reads `editor`, NOT `PaneKind::label()`'s `code`.** The M15
  #259 recast made the split pane a full `EditorSurface` — the SAME type an editor tab holds
  (workspace.rs:325-329), typing/saving like the editor tab (#355/#357) — so "editor" is the
  honest answer to "where will my keys go" and matches the editor-TAB arm. FileTree/Git reuse the
  established `files` / `git` vocabulary (workspace.rs:227-228). Whether the fn delegates
  terminal/files/git to `PaneKind::label()` with an editor override, or owns its full string
  table, is Phase 2's pick — the STRINGS themselves are locked here.
- **D3 — The derivation is a new pure fn in `status_bar.rs`; `cockpit_status` is untouched.** The
  module is already the PURE owner of footer text (status_bar.rs:1-2) and `cockpit_status`'s
  `focused: &str` parameter (status_bar.rs:64-84, the `format!("focus: {focused}")` at :82) stays
  byte-stable — zero churn on its four existing composition tests. The app.rs shim shrinks to
  gathering inputs (active tab content, focused pane kind, agent/remote lookups) and calling the
  pure fn. **Rejected:** widening `cockpit_status`'s signature (needless test churn for no
  behavior); deriving inline in the masked render (exactly the untestable shape that shipped this
  bug).
- **D4 — Agent/remote arms byte-identical.** An agent pane's label stays `run.label`, a remote
  pane's stays `remote.host` (today's app.rs:18408-18411), and the precedence agent → remote →
  kind is preserved (the maps key terminal-hosted panes only, so the order is unobservable today —
  preserving it is the zero-surprise choice).
- **D5 — The #201 tab-title half is out of scope** (verified working-as-designed — see Out). The
  spec fixes the STATUS BAR only.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal tab is active AND the focused grid pane is an editable code pane (`PaneContent::CodeView`), the status bar shall read `focus: editor`. | exact-string unit on the pure fn; driven capture of the QA repro (click into an editor split — the #191 accent border on the editor pane AND `focus: editor` in the SAME frame); if capture is env-blocked, the locked-screen fallback: units + mechanism trace, re-verify when unlocked |
| REQ-002 | WHEN a terminal tab is active AND the focused pane is a plain terminal pane (no agents/remotes entry), the status bar shall read `focus: terminal`. | exact-string unit — the unchanged-behavior pin for the common case |
| REQ-003 | WHEN a terminal tab is active AND the focused pane has an agents-map entry (or, failing that, a remotes-map entry), the label shall be that run's label (or that remote's host) — byte-identical to today's strings. | exact-string units for both arms + the precedence order |
| REQ-004 | WHEN a terminal tab is active AND the focused pane is a FileTree / Git panel, the status bar shall read `focus: files` / `focus: git`. | exact-string unit per kind (the PaneKind::label vocabulary, workspace.rs:227-228) |
| REQ-005 | WHEN an editor tab is active the label shall read `focus: editor`, and WHEN a cockpit tab is active it shall read `focus: cockpit` — regardless of any agents/remotes entries on the background terminal grid (no leak). | units on the tab-kind gate, INCLUDING the leak case: a background agent pane + an editor tab active → `focus: editor`, not the agent's label |
| REQ-006 | WHILE a terminal tab is active, the label shall derive from the SAME focused `PaneId` that draws the accent focus border — the footer always names the bordered pane's kind. | structural review: ONE `focused` source feeds both (the border's `pane_id == focused`, app.rs:16633, and the label's resolution, app.rs:14775) + the REQ-001 capture showing border and label agree |
| REQ-007 | WHILE the crate builds, the label derivation shall live on the pure `status_bar.rs` seam at cov/MSI 100, with the render shim masked and exhaustive matches over the closed `PaneKind`/tab-kind enums (a future pane kind fails the build into this label, never silently reads "terminal"). | gate green (`--diff`), cov/MSI 100 on status_bar.rs; `cargo mutants --list -f` on the ACTUAL file before claiming the kill set; inspect confirms exhaustive `match`, no `_ =>` arm over `PaneKind` |

## Floors (constitution)
The pure seam (status_bar.rs: the new fn + `cockpit_status`) at **cov/MSI 100**; the app.rs render
delta rides the existing masked shim. Exhaustive `match` over `PaneKind` and the tab-content kind
(closed enums — REQ-007's no-wildcard rule); no `unwrap` on input paths; exact-string assertions
(never `contains`) per the #201 idiom; test code avoids never-run branches.

## Phase Plan
- **P2 Design** — settle the pure-fn signature (candidate: `focus_label(tab: FocusTab, kind:
  Option<PaneKind>, agent: Option<&str>, remote: Option<&str>) -> String`, with `FocusTab` a tiny
  local enum or a reuse of an existing tab-kind view; decide delegate-to-`PaneKind::label()`-with-
  editor-override vs own-string-table per D2); the exact shim gather (active tab content match +
  the two map lookups) and its ~10-line diff at app.rs:18406-18414; per-REQ test plan incl. the
  REQ-005 leak construction.
- **P3 Implement** — status_bar.rs first (fn + doc), then the app.rs shim swap; no other files.
- **P3.5 Inspect** — adversarial: is the tab-kind gate actually consulted before the maps (the
  leak case)? does any other path still compose a focus string (grep `"focus`)? label and border
  truly share ONE `focused` source? strings byte-identical for agent/remote? provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; `cargo mutants --list -f` on the ACTUAL
  status_bar.rs; driven capture of the QA repro (focus Marley first; quit stale instances; the
  locked-screen fallback if env-blocked); gate green (`--diff`).
- **P5 Complete** — CHANGELOG; archive; AAR (candidate lesson: a hardcoded else-arm in a shim is a
  label lie waiting for the next pane kind — closed-enum matches belong on the pure side of the
  seam); close #382.
