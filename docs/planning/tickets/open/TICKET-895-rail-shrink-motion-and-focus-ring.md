# TICKET-895 — The rail's shrink in motion, and one focus ring everywhere

- **Ticket:** LOCAL #895 (feature, the Marley look, W6)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** from TICKET-858 (2026-10-22): #858 made the needs-you pulse.
- **Boards:** States
- **Size:** S–M
- **Status:** open

## Summary

The rail shrinking to its strip in 180 ms (its labels fading by 60 ms, its width easing), and one
focus ring everywhere: 2 px of the coat's orange just outside the focused control's shape. Both
honour reduced motion.

## Acceptance

Each looks as the board shows; shot in Marley Night and Marley Day.

## Notes (2026-10-22)

- The rail's fold already eases its width over 180 ms and skips the ease under reduced motion
  (#830: `rail_strip.rs`'s `EASE` and `follow_strip_setting`). A burst of shots 60 ms apart
  (`script/e2e/895-rail-fold-and-focus.sh`) caught only the end: a shot takes longer than the
  fold. The labels do not fade; the strip replaces them at the fold's start.
- The focus ring is not one ring everywhere: the rail's focused tile has the coat's border, while
  the Settings window's focused search field shows almost none. One ring on every focused control
  is a change across Zed's inputs (an outline outside the shape, which gpui draws as a border
  inside it today): a question for Chad before it is planned.
