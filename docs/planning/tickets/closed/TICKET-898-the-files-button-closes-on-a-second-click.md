# TICKET-898 — The rail's Files button closes the tree on a second click

- **Ticket:** LOCAL #898 (bug, from #868)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [898-files-button-toggles.spec.md](../../pipeline/completed/898-files-button-toggles.spec.md)
- **Source ticket:** Chad, 2026-10-22: "the files open tab you made doesn't close when you reclick
  it".
- **Size:** S
- **Status:** closed

## Summary

The rail header's Files button runs Zed's `project_panel::ToggleFocus`, which closes the tree
only while the tree has the focus. Once the focus is anywhere else (the editor, the rail), a
second click focuses the tree again instead of putting it away.

## Acceptance

A click on Files while the tree shows closes it, wherever the focus is.
