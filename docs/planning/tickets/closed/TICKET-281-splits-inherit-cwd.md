# TICKET-281 — New splits/tabs inherit the focused pane's live cwd

- **Forge:** #281 `ff81ab5c-41a3-4298-8dd4-b2e7fc800f1a` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/281-splits-inherit-cwd.spec.md` (ae4f0c91-8c57-46d8-8380-48dbacf1afc0)

## Summary
Splits (⌘⇧L/⌘⇧J + menu) and new terminals (⌘T/⌘D/"+") spawn in the
focused pane's live prompt pwd (the #201 shell-integration source)
when it's a real absolute directory, else the project root (#160's
behavior, now the fallback). One pure validation core (`valid_dir_or`)
shared with the #205 restore path — strengthened with an absolute
check. Open-project/launcher/restore untouched; no settings knob (D4).

## Acceptance
Spec REQ-001..003: the validation core + delegation, the inherit
flows headless, and the fallback identity.
