# TICKET-759 — Follow Omarchy's theme

- **Ticket:** LOCAL #759 (feature, the finish line: item 9, follow Omarchy's theme)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [759-follow-omarchys-theme.spec.md](../../pipeline/completed/759-follow-omarchys-theme.spec.md)
- **Source ticket:** Chad, 2026-10-11: "if this is easy to do and will be across mac, windows, and
  omarchy then lets do it. if not i dont want to maintain this. if its just here is an omarchy theme
  then maybe". Told it is Omarchy-only by nature (on macOS and Windows Zed already follows the
  system's light and dark), about a day, one template file and a setting off by default: "lets go
  with 9 as well". The T3 Code survey's item 7 (`docs/t3code_architecture/README.md`) and report 02
  (§2.20, item 1, open question 1). Plan: [the finish line](../../design-notes/finish-line-2026-10-11.md),
  Now, with the build-now items, before #445.
- **Status:** closed

## Summary
An Omarchy theme switch retints the terminals, Chromium, Claude Code, VS Code, Obsidian and T3 Code;
Marley keeps its old colours. With `marley.omarchy_theme` on (off by default), Marley puts a Zed
theme template, `marley.json.tpl`, in Omarchy's user templates folder and a small hook in
`theme-set.d`. At each theme switch Omarchy renders the template from the new palette and the hook
copies the result into Marley's themes folder as `omarchy.json`, which Zed's theme watcher reloads.
Turning it on also renders the current theme once (`omarchy-theme-refresh`) and selects the
"Omarchy" theme; turning it off removes Marley's files and puts back the theme chosen before.
Without Omarchy the setting does nothing.

## Acceptance
With the setting on, Marley draws in the current Omarchy theme's colours and follows a theme switch
without a restart; turned off, it is back in the theme chosen before, with no Marley file left in
Omarchy's folders; with no Omarchy, turning it on changes nothing.
