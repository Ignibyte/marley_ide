# TICKET-203 — background-pane command-finish notification (tab flash)

- **Forge ticket:** #203 (4f5fac35-0985-4986-888e-203710700ea5) (feature, M12.2)
- **Owner:** autonomous (/work 195-222)
- **AAR:** ee7be81c-460e-474d-a06f-12f1394d2c0a
- **Pipeline doc:** ../../pipeline/active/warp-bg-notify.spec.md
- **Source ticket:** sprint #25 (f3ecc094) — M12.2 cockpit polish
- **Status:** closed

## Summary
Signal when a long-running command finishes in a NON-focused pane — today you have to watch the pane.
Flash the pane's tab (success vs failure) once a command completes in a background pane after running
past a duration threshold. A pure `should_notify(pane_focused, status, elapsed_secs, threshold_secs)
-> Notify` (cov/MSI 100) makes the decision; the masked shim stamps per-pane command start times
(the Block model tracks no elapsed), detects the Running→Finished transition in the #173 pump, and
raises a per-tab flash (reusing `flash.rs` + `exit_status_kind`) cleared on focus. The optional OS
notification is deferred (follow-up).

## Acceptance
Run `sleep 3` in a background pane and focus another → on completion the background tab flashes
(success); a failing command flashes failure; a focused-pane finish and a quick command do not flash.
Full EARS criteria (REQ-001…005) live in the pipeline spec.
