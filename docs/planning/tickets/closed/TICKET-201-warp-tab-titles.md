# TICKET-201 — cwd-aware tab titles (the CWD-basename fallback tier)

- **Forge ticket:** #201 (419815b7-21ae-4c5d-998f-ab8db82d4f57) (feature, M12.2)
- **Owner:** autonomous (/work 195-222)
- **AAR:** cc1903eb-0092-4302-848b-38da214b7309
- **Pipeline doc:** ../../pipeline/active/warp-tab-titles.spec.md
- **Source ticket:** sprint #25 (f3ecc094) — M12.2 cockpit polish
- **Status:** closed

## Summary
Complete the ticket's `custom → command → cwd basename → generic` tab-title chain. The rename editor,
the persisted `custom_title`, and the running-command basename derivation already ship (#177 / #157);
the one missing tier is the **CWD basename** — an unnamed no-command tab currently shows the static
"terminal N" instead of its working directory's last path component. Extend the pure `display_title`
with a cwd tier (reusing `prompt::pwd_label`) and wire the pane's shell-reported pwd (`block.prompt.pwd`)
into `live_tab_title`.

## Acceptance
An unnamed tab with no running command shows its cwd basename (e.g. `Marley` for
`~/Projects/ignibyte/Marley`); a running command still wins (`vim`); a custom rename still wins and
persists. Full EARS criteria (REQ-001…005) live in the pipeline spec.
