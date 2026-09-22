# 395-empty-workspace — Notes

## Phase 1 — Plan (Opus, 2026-07-22, /goal /work 390-395)
- **Request:** the VISIBLE capstone of "terminate the last terminal" — flip the never-empties guards +
  ship the empty-workspace UI on #392's totality substrate. forge #395 `51abebc2-7d0e-43b9-8161-272fdbce5401`;
  sprint #38 M27. The LAST numbered ticket of `/work 390-395` (the D4 split of #392 created it + made #396
  the out-of-range follow-up).
- **Drafted FRESH** (no queued spec — #395 was created as the #392 D4-split follow-up). Scope + Reference §20
  + Prior art + D1-D5 lifted from the #392 spec (which was originally drafted with THIS scope, before the
  split moved the totality to #392) + the #392 completed-notes' guard-dissolve surface gauge.
- **HARD DEP #392 SHIPPED (`3f18f01`):** the totality substrate. #392's `try_active_tab`/`try_workspace`
  twins make the app TOTAL over zero-tab + no-terminal (so dissolving the guards here is CRASH-SAFE — the
  ~40 sites #392 routed through the twins already survive the empty states); #392 also added the
  render-center `tab_count()==0 → blank div` branch that THIS ticket fills with hints, and the
  `restore_shell_parses_a_zero_tab_project` codec test.
- **Surface gauge (from the #392 notes, verified there):**
  - **Guard-dissolve = SMALL:** `close_tab_refusal` (tabs.rs:~115) → IndexOutOfRange-only; the
    `TabError::LastTab`/`LastTerminal` variants (tabs.rs:~600/603) + their SOLE prod consumer
    **app.rs:~7322** (a status flash `Err(LastTerminal | LastTab) => …`) removed; the #387-pinned tests
    (`close_tab_refusal_truth_table` R4/R6, `close_last_terminal_guard`, any `close_tab_section_vocab` row)
    rewritten. ~23 refs, mostly tests. **Design MUST grep `LastTab`/`LastTerminal` for the FULL consumer
    set** (the #392 gauge said "the ONE prod consumer" but the design re-verifies — a missed match arm is
    a compile error once the variant is gone, so the compiler is the backstop, but grep first).
  - **Empty-center render:** #392's `tab_count()==0` branch currently draws a blank `bg(background)` div at
    center_bounds; #395 fills it with the muted hint rows (new terminal ⌘T · open file ⌘P · the section ＋s).
  - **Empty Terminal header:** the #385 section-header render already renders Editor/Browser empty; confirm
    the Terminal section renders its header when it has zero rows (likely free).
- **Prior-art sweep:** in-repo decisive (#387 close_tab_refusal, #392 substrate + blank-center, #385
  empty-header, #247 launcher-boundary, #234 zero-project-persist); no Warp analog; gpui no owner (pure
  Marley policy). Recorded in spec.
- **UI-affecting (VISIBLE) → the validate phase MUST driven-capture the empty state** via the #390 pre-seed
  technique (isolated HOME's settings.toml `workspace.shell` = a cockpit-only/zero-tab project → launch →
  `screencapture -l<winid>` → READ the empty-center + empty Terminal header). Synthetic INPUT is env-blocked
  this session, but the pre-seed stages the empty state directly (it IS persisted — a zero-tab project line),
  UNLIKE #393's transient menu. chad EYEBALLS the empty-workspace UX here.
- **AAR opened:** `6ede7abc-8fc3-44b6-837b-1bf5744703df`.

## Phase 2 — Design (Opus, 2026-07-22, /goal /work 390-395)

### (0) Sizing — ONE coherent slice (SMALL guard-dissolve + MODERATE empty-UI)
The guard-dissolve is surgical (1 pure-fn shrink + 2 variant removals + 1 app.rs match arm + a stale
comment); the empty-center hints are a masked render fill; the empty Terminal header is #385-free. NOT a
totality audit (#392 did that + made this crash-safe). Confirmed one slice.

### (1) The guard-dissolve (tabs.rs) — surgical, exhaustively verified
- **`close_tab_refusal`** (tabs.rs:112) shrinks from `(tab_count, idx, target_is_terminal, terminal_count)
  -> Option<TabError>` to **`(tab_count, idx) -> Option<TabError>`** = `if idx >= tab_count {
  Some(IndexOutOfRange) } else { None }` (the `tab_count==1 → LastTab` and `target_is_terminal &&
  terminal_count==1 → LastTerminal` checks DELETED). Its 2 dropped params vanish from `close_tab`
  (tabs.rs:316), which no longer computes `target_is_terminal`/`terminal_count`.
- **`TabError`** (tabs.rs:615): remove `LastTab` + `LastTerminal`; **only `IndexOutOfRange` survives** (still
  used by switch_tab/switch_project/close_project). §0 — no dead variants.
- **The SOLE production consumer** (grep-verified — every other `LastTab`/`LastTerminal` hit is a comment or
  a test): **app.rs:7445** `close_tab_at`'s `Err(TabError::LastTerminal | TabError::LastTab) => { status_flash
  "can't close the last terminal" }`. REMOVE that arm → the match becomes `Ok(removed) => {…reap+persist…}` +
  `Err(TabError::IndexOutOfRange) => {}` (exhaustive, single-variant; a stale-index close is a no-op). The
  `std::thread::spawn(move || drop(removed))` reap (7442) is UNTOUCHED (REQ-005). The compiler is the
  backstop — once the variants are gone, any missed consumer is a build error.
- **Stale comment** app.rs:8860 ("The last-terminal guard refuses via a flash") → removed/updated (the guard
  is gone). The ⌘W `close-pane` chain (8858) is ALREADY total via #392's `try_active_tab` — closing the last
  tab leaves a zero-tab project, and ⌘W on an already-empty project is a safe `IndexOutOfRange` no-op; it
  never cascades to `close_project` (D2 — the workspace stays open). No code change there beyond the comment.

### (2) The rewritten guard tests (D3 — THE deliverable, done loudly)
- **`close_tab_refusal_truth_table`** (tabs.rs:1150) → the shrunk table over `(tab_count, idx)`: R1
  `(2,2)==Some(IndexOutOfRange)` (idx==count boundary), R2 `(2,3)==Some(IndexOutOfRange)` (idx>count), R3
  `(2,0)==None` (in-range), **R4-NEW `(1,0)==None`** (the SOLE tab now CLOSES — was `LastTab`), **R5-NEW
  `(0,0)==Some(IndexOutOfRange)`** (a zero-tab project: 0>=0 → you can't close a nonexistent tab). These kill
  the shrunk fn's only mutant class (`>=`→`>`/`<`/`<=`/`==`): R1 (idx==count) kills `>=`→`>`; R3 (idx<count)
  kills `>=`→`<=`; R5 (idx==count==0) reinforces the boundary. cov/MSI 100.
- **`close_last_terminal_guard`** (tabs.rs:1127) → rewritten to the NEW policy: `[T,C]` close(0) → **Ok**
  (the terminal closes, leaving `[C]`); `[T,T]` close either → Ok; a single tab close → **Ok** (leaving a
  zero-tab project — was `LastTab`). Rename to `close_tab_never_refuses_in_range` for honesty.
- **`close_tab_section_vocab`** (tabs.rs:1178) — UNCHANGED (it closes non-last Editor/Browser tabs, which
  succeed under BOTH policies; it never encoded the refusal). Optionally EXTEND with a "close the last
  terminal → Ok + zero-tab" row (the new capability) — decide at implement (keep it lean).

### (3) The empty Terminal section header (#385 parity) — FREE
The rail iterates `RailSection::ALL` (4 sections) and renders each header regardless of row count (the
#385/#386 sectioned shell); the empty Editor/Browser already render a muted header with no rows. The
Terminal section is not special-cased to hide when empty → its empty muted header renders for free. Confirm
at implement (grep the section-header render for a row-count gate; expected none).

### (4) The empty-center hint UI (app.rs — fill #392's blank branch, masked)
#392's `if tab_count()==0` branch (app.rs:16831) draws a blank `bg(colors.background)` div at
`center_bounds`. #395 fills it with a **centered, muted hint panel** (flex-col, items_center,
justify_center, gap) — NOT the launcher (D2 — the rail + section headers stay). Content (masked-shim copy,
capture-verified; no keycap-chip helper exists — #222 was inline, so use simple styled text):
```
        Nothing open in this workspace          (muted, larger)
        ⌘T    New terminal
        ⌘P    Open file
        ＋     Create from a section header
```
The shortcut glyph in a subtle bordered/muted chip (inline, the #222 idiom) or plain muted text — implement
picks the cheapest that reads clearly. No pure seam (static content); the render is the masked shim, verified
by the driven capture. `center_bounds` geometry reused from the #392 branch.

### (5) Persistence — the round-trip is ALREADY complete
`serialize_shell` (grid_layout.rs) writes `<root>\t<active_tab>` then loops `project.tabs` — a zero-tab
project's loop body never runs → the line is `<root>\t<active_tab>` with NO entries. Restore (#392's
`restore_shell_parses_a_zero_tab_project`) parses that → empty tabs. So the round-trip works with NO codec
change (the default-seed for a NEW workspace is a separate boot path, unchanged — D4). REQ-003 = just a
serialize→restore round-trip TEST on a zero-tab `ProjectLayout`.

### Reference (§20) — confirmed
Convention (IDE empty-state hint panel; the rail is Marley's own sectioned shell); the #392 template holds.
No copyleft source read. N/A for a Warp/Zed match (Warp always holds ≥1 session surface; Zed tolerates zero —
observation only).

### File manifest
- **`crates/marley_app/src/tabs.rs`** — `close_tab_refusal` shrink (sig `(tab_count, idx)` + body); `close_tab`
  drops the `target_is_terminal`/`terminal_count` computation; `TabError` loses `LastTab`+`LastTerminal`; the
  rewritten `close_tab_refusal_truth_table` + `close_tab_never_refuses_in_range` (was `close_last_terminal_guard`)
  + a serialize-zero-tab round-trip test (or in grid_layout.rs).
- **`crates/marley_app/src/app.rs`** — `close_tab_at` error arm → `Err(TabError::IndexOutOfRange) => {}`; the
  #392 blank-center branch → the muted hint panel; the stale 8860 comment.
- **`crates/marley_app/src/grid_layout.rs`** — (if the round-trip test lands here) a `serialize` +
  `restore_shell` round-trip on a zero-tab project.
- **`crates/marley_app/src/headless_drive.rs`** — a `#[gpui::test]` ⌘W-closes-the-last-tab drive (REQ-002).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-004 | `close_tab_refusal_truth_table` (rewritten, shrunk) — IndexOutOfRange only; in-range always None | pure unit (cov/MSI 100) |
| REQ-001/002 | `close_tab_never_refuses_in_range` — `[T,C]`/`[T,T]`/single-tab all close Ok (the sole terminal + the last tab close, leaving `[C]`/`[T]`/zero-tab) | pure unit |
| REQ-002 | `headless_drive`: boot 1-project-1-terminal-tab → simulate ⌘W → `tab_count()==0`, no panic (the render survives on #392's substrate) | `#[gpui::test]` |
| REQ-003 | `serialize_shell` + `restore_shell` round-trip on a zero-tab `ProjectLayout` → restores empty; a NEW workspace still default-seeds a terminal (boot path unchanged) | pure unit |
| REQ-004 | grep-clean: no `TabError::LastTab`/`LastTerminal` anywhere (inspect) | inspect grep |
| REQ-005 | the existing reaper/close tests stay green (the reap path untouched) | existing suite |

**DRIVEN CAPTURE (this IS user-visible — chad EYEBALLS):** TWO legs — (a) the headless ⌘W drive above proves
the guard-dissolve + no-panic end-to-end (state); (b) a PIXEL capture of the empty-center hints + the empty
Terminal header via the **#390 pre-seed technique** — the empty state IS PERSISTED (a zero-tab project line),
so pre-seed an isolated `HOME=$ISO`'s `.marley/config/settings.toml` `[workspace] shell = "0\n/x\t0"` (a
zero-tab project) OR a cockpit-only project → launch `HOME=$ISO target/debug/marley` → `screencapture
-l<winid>` → READ the PNG (assert the hint panel + the empty Terminal header render). Unlike #393's transient
mouse-only menu, screencapture WORKS here (no synthetic input needed — the state is pre-seeded). State the
result at validate.

### Risks / decisions
- **The shrunk `close_tab_refusal` is nearly trivial** (`idx >= tab_count`) — kept as the named pure seam
  (the D3 test rewrite targets it; mirrors switch_tab's inline check but stays the documented close seam).
- **`close_tab_section_vocab` unchanged** — it never encoded the refusal (non-last closes); don't churn it.
- **The empty state is REACHED by closing, never created** (D4) — the boot default-seed + new-workspace path
  are untouched; the design adds no auto-empty.
- **Inspect critics:** a missed `LastTab`/`LastTerminal` consumer (grep + the compiler backstop); the ⌘W
  chain never cascades to `close_project` (D2); the empty-state keyboard ladder is total (#392 re-confirm);
  restore-empty vs default-seed precedence; the hint-panel doesn't leak into a non-empty render.

## Phase 3 — Implement (Opus, 2026-07-23, /goal)
Built; `cargo check --workspace` + `--tests` green, `cargo fmt` clean, the full suite **840 pass** (the
guard tests were REWRITTEN in place, not net-added). The variant removal surfaced the missed consumers as
compile errors (the design's backstop) — all fixed.

- **tabs.rs (the guard-dissolve + the D3 rewrites):** `close_tab_refusal` shrunk to `(tab_count, idx) ->
  Option<TabError>` = `if idx >= tab_count { Some(IndexOutOfRange) } else { None }` (the LastTab/LastTerminal
  checks deleted); `close_tab` drops the `target_is_terminal`/`terminal_count` computation; **`TabError` now
  has ONLY `IndexOutOfRange`** (`LastTab`/`LastTerminal` removed). Three guard tests REWRITTEN (D3):
  `close_tab_refusal_truth_table` (the shrunk 5-row table — IndexOutOfRange-only), `close_last_terminal_guard`
  → RENAMED `close_tab_never_refuses_in_range` (in-range always Ok — the sole terminal + the last tab close),
  `close_last_is_refused` → RENAMED `close_last_tab_empties_the_project` (the last tab closes to a zero-tab
  project; re-close → IndexOutOfRange). `close_tab_section_vocab` UNCHANGED (its non-last closes were always
  allowed) + its stale comment fixed (it named the renamed test).
- **app.rs:** `close_tab_at`'s error match → `Ok(removed) => {…reap+persist, UNTOUCHED…}` +
  `Err(TabError::IndexOutOfRange) => {}` (the status-flash arm removed; the reap `thread::spawn(drop)` intact
  — REQ-005); the #392 blank-center branch → a **muted hint panel** (centered flex-col: "Nothing open in this
  workspace" + ⌘T New terminal / ⌘P Open file / ＋ Create from a section header, via `colors.muted` + the
  #385/fleet muted-text idiom); the stale ⌘W comment updated (the guard dissolved; ⌘W on empty is an
  IndexOutOfRange no-op, never cascades to close_project).
- **The ONLY production consumer was app.rs's status flash** (grep-confirmed: every other LastTab/LastTerminal
  hit is now a historical comment); the compiler surfaced ONE extra test consumer (`close_last_is_refused`)
  I'd missed in the design's named list — rewritten.
- **Persistence:** NO code change — `serialize_shell` already emits `<root>\t<active_tab>` (no entries) for a
  zero-tab project; the round-trip works. (The round-trip TEST + the headless ⌘W test are Phase 4 — new REQ
  proofs, not compile-needed, per the implement discipline; the D3 guard-test rewrites WERE compile-needed +
  are the ticket's point, so they landed here.)
- **The empty Terminal header is #385-free** (the rail renders all 4 section headers regardless of row count)
  — no code needed; confirm in the driven capture.

## Inspect (Phase 3.5) — 2026-07-23 (Opus, /goal)
Two critics over the diff (tabs.rs + app.rs). Lenses: (1) empty-state totality + reachability + the
no-cascade-to-close_project; (2) the D3 test-rewrite honesty + the render + simplification/stale-docs.
**Both PASS, no HIGH; 1 MED + a batch of LOW stale-comment findings, all FIXED.** cargo check/fmt clean,
840 tests pass.

**F1 [MED, CONFIRMED, FIXED] — `close_tab_at`'s docstring described the DELETED status flash.** (Critic 2.)
The doc (app.rs:7417) still said "a refused close (the last terminal tab / the last tab) shows a status
flash" — but this ticket DELETED that flash (the arm is now `Err(TabError::IndexOutOfRange) => {}`). On the
very fn the diff changed → actively misleads the next reader. **Fix:** rewrote the docstring to the new
behavior (an in-range close always succeeds; a stale index is a silent no-op). → the doc-drift class below.

**F2 [LOW ×13, CONFIRMED, FIXED] — stale "test-only reachable until #395/#392" reachability comments.**
(Both critics.) #391/#392 left ~13 comments asserting the zero-tab / no-terminal state is "test-only
reachable until #395" (or "#392") — but #395 DISSOLVES the guards, so that state is now PRODUCTION-reachable
(a user empties a workspace). The code at every site is total (routes through #392's `try_active_tab`/
`try_workspace`), so this was cosmetic drift, not a bug — but leaving them asserting a false unreachability
is misleading (§21). **Fix:** swept all 13 (app.rs ×8, tabs.rs ×4, grid_layout.rs ×1) to "reachable on an
empty / no-terminal workspace since #395". → `PR-claude-dissolving-a-guard-must-sweep-the-reachability-comments-001`.

**F3 [out-of-scope, NOT fixed — a follow-up idea].** `launch_agent`/`split_file_pane`/`open_kind_pane`
silently no-op on an emptied project (now production-reachable; you can't split/agent into nothing). The
guards are CORRECT (pre-existing no-terminal behavior, not a #395 regression), and the empty-center hints
advertise only ⌘T/⌘P/＋ — all of which WORK (⌘T `new_terminal_pane` creates a tab, ⌘P the finder, the
Terminal/Editor/Browser ＋s create). A future polish could make split/agent seed the first tab. Noted, not
a #395 fix.

**Clean (verified concretely):** the guard-dissolve is COMPLETE (every `LastTab`/`LastTerminal` hit is now a
comment; `TabError` single-variant; `cargo check --tests` clean — the compiler backstop confirms no missed
consumer); `close_tab_at`'s match is exhaustive + the reap `thread::spawn(drop)`+persist UNTOUCHED (REQ-005);
**the empty state is TOTAL** — closing the last tab → zero-tab project → the render draws the hints and NO
bare `active_tab()` runs (Critic 1 traced: the `tab_count()==0` branch, the empty-`rect_list` pane loop runs
0 iterations, the status bar via `try_active_tab`), ⌘W on an already-empty project is a safe IndexOutOfRange
no-op, and closing the last tab NEVER cascades to `close_project` (D2 — the workspace stays open); the shrunk
`close_tab_refusal` is correct + mutation-complete (R1 idx==count, R3 idx<count, R5 idx==0 kill every `>=`
swap); the 3 rewritten tests HONESTLY pin the new policy (they close down to `tab_count()==0` — would fail
under the old refusal — not tautologies); the hint render is scoped ONLY to the empty branch (the else-chain
byte-identical); the hints are TRUTHFUL (⌘T→new-tab→`new_terminal_pane`, ⌘P→open-file-finder, ＋→section
menus — all grep-verified).

`status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`.

## Phase 4 — Validate (Opus, 2026-07-23, /goal) — GATE GREEN [diff] + DRIVEN CAPTURE ✅

**Tests added (4):**
- `grid_layout::serialize_zero_tab_project_round_trips` (REQ-003 persist) — a zero-tab `ProjectLayout`
  serializes to `0\n/x\t0` (no entries) and restores empty. cov/MSI 100 pure seam.
- `headless_drive::cmd_w_closes_the_last_tab_to_empty_headless` (REQ-002) — boot 1-terminal → ⌘W →
  `tab_count()==0` → tick_pump survives (the empty render is total, no panic).
- `headless_drive::boot_restores_a_zero_tab_project_empty` (REQ-003 restore) — boot a persisted zero-tab
  shell blob → `tab_count()==0` (restores empty, no force-seed).
- `tabs::project_empty_has_no_tabs` — `Project::empty` yields tab_count 0 + `try_active_tab` None. cov/MSI 100.

**A REAL RED, FIXED AT SOURCE (a validate-phase discovery — the boot-restore test caught it):** REQ-003's
RESTORE side was unmet — the boot restore loop (app.rs:2101) **force-seeded a terminal** onto any project
with no terminal tab ("A project must keep ≥1 terminal tab (the workspace() invariant)"), so a persisted
empty project restored with `tab_count 1`, not 0. But that invariant is exactly what #391/#392 DISSOLVED
(the app is total over a no-terminal / zero-tab project via the `try_*` twins). **Fix:** added
`Project::empty(name, root)` (tabs.rs, the `Workspace::empty` analog) and relaxed the restore loop — no
force-seed; a persisted zero-tab project restores through `Project::empty` (empty), a normal project through
the byte-identical `Project::new`+add_tab path. The unused boot PTY reaps at the existing "restored
terminal-less" arm (app.rs:2139). **Self-review (the delta is a boot-path change, made in validate):** the
`Some(first_tab)` arm is byte-identical to the old restore (normal projects unaffected); a cockpit-only
persisted project now restores as-is (correct + newly-relevant — pre-#395 you couldn't persist one, since
the last terminal couldn't close); an all-empty project → a Workspace with 1 empty project (NOT the launcher
— D2 holds, the launcher is zero-PROJECTS); `pane_blocks`/`spawn_session_in` still used by the terminal-tab
restore. Triple-proven (the `project_empty` unit + the `boot_restores` headless + the live pixel capture).

**Runs (actual):**
- `cargo nextest run -p marley` → **844 passed, 2 skipped** (840 + 4 new).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage **100% lines/functions** (tabs.rs
  incl. the shrunk `close_tab_refusal` + `Project::empty`; grid_layout.rs the round-trip), gate:5 mutation
  **MSI 100%** (the shrunk `close_tab_refusal`'s `>=` mutants killed by the rewritten truth table R1/R3/R5;
  `Project::empty` + the round-trip killed), gate:15 visual/AX green. The app.rs restore-loop + empty-center
  render + close_tab_at are masked shims (ACCEPTED-UNTESTABLE exclude).

**DRIVEN CAPTURE — ✅ PASSED (the #390 pre-seed technique; chad's eyeball-target, verified in pixels).** The
empty state is PERSISTED (a zero-tab project line) AND — now that the restore-empty fix landed — BOOT-reachable,
so the pre-seed works (no synthetic input needed). Pre-seeded an isolated `HOME=$ISO`'s
`.marley/config/settings.toml` `[workspace] shell = "0\n<proj>\t0"` (a zero-tab project), launched
`HOME=$ISO target/debug/marley`, `screencapture -l<winid>`, and READ the PNG. **It renders EXACTLY as
designed:** (a) the empty-center muted hint panel — "Nothing open in this workspace" + ⌘T New terminal / ⌘P
Open file / ＋ Create from a section header, centered; (b) the rail shows the OPEN workspace "empty-project"
with all four EMPTY section headers (Editor · Terminal · Panes · Browser — the #385 empty-header parity, incl.
the empty Terminal header); (c) it is NOT the launcher — the workspace + rail + top bar are present, status
bar "focus: cockpit" (the #392 neutral empty focus). The boot RESTORED the persisted empty project empty (the
fix, live). Killed my isolated instance (pid, only mine — chad's app was not running); never touched chad's
`~/.marley`. Capture: `scratchpad/iso-395/empty-workspace.png`.

**Pre-existing exclusions:** none. The pure seams (`close_tab_refusal`, `Project::empty`, the codec) carry
cov/MSI 100; the app.rs render/restore/close shims stay in the exclude (masked, driven-capture + headless
verified).

`status: Phase 4 — Validate PASS; ready for Phase 5 — Complete`.
