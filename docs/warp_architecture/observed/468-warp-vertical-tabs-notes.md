# Observed: Warp's vertical tab list (2026-09-23)

Chad's reference for the rail restyle (#468): "The left pane lets make it look like the warp.dev
one. It basically puts more padding around the project + terminal to make is a bit better. makes
the icons bigger etc." Observed from a screenshot of Warp's left pane at 1:1 on the dev box. The
screenshot itself is not committed: it shows Chad's sessions, host and branches. These notes keep
only the layout.

## What the pane shows

- **Search row** at the top, about 38px high: a magnifier, "Search tabs…", and a filter icon and
  a `+` at the right edge. A 1px line under it.
- **Sections**, one per tab, each opened by a small muted label (about 11px: "Manager",
  "Rustal") at the pane's left padding, about 8px in. A 1px line runs the pane's full width
  between sections, about 87px apart for a one-row section.
- **Rows**, one per pane in the tab, about 46px high:
  - a round icon container, about 28px across, filled a shade lighter than the pane, with a
    14px glyph in it (`>_` for a shell; an agent's own mark, in its color, for an agent);
  - about 10px to the text: a title of about 13px over a muted second line of about 10.5px
    (`~`, a path, or a branch with its icon);
  - a muted `Ctrl <n>` at the right edge, level with the title.
  - The icon sits about 11px further in than the section label.
- **The selected row** is a card: rounded corners (about 5px), a fill a step lighter than the
  pane, and a 1px border a step lighter again, inset about 22px from the pane's left edge and
  14px from its right.
- **An explicit tab group** ("New Group") draws a chevron, a 14px name and a muted count ("1
  tab"), over a band a shade lighter than the pane that holds its rows.
- Unselected rows draw no fill and no border.

## What Marley takes (#468)

The padding, the 28px round icon, the two-line title, the selected card and the lines between
groups. A project's header plays the section label, muted, with Marley's own chevron and `+`.
Marley has no binding for the n-th row, so the `Ctrl <n>` hints stay out; tab groups have no
Marley counterpart, so neither does their band.
