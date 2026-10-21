# Pickers — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-839-pickers.md
- **Pipeline spec:** 839-pickers.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #868; Chad's "lets continue on".
- **Classification / tier:** feature, M. Marley crate (`agents.rs`, a hook module) and two small
  Zed hunks (`command_palette.rs`, `picker.rs`).
- **Recall (§18.3):** L-claude-836 (full-width buttons center), PR-claude-747; the Zed hooks of
  #834 and #838 as the pattern. The brain (consultation 250006bb): nothing on this seam.
- **Discovery:** `CommandPaletteDelegate` (command_palette.rs:196), `Command { name, action, usage }`
  (:212), `matches_updated` (:340), `render_match` (:764), the picker made at :147
  (`Picker::uniform_list`); `Picker::new`'s preview layout load (picker.rs:596,
  `persistence::load_last_preview_layout(D::name())`, `Layout::Hidden` by default); the file
  finder's name "file finder"; `NewAgentDelegate::render_match` (agents.rs:1545), `Choice` (:1173),
  `Start::place` (:1206); the Palette board's data (groups, agent lines).

### Design
- **Zed hunk, command_palette.rs:** `pub struct MarleyPaletteGroups(pub fn(&App) -> bool)`, a
  `Global`. When it answers true: the palette is a `Picker::list` (rows of two heights), and with an
  empty query `matches_updated` stable-sorts the matches by group (0 used before, 1 a `marley:`
  command, 2 the rest); `render_match` puts the group's heading (RECENT, MARLEY, EDITOR AND
  WORKSPACE) over a row whose group differs from the row before. Ledger row.
- **Zed hunk, picker.rs:** `pub struct MarleyPreviewDefault(pub fn(&'static str, &App) ->
  Option<preview::Layout>)`; `Picker::new` asks it for the delegate's name when nothing is stored.
  Ledger row.
- **Marley, `pickers.rs` (new):** `init` sets both: groups in the Marley layout; `Right` for
  "file finder" in the Marley layout.
- **Marley, `agents.rs`:** `Choice.about: SharedString` (a line per agent: Marley, Rusty, Claude
  Agent, Zed's agent, a CLI "The CLI, in a block terminal", one not found "Not on Marley's PATH
  yet" where the list keeps it); `render_match` draws the choice as a two-line card: its icon on a
  tinted tile, name over the line, the place at the end.
- **File manifest:** `pickers.rs` (new), `marley_workbench.rs` (mod, init), `agents.rs`,
  `crates/command_palette/src/command_palette.rs`, `crates/picker/src/picker.rs` (Zed, ledger
  rows), the scenario.

### Visual check plan
As the spec's UI proof: five shots.

### Risks
- `Picker::list` over a thousand commands: gpui's list measures rows lazily; check the palette
  opens without a lag in the probe.

## Phase 2 — Code
- **Built:** the two Zed hooks with ledger rows (`MarleyPaletteGroups`: the list kind, the sort,
  the headings, the delegate's `ListItem` an `AnyElement`; `MarleyPreviewDefault` as a fallback
  after the stored layout), `pickers.rs`, `Start::about` and `render_choice`; the
  `command_palette` dependency.
- **Deviation:** `render_match` of the New Agent picker went over 100 lines, so the card is
  `render_choice`. Only installed CLIs are listed, so the board's "Not on Marley's PATH yet" line
  has no row to go on.
- **Review:** the hooks copy their function pointers before calling; the sort is stable, so each
  group keeps Zed's order (used ones by recency); headings only with an empty query.
- **Gate:** `just gate-diff` green (17 gates and the receipt).


## Phase 3 — Test
- Scenario `script/e2e/839-pickers.sh` (sway), the gated build. The probe's shot 02 caught "op"
  half typed with the headings still up (the matches come asynchronously); its settle grew to 5 s.
- `839-01-palette`: RECENT (marley: open threads first, then the profile's earlier ones), MARLEY
  (accept suggestion, ask agent, …) (REQ-001).
- `839-02-typed`: "open": marley: open threads, open browser, zed: open log…, no headings (REQ-001).
- `839-03-agents`: cards for Zed Agent, Claude Agent (Thread), Claude Code, Codex, Gemini CLI,
  OpenCode (Terminal), each icon on a tile with its line (REQ-002).
- `839-04-files`: "lib": lib.rs selected, its text in the preview at the right, the footer's
  Preview (REQ-003).
- `839-05-day`: Marley Day, RECENT with marley: new agent first (REQ-001).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added), the guide's "The pickers" and the New Agent picker passage, the
  architecture note's "The pickers", the look plan's W3.3; two ledger rows.
- **Knowledge:** L-claude-839-a-uniform-list-cannot-take-a-heading-row-001.
- **Brain:** decision on consultation 250006bb.
- Ticket closed, pipeline archived.
