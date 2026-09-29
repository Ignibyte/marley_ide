# TICKET-558 — Save as Workflow: a block's command becomes a task in `tasks.json`

- **Ticket:** LOCAL #558 (feature, prong 1 T4 (tasks) with T1 (block actions); the Warp blocks note, recommendation 5)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/558-save-as-workflow.spec.md
- **Source ticket:** docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md ("Save as Workflow"; Chad's answer 2, 2026-09-26: "if it works tasks.json it would make sense not to conflict")
- **Status:** closed

## Summary
Save as Workflow on a block opens an editor with the block's command, a name, and the
parameters the rules guessed (numbers, URLs, the branch, paths that exist), each written
`{{name}}` in the command with the original token as its default. Saving appends a task to the
project's `.zed/tasks.json` (or the global `tasks.json`), comments kept, with the parameters
under a `marley` key Zed ignores. The task shows in Zed's own task picker at once; choosing it
asks for the parameters with their defaults, fills the command and runs it as a Zed task in the
center. No Marley YAML, no second store: a workflow is a task.

## Acceptance
Save as Workflow from a hovered block shows the editor with the guessed parameters; Save
appends the task to `.zed/tasks.json` without touching what was there; the task shows in Zed's
task picker; running it asks for the parameters and runs the filled command as a task in the
center; a task without parameters runs at once. The full EARS criteria live in the pipeline
spec.
