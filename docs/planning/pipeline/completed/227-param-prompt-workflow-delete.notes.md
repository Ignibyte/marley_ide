# 227 — param-prompt modal + workflow delete — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-227-param-prompt-modal-workflow-edit.md
- **Pipeline spec:** 227-param-prompt-workflow-delete.spec.md

## Phase 1 — Plan
- **Request:** batch position 7 (last): the #204 follow-ups. Auto-approved.
- **Classification / tier:** feature, single slice after the recorded scope
  cut (D3: rename/reorder/template-edit out).
- **Recall (§18.3):** invoke_workflow (app.rs:3703) inserts the raw template
  — the fork point; the save-append at :17395 embodies the R1 id↔index
  coupling ("append-only was safe there — v1 workflows never delete", the
  :682 comment); rebuild_pane_commands (:7711) is the rebuild precedent;
  substitute/params_of pure + tested (#204); naming_workflow draft
  (:3720) + naming_overlay_card (Recipe C, #414) + the #415 roster rule are
  this batch's substrate; text_input_blocked + the choke-point test pattern
  (#399/#378) for draft discipline; F-#203 pump-dirty N/A (no pump state).
- **Sweep:** Warp's handlebars workflow-argument templating recorded in the
  behavior maps (06-platform-settings-infra.md:147) — the class matched;
  the modal is Marley's arrangement of shipped chrome. FinderState is the
  picker, verbatim. gpui: no form primitive. POC: NamePaneCard.tsx is the
  React reference for the card family; the POC has no workflow persistence →
  the React half uses a fixture row.
- **Decisions:** D1 insert-not-run; D2 delete = full block rebuild + the
  SAVE path converts to the same builder (one owner); D3 scope cut; D4 both
  overlays join the roster; D5 Recipe C chrome. See spec.

## Phase 2 — Design

**Approach (§20 confirmed as planned).** All state pure-first; the app shim
arranges shipped chrome. Two new transient overlays, both #415-roster members.

**Signatures / placements:**
- `workflows::ParamPrompt` — `{workflow_index: usize, name: String,
  template: String, params: Vec<String>, values: Vec<String>, current: usize}`;
  `new(index, &Workflow) -> Option<Self>` (None when params empty);
  `push_char(char)` / `backspace()` (edit values[current]);
  `advance() -> Advance::{Next, Done}`; `filled() -> Result<String,
  SubstituteError>` (zip params×values → BTreeMap → substitute — total: every
  param gets a value, so MissingParam is unreachable from the modal, REQ-003).
- app.rs state: `param_prompt: Option<ParamPrompt>` + `workflow_picker:
  Option<FinderState>` (the house fuzzy picker, reused verbatim — selection
  via `results()` index, the symbols-picker idiom; `chosen()` is Path-typed,
  not used).
- Key arms: BOTH in the root `on_key_down` listener beside the
  naming_workflow arm (:17375 region — the draft-family block): param_prompt
  {escape → None; enter → advance/Done → `insert_at_prompt(filled)` → None;
  backspace; space/printables → push_char}; workflow_picker {escape; enter →
  delete picked + persist + rebuild; up/down; backspace; printables → push}.
- `insert_at_prompt(&mut self, text: &str)` — FACTORED from invoke_workflow's
  insert block (buffer.edit + caret advance), reused by the modal's Done
  (masked, like its source).
- `invoke_workflow` forks: `ParamPrompt::new` Some → `close_transient_overlays`
  + open the modal; None → today's substituted insert (unchanged path).
- `begin_workflow_delete()` — empty workflows → `status_flash("no saved
  workflows")`; else roster-close + `workflow_picker = Some(FinderState::new())`.
  Reached via a NEW static palette command "Delete Workflow…" (action string
  `"delete-workflow"`, the `save-workflow`:3646 dispatch idiom; id in the
  static block).
- `rebuild_workflow_commands(&mut self)` — retain-out the
  `WORKFLOW_BASE..WORKFLOW_BASE+1000` id range, re-push from
  `self.workflows.iter().take(1000)` in order (the boot loop :2290's exact
  shape). Called at DELETE and REPLACING the save-append (:17392-17404) —
  D2's one-owner (append-drift dies).
- Renders: modal = `naming_overlay_card` (Recipe C) — header
  "Workflow: {name}" (muted), one row per param (`{param}: {value}`, the
  CURRENT row foreground + the ▏caret glyph, others muted), hint row
  "Enter — next · Esc — cancel" (muted); picker = `quarter_overlay_card`
  (Recipe B) + `icon_label` header "Delete workflow…" + name rows (selected
  = accent fill, the finder row idiom) + the command as a muted suffix.
- Membership sweeps (ALL required): `close_transient_overlays` roster ×2;
  `text_input_blocked` (:10915) ×2; `OverlayStates` snapshot (:10952) ×2 —
  which EXTENDS browser.rs `overlay_is_up` + the EXHAUSTIVE 26-state flip
  table to 28 (the table's name is its contract); the naming-draft
  choke-point pattern gains the two flips.

**Manifest.**
- marley-web (FIRST): `components/overlays/ParamPromptCard.tsx` (the Recipe-C
  card arrangement, NamePaneCard's look) + a fixture `Workflow: deploy
  {{host}} {{env}}` palette row in `CommandPalette.tssx`/App state to open it;
  typecheck + drive + screenshot READ.
- `crates/marley_app/src/workflows.rs` — ParamPrompt (+ units at validate).
- `crates/marley_app/src/app.rs` — the two states, two key arms, fork,
  factored insert, begin/delete/rebuild fns, two renders, roster/blocked/
  snapshot memberships, the static command, the save-append conversion.
- `crates/marley_app/src/browser.rs` — OverlayStates fields + overlay_is_up +
  the flip table 26→28.

**Test plan.**

| REQ | Test | Assert |
|---|---|---|
| REQ-001 | headless: no-param invoke inserts; with-param invoke opens the modal (snapshot field) and inserts NOTHING | both arms |
| REQ-002 | ParamPrompt units (push/backspace/advance/Done full-value, multibyte, empty-template guard) + headless drive: open → type "web" → enter → type "prod" → enter → prompt text == substituted | the machine + the flow |
| REQ-003 | unit: filled() with all-empty values == template with empties (Ok, not MissingParam) | zip totality |
| REQ-004 | headless: save 3 → delete the middle via the picker → invoke the id at index 1 → the SURVIVING second workflow's command inserts; settings reload asserts len 2 | R1 rebuild |
| REQ-005 | roster: the #415-tightened smoke pattern — open each, `close_transient_overlays`, assert gone; `text_input_blocked` flips; the browser.rs flip table 28/28 | membership |
| REQ-006 | suite + gate `--diff` | green |
| parity | the React card capture (READ at implement) ↔ the Rust render reviewed 1:1 at inspect; live pixels ride #417 | recorded |

**Risks.** The key-arm ORDER (the two arms must precede the pickers/terminal
fall-through — beside naming_workflow is correct by construction: that arm
already precedes them). The save-append conversion changes a shipped path —
REQ-004's drive covers save+delete+invoke together. FinderState's results()
ordering under filter — the delete maps through `results` (the shipped
finder discipline), never raw index.

## Phase 3 — Implement
- **React FIRST:** `ParamPromptCard.tsx` (the NamePaneCard family shape) +
  `paramPrompt` AppState + the Workspace key branch (the paneNaming
  discipline) + the palette fixture row "Workflow: deploy" (`{{host}}/{{env}}`
  modeled). Driven at 5173: fixture invoked, "web01" typed, Enter advanced,
  "prod" typed — screenshot READ (scratchpad 227-react-param-prompt.png):
  muted caption, filled host line muted, current env line foreground with
  the ▏caret, the hint row. Look settled; typecheck clean.
- **Rust:** `ParamPrompt` machine in workflows.rs (new/push_char/backspace/
  advance/filled + `Advance`); app.rs — the two states + inits,
  `DELETE_WORKFLOW_ID = CommandId(32)` (**31 was TAKEN** by the #406
  browser-reload — caught by the pre-write grep; the design said "31 next
  free" from a stale comment) + static row + dispatch arm; `invoke_workflow`
  forks to the modal; `insert_at_prompt` factored; `begin_workflow_delete` /
  `delete_workflow_at` / `rebuild_workflow_commands` (retain + re-push, the
  one builder); the save-append at the naming_workflow Enter arm CONVERTED to
  the rebuild (D2); the two key arms beside the draft family; the two renders
  (Recipe C modal; Recipe B picker with query-in-header + name·command rows);
  roster ×2, text_input_blocked ×2, overlay_snapshot ×2; browser.rs
  OverlayStates 26→28 (fields, overlay_is_up, the all-false builder, the flip
  table renamed `..._28_states` with both flips).
- `cargo check --all-targets` 0 errors; clippy clean; fmt clean.
- Deviations: the id 31→32 correction (recorded above); none else.

## Phase 3.5 — Inspect

Three critics (machine+arms; R1/persistence; roster+reuse+provenance), all
tracing to source and running tests.

| # | Sev | Finding | Verdict | Action |
|---|---|---|---|---|
| F1 | HIGH | browser.rs:339 count assert still `26` — the renamed flip-table test FAILED deterministically (both critics ran it) | REAL | Fixed → 28, "frozen" dropped. |
| F2 | MED | The two mouse DIFF-openers (⌘-click agent row; git-panel path click) set `diff = Some` without roster-closing — the diff painted over a keyboard-live hidden modal; blind Enter in the hidden picker DELETES+persists | REAL (the class inverted: our arms sit above, so the modal keeps keys while the diff paints on top) | Fixed — both openers call `close_transient_overlays()` first. F- appended. |
| F3 | MED | `begin_fleet_dispatch`'s refuse-guard listed only naming_workflow/naming_pane — the seat "send" mouse chip opened the dispatch card keyboard-dead over the new modals | REAL | Fixed — guard gained both states. Same F-. |
| F4 | MED | The picker render diverged from the family (fused query header, muted unselected rows, a no-matches row) — voiding the spec's "verbatim shipped look, no React half" waiver | REAL | Fixed — family-true: static icon_label header, the query on its own `›` line (the launcher precedent), default-foreground rows, no-matches row dropped; comment updated. |
| F5 | LOW | `filled()`'s Err (hand-desynced settings `params`) silently inserted nothing, values lost | REAL | Fixed — Err flashes "workflow params out of sync — re-save it". |
| F6 | LOW | `ParamPrompt.workflow_index` written-never-read with a doc claiming the opposite | REAL | Fixed — field dropped (`new(&Workflow)`); the struct doc now records the captured-template-by-design stance. Spec field-list deviation noted. |
| F7 | LOW | Stale "frozen 26-state" in app.rs's builder doc | REAL | Fixed → 28. (embedded-browser-model.md's "26" sits in the PRE-EXISTING dirty intake files — not touched.) |
| F8 | LOW | The history-picker Enter carried the exact inline duplicate of `insert_at_prompt` | REAL | Fixed — converted to the helper. (finder-pick/paste/ghost-accept share the idiom inside running-command guards — recorded as optional follow-up, not scope-crept.) |
| F9 | LOW | ParamPrompt ships untested (the pure module is cov/MSI-floored) | REAL, PHASE-4-OWNED | Validate MUST deliver the REQ-002/003 units + the headless drives + the choke-point flips. |
| F10 | INFO | The connect band mints uncapped — ≥2001 hosts would collide into [3000,4000) and the rebuild's retain would strip them (absurd config; pre-existing) | ACCEPTED | Recorded; a `.take(1000)` on the connect loop is the note-only fix. |
| F11 | INFO | Picker selection can exceed the 20 rendered rows (>20 workflows) — byte-identical to the shipped finder posture | ACCEPTED (family-consistent) | Windowing is a shared family follow-up. |

Verified sound: the machine's invariants (current bounded, values born
params-length, zip totality, multibyte); arm order (nothing above the new arms
can co-open — naming_workflow opens only from palette Enter); the palette
dispatch order (roster-close in the fork precedes the trailing palette close);
the R1 rebuild (half-open [3000,4000) clips no neighbor; order stable at
render because add/pane bands strip+re-append per open — end-append is the
house posture); persistence (whole-list set; no reload watcher to fight);
Done-uses-captured-template (delete can't misdispatch); POC parity EXACT
(codepoint-identical hint/caret); provenance clean.

Ledger append: F-claude-227-new-modals-exposed-two-more-unrostered-mouse-
openers-001 (sharpens the PR-001 sweep to hand-GUARDS + unrostered openers).

## Phase 4 — Validate
- **Units (workflows.rs):** `param_prompt_new_gates_on_params` (full-value
  initial state), `param_prompt_ops_edit_current_and_advance` (current-only
  edits, multibyte, Done-no-increment + idempotence, 1-param Done-first),
  `param_prompt_filled_zip_total_and_desync_err` (Ok with values, Ok with
  all-empties, the hand-desync MissingParam arm). 6/6 in the module.
- **Headless drive:** `workflow_param_prompt_and_delete_headless` — REQ-001
  both fork arms (no-param inserts "ls -la"; param invoke opens the modal and
  inserts NOTHING), REQ-002 the guided flow lands "deploy web01 prod"
  cumulatively at the prompt, REQ-004 delete-the-middle re-mints
  [(3000, plain), (3001, third)] and index 1 dispatches the SURVIVOR,
  REQ-005 both overlays block input + roster-clear, plus the empty-list
  flash guard. PASS first run. (Three fields went `pub(crate)` for the
  drive — the #398 notify_ticks precedent, comments in place.)
- **Runs:** suite **2155 passed, 5 skipped, 0 failed**; doctests ok; gate
  **GATE GREEN [diff], 15 passed 0 failed** FIRST TRY (coverage 100%, MSI
  100% over the new machine), receipt written.
- **Live drive:** standing environment block (0×0 windows all session); the
  React half was driven + captured at implement (the approved reference);
  the Marley pixel pair rides #417 as recorded throughout the batch.
- Pre-existing: `block v0.1.6` (transitive) — not in scope.

## Phase 5 — Complete
- CHANGELOG entry; app_shell.md workflows paragraph extended (#227). Ledger:
  the F- appended at inspect; AD not needed (the R1/one-builder decision
  lives in the #204 R1 note + this pipeline's record; the roster-sweep
  sharpening lives in the F-).
- POC halves committed (ParamPromptCard + fixture — the React-approved
  reference); parity doc untouched (the card family rows already cover the
  shape; the fixture is POC-only modeling).
- Ticket → closed/; backlog clean; archived.
