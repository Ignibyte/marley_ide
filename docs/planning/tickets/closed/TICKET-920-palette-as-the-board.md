# TICKET-920 — The command palette as the board draws it

- **Ticket:** LOCAL #920 (feature, the Marley look, the command palette)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [920-palette-as-the-board.spec.md](../../pipeline/completed/920-palette-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/C-threads.md, boards Palette), build order line 8
- **Boards:** Palette (its command palette column)
- **Size:** M
- **Status:** closed

## Summary

In the Marley layout the command palette shows each command by a plain name ("Open Browser",
not "marley: open browser"), with a small glyph tile before it. The field carries a "Commands"
chip at its right, and the footer reads "Enter to run · Ctrl Enter to bind a key · ? for help".
Typing "?" lists the help commands. Typing still matches Zed's names, so every scenario, alias and
the palette's history keep working. #839 built the groups on an empty query; this ticket draws the
rows, the field and the footer as the board does.

## Acceptance

In the Marley layout, the palette's rows show a tile and a plain name, the field shows the
Commands chip, the footer reads as the board, and "?" lists help; in Zed's layout the palette is
Zed's. Shot in Marley Night and Marley Day.
