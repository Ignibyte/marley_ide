# Prompts as the Dialogs board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-870-prompts-as-the-dialogs-board.md
- **Pipeline spec:** 870-prompts.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #869.
- **Classification / tier:** feature, M. One Zed hunk (`ui_prompt/src/ui_prompt.rs`), a Marley
  module.
- **Recall (§18.3):** the hook pattern (#834, #838, #839, #869); #840's three button weights. The
  brain: nothing on this seam.
- **Discovery (Explore):** the board's six dialogs are four kinds of surface. The close and quit
  guard (`close_guard.rs:136`, Warning, [Close|Quit, Show, Cancel]) and 17 other Marley
  questions are `window.prompt`s, as are Zed's save prompts (`pane.rs:2418`, `:2015`,
  `workspace.rs:4076`) and `confirm_quit`. Trust is Zed's `SecurityModal`; Rusty-not-connected is
  a toast (#869's look); the second launch shows nothing in the running Marley; settings and
  keymap changes, seats, `agent_start` and pairing are app notifications. So this ticket is the
  prompts; the modal and the asking notifications are TICKET-871.
- **Decisions:** D1–D4 in the spec.

### Design
- **Zed hunk (`ui_prompt.rs`):** `pub struct MarleyPromptLook(pub fn(MarleyPrompt<'_>, &mut
  Window, &mut App) -> Option<AnyElement>)` and `pub struct MarleyPrompt<'a> { level, message:
  &'a str, detail: Option<&'a str>, actions: &'a [SharedString], cancel: Option<usize>, active:
  usize, answer: Rc<dyn Fn(usize, &mut Window, &mut App)> }`. `ZedPromptRenderer` keeps the plain
  message and detail and which action is the `PromptButton::Cancel`; `render` asks the hook, and
  when it answers puts that element in the keyed, focused `v_flex` in place of Zed's card. The
  answer callback emits `PromptResponse` on the renderer. Ledger row.
- **Marley `prompts.rs`:** `init` sets the hook; `card`: none outside the Marley layout; a
  `v_flex` 420 px wide, rounded 16, `elevated_surface_background`, a border (the warning border
  for a Warning), the title (semibold), the body (muted, small), the buttons reversed with
  `role` (D2) mapped to styles: quiet `Subtle`, plain `Outlined`, primary `Tinted(Accent)`,
  danger `Tinted(Error)`; D4's border on the chosen one.
- **File manifest:** `crates/ui_prompt/src/ui_prompt.rs` (Zed), `ui_prompt` in
  `marley_workbench`'s `Cargo.toml`, `prompts.rs` (new), `marley_workbench.rs`; the scenario.
- **Visual check plan:** REQ-001/002 by the save prompt (Zed's, primary) and the close guard
  (Marley's, danger, with a detail); REQ-001 in Day. Not reached: the Critical level (no prompt
  in Marley or Zed's common paths raises one on demand).
- **Risks:** a Zed prompt whose message uses Markdown shows its marks (D3); the hook is asked at
  every render of an open prompt, cheap.

## Phase 2 — Code
- **Built:** in `ui_prompt.rs`, `MarleyPromptLook` and `MarleyPrompt`, three `marley_` fields on
  `ZedPromptRenderer` (the level, the plain message and detail, the cancel action's index from
  `PromptButton::is_cancel`), the hook asked at the top of `render` (before `ThemeSettings` holds
  `cx`), and Marley's card put in a frame with Zed's key context, focus and six actions. Zed's own
  card is still built and dropped when Marley answers, so Zed's chain stays untouched. Marley's
  `prompts.rs` (`card`, `role`, `render_button`, `plain`), `ui_prompt` in its `Cargo.toml`.
- **Deviations:**
  - D2's styles: `ButtonStyle` has no solid fill, so each button is a `Stateful<Div>` drawing
    its role: quiet on nothing, plain on `element_background` with a border, primary and danger
    solid in `text_accent` or `status().error` with the background colour as ink. The first run
    used `Subtle`/`Outlined`/`Tinted`, which drew Don't Save and Cancel alike and Save barely
    tinted (`870-01-save`, first run); the board's weights need the solid fill.
  - D3: Zed's save prompt writes the file name in backticks, which the first run showed; `plain`
    drops them.
  - "Not Now" and "Dismiss" are quiet too; the card's border is the warning border when the main
    button is a danger (the board's amber border on the close and quit dialogs).
- **Review:** the answer callback updates the renderer through a weak handle, after the click, so
  nothing updates it during its own render; the hook is asked with the renderer's fields borrowed
  apart from `cx`.
- **Gate:** red once on clippy (`needless_pass_by_value` on the hook's argument, now
  `&MarleyPrompt`; `clone_on_ref_ptr`; a redundant closure), fixed at source, then green.


## Phase 3 — Test
- **Scenario:** `script/e2e/870-prompts.sh` (sway): #840's stand-in in project `b`, waiting on a
  permission; README edited, `pane: close active item`; then the terminal, the same.
- `870-01-save`: Zed's save prompt as a card, "README.md contains unsaved edits. Do you want to
  save it?" (no backticks), Cancel quiet at the left, Don't Save filled with a border, Save solid
  orange at the right. REQ-001, REQ-002 met.
- `870-02-close`: the close guard's card, an amber border, "Close Claude Code in b? It is
  waiting." over "b · Claude Code · waiting", Cancel, Show, Close solid red. REQ-001, REQ-002 met.
- `870-03-day`: the same in Marley Day, a cream card, Close red with cream ink. REQ-001 met.
- The shots predate the gate's clippy fixes, which changed no drawing (a reference, `Rc::clone`,
  `Styled::bg`).
- Pre-existing — not in scope: gpui's line wrapper does not count "?" as part of a word, so a
  title that just fills the line can break before its "?" (`870-01-save`'s "save it / ?"); Zed's
  own labels share it.
- Not reached: the Critical level, and the arrow keys' focus border (D4), which needs a key the
  scenario's keyboard sends to the focused prompt; not shot.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (Prompts), look-plan W3.4 status, the ledger row.
- Knowledge: AD-claude-870-prompts-drawn-by-marley-in-zeds-frame-001. Brain: `brain decide` on
  consultation 0fbecdb5.
- Ticket closed; pipeline archived.
