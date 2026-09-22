# TICKET-229 — Agent-launcher popup should dismiss on click-outside

- **Forge ticket:** #229 (7062e0a2-6926-415c-8fd2-4ca03d6bdc5c) (bug, M13)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018 (autonomous /work 228-237 run)
- **AAR:** efe9cef1-9c72-480c-b7fd-5bbb98b33995
- **Pipeline doc:** ../../pipeline/active/agent-launcher-dismiss.spec.md
- **Source ticket:** sprint #26 M13 "The Workspace Cockpit" (fd182395) — chad live feedback #5
- **Status:** closed

## Summary
Launching an agent (⌘⇧A) opens the #181 agent-launcher picker, but clicking outside it doesn't close it —
the launcher renders as a single `.occlude()` box with no full-screen backdrop, so it only dismisses via
Escape/Enter. Fix: add a full-screen occluding backdrop with a click-away dismiss, mirroring the proven
#166 context-menu backdrop. A shim render fix.

## Acceptance
Opening the launcher and clicking outside its box dismisses it; an inside click does not (the box occludes,
keyboard nav unaffected); Escape/Enter behavior is unchanged. Full EARS (REQ-001..003) in the pipeline spec.
