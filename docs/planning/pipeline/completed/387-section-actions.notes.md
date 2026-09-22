# 387-section-actions — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** slice 3 of chad's sectioned shell — per-section ＋ actions + guard re-expression.
- **Sprint:** #37 M26; forge #387 `e2ffcfbb-06dd-4c66-90e4-e25758769e64`. HARD DEP: #385
  (independent of #386, though the hit-target overlap with #386's header-click collapse is an
  inspect lens if both have shipped).
- **Code grounding (2026-07-22 map + shipped history):**
  - `close_tab` tabs.rs:200-215 — refuses `LastTab` and `LastTerminal`; the guard pair to
    re-express. The #202 parking record documents the ~90-site ≥1-terminal dependency; this ticket
    PINS, never relaxes.
  - `open_or_switch_cockpit` tabs.rs:286 / `open_cockpit_section` app.rs:3665 — Browser＋'s verb.
  - New-terminal path: the #140 top-bar "+" lineage; `spawn_terminal_tab_in` (#294 extraction),
    cwd inheritance #281 — Terminal＋'s verb.
  - Editor＋: the existing open-file flow (palette/tree path) — exact entry point chosen in P2.
- **Prior-art sweep:** recorded in spec — reuse-only (D1); no external owner.
- **Open items for P2:** Editor＋ entry point (file picker vs Files-panel focus); multi-project
  routing (the ＋ must act on ITS project row's project); hit-target separation from the #386
  header-click toggle.

## Phase 1 — Promote to active (Opus, 2026-07-22)
Promoted the drafted queued spec → `active/` (the FINAL ticket of `/work 385-389`; #385 hard-dep
SHIPPED `29b4218`, #386/#389 also shipped, so the #386 header-click hit-target overlap IS an inspect
lens). §3 re-confirmed — 387 is the sole active spec (389 archived at its Phase 5). **AAR opened:**
`377f2b44-d8bd-4190-87ae-6b665071bd20`.

**Anchors RE-VERIFIED against the CURRENT tree (they DRIFTED post-#385/#386 — the spec body is now
corrected; design must use these):**
- `close_tab` at **tabs.rs:254** — returns `LastTab` at **259 (checked FIRST)**, then `LastTerminal`
  at **264**; `TabError::LastTab`/`LastTerminal` variants at **539/542** (spec's old "200-215" was
  stale).
- `open_or_switch_cockpit` at **tabs.rs:340** (spec's old "286" was stale).
- New-terminal verbs: `new_terminal_pane` **app.rs:6604** + `spawn_terminal_tab_in` **app.rs:6623**.
- **D4 load-bearing signal for design (verify, don't assume):** the shipped `close_tab` checks
  `LastTab` (tabs.len()==1) BEFORE `LastTerminal`, and the test at tabs.rs:1003 asserts "single tab →
  **LastTab** precedence" (995 asserts `LastTerminal` for a terminal among other tabs). So D4's stated
  hypothesis ("`LastTab` SUBSUMED by `LastTerminal`") looks **FALSE as written** — for the genuinely
  last tab, `LastTab` still fires first. The real section-semantics truth table (design output): with
  Terminal never-empties, `LastTab` is only reachable when the ONE remaining tab is the sole terminal;
  `LastTerminal` fires when >1 tab but exactly one is a terminal. Design must PIN this exact table in
  tests and surface the "not subsumed" finding explicitly (never a silent behavior change — D4).
- **Reference §20 unchanged** (N/A — Marley-specific; the sectioned-shell design is chad's own, no
  Warp/Zed analog) — the filled draft stands. **Decisions D1–D5 unchanged.** EARS REQ-001..006 stand.

## Phase 2 — Design (2026-07-22)

### Architecture / approach
Lands on the #385/#386 rail: the `RailLevel::Section` header row (app.rs:16331) that already carries the
#386 ▸/▾ chevron + click-to-collapse. Two PURE seams in the gpui-free `tabs.rs`, one masked render/dispatch
in `app.rs`. Reference §20 = **N/A — Marley-specific** (confirmed — chad's sectioned shell; no Warp/Zed
analog; clean-room wall not engaged, gpui hover/`group_hover` is adoption).

- **Pure seam 1 — the section→action routing table (`tabs.rs`).**
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum SectionAction { NewTerminal, OpenFile, OpenCockpit(RightSection) }
  pub fn section_action(section: RailSection) -> SectionAction {
      match section {
          RailSection::Editor   => SectionAction::OpenFile,
          RailSection::Terminal => SectionAction::NewTerminal,
          RailSection::Browser  => SectionAction::OpenCockpit(RightSection::Forge),
      }
  }
  ```
  Total over the 3 sections → cov/MSI 100. (`RightSection` is already in `tabs.rs`'s scope via
  `open_or_switch_cockpit`.)
- **Pure seam 2 — the guard truth table (`tabs.rs`), extracted from `close_tab` (D2 behaviour-IDENTICAL).**
  ```rust
  fn close_tab_refusal(tab_count: usize, idx: usize, target_is_terminal: bool,
                       terminal_count: usize) -> Option<TabError> {
      if idx >= tab_count { return Some(TabError::IndexOutOfRange); }
      if tab_count == 1   { return Some(TabError::LastTab); }
      if target_is_terminal && terminal_count == 1 { return Some(TabError::LastTerminal); }
      None
  }
  ```
  `close_tab` delegates: `let target_is_terminal = self.tabs.get(idx).is_some_and(|t| t.grid().is_some());
  let terminal_count = self.tabs.iter().filter(|t| t.grid().is_some()).count();` then match the refusal.
  Panic-free (`.get(idx)`, no bare index) and mutation-clean (no `<` in the shim). **This is the
  "re-express + test-pin in section vocabulary" deliverable** — NOT a behaviour change.
- **The ＋ affordance (`app.rs` §16331 render arm — masked).** After the existing chevron + `flex_1` label,
  append a ＋ child on the RIGHT, hover-revealed via the #217 idiom: `entry.group(format!("rail-sec-{p}-{}",
  row.label))` + the ＋ child `.opacity(0.).group_hover(<same name>, |s| s.opacity(1.))` (mirrors
  block-actions app.rs:17107/17147). The ＋ gets its OWN `on_mouse_down(Left, …)` that calls
  `view.dispatch_section_action(p, section)` then **`cx.stop_propagation()`** + `cx.notify()`.
- **The dispatch shim (`app.rs` — `#[cfg_attr(test, mutants::skip)]`).** Mirrors the #174 cross-project
  idiom the Tab/Pane rows use (app.rs:16274-16281):
  ```rust
  fn dispatch_section_action(&mut self, p: usize, section: RailSection) {
      let changed = p != self.shell.active_project_index();
      let _ = self.shell.switch_project(p);
      if changed { self.sync_active_project(); }
      match section_action(section) {
          SectionAction::NewTerminal      => self.new_terminal_pane(),               // app.rs:6604 (persists)
          SectionAction::OpenFile         => self.start_file_finder(),               // extracted; see below
          SectionAction::OpenCockpit(s)   => { self.shell.active_project_mut().open_or_switch_cockpit(s);
                                               self.persist_grid(); }                // tabs.rs:340
      }
  }
  ```
- **Reuse extraction (`app.rs`) — `fn start_file_finder(&mut self)`** = the 3 finder-open lines
  (`self.finder = FinderState::new(); self.finder_open = true; self.finder_split = false;`), called by BOTH
  the existing `"open-file-finder"` command arm (app.rs:8426) AND the dispatch. Single-source reuse (D1) —
  the ＋ opens the SAME ⌘P finder that yields a CodeView tab under Editor (REQ-002).

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/tabs.rs` | ADD `SectionAction` enum + pure `section_action`; ADD pure `close_tab_refusal`; REFACTOR `close_tab` to delegate (behaviour-identical); ADD unit tests (routing + guard truth table + the D4 not-subsumed proof). |
| `crates/marley_app/src/app.rs` | ADD the hover-revealed ＋ to the `RailLevel::Section` render arm (group + group_hover + own on_mouse_down + stop_propagation); ADD `dispatch_section_action` shim; EXTRACT `start_file_finder` and call it from the `"open-file-finder"` arm + the dispatch. |

### Regression Test Plan
| # | Test (`tabs.rs` unit unless noted) | Proves | Coverage |
|---|---|---|---|
| T1 | `section_action(Editor)==OpenFile`, `(Terminal)==NewTerminal`, `(Browser)==OpenCockpit(Forge)` | REQ-001/002/003 routing | pure MSI 100 |
| T2 | `close_tab_refusal`: `idx>=count`→`IndexOutOfRange`; `count==1`→`LastTab`; `is_term&&term_count==1`→`LastTerminal`; else `None` (all branches, both sides of each comparator) | REQ-004 guard table | pure MSI 100 |
| T3 | `close_tab` integration in section vocab: `[T]`→`LastTab`; `[T,C]` close C→`Ok` (Browser empties); `[T,V]` close V→`Ok` (Editor empties); `[T,C]` close T→`LastTerminal`; `[T,T]` close either→`Ok` | REQ-004/005 unchanged outcomes | pins behaviour |
| T4 | **D4 not-subsumed proof:** `close_tab_refusal(1, 0, true, 1)==Some(LastTab)` (a sole lone terminal → `LastTab`, NOT `LastTerminal`) — the two guards are DISTINCT | REQ-004 (D4) | pure MSI 100 |
| D1 | driven: click **Terminal＋** → a new terminal tab appears under Terminal, focused | REQ-001 | capture |
| D2 | driven: click **Editor＋** → the ⌘P finder opens | REQ-002 | capture |
| D3 | driven: click **Browser＋** → the Forge cockpit tab appears under Browser | REQ-003 | capture |
| D5 | driven: with a terminal present, close the last **Editor** tab → succeeds; the empty muted "Editor" header still renders | REQ-005 | capture |
| D6 | driven: hover a section header → ＋ appears; move off → ＋ gone | REQ-006 hover-reveal | capture pair |

**Uncoverable by unit (masked, driven-validated):** the ＋ render + `on_mouse_down` dispatch, and the live
verbs it calls (`new_terminal_pane` PTY spawn, the finder modal, `open_or_switch_cockpit`) — all already
`mutants::skip` shims tested elsewhere. If the machine is locked/unavailable at Validate, fall back to
units + mechanism (PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism) — the routing
+ guard seams carry the behaviour; the dispatch is byte-identical to the proven #174 idiom.

### Risks / decisions
- **D-STOP-PROPAGATION (load-bearing, the #386 overlap):** the ＋'s `on_mouse_down` MUST
  `cx.stop_propagation()` — else the same click ALSO fires the parent header row's #386 collapse toggle.
  This is THE inspect lens the spec names (hit-target overlap). Chevron left / ＋ right also keeps the
  pixel targets disjoint.
- **D-EDITOR＋-IS-THE-FINDER:** Editor＋ opens the ⌘P file finder (`start_file_finder`), NOT the Files-panel
  toggle (`open-files`) — the finder is the "open-file flow that yields a CodeView tab" REQ-002 asks for.
  Locked (the last P2 open item, resolved).
- **D-GUARD-EXTRACTION behaviour-identical (D2):** `close_tab` delegates to the pure `close_tab_refusal`;
  `.get(idx).is_some_and(...)` keeps it panic-free + mutation-clean. Byte-identical outcomes — pinned by T3.
- **D-D4-NOT-SUBSUMED (verified, surfaced):** `LastTab` (count==1, checked FIRST) and `LastTerminal` (>1
  tab, exactly 1 terminal) are DISTINCT and BOTH reachable; the spec's "LastTab subsumed by LastTerminal"
  hypothesis is **FALSE**. Pinned by T4; surfaced here, never a silent behaviour change (D4 honoured).
- **D-PLUS-ACTIVATES-PROJECT:** clicking ＋ on project p's header switches the active project to p (mirrors
  the #174 Tab/Pane cross-project sync) — required (the verbs are active-project-scoped) and natural UX.
- **D-PER-ROW-GROUP:** the hover-reveal group name is per-`(p, label)` so hovering one header reveals only
  ITS ＋, never a sibling section's.

## Phase 3 — Implement (2026-07-22)
Built to the manifest; `cargo check --workspace` AND `--workspace --tests` both green (only the pre-existing
`block v0.1.6` future-incompat warning), `cargo fmt` clean.

**tabs.rs:**
- ADDED pure `SectionAction { NewTerminal, OpenFile, OpenCockpit(RightSection) }` + `pub fn
  section_action(RailSection) -> SectionAction` (Editor→OpenFile, Terminal→NewTerminal,
  Browser→OpenCockpit(Forge)) — total, gpui-free.
- ADDED pure `fn close_tab_refusal(tab_count, idx, target_is_terminal, terminal_count) -> Option<TabError>`
  (IndexOutOfRange / LastTab / LastTerminal / None), and REFACTORED `close_tab` to delegate via
  `self.tabs.get(idx).is_some_and(|t| t.grid().is_some())` + the terminal filter-count. Behaviour-IDENTICAL
  (D2); the doc comment now names the D4 not-subsumed fact.

**app.rs:**
- EXTRACTED `fn start_file_finder(&mut self)` (the 3 finder-open lines) and pointed the `"open-file-finder"`
  command arm at it — single-source reuse (D1).
- ADDED `#[cfg_attr(test, mutants::skip)] fn dispatch_section_action(&mut self, p, section)` mirroring the
  #174 cross-project idiom (switch_project + sync-if-changed) then `match section_action(section)` → the
  existing verb (`new_terminal_pane` / `start_file_finder` / `open_or_switch_cockpit`+persist).
- ADDED the hover-revealed ＋ to the `RailLevel::Section` render arm: a per-row `.group("rail-sec-{p}-{label}")`
  + the ＋ child `.opacity(0.0).group_hover(<same>, |s| s.opacity(1.0))` (the exact #217 idiom, app.rs:17176)
  with its own `on_mouse_down` → `dispatch_section_action(p, section)` + **`cx.stop_propagation()`** +
  `cx.notify()`. Wired INSIDE the existing `if let Some(section)` so both the ＋ and the #386 collapse
  listener share the unwrapped section.

**Deviations from design (all minor, none change scope):**
1. `close_tab_refusal` sits grouped with `section_action` (module-level, near `RailSection`) rather than
   literally above `close_tab` — a module-level free fn cannot live inside the impl block; grouping the two
   pure seams reads cleaner. Behaviour unaffected.
2. Added a direct-hover accent to the ＋ (`.hover(|d| d.text_color(accent))`) — mirrors the block-actions
   #217 element (app.rs:17178); a fidelity match, not new scope.
3. Used a plain ASCII `"+"` glyph (not the fullwidth `＋`) — conventional chrome affordance at caption size;
   trivially adjustable if the driven capture reads too thin.

## Inspect (Phase 3.5 — 2026-07-22)
**Method:** 2 independent critics over the diff (A: correctness + guard-equivalence; B: gpui-idiom + state
+ mutation-readiness) PLUS my own trace of the gpui 0.2.2 mouse-dispatch source and the persist round-trip.

**The load-bearing claim — CONFIRMED SOUND (three ways):** the ＋'s `cx.stop_propagation()` DOES prevent the
parent header's #386 collapse from also firing. gpui registers the parent's listeners before the child's
(div.rs:1855 paint_mouse_listeners → :1865 children), and the Bubble phase walks the flat vec in REVERSE
(window.rs:3705 `.iter_mut().rev()`) with break-on-`!propagate_event` (:3708) → the ＋ child fires FIRST and
halts before the collapse handler. Clicking elsewhere on the header → the ＋ hitbox isn't hovered → only the
collapse fires. (Critic B noted the block-actions precedent's parent has NO on_mouse_down, so #387 is the
first immediate-parent case — the flat-vec-reverse mechanism handles it identically; the source trace, not
the precedent, is what clears it.)

**Findings — 2 confirmed-real, FIXED; the rest CONFIRMED-OK:**
| # | sev | finding | verdict | fix |
|---|-----|---------|---------|-----|
| F1 | MED | The `OpenFile` dispatch arm did NOT persist the cross-project switch, unlike the #174 Tab/Pane rows and the sibling NewTerminal/OpenCockpit arms. `serialize_shell` DOES persist `active_project` (grid_layout.rs:330; restored at boot app.rs:2139), and the finder's own persist fires only on ⌘↵-open — so Editor＋ on a non-active project → cancel → relaunch reverts the active project (self-healing on any later persist). | REAL (me + critic A) | Added `if project_changed { self.persist_grid(); }` to the OpenFile arm (parity with #174). |
| F2 | LOW | `start_file_finder`'s doc claimed "a picked file opens in a CodeView tab" — but the finder's DEFAULT plain-↵ inserts the path into the focused terminal (the #97 gesture); only ⌘↵ opens the CodeView tab. REQ-002 is literally met (a resulting CodeView tab DOES file under Editor via ⌘↵), but the doc overstated the default. | REAL (critic A) | Reworded the doc to name both gestures accurately + flagged a "default-to-editor-open when launched from Editor＋" follow-up (new finder logic, beyond this ticket's reuse-only D1 scope). |

**Rejected / no-fix (CONFIRMED-OK):**
- **`close_tab` behaviour-identical** (critic A traced every case: idx≥len → `.get`=None→false, no panic, IndexOutOfRange; len==1 → LastTab; LastTerminal branch idx<len so `.get(idx).is_some_and` == old `tabs[idx].grid()`; the unconditional `terminal_count` is a pure count). No divergence.
- **D4 not-subsumed** — `close_tab_refusal(1,0,true,1)==LastTab`, `(2,0,true,1)==LastTerminal`: distinct + both reachable (both critics).
- **opacity(0) hit-catcher** — opacity is paint-only, the ＋ keeps a hitbox; but any click there requires a hover that reveals it, and the region is a disjoint ~1-char right edge. LOW, by-design, no fix (both critics).
- **group-name uniqueness** (`rail-sec-{p}-{label}`, digit/hyphen-free labels), **dispatch state integrity** (mirrors `jump_to_pane`; safe `cx.listener` `&mut self`), **routing total + reuse-only**, **no new unwrap/panic paths** — all CONFIRMED-OK.

**Validator carry-forward (Phase 4 — from critic B, MUST include):**
- `section_action`: 0 viable mutants (no `Default` derive → the body→`Default::default()` mutant is unviable,
  the #203 lesson) — but coverage needs all 3 arms asserted (Editor→OpenFile, Terminal→NewTerminal,
  Browser→OpenCockpit(Forge)).
- `close_tab_refusal`: viable mutants (comparator swaps + body→`None` since `Option::default()==None`). The
  existing `close_tab` tests hit LastTab/LastTerminal but NOT the boundary/`&&` rows. Add the 7-row table as
  DIRECT unit tests: R1 `(2,2,_,_)`→IndexOutOfRange, R2 `(2,3,_,_)`→IndexOutOfRange, R3 `(2,0,false,2)`→None,
  R4 `(1,0,true,1)`→LastTab (D4), R5 `(2,1,false,1)`→None (kills `&&`→`||`), R6 `(2,0,true,1)`→LastTerminal,
  R7 `(2,0,true,2)`→None (kills `==1`→`!=1`). Then `cargo mutants --list -f crates/marley_app/src/tabs.rs`
  to confirm the generated operator set matches these killers (the standing "trace the real list" lesson).

## Phase 4 — Validate (2026-07-22)

### Tests added (tabs.rs `#[cfg(test)]`) — all pass
- `section_action_routing` (T1) — the 3 arms (REQ-001/002/003). `SectionAction` has no `Default` derive → the
  body→`Default::default()` mutant is UNVIABLE (0 viable mutants); the 3 asserts carry coverage.
- `close_tab_refusal_truth_table` (T2) — the 7 rows R1–R7 (REQ-004 + D4 not-subsumed). Direct calls to the
  pure guard; kills the comparator/`&&`/`==1` mutants the integration tests miss.
- `close_tab_section_vocab` (T3) — REQ-004/005: [T,code] close code→Ok (Editor empties), [T,cockpit] close
  cockpit→Ok (Browser empties). Complements the pre-existing `close_last_terminal_guard` ([T,C]/[T,T]/single),
  which still passes UNCHANGED (the refactor is behaviour-identical).

### Mutation — traced the REAL list (not guessed), gate:5 MSI 100
`cargo mutants --list -f crates/marley_app/src/tabs.rs` for the seams:
- `section_action` (92): only `→ Default::default()` — UNVIABLE (no Default), 0 survivors.
- `close_tab_refusal` (112): `→None`, `→Some(Default::default())`, `>=`→`<`, `==`→`!=` (×2, at 115 & 118),
  `&&`→`||`. The real set was SMALLER than critic B hypothesised (only `>=`→`<`, no `>`/`<=` variants) —
  vindicating the "trace, don't guess" rule. Traced each to its killer: `→None`←R1; `>=`→`<`←R1+R3;
  `==`→`!=`(115)←R3+R4; `&&`→`||`←R5; `==`→`!=`(118)←R6+R7; `→Some(Default)`←R3/R5/R7 (or unviable). All killed.

### Gate — GATE GREEN [diff], 15/15 (the .rs commit receipt is written)
`scripts/gates.sh --diff`: gate:4 coverage **100% lines** ✓, gate:5 mutation **MSI 100%** ✓, plus rustfmt,
clippy -D, nextest+doctests, audit/deny/machete, gitleaks, shellcheck, no-suppressions, source-bans, docs,
miri, visual/AX — all green. One RED fixed at source mid-phase: gate:14 rejected a public→private intra-doc
link (`close_tab`'s doc linked `[`close_tab_refusal`]`, a private fn — `-D rustdoc::private-intra-doc-links`)
→ changed to a plain-code `` `close_tab_refusal` `` reference; re-ran → green.

### Driven live-app verification — CAPTURE path confirmed the render; INPUT path env-blocked → mechanism
Launched the fresh gate-built binary and captured it (screencapture -l works):
- **Confirmed via capture:** the M26 sectioned rail RENDERS with this binary (Editor / Terminal / Browser
  section headers + chevrons), and **at REST no ＋ appears on any section header** — REQ-006's hidden-at-rest
  half, visually verified.
- **Synthetic INPUT is environment-blocked:** 3 attempts (2 `clickat`, 1 `tab`+`enter`) to drive the
  launcher's "New empty workspace" — none registered (the launcher never advanced), while captures kept
  working. This is the established hazard (synthetic CGEvents need Accessibility perm + a real `.app` bundle;
  the bare `target/debug/marley` on this HD can't receive them — captures via CGWindowList are unaffected).
  Per PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism, I did NOT fake a capture and
  did NOT keep retrying past 3.
- **Fallback — units + mechanism** for the interactions the input path couldn't reach:
  - REQ-006 reveal half: the ＋ uses the BYTE-IDENTICAL #217 `group_hover(opacity 0→1)` idiom proven in
    block-actions (app.rs:17176).
  - REQ-001/002/003: the routing is unit-proven (cov/MSI 100), and `dispatch_section_action` is
    byte-identical to the proven #174 cross-project idiom → the existing verbs (new_terminal_pane /
    start_file_finder / open_or_switch_cockpit).
  - The ＋-click-does-not-collapse (#386 overlap): gpui-source-verified at inspect (reverse-bubble +
    break-on-stop_propagation, window.rs:3705/3708), not merely asserted.
- **Data safety:** the first launch accidentally used chad's REAL config (`MARLEY_CONFIG_DIR` is not the
  app's env var — it's `$HOME/.marley/config`); I only CAPTURED (pressed nothing) on that instance, then
  relaunched under an isolated `HOME`. chad's `~/.marley/config/settings.toml` mtime is UNCHANGED (Jul 22
  18:07 before and after) — nothing of his was written. Both test instances killed; scratchpad dirs cleaned.

### Pre-existing failures
None in scope. (The `block v0.1.6` future-incompat is a pre-existing upstream-dep warning, not a gate failure.)
