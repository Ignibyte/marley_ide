# TICKET-383 — selftest: the `focus` verb's fixed click point misfires on restored layouts

- **Forge ticket:** #383 4ddc8936-2e8f-4b0e-a3ce-5f7ec0926770 (chore, M25 — sprint #36 "M25 — App-Grade QA Hardening")
- **Owner:** unassigned — claimed at /work promotion
- **AAR:** PENDING — aar-open runs at /work promotion (Phase-1 drafted docs-only)
- **Pipeline doc:** ../../pipeline/queued/383-selftest-focus-verb.spec.md (moves to ../../pipeline/active/ at promotion)
- **Source ticket:** the 2026-07-21 live-QA-run findings batch (sprint #36)
- **Status:** closed

## Summary
Found in the 2026-07-21 live QA run: `drive.swift focus` clicks a FIXED window fraction —
`click(w.x + w.w / 2, w.y + w.h * 0.12)` = (0.5, 0.12) (scripts/selftest/drive.swift:214) — which
silently assumes the boot-default layout (terminal in the center). On a RESTORED layout (Files
panel open + an editor split) that point lands in the file tree: the run's first `focus` clicked
the `.cargo/audit.toml` row, opened it in the Editor tab, and the follow-on `type:` keystrokes
went somewhere unhelpful — costing several diagnostic rounds before the harness (not the app) was
identified. Fix, harness-only: (1) make `focus` layout-independent — activation without ANY
content interaction (title-bar-region click vs programmatic raise is an honest open fork for
design; a caller-supplied-fraction pane-targeting idiom covers the old second job); (2) promote
the two re-learned README rules to a prominent pre-drive checklist — activate (`osascript …
frontmost`) IN THE SAME shell command as the drive verbs (keys silently vanish otherwise —
re-confirmed live), and never trust `focus` for pane targeting on a restored workspace (use
`clickat:` with coordinates read from a fresh capture). No app code; `scripts/selftest/` only.

## Acceptance
Headline: `focus` against a restored workspace brings Marley frontmost with ZERO content
interaction (a before/after capture shows no tree file opened, no new tab); the boot-default
canonical flow (`focus "type:echo hi" enter`) still executes; the README carries the
same-shell-command activation checklist and the fresh-capture `clickat:` pane-targeting rule, with
the stale "focus … focus the pane" claims corrected; every other verb behavior-identical. Full
EARS criteria live in the pipeline spec.
