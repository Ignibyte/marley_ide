# viewer guards + persistence (M4 FINALE) — Notes

- **Forge ticket:** #106 `387b845e-4e88-4bc6-a2b2-e62ac7f7f195` · **AAR:** `39e94a27-8066-45bd-b2cd-4dc0dcdba0f9`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-106-viewer-guards.md

## Phase 1 — Plan
- **Request:** forge #106 (M4 10/10, FINALE) — binary/size guards + a configurable viewer tab width.
- **Pre-flight:** open_file_in_viewer (#98) uses read_to_string (→ read bytes + guard); settings.rs has the
  read-only TermCols pattern (no persist fn) to mirror for code.tab_width; the applied_from test extends.
- **Decisions:** D1 NUL-in-8000 + len≤max; D2 code.tab_width read-only, clamp ≥1; D3 flash placeholder.
- **DEVIATION:** File/Diff view-mode persistence dropped (#103 folded it into ⌘⇧D); code.tab_width persists instead.
- **AAR id:** `39e94a27-8066-45bd-b2cd-4dc0dcdba0f9`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_view.rs (PURE):** `is_probably_binary(bytes: &[u8]) -> bool` = `bytes.iter().take(8000).any(|&b| b==0)`; `viewer_size_ok(len: usize, max: usize) -> bool` = `len <= max`.
- **settings.rs:** `define_setting!(CodeTabWidth: u16 = 4, "code.tab_width")` (read-only, mirrors TermCols — NO persist fn); AppliedSettings gains `code_tab_width: usize`; applied_defaults → `CodeTabWidth::default_value() as usize`; applied_from → `(manager.get::<CodeTabWidth>().max(1)) as usize`. Update EVERY AppliedSettings{} literal (applied_defaults, applied_from, all test literals). Extend applied_from_reads_saved_values TOML with `[code]\ntab_width = 8` + `code_tab_width: 8`; ADD a clamp test (tab_width=0 → 1) to kill the max(1)-removal mutant.
- **app.rs SHIM:** `const VIEWER_MAX_BYTES: usize = 2*1024*1024`; RootView `code_tab_width: usize` field (= applied.code_tab_width in new()); open_file_in_viewer: `match std::fs::read(&path) { Ok(bytes) if viewer_size_ok(bytes.len(), VIEWER_MAX_BYTES) && !is_probably_binary(&bytes) => code_view=Some(CodeViewState::new(path,&String::from_utf8_lossy(&bytes),self.code_tab_width,CODE_MAX_COLS)); Ok(_) => status_flash "cant open (binary or >2MB)"; Err(_) => {} }`.
- **Mutation targets:** is_probably_binary NUL/take; viewer_size_ok <=; the tab_width max(1)+cast.
- **Test plan:** is_probably_binary_cases (text/NUL/empty); viewer_size_ok_cases (at/over/zero); applied_from code_tab_width=8; code_tab_width clamps 0→1. cov/MSI 100. The open guard masked.
- **Risks:** every AppliedSettings literal must add the field (compiler-enforced); clamp ≥1 avoids %0-panic in expand_tabs; NUL-only heuristic (UTF-8-ratio deferred).

## Phase 3 — Implement
- **Built:** code_view.rs is_probably_binary + viewer_size_ok (+ tests); settings.rs CodeTabWidth read-only setting + AppliedSettings.code_tab_width (applied_defaults=4, applied_from clamped ≥1) + all 5 literals updated + the applied_from test (tab_width=8) + a clamp test (0→1); app.rs VIEWER_MAX_BYTES const + code_tab_width field (=applied.code_tab_width) + open_file_in_viewer reads bytes → viewer_size_ok && !is_probably_binary → open with the configured width, else a placeholder flash.
- **DEVIATIONS:** (1) removed the now-dead CODE_TAB_WIDTH const (the setting replaced it). (2) File/Diff view-mode persistence dropped (#103 folded it into ⌘⇧D) → code.tab_width persisted instead (read-only, no dead persist fn). (3) NUL-only binary heuristic (UTF-8-ratio deferred).
- **Verification:** fmt; check 0 err; clippy OK; 6 tests pass (guards + tab_width + applied_from + clamp).

## Phase 3.5 — Inspect
- **Method:** self-review across the 3 files (2 pure guards + a read-only setting + a masked open guard); gate cargo-mutants is the MSI authority.
- **Lenses — no findings:** is_probably_binary (NUL in first 8000 — tested text/NUL/empty); viewer_size_ok (len≤max — tested at/over/zero); code.tab_width (read via applied_from, clamped ≥1 so expand_tabs can't %0-panic — tested 8 + the 0→1 clamp; applied_defaults=4); the open guard reads bytes, gates on BOTH size + binary, from_utf8_lossy (no panic on non-UTF8), placeholder flash on reject, Err ignored; the setting is read-only (no persist fn → no dead code, mirrors TermCols). No unwrap on IO. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** is_probably_binary_cases + viewer_size_ok_cases (code_view); code_tab_width_clamps_to_at_least_one + applied_from_reads_saved_values(tab_width=8) + applied_defaults (settings). `cargo nextest` → 6 pass.
- **Self-test:** opening a binary/huge file → placeholder needs a synthetic open (ENV-BLOCKED); the pure guards + the setting are engine-tested cov/MSI 100; the open guard masked.
- **Gate finding + fix:** first run RED on MSI 60% — the new const `VIEWER_MAX_BYTES = 2 * 1024 * 1024` left 4 unkillable `*`-operator mutants (arithmetic in a diff-ed const, no behavioral test pins the value). Fixed: a bare literal `2_097_152` (matches the other app.rs literal consts). Re-gate → GREEN 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG + app_shell.md M4 section; aar-submit(4); prevention-rule claude-const-arithmetic-in-diff-use-literal-001; forge #106 → done. **M4 10/10 — FINALE, M4 COMPLETE.** is_probably_binary + viewer_size_ok + code.tab_width (cov/MSI 100) + the guarded open. Gate caught the const-arithmetic mutants → literal.
