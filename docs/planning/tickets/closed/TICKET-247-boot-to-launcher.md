# TICKET-247 — Boot to the launcher when empty + a New-empty-workspace launcher action

- **Forge ticket:** #247 (3b3b1422-9912-4ff7-93f4-02d7f49ae8a2) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** db554f83-c956-4f7a-8b24-cfcf9485fed4
- **Pipeline doc:** ../../pipeline/completed/boot-to-launcher.spec.md
- **Source:** the M14 round; 8th ticket. A #234 follow-on (independent of the #242 editable editor).
- **Status:** closed

## Summary
Today boot always force-seeds a default workspace, so a genuinely fresh/empty start never lands on the #234
launcher. (a) When the persisted shell has no projects to restore (`restored.is_none()`), boot to the launcher
(build an empty `Workspace` — a new zero-project constructor — instead of seeding) and AUDIT every accessor
reachable in the empty-boot state (the #234 relax-invariant class; the known landmine is the boot recents-fold).
(b) Add a "New empty workspace" launcher button that creates a workspace rooted at a real default dir (`$HOME` —
the PTY needs a live cwd, the #205 lesson) via the shipped `open_project_path`, no folder picker.

## Acceptance
A cleared/fresh session boots to the launcher (no seed, no crash); "New empty workspace" creates a home-rooted
workspace without a picker; a SAVED session still restores normally (regression). `Workspace::empty` cov/MSI 100;
driven-validated with a settings.toml backup/restore (chad's session protected). Full EARS in the pipeline spec.
