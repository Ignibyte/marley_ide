# TICKET-801 — Component templates from files

- **Ticket:** LOCAL #801 (feature, agent components)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** ../../pipeline/queued/801-component-templates-from-files.spec.md
- **Source ticket:** Chad, 2026-10-17: "Maybe even research how we could make this pluggable to add outside components without recompile."
- **Status:** open

## Summary
A JSON file in `~/.config/marley/components/` defines a new component: its name, description,
the inputs it takes (a JSON Schema) and a body laid out from built-in components with
placeholders for those inputs. Marley loads the folder at start and again when a file changes.
A template shows in the Component Library and in `component_list`, and an agent uses it in
`component_show` like a built-in. No code runs and nothing is recompiled.

## Acceptance
A template file dropped into the folder appears in the library and draws in a thread without a
restart; a broken file shows its error in the library and is not offered to agents.
