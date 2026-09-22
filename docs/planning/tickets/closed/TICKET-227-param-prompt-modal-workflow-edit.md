# TICKET-227 — M12.2 — #204 follow-ups: interactive param-prompt modal + workflow edit/delete

- **Ticket:** LOCAL #227 (feature, M12.2)
- **Tags:** M12.2, cockpit, workflows, palette, follow-up
- **Created:** 2026-07-10
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id f48f91d8-0e02-499d-b289-ce08e53cfc95)
- **Status:** closed

## Description

Follow-ups deferred from #204 (command-palette workflows v1 — shipped: pure substitute/params_of + the Workflows settings round-trip + palette save/list/run; the v1 invoke INSERTS the command/template at the prompt for the user to fill+run). Three items:

(a) A dedicated interactive multi-{{param}} prompt MODAL — on invoking a workflow that HAS params, collect each {{param}} value in a modal (reuse the #204 `naming_workflow` inline-draft mechanism, one field per param), then `substitute(template, args)` → insert the filled command (or auto-run). v1 inserts the RAW template with {{tokens}} for the user to fill inline; this makes it a guided prompt. The pure `substitute` + `params_of` are done (#204) and directly drive this.

(b) A workflow EDIT / DELETE UI — v1 is save + run only (a workflow is removed by hand-editing the settings file). NOTE (from #204 R1): the v1 palette-command scheme APPENDS `CommandId(WORKFLOW_BASE + index)`, which assumes no id reuse; a DELETE must do a full palette-command REBUILD (regenerate the whole command list from `self.workflows`), NOT an in-place remove, or the id↔index mapping drifts. Also consider rename/reorder.

(c) OPTIONAL: a live-capture re-verify of the #204 palette save→invoke flow. #204's driven captures were ENV-BLOCKED (the macOS machine was LOCKED at validate — the lock screen blocked capture/synthetic-input); REQ-005/006 were carried by the settings round-trip unit + the critic-traced invoke-insert (byte-identical to the shipped, live-proven history-finder/completion idioms). A quick live capture (save `deploy=git push origin` → palette shows "Workflow: deploy" → invoke → inserts at the prompt) would close the visual loop once the machine is unlocked.

Deps: workflows.rs (#204 pure seam, DONE), app.rs palette shim, the #204 naming_workflow draft mechanism.
