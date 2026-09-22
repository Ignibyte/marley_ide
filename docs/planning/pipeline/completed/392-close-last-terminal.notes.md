# 392-close-last-terminal — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad — "we should be able to terminate the last terminal. Not sure why thats a
  requirement but we should fix that." Depth locked via AskUserQuestion: **dissolve BOTH guards**
  (LastTerminal + LastTab; a workspace may be fully empty). Sprint #38 M27; forge #392
  `2157791d-4d6b-4207-9322-0c73378c3e4f`. **HARD DEP: #391** (the totality audit).
- **Why this is now SMALL:** #387 already extracted the guards into the pure `close_tab_refusal`
  truth table — the dissolve is shrinking one pure fn + removing two enum variants + rewriting the
  pinned tests + the empty-state UI. The RISK lives in #391 (the substrate), by design.
- **The #387 test inventory to rewrite:** `close_tab_refusal_truth_table` (R4 LastTab, R6
  LastTerminal rows), `close_tab_section_vocab`, `close_last_terminal_guard` (M10-era). Rewriting
  pinned tests is the DELIBERATE act (D3) — changelog'd, surfaced.
- **Empty-state boundary (D2):** the #247 launcher = zero WORKSPACES; this ticket's empty-center =
  an open workspace with zero tabs. Do not conflate (the #202-era A/B/C fork's lesson).
- **Inspect lens to carry:** ⌘W on the last tab must not cascade into close-project; the keyboard
  ladder over an empty project must be a total no-op set.
- **Prior-art sweep:** in-repo (#387 seam, #385 empty headers, #247/#234 zero-state precedents) +
  the IDE empty-hint convention. Recorded in spec.

## Phase 2 — Design: THE D4 SIZING FINDING (2026-07-23) — #392 is 2-3 slices, surfaced to chad
The design's first act (the D4 sizing call) found #392 is materially bigger than one slice — a SECOND
totality audit surfaced:
- **Guard-dissolve = trivial** (as drafted): `close_tab_refusal`→IndexOutOfRange-only; remove the 2
  `TabError` variants + the ONE prod consumer (app.rs:7322, a status flash) + rewrite the #387 tests.
- **NEW — the ZERO-TAB / `active_tab()` totality (a whole second audit, parallel to #391):** dissolving
  `LastTab` makes a **zero-tab project reachable**, and `active_tab()`/`active_tab_mut()` are **bare
  indexes** `&self.tabs[self.active]` (tabs.rs) → they PANIC on zero tabs. **15 `active_tab()` + 3
  `active_tab_mut()` call sites** in app.rs (the render center dispatch, the status bar, the titlebar,
  handlers) — every one panics on a zero-tab project. This is a #391-scale audit in its own right
  (an `active_tab()`-Option twin + threading it through the render/status/handlers).
- **The no-terminal USER-HANDLER totality (~15 helpers)** — absorbed from #391's D4-split, still required.
- **The empty-workspace UI** (empty-center placeholder + empty Terminal header).
- **`adjust_active` already 0-safe** (returns 0 for new_len==0) — so close CAN empty a project; the panic
  is purely the downstream `active_tab()` bare indexes.

**So "close the last terminal" = guard-dissolve (trivial) + TWO totality audits (no-terminal handlers +
zero-tab active_tab) + the empty-UI.** That is 2-3 pipeline slices, not one — and a split cannot leave a
crash-gap (once the guards dissolve, a user CAN reach the empty state and press any key). The clean,
crash-safe decomposition is: **do the totality enablers FIRST (behavior-neutral, guards ON — exactly the
#391 pattern), THEN dissolve the guards + ship the empty-UI last.** This extends the "close the last
terminal" work beyond one ticket → beyond the stated `/work 390-394` range. **Surfaced to chad — his
call (it changes the goal's ticket count + sequencing).** #392 is PARKED at Phase 2 pending that decision.

### RESOLUTION (chad, 2026-07-23): "Split it." → #392 RE-SCOPED to the TOTALITY; #395 created for the capstone
chad chose the split (the safe #391 pattern). **#392 (this ticket) = the no-terminal + zero-tab TOTALITY,
behavior-neutral (guards ON, test-only reachable).** **#395 (`51abebc2`, created) = the capstone** —
dissolve the guards + the empty-workspace UI (depends on #392). Spec re-scoped (title + Scope). Goal is
now effectively `/work 390-395`.

### #392 Design — the totality (extends #391's `try_workspace` with a `try_active_tab` twin)
**Contract (tabs.rs):** add `try_active_tab(&self) -> Option<&Tab<S>>` + `try_active_tab_mut(&mut self) ->
Option<&mut Tab<S>>` (`self.tabs.get(self.active)`), the total twins of the bare-index `active_tab()`/
`active_tab_mut()`. Harden `terminal_grid_index` (tabs.rs:375) `self.tabs[self.active]` → panic-free via
`self.tabs.get(self.active)`.

**The reachable `active_tab()` sites (inline audit — route through `try_active_tab`/a `tab_count()>0`
guard; behavior-neutral, byte-identical when tabs exist):**
- **ALWAYS-RUN (panic every frame/tick on zero tabs):** the render **center dispatch** (app.rs:16677
  `active_cockpit = active_tab().cockpit_section()` — add a `if tab_count()==0 { blank center } else {…}`
  guard FIRST, so 16693/16707's `active_tab().code_view()` are only reached with tabs; #395 fills the blank
  with the hint UI); the **status bar** (18684 `active_tab().content` match); **`active_is_terminal`**
  (16812); the **pump tick** (1460 `active_tab().grid()`).
- **KEYSTROKE (panic on any key on a zero-tab project):** the keymap **`key_context()`** reads (15557,
  17989) — an empty project has no active tab ⇒ no key context ⇒ the chord no-ops; the terminal
  **raw-key router** (15822, no-terminal — via `try_workspace`); 15857/15887 `active_tab().grid()`.
- **HANDLER:** 8741, 9396, 13932 (close/editor handlers) — total via `try_active_tab`.

**The no-terminal USER-HANDLER helpers (via #391's `try_workspace`, from the #391 ledger):**
`split_focused_pane`→no-op, `new_terminal_pane`→create-anyway (`spawn_terminal_tab_in`, grid-free),
`focus_neighbor`→no-op, the agent/remote/overlay/prompt-insert/comment helpers→no-op, the deferred-input
closures (15196/16460/16937/17011). All `try_workspace` + no-op/create.

**File manifest:** `tabs.rs` (the `try_active_tab*` twins + `terminal_grid_index` harden + the twin unit
tests + a zero-tab-close/round-trip test) + `app.rs` (route the reachable active_tab + handler sites
through the twins — masked shims). **Guards STAY (behavior-neutral):** `close_tab_refusal` UNCHANGED here
(#395 shrinks it); the empty states are test-constructed.

**Regression Test Plan:** `try_active_tab`/`try_active_tab_mut` — Some(active) for a normal project, None
for a zero-tab `Project` (cov/MSI 100 pure); `terminal_grid_index` None on zero-tab; a zero-tab `Project`
drives the totalled model seams (no panic); the FULL suite byte-identical (guards on, REQ-002); the codec
zero-tab parse (extend #391's T5). **NO driven capture** — behavior-neutral, zero user-visible change (the
empty state is #395's). The app.rs render/handler shims are in the ACCEPTED-UNTESTABLE coverage exclude
(the #391 precedent).

**Risks/decisions:** D-TRY-ACTIVE-TAB (the twin, over making `active_tab()` itself Option — the #391 D-c
rationale: minimal churn, keep the panicking accessor for gated sites); D-BLANK-CENTER-ON-ZERO-TAB (#392
renders a blank center on zero tabs — total but minimal; #395 fills the hint UI — the split boundary);
D-BEHAVIOR-NEUTRAL (guards on; the full suite is the proof); a zero-tab `Project` for tests is built via a
`#[cfg(test)]` constructor or by draining tabs directly (confirm the cleanest at validate).

## Phase 3 — Implement (2026-07-23)
Built; `cargo check --workspace` + `--tests` green, `cargo fmt` clean, the FULL marley suite **830 pass
UNCHANGED** (behaviour-neutral confirmed — no test touched). Diff: tabs.rs +22, app.rs +298 (mechanical
routing).

- **tabs.rs:** `try_active_tab()`/`try_active_tab_mut() -> Option<&(mut) Tab<S>>` (`self.tabs.get(self.
  active)`); `terminal_grid_index` hardened (`.get(self.active).is_some_and(grid)`). `close_tab_refusal`
  UNCHANGED (guards stay — #395 shrinks it).
- **app.rs (the app.rs half delegated to a subagent under this implement, then verified):** every
  zero-tab / no-terminal reachable site routed through the twins, byte-identical when tabs/terminals
  exist —
  - **Zero-tab (try_active_tab):** the render CENTER gained a `tab_count()==0 → blank center div` first
    branch (the active_tab dispatch moved into the else; #395 fills the blank with hints); the status bar
    (`try_active_tab().map(|t| &t.content)`, `None => FocusTab::Cockpit`); `active_is_terminal`; the pump
    tick; the two `key_context()` reads (`try_active_tab().map(key_context).unwrap_or_default()` —
    `&[KeyContext]: Default` = empty = the chord no-ops); the key-handler `grid()` reads; `active_editor`/
    `active_editor_mut` (`try_active_tab()?`); close-pane `multi_pane_terminal`.
  - **No-terminal (try_workspace, from #391):** whole-body early-return guards on `split_focused_pane`,
    `launch_agent`, `open_remote_target`, `open_kind_pane`, `split_file_pane`; `new_terminal_pane` cwd read
    made None-safe → **create-anyway**; the RAW-KEY router + ~30 focused_terminal handler reads converted
    to `try_workspace…and_then(focused_terminal)` + no-op; the focus-nav arms, agent/rerun/clear/
    block-jump/prompt-insert/comment/history/finder/workflow helpers, the keydown closures, the rail
    pane-row click, `top_search_sources`.
- **DEVIATION (a thoroughness WIN, all byte-identical):** the subagent converted MORE handler sites than
  the design's minimal list — a comprehensive grep of every `workspace()`/`workspace_mut()`/
  `focused_terminal`/`active_tab()` user-path site, so the totality is complete, not just the enumerated
  core. Correctly LEFT as gated: `persist_grid` (#391-done), the empty-`rect_list` per-pane render loops,
  the transitively-guarded inner calls, `focused_pane_for_test` (test accessor), `pane_group` (dead, zero
  callers).

## Phase 1 — Promote to active (Opus, 2026-07-23)
Promoted queued → `active/` (third of `/work 390-394`; HARD DEP **#391 SHIPPED `d8acc8b`** — the
`try_workspace`/`try_workspace_mut` contract + always-run totality). §3 re-confirmed — 392 the sole
active spec. **AAR opened:** `f3583ee3-a5bf-455b-80b6-dba092bb341a`.

**⚠️ SCOPE GREW (the #391 D4 absorption):** the Phase-1 draft (above) called #392 "SMALL" — that was
BEFORE #391's design split moved the **~30-site USER-HANDLER totality + the 6 product questions INTO
#392**. #392 is now: the (small) guard-dissolve + (moderate) empty-UI + the (LARGE) user-handler
totality. The design phase makes the **D4 sizing call** (one coherent "the empty workspace works" slice
vs a core+polish split).

**Surface gauge (verified this session):**
- **Guard-dissolve = SMALL:** `close_tab_refusal` (tabs.rs:115) shrinks to IndexOutOfRange-only; the
  `TabError::LastTab`/`LastTerminal` variants (tabs.rs:600/603) + their SOLE production consumer
  **app.rs:7322** (`Err(LastTerminal | LastTab) => …`) removed; the #387-pinned tests (tabs.rs:1083/
  1132/1138/1388 + `close_tab_section_vocab`/`close_last_terminal_guard`) rewritten. ~23 refs, mostly
  tests.
- **User-handler totality = LARGE (the bulk):** the full ~30-site ledger is in the COMPLETED #391 notes
  (`391-no-terminal-totality.notes.md`, the "USER-TRIGGERED" table) — the design starts from it, no new
  Explore needed. All use #391's `try_workspace`/`try_workspace_mut`. The CRITICAL one: the terminal
  **RAW-KEY router (app.rs:15822)** → no-op (a keystroke on the empty cockpit tab must not route to a
  non-existent terminal).
- **Empty-center render:** the render center's else-chain (`active_cockpit` → cockpit_body; `else if
  code_view` → editor; **final else → the terminal grid**, app.rs ~16690) is where the empty-workspace
  placeholder branch goes (when the active project has zero tabs / the final else has no grid).
- **#391-forward hardening:** `terminal_grid_index`'s bare `self.tabs[self.active]` (tabs.rs:375) →
  `.get(self.active)` for the now-reachable zero-tab project; the deferred-input closures (app.rs
  15196/16460/16937/17011) join the user-handler set.
- **Reference §20 unchanged** (convention: IDE empty-state hint panel; Marley's own rail). EARS +
  Locked-In Decisions (D1 both-guards, D2 empty≠launcher, D3 test-rewrite-is-the-deliverable, D4
  default-seed-unchanged, D5 variants-removed) stand + the #391-absorbed handler scope.

## Inspect (Phase 3.5) — 2026-07-22 (Opus, /goal)
Two independent critics over `git diff -- crates/marley_app/src/app.rs tabs.rs` (app.rs +298, tabs.rs
+22). This is a LARGE mechanical diff whose correctness rests on TWO properties — **byte-identity** (the
D1 invariant: guards still ON, so production, where a terminal+tabs always exist, must be unchanged) and
**completeness** (no always-run/keystroke site still panics on the now-TEST-reachable zero-tab or
no-terminal project — a miss wouldn't show in the 830-test suite since it's test-only reachable). One
critic per property.

**Critic A — BYTE-IDENTITY: CLEAN (zero HIGH, zero MED).** Traced all ~55 conversions; confirmed the 7
highest-risk spot-checks are byte-identical when a terminal + tabs exist: (a) the render-center
`tab_count()==0` guard inlines the existing `active_cockpit`/`code_view`/terminal dispatch into the else,
no branch reordered/dropped; (b) the status bar `try_active_tab().map(|t| &t.content)` — `None =>
FocusTab::Cockpit`, the `Some(Terminal/Cockpit/CodeView)` arms preserve the old bodies; (c) `key_context()
→ try_active_tab().map(key_context).unwrap_or_default()` — `&[KeyContext]: Default` = empty slice ⇒ no
binding ⇒ chord no-ops, identical with a tab; (d) `new_terminal_pane` cwd read None-safe + identical cwd
with a terminal; (e) the whole-body `try_workspace_mut().is_none()` early-returns no-op ONLY with no
terminal, run the full old body otherwise; (f) rerun-last-failed `states()` yields the same states. My own
re-derivation matched each.

**Critic B — COMPLETENESS: COMPLETE, no MISS.** Exhaustively classified every bare accessor to its gate.
The three residual bare `active_tab()` reads (app.rs 16780/16795/16809) all sequence strictly AFTER the
leading `if tab_count()==0` guard (16766) in the render center-dispatch chain — nothing bare runs before
it. Both `key_context()` sites (15633 keydown, 18098 palette hints) are total. The raw-key readline router
(15976) is gated by `&& try_active_tab().is_some_and(grid)`. The pump tick's one bare `workspace()` (1465)
is inside a `try_active_tab().is_some_and(grid)` guard. Every action handler (launch_agent, split_*,
open_*, send/broadcast-to-agents, close-pane, persist_grid) has an explicit `try_*` guard preceding the
bare call. Render-region bare `workspace()` calls (16549/16925/17043+) are all behind `active_is_terminal`
or inside the empty-`rect_list` pane loop (`Vec::new()` when not a terminal) → never run. Non-production
residue: `focused_pane_for_test` (2589, `#[cfg(test)]`) and `pane_group` (9005, zero production callers) —
neither on the always-run/keystroke surface.

**The ONE finding (LOW, CONFIRMED, FIXED):** `activate_search_hit` @ 4819 (`HitKind::Session`) was the sole
bare `workspace_mut()` whose safety rested on a **cross-function invariant** (a Session hit can only be
produced when `top_search_sources`'s `try_workspace` yielded pane ids ⇒ a live workspace) rather than a
local guard. Safe TODAY (verified: `top_search_sources` @ 4772 derives `ids` from
`try_workspace().map(pane_ids).unwrap_or_default()` → empty on a no-terminal project → the `if let
Some(id)` never enters), but a latent footgun: #393 (section ＋), #394 (ContentId registry), and chad's
stated "global pane section" for cross-workspace panes could let `top_search_sources` yield sessions across
workspaces, turning 4819 into a live panic. Since #392 IS the totality substrate whose job is to eliminate
exactly this class, I applied the defensive `if let Some(w) = self.try_workspace_mut()` guard —
byte-identical in the reachable case (a Session hit ⇒ a live workspace), a no-op in the (currently
unreachable) empty case. `cargo check` clean, `cargo fmt` clean, **830 tests still pass byte-identical.**
→ `PR-claude-totality-substrate-close-cross-function-invariant-accessors-001` (in a no-panic totality
ticket, convert even the bare accessors that are safe-by-invariant-today to their local `try_*` guard when
an imminent feature could break the invariant — the substrate's value is locality, not just today's
reachability).

**Lenses covered:** byte-identity (Critic A), completeness/reachability (Critic B), plus my own
skeptical re-check of both. No unwrap/expect/panic reintroduced (the change REMOVES panic reachability); no
borrow issue from the subagent's conversions (compile + 830 tests green). No secrets/provenance/clean-room
concern (§20 N/A — pure Marley policy, no source read).

`status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`.

## Phase 4 — Validate (2026-07-22, Opus, /goal) — GATE GREEN [diff]
Behaviour-neutral refactor (guards ON → the empty states are TEST-ONLY reachable), so the proof is
**byte-identity + the totalled pure seams don't panic + cov/MSI 100 on the new seams**.

**Tests added (2, per the design's Regression Test Plan):**
- `tabs.rs::try_active_tab_twins_totality` — with-tabs `try_active_tab()` returns `Some` with a CONTENT
  assert (title `"c"`, kills the `-> None` body mutant); the mut twin writes through and is observed via
  `active_tab()` (kills `None` + the leaked-fresh-tab mutants); zero-tab (`p.tabs.clear()`) → BOTH twins
  `None` (kills the 4 `Some(<default tab>)` body mutants). Plus a zero-tab line in
  `terminal_grid_index_cases` (`empty.terminal_grid_index() == None`, no bare-index panic).
- `grid_layout.rs::restore_shell_parses_a_zero_tab_project` — `restore_shell("0\n/x\t0")` → 1 project,
  `active_tab == 0`, `tabs` empty (the zero-tab codec parse; extends #391's no-terminal parse; reachable
  only in tests until #395).

**Mutation (gate:5, the KEY proof for the pure seams):** the REAL mutant set was traced up front via
`cargo mutants --list -f tabs.rs` (not guessed — the PR-…-trace-the-real-list rule). The twins add **10
new viable mutants** (`try_active_tab` @356: `None` + 4 `Some(Box::leak(<default Tab>))`; `try_active_tab_mut`
@362: the same 5) — ALL killed by the totality test (the zero-tab `None` asserts kill the `Some(default)`
variants; the with-tab content/write-through asserts kill the `None` variants). `terminal_grid_index`
@391's 3 (`None`/`Some(0)`/`Some(1)`) are UNCHANGED (the harden added no operator/`<` mutant — the method
calls are unmutated) and stay killed by the existing `terminal_grid_index_cases`.

**Runs (actual):**
- `cargo nextest run -p marley` → **832 passed, 2 skipped** (the 830 pre-existing byte-identical + the 2
  new). REQ-002 (full suite byte-identical) confirmed.
- `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15 passed / 0 failed. gate:4 coverage **100% lines**
  (tabs.rs + grid_layout.rs pure seams; app.rs render/handler/`activate_search_hit` shims are in the
  ACCEPTED-UNTESTABLE coverage exclude — the #391 precedent, byte-identity-proven by the two critics),
  gate:5 mutation **MSI 100%**, gate:15 visual/AX green. The `--diff` receipt for `/commit` is written.

**Driven capture — N/A (explicitly, not skipped).** #392 is behaviour-neutral: guards are ON, so a
zero-tab / no-terminal project is UNREACHABLE in production — the app renders and responds
byte-identically (the new `tab_count()==0 → blank center` branch and every totalled handler are
test-only reachable). There is no pixel to change and nothing to drive. **#395 is where the empty state
becomes user-reachable + visible** (dissolve the guards + fill the empty-center hints) — that is where
the pre-seed driven capture lands and chad eyeballs the empty-workspace UX. This matches the design
(D-BEHAVIOR-NEUTRAL) and REQ-002.

**Pre-existing exclusions:** none new. The app.rs render/input shims remain in the coverage/mutation
exclude (gates.sh:189-192, the M27 #391 precedent) — masked, mechanism-verified by the inspect critics'
byte-identity proof; the pure `try_active_tab*`/codec seams carry cov/MSI 100 here.

`status: Phase 4 — Validate PASS; ready for Phase 5 — Complete`.
