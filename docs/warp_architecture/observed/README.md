# Observed Warp / Zed behavior (§20)

Durable captures — screenshots + behavior notes — of the **reference apps**, cited by pipeline specs'
`## Reference (§20)` sections (enforced by `.claude/hooks/enforce-warp-reference.sh`).

- **Warp** — the terminal / cockpit / command-Blocks / UX reference.
- **Zed** — the editor reference (a same-`gpui`-stack reference; used once the editor-experience train begins).

**Clean-room (§20):** these record **observed BEHAVIOR only** — how the reference app looks and responds. Never
copy or transcribe Warp/Zed **source** (Warp is AGPL, Zed is GPL — a reworded translation is still a derivative
work). Marley reimplements the behavior from scratch and reuses only the permissive `gpui` / `alacritty_terminal`
crates.

Unlike the ephemeral session scratchpad, files here are **committed** — so a spec's reference survives and can be
reviewed at inspect (the §18.1 provenance check).

## Convention
- Name captures for the ticket + subject, e.g. `248-example.png`, `250-warp-input-caret.png`,
  `M16-zed-multicursor.png`.
- Pair a capture with a short note (what behavior it demonstrates) when the pixels alone aren't self-evident.

## Captures

### `250-warp-monospace-grid-caret.png` — the monospace cell grid + block caret (ticket #250)
Warp renders all text on a **fixed-width monospace cell grid**: every `char` occupies exactly one cell, so a
character's offset in a line maps **1:1** to its display column (indentation, tree-drawing glyphs `└`, and bullets
`●` all land on identical columns down the view). Tabs advance to the next tab-stop (a multiple of the tab width),
not a flat run of spaces. The input **caret is a solid block occupying one whole cell** at the prompt (`>`),
sitting exactly on a column boundary.

**Why #250 matches this:** the faithful editor must render from the real text (the Buffer) on this same cell
grid, so on-screen columns equal char offsets — the prerequisite for the caret (#251) to land precisely and for
save (#252) not to corrupt. #250 reproduces the offset↔column contract as a pure, tested mapping fn (tab-stop
aware, one char = one column in v1) — the single source of truth the render uses now and the caret/#251 +
mouse/#254 consume next. Captured non-invasively from a running Warp window (no interaction); clean-room —
observed rendering behavior only, no Warp source read. (The visible content is this Marley work session; the
grid + block caret are the reference, not the text.)
