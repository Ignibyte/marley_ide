# TICKET-226 — M12.2 — #203 follow-ups: OS notification + notify-state cleanup on manual close

- **Ticket:** LOCAL #226 (feature, M12.2)
- **Tags:** M12.2, cockpit, notifications, follow-up
- **Created:** 2026-07-10
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id f8200e06-1d37-4d5a-906a-d00968142e3f)
- **Status:** closed

## Description

Follow-ups deferred from #203 (background-command completion badge — shipped: the in-app per-tab ● badge via pure `notify::should_notify` + a pump tick-counter). Three items:

(a) OPTIONAL OS notification adapter — the ticket #203 said "optionally raise an OS notification". The in-app badge is the shipped core; this adds a macOS system notification (NSUserNotification/UNUserNotification behind a small adapter, platform call kept out of pure code) raised when a background command finishes past the threshold WHILE Marley is unfocused/backgrounded. Deferred from #203 because a platform notification adapter is unsafe/platform code that can't be headlessly drive-validated + adds a dependency. Reuse the existing pure `should_notify` decision; the adapter is a new masked seam.

(b) F4 (inspect LOW) — scrub `notify_ticks: HashMap<PaneId,u32>` at the MANUAL close sites (close_tab_at / close_project_at / close-pane / ⌘W), not just the dead-pane reap where it's currently removed. Today a manually-closed pane leaks a u32 (bounded, harmless — PaneIds are globally unique #167 so no aliasing — but untidy). Mirror the existing `agents`/`remotes` scrub at those sites.

(c) F5 (inspect LOW) — `tab_flashes: HashMap<(usize,usize),Notify>` is keyed by POSITIONAL (project, tab) indices; a tab/project close shifts later indices, so a surviving unviewed badge can briefly show on the WRONG tab until clear-on-view heals it (transient, narrow: close a lower-indexed sibling while an unviewed badge exists). Fix: key by a stable id, or scrub the affected keys on close. Matches the `renaming_tab` positional-key precedent (same latent issue).

Deps: notify.rs (#203 pure seam, DONE), app.rs pump + close paths, marley_command (for the OS-notify adapter if it shells out) or a small objc2/AppKit adapter crate.
