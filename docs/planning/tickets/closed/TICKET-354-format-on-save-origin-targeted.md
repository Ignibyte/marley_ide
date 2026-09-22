# TICKET-354 — format-on-save: origin-targeted save-on-switch (#314 follow-up)

- **Ticket:** LOCAL #354 (feature, M20)
- **Tags:** M20, editor, lsp, formatting, follow-up, 314
- **Created:** 2026-07-18
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id b6bb8991-7659-4aa6-b1e4-7e0d8b78e216)
- **Pipeline doc:** ../../pipeline/completed/354-format-save-origin.spec.md
- **Status:** closed (2026-08-11 — shipped: ContentId-bound origin save; every completion lane saves the origin; background consent table; owning-root didSave; inspect hardened the supersede/reload/twin seams)

## Description

#314 format-on-save defers the save while the format round-trips. Today the completion (apply success / settle on Err/stale / the deadline) guards on `active_is_origin(key)` — it does a plain save ONLY when the active editor is still the ORIGIN. If the user switches tab/project during the round-trip (50ms-2s), the deferred save is ABANDONED: the origin editor stays visibly DIRTY (re-savable — no data loss, no wrong-editor save, no misleading flash), but the ⌘S did not complete. This is the bounded fix for the C1-HIGH lost-save (a critic caught the original bug: the save fired against the ACTIVE editor, not the origin — saving the wrong file). The PROPER fix: at completion, save the ORIGIN editor by locating it (path_from_file_uri(key.uri) → locate_open_file), not the active one — mirroring apply_one_file's by-path apply. BLOCKED on save_active being active-only: it reads active_project().active_tab().editor() and drives the #275/#284 external-conflict arm/disarm machinery through active_*_mut() surface methods; a by-(pi,ti) save needs those parameterized (non-active variants of arm_save/disarm_save/mark_saved/set_conflict, etc.), a moderate refactor. Prevention rule: PR-claude-async-completion-binds-to-origin-not-reread-active-001. Referenced as #355 in settle_pending_save / check_format_save_deadline comments + roadmap + editor.md. M20, low-pri (format-on-save is opt-in, default OFF; the current behavior is safe, just incomplete on a mid-format switch).
