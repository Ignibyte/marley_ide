# Format-on-save — origin-targeted save-on-switch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-354-format-on-save-origin-targeted.md
- **Pipeline spec:** 354-format-save-origin.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** TICKET-354 — format-on-save: origin-targeted save-on-switch
  (#314 follow-up). Top of the BACKLOG queue; picked as /work next.
- **Classification / tier:** work pipeline, feature, M20, one shippable slice
  (marley_app only; possibly a marley_lsp helper reuse — no new crates).
- **Recall (§18.3):**
  - `PR-claude-async-completion-binds-to-origin-not-reread-active-001` (HIGH) —
    the rule minted FROM this bug: completions act on the origin captured at
    request time; locate it at completion; trace the cross-target race
    explicitly (a single-target trace passes while the bug ships).
  - `BF-claude-deferred-save-fires-against-active-not-origin-editor-001` (HIGH)
    — the original #314 failure record: every completion path saved the
    ACTIVE editor; the bounded fix (`active_is_origin` guard) abandons on
    switch; the origin-targeted save was filed as this follow-up, blocked on
    `save_active`'s active-only #275/#284 conflict machinery.
  - `BF-claude-extchange-armed-write-leaves-stale-arm-001` — #284: an arm
    consumed by a write must be cleared on EVERY exit path; the parameterized
    machinery must preserve the racing-branch disarm.
  - Skip-detach trap (strike SIX — `DL-cluster-303d3ed43a72`,
    `BF-claude-mutants-skip-detach-trap-recurred-252`, hit ON `save_active`
    itself in #252): inserting a sibling fn above a `mutants::skip` shim
    re-binds the attr. This refactor inserts by-target siblings around
    `save_active`/`do_plain_save` — `cargo mutants --list -f app.rs` after
    every structural edit near a skip, grep the shim names absent.
  - #314 as-built (`completed/314-lsp-formatting.notes.md`): ONE-WRITE
    invariant (every path TAKES the latch), deadline is an own 2 s countdown
    (`ticks_remaining`, the #203 idiom), I-1 fix (deadline clears
    `formatting_request` so a late answer drops), C1-MEDIUM (a new ⌘S clears
    an orphaned latch up front).
- **Discovery:** seam facts below from the §18.2 Explore sweep (see the
  Phase-1 seam map appendix); the current completion sites are
  `apply_formatting_response` (app.rs:9175, active-read of version/caret at
  :9194), `settle_pending_save` (:9280, origin guard :9291),
  `check_format_save_deadline` (:9352, origin guard :9368),
  `complete_parked_save` (:9263), `do_plain_save` (:9302) →
  `save_active` (:9382, the active-only write + #275/#284 machine +
  active-root didSave at :9430).
- **Decisions:** D-ORIGIN-BIND, D-ONE-WRITE-KEPT, D-ONE-MACHINE,
  D-CONFLICT-SAFE-BACKGROUND, D-OWNING-ROOT-DIDSAVE, D-CLOSED-ORIGIN-DROPS,
  D-NO-UI-DELTA (see spec). Scope boundary: cross-project response DELIVERY
  stays TICKET-413 (the deadline lane covers the save till then).

### Phase-1 seam map (Explore, §18.2)
- **HEADLINE: the ticket text is stale — #397 dissolved the anticipated
  refactor.** `locate_open_file` is DELETED (#397, editors onto registry);
  `(pi,ti)` no longer identifies a file (a tab's `EditorSurface` is a list of
  views; file state lives in the registry as `Content::Editor`). The
  active_* guard methods are one-line delegates to `EditorInstance` methods
  that are ALREADY target-agnostic.
- **Locator precedent:** `RootView::buffer_for_open_path_mut` (app.rs:12038)
  scans ALL roots via `self.content.iter()` + `editor_surface::same_file`
  (ers.rs:31 — canonicalizing compare, the #322 F-CORR-1 belt), lexicographic
  first-root tie-break (D-OPEN-DEDUPE-SCOPE). Generalize to
  `instance_for_open_path_mut(&Path) -> Option<&mut EditorInstance>`;
  `buffer_for_open_path_mut` becomes `.map(...buffer_mut)` over it.
- **`EditorInstance`** (editor_surface.rs:56): fields root/path/buffer/
  saved_version/nonce/disk/conflict/armed_at/conflict_observed; methods
  `path()` :130, `buffer()` :135, `is_dirty()` :152, `mark_saved()` :162,
  `disk()` :167, `set_disk` :173, `conflict()` :178, `set_conflict` :184
  (None also clears armed_at + conflict_observed), `arm_save` :206,
  `disarm_save` :212, `set_conflict_observed` :222, `reload` :232,
  `root()` :125 — **the owning-root answer for didSave routing** (no
  path→root helper exists anywhere; none needed).
- **`save_active`** (app.rs:9382): head calls `check_active_file_external`
  (app.rs:8970, active-only; the pure decision table
  `extchange::external_action` is target-agnostic — the ONE genuinely new
  by-target fn is an instance-scoped external check). didSave today:
  `active_project().root` → host (app.rs:9429-9432); `LspHost::did_save`
  (lsp_host.rs:289) no-ops when the doc isn't open on that host — mis-route
  today is silent, not wrong.
- **Completion sites:** `active_is_origin` :9253 (predicate → becomes a
  locator), `complete_parked_save` :9263, `settle_pending_save` :9280,
  `check_format_save_deadline` :9352, `do_plain_save` :9302 (funnel),
  `apply_formatting_response` :9175 (reads active version/caret :9194-9199;
  caret re-seat :9228-9241 is VIEW-scoped — stays active-guarded; the edit
  apply is already by-path via `apply_one_file` :11967).
- **Latch structs** (editor_format.rs): `FormattingKey{uri, version}` :11;
  `PendingFormatSave{key, ticks_remaining}` :23. No origin identity beyond
  uri — Explore suggests an optional `origin: ContentId` bound at park
  (O(1) `get_mut(id)`, robust to path respelling, natural None when closed);
  design picks vs path-decode+scan.
- **Pump/consume:** every host `drain()`s (app.rs:1926-1931) but consume is
  ACTIVE-ROOT-ONLY (`consume_lsp_responses(&active_root)` app.rs:1996/12509)
  — TICKET-413 confirmed. `check_format_save_deadline` runs unconditionally
  (app.rs:1988), so cross-PROJECT switches complete via the deadline lane
  until 413; tab-switch-within-project completes fully via the response.
  #354 is the Formatting arm's non-active-root reasoning 413 will need.
- **Flash** (flash.rs:13, `status_flash` app.rs:630): GLOBAL per-app. Design
  decision: name the file for a background-origin save vs suppress; the write
  is correctness, the flash is ergonomics.
- **Test lane:** all #314 tests in `headless_drive.rs` `#[cfg(test)]`
  (fixtures :9092-9103, seven format tests :9108-9351; harness
  `open_editor_file` :7551). Hooks (app.rs): `formatting_key_for_test`
  :11358 (ACTIVE-bound — needs a by-path twin), `drive_formatting_for_test`
  :11375, `park_format_save_for_test` :11393, `set_format_on_save_for_test`
  :11401, `pending_format_save_for_test` :11406,
  `tick_format_deadline_for_test` :11417, `editor_dirty_for_test` :11427
  (active-bound — needs by-path twin). **COVERAGE GAP: no test exercises the
  origin-guard/switch behavior at all today** (both guard sites are
  `mutants::skip` shims). New drives: two editors AND two projects; park on
  A, activate B, land response/Err/deadline; assert A saved + B untouched.
- **Baseline:** `cargo nextest run -p marley -E 'test(/format|save/)'` →
  12/12 PASS pre-change (crate name is `marley`, not marley_app).

## Phase 2 — Design

### Architecture / approach
The latch binds the ORIGIN as a `ContentId` at park time; every completion
resolves it O(1) through the registry and acts on that instance. "Active" stops
being an identity and becomes a CONSENT MODE derived at completion
(`active_id == origin` ⇒ interactive semantics = today's `do_plain_save`
verbatim; else ⇒ background semantics). One save machine, one write tail.

- **D8-ORIGIN-ID** — `PendingFormatSave` gains `origin: ContentId` (Copy,
  app-lifetime id, content_registry.rs:23), captured in `begin_format_on_save`
  from the active surface's `active_id()` (editor_surface.rs:395). Close kills
  the id → `registry.get_mut` returns None → REQ-006 drop falls out naturally.
  A REOPENED file is a new instance/id — the parked ⌘S belonged to the dead
  buffer generation, so dropping is correct (the strict D-CLOSED-ORIGIN-DROPS
  reading). Rejected alternative: path-decode + scan at completion — it would
  resurrect reopened files and can diverge from the applied-edit target on
  nested-root twins (D-OPEN-DEDUPE-SCOPE: same path, two instances).
- **D9-ONE-FUNNEL** — new `save_editor_by_id(id, cause)` owns the whole
  completion policy; `cause ∈ {FormatSettle, FormatDeadline}` drives flash
  wording. Branches:
  - `id` dead / not an editor → silent drop (REQ-006).
  - `id == active_id` → EXACTLY today's path: deadline pre-flash
    ("Formatter timed out — saved") then `do_plain_save()` — preserving
    today's flash-override order where the #275 arm-flow flash beats the
    deadline flash (REQ-007 hard guarantee).
  - background → the background arm (D10): external table → write/hold →
    `write_and_mark(id)`; flash only on Deadline-write
    ("Formatter timed out — saved <file name>") or write-error
    ("Save failed <file name>: e"); silent otherwise.
- **D10-BACKGROUND-CONSENT** — the background arm applies
  `extchange::external_action(snapshot, disk, dirty)` (already pure +
  target-agnostic) to the ORIGIN instance:
  - `Noop` → write.
  - `CleanReload` (clean buffer, newer disk) → SKIP the write entirely — a
    clean buffer has nothing to save, and #275's D4 stance ("background tabs
    check on their own activation") already owns the reload. No view-coupled
    side effects (inlay/flash/git-marks reload plumbing stays active-only).
  - `Conflict` (dirty + changed disk) → set `Changed` conflict +
    `conflict_observed` on the instance, NO write, NO arm, NO flash — the
    banner greets the user's return (REQ-004, D-CONFLICT-SAFE-BACKGROUND).
  - `Deleted` → set `Deleted` conflict, then WRITE (recreate — #275 needs no
    arming for Deleted); the write tail clears the conflict as today.
  - **D-ARMED-BACKGROUND-HOLDS**: a background origin CAN carry `armed_at`
    (armed while active, then switched before the second-press completion
    landed). Strict REQ-004: still NO write; the license is left UNTOUCHED
    (not consumed, not disarmed) — #284 staleness re-checks it on the user's
    interactive return. Overwriting-under-conflict stays an active-consent
    act, full stop.
- **D11-ONE-WRITE-TAIL** — extract `save_active`'s tail (fs::write →
  owning-root didSave → `mark_saved`/`set_disk` → the #284 racing-length
  cross-check with its disarm) into `write_and_mark(id) -> io::Result<()>`,
  instance-addressed. `save_active` keeps its interactive head (active
  external check + #275/#284 arm flow) and calls `write_and_mark(active id)`.
  didSave routes by `EditorInstance::root()` (editor_surface.rs:125) —
  D-OWNING-ROOT-DIDSAVE. For the active editor owning root == active root in
  every normal open; in the add-to-pane cross-project edge today's
  active-root call was a silent no-op (host doc-guard), owning-root delivers
  it correctly — an improvement inside REQ-005's class, noted.
- **D12-RESPONSE-RESOLUTION** — `apply_formatting_response` resolves ONE
  target and uses it for guard + apply + save:
  1. superseded guard (`formatting_request != key`) unchanged; Err/Abandoned →
     `settle_pending_save` unchanged.
  2. target id = the parked latch's `origin` when the latch key matches
     (authoritative — the ⌘S's instance); else (latchless ⌥⇧F)
     `path_from_file_uri(key.uri)` → `editor_id_for_open_path(path)` (the
     scan). Unresolvable → settle (which safely drops/saves via the latch).
  3. STALE guard reads the TARGET instance's `buffer().version()` — not the
     active editor's.
  4. empty-edits → `complete_parked_save` (unchanged semantics, origin save).
  5. apply through `apply_edits_to_open_editor(target_id, edits, enc)` — the
     open-instance half of `apply_one_file` extracted and id-addressed, so
     the applied buffer and the saved buffer are THE SAME INSTANCE by
     construction (kills the nested-twin divergence where a path re-scan
     could pick the lexicographic sibling).
  6. caret re-seat is VIEW-scoped: only when target == active id (background
     views reconcile on activation — the established #322/#397 posture for
     background buffer edits).
  7. `complete_parked_save(&key)` → take latch → `save_editor_by_id`.
  Consequence (in-scope generalization): a latchless ⌥⇧F whose answer lands
  after a switch now formats the ORIGIN buffer (leaves it dirty, no save) —
  per-document semantics matching the Zed/LSP reference, same D-ORIGIN-BIND
  rationale.
- **D13-RETIRE-ACTIVE-IS-ORIGIN** — `active_is_origin` is deleted with its
  callers. Its `has_path` guard was SURFACE-granular (all rows of the pane),
  so a same-project FILE-ROW switch passed the guard and `save_active` then
  wrote the newly-active file: an unrequested plain save of B (its own
  content — never cross-file corruption) while A stayed abandoned-dirty.
  The id-granular design closes that residue class; capture as a failures.md
  append at complete.
- **D14-LOCATOR** — new `editor_id_for_open_path(&Path) -> Option<ContentId>`
  = the scan from `buffer_for_open_path_mut` (same_file + lexicographic-first
  root, app.rs:12038); `buffer_for_open_path_mut` and `apply_one_file`'s
  open-file arm rewire through it. One scan, one tie-break rule.

**§20 confirm** — Reference (Zed editor behavior + published LSP spec) holds:
format-on-save completes on the DOCUMENT that was saved, focus-independent;
LSP formatting/didSave are URI-addressed. This design binds completion to the
document instance and routes didSave by its owning root — behavior-level
match; mechanism (registry ids, latch, consent modes) is Marley's own. N/A
stays N/A for React parity (no UI delta; flash wording only). No copyleft
source read.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/editor_format.rs` | `PendingFormatSave` + `origin: ContentId` field + docs (pure struct; unit tests in-module if present) |
| `crates/marley_app/src/app.rs` | `editor_id_for_open_path` (new, extracted scan); `buffer_for_open_path_mut` delegates; `apply_edits_to_open_editor(id,…)` extracted from `apply_one_file`'s open arm; `apply_formatting_response` per D12; `active_is_origin` deleted; `complete_parked_save`/`settle_pending_save`/`check_format_save_deadline` → take + `save_editor_by_id`; `save_editor_by_id(id, cause)` (new, D9/D10); `write_and_mark(id)` (new, D11) + `save_active` tail rewired onto it; `begin_format_on_save` captures `origin`; test hooks: `park_format_save_for_test` grows origin resolution, new `editor_dirty_for_path_for_test(path)` |
| `crates/marley_app/src/headless_drive.rs` | the 8 new drives below |
| *(no change)* | `editor_surface.rs`, `content.rs`, `content_registry.rs`, `lsp_host.rs` — every needed accessor exists (`active_id`, `as_editor{,_mut}`, `get{,_mut}`, `iter`, `root()`, `sent_bodies_for_test`) |

### Regression test plan
All in `headless_drive.rs` (the #314 lane: fixtures :9092, `open_editor_file`
:7551, `seed_two_projects` :49 + `shell.switch_project`, `EditorMut::activate`
for file-row switch; hooks `park_format_save_for_test` /
`drive_formatting_for_test` / `tick_format_deadline_for_test` /
`sent_bodies_for_test`). House order: observations → `reap_sessions` → assert.

| Test (new unless noted) | Proves |
|---|---|
| `format_save_completes_on_origin_after_file_switch_headless` — A dirty, park(A), activate B's row, push formatted response, drain | REQ-001 (A's disk formatted + clean, latch gone) + REQ-003 (B dirty, B's disk untouched) |
| `format_save_err_settles_plain_save_on_switched_origin_headless` — park(A), switch to B, push Err | REQ-002/Err (A plain-saved with raw buffer text, clean) + REQ-003 |
| `format_save_stale_version_plain_saves_origin_after_switch_headless` — park(A) at V, type in A (V+1), switch, push V-response | REQ-002/stale (A saved with TYPED text, formatted edits NOT applied) |
| `format_save_deadline_plain_saves_switched_origin_headless` — park(A), switch to a TERMINAL tab (active_editor None), tick out the countdown; then push the late response | REQ-002/deadline (A saved; flash carries A's file name; `formatting_request` cleared; late answer dropped, A stays clean) |
| `format_save_never_writes_the_nonorigin_active_editor_headless` — B ACTIVE + DIRTY, settle fires | REQ-003 pointed witness (B not written, still dirty; A saved) — the C1/same-surface class regression |
| `format_save_background_conflict_holds_write_and_arm_headless` — park(A), switch, externally rewrite A's file, complete | REQ-004 (disk keeps the external content; A dirty; conflict `Changed` + observed set; `armed_at` None) |
| `format_save_didsave_routes_to_owning_root_host_headless` — two projects; A in P1 (Ready host), park, `switch_project` to P2, deadline | REQ-005 (P1's `sent_bodies` gains a `textDocument/didSave` for A; P2's has none) — deadline lane; response lane is #413 |
| `format_save_closed_origin_drops_latch_without_write_headless` — park(A), close A's row, deadline | REQ-006 (no write — disk content/mtime unchanged; latch None; no flash) |
| *(existing)* the 12 baseline format/save tests | REQ-007 — green untouched, incl. `ext_save_arm_and_deleted_headless` guarding the `save_active` tail extraction |

- **Mutation posture:** new app.rs fns are drive-verified `mutants::skip`
  shims (house policy for the registry/fs/gpui-adjacent layer — same as
  `save_active`/`do_plain_save`/`buffer_for_open_path_mut` today); NO new
  pure logic lands in app.rs beyond delegation. `editor_format.rs` stays
  fully mutation-covered (a pure struct change). **Skip-detach discipline
  (strike six):** `cargo mutants --list -f crates/marley_app/src/app.rs`
  before AND after; grep each shim name absent from the list.
- **trybuild:** none — no new public type contract (ContentId stays the
  registry's private-field newtype; no cross-crate seam change).
- **Visual/AX:** N/A (no UI delta).
- **Uncoverable:** nothing new — all paths headless-drivable.

### Risks / decisions
- R1 `save_active` tail extraction must preserve #275/#284 byte-for-byte —
  guarded by the ext_* baseline drives; any deviation is a red.
- R2 skip-detach (strike six) — the `--list` check is mandatory, twice.
- R3 apply/save divergence on nested-root twins — closed by D12.5 (one
  resolved id for apply AND save).
- R4 owning-root didSave upgrades a silent no-op to a real notification in
  the cross-project-pane edge — intended (REQ-005 class).
- R5 D-ARMED-BACKGROUND-HOLDS: strictest reading of REQ-004; the armed
  license survives untouched for the interactive return.
- R6 background CleanReload skips both write AND reload (D4 owns activation
  reload) — a clean buffer never needs a completion write.

## Phase 3 — Implement
- **React-first: N/A** (spec: no UI delta — flash wording only, inside the
  shipped flash affordance).
- **Built (per manifest):**
  - `editor_format.rs`: `PendingFormatSave.origin: ContentId` (D-ORIGIN-BIND
    doc), `SaveCause {Settle, Deadline}`, pure `file_label(path)` (flash
    label; 2 mutants → Phase-4 kill list).
  - `editor_surface.rs`: +`EditorRef::active_id()` (the identity read the
    funnel compares).
  - `app.rs`: `editor_id_for_open_path` (the one by-path scan, lex-first
    tie-break) + `apply_edits_to_open_editor(id,…)` (the open-arm of
    `apply_one_file`, id-addressed per D12.5); `apply_one_file` rewired
    through both; `write_and_mark(id)` (D11 tail — owning-root didSave via
    `instance.root()`); `save_editor_by_id(id, cause)` (D9/D10 consent-mode
    funnel; active arm = pre-flash + `do_plain_save` in today's exact order);
    `save_active` head unchanged, tail → `write_and_mark(active id)`;
    `settle_pending_save` → take + funnel; `check_format_save_deadline` →
    take + funnel(Deadline); `apply_formatting_response` per D12 (one target
    for guard/apply/save; view-scoped caret re-seat + follow);
    `begin_format_on_save` captures origin pre-send; `active_is_origin`
    DELETED; hooks: `park_format_save_for_test` resolves origin,
    +`editor_dirty_for_path_for_test`.
- **Deviations from design (all recorded):**
  1. `complete_parked_save` FOLDED into `settle_pending_save` — post-D12 both
     reduced to the identical "take latch → origin save (Settle)" operation;
     two names for one body is drift bait. (Design listed both.)
  2. `buffer_for_open_path_mut` DELETED, not delegated — its single caller
     was `apply_one_file`'s open arm, which now goes id-first; a dead
     delegate is banned dead code.
  3. `editor_surface.rs` touched beyond "no change": +`active_id` accessor
     (needed by the funnel), −`has_path` and −`active_mark_saved` (orphaned
     by retiring `active_is_origin` and by the tail extraction; `-D warnings`
     forbids carrying them). `has_path`'s row-membership assertions dropped
     with it; the join test now marks saved via `instance_mut().mark_saved()`
     (same semantic, same assertions).
- **Checks:** `cargo check --workspace` clean; clippy clean except the
  EXPECTED transient `editor_dirty_for_path_for_test` unused warning (its
  callers are the Phase-4 drives); rustdoc `-D warnings` exit 0; `cargo
  mutants --list -f app.rs` 90 mutants before AND after with 0 on the
  shim/new-fn names (strike-six: no detach); `editor_format.rs` gained
  exactly the 2 `file_label` mutants (kill in Phase 4);
  `EditorRef::active_id` mutant is the `Default::default()` replacement
  (unviable if `ContentId: !Default`, else killed by a join-parity assert —
  Phase 4 confirms).

## Phase 3.5 — Inspect
Three independent critics (correctness/races; state-integrity/conflict-machine;
simplification+provenance) + the lead's own trace. All verdicts evidence-based.

| # | Sev | Finding (source) | Verdict | Fix |
|---|---|---|---|---|
| F1 | MED | Cross-file ⌘S supersede: `begin_format_on_save`'s unconditional latch clear silently discards a DIFFERENT file's parked, consented save (lead L-1 + critics 1,2 independently — triple-found; asymmetry proof: a non-Rust B ⌘S left the latch alone and A completed) | CONFIRMED | different-origin latch is SETTLED (`save_editor_by_id(old.origin, Settle)`) before parking; same-origin keeps the #314 supersede |
| F2 | MED | Reload version-epoch collision (PRE-EXISTING): `reload_active_from_disk` resets the version epoch but left `formatting_request` alive — a clean-at-open ⌘S (v0) + agent rewrite + reload lets the late answer pass v0==v0 and splice stale whole-file edits, then the parked save WRITES the corruption (critic 1; the #401 inlay class on the formatting key) | CONFIRMED (pre-existing, same reachability old/new — fixed because it is this ticket's seam and the #401 pattern is one line) | reload also clears `formatting_request` (response dies superseded; the latch deadline plain-saves the RELOADED text — safe) |
| F3 | LOW | Latchless ⌥⇧F lane resolved `key.uri` via the ALL-roots lexicographic scan — could hand a nested-root TWIN of a since-closed origin to stale edits (critic 1) | CONFIRMED | resolution pinned to the RESPONDING root (`i.root() == root && same_file`, unique by D-OPEN-DEDUPE-SCOPE) |
| F4 | MED | `file_label` duplicated the shipped, tested, doc-designated `tabs::file_basename` (critic 3) | CONFIRMED | deleted; funnel uses `crate::tabs::file_basename` (also removes the new mutation surface) |
| F5 | LOW | `save_editor_by_id`'s `mutants::skip` lacked the house shim marker/justification (critic 3) | CONFIRMED | `Shim.` + `// shim:` justification naming the drive coverage |
| F6 | NIT | Per-candidate `to_path_buf` in the scan not borrow-required (critic 3) | CONFIRMED | borrowed `(i.root(), id)` |
| F7 | LOW | `park_format_save_for_test` bound origin by path scan (could bind a nested-root sibling in a dual-root fixture) where production binds the ACTIVE instance (critic 2) | CONFIRMED (test-only) | hook binds `active_id()` + loud uri cross-check |
| F8 | LOW | Background write when snapshot is `None` (dismissed-Deleted → untracked → agent recreates → parked save clobbers with the user not looking) (critic 2) | ACCEPTED residue — exact parity with the ACTIVE ⌘S's documented #275 "dismissed removal stays quiet" semantic; diverging the two machines would be worse | none (recorded) |
| F9 | — | Unused `editor_dirty_for_path_for_test` warning (critic 3) | Acknowledged — deliberate P4 staging; its callers are the Phase-4 drives | P4 lands callers |
| F10 | — | Double path fetch across funnel/tail (critic 3) | REJECTED as a change — verified no divergence window (same synchronous tick); `write_and_mark`'s self-contained dead-id contract is worth the second lookup | none |

**Clean verdicts (evidence in the critic transcripts):** #275/#284 extraction
statement-identical (3 inert drifts named: selection-home sync skip — version
never bumps on selection; text snapshot point; instance-vs-view path — equal by
construction); one-write invariant holds on every path incl. deadline-vs-late
response both orders; ContentId monotonic → dead-id lanes total, no panic;
flash enumeration truthful (no "saved" without a write); REQ-007 verified
line-by-line vs HEAD; ⌥⇧F switched now background-formats (documented D12);
provenance CLEAN (behavior-level only, all identifiers trace in-repo);
security CLEAN (write path unsteerable by a hostile server — key.uri is ours,
edits bounds-checked; didSave doc-guarded; no spawn/secret surface).
**P4 additions from inspect:** a cross-file-⌘S settle drive (F1) and a
background CleanReload-skip drive (the D10 arm had no planned row).
**Post-fix checks:** fmt OK; clippy clean (known F9 transient only); mutants
--list 90/0 shim hits (no detach); editor_format.rs 0 mutants; both #314
drives PASS.

## Phase 4 — Validate
- **Tests added (10 new drives in `headless_drive.rs`, + 2 test hooks
  `editor_conflict_state_for_path_for_test` / `begin_format_on_save_for_test`):**
  `format_save_completes_on_origin_after_file_switch` (REQ-001+003),
  `format_save_abandoned_settles_plain_save_on_switched_origin` (REQ-002/Err),
  `format_save_stale_version_plain_saves_origin_after_switch` (REQ-002/stale),
  `format_save_deadline_plain_saves_switched_origin` (REQ-002/deadline +
  named flash + late-response drop),
  `format_save_never_writes_the_nonorigin_active_editor` (REQ-003 / the
  C1-HIGH+D13 witness), `format_save_background_conflict_holds_write_and_arm`
  (REQ-004), `format_save_background_clean_reload_skips_write` (D10),
  `format_save_closed_origin_drops_latch_without_write` (REQ-006),
  `format_save_cross_file_save_settles_prior_parked_save` (inspect F1),
  `format_save_didsave_routes_to_owning_root_host` (REQ-005 — the fake_ls
  REAL-server lane per PR-...-never-prove-the-live-wire; asserts the didSave
  wire body on the owning root and zero wire on the active project).
  REQ-007 = the untouched 12-test #314/#275 baseline (green).
- **Runs (actual):** `cargo nextest run --workspace` → **2101/2101 passed,
  5 skipped**; doctests pass. Full `scripts/gates.sh --diff` →
  **GATE GREEN [diff], 15/15** (receipt written).
- **One red fixed en route:** gate:4 (coverage 100-floor) flagged 2
  never-run closures in the new didSave drive — a `canonicalize`
  `unwrap_or_else` fallback (fixture always canonicalizes → `expect`) and an
  `.any()` closure over the deterministically-EMPTY P2 body list (→ a
  stronger `bodies_b.is_empty()` assert). Both source-fixed; drive re-passes.
- **Fixture drift found while writing:** a lone-row surface close returns
  `CloseOutcome::WouldDrain` (not `Removed`) — the REQ-006 drive opens B
  first so A's close is a real removal; and wire uris carry the CANONICAL
  tempdir spelling (/private/var vs /var) — asserts compare like for like.
- **Live-app drive:** N/A — no visual/affordance surface changed (spec
  parity N/A). The mid-format switch race (50 ms–2 s against a live
  rust-analyzer) is not deterministically reproducible via the selftest
  harness; the headless drives own this seam end-to-end, and the fake_ls
  lane covers the real wire. Not silently skipped — recorded here.
- **Pre-existing failures:** none (workspace fully green before and after).
- **trybuild:** none new — no new public type contract (per design).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md Unreleased/Changed entry (TICKET-354);
  `docs/marley_architecture/editor.md` format-on-save bullet rewritten to the
  as-built id-binding (+ the retired guard's granularity note);
  `docs/marley_architecture/app_shell.md` gains the M20 #354
  instance-addressed-save bullet; `docs/marley_architecture/roadmap.md` #354
  follow-up marked SHIPPED. Parity sync: N/A (no UI delta).
- **Ledger appends (§19):** at inspect —
  `F-claude-354-supersede-slot-drops-cross-file-parked-save-001`,
  `F-claude-354-reload-epoch-collision-formats-stale-save-001`,
  `F-claude-314-origin-guard-was-surface-granular-001`,
  `PR-claude-single-slot-latch-supersede-must-settle-not-discard-001`,
  `PR-claude-version-epoch-guards-invalidate-on-reload-001`; at complete —
  `L-claude-354-recheck-a-queued-tickets-blocker-against-head-001`,
  `L-claude-354-empty-iterator-closures-fail-the-coverage-floor-001`,
  `AD-claude-354-saves-are-instance-addressed-active-is-a-consent-mode-001`.
- **Ticket:** TICKET-354 closed → `tickets/closed/`; no stale BACKLOG row
  (removed at promotion).
- **Archive:** pair moved to `docs/planning/pipeline/completed/`.
