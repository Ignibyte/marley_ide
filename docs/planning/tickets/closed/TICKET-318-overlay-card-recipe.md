# TICKET-318 — Extract the shared overlay-card recipe (hover / def-picker / palette / finder / history)

- **Ticket:** LOCAL #318 (chore, M20)
- **Tags:** M20, refactor, gpui, overlay, from-inspect, 312-followup
- **Created:** 2026-07-15
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id aad4a0ae-133d-41ee-8887-818da992a7b6)
- **Status:** closed (2026-08-12, pipeline 318 complete — gate green, receipt written; after-capture battery = TICKET-417)

## Description

Born from the #312 inspect (finding F17, correctly scoped OUT of #312).

PROBLEM: the overlay-card builder chain is now copy-pasted five times. `def_picker_overlay` (#312) is 15-of-16 builder calls byte-identical to `hover_card_overlay` (#311, shipped one commit earlier): `.absolute().left(px).top(px).w(px).max_h(px).occlude().flex().flex_col().bg(colors.surface).rounded(colors.corner_radius).overflow_hidden().border_1().border_color(colors.border).font_family(TERMINAL_FONT).text_size(px(TERMINAL_FONT_SIZE))`. The 8-call style core also appears verbatim in the palette, finder, and history overlays. No shared card helper exists — `menu_origin` (context_menu.rs) is positioning only, and #312 already reuses it correctly.

WHY IT WAS NOT DONE IN #312: extracting for `def_picker` alone yields a one-caller helper (a net loss). Capturing hover + def_picker only gets 2 of 5. The real win — `fn overlay_card(colors: &ThemeColors) -> gpui::Div` returning the shared style calls, each caller adding its own position/size — must also touch palette/finder/history, which use hand-rolled `bounds.w * 0.25` positioning rather than `menu_origin` and would need converting. That is its own blast radius and its own driven capture.

SCOPE:
- Extract the shared style core into one helper next to `menu_origin`.
- Convert palette/finder/history off hand-rolled positioning onto `menu_origin`'s clamp (so all five overlays clamp identically to the window).
- Keep each caller's own w/max_h/origin.

WATCH OUT:
- `BF-lsp-hover-extracted-helper-new-mutation-surface-001`: extracting an inline expression into a NAMED fn creates a NEW standalone cargo-mutants target — pair the extract with a direct unit test, or the MSI floor of 100 goes RED. Run `cargo mutants --list -f <file>` for the REAL set rather than guessing.
- A driven capture must show all five overlays unchanged pixel-wise (this is a pure refactor; any visual delta is a bug).
- Related follow-up already noted in #312's inspect ledger: the shipped palette/finder CLAMP their selection while #312's DefPicker WRAPS. If the pickers are being unified anyway, decide one behaviour deliberately rather than inheriting the split.
