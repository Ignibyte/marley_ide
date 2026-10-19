# TICKET-815 — A release build carries every file its embedded folders hold

- **Ticket:** LOCAL #815 (bug, the release install #502)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/815-a-release-build-carries-every-embedded-file.spec.md
- **Source ticket:** Chad, 2026-10-18: "the icons for home, marley, and rusty did not appear after
  reboot".
- **Status:** closed

## Summary
#807's three icons (the house, the cat, the butler) show in a debug Marley but not in the
installed release build. The installed binary names `marley_home`, `marley_cat` and
`rusty_butler` in `IconName` but embeds no `icons/marley_*.svg` or `icons/rusty_butler.svg`, so
gpui draws a blank where each one goes. The release `libassets` was last compiled on 2026-10-17 at
19:07, and the SVGs were written at 21:19. The installs at `15a4ade22d` and `ba5aacce78` reused
that crate.

`util::fs_embed!` is rust-embed in a release build. Its derive lists the folder when it expands,
and with the `compression` feature `include-flate` writes each file's bytes into the code without
an `include_bytes!`. Cargo learns of no file in the folder, so neither a new file nor an edit to
one rebuilds the crate. A debug build reads the checkout at runtime, so #807's check saw the
icons. The same gap holds for every `fs_embed!` in Marley's binary: `assets` (icons, themes,
fonts), `settings` (the default settings and keymaps), `marley_workbench` (the docs bundle and
the changelog), `agent` (prompt templates) and `grammars`. An edit there reaches an install only
when its crate compiles for another reason.

## Acceptance
A release build made after a file under an embedded folder is added or changed carries the new
file, and the installed Marley draws Home's, Marley's and Rusty's icons. A debug build does not
recompile when an asset changes.
