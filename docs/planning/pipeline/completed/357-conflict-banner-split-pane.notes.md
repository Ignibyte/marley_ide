# Conflict banner on the focused split pane (#357) — Notes

- **Forge ticket:** #357 (02cbc091-eebf-4ca2-a73d-8b58fbaee5a6)
- **AAR:** 43fac3b4-aa2d-4df2-ade9-f753fcebb936
- **Pipeline spec:** 357-conflict-banner-split-pane.spec.md

## Phase 1 — Plan / Phase 2 — Design
- **Seam:** the banner is INLINE in the editor-tab render block (app.rs:15621–15690): `self.active_editor()
  .and_then(|s| s.active_conflict()).map(|conflict| { … card … })`, added via `.children(banner)` (15703).
  Buttons: `ext_keep_mine` (6882) / `ext_reload` (6915) / `ext_dismiss_deleted`. `EditorSurface::active_conflict()`
  (editor_surface.rs:376); `ExtConflict { Changed, Deleted }` (extchange.rs). `active_editor()` is focus-aware
  (#259) → resolves the focused split pane's surface.
- **Design:** extract `fn ext_conflict_banner(&self, colors: &ThemeColors, cx: &mut Context<Self>) ->
  Option<gpui::Div>` (`mutants::skip`) = the inline build verbatim. Editor-tab block calls it. Split-pane render
  adds it to the FOCUSED pane's body (gated `workspace().focused() == pane_id`). `.absolute` top-right is
  parent-relative → each site positions it in its own area with no coord change.
- **Decisions:** D1 extract+reuse (no dup) · D2 focused-pane-only (conflict armed only on the focused surface;
  buttons act on `active_editor()`).
- **Prior-art (§20):** the banner + `ext_*` handlers + focus-aware `active_editor()` already exist — pure render
  RELOCATION, no new logic. N/A reference (Marley-specific #275/#284 surface).
- **Test plan:** REQ-001 headless drive (split → arm a conflict → render + drive `ext_reload`/`ext_keep_mine`);
  REQ-002 existing #275 editor-tab tests green (byte-equivalent extraction). Render shim `mutants::skip`,
  app.rs coverage-excluded (#307 pattern).

## Phase 3 — Implement
- Extracted `ext_conflict_banner(&self, colors, cx) -> Option<gpui::Div>` (`mutants::skip`) from the inline
  editor-tab banner — byte-equivalent (same card, buttons, `ext_*` handlers).
- Editor-tab block: `let banner = self.ext_conflict_banner(&colors, cx);` (the `.children(banner)` stays).
- Split-pane render: added `.relative()` to the pane `body` (positioning context) + on the FOCUSED pane,
  `body = body.children(self.ext_conflict_banner(&colors, cx));` (reusing the hoisted `let focused`).
- **Deviations:** none. **Compile:** `cargo check -p marley` clean.
## Phase 3.5 — Inspect
One general-purpose critic. **Verdict: CLEAN — SHIP, no fixes.** It did a byte-exact diff (extracted fn vs the
removed inline: 63 lines, ZERO difference), ran `ext_conflict_banner_flows_headless` (1 passed), and `cargo check`.

| Lens | Verdict |
|---|---|
| Byte-equivalence | CLEAN — the extracted fn is byte-identical to the removed inline (only the #185 click-through comment dropped; the `.block_mouse_except_scroll()` it described is retained). Same card, arms, handlers. |
| `active_editor()` resolves the focused split pane | CLEAN — focus-aware (`grid.focused_editable_surface`); the banner reader + the `ext_*` button writers both act on the focused pane's surface (agree). |
| Focused-only / no double banner | CLEAN — the editor-tab + split-pane sites are mutually exclusive by `TabContent` variant (`rect_list` empty unless `active_is_terminal`); the `focused` gate matches exactly one pane; conflicts arm only on the focused surface. |
| Positioning | CLEAN — `.relative()` is the right containing block; no layout shift (absolute is out-of-flow); the top-right card is in-bounds (not clipped). |
| Regression | CLEAN — handlers byte-unchanged + still `pub(crate)`; the flows test passes; the `mutants::skip` detach trap respected (the new fn carries its own attr). |

**LOW (informational, not defects):** an unfocused pane's banner is hidden until refocus (INTENDED — mirrors the
editor-tab banner on a tab switch + the #259 focus-owns-the-slots model); a very narrow split pane could
left-clip the wider "File removed…" card via `body`'s `overflow_hidden` (cosmetic edge → possible follow-up).
No `failure-record` — no bug.
## Phase 4 — Validate
- No new pure seam (a render extraction). The critic proved byte-equivalence (63 lines, ZERO diff) + ran
  `ext_conflict_banner_flows_headless` (**1 passed** — the #275 handlers are unchanged) + `cargo check` clean.
- **REQ-001** (split-pane banner): carried by composition — the byte-equivalent fn ∘ focus-aware `active_editor()`
  (resolves the focused split pane) ∘ the tested `ext_*` handlers. Render arm `mutants::skip` / app.rs
  coverage-excluded (#307 pattern).
- **REQ-002** (editor-tab unchanged): the byte-exact diff + the passing flows test.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (3:45) — 15/15 (no new mutants/coverage since the
  diff is all `mutants::skip` render; existing tests + static gates green). Receipt written.
- Pre-existing: none.
## Phase 5 — Complete
- **Docs (§21):** `CHANGELOG.md` → a `### Added` entry (#357); `app_shell.md` → a #357 note on the #259 section.
- **AAR (forge):** `aar-submit` completed, effectiveness **5** (a clean mechanical extraction; inspect clean).
  No `failure-record` — no bug.
- **Lessons:** extract-and-reuse a render builder (DRY) beats duplicating; a byte-exact-diff extraction + the
  existing handler test carry a render change (the #307 excluded-render pattern). The focus-aware `active_editor()`
  (#259) meant the banner content + button targets were ALREADY correct for a split pane — the fix was pure
  RELOCATION, zero new logic.
- **Close:** forge #357 → done; ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.
