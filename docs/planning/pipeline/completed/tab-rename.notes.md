# M11 #177 — tab rename + persisted titles — Notes

- **Forge ticket:** #177 `46ffa2bc-4643-4b56-abac-b9087cbd904b` · **AAR:** `1f1cfcd2-ca26-42a4-ae8f-2ff5a6f99e61`

## Phase 1 — Plan / Phase 2 — Design (folded)
- custom_title rides Tab (Option — None keeps every #157 behavior); the T= payload splits once on \x1f
  (absent → old-blob byte-compat); sanitize at write only (restore trusts the writer, the #163 stance);
  the rename editor mirrors the Search-tabs key routing; commit persists via the #163 piggyback.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- tabs.rs: Tab.custom_title (pub Option; the 3 constructors init None); Project::tab_mut.
- titlebar.rs: display_title (custom-if-nonblank > rail_tab_title's command > fallback).
- grid_layout.rs: TabLayout::Terminal → { blob, title }; serialize writes `title\x1f` before the blob only
  when the sanitized title is non-empty (untitled = the pre-#177 BYTES); restore split_once('\x1f');
  sanitize_title (strip \t\n\r\x1f, trim, cap 60) pub — the D2 write-side stance.
- app.rs: renaming_tab state; the tab-row double-click seeds the draft from the effective title; the rename
  key branch FIRST among modals (enter commits — empty CLEARS custom — + persist; esc/backspace/printable;
  others swallowed); the row's label cell renders draft+▏ (accent border) while renaming; live_tab_title →
  display_title; the boot restore + build_shell_layout carry custom_title.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (lifecycle/codec/modal/breadth lenses; full suite + clippy).

- **[CRITICAL → FIXED] a stale pinned-bytes assert** — I added a title to the #163 round-trip fixture but
  left its golden wire pin at the old bytes → the suite FAILED. Updated the pin. (The golden-wire assert
  earning its keep: it caught the wire change immediately.)
- **[MAJOR → FIXED] index-shift wrong-tab commit** — renaming_tab holds (project, tab) indices; a mouse
  close (tab ×, project ×, the menu ClosePane) shifts them → the editor would commit to the wrong tab or
  dangle invisibly. close_tab_at + close_project_at now dismiss the rename (covers every close caller).
- **[MAJOR → FIXED] space + uppercase were untypeable** — gpui names the spacebar "space" (5 chars → the
  1-char arm missed it) and resolves shift only into key_char. Added a "space" arm + push key_char (the
  bare key as the lowercase fallback). PROVEN live: "Build Box" typed with both.
- **[MINOR → FIXED] modal inversion** — a right-click (menu) or the palette could co-exist with the rename;
  the right-click now dismisses the rename, and the double-click clears completion + context_menu.
- **[MINOR → FIXED] empty seed** — a command-less tab opened a blank editor; now seeds with the row's
  static label (captured before the move closure). PROVEN: the editor opened seeded "terminal 1".
- **[MINOR → FIXED] cockpit/code renames would silently revert** (their wire carries no title) — the
  double-click now REFUSES non-terminal rows (grid().is_some()).
- Critic-verified clean: codec back-compat (old T=t byte-identical; '=' titles safe — only the 2-char T=
  prefix is stripped; hand-edited double-\x1f degrades gracefully); no stale .title display surface (the
  rail is the only tab-title render and it goes through live_tab_title→display_title). Added the critic's
  mutation-gap tests (blank-title→untitled-bytes; '='-title round-trip; empty-blob restore).

Lenses: index-shift lifecycle, key naming, modal exclusivity, codec back-compat, display breadth.

## Phase 4 — Validate
- **Tests:** sanitize_title_cases (all 4 framing chars + \x1f in one input, trim, whitespace→empty, the
  60/61 cap); display_title_precedence (custom > blank-falls-through > command > fallback > empty-custom);
  shell_codec_titles (titled golden wire, untitled OLD bytes, write-sanitize, blank→untitled-bytes,
  '='-title round-trip, empty-blob, double-\x1f split). 3 new; full suite 267/267.
- **Self-test (REQ-003, typed — #172/#177 harness):** added a `dblclickat` verb (a real down/up ×2 with
  mouseEventClickState — two plain clicks never coalesce to a double). tr_t1.png: double-click opened the
  accent-bordered editor seeded "terminal 1", "Build Box" typed in (space + uppercase). tr_fresh_crop.png:
  cleared + typed "Release Run" + Enter → the rail row renamed + the blob shows `T=Release Run\x1fH:t,t,t,t,t`.
  tr_survived_crop.png: after a full kill+relaunch the row STILL reads "Release Run" — the #163
  "titles regenerate" limit CLOSED. (Harness note: the modal text field needs CONTINUOUS key focus — a
  rename must run in one drive.swift call from a focused state; cross-invocation focus is flaky, documented.)
- **Gate:** first RED gate:4+5 (tab_mut untested — the shim-only pure fn; added tab_mut_cases) → GREEN 15/15, MSI 100.

## Phase 5 — Complete
- CHANGELOG + app_shell #177 note + AX-memory harness update; forge #177 → done. **M11 6/8.** LESSONS: index-keyed modal state dismisses on any structural mutation; text inputs need key_char + a "space" arm; back-compat codec growth = optional field behind an impossible byte; a shim-only pure fn still needs its own unit test.
