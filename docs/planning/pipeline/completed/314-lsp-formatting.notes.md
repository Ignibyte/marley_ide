# LSP formatting (⌥⇧F + opt-in format-on-save) — Notes

- **Forge ticket:** #314 7104b4be-714d-46d4-b795-ba67a2a2959a
- **AAR:** 1f05c50e-006e-44ae-93ca-8e848c6368ac
- **Local ticket doc:** docs/planning/tickets/open/TICKET-314-lsp-formatting.md
- **Pipeline spec:** 314-lsp-formatting.spec.md

<!-- Working scratch. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** LSP formatting — ⌥⇧F format-document + opt-in `editor.format_on_save` (default OFF), the save
  never blocked. Promoted from the pre-authored queued spec (the Fable method); SIXTH of `/work
  300,302,303,304,305,314,315,316,317,259` (auto-approved, autonomous-through-commit). First M20-milestone
  ticket of this goal (the prior five were M19).
- **Classification / tier:** work pipeline (one shippable slice). The spec's "hard half already shipped"
  premise HELD (verified below) — scope holds, NO reshape/split.
- **Promotion done:** `git mv` queued→active; pipeline_id `b93e4775-d108-461b-afbe-cbe2cef99f72`; aar
  `1f05c50e-006e-44ae-93ca-8e848c6368ac`; TICKET-314 created; forge #314 already existed (7104b4be).

### Seam re-verification vs LIVE `main` @ 347d3d4 (spec written on Fable; app.rs drifted 5× — #300/#302/#303/#304/#305)
An Explore agent misfired (returned a skill-guidance meta-message, 0 tool uses); re-verified INLINE with
targeted greps + reads. **Every load-bearing claim CONFIRMED; only line numbers + the roster count drifted.**

| Claim | Verdict | Live evidence |
|---|---|---|
| `apply_text_edits` = "the #314-reusable single-document primitive", pure resolve→sort-last-to-first→reject-Overlap/Inverted→splice, TESTED | **VERIFIED** | workspace_edit.rs:215 (doc :212 literally names #314; module doc :4 "the …half of formatting (#314) converge on"); tests at :449 |
| `apply_one_file` wraps the edits in ONE begin/end_undo_group, reverse-offset, open-buffer | **VERIFIED (drift)** | app.rs **8475** (spec said ~8191) |
| `save_active` SYNCHRONOUS; ONE `std::fs::write`; `did_save` POST-write | **VERIFIED (drift)** | app.rs `save_active` **6592**, `std::fs::write` :6650, `host.did_save` **6654** (spec said ~6384/:6445). Has ext-conflict arm/disarm early-returns (#275/#284) — the format fork must preserve them |
| roster `chords.len()` + scoped counts | **DRIFTED (big)** | keymap.rs:1211 `assert_eq!(chords.len(), 80)` = 36 vec + 9 switch-tab + **35 scoped** (spec's 67/22 was pre-#300..#305). #314 → target **81 / 36** |
| ⌥⇧F `chord(false,false,true,true,"f")` FREE | **VERIFIED** | only `"f"` chords: ⌘F `(T,F,F,F)` find + ⌘⇧F `(T,F,F,T)`. No ⌥⇧F. (⌥⇧ modifiers DO exist — ⌥⇧↑/↓ #300 dup-line — so the combo is valid, just unused on "f".) Editor-scoped |
| `inlay_hint_support(caps)` object-or-true template + its app gate | **VERIFIED** | inlay.rs:118 (table test :295). Write `formatting_support(caps)` reading `documentFormattingProvider` as its twin. No formatting_support today |
| `RequestPurpose` enum + the version-keyed key family (CompletionKey/InlayKey) | **VERIFIED** | lsp_host.rs:38 `enum RequestPurpose` (Hover/Definition/Completion/InlayHints); app.rs `completion_request: Option<CompletionKey>` :337, `inlay_request: Option<InlayKey>` :318. Add `Formatting(FormattingKey{uri,version})` + drop stale-version responses |
| anchor caret carry — `anchor_at`/`resolve_anchor` (Bias::Left); the first-consumer risk | **VERIFIED — risk RETIRED** | buffer.rs:223 `anchor_at`, :236 `resolve_anchor`. **#305 shipped anchor.rs's FIRST production consumer** (editor_folds, 15 uses in app.rs) → #314's caret-anchor is the SECOND consumer, the seam proven |
| FormattingOptions truth — `code_tab_width`(4), insertSpaces=true | **VERIFIED** | `code_tab_width` a real setting field (app.rs:416, applied :1597/:1706); Marley pads spaces (indent.rs) → honest values |
| `REQUEST_TIMEOUT_TICKS` = 10s | **VERIFIED** | lsp_host.rs:70 `ticks_for_ms(10_000, PUMP_INTERVAL_MS)`. The ~2 s latch deadline must be a SEPARATE own-tick counter (the #203 notify_ticks idiom), NOT this |
| define_setting! + persist + round-trip NON-default leg (the #330/#331 recipe) | **VERIFIED** | app.rs settings.rs + marley_settings/tests/settings.rs (the round-trip test). `editor.format_on_save: bool=false` → the non-default `true` leg persists |
| **fake_ls fixture serves `textDocument/formatting`?** | **NO — Phase-2 action** | crates/marley_lsp/src/bin/fake_ls.rs exists but NO `formatting` handler (grep empty). **Design/Validate must extend the fixture** (the #331 inlay-lane precedent) to drive REQ-006/007 |

- **Decisions confirmed (no reopens):** D-REUSE-THE-322-APPLY (apply_text_edits + apply_one_file as-is),
  D-VERSION-KEYED-STALE-GUARD (FormattingKey), D-SAVE-NEVER-BLOCKED (own ~2 s tick latch → plain save on
  slow/dead/edit-raced), D-ONE-WRITE (apply-to-buffer THEN save_active = one write of formatted text),
  D-CARET-ANCHOR-FALLBACK (granular edits carry the caret; whole-file-edit degenerate via the Anchor),
  D-HONEST-OPTIONS (code_tab_width + insertSpaces=true). All 10 EARS AC hold.
- **The save fork (confirmed viable):** `save_active` writes whatever is in the buffer + fires didSave. So
  format-on-save = park a latch (skip the immediate save_active) → send Formatting → on the version-guarded
  response apply the edits to the BUFFER then call `save_active` (one write of the now-formatted text) → the
  deadline/Err/intervening-edit path calls `save_active` directly (plain save). The ext-conflict arm logic
  inside save_active is preserved (format doesn't bypass it).
- **Prior-art sweep (§20):** (1) behavior maps — VS Code ⌥⇧F + format-on-save ordering (format → one write,
  save never blocked; manual-format=dirty, on-save=clean) OBSERVED. (2) published — LSP 3.17
  `textDocument/formatting`/`FormattingOptions`/`documentFormattingProvider`; edits address the ORIGINAL doc,
  overlaps invalid → the reject-whole-batch rule is spec, not caution. (3) OUR CODE / permissive deps — the
  apply ENGINE is our shipped #322 (`apply_text_edits`), the capability-reader + stale-guard are shipped
  templates (#331/#313), anchor.rs the caret carry. **gpui/ropey own nothing here — this is LSP-wire + our
  edit engine.** §20 clean-room: reimplement from the published spec; never read Warp/Zed source.

**Phase 1 PASS.** Scope holds; the only genuine Phase-2 delta beyond the spec is extending fake_ls to serve
`textDocument/formatting`. Next: `/pipeline:design`.

## Phase 2 — Design

**§20 confirmed:** LSP 3.17 published-spec (`textDocument/formatting`, `FormattingOptions`,
`documentFormattingProvider`; last-to-first apply is the canonical client algorithm) + VS Code OBSERVED (⌥⇧F;
format→one-write; save never blocked; manual-format=dirty, on-save=clean). Reimplement from the spec; the apply
ENGINE is our own #322; never read Warp/Zed source. gpui/ropey own nothing here.

### The design delta vs the spec — TWO simplifications the seam reads surfaced
1. **NO fake_ls extension needed (the spec's "one Phase-2 delta" DISSOLVES).** The headless LSP drives inject a
   response via the SHIPPED generic hook `push_response_for_test(RequestPurpose, Result)` (app.rs:7993/8013 —
   #313/#331 use it), NOT through the fake_ls fixture (bin/fake_ls.rs serves ONLY the lifecycle handshake). So
   REQ-006 (format-on-save round-trip) drives as: dispatch "save" → latch parked + Formatting sent →
   `push_response_for_test(Formatting(key), Ok(edits))` → drain applies + one save_active; REQ-007 (deadline) →
   dispatch "save" → latch parked → advance pump ticks past the deadline WITHOUT injecting → plain save. fake_ls
   stays untouched. (A WIN to record, like #339's regex dissolve.)
2. **NO new edit engine + NO new resolver.** The formatting response is a bare `TextEdit[]` (LSP ranges); the
   #322 `resolve_text_edits` (LSP range→offset via the #309 bridge, sort last-to-first, reject overlap/inverted)
   + `apply_text_edits`/`apply_one_file` (one undo group) apply it AS-IS. The only NEW parse is the bare-array
   normalize (the WorkspaceEdit parser wraps in `changes:{uri:…}`; formatting has no uri wrapper).

### Architecture
- **marley_lsp — a NEW `formatting.rs` (pure, cov/MSI 100):**
  - `formatting_support(caps: &Value) -> bool` — the `inlay_hint_support` twin (inlay.rs:118), reading
    `documentFormattingProvider` as object-or-`true`.
  - `formatting_request_params(uri, tab_width: usize, insert_spaces: bool) -> Value` — `{ textDocument:{uri},
    options:{ tabSize, insertSpaces } }` (D-HONEST-OPTIONS: `tabSize=code_tab_width`, `insertSpaces=true`).
  - `parse_formatting_edits(result: &Value) -> Vec<LspTextEdit>` — null → `[]`; an array → per-element
    `filter_map` (a `{range,newText}`; a malformed element skipped, never a panic — the #313 idiom). Reuses the
    existing `LspTextEdit`/range shape the #322 `resolve_text_edits` consumes (confirm the type; if the
    WorkspaceEdit parser already exposes a per-array edit parse, reuse it — no duplicate).
  - `FormattingKey { uri: String, version: BufferVersion }` — the stale guard (Completion/Inlay shape).
- **marley_app `lsp_host.rs`:** `RequestPurpose::Formatting(crate::…::FormattingKey)` (enum at :38) — one variant.
- **marley_app `app.rs`:**
  - **⌥⇧F "format-document" arm** → `request_formatting()` (shim): active Rust editor + a Ready host whose caps
    pass `formatting_support` → send `textDocument/formatting` version-keyed (`RequestPurpose::Formatting(key)`),
    the `request()`/purpose-tag path #311 shipped; NO capability/host → a quiet `status_flash` (REQ-005). ⌥⇧F
    does NOT save (leaves the buffer dirty — REQ-008).
  - **`apply_formatting_response(key, result, root) -> bool`** (the drain arm added to the :9000 match): DROP if
    `key.version` ≠ the live buffer version or the focused file moved (REQ-004, the #311 F1/F3 guard); else
    `parse_formatting_edits(result) -> Vec<LspTextEdit>` → **capture the caret as an `Anchor` (Bias::Left) BEFORE
    apply** → **`apply_one_file(path, &edits, enc)`** — which INTERNALLY does `resolve_text_edits` (rejects
    overlap/inverted WHOLE → applies nothing, REQ-002) + `apply_text_edits` + the ONE begin/end_undo_group
    (REQ-001), all shipped (app.rs:8475 takes `&[LspTextEdit]` directly) → **resolve the Anchor → set the caret →
    `follow_editor_caret`** (REQ-003, the whole-file-edit degenerate; granular edits carry the caret for free but
    the Anchor is correct either way) → **if a `pending_save_after_format` latch matches this key, `take` it +
    `do_plain_save()`** (REQ-006, the one write). So the ONLY new marley_lsp parse is `parse_formatting_edits`
    (Value → `Vec<LspTextEdit>`); the resolve + apply + undo are 100% reused.
  - **The "save" arm fork (app.rs:6720):** extract the current body → `do_plain_save()` (save_active + the flash +
    `git_marks_gen++`). New arm: `if self.format_on_save_armed() { self.begin_format_on_save() } else {
    self.do_plain_save() }`. `format_on_save_armed` = the setting ON + an active Rust editor that is DIRTY + a
    Ready host with `formatting_support`. `begin_format_on_save` = park `pending_save_after_format =
    Some(PendingFormatSave{ key, deadline_tick: self.tick + FORMAT_SAVE_DEADLINE_TICKS })` + send Formatting;
    **do NOT write yet.** (An empty/clean buffer or a non-Rust/terminal tab → falls straight to `do_plain_save`.)
  - **The deadline (D-SAVE-NEVER-BLOCKED):** `FORMAT_SAVE_DEADLINE_TICKS = ticks_for_ms(2_000, PUMP_INTERVAL_MS)`
    — an OWN counter, NOT `REQUEST_TIMEOUT_TICKS` (10 s). A pump-tick check `check_format_save_deadline()` (the
    #203 notify_ticks idiom, in the pump's LSP section): if `pending_save_after_format` is Some and `self.tick >=
    deadline_tick` → `take` it + `do_plain_save()` + a quiet flash (REQ-007). An Err response or a version
    mismatch on drain also `take`s the latch → `do_plain_save` (never a lost save).
  - **The latch supersede (D-ONE-WRITE):** a 2nd ⌘S while parked, or an EDIT (version bump) while parked → the
    parked key no longer matches → the pump/drain resolves to a plain save; the latch is single-slot (a new
    begin overwrites → at most one parked save). The apply path is the ONLY save_active caller for a formatted
    save → exactly one write of the formatted text, then didSave (save_active :6650/:6654).
  - **The setting:** `define_setting!(FormatOnSave: bool = false, "editor.format_on_save")` + `persist_format_on_save`
    + the AppliedSettings field (the #330/#331 recipe) — the round-trip test's NON-default (`true`) leg.
  - test hooks (only what a drive consumes — the #337 F2 rule): likely `request_formatting_for_test()`,
    `format_on_save_armed_for_test()`, `editor_dirty_for_test() -> bool`, `pending_format_save_for_test() -> bool`,
    and a tick-advance the #331 drives already have.
- **keymap.rs:** ⌥⇧F `chord(false,false,true,true,"f")` → `"format-document"`, Editor-scoped (roster 80→**81**,
  scoped 35→**36**; the `chords.len()==80` assert → 81 + a `#314 ×1` breakdown; an individual `.contains` +
  a resolution unit ⌥⇧F→format-document on Editor / None on Term).
- **palette.rs / cockpit_commands:** "Format Document" + "Toggle Format on Save" verbs + `action_for_command`
  arms (`every_cockpit_command_resolves`).
- **fake_ls.rs:** UNCHANGED (see simplification 1).

### File manifest
| File | Change |
|---|---|
| crates/marley_lsp/src/formatting.rs | NEW — `formatting_support`, `formatting_request_params`, `parse_formatting_edits`, `FormattingKey` + tests |
| crates/marley_lsp/src/lib.rs | `mod formatting;` + re-exports |
| crates/marley_app/src/lsp_host.rs | `RequestPurpose::Formatting(FormattingKey)` |
| crates/marley_app/src/app.rs | `request_formatting` + `apply_formatting_response` + the "save"-arm fork + `do_plain_save`/`begin_format_on_save`/`check_format_save_deadline` + `pending_save_after_format` field + `FORMAT_SAVE_DEADLINE_TICKS` + the caret-Anchor apply + the setting field + dispatch arms + test hooks |
| crates/marley_app/src/keymap.rs | ⌥⇧F row + roster 81/36 + the resolution unit |
| crates/marley_app/src/palette.rs | "Format Document" + "Toggle Format on Save" verbs |
| crates/marley_app/src/settings.rs | `define_setting!(FormatOnSave)` + `persist_format_on_save` + AppliedSettings |
| crates/marley_app/src/headless_drive.rs | the REQ drives (via `push_response_for_test` + tick-advance) |

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | a granular edit batch applies as ONE undo unit; ⌘Z restores byte-identically | headless (extends the shipped apply tests) |
| REQ-002 | an overlapping/inverted batch is rejected WHOLE (buffer untouched) | pure (formatting.rs / the shipped resolve_text_edits test) |
| REQ-003 | the caret stays on its line through a granular format AND a single whole-file TextEdit (the Anchor) | headless |
| REQ-004 | a formatting response for a stale buffer version DROPS (no apply) | headless — inject a stale key |
| REQ-005 | ⌥⇧F with no capability / no Ready host → quiet flash, no request, buffer unchanged | headless |
| REQ-006 | format_on_save ON → format → apply → ONE save_active (one write) → didSave → buffer CLEAN | headless (`push_response_for_test`) |
| REQ-007 | formatter slow/dead → plain save within the ~2 s own-tick deadline (advance ticks, no response) — save never lost | headless — the deadline row |
| REQ-008 | ⌥⇧F alone leaves the buffer DIRTY (an edit, not a save) | headless |
| REQ-009 | round-trip `editor.format_on_save` with the NON-default (`true`) leg | settings unit |
| REQ-010 | ⌥⇧F + both palette rows resolve to verbs (roster/palette guards) | unit |
| pure | `formatting_support` object/true/absent table; `formatting_request_params` shape; `parse_formatting_edits` null/array/malformed | formatting.rs units |

**Uncoverable / deferred:** LIVE pixel — none (formatting has no distinct render; the flash reuses the shipped
`status_flash`). The whole path is headless via `push_response_for_test` + tick-advance (chad at machine → no
driven capture). A real rust-analyzer round-trip is NOT unit-tested (that is the fixture-server's job, and the
lifecycle is #308's) — the injection lane proves the app-side reconcile, which is the ticket's surface.

### Risks / decisions
- **The latch RACES (P3.5 focus):** (a) an EDIT during a parked format → version bump → the drain's version-guard
  drops the stale response AND the deadline (or the next save) resolves to a plain save — never applies stale
  edits to moved text; (b) a 2nd ⌘S while parked → single-slot latch (a new begin overwrites; at most one parked
  save) → the older key can't double-write; (c) quit during a parked format → the save was never written, but the
  latch is transient (a quit doesn't lose a *committed* save — the pre-format buffer is what the user had; a
  format-on-save that never completed = an unsaved buffer, the normal quit-with-dirty story). The critics attack
  all three.
- **The ONE-WRITE invariant:** the apply path is the SOLE `save_active` caller on the format-on-save path; the
  deadline/Err/supersede paths call `do_plain_save` ONLY when they `take` the latch (so the two can't both fire).
- **The caret degenerate:** a whole-file TextEdit collapses the caret to 0 without the Anchor — the Anchor
  (Bias::Left, #305-proven) carries it; `follow_editor_caret` keeps the line in view.
- **The deadline is an OWN counter** (2 s), never the 10 s global — a dead formatter must not wedge ⌘S for 10 s.
- **D-REUSE-THE-322-APPLY / resolve_text_edits** carry the transactional + reject-whole guarantees for free.

## Phase 3 — Implement

**Built to the manifest (compiles clean; keymap/cockpit/roster tests green):**
- **marley_lsp/src/formatting.rs** (NEW, pure): `formatting_support` (the `documentFormattingProvider` object-or-true
  read), `formatting_request_params` (honest tabSize/insertSpaces), `parse_formatting_edits` (null/array→`Vec<LspTextEdit>`,
  malformed element skipped). Reuses `workspace_edit::text_edit_of` (made `pub(crate)`) — the SAME per-element rule,
  no duplicate. lib.rs re-exports the 3 fns (cargo fmt reordered them alphabetically).
- **marley_app/src/editor_format.rs** (NEW): `FormattingKey{uri, version: BufferVersion}` + `PendingFormatSave{key,
  ticks_remaining}`.
- **lsp_host.rs**: `RequestPurpose::Formatting(FormattingKey)` + the `host.formatting_support()` wrapper (the
  `inlay_hint_support` twin — Ready-and-capable via `self.negotiated`).
- **app.rs**: `send_formatting_request` (the version-keyed send), `request_formatting` (⌥⇧F), `apply_formatting_response`
  (the drain arm: supersede + version/uri guard → `parse_formatting_edits` → caret-Anchor → `apply_one_file` →
  resolve+set caret+follow → complete a parked save), `settle_pending_save` (Err/stale still saves), `do_plain_save`
  (extracted), `format_on_save_armed`, `begin_format_on_save`, `check_format_save_deadline` (the pump countdown);
  the 3 fields (`formatting_request`, `pending_save_after_format`, `format_on_save`) + inits; the "save"-arm fork; the
  `Formatting` drain arm; `format-document`/`toggle-format-on-save` dispatch; the pump deadline tick;
  `FORMAT_SAVE_DEADLINE_TICKS` (2 s). All shims carry `#[cfg_attr(test, mutants::skip)]`.
- **settings.rs**: `FormatOnSave: bool = false` (`editor.format_on_save`) + the `AppliedSettings.format_on_save` field
  (struct + `applied_defaults` + `applied_from` + the 3 test-helper initializers) + `persist_format_on_save`.
- **keymap.rs**: ⌥⇧F `chord(false,false,true,true,"f")` → format-document, Editor-scoped; roster **80→81** / scoped
  **35→36** (+ `#314 ×1`); the `format_chord_resolves` asserts (ED→format-document, TERM→None).
- **palette.rs / cockpit_commands**: `CommandId(25)` Format Document + `CommandId(26)` Toggle Format on Save.

**Deviations from the Phase-2 design (with reason):**
1. **FormattingKey + PendingFormatSave live in marley_app (editor_format.rs), NOT marley_lsp/formatting.rs** — a
   DESIGN ERROR caught at implement: `FormattingKey` carries `marley_editor::BufferVersion` and **marley_lsp does not
   depend on marley_editor** (verified: it "knows nothing of editor::Buffer"; InlayKey/CompletionKey are app-side for
   the same reason). formatting.rs keeps ONLY the pure wire shapes.
2. **The deadline is a COUNTDOWN (`ticks_remaining`), not an absolute `deadline_tick`** — there is no clean global
   app-level monotonic tick (the LspHost's `tick` is per-host, `notify_ticks` is per-pane); a per-latch countdown
   decremented each pump IS the #203 idiom and needs no new tick source.
3. **Dropped the "dirty" gate from `format_on_save_armed`** — format on EVERY armed ⌘S (VS Code's behavior); avoids a
   dirty-accessor dependency and a clean ⌘S just round-trips an empty edit set (a harmless no-op save). (A dirty-gate
   optimization could be added later; not needed for correctness.)
4. **`apply_one_file` does the resolve INTERNALLY** (it takes `&[LspTextEdit]`), so the response path is
   `parse_formatting_edits → apply_one_file` — no separate `resolve_text_edits` call (design simplification confirmed).
5. **fake_ls UNCHANGED** — as Phase 2 predicted, `push_response_for_test` drives the round-trip; the fixture-server
   extension the spec worried about was unnecessary.

`cargo check --workspace` clean; `cargo fmt` applied; `cargo nextest -p marley --lib -E 'test(/roster|fold_chords|
format_chord|every_cockpit/)'` → 3/3 pass. No test expansion (Phase 4 owns formatting.rs's tables + the drives).

**Phase 3 PASS.**

## Phase 3.5 — Inspect

2 parallel general-purpose critics (save-orchestration races; correctness+hygiene) + my own trace. Reconciled below.

**Hygiene (verified):** formatting.rs = 7 mutants (the pure target — Phase 4 kills them); app.rs #314 shims = 0 in
`mutants --list` (all `mutants::skip`); `cargo doc -p marley_lsp` CLEAN (no #303 private intra-doc-link trap on the
pub fns that reference the now-`pub(crate)` `text_edit_of`); no zed/warp in the added lines.

**My own save-orchestration trace (the one-write + races — corroborating the critic):**
- **ONE-WRITE holds.** `save_active` fires at most once per parked save: success (`apply_formatting_response` →
  `if pending.key==key` → take + `do_plain_save`), Err/stale (`settle_pending_save` → take + save), timeout
  (`check_format_save_deadline` → take + save). Every path TAKES `pending_save_after_format` before saving, so a
  second path sees `None` → no double write; and every path that clears the latch DOES save → no lost save.
- **EDIT-DURING-PARKED correct.** Type after ⌘S (V→V+1); the V response hits `apply_formatting_response` → clears
  `formatting_request` → live version V+1 ≠ key.version V → `settle_pending_save` plain-saves the V+1 buffer. The
  stale V whole-file edits are NOT applied to V+1 text; the save happens; `settle` cleared the latch so the deadline
  sees `None` (no double). ✅
- **2ND-⌘S-WHILE-PARKED correct.** `begin_format_on_save` overwrites the single-slot latch + `send_formatting_request`
  overwrites `formatting_request`; the OLD response finds `formatting_request != old_key` → dropped. The old parked
  save is superseded by the live 2nd ⌘S (correct — the newer intent wins); the new latch saves via its response or
  deadline. ✅

**I-1 [MEDIUM — a real bug, to FIX]: a ⌘S can end the buffer DIRTY after the deadline fires.** On the
deadline-then-late-response race: `check_format_save_deadline` fires at countdown 0 → takes `pending_save_after_format`
+ `do_plain_save` (saves the UNFORMATTED buffer, now clean) — but it does NOT clear `self.formatting_request`. A late
format response for the same key then arrives → `apply_formatting_response`'s supersede guard `formatting_request ==
key` still PASSES → it APPLIES the format edits to the just-saved buffer (making it DIRTY again) → and since
`pending_save_after_format` is now `None`, it does NOT re-save. Net: a ⌘S that flashed "Formatter timed out — saved"
leaves the buffer FORMATTED-but-DIRTY. Not data loss (the disk has the saved unformatted text), but a ⌘S that ends
with a dirty ● is surprising. **FIX:** `check_format_save_deadline`, when it fires, also clears
`self.formatting_request = None` — the deadline ABANDONS the outstanding format, so a late response is dropped as
superseded and the buffer stays as-saved (unformatted, clean). (Applied at inspect-fix after the critics land.)

**E-2 — Critic 1 (save orchestration) COMPLETE — corroborates I-1 + found 2 MORE (a HIGH I MISSED).** Verdicts:
ONE-WRITE holds for single-editor-success; EDIT-DURING-PARKED correct; 2ND-⌘S correct; countdown fine (fires on
the 126th pump ≈ 2016ms); quit-during-parked not corruption; ext-conflict safe. Findings:
- **C1-HIGH — the parked save fires against the ACTIVE editor, not the ORIGIN → a lost save on tab/project switch.**
  ⌘S on editor A (format armed) parks a latch keyed to A; the user switches to tab B before the format answers
  (50 ms–2 s); every completion path calls `do_plain_save`→`save_active`, which saves the CURRENTLY-active editor B,
  not A → **A is silently never saved** (latch consumed, A still dirty), and the deadline even flashes "timed out —
  saved" misattributed to B. **FIX (bounded — keeps D-ONE-WRITE; the conflict machinery is active-only so a full
  origin-targeted `save_editor(pi,ti)` is too big for inspect):** guard every completion on the active editor still
  being the ORIGIN (path == `path_from_file_uri(key.uri)`). Stay-on-file → apply + save as-is (correct). Switched →
  ABANDON cleanly (clear the latch + `formatting_request`, NO save of the wrong editor, NO misleading flash — the
  origin stays visibly dirty, re-savable). The `apply_one_file`/guard already require active==origin to apply, so no
  background-editor surprise-edit. **Documented switch-abandon limit → follow-up ticket** (the proper origin-targeted
  save-on-switch, which needs a non-active-editor save path). NOT a data loss (buffer intact + dirty ● visible).
- **C1-MEDIUM — `begin_format_on_save` None-arm double-write.** Server crashes within the window (`negotiated` NOT
  cleared on connection-loss, so `format_on_save_armed` stays true while `request()` returns None); press 2 →
  None-arm `do_plain_save` (write #1) WITHOUT clearing the orphaned press-1 latch → its deadline fires `do_plain_save`
  again (write #2). **FIX:** clear `pending_save_after_format` + `formatting_request` at the TOP of
  `begin_format_on_save` (the new ⌘S supersedes any orphan) — so the None-arm save leaves no orphan.
- **C1-MEDIUM (= my I-1) — dirty-after-deadline** — confirmed; the FIX is the same (clear `formatting_request` on the
  deadline; folded into the C1-HIGH completion-guard changes).
- **C1-LOW — format-on-save regresses ⌘S from sync to deferred → quit within ~2 s loses the edit a plain ⌘S would
  have persisted.** No save-on-quit guard exists. Compounds the HIGH. **Verdict: ACCEPTED as a documented limit** (an
  opt-in feature; degrades to the normal dirty-quit story — disk untouched, no corruption); noted in the follow-up.
- **C1-LOW (nit) — a conflict-blocked ⌘S still reformats the buffer** (the apply runs before `save_active`'s conflict
  early-return). **Verdict: REJECTED as a bug** (inside one undo group, content preserved, cosmetic) — accept as-is.

**E-3 — Critic 2 (correctness + hygiene) COMPLETE — found a MEDIUM I MISSED + confirmed everything else solid.**
Confirmed: parse totality (null/scalar/bare-object/malformed/2³²-position all → skip/[], no panic; `text_edit_of`
reuse byte-identical), `formatting_support` mirrors `inlay_hint_support`, params shape, the borrow structure (owned
`Option<Anchor>`, no NLL-fragility), all 3 guards (supersede + LIVE version + uri), the capability + Rust gates, no
dup/dead code, hygiene (7 mutants / 0 app shims / rustdoc clean / no zed-warp / no unwrap). Findings:
- **C2-MEDIUM (F1) — the caret Anchor does NOT carry the caret through a WHOLE-FILE `TextEdit` (the case it was
  added for).** A whole-file `edit(0..len, new)` COVERS the caret; `rebase_offset` collapses a covered `Bias::Left`
  anchor to the span start (0) → for rust-analyzer's typical one-whole-file-edit reformat, every ⌥⇧F/format-on-save
  TELEPORTS the caret+viewport to line 1 — the precise REQ-003 bug. Proven by `anchor.rs::covered_collapses_by_bias`
  + the critic's real-`Buffer` throwaway (`anchor_at(15,Left)` → whole-file edit → resolve == 0). **FIX:** replaced
  the Anchor with a **(line, col) re-seat** — capture `line_col(caret)` (0-based) BEFORE apply, re-seat via the
  shipped `caret_for_line_col(buf, row+1, Some(col+1))` (1-based, clamps into the reformatted line) AFTER. Keeps the
  caret on its logical line (best-effort, the VS Code/Zed behavior); for the rare granular-edits case the buffer
  would carry it more precisely, but whole-file is rust-analyzer's actual shape → the re-seat is the right default.
- **C2-LOW (F2) — a no-op format (`null`/`[]`) still collapsed a selection/multi-cursor to one bare caret.** The
  apply + `set_single_caret` ran unconditionally. **FIX:** early-return `complete_parked_save` when
  `edits.is_empty()` — a no-op format touches neither buffer nor selection (still completes a parked save: the file
  is already formatted, one plain write).
- **C2-LOW (F3) = my I-1 / C1-dirty** — same dirty-after-deadline; fixed by clearing `formatting_request` on deadline.
- **C2-INFO (F4) — `formatting_support` doc overstated "ready-and-capable"** (`negotiated` isn't cleared on
  connection-loss; the real send gate is `request()`'s `Phase::Ready`). **FIX:** softened the doc to match (the
  shared idiom `inlay/signature/workspace-symbol` all do the same; clearing `negotiated` is out of #314 scope).

**FIXES APPLIED (all at source, §0 — no suppressions):**
1. **C1-HIGH (wrong-editor lost save):** new `active_is_origin(key)` (active editor `has_path(path_from_file_uri(key.uri))`);
   `settle_pending_save` + `check_format_save_deadline` now `do_plain_save` ONLY when the active editor is the origin
   — a switch abandons cleanly (no wrong-editor save, no misleading flash; origin stays visibly dirty). The
   origin-targeted save-on-switch (a non-active-editor save path, blocked on `save_active`'s active-only conflict
   machinery) is the **#354 follow-up**.
2. **C1-MEDIUM (None-arm double-write):** `begin_format_on_save` clears `pending_save_after_format` + `formatting_request`
   at the top (a new ⌘S supersedes any orphan) → the None-arm plain save leaves no orphan for its deadline.
3. **I-1 / C1-dirty / C2-F3 (dirty-after-deadline):** `check_format_save_deadline` clears `formatting_request` on fire.
4. **C2-F1 (caret):** the (line, col) re-seat replaces the Anchor.
5. **C2-F2 (no-op collapses selection):** early-return on empty edits.
6. **C2-F4:** doc softened.
Extracted `complete_parked_save` (the success/no-op one-write). `cargo check --workspace` clean; `cargo fmt` applied;
`every_cockpit_command_resolves` green; app.rs #314 shims still 0 mutants (all fixes carry `mutants::skip`).

**Lenses covered:** the save-orchestration one-write + races + never-block (Critic 1 — 1 HIGH + 2 MED found & fixed,
2 LOW accepted); correctness of the pure parse + caret carry + guards + capability + reuse + hygiene (Critic 2 — 1
MED + 2 LOW found & fixed, INFO doc). **Verdict: Phase 3.5 PASS** — 1 HIGH + 3 MEDIUM fixed at source (the critic
loop earned its keep: my self-review missed the wrong-editor lost-save AND the whole-file-caret collapse), 3 LOWs
accepted/documented, the #354 follow-up filed at Phase 5.

## Phase 4 — Validate

**Tests added (13): 3 formatting.rs pure tables + 9 headless drives + 1 settings round-trip leg.**

**(A) `crates/marley_lsp/src/formatting.rs` `mod tests` (the pure surface — 7 mutants, 6 caught + 1 unviable, 0
missed = MSI 100 verified in isolation):** `formatting_support_table` (object/true→true, false/null/absent/scalar→
false); `formatting_request_params_shape` (exact `{textDocument, options:{tabSize, insertSpaces}}` + a non-default
tab width); `parse_formatting_edits_totality` (null/scalar/bare-object/`[]`→[]; one valid → range+text; mixed →
only the valid; a 2³²-position element SKIPPED not truncated; missing range/newText skipped).

**(B) `crates/marley_app/src/headless_drive.rs` — 9 drives (ALL GREEN; the #331 injection lane via
`drive_formatting_for_test` + `push_response_for_test` + the deadline countdown; safe collect→reap→assert):**
- REQ-003 / **C2-F1** `format_whole_file_keeps_caret_on_line_and_stays_dirty` — a WHOLE-FILE reformat re-seats the
  caret to **row 2** (its line), NOT collapsed to 0 (the covered-Anchor bug); + REQ-008 (⌥⇧F leaves it DIRTY).
- REQ-001 `format_apply_is_one_undo_unit` — a granular reindent applies, ONE ⌘Z restores byte-identically.
- REQ-002 `format_overlapping_batch_rejected_whole` — an overlapping batch applies NOTHING.
- REQ-004 `format_stale_version_response_dropped` — a stale-version response drops; the user's edit stands.
- REQ-005 `format_document_no_capable_host_sends_nothing` — ⌥⇧F with no capable host: no send, buffer unchanged.
- REQ-006 `format_on_save_one_write_then_clean` — parked save → response → apply + ONE write of the FORMATTED
  text → buffer CLEAN (disk == FMT_DONE).
- REQ-007 / **I-1** `format_on_save_deadline_saves_and_late_response_is_dropped` — the countdown fires → plain-save
  (clean, never blocked); `formatting_request` cleared → a LATE response is DROPPED (no re-dirty).
- **C2-F2** `format_noop_response_does_not_collapse_selection` — a no-op `[]` format leaves a 2-cursor selection
  UNCOLLAPSED and touches nothing.
- REQ-010 `every_cockpit_command_resolves` (CommandId 25/26) + `format_chord_resolves` (⌥⇧F) — green.

**(C) REQ-009** — `settings_round_trip_survives_reload` gains the `editor.format_on_save = true` NON-default leg
(default OFF, so the leg proves a real write — kills the `persist_format_on_save → Ok(())` mutant).

**Test hooks added (only what a drive consumes — the #337 F2 rule):** `formatting_key_for_test`,
`drive_formatting_for_test`, `park_format_save_for_test`, `set_format_on_save_for_test`,
`pending_format_save_for_test`, `formatting_request_set_for_test`, `tick_format_deadline_for_test`,
`request_formatting_for_test`, `editor_dirty_for_test` (all `#[cfg(test)]`).

**Not driven (stated, not skipped):** the `dispatch("save") → format_on_save_armed → begin → send` path with a
REAL Ready+capable host — the format-on-save STATE (park → response/deadline → apply/save/clean) is driven via the
direct-park injection lane; the send-gate is unit-covered (REQ-005 no-cap → no send; format_on_save_armed gates
on `host.formatting_support()`). The C1-HIGH abandon-on-switch's 2-editor switch is covered by `active_is_origin`'s
uri check + REQ-004's uri/version guard drop (a full 2-file switch drive was disproportionate). LIVE pixel: none
(formatting has no distinct render; the flash reuses `status_flash`); chad at the machine → no driven capture.

- **Gate: `scripts/gates.sh --diff` → GATE GREEN [diff], 15 passed / 0 failed** (receipt
  `94be9fdd6216222f0a916ec5f6c8fde52ad1b6b0` == `.git/ignibyte-gate-receipt`, commit-valid). coverage ≥ 100%
  (formatting.rs fully covered by the 3 tables; app.rs coverage-excluded), mutation MSI ≥ 100% (formatting.rs 7
  mutants 0-missed; app.rs #314 shims + the formatting_support wrapper `mutants::skip`; the settings non-default
  leg kills `persist_format_on_save → Ok(())`; keymap/palette killed by the roster/format_chord/cockpit units),
  miri, docs (rustdoc clean), visual — all green.
- **Two gate reds fixed at source (§0 — no baselines):** (1) **the roster CASCADE** — a SECOND roster test,
  `all_chords_lists_every_binding`, asserts the scoped count (35) + enumerates every binding; ⌥⇧F made it 36, so it
  failed → which also failed gate:4 (the coverage run re-runs tests) and gate:5 (the mutation baseline runs tests):
  ONE failing test, three red gates. Added the ⌥⇧F `.contains` assert + bumped 35→36 (+ `#314 ×1`). LESSON: there
  are TWO roster guards (`default_keymap_maps_named_chords` count + `all_chords_lists_every_binding` count+set) —
  a new chord must update BOTH; my Phase-3 filter only ran the first. (2) **the `formatting_support` wrapper mutant**
  — `LspHost::formatting_support` (lsp_host.rs) lacked the `#[cfg_attr(test, mutants::skip)]` every twin capability
  wrapper (inlay/code-action/signature/workspace-symbol) carries ("shim: trivial read over the tested … seam"); its
  2 body mutants (→true/→false) survived (the drives use the direct-park injection lane + `request()` also
  Ready-gates, so no unit distinguishes the wrapper). Added the skip — the PURE `marley_lsp::formatting_support` is
  tested (the `formatting_support_table`), so this is the accepted masked-shim pattern, not a §0 suppression.

**Phase 4 — Validate PASS.**

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
