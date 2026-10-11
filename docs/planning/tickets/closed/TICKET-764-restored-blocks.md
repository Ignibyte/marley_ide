# TICKET-764 — Restored blocks

- **Ticket:** LOCAL #764 (feature, the finish line)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [764-restored-blocks.spec.md](../../pipeline/completed/764-restored-blocks.spec.md)
- **Source ticket:** split from #751 (restored scrollback, slice 2) on 2026-10-11, when Chad asked to "spec all" of the survey list. Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md).
- **Status:** closed

## Summary
Each finished block's command, exit code, folder, host and times saved beside #751's scrollback log with the offsets where its prompt and output began and ended, and redrawn on the restored rows as pills and headers marked restored, with no Rerun.

## Acceptance
After a restart a terminal's earlier commands show as restored blocks over their dimmed output.
