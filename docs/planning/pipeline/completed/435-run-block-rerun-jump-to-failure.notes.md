# Run blocks — per-block rerun + block-scoped jump-to-failure — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-435-run-block-rerun-jump-to-failure.md
- **Pipeline spec:** 435-run-block-rerun-jump-to-failure.spec.md

## Phase 1 — Plan
- **Request:** TICKET-435 — M33 Phase C thread 2 (shelf: the wedge's payoff
  loop): a #434 run Block's identity powers (a) per-block RERUN as a fresh
  identity-carrying Block and (b) one-keystroke block-scoped
  jump-to-failure with ref cycling. **Hard dependency: #434** — at draft
  time #434's queued spec does NOT exist yet (queued/ was empty); this plan
  grounds on TICKET-434's doc, the m33 shelf note, and the Zed fusion map
  §8, and P2 must re-read the real #434 spec/design before deciding.
- **Classification / tier:** work pipeline, one shippable slice (identity
  consumption + two affordances + one keystroke over shipped infra).
  React-first: zone A minimal POC mirror (affordance chrome only).
- **Recall (§18.3):**
  - **#330** (lessons.md:304): `open_and_place_caret` does NOT push the
    NavStack — every sibling jump captures `(path, caret)` BEFORE and
    pushes `NavLoc` explicitly after. `open_file_at` (app.rs:5999) is the
    same class: opens + `set_single_caret` only. → D2.
  - **#289 root lesson** (lessons.md:692): compare freshly-resolved refs
    against stored paths using the IDENTICAL resolution root
    (`self.project_root`), never a parallel field — note today's
    `jump_to_block_failure` resolves via `active_project().root`
    (app.rs:8932), the exact parallel-field shape the lesson warns about;
    design must not copy it. → D5.
  - **PR-1345** (prevention-rules.md:1345): a workspace-wide "scan every
    terminal" must iterate ALL grids, never `terminal_grid_index()` (the
    #295 cross-tab miss). Jump target selection inherits this iff it goes
    workspace-wide. → D6.
  - **#331/#327 stale-latch class** (430's D3 precedent; lessons.md:34 and
    the #401 raced-cache entry at lessons.md:331): derived state cached
    from an earlier snapshot goes stale/wedges — re-derive from current
    data at use time. → D3 (refs stateless; only the cycle cursor is
    state).
  - **#427 documented drop** (427 spec: "NO persistence — the writer drops
    the tab", Out: "Persistence across restart (documented drop)"): the
    transient-across-restart precedent for D1's identity lifetime.
  - **#40** idle-injection rule: `rerun_block` (app.rs:8958) only writes
    when `!is_command_running()` — identity rerun keeps the guard.
  - PR-claude-new-file-mutants (prevention-rules:747 per the 430 notes):
    new pure files `git add -N` before the `--diff` gate → REQ-007.
  - Mac-mini mutants policy (2026-08-13): local runs scoped `-f` + capped
    threads; full sweeps on the dev-box lane — baked into P4.
- **Discovery:**
  - **Block model** — crates/terminal_blocks/src/block.rs: `Block` carries
    `id: BlockId(u64)` :60-78 (id/index/session_id/`command: String`/
    `state`/`exit_code: ExitCode`/`prompt: PromptInfo{pwd,git_branch,…}`/
    private `output: Vec<StyledLine>`). NO run/task identity field exists.
    `rerun_command()` R30 :102 (Some iff Finished + non-empty),
    `copy_text` R29 :92, `last_rerunnable` :162, `output_text` R19 :113.
    `open_running` :181 is the SOLE id mint, invoked from apply.rs:66 on
    `DcsHook::Preexec` — so a #434 spawn via `write_command`
    (session.rs:347, R14) produces a block the APP never hands a value to
    directly; identity attach needs a correlation (pending-spawn queue
    consumed at block-open, command-text match, or a terminal_blocks
    metadata API). Both D1 residence options noted for design.
  - **Per-block affordances TODAY** — hover header actions exist
    (app.rs:23049-23134): ⧉ cmd / ⧉ out / ↻ run, #217 hidden-at-rest
    `group_hover("block-actions")`; ↻ calls `rerun_block` (app.rs:8958,
    #175, #40 guard, plain string `write_command`). Block context menu:
    context_menu.rs:115-127 — `BLOCK_MENU_ITEMS` (Rerun/copies) +
    `FAILED_BLOCK_MENU_ITEMS` ("Jump to Failure" LEADS, #213); dispatch
    app.rs:8851-8860. So #435 is NOT minting the first block affordance —
    it upgrades the grammar with identity-true rerun + keystroke jump.
  - **The ref fold** — links.rs: `first_failure_ref` :207 (first
    line-carrying ref, the #213 jump target), `parse_trace_frames` :216
    (#291 — ALL frames in order: the natural cycle list),
    `diagnostics_for_file` :258 (#289). Today's jump:
    `jump_to_block_failure` app.rs:8926 = `first_failure_ref(output_text)`
    → `open_file_at(resolve_under_root(active_project().root, …))` —
    menu-only, first-ref-only, NO cycling, NO NavStack push (#330 class).
    NavStack itself: editor_nav.rs `NavLoc`/`NavStack` :15-61
    (push/pop/depth).
  - **F8 cycle shape (#290/#310)** — keymap.rs:417-427: F8/⇧F8 are
    EDITOR-context (`next-diagnostic`/`prev-diagnostic`); handler
    app.rs:10753 walks `open_file_diagnostic_rows` via pure
    `code_view::next_diagnostic`/`prev_diagnostic` (:455/:464) with WRAP
    (test :1016). Terminal-context F8 is FREE — the natural chord
    candidate, Phase 2's call. Target-selection precedent:
    `rerun-last-failed` #292 (app.rs:11060, palette.rs:138) via pure
    `block_status::last_failure_block_index` :41 (rposition on
    StatusKind::Failure; `exit_status_kind` :26 classifies).
  - **Parity** — marley-web/docs/MARLEY-PARITY.md zone A row: "Terminal
    blocks | 03, 23 | components/TerminalView.tsx"; the POC file
    (artifacts/marley-ide/src/components/TerminalView.tsx) already models
    blocks + the block context menu with failed-only "Jump to Failure"
    (:110). Zone A = Marley-authoritative once shipped; new chrome is
    still built POC-first (the enforce-react-parity.sh contract).
- **Decisions:** D1–D6 locked in the spec: identity transient
  (scrollback-yes / restart-no, #427 precedent; residence = Phase 2 pick
  between Block metadata and an app-side BlockId map, correlation story
  required); explicit NavStack pushes (#330); stateless ref re-derivation
  with a BlockId-keyed cycle cursor as the only state (#331/#327 class);
  identity rerun through the #434 spawn path (never a bare string replay),
  #40-guarded; one resolution-root field (#289); press-time target
  derivation with PR-1345 iff workspace-wide. Chord, cycle-list fn
  (`parse_trace_frames` vs per-line first-refs), and identity residence
  are explicitly Phase 2 calls bounded by D1/D3/D6 + REQ observables.
- **Prior-art verdicts (§20, three legs, honest):**
  1. Behavior maps — **Warp HAS per-block rerun** (warp_architecture
     subsystems/03 capability table: Blocks = "…, duration, rerun";
     Marley's R30 already Matched) and NO jump-to-failure (no editor —
     03's fusion-wedge paragraph). **Zed**: 08-terminal-tasks-fusion.md —
     rerun is a TAB hover button (`rerun_button` terminal_view.rs:1082 →
     `Rerun{task_id, reevaluate_context,…}` re-spawning the stored
     `SpawnInTerminal`); jump-to-failure only via the generic hyperlink,
     never task-scoped; §8's table + "Concrete reimplementation" steps 4-5
     (rerun = re-spawn Block-metadata spec; block-scoped failure scan) are
     this ticket's brief. Clean-room: maps only, no AGPL/GPL source.
  2. Published — none found and none likely: no spec/protocol governs
     per-block rerun or block-scoped failure navigation (VS Code's
     shell-integration "rerun command" decoration is product behavior,
     not a published contract). Recorded as honestly empty.
  3. Permissive deps — none owns the seam: `terminal_blocks` (Block
     model, DCS hooks) and `links.rs` (ref fold) are Marley's own;
     `alacritty_terminal` models grids, not blocks. Integration of
     shipped infra; no new dependency.

## Phase 2 — Design

Designed at HEAD 5a596b1 — #434 SHIPPED (its Out line left identity to this
ticket; the spawn path `run_runnable` + the D6 mint are live substrate).

- **D1 PICK — the identity rides the BLOCK, staged in the model** (the
  `staged_prompt` shape exactly): `Block.run_tag: Option<String>` (opaque to
  `terminal_blocks` — the crate stays grammar/toolchain-free);
  `SessionModel.staged_run_tag: Option<String>` consumed by the NEXT
  Preexec's `open_running` (apply.rs — the SAME correlation point PromptInfo
  uses, so the tag binds to exactly the block the spawn's `write_command`
  births); `TerminalSession::stage_run_tag(tag)` pub. Session-scoped by
  construction (the BlockList is never serialized — the #427 documented-drop
  precedent); survives scrollback (any listed block keeps its field). The
  app-side codec lives in `marley_app::runnables` (pure):
  `encode_run_tag(&RunnableKind) -> String` (`"test:<path>"` / `"main"`) and
  `decode_run_tag(&str) -> Option<RunnableKind>` — round-trip unit-pinned.
  A stale/foreign tag decodes `None` → the block behaves as a plain block
  (fail-open to the OLD behavior, never a bogus rerun).
- **D4 — the ↻ becomes identity-aware; zero new chrome for rerun**: the
  existing #175 hover ↻ / menu Rerun handler branches — a block whose
  `run_tag` decodes routes through `run_runnable(&kind)` (staging a FRESH tag
  → the new tail block is itself a run block, #433-ready); a plain block
  keeps `rerun_block` byte-identical (REQ-006). The run-block IDENTITY
  MARKER (the one new chrome): a muted ▶ before the command text in the
  block header — POC-first (zone A minimal mirror).
- **THE CHORD — Terminal-scoped F8** (the #290 next-error convention on the
  other surface; keymap: F8 is Editor-scoped and "resolves nothing
  elsewhere" — free). The loop closes tight because #434's spawn FOCUSES the
  terminal pane (`jump_to_pane`): ▶ → watch → F8 (terminal focus) → editor
  at the failure → fix → ▶.
- **D6 — target at press time**: v1 scope = the active workspace grid
  (`try_workspace` — the same grid the #434 spawn targets); iterate panes
  `.0`-sorted, first pane owning a FAILED RUN block (run_tag present +
  `exit_status_kind == Failure`), its LATEST such block. Pure fn
  `latest_failed_run_block(blocks: &[(bool, StatusKind)]) -> Option<usize>`
  (per-pane; is_run + Failure, last index).
- **D3 — ordered refs, re-derived per press**: the cycle list must follow
  OUTPUT ORDER (the #290 walk convention), which `links::file_refs` breaks
  (it sorts for the producer). New `links::file_refs_ordered(output) ->
  Vec<(PathBuf, usize, Option<usize>)>` — per line, the File-with-line scan
  + the trace-frame fallback, FIRST-OCCURRENCE deduped, document order,
  cols carried; `file_refs` RE-EXPRESSES over it (ordered → strip cols →
  sort+dedup — one derivation, PR-1691). The cycle cursor:
  `run_jump_cursor: Option<(PaneId, BlockId, usize)>` app field — reset to 0
  when the derived target key changes, else `(c + 1) % len` (the pure
  `next_cycle_index(len, prior: Option<usize>) -> usize` in
  `marley_app::runnables`); discarded when no failed run target derives
  (clears on green — REQ-004 falls out of derive-at-press).
- **D2/D5 — the jump**: resolve each ref `resolve_under_root(project_root, …)`
  (the SAME field the #213 jump uses — one root family); open via the
  verify-the-landing `open_file_at` shape with the caret at line/col; the
  NavStack origin captured BEFORE and pushed AFTER a successful open (the
  goto-definition/#427 shape) — EVERY press pushes. Failed open / refless /
  no target → flash, zero side effects.

### File manifest
- **POC (FIRST):** `components/TerminalView.tsx` — the run-block ▶ header
  marker (mock `runTag` on a block; muted, before the command); capture.
- `crates/terminal_blocks/src/{block.rs,apply.rs,session.rs}` — `run_tag`
  field + staging + Preexec consume + `stage_run_tag` + units (both paths).
- `crates/marley_app/src/runnables.rs` — the tag codec + `next_cycle_index`
  + `latest_failed_run_block` + units.
- `crates/marley_app/src/links.rs` — `file_refs_ordered` + `file_refs`
  re-expressed + units.
- `crates/marley_app/src/app.rs` — `run_runnable` stages the tag; the
  ↻/menu-Rerun branch; the header ▶ marker; Terminal-F8 keymap row +
  `jump_to_run_failure` (target scan → ordered refs → cursor → jump + push);
  `run_jump_cursor` field.
- `crates/marley_app/src/keymap.rs` — the Terminal-scoped F8 row.

### Regression test plan
| REQ | Test |
|---|---|
| REQ-001 | drive: stage+seed a FAILED tagged block (the real correlation path) → the rerun branch → history carries the re-minted command + a fresh tag STAGED (the next birth is a run block); the original block untouched |
| REQ-002 | drive: F8 → editor at ref 1 (exact line/col), NavStack pushed (⌃- returns) |
| REQ-003 | unit `next_cycle_index` (None→0, wrap) + drive: F8 ×3 cycles refs and wraps, each press pushes |
| REQ-004 | drive: a PASSING tagged rerun supersedes → F8 flashes, cursor discarded |
| REQ-005 | unit (refless → None short-circuit) + drive: failed refless block → flash, no open, no push |
| REQ-006 | existing #175/#213 suites green; drive: an untagged block's ↻ routes the OLD rerun_block |
| REQ-007 | add -N; gate green (tag codec, cycle, ordered refs, target fn at cov/MSI 100) |
| parity | POC run-block ▶ header marker ↔ live block header; pixel-sample |

### Risks
- R1: the staged-tag correlation must not leak onto a USER-typed command that
  races the spawn — the stage happens in the same sync region as
  `write_command` (no interleave); a user's next command AFTER ours consumes
  nothing (the tag was already taken by our Preexec). The unit pins
  stage→preexec→take + a second preexec getting None.
- R2: `file_refs` re-expression must keep the #433 producer byte-identical
  (its unit table pins it).
- R3: F8 in the Terminal context must not shadow anything (keymap audit: F8
  rows are Editor-scoped; the PR-shadow sweep re-runs at inspect).

## Phase 3 — Implement

**React-first (built + verified BEFORE Rust):** `TerminalView.tsx` renders a
muted ▶ ahead of the command IFF `block.runTag` (new field on `TerminalBlock`);
`Workspace.tsx`'s #434 onRunCommand spawn now stages a tag onto the born block,
and the menu Rerun passes the source block's tag through (identity-aware).
Typecheck green. Captured `.playwright-mcp/435-poc-run-block-marker.png` and
READ it: two ADJACENT `cargo run` blocks — the old typed one plain, the
gutter-spawned one carrying the muted ▶ — the identity contrast exactly as
designed.

**Rust (per manifest):**
- `terminal_blocks`: `Block.run_tag` (opaque, doc'd), `SessionModel.
  staged_run_tag` consumed by the Preexec arm onto the block it births
  (take-once — a later user command consumes None), cleared at InitShell with
  the staged prompt; `TerminalSession::stage_run_tag` pub delegate. Drives
  seed tagged blocks via the REAL path: `stage_run_tag()` then
  `seed_finished_block_for_test` (the seed rides apply_hook's Preexec, which
  consumes the staged tag) — no seed-signature change.
- `marley_app::runnables`: `encode_run_tag`/`decode_run_tag` (round-trip;
  stale/foreign → None → plain-block behavior), `next_cycle_index`,
  `latest_failed_run_block`.
- `marley_app::links`: `file_refs_ordered` (document order, first-occurrence
  dedup on (path,line), cols carried) built over the NEW single match block
  `file_links_with_col_on_line`; `file_links_on_line` and `file_refs` are now
  projections of it (one derivation — PR-1691; #433's producer byte-identical:
  strip cols → sort + dedup).
- `app.rs`: `run_runnable` stages the encoded tag in the same sync region as
  `write_command`; `rerun_block` branches — decodable tag → D6 re-mint +
  history.record + fresh tag staged + viewport re-anchor, plain → byte-identical
  #175 path; `run_failure_target` (id-sorted pane scan → pure
  latest_failed_run_block) + `jump_to_run_failure` (ordered refs re-derived per
  press, cycle cursor `run_jump_cursor`, resolve_under_root + open_file_at,
  NavStack origin pushed per successful jump, flash on no-target/refless);
  the header ▶ marker (muted, `.when(run_tag.is_some())`) in BOTH block-header
  render sites; the `"run-jump-failure"` dispatch arm.
- `keymap.rs`: Terminal-scoped F8 → `run-jump-failure` (disjoint from the
  Editor F8 rows — the #265 ⌘F precedent).

**Deviations from design (with reason):**
1. `latest_failed_run_block` takes `&[(bool, bool)]` (is_run, failed) not
   `(bool, StatusKind)` — the caller folds `exit_status_kind == Failure` to a
   bool; simpler pure boundary, same semantics.
2. The identity-aware rerun respawns in the block's OWN pane (the ↻ the user
   clicked) rather than routing through `run_runnable`'s idle-pane walk — the
   walk exists for spawns with NO origin pane; re-targeting a rerun to another
   pane would surprise. The #40 idle guard + flash-less no-op semantics of
   rerun_block hold for both branches.
3. The ▶ marker landed in BOTH header render sites (the main pane render and
   the split render) — one site would desync the surfaces.

`cargo check --workspace --tests` green; `cargo fmt --all` applied; POC
typecheck green.

## Inspect (Phase 3.5)

Three parallel critics (correctness / state-lifecycle / reuse-render-parity)
over the Phase 3 diff. Findings, verdicts, fixes:

1. **[HIGH — real] REQ-004 violated: a passing rerun did not clear the F8
   target** (correctness). `rposition(is_run && failed)` targeted the latest
   failed run block EVER — after fail→rerun→pass the OLD failure still
   targeted. FIX: `latest_failed_run_block` now takes execution-ordered
   `(Option<&str> tag, StatusKind)` and skips a Failure whose tag has a LATER
   same-tag `Success` in the pane (green retires the identity; a later
   same-tag Failure re-arms; Running has not superseded). Cross-pane
   supersession = recorded v1 bound (blocks carry no cross-session order —
   no global clock — and the identity ↻ reruns in its OWN pane, so the
   REQ-004 scenario is exactly per-pane). This REVERSES Phase-3 deviation 1:
   the design's `(tag, StatusKind)` shape was needed after all.
2. **[MED — real] stale keymap pin** (correctness; REPRODUCED red):
   `f8_diagnostic_nav_editor_scoped_and_unique` pinned F8→None on TERM, and
   the two roster censuses (82-chord / 38-scoped) were off by the new row.
   FIX: pin updated to `Some("run-jump-failure")` (+ ⇧F8 stays None), both
   censuses updated with the `#435 ×1` tally. Lesson re-learned: `cargo check
   --tests` compiles, it does not RUN — the phase-3 claim "tests compile" was
   never "tests pass".
3. **[MED — real] failed open pushed NavStack + moved the caret in the WRONG
   buffer** (correctness). `open_file_at` returns `()` (missing file = silent
   no-op) and the caret arm then ran against the still-active previous
   editor. FIX: route through the #330 bool-returning `open_and_place_caret`
   (canonical landing check + `caret_for_line_col` + deferred center); on
   miss → flash, NO push, NO cursor advance (D2's zero-side-effects honored).
4. **[MED — real] the seed's embedded InitShell wiped a pre-staged tag on a
   fresh session** (correctness + state, found independently by both). FIX:
   `seed_finished_block_for_test` registers via direct `registry.insert`
   (epoch-neutral; live InitShell semantics untouched) — drives can stage
   then seed on a fresh model and the tag lands.
5. **[MED — real] bare F8 stole keys from alt-screen TUIs / running
   commands** (correctness). The keymap arm resolves BEFORE the R40 raw
   route; F8 is the first UNMODIFIED Terminal row ever (mc binds F8=delete).
   FIX: the ladder's keymap arm falls through (no dispatch) for
   `run-jump-failure` when the focused terminal `is_alt_screen() ||
   is_command_running()` — the raw route streams the key. The tight loop is
   untouched (after a failed run the session is idle, cooked, normal screen).
6. **[MED — real] a FAILED `write_command` left a stale staged tag** (state):
   the next unrelated command's Preexec would take it — fake ▶, and its ↻
   would REPLACE the user's command with the re-mint. FIX: both producers
   stage only when `write_command(&cmd).is_ok()` (safe — Preexec is consumed
   only in this thread's pump; nothing interleaves).
7. **[LOW — real] mislabel windows: sub-pump double-stage overwrite +
   already-buffered Preexec binding the tag** (state + correctness). FIX
   (kills the whole class): the stage is a `(tag, command)` pair and the
   Preexec MATCH-BINDS — a mismatching Preexec leaves the stage for the
   spawn's own; `stage_run_tag` refuses to overwrite a still-staged pair. A
   tag can now only ever label the exact command it was minted for —
   mislabel impossible by construction; worst case is a LOST identity
   (plain block), fail-safe.
8. **[minor — real] duplicated enumerate+sort pane walk** (reuse). FIX:
   extracted `workspace_pane_ids_sorted()`; `idle_workspace_panes` filters
   it, `run_failure_target` scans it unfiltered (F8 is read-only — a busy
   pane still holds its blocks).
9. **[nit — real] needless `refs[ix].clone()`** (reuse). FIX: `swap_remove`
   (in-bounds by the `% len`).
10. **[parity] marker gap** (reuse): POC `mr-1.5` = 6px vs `gap_1` = 4px.
    FIX: `.gap(px(6.))` at both header sites. Remaining recorded deltas for
    the MARLEY-PARITY entry: POC opacity-60 muting vs `colors.muted`; POC
    tooltip `title="Run block"` (no gpui tooltip on the marker); POC rerun
    replays the stored command+tag vs Rust re-mints through the codec
    (identical today — deterministic mint + round-trip codec).
11. **[rejected] cursor staleness on pane close / project switch** (state):
    verified structurally inert — PaneId/BlockId allocators are monotone
    (never recycled), the pair filter mismatches any dangling cursor, and
    `% len` keeps any stale index in-bounds; self-heals at next derive. No
    clearing hooks added (optional hygiene, not a bug).
12. **[info] `first_failure_ref` ≠ `file_refs_ordered().first()`** (reuse):
    they differ on Python `File "x.py", line N` traces (the #213 producer is
    scan-only). Unifying would make #213 consumers trace-aware — a behavior
    change for a future ticket, recorded here, not done.
13. **[info] a spawn racing a fresh shell's InitShell loses its tag** —
    fail-safe by direction (plain block, never a mislabel); documented on the
    field.

Verification after fixes: `cargo nextest run --workspace` — **2278/2278
passed** (the keymap pin, both censuses, terminal_blocks 140/140 included).
`cargo fmt --all` clean. POC `tsc --noEmit` clean (critics re-verified).

Lenses covered: correctness (AC/edges/panics/borrows), state integrity
(staging lifecycle, id reuse, serialization, epochs), security/provenance
(no new spawn surface — the tag never reaches a PTY; encode/decode is a
closed grammar; §20 clean), reuse/render (duplication, geometry, parity).

## Phase 4 — Validate

**Units (new, all green):** runnables — codec round-trip + foreign→None
(REQ-001/006), `next_cycle_index` start/advance/wrap + stale-prior in-bounds
(REQ-003), the REQ-004 supersession TABLE (green retires, different tag
doesn't, later failure re-arms, Running hasn't superseded, order matters,
plain blocks invisible); links — `file_refs_ordered` document order +
first-col dedup + trace union + the `file_refs` projection pin
(REQ-002/003); terminal_blocks — match-bind/take-once/refuse-overwrite,
InitShell discard, seed-after-stage lands on the FIRST seeded block (the
inspect-MED pin), and the crate-local session-delegate pin (the L-433
per-crate-mutation lesson).

**Drives (3 new, green first run):**
- `run_block_rerun_is_identity_aware_headless` (REQ-001+006): plain failed
  block → rerun-last-failed → history EMPTY (old path), stage-less Preexec
  births PLAIN; tagged failed block seeded with a DIFFERENT spelling → the
  rerun records the D6 RE-MINT (proving re-mint-through-tag, not command
  echo), the fresh staged tag binds the next matching Preexec's block, the
  original block untouched (tag + exit 101 intact).
- `run_jump_failure_jumps_cycles_and_pushes_headless` (REQ-002+003): F8 →
  editor at ref 1 in OUTPUT order (b.rs before a.rs) at the EXACT 1-based
  2:5 → 0-based (1,4); press 2 → ref 2 + NavStack push; press 3 → WRAP +
  push (depths 0→1→2; the no-editor first press pushes nothing).
- `run_jump_supersession_and_refless_fail_closed_headless` (REQ-004+005):
  fail(test:x)+pass(test:x) → flash "No failed run", nothing opened,
  nothing pushed; fail(test:y) refless → flash "No file refs in failed
  run", still nothing.

**Suites:** `cargo nextest run --workspace` **2289/2289** + doctests clean.

**LIVE drive (bundled app; captures READ):**
- Boot layout crafted into `~/.marley/config` (user's settings backed up +
  RESTORED after): one Marley-root project, Terminal tab + Code tab on a
  TEMPORARY failing integration test (`crates/marley_app/tests/
  aaa_demo_435.rs`, deleted after) — the app booted straight into the
  editor with the #434 gutter ▶ on the test row (`435-live-editor.png`).
- Mouse-click on the ▶ → the REAL `cargo test demo_fails_435` ran and
  FAILED in the revealed, focused terminal: `435-live-marker.png` shows the
  block header **✗ ▶ cargo test demo_fails_435** — THE run-block identity
  marker LIVE (the staged tag survived the real InitShell + match-bound at
  the real Preexec), with the real panic ref
  `crates/marley_app/tests/aaa_demo_435.rs:6:5` linkified in the output.
- **F8 live: NOT deliverable on this machine today — stated explicitly.**
  Synthetic UNMODIFIED keys (plain typing AND F8, via drive.swift CGEvent
  HID-tap, System Events keystroke, and key-code with the fn flag) never
  reached ANY app while ⌘-chords (⌘T verified — through the #435-restructured
  keymap arm, live) and mouse clicks worked; a second interactive session was
  actively using the machine (focus contention), and typed probes risked
  cross-session injection, so keyboard attempts were STOPPED after 4 tries
  (the harness rabbit-hole rule). Evidence closing the gap: the F8→
  `run-jump-failure` binding + Terminal scoping is unit-pinned; the ladder
  dispatch of Terminal rows is the shipped #265 machinery, and the ONE new
  branch in it demonstrably dispatches (the live ⌘T went through it); the
  handler's complete behavior (target, order, cycle, wrap, push, flashes,
  supersession) is pinned by the three headless drives ON the real app
  entity. Follow-up recorded in the ticket: re-verify the literal keypress
  live on an idle machine.
- Machine state restored: app quit, user's `~/.marley/config` restored, the
  temp failing test deleted. `drive.swift` gained `f8` + fn-flagged F-key
  posting (harness-only).

**Parity pair (pixel-sampled, not eyeballed):**
`.playwright-mcp/435-poc-run-block-marker.png` (POC: adjacent plain vs
▶-marked `cargo run` blocks) ↔ `scratchpad/435-live-marker.png` (live).
Standing families EXACT: POC body (11,12,15) ↔ live pane (14,15,17); live
header band (25,26,28) = the band family's Rust side. The ▶: POC srgb
(90,92,101) (muted × opacity-60) ↔ live (109,110,112) (`colors.muted` full
opacity) — the inspect-recorded muting-mechanics delta, same muted-glyph
grammar (▶ before the command, 6px gap both sides after the `.gap(px(6.))`
parity fix). Verdict: **PASS**.

**Gate:** four `--diff` iterations, all fixes at source:
1. RED gate:4+5 — the missed mutant `i+1 → i*1` in
   `latest_failed_run_block` was EQUIVALENT (the delta element is the
   Failure under test, never a Success) → re-expressed as the reverse-walk
   `greened` set (no index arithmetic = no mutant surface; also O(n)); the
   uncovered `apply.rs:126` (grammatically-unreachable current_mut None arm
   right after open_running) → flattened to take-then-assign.
2. RED gate:4 — 2 "missed lines" in headless_drive that NO line view showed:
   the uncalled-INSTANTIATION class — per-item closures (`.map(to_string)`,
   `.any(==)`) over the plain-rerun history vec that is asserted EMPTY, so
   the closures never run and llvm counts their bodies missed. Fixed by
   asserting `is_empty()` directly (stronger assert, zero dead closures).
3. **GATE GREEN [diff] — 15/15**; receipt written (gate4.log).

No pre-existing failures touched; none in scope.

## Phase 5 — Complete

- §21 (a) CHANGELOG: Unreleased → Added (the fusion wedge closes: identity,
  match-bound staging, identity-aware ↻, Terminal-F8 with supersession +
  PTY-yield).
- §21 (b) Architecture: editor.md gains the #435 section (the identity
  architecture, the staging law, the supersession derivation, the chord law).
- §21 (c) Parity sync: MARLEY-PARITY Terminal-blocks row — #435 LANDED (the
  ▶ grammar both sides; staging-story + muting/tooltip deltas recorded; gap
  6px both after the Rust parity fix). POC was built first and needed no
  back-port.
- Knowledge (§19): AD-claude-435 (identity architecture + staging law +
  supersession-at-derive + the chord law), L-claude-435 (uncalled-
  instantiation coverage class; equivalent-mutant re-expression; the
  machine-in-use live-drive recipe). F-claude-435-a +
  PR-claude-unmodified-terminal-chords-yield-to-the-pty-001 were appended at
  inspect.
- Ticket → closed; pipeline pair → completed.
- Follow-up recorded in the ticket: re-verify the literal F8 keypress live on
  an idle machine (the handler + binding are pinned; only the physical
  delivery leg was environment-blocked at validate).
