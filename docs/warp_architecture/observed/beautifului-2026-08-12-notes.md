# Observed: beautifului.dev — Sidebar Nav + Task Rows (2026-08-12)

Chad-picked visual reference for the M31 simple-rail redo ("I really like the components
here"). Site: <https://www.beautifului.dev/> — a visual gallery of AI-native UI primitives
(mockups, not a code kit). Captured live via Playwright, CSS-px scale.

## beautifului-sidebar-nav-2026-08-12.png (`#sidebar-nav`)

The target grammar for the new rail:

- Workspace header: avatar chip ("C") + name ("Creamery Ops") + muted subtitle
  ("Production Workspace") + up/down chevron — a switcher, top of the column.
- Quick-search field directly under it, with a `/` keycap hint at the right edge.
- Accent action row: "New task" in accent blue + a filled accent ＋ circle at the right.
- Section headers: small-caps, tracked, muted ("WORKSPACE", "OBJECTS") — labels only,
  no chevrons, generous top spacing.
- Rows: muted icon + label, ONE flat indent level, quiet (no fills at rest).
- Selection: exactly ONE row lit — subtle rounded-rect fill, slightly lighter than the
  column bg, with a right-aligned count badge ("Agent tasks · 4"). Nothing else lights.

## beautifului-task-rows-2026-08-12.png (`#task-rows`)

The left-edge circle indicator grammar (what replaces Marley's ancestor lighting):

- Every row leads with a status circle at the far left: filled green + white check
  (completed), or a numbered ring with a progress arc (in flight).
- Rows are full-width capsules with big radius, muted meta text right-aligned
  ("12 suppliers"), status chip ("Completed"), trailing disclosure chevron.

## What Marley takes from this (design intent, settled at #418)

One selected row ever; small left dot/circle as the "open elsewhere / live" indicator;
small-caps quiet section headers; flat hierarchy feel; add-verb as an accent ＋. Palette,
type, and geometry stay Marley's own (type_scale Nav/Caption, accent washes, 6px radius) —
this reference is grammar, not skin.
