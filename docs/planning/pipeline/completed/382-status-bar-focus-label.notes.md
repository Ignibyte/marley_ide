# Status bar focus label — derive from the focused pane's kind — Notes

- **Forge ticket:** #382 (d145a213-f2e9-43c2-b6e0-79836bfc87e0) — bug, milestone M25, sprint #36 "M25 — App-Grade QA Hardening"
- **AAR:** 3813a196-461c-49b3-9792-69ca9c9b6ccf
- **Local ticket doc:** docs/planning/tickets/open/TICKET-382-status-bar-focus-label.md
- **Pipeline spec:** 382-status-bar-focus-label.spec.md (pipeline_id d9f2681a-082f-410d-9e79-15f66ed46b48)

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** From the 2026-07-21 live QA run (capture 14-editor.png), sprint #36 /spec batch:
  clicking into a `workspace.rs` editor split inside a terminal tab's grid moves the accent focus
  border to the editor pane and flips the rail row (both correct), but the footer keeps reading
  `focus: terminal`. Ticket hypothesis: the label derives from the TAB kind. Fix requested: derive
  it from the FOCUSED PANE's kind (terminal / editor / cockpit), matching the border. The sibling
  observation — the tab title falling back "Marley" → "terminal 1" — read intentional (#201) and
  is to be scoped out.
- **Classification / tier:** bug, cosmetic, low priority. Textbook pure-seam shape: one new pure
  fn in the already-pure `status_bar.rs` + a ~10-line masked-shim swap in app.rs. One crate, no
  settings, no persistence, no wire.
- **Forge recall (§18.3):** Phase 1 drafted docs-only; recall to be re-run at /work promotion.
  Priors applied from disk/memory: the #201 `titlebar::display_title` pure-fn + exact-string-test
  idiom (titlebar.rs:84-102) — the shape this fix mirrors; the
  `PR-claude-per-pane-render-affordance-must-gate-on-is-focused` family (#200) — this bug is the
  same divergence class on the OTHER side (chrome not tracking the focused pane); the
  syntactic-form mutants lesson (`cargo mutants --list -f` on the ACTUAL code);
  `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` +
  focus-Marley-before-driving + quit-stale-instances for the P4 capture.
- **Discovery — the verified mechanism (the central claim, checked against the code):**
  - **The derivation is NOT tab-kind — it is tab-kind-INVARIANT.** app.rs:18408-18414:
    `footer_focus` = `self.agents.get(&focused)` → `run.label.clone()`, else
    `self.remotes.get(&focused)` → `remote.host.clone()`, else the **hardcoded string literal
    `"terminal"`**. No `PaneKind`, no `TabContent` anywhere in the chain. The ticket's hypothesis
    is corrected in the spec: the label lies for EVERY non-terminal focused pane kind, in every
    tab kind.
  - **`focused` is the grid's focused PaneId:** `let focused = self.workspace_mut().focused();`
    (app.rs:14775, inside `render`, app.rs:14702). `workspace()` (app.rs:5288-5294, doc
    :5283-5287) = the active tab's grid when it is a terminal tab, ELSE THE FIRST TERMINAL TAB's
    grid — so with an editor/cockpit tab active the footer reads a BACKGROUND grid, and a
    background agent pane's label can leak into the footer. The bug is strictly WORSE than
    reported; D1 gates on the active tab's kind to close it.
  - **The pure seam already exists and is label-agnostic:** `status_bar.rs` is PURE, gpui-free
    (:1-2); `cockpit_status(…, focused: &str, …)` (:64-84) appends
    `format!("focus: {focused}")` (:82). ONE callsite (app.rs:18431-18438). So the fix extends
    status_bar.rs with the derivation and leaves `cockpit_status` + its tests untouched (D3).
    (Its existing tests already pass "editor" as an exemplar string, e.g. :199 — the pure fn never
    knew the shim would hardcode.)
  - **The pane-kind domain (enumerated honestly):** `PaneKind { Terminal, FileTree, CodeView,
    Git }` (workspace.rs:210-220), derived from the content variant by `PaneState::kind()`
    (:361-368); `PaneKind::label()` owns "terminal"/"files"/"code"/"git" (:224-231).
    `PaneContent::CodeView(EditorSurface)` is the M15 #259 recast (:325-329) — a FULL editor
    surface, same type as an editor tab's, which is why D2 locks its footer word as `editor`
    (not label()'s "code"). Tab domain: `TabContent { Terminal(PaneGrid), Cockpit(RightSection),
    CodeView(EditorSurface) }` (tabs.rs:23-31). Overlays: `agents: HashMap<PaneId, AgentRun>`
    (app.rs:185), `remotes: HashMap<PaneId, Remote>` (app.rs:377). The launcher never shows the
    footer (early return, app.rs:14726-14727) — no launcher arm.
  - **The #259 resolution shape to mirror:** `active_editor()` (app.rs:9256-9267) — editor tab →
    its own surface, else `tab.grid().and_then(|g| g.focused_editable_surface())`. The label fix
    is the same tab-then-focused-pane resolution returned as a word; the render already proves the
    per-pane focus test (`self.workspace().focused() == pane_id`, app.rs:17487).
  - **The border invariant's other half:** `is_focused = pane_id == focused` (app.rs:16633), the
    #191 full 4-edge accent overlay per the comment at :16643-16644. Label and border both key on
    the ONE `focused` binding (app.rs:14775) → REQ-006 is structural, not aspirational.
  - **The #201 half verified intentional (scoped OUT):** `live_tab_title` (app.rs:7169) reads
    `t.grid().and_then(|g| g.terminal(g.focused()))` (app.rs:7178) — `terminal()` is `None` for a
    CodeView pane (workspace.rs:371-376), so custom/command/cwd all skip and
    `display_title(custom, command, cwd, fallback)` (call app.rs:7188; tiering
    titlebar.rs:84-102) lands on the static "terminal N" fallback. Working as the locked #201
    tier design; the "should a focused editor split title show its filename" question is a
    separate product ask.
- **Prior-art sweep:** Zed behavior map documents status-bar cells as focused-item-reactive
  `StatusItemView`s (docs/zed_architecture/subsystems/07-workspace-panes-palette.md:72,
  :248-250 — "diagnostics summary, cursor position, language selector"); Warp map confirms the
  footer is `[Marley-original]` (docs/warp_architecture/subsystems/00-overview.md:45) — no Warp
  analog. Permissive deps: none owns app-chrome label text (gpui renders divs) — pass recorded.
  Highest-yield leg = our own shipped seams (kind()/label()/TabContent/active_editor) — the fix
  recomposes, invents nothing. No copyleft source read (§20 clean).
- **Decisions:** D1 tab-kind gate first (Terminal → per-pane agent→remote→kind; CodeView tab →
  "editor"; Cockpit tab → "cockpit"; closes the background-leak); D2 focused CodeView pane =
  `editor` (the #259 recast; files/git keep the PaneKind vocabulary); D3 new pure fn in
  status_bar.rs, `cockpit_status` signature untouched; D4 agent/remote strings byte-identical;
  D5 the #201 tab-title half OUT.
- **Open questions (D-OPEN for Phase 2):**
  - **D-OPEN-FN-SHAPE** — the exact pure-fn signature: a tiny local tab-kind enum vs reusing an
    existing view; delegate terminal/files/git to `PaneKind::label()` with an editor override vs
    owning the full string table. The locked string TABLE (spec D2) constrains either shape.
  - **D-OPEN-SHIM-GATHER** — where the shim matches the active tab's content (before the grid
    walk it already does, or beside the existing `footer_focus` site) — a pure code-motion pick.
- **EARS drafted (full table in the spec):** REQ-001 editor split → `focus: editor` (+ the QA
  repro capture); REQ-002 plain terminal pin; REQ-003 agent/remote arms byte-identical; REQ-004
  files/git kinds; REQ-005 editor/cockpit TABS + the no-leak clause; REQ-006 label ≡ border (one
  `focused` source); REQ-007 pure seam cov/MSI 100, exhaustive closed-enum matches.
- **Forge ids:** ticket #382 d145a213-f2e9-43c2-b6e0-79836bfc87e0 (bug); sprint #36 "M25 —
  App-Grade QA Hardening", milestone M25. Provenance: 2026-07-21 live QA run, capture
  14-editor.png. Lineage: #94 footer, #188 agents-segment click, #259/#355/#357 editable split,
  #191 focus border, #201 title tiering (out).
- **Plan verification (Opus /work, 2026-07-22) — the core claim CONFIRMED + "before" evidence
  already captured live:**
  - `footer_focus` (app.rs:18415-18421, read directly) is EXACTLY `agents.get(&focused).label` →
    `remotes.get(&focused).host` → `"terminal".to_string()` — tab-kind-INVARIANT, never consults
    pane kind. The drafter's correction of the ticket's hypothesis holds.
  - `cockpit_status` (status_bar.rs:64-84) is PURE, takes `focused: &str`, `format!("focus: {focused}")`
    at :82; its 4 existing composition tests pass literal strings → a NEW `focus_label` fn beside it
    (cockpit_status UNCHANGED) is zero-churn (D3 holds). status_bar.rs module doc :1-3 = gpui-free pure.
  - Types confirmed closed + present: `PaneKind{Terminal,FileTree,CodeView,Git}` + `label()`
    (workspace.rs:210-231), `PaneState::kind()` (:361-368), `TabContent{Terminal,Cockpit,CodeView}`
    (tabs.rs:23-31). **D2 subtlety verified:** `PaneKind::label()` maps CodeView→"code", but the fix
    must map a focused CodeView pane → **"editor"** (the #259 EditorSurface) — so `focus_label` OWNS the
    string mapping with an exhaustive `match PaneKind` (no wildcard, REQ-007), not a `label()` delegate.
  - **LIVE "before" evidence already in hand:** during #381's drive the boot capture (`381-boot.png`)
    showed the footer reading "focus: terminal" while the Editor tab was highlighted — the exact bug.
    The "after" ("focus: editor") is the Validate capture.
  - Verdict: plan solid; a textbook pure-seam + masked-shim recomposition. No decisions revised.

## Phase 2 — Design

**Architecture / approach.** A pure decision fn + a masked shim swap — the textbook shape the bug
violated (it derived the label inline in the untestable render). §20 confirmed N/A (Marley-original
footer; the observed IDE convention — a status bar describes the FOCUSED item — is the anchor; no
copyleft source read).

**The pure seam (status_bar.rs — already the gpui-free owner of footer text):**
```rust
use crate::workspace::PaneKind;

/// Which tab is active + (for a terminal tab) the focused pane's identity — the input to `focus_label`.
pub enum FocusTab {
    Terminal { agent: Option<String>, remote: Option<String>, pane: PaneKind },
    Editor,
    Cockpit,
}

/// The footer's focus word — "where will my keys go", matching the #191 accent border.
pub fn focus_label(tab: &FocusTab) -> String {
    match tab {
        FocusTab::Editor => "editor".to_string(),
        FocusTab::Cockpit => "cockpit".to_string(),
        FocusTab::Terminal { agent, remote, pane } => {
            if let Some(a) = agent { a.clone() }
            else if let Some(r) = remote { r.clone() }
            else {
                match pane {                       // exhaustive over the closed PaneKind (REQ-007)
                    PaneKind::CodeView => "editor", // D2: the #259 EditorSurface, NOT label()'s "code"
                    PaneKind::Terminal => "terminal",
                    PaneKind::FileTree => "files",
                    PaneKind::Git => "git",
                }.to_string()
            }
        }
    }
}
```
No wildcard on `PaneKind` — a future variant fails the build INTO this label (REQ-007), never silently
reads "terminal". `cockpit_status` is UNCHANGED (`focused: &str`, `format!("focus: {focused}")`) — its 4
composition tests don't churn (D3). **The REQ-005 leak is now STRUCTURALLY IMPOSSIBLE:** `FocusTab::Editor`
and `::Cockpit` carry no agent/remote fields, so a non-terminal tab has nothing to leak — the type makes
the bug unrepresentable, stronger than a runtime gate.

**The shim swap (app.rs, replacing :18415-18421):**
```rust
let active_tab = self.shell.active_project().active_tab();
let tab = if let Some(grid) = active_tab.grid() {
    crate::status_bar::FocusTab::Terminal {
        agent: self.agents.get(&focused).map(|r| r.label.clone()),
        remote: self.remotes.get(&focused).map(|r| r.host.clone()),
        pane: grid.state(focused).map(|s| s.kind()).unwrap_or(crate::workspace::PaneKind::Terminal),
    }
} else if active_tab.cockpit_section().is_some() {
    crate::status_bar::FocusTab::Cockpit
} else {
    crate::status_bar::FocusTab::Editor
};
let footer_focus = crate::status_bar::focus_label(&tab);
```
`active_tab().grid()`→Some ⇔ terminal tab (tabs.rs:79); `.cockpit_section()`→Some ⇔ cockpit (:100); else
an editor tab (:125). For the terminal case, `grid.state(focused).map(|s| s.kind())` (PaneGrid::state
workspace.rs:617 → PaneState::kind :361) gives the focused pane's kind. The `unwrap_or(Terminal)` default
lives in the coverage-excluded shim (a terminal tab's focused pane is always registered — defensive). The
non-terminal arms NEVER read `self.agents`/the grid — the leak is closed by not looking. Borrows: three
disjoint immutable field reads (`self.shell`, `self.agents`, `self.remotes`) — no conflict.

**File manifest:**
| File | Change |
|---|---|
| `crates/marley_app/src/status_bar.rs` | NEW `pub enum FocusTab` + `pub fn focus_label` + `use crate::workspace::PaneKind` + `#[cfg(test)]` units. `cockpit_status` untouched. |
| `crates/marley_app/src/app.rs` | Replace the :18415-18421 `footer_focus` chain with the tab-gate gather + `focus_label(&tab)` call (~12 lines, masked shim). |

**Regression Test Plan (status_bar.rs units on `focus_label`):**
| Test | Proves | AC |
|---|---|---|
| editor tab → "editor" | `FocusTab::Editor` | REQ-005 |
| cockpit tab → "cockpit" | `FocusTab::Cockpit` | REQ-005 |
| terminal + `pane: CodeView`, no agent/remote → "editor" | the D2 override (the QA repro) | REQ-001 |
| terminal + `pane: Terminal` → "terminal" | unchanged common case | REQ-002 |
| terminal + `agent: Some("run-x")` → "run-x"; + `remote: Some("host")` (no agent) → "host"; agent BEFORE remote | byte-identical arms + precedence | REQ-003 |
| terminal + `pane: FileTree` → "files"; `pane: Git` → "git" | the PaneKind vocab | REQ-004 |
| terminal + agent set BUT `pane: CodeView` → the agent label (agent wins over kind) | precedence pin | REQ-003 |
- **cockpit_status's 4 existing tests** (`cockpit_status_*`) stay green byte-for-byte (D3 — signature
  unchanged). REQ-006 (label ⇔ border share one `focused`) + REQ-007 (exhaustive match) = structural
  inspect + the coverage/mutation gate. `cargo mutants --list -f status_bar.rs` at validate for the real
  kill set (the enum arms + the PaneKind match).
- **Uncoverable in units:** the shim's tab-gate + `unwrap_or` default (app.rs, coverage-excluded) — proven
  by the LIVE capture (editor split focused → "focus: editor" + the border agree) + the existing suites.

**Risks / decisions:** the `unwrap_or(PaneKind::Terminal)` default is a defensive shim-only fallback (a
terminal tab's focused pane is always in the grid); it can't create a pure-seam coverage hole (it's in
app.rs). The agent/remote clone-per-frame is negligible (footer render). No `unwrap` on a real input path.

## Phase 2 — Design (status)
Design PASS — pure fn shape settled (the leak made unrepresentable by the enum), shim swap exact,
per-REQ test plan. Ready for Implement.

## Phase 3 — Implement
Built to the manifest:
- **`status_bar.rs`**: added `use crate::workspace::PaneKind;`, `pub enum FocusTab { Terminal { agent,
  remote, pane }, Editor, Cockpit }`, and `pub fn focus_label(tab: &FocusTab) -> String` (exhaustive
  `match FocusTab` + exhaustive `match PaneKind`, CodeView→"editor" per D2). `cockpit_status` untouched.
  Tests deferred to Validate (the cov/MSI 100 surface).
- **`app.rs`**: replaced the :18415-18421 hardcoded `footer_focus` chain with the tab-gate gather
  (`active_tab().grid()`→Terminal / `.cockpit_section()`→Cockpit / else Editor) + `focus_label(&tab)`.

**Deviations: none.** `cargo check -p marley` clean (only the pre-existing `block v0.1.6` dep note);
`cargo fmt --check` CLEAN. `PaneId` is `Copy` — `grid.state(focused)` (by value) and
`self.agents.get(&focused)` (by ref) both compile; the three field borrows (`self.shell`,
`self.agents`, `self.remotes`) are disjoint immutable reads, no conflict. The old `"terminal".to_string()`
footer else-arm is gone (the remaining hits at app.rs:8820+ are unrelated pre-existing code).

## Phase 3 — Implement (status)
Implement PASS — pure fn + masked shim swap compile fmt-clean; ready for Inspect.

## Phase 3.5 — Inspect
Two independent critics over the diff. **Verdict: PASS — 1 LOW fixed, rest CLEAN.** Lenses:
(1) correctness, (2) leak-fix + simplification + provenance.

**[LOW — critic 1] FIXED: the tab-axis gate wasn't exhaustive.** The shim used an `is_some()` predicate
chain (`grid()` → `cockpit_section()` → `else = Editor`). Correct today (TabContent is closed
`{Terminal, Cockpit, CodeView}`), but a FUTURE 4th variant (an embedded browser — a live roadmap pillar)
has both accessors `None` and would silently fall into `Editor`, mislabeled with NO compile error — a
conformance gap against the ticket's OWN REQ-007 ("exhaustive matches over the closed PaneKind/**tab-kind**
enums; a future kind fails the build"). **Fix:** replaced the chain with an exhaustive
`match &self.shell.active_project().active_tab().content { TabContent::Terminal(grid) => …,
Cockpit(_) => …, CodeView(_) => Editor }` (TabContent is `pub`, tabs.rs:41; added to the `crate::tabs`
import). Now a new tab kind fails the build INTO this label, symmetric with `focus_label`'s exhaustive
`PaneKind` match. `cargo check` + `cargo fmt --check` clean. (Bonus: the match destructures
`Terminal(grid)` directly — no separate `grid()` call.)

**Confirmed clean (both critics, against source):**
- **Precedence byte-identical** — same `focused` PaneId (app.rs:14782), same `agents→run.label` /
  `remotes→remote.host` maps, agent-before-remote; the agent/remote strings unchanged (the leak fix is
  the ONLY behavioral delta: those maps are now read only in the Terminal arm).
- **Leak closed BY CONSTRUCTION** — `FocusTab::Editor`/`::Cockpit` are field-less, so an agent/remote
  label is structurally unrepresentable for a non-terminal tab; the shim's non-terminal arms read
  neither `self.agents`/`self.remotes` nor the grid. Stronger than a runtime guard.
- **Exhaustive `PaneKind` match** — CodeView→"editor" (D2, NOT `label()`'s "code"), no wildcard;
  correctly OWNS its table (a `label()` delegate would drop the CodeView override + the build-fail
  guarantee — critic 2 confirmed: do NOT collapse).
- **Borrow / panic clean** — `PaneId` is `Copy` (layout.rs:9); the mutable `workspace_mut()` borrow at
  :14782 ends before :18412; disjoint immutable field borrows; `unwrap_or(Terminal)` is an infallible
  shim-only default (app.rs, coverage-excluded); `active_project()`'s empty-panic is guarded by the
  launcher early-return (:14733).
- **REQ-006** — the label's `focused` is the SAME binding the #191 accent border keys on
  (`is_focused = pane_id == focused`, app.rs:16640); when the active tab is non-terminal the border loop
  is empty and the label goes to Cockpit/Editor without touching `focused`. They agree.
- **status_bar.rs coverage-INCLUDED** (not in gates.sh:222) → the seam is gate-enforced; `focus_label`
  adds exactly 2 function-level mutants, both killable — but cov-100 forces a unit PER ARM (~7-8:
  editor/cockpit + agent + remote + 4 pane kinds), the Validate deliverable. `cockpit_status` untouched
  (4 tests don't churn); §20 own-code, no secret/unsafe.

**Fixes applied: 1** (the exhaustive tab match). No `failure-record` — the LOW is a latent conformance
gap (no bug today), not a shipped failure. Prevention-rule candidate for Complete: a closed-enum GATE
must use an exhaustive `match`, not an `is_some()` predicate chain, so a new variant fails the build.

## Phase 3.5 — Inspect (status)
Inspect PASS — 1 LOW (tab-axis exhaustiveness) fixed at source; leak closed structurally; ready for
Validate.

## Phase 4 — Validate
**Tests added: 3 `focus_label` unit fns** (status_bar.rs) covering EVERY arm — `focus_label_non_terminal_tabs`
(editor→"editor", cockpit→"cockpit"; REQ-005), `focus_label_terminal_pane_kinds` (CodeView→"editor" [D2,
REQ-001], Terminal→"terminal" [REQ-002], FileTree→"files"/Git→"git" [REQ-004]),
`focus_label_agent_remote_precedence` (agent→label, remote→host, agent-beats-remote-beats-kind [REQ-003]).

**Results:**
- `cargo nextest run -p marley focus_label` → 3/3 pass; **full `-p marley` suite → 808 passed, 2 skipped**
  (the +3 are mine; REQ-005 — nothing broke; `cockpit_status`'s 4 tests unchanged, D3).
- `cargo mutants --list -f crates/marley_app/src/status_bar.rs` → `focus_label` adds exactly 2 mutants
  (`String::new()`, `"xyzzy".into()`), both killed by the units (every assertion expects a concrete
  string ≠ ""/≠ "xyzzy"); the `FocusTab` enum adds 0. Gate:5 confirms.
- **`scripts/gates.sh --diff` → GATE GREEN [diff] 15/15** (coverage ≥100% incl. status_bar.rs — every
  `focus_label` arm covered; MSI ≥100%). First-attempt green.

**LIVE (REQ-001/006 — proven on the running app):**
- Launched the fresh #382 bundle; a terminal pane was focused → footer correctly read "focus: terminal"
  (`382-after-boot.png` — the unchanged common case, REQ-002).
- Clicked into the workspace.rs EDITOR SPLIT (a `CodeView` pane inside the terminal tab's grid — the
  exact QA repro): `382-after-editor.png` shows the #191 cyan focus border move to the editor pane, the
  rail flip to "pane 2", AND the footer now read **"focus: editor"** — the label MATCHES the border
  (REQ-001 + REQ-006). Pre-#382 this same interaction left the footer at "focus: terminal" (the
  2026-07-21 QA `14-editor.png` + the earlier `381-boot.png` this run). Marley quit cleanly after.
- The tab title still reads "terminal 1" in that frame — the OUT-of-scope #201 fallback (a focused
  editor split has no cwd), working as designed; the footer (the #382 surface) is correct.

## Phase 4 — Validate (status)
Validate PASS — 3 units + suite green, 2 mutants killed, gate GREEN [diff]; REQ-001/006 proven live
(editor split focused → "focus: editor", matching the border). Ready for Complete.

## Phase 5 — Complete
- **CHANGELOG:** entry under `### Added` (the footer `focus:` label reflects the focused pane's kind;
  the background-agent leak closed).
- **Architecture docs:** `docs/marley_architecture/app_shell.md` — a #382 bullet in the footer/cockpit
  section (the `FocusTab` exhaustive-match gate + `status_bar::focus_label`).
- **Forge knowledge:** `PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001` (2381467f —
  a closed-enum GATE must use an exhaustive `match`, not an `is_some()` predicate chain, so a new variant
  fails the build; the inspect F1 catch, REQ-007 generalized to the shim's tab axis); `aar-submit`
  3813a196 (completed, effectiveness 5, 1 novel finding). NO `failure-record` — the F1 LOW was a latent
  conformance gap (no shipped bug). `ticket-comment` + `ticket-close` #382 → done.
- **Ticket doc** → `docs/planning/tickets/closed/`, status closed. **Pipeline** → `completed/`.

Complete PASS.
