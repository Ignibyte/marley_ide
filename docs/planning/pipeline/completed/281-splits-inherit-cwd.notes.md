# 281-splits-inherit-cwd — Notes

- **Forge ticket:** #281 ff81ab5c-41a3-4298-8dd4-b2e7fc800f1a
- **AAR:** 0d5b24ce-335a-4209-9965-d5c92d4e97e4
- **Local ticket doc:** docs/planning/tickets/open/TICKET-281-splits-inherit-cwd.md
- **Pipeline spec:** 281-splits-inherit-cwd.spec.md
- **Agent session (owner):** ede913c3-d048-4f39-ad2b-b21cef1efc8e

## Phase 1 — Plan (combined with Phase 2)
- /goal batch ticket 8 of 10 (terminal polish #4).
- **Recon:** the spawn sites are exactly TWO fns — `new_terminal_pane`
  (app.rs:3432; reached by ⌘T "new-tab":4464, ⌘D "new-terminal":4497,
  and the "+" button) and `split_focused_pane` (:3452; ⌘⇧L/⌘⇧J via
  keymap + MenuAction::SplitRight/Down). Both hardcode
  `active_project().root` (#160). `cwd_or_root` (:617) is the #205
  validator (`Some(Some(c)) non-empty ∧ is_dir`); `PromptInfo.pwd:
  Option<String>` (block.rs:48) is the live #201 source via
  `focused_terminal().session.current_prompt()`.

## Phase 2 — Design
- **Pure (app.rs, next to cwd_or_root):**
  `fn valid_dir_or(cand: Option<&str>, root: &Path) -> PathBuf` —
  `Some(c) if !c.is_empty() && Path::new(c).is_absolute() &&
  Path::new(c).is_dir() => c.into(), _ => root.into()`.
  `cwd_or_root` body becomes
  `valid_dir_or(cwd.and_then(|c| c.as_deref()), root)` — ONE
  authority (D3; the absolute check strengthens #205 restores — a
  relative persisted cwd resolving against the app process cwd was a
  latent wrong; #205's tests use absolute tempdirs so they stay
  green, plus a new relative-rejection row).
- **Sites:** both fns read
  `let live = self.workspace_mut().focused_terminal().and_then(|t|
  t.session.current_prompt().and_then(|p| p.pwd.clone()));` then
  `let cwd = valid_dir_or(live.as_deref(), &root);` and spawn via
  `spawn_session_in(&cwd, …)`. No other site changes (open-project /
  launcher / restore / agents / remote audited at recon — separate
  fns).
- **Manifest:** app.rs (the core + 2 fn threads) ·
  headless_drive.rs (flows).
- **Test plan:** REQ-001 valid_dir_or table (Some+absolute+is_dir →
  cand; relative real dir → root; missing → root; empty → root;
  None → root; a FILE (not dir) → root) + cwd_or_root delegation
  vectors (the #205 rows re-asserted through the new core);
  REQ-002/003 headless: cd into a tempdir subdir via the PTY (write
  a prompt pwd via the session's shell integration — or set the
  prompt pwd via the masked test seam if cd-latency flakes; assert
  the SPAWNED session's cwd — `spawn_session_in` receives it; observe
  via the new pane's prompt pwd after boot or the session's spawn
  arg… the headless flow asserts through the spawn decision by
  reading the new pane's initial pwd once shell integration reports;
  bounded parked polls) + the deleted-dir fallback.
- **Risks:** R1 prompt-pwd staleness (pwd updates on prompt render —
  a cd that hasn't re-prompted yet inherits the OLD dir; identical to
  Warp's behavior, accepted); R2 the headless PTY prompt-pwd timing —
  bounded polls like the #274 flow.

## Phase 3 — Implement
- `valid_dir_or` (the ONE authority: non-empty ∧ absolute ∧ is_dir,
  else root); `cwd_or_root` delegates; both spawn fns read the
  focused prompt pwd and thread through it. No deviations. check +
  fmt clean; suite 1001/1001 (the #205 restore vectors stay green
  through the delegation).

## Phase 3.5 — Inspect
### Ledger (critic started; hit the model usage limit after confirming
the #205 delegation vectors green — the remaining hunts finished
INLINE by the owner with the same evidence standard)
| # | Finding | Verdict | Outcome |
|---|---|---|---|
| S1 | Site inventory (every `spawn_session_in` caller): 591 boot fallback (process cwd) · 1056/1127/1216 the #205 restore paths (own persisted cwds) · 2214 `launch_agent` · 3631 open_project_path (a new project at ITS root) · 3454/3485 the two THREADED sites. | complete | `launch_agent` stays at the PROJECT ROOT deliberately (an agent's context is the repo; the ticket scoped splits/tabs only — a future "agent inherits cwd" is its own product call, noted). All others correctly untouched. |
| S2 | ⌘T from an EDITOR tab inherits the HIDDEN pane's cwd (the #71 fallback). | KEPT, documented | Unlike #278's HIGH this is a READ-ONLY inherit — no hidden state mutates; the most-recent working terminal is the natural cwd donor (the Warp new-tab convention). |
| S3 | Security: the pwd flows into `SessionOptions.cwd: PathBuf` → the PTY exec cwd — never shell-interpolated. | clear | — |
| S4 | Edges: symlinked dir accepted (is_dir follows — every terminal's behavior); a mode-000 dir passes is_dir and the spawn's `if let Ok` drops the pane (the pre-existing bad-cwd class, bounded); whitespace-only → root. | clear | REQ-001 rows cover the && clause family (empty / relative-real-dir / absolute-missing / file / None). |
| S5 | The #205 delegation: the 6 matched restore/cwd tests GREEN (verified by the critic pre-limit); the one behavior change (a RELATIVE persisted cwd that resolved against the app cwd) is the intended D3 strengthening — no test pinned the old accident. | clear | — |
| S6 | `current_prompt().pwd` is the #201 LIVE tracked pwd (cd-following staged context), not the last block's. | clear | — |

## Phase 4 — Validate
- **Unit:** `valid_dir_or_decision_table` — real-tempdir vectors
  (winning row · None · empty · RELATIVE-real "." · absolute-missing ·
  a FILE · whitespace-only) killing each `&&` clause + the
  body-constant mutants, plus the cwd_or_root delegation rows (the
  #205 shape re-pinned through the new core).
- **Headless:** `new_tab_inherits_live_cwd_headless` — REAL PTYs
  end-to-end: cd into a tempdir subdir → the shell-integration pwd
  tracks it (bounded polls) → ⌘T → the NEW shell starts (and reports)
  in the inherited dir. TWO test-side traps fixed en route: (1) the
  post-⌘T read must use the ACTIVE tab's grid (terminal_grid_index
  points at the FIRST terminal tab); (2) macOS /var→/private/var —
  a cd'd shell keeps the LOGICAL path while a fresh shell reports
  PHYSICAL getcwd → the comparison canonicalizes both sides.
- **Gate:** first run RED ×2 — gate:14 brand-scrub (my comment named
  the reference terminals; rephrased to "the universal convention")
  and gate:4 on the flow's early-break loop shapes (poll bodies
  restructured to always-execute while-loops; the cd closure
  simplified). Re-run: **GATE GREEN [diff] — 15/15**; suite
  1003/1003; no exclusions.
- **Driven capture:** the screen probe is non-black now (max RGB 38 —
  waking/dim) but not confirmably interactive; the cd+split capture
  stays in the unlock re-verify batch (one minute covers all six
  pending items when confirmed live).

## Phase 5 — Complete
- CHANGELOG under "### Added"; app_shell.md's spawn section notes the
  inheritance + the ONE validation authority.
- AAR 0d5b24ce submitted (completed). Forge #281 closed (done); local
  ticket → closed/.
- Lessons: (1) fmt-reflowed anchors — after any cargo fmt, re-read
  before batch-replacing (two aborted batches); (2) macOS logical-vs-
  physical paths in PTY tests — canonicalize both sides; (3) the
  active-tab-vs-first-tab read distinction in multi-tab flows.
